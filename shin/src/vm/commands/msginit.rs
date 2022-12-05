use crate::vm::Vm;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;

pub struct MSGINIT;

impl super::Command<command::runtime::MSGINIT> for MSGINIT {
    type Result = CommandResult;

    fn start(command: command::runtime::MSGINIT, vm: &mut Vm) -> Self::Result {
        vm.state.msg_info.msginit = Some(command.messagebox_param);
        command.token.finish()
    }
}
