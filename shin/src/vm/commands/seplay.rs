use super::prelude::*;

pub struct SEPLAY;

impl super::Command<command::runtime::SEPLAY> for SEPLAY {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::SEPLAY, state: &mut VmState) {
        warn!("TODO: SEPLAY state: {:?}", command);
    }

    fn start(command: command::runtime::SEPLAY, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: SEPLAY: {:?}", command);
        command.token.finish()
    }
}
