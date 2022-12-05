use super::prelude::*;

pub struct WIPE;

impl super::Command<command::runtime::WIPE> for WIPE {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::WIPE, state: &mut VmState) {
        todo!()
    }

    fn start(command: command::runtime::WIPE, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: WIPE: {:?}", command);
        command.token.finish()
    }
}
