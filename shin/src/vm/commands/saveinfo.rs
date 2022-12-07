use super::prelude::*;

pub struct SAVEINFO;

impl super::Command<command::runtime::SAVEINFO> for SAVEINFO {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::SAVEINFO, state: &mut VmState) {
        state
            .save_info
            .set_save_info(command.level, command.info.clone());
    }

    fn start(command: command::runtime::SAVEINFO, _vm: &mut Vm) -> Self::Result {
        command.token.finish()
    }
}
