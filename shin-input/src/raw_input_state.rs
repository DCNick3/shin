use enum_map::{enum_map, EnumMap};
use gilrs::{EventType, GamepadId, Gilrs};
use glam::{vec2, Vec2};
use indexmap::IndexMap;
use petitset::PetitSet;
use tracing::{error, warn};
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::inputs::{GamepadAxis, GamepadButton, MouseButton, VirtualGamepadButton};

#[derive(Clone)]
pub struct MouseState {
    /// Mouse buttons state, simple state of each button
    pub buttons: EnumMap<MouseButton, bool>,
    pub position: Vec2,
    pub scroll_amount: f32,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            buttons: enum_map! { _ => false },
            position: vec2(0.0, 0.0),
            scroll_amount: 0.0,
        }
    }
}

#[derive(Clone, Default)]
pub struct UnifiedGamepadState {
    pub buttons: EnumMap<GamepadButton, bool>,
    pub axes: EnumMap<GamepadAxis, f32>,
    pub virtual_keys: EnumMap<VirtualGamepadButton, bool>,
}

impl UnifiedGamepadState {
    pub fn new() -> Self {
        Self {
            buttons: enum_map! { _ => false },
            axes: enum_map! { _ => 0.0 },
            virtual_keys: enum_map! { _ => false },
        }
    }

    fn update_virtual_keys(&mut self) {
        self.virtual_keys = self.virtual_keys.map(|key, prev_state| {
            let (axis, thresholds) = match key {
                VirtualGamepadButton::StickLUp => (GamepadAxis::LeftStickY, (0.4, 0.7)),
                VirtualGamepadButton::StickLDown => (GamepadAxis::LeftStickY, (-0.4, -0.7)),
                VirtualGamepadButton::StickLLeft => (GamepadAxis::LeftStickX, (-0.4, -0.7)),
                VirtualGamepadButton::StickLRight => (GamepadAxis::LeftStickX, (0.4, 0.7)),
                VirtualGamepadButton::StickRUp => (GamepadAxis::RightStickY, (0.4, 0.7)),
                VirtualGamepadButton::StickRDown => (GamepadAxis::RightStickY, (-0.4, -0.7)),
                VirtualGamepadButton::StickRLeft => (GamepadAxis::RightStickX, (-0.4, -0.7)),
                VirtualGamepadButton::StickRRight => (GamepadAxis::RightStickX, (0.4, 0.7)),
            };

            let threshold = if prev_state {
                thresholds.0
            } else {
                thresholds.1
            };
            let matches_threshold = if threshold > 0.0 {
                self.axes[axis] >= threshold
            } else {
                self.axes[axis] <= threshold
            };

            matches_threshold
        });
    }
}

#[derive(Clone)]
pub struct GamepadsState {
    pub gamepads: IndexMap<GamepadId, UnifiedGamepadState>,
    pub unified: UnifiedGamepadState,
}

impl GamepadsState {
    pub fn new() -> Self {
        Self {
            gamepads: IndexMap::new(),
            unified: UnifiedGamepadState::new(),
        }
    }

    pub fn is_held(&self, button: GamepadButton) -> bool {
        self.unified.buttons[button]
    }

    pub fn is_vheld(&self, axis: VirtualGamepadButton) -> bool {
        self.unified.virtual_keys[axis]
    }
}

pub struct RawInputAccumulator {
    gilrs: Option<Gilrs>,
    state: RawInputState,
}

