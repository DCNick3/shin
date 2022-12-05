use super::prelude::*;

pub struct MSGINIT;

impl super::Command<command::runtime::MSGINIT> for MSGINIT {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::MSGINIT, state: &mut VmState) {
        todo!()
    }

    fn start(command: command::runtime::MSGINIT, vm: &mut Vm) -> Self::Result {
        vm.state.msg_info.msginit = Some(command.messagebox_param);
        command.token.finish()
    }
}
