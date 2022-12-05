use crate::vm::Vm;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;
use tracing::warn;

pub struct WIPE;

impl super::Command<command::runtime::WIPE> for WIPE {
    type Result = CommandResult;

    fn start(command: command::runtime::WIPE, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: WIPE: {:?}", command);
        command.token.finish()
    }
}
