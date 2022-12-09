use super::prelude::*;

impl super::StartableCommand for command::runtime::SGET {
    fn apply_state(&self, _state: &mut VmState) {
        // nothing to do
    }

    fn start(
        self,
        _context: &UpdateContext,
        vm_state: &VmState,
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        let value = vm_state.globals.get(self.slot_number);
        self.token.finish(value).into()
    }
}
