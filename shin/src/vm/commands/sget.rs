use super::prelude::*;

pub struct SGET;

impl super::Command<command::runtime::SGET> for SGET {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::SGET, state: &mut VmState) {
        todo!()
    }

    fn start(command: command::runtime::SGET, vm: &mut Vm) -> Self::Result {
        let value = vm.state.globals.get(command.slot_number);
        command.token.finish(value)
    }
}
