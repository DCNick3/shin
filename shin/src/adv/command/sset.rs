use super::prelude::*;

impl StartableCommand for command::runtime::SSET {
    type StateInfo = ();
    fn apply_state(&self, state: &mut VmState) {
        state.persist.set(self.slot_number, self.value);
    }

    fn start(
        self,
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _state_info: (),
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        self.token.finish().into()
    }
}
