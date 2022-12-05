use crate::vm::Vm;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;
use tracing::warn;

pub struct SEPLAY;

impl super::Command<command::runtime::SEPLAY> for SEPLAY {
    type Result = CommandResult;

    fn start(command: command::runtime::SEPLAY, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: SEPLAY: {:?}", command);
        command.token.finish()
    }
}
