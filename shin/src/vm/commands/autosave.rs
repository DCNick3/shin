use crate::vm::Vm;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;
use tracing::warn;

pub struct AUTOSAVE;

impl super::Command<command::runtime::AUTOSAVE> for AUTOSAVE {
    type Result = CommandResult;

    fn start(command: command::runtime::AUTOSAVE, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: AUTOSAVE: {:?}", command);
        command.token.finish()
    }
}