impl RawInputAccumulator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            gilrs: match Gilrs::new() {
                Ok(gilrs) => Some(gilrs),
                Err(gilrs::Error::NotImplemented(gilrs)) => {
                    warn!("Gamepad input is not implemented on this platform");
                    Some(gilrs)
                }
                Err(err) => {
                    error!("Failed to initialize gamepad input: {:?}", err);
                    None
                }
            },
            state: RawInputState::new(),
        }
    }

    pub fn on_winit_event(&mut self, event: &WindowEvent) {
        let state = &mut self.state;

        match event {
            WindowEvent::KeyboardInput {
                event,
                // On some platforms, winit sends "synthetic" key press events when the window
                // gains or loses focus. These are not implemented on every platform, so we ignore
                // winit's synthetic key pressed and just reset keyboard state on unfocus.
                is_synthetic: false,
                ..
            } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            state.keyboard.insert(keycode);
                        }
                        ElementState::Released => {
                            state.keyboard.remove(&keycode);
                        }
                    }
                }
            }
            WindowEvent::Focused(false) => {
                state.keyboard.clear();
            }
            WindowEvent::CursorMoved { position, .. } => {
                state.mouse.position = vec2(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // press virtual mouse buttons
                // TODO: handle it in a smarter way or smth...
                let amount = match delta {
                    &winit::event::MouseScrollDelta::LineDelta(_x, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(p) => (p.y / 120.0) as f32, /* this value is windows-specific */
                };

                if amount > 0.0 {
                    state.mouse.buttons[MouseButton::WheelUp] = true;
                } else {
                    state.mouse.buttons[MouseButton::WheelDown] = true;
                }
                state.mouse.scroll_amount = amount;
            }
            &WindowEvent::MouseInput { button, state, .. } => {
                if let Some(button) = convert_winit_mouse_button(button) {
                    self.state.mouse.buttons[button] = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    }
                }
            }
            _ => {
                // don't care about other events
            }
        }
    }

    pub fn start_frame(&mut self) -> RawInputState {
        if let Some(gilrs) = &mut self.gilrs {
            let gamepads = &mut self.state.gamepads;
            while let Some(event) = gilrs.next_event() {
                let gamepad = gamepads.gamepads.entry(event.id).or_default();
                match event.event {
                    EventType::ButtonPressed(button, _) => {
                        if let Some(button) = GamepadButton::from_gilrs(button) {
                            gamepad.buttons[button] = true;
                        }
                    }
                    EventType::ButtonRepeated(_button, _) => {
                        // we do not handle repeated events, our input system has its own way of synthesizing these
                    }
                    EventType::ButtonReleased(button, _) => {
                        if let Some(button) = GamepadButton::from_gilrs(button) {
                            gamepad.buttons[button] = false;
                        }
                    }
                    EventType::ButtonChanged(_button, _value, _) => {
                        // we do not support analog buttons
                    }
                    EventType::AxisChanged(axis, value, _) => {
                        if let Some(axis) = GamepadAxis::from_gilrs(axis) {
                            gamepad.axes[axis] = value;
                        }
                    }
                    EventType::Connected | EventType::Disconnected => {
                        // reset the state just in case
                        *gamepad = UnifiedGamepadState::new();
                    }
                    EventType::Dropped | EventType::ForceFeedbackEffectCompleted => {
                        // nothing to do here
                    }
                    _ => {
                        // ignore events added in the future
                    }
                }
            }

            for (_, gamepad) in &mut gamepads.gamepads {
                gamepad.update_virtual_keys();
            }

            // fuse all gamepads into a single state
            gamepads.unified =
                gamepads
                    .gamepads
                    .values()
                    .fold(UnifiedGamepadState::new(), |acc, gamepad| {
                        UnifiedGamepadState {
                            buttons: acc
                                .buttons
                                .map(|button, value| gamepad.buttons[button] | value),
                            axes: acc.axes.map(|axis, value| gamepad.axes[axis].max(value)),
                            virtual_keys: acc
                                .virtual_keys
                                .map(|key, value| gamepad.virtual_keys[key] | value),
                        }
                    });
        }

        self.state.clone()
    }

    pub fn finish_frame(&mut self) {
        // NOTE: this should be done __after__ everything has handled the events
        self.state.mouse.scroll_amount = 0.0;
        self.state.mouse.buttons[MouseButton::WheelUp] = false;
        self.state.mouse.buttons[MouseButton::WheelDown] = false;
    }
}

#[derive(Clone)]
pub struct RawInputState {
    /// Keyboard state, set of pressed keys
    pub keyboard: PetitSet<KeyCode, 16>,
    pub mouse: MouseState,
    pub gamepads: GamepadsState,
    // TODO: touchscreen
    // need to figure out how exactly the game handles it first though
}

impl RawInputState {
    pub fn new() -> Self {
        Self {
            keyboard: PetitSet::new(),
            mouse: MouseState::new(),
            gamepads: GamepadsState::new(),
        }
    }
}

// impl Display for RawInputState {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!(f, "RawInputState: {{")?;
//         writeln!(
//             f,
//             "  keyboard: [{}]",
//             self.keyboard.iter().map(|v| format!("{:?}", v)).join(", ")
//         )?;
//         writeln!(
//             f,
//             "  mouse.buttons: [{}]",
//             self.mouse
//                 .buttons
//                 .iter()
//                 .filter_map(|(but, state)| state.then(|| format!("{:?}", but)))
//                 .join(", ")
//         )?;
//         writeln!(f, "}}")?;
//         Ok(())
//     }
// }

// impl OverlayVisitable for RawInputState {
//     fn visit_overlay(&self, collector: &mut crate::render::overlay::OverlayCollector) {
//         collector.overlay(
//             "Input State",
//             |_ctx, top_left| {
//                 top_left.label(format!(
//                     "Input State: [{}] [{}]",
//                     self.mouse_buttons
//                         .iter()
//                         .filter_map(|(but, state)| state.then(|| format!("{:?}", but)))
//                         .join(", "),
//                     self.keyboard.iter().map(|v| format!("{:?}", v)).join(", ")
//                 ));
//             },
//             true,
//         );
//     }
// }

#[inline]
fn convert_winit_mouse_button(winit: winit::event::MouseButton) -> Option<MouseButton> {
    match winit {
        winit::event::MouseButton::Left => Some(MouseButton::Left),
        winit::event::MouseButton::Right => Some(MouseButton::Right),
        winit::event::MouseButton::Middle => Some(MouseButton::Middle),
        // TODO: how should we model those?
        winit::event::MouseButton::Back | winit::event::MouseButton::Forward => None,
        winit::event::MouseButton::Other(_) => None,
    }
}
