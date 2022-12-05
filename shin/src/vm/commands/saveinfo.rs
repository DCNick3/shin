use crate::vm::Vm;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;

pub struct SAVEINFO;

impl super::Command<command::runtime::SAVEINFO> for SAVEINFO {
    type Result = CommandResult;

    fn start(command: command::runtime::SAVEINFO, vm: &mut Vm) -> Self::Result {
        vm.state
            .save_info
            .set_save_info(command.level, command.info);
        command.token.finish()
    }
}
