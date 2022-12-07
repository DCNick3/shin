use super::prelude::*;

impl super::StartableCommand for command::runtime::SSET {
    fn apply_state(&self, state: &mut VmState) {
        state.globals.set(self.slot_number, self.value);
    }

    fn start(self, _vm_state: &VmState, _adv_state: &mut AdvState) -> CommandStartResult {
        self.token.finish().into()
    }
}
