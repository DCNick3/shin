use super::prelude::*;

pub struct SAVEINFO;

impl super::Command<command::runtime::SAVEINFO> for SAVEINFO {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::SAVEINFO, state: &mut VmState) {
        todo!()
    }

    fn start(command: command::runtime::SAVEINFO, vm: &mut Vm) -> Self::Result {
        vm.state
            .save_info
            .set_save_info(command.level, command.info);
        command.token.finish()
    }
}
