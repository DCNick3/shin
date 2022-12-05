use super::prelude::*;

pub struct SSET;

impl super::Command<command::runtime::SSET> for SSET {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::SSET, state: &mut VmState) {
        todo!()
    }

    fn start(command: command::runtime::SSET, vm: &mut Vm) -> Self::Result {
        vm.state
            .globals_info
            .set(command.slot_number, command.value);
        command.token.finish()
    }
}
