use super::prelude::*;

pub struct PAGEBACK;

impl super::Command<command::runtime::PAGEBACK> for PAGEBACK {
    type Result = CommandResult;

    fn apply_state(_command: &command::runtime::PAGEBACK, _state: &mut VmState) {
        warn!("TODO: PAGEBACK state: {:?}", _command);
        // TODO: I __think__ we should have a way to store this in the state
        // I am still not sure of the paradigm, lol
        // ignore for now (along with WIPE)
    }

    fn start(command: command::runtime::PAGEBACK, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: PAGEBACK: {:?}", command);
        command.token.finish()
    }
}
