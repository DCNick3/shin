use crate::input::inputs::{KeyCode, MouseButton};
use crate::input::{Action, ActionMap, InputSet};
use enum_map::{enum_map, Enum};

// Action available in all ADV contexts
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Enum)]
pub enum AdvMessageAction {
    Advance,
    Backlog,
    Rollback,
}

impl Action for AdvMessageAction {
    fn default_action_map() -> ActionMap<Self> {
        fn map(v: AdvMessageAction) -> InputSet {
            match v {
                AdvMessageAction::Advance => [
                    MouseButton::Left.into(),
                    KeyCode::Enter.into(),
                    KeyCode::Space.into(),
                ]
                .into_iter()
                .collect(),
                AdvMessageAction::Backlog => [].into_iter().collect(),
                AdvMessageAction::Rollback => [].into_iter().collect(),
            }
        }

        ActionMap::new(enum_map! { v => map(v) })
    }
}
