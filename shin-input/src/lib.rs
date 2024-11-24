pub mod inputs;

mod action;
mod raw_input_state;

pub use action::{Action, ActionSignal, ActionState, ActionsState, DummyAction};
pub use raw_input_state::{RawInputAccumulator, RawInputState};
