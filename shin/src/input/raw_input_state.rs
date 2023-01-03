use crate::input::action::UserInput;
use crate::input::inputs::{KeyCode, MouseButton};
use enum_map::{enum_map, EnumMap};
use itertools::Itertools;
use petitset::PetitSet;
use std::fmt::Display;
use winit::event::{ElementState, WindowEvent};

pub struct RawInputState {
    /// Keyboard state, set of pressed keys
    keyboard: PetitSet<KeyCode, 16>,
    /// Mouse buttons state, simple state of each button
    mouse_buttons: EnumMap<MouseButton, bool>,
    mouse_scroll_amount: f32,
    // TODO: dummy gamepad state for now
    gamepad: (),
    // TODO: mouse position?
    // How do we even handle mouse position?
}

impl RawInputState {
    pub fn new() -> Self {
        Self {
            keyboard: PetitSet::new(),
            mouse_buttons: enum_map! { _ => false },
            mouse_scroll_amount: 0.0,
            gamepad: (),
        }
    }

    /// Returns the current state of the given button, and optionally the value (useful for axis)
    pub fn is_pressed(&self, input: &UserInput) -> Option<f32> {
        match input {
            UserInput::Keyboard(key_code) => self.keyboard.contains(key_code).then_some(1.0),
            UserInput::MouseButton(button) => self.mouse_buttons[*button].then_some(1.0),
            UserInput::GamepadButton(_) => None,
        }
    }

    // TODO: handle the sticks better?

