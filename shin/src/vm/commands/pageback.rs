use crate::vm::Vm;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;
use tracing::warn;

pub struct PAGEBACK;

impl super::Command<command::runtime::PAGEBACK> for PAGEBACK {
    type Result = CommandResult;

    fn start(command: command::runtime::PAGEBACK, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: PAGEBACK: {:?}", command);
        command.token.finish()
    }
}
