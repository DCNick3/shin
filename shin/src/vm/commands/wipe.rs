use super::prelude::*;

pub struct WIPE;

impl super::Command<command::runtime::WIPE> for WIPE {
    type Result = CommandResult;

    fn apply_state(_command: &command::runtime::WIPE, _state: &mut VmState) {
        warn!("TODO: WIPE state: {:?}", _command);
        // we don't track wipes yet
    }

    fn start(command: command::runtime::WIPE, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: WIPE: {:?}", command);
        command.token.finish()
    }
}
