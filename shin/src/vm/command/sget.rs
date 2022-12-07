use super::prelude::*;

impl super::StartableCommand for command::runtime::SGET {
    fn apply_state(&self, state: &mut VmState) {
        todo!()
    }

    fn start(self, vm: &mut Vm) -> CommandStartResult {
        let value = vm.state.globals.get(self.slot_number);
        self.token.finish(value).into()
    }
}
