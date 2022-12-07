use super::prelude::*;

impl super::StartableCommand for command::runtime::SSET {
    fn apply_state(&self, state: &mut VmState) {
        todo!()
    }

    fn start(self, vm: &mut Vm) -> CommandStartResult {
        vm.state.globals.set(self.slot_number, self.value);
        self.token.finish().into()
    }
}
