use crate::input::buttonlike::ButtonState;
use crate::input::inputs::{GamepadButtonType, KeyCode, MouseButton};
use crate::input::raw_input_state::RawInputState;
use enum_map::{enum_map, Enum, EnumMap};
use petitset::PetitSet;

// pub enum Action {
//     Confirm,    // A / Enter / Space
//     Cancel,     // B / Escape
//     Menu,       // Y / Tab / Escape
//     Rollback,   // Up / MouseWheelUp / PageUp
//     Continue,   // A / MouseClick / Enter / Space
//     Backlog,    // X / MouseRightClick
//     ScrollUp,   // Stick Up / MouseWheelUp / PageUp
//     ScrollDown, // Stick Down / MouseWheelDown / PageDown
// }

// TODO: add a derive or smth
pub trait Action: Enum + Copy + Clone + Send + Sync + 'static {
    fn default_action_map() -> ActionMap<Self>;
}

struct ActionData {
    state: ButtonState,
    amount: f32,
}

impl ActionData {
    fn press(&mut self, amount: f32) {
        if self.state != ButtonState::Pressed {
            self.state = ButtonState::JustPressed;
        }
        self.amount = amount;
    }

    fn release(&mut self) {
        if self.state != ButtonState::Released {
            self.state = ButtonState::JustReleased;
        }
        self.amount = 0.0;
    }

    fn update(&mut self) {
        self.state = match self.state {
            ButtonState::JustPressed => ButtonState::Pressed,
            ButtonState::JustReleased => ButtonState::Released,
            _ => self.state,
        };
    }

    fn reset(&mut self) {
        self.state = ButtonState::Released;
        self.amount = 0.0;
    }
}

pub struct ActionState<T: Action> {
    action_data: EnumMap<T, ActionData>,
}

impl<T: Action> ActionState<T> {
    pub fn new() -> Self {
        Self {
            action_data: enum_map! { _ => ActionData { state: ButtonState::Released, amount: 0.0 } },
        }
    }

    pub fn update(&mut self, action_map: &ActionMap<T>, raw_input_state: &RawInputState) {
        self.action_data.values_mut().for_each(|d| d.update());

        let pressed = action_map.which_pressed(raw_input_state);
        for ((_action, pressed), data) in pressed.into_iter().zip(self.action_data.values_mut()) {
            if let Some(amount) = pressed {
                data.press(amount);
            } else {
                data.release();
            }
        }
    }

    pub fn reset(&mut self) {
        for action_data in self.action_data.values_mut() {
            action_data.reset();
        }
    }

    pub fn is_just_pressed(&self, action: T) -> bool {
        self.action_data[action].state == ButtonState::JustPressed
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UserInput {
    // NOTE: no input chords support
    Keyboard(KeyCode),
    MouseButton(MouseButton),
    GamepadButton(GamepadButtonType),
}

impl From<KeyCode> for UserInput {
    fn from(key_code: KeyCode) -> Self {
        UserInput::Keyboard(key_code)
    }
}

impl From<MouseButton> for UserInput {
    fn from(mouse_button: MouseButton) -> Self {
        UserInput::MouseButton(mouse_button)
    }
}

impl From<GamepadButtonType> for UserInput {
    fn from(gamepad_button_type: GamepadButtonType) -> Self {
        UserInput::GamepadButton(gamepad_button_type)
    }
}

pub struct ActionMap<A: Action> {
    action_map: EnumMap<A, PetitSet<UserInput, 8>>, // OR is applied to the sources
}

pub type InputSet = PetitSet<UserInput, 8>;

impl<A: Action> ActionMap<A> {
    pub fn new(action_map: EnumMap<A, PetitSet<UserInput, 8>>) -> Self {
        Self { action_map }
    }

    pub fn which_pressed(&self, input_state: &RawInputState) -> EnumMap<A, Option<f32>> {
        self.action_map.map_ref(|action, inputs| {
            inputs
                .iter()
                // flat map acts as an OR
                .flat_map(|input| input_state.is_pressed(input))
                // return the first match (this might be important!)
                .next()
        })
    }
}
