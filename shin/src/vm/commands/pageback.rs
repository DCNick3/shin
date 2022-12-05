use super::prelude::*;

pub struct PAGEBACK;

impl super::Command<command::runtime::PAGEBACK> for PAGEBACK {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::PAGEBACK, state: &mut VmState) {
        todo!()
    }

    fn start(command: command::runtime::PAGEBACK, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: PAGEBACK: {:?}", command);
        command.token.finish()
    }
}
