use super::prelude::*;

impl super::StartableCommand for command::runtime::MSGINIT {
    fn apply_state(&self, state: &mut VmState) {
        todo!()
    }

    fn start(self, vm: &mut Vm) -> CommandStartResult {
        vm.state.messagebox_state.msginit = Some(self.messagebox_param);
        self.token.finish().into()
    }
}
