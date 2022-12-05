use crate::vm::Vm;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;

pub struct SGET;

impl super::Command<command::runtime::SGET> for SGET {
    type Result = CommandResult;

    fn start(command: command::runtime::SGET, vm: &mut Vm) -> Self::Result {
        let value = vm.state.globals_info.get(command.slot_number);
        command.token.finish(value)
    }
}