    pub fn on_winit_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    let keycode = convert_winit_keycode(keycode);
                    match input.state {
                        ElementState::Pressed => {
                            self.keyboard.insert(keycode);
                        }
                        ElementState::Released => {
                            self.keyboard.remove(&keycode);
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // press virtual mouse buttons
                // TODO: handle it in a smarter way or smth...
                let amount = match delta {
                    &winit::event::MouseScrollDelta::LineDelta(_x, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(p) => (p.y / 120.0) as f32, /* this value is windows-specific */
                };

                if amount > 0.0 {
                    self.mouse_buttons[MouseButton::WheelUp] = true;
                } else {
                    self.mouse_buttons[MouseButton::WheelDown] = true;
                }
                self.mouse_scroll_amount = amount;
            }
            &WindowEvent::MouseInput { button, state, .. } => {
                if let Some(button) = convert_winit_mouse_button(button) {
                    self.mouse_buttons[button] = match state {
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

    pub fn update(&mut self) {
        // NOTE: this should be done __after__ everything has handled the events
        self.mouse_scroll_amount = 0.0;
        self.mouse_buttons[MouseButton::WheelUp] = false;
        self.mouse_buttons[MouseButton::WheelDown] = false;
    }
}

impl Display for RawInputState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RawInputState: {{")?;
        writeln!(
            f,
            "  keyboard: [{}]",
            self.keyboard.iter().map(|v| format!("{:?}", v)).join(", ")
        )?;
        writeln!(
            f,
            "  mouse_buttons: [{}]",
            self.mouse_buttons
                .iter()
                .filter_map(|(but, state)| state.then(|| format!("{:?}", but)))
                .join(", ")
        )?;
        writeln!(f, "}}")?;
        Ok(())
    }
}

#[inline]
fn convert_winit_keycode(winit: winit::event::VirtualKeyCode) -> KeyCode {
    use winit::event::VirtualKeyCode;
    match winit {
        VirtualKeyCode::Key1 => KeyCode::Key1,
        VirtualKeyCode::Key2 => KeyCode::Key2,
        VirtualKeyCode::Key3 => KeyCode::Key3,
        VirtualKeyCode::Key4 => KeyCode::Key4,
        VirtualKeyCode::Key5 => KeyCode::Key5,
        VirtualKeyCode::Key6 => KeyCode::Key6,
        VirtualKeyCode::Key7 => KeyCode::Key7,
        VirtualKeyCode::Key8 => KeyCode::Key8,
        VirtualKeyCode::Key9 => KeyCode::Key9,
        VirtualKeyCode::Key0 => KeyCode::Key0,
        VirtualKeyCode::A => KeyCode::A,
        VirtualKeyCode::B => KeyCode::B,
        VirtualKeyCode::C => KeyCode::C,
        VirtualKeyCode::D => KeyCode::D,
        VirtualKeyCode::E => KeyCode::E,
        VirtualKeyCode::F => KeyCode::F,
        VirtualKeyCode::G => KeyCode::G,
        VirtualKeyCode::H => KeyCode::H,
        VirtualKeyCode::I => KeyCode::I,
        VirtualKeyCode::J => KeyCode::J,
        VirtualKeyCode::K => KeyCode::K,
        VirtualKeyCode::L => KeyCode::L,
        VirtualKeyCode::M => KeyCode::M,
        VirtualKeyCode::N => KeyCode::N,
        VirtualKeyCode::O => KeyCode::O,
        VirtualKeyCode::P => KeyCode::P,
        VirtualKeyCode::Q => KeyCode::Q,
        VirtualKeyCode::R => KeyCode::R,
        VirtualKeyCode::S => KeyCode::S,
        VirtualKeyCode::T => KeyCode::T,
        VirtualKeyCode::U => KeyCode::U,
        VirtualKeyCode::V => KeyCode::V,
        VirtualKeyCode::W => KeyCode::W,
        VirtualKeyCode::X => KeyCode::X,
        VirtualKeyCode::Y => KeyCode::Y,
        VirtualKeyCode::Z => KeyCode::Z,
        VirtualKeyCode::Escape => KeyCode::Escape,
        VirtualKeyCode::F1 => KeyCode::F1,
        VirtualKeyCode::F2 => KeyCode::F2,
        VirtualKeyCode::F3 => KeyCode::F3,
        VirtualKeyCode::F4 => KeyCode::F4,
        VirtualKeyCode::F5 => KeyCode::F5,
        VirtualKeyCode::F6 => KeyCode::F6,
        VirtualKeyCode::F7 => KeyCode::F7,
        VirtualKeyCode::F8 => KeyCode::F8,
        VirtualKeyCode::F9 => KeyCode::F9,
        VirtualKeyCode::F10 => KeyCode::F10,
        VirtualKeyCode::F11 => KeyCode::F11,
        VirtualKeyCode::F12 => KeyCode::F12,
        VirtualKeyCode::F13 => KeyCode::F13,
        VirtualKeyCode::F14 => KeyCode::F14,
        VirtualKeyCode::F15 => KeyCode::F15,
        VirtualKeyCode::F16 => KeyCode::F16,
        VirtualKeyCode::F17 => KeyCode::F17,
        VirtualKeyCode::F18 => KeyCode::F18,
        VirtualKeyCode::F19 => KeyCode::F19,
        VirtualKeyCode::F20 => KeyCode::F20,
        VirtualKeyCode::F21 => KeyCode::F21,
        VirtualKeyCode::F22 => KeyCode::F22,
        VirtualKeyCode::F23 => KeyCode::F23,
        VirtualKeyCode::F24 => KeyCode::F24,
        VirtualKeyCode::Snapshot => KeyCode::Snapshot,
        VirtualKeyCode::Scroll => KeyCode::Scroll,
        VirtualKeyCode::Pause => KeyCode::Pause,
        VirtualKeyCode::Insert => KeyCode::Insert,
        VirtualKeyCode::Home => KeyCode::Home,
        VirtualKeyCode::Delete => KeyCode::Delete,
        VirtualKeyCode::End => KeyCode::End,
        VirtualKeyCode::PageDown => KeyCode::PageDown,
        VirtualKeyCode::PageUp => KeyCode::PageUp,
        VirtualKeyCode::Left => KeyCode::Left,
        VirtualKeyCode::Up => KeyCode::Up,
        VirtualKeyCode::Right => KeyCode::Right,
        VirtualKeyCode::Down => KeyCode::Down,
        VirtualKeyCode::Back => KeyCode::Backspace,
        VirtualKeyCode::Return => KeyCode::Enter,
        VirtualKeyCode::Space => KeyCode::Space,
        VirtualKeyCode::Compose => KeyCode::Compose,
        VirtualKeyCode::Caret => KeyCode::Caret,
        VirtualKeyCode::Numlock => KeyCode::Numlock,
        VirtualKeyCode::Numpad0 => KeyCode::Numpad0,
        VirtualKeyCode::Numpad1 => KeyCode::Numpad1,
        VirtualKeyCode::Numpad2 => KeyCode::Numpad2,
        VirtualKeyCode::Numpad3 => KeyCode::Numpad3,
        VirtualKeyCode::Numpad4 => KeyCode::Numpad4,
        VirtualKeyCode::Numpad5 => KeyCode::Numpad5,
        VirtualKeyCode::Numpad6 => KeyCode::Numpad6,
        VirtualKeyCode::Numpad7 => KeyCode::Numpad7,
        VirtualKeyCode::Numpad8 => KeyCode::Numpad8,
        VirtualKeyCode::Numpad9 => KeyCode::Numpad9,
        VirtualKeyCode::NumpadAdd => KeyCode::NumpadAdd,
        VirtualKeyCode::NumpadDivide => KeyCode::NumpadDivide,
        VirtualKeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
        VirtualKeyCode::NumpadComma => KeyCode::NumpadComma,
        VirtualKeyCode::NumpadEnter => KeyCode::NumpadEnter,
        VirtualKeyCode::NumpadEquals => KeyCode::NumpadEquals,
        VirtualKeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
        VirtualKeyCode::NumpadSubtract => KeyCode::NumpadSubtract,
        VirtualKeyCode::AbntC1 => KeyCode::AbntC1,
        VirtualKeyCode::AbntC2 => KeyCode::AbntC2,
        VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
        VirtualKeyCode::Apps => KeyCode::Apps,
        VirtualKeyCode::Asterisk => KeyCode::Asterisk,
        VirtualKeyCode::At => KeyCode::At,
        VirtualKeyCode::Ax => KeyCode::Ax,
        VirtualKeyCode::Backslash => KeyCode::Backslash,
        VirtualKeyCode::Calculator => KeyCode::Calculator,
        VirtualKeyCode::Capital => KeyCode::Capital,
        VirtualKeyCode::Colon => KeyCode::Colon,
        VirtualKeyCode::Comma => KeyCode::Comma,
        VirtualKeyCode::Convert => KeyCode::Convert,
        VirtualKeyCode::Equals => KeyCode::Equals,
        VirtualKeyCode::Grave => KeyCode::Grave,
        VirtualKeyCode::Kana => KeyCode::Kana,
        VirtualKeyCode::Kanji => KeyCode::Kanji,
        VirtualKeyCode::LAlt => KeyCode::LAlt,
        VirtualKeyCode::LBracket => KeyCode::LBracket,
        VirtualKeyCode::LControl => KeyCode::LControl,
        VirtualKeyCode::LShift => KeyCode::LShift,
        VirtualKeyCode::LWin => KeyCode::LWin,
        VirtualKeyCode::Mail => KeyCode::Mail,
        VirtualKeyCode::MediaSelect => KeyCode::MediaSelect,
        VirtualKeyCode::MediaStop => KeyCode::MediaStop,
        VirtualKeyCode::Minus => KeyCode::Minus,
        VirtualKeyCode::Mute => KeyCode::Mute,
        VirtualKeyCode::MyComputer => KeyCode::MyComputer,
        VirtualKeyCode::NavigateForward => KeyCode::NavigateForward,
        VirtualKeyCode::NavigateBackward => KeyCode::NavigateBackward,
        VirtualKeyCode::NextTrack => KeyCode::NextTrack,
        VirtualKeyCode::NoConvert => KeyCode::NoConvert,
        VirtualKeyCode::OEM102 => KeyCode::OEM102,
        VirtualKeyCode::Period => KeyCode::Period,
        VirtualKeyCode::PlayPause => KeyCode::PlayPause,
        VirtualKeyCode::Plus => KeyCode::Plus,
        VirtualKeyCode::Power => KeyCode::Power,
        VirtualKeyCode::PrevTrack => KeyCode::PrevTrack,
        VirtualKeyCode::RAlt => KeyCode::RAlt,
        VirtualKeyCode::RBracket => KeyCode::RBracket,
        VirtualKeyCode::RControl => KeyCode::RControl,
        VirtualKeyCode::RShift => KeyCode::RShift,
        VirtualKeyCode::RWin => KeyCode::RWin,
        VirtualKeyCode::Semicolon => KeyCode::Semicolon,
        VirtualKeyCode::Slash => KeyCode::Slash,
        VirtualKeyCode::Sleep => KeyCode::Sleep,
        VirtualKeyCode::Stop => KeyCode::Stop,
        VirtualKeyCode::Sysrq => KeyCode::Sysrq,
        VirtualKeyCode::Tab => KeyCode::Tab,
        VirtualKeyCode::Underline => KeyCode::Underline,
        VirtualKeyCode::Unlabeled => KeyCode::Unlabeled,
        VirtualKeyCode::VolumeDown => KeyCode::VolumeDown,
        VirtualKeyCode::VolumeUp => KeyCode::VolumeUp,
        VirtualKeyCode::Wake => KeyCode::Wake,
        VirtualKeyCode::WebBack => KeyCode::WebBack,
        VirtualKeyCode::WebFavorites => KeyCode::WebFavorites,
        VirtualKeyCode::WebForward => KeyCode::WebForward,
        VirtualKeyCode::WebHome => KeyCode::WebHome,
        VirtualKeyCode::WebRefresh => KeyCode::WebRefresh,
        VirtualKeyCode::WebSearch => KeyCode::WebSearch,
        VirtualKeyCode::WebStop => KeyCode::WebStop,
        VirtualKeyCode::Yen => KeyCode::Yen,
        VirtualKeyCode::Copy => KeyCode::Copy,
        VirtualKeyCode::Paste => KeyCode::Paste,
        VirtualKeyCode::Cut => KeyCode::Cut,
    }
}

#[inline]
fn convert_winit_mouse_button(winit: winit::event::MouseButton) -> Option<MouseButton> {
    match winit {
        winit::event::MouseButton::Left => Some(MouseButton::Left),
        winit::event::MouseButton::Right => Some(MouseButton::Right),
        winit::event::MouseButton::Middle => Some(MouseButton::Middle),
        winit::event::MouseButton::Other(_) => None,
    }
}
