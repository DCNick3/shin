use super::prelude::*;

pub struct AUTOSAVE;

impl super::Command<command::runtime::AUTOSAVE> for AUTOSAVE {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::AUTOSAVE, state: &mut VmState) {
        todo!()
    }

    fn start(command: command::runtime::AUTOSAVE, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: AUTOSAVE: {:?}", command);
        command.token.finish()
    }
}
