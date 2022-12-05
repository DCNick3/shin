use super::prelude::*;

pub struct SESTOPALL;

impl super::Command<command::runtime::SESTOPALL> for SESTOPALL {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::SESTOPALL, state: &mut VmState) {
        todo!()
    }

    fn start(command: command::runtime::SESTOPALL, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: SESTOPALL: {:?}", command);
        command.token.finish()
    }
}
