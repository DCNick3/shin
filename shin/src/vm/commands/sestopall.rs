use crate::vm::Vm;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;
use tracing::warn;

pub struct SESTOPALL;

impl super::Command<command::runtime::SESTOPALL> for SESTOPALL {
    type Result = CommandResult;

    fn start(command: command::runtime::SESTOPALL, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: SESTOPALL: {:?}", command);
        command.token.finish()
    }
}
