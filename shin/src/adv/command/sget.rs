use shin_core::format::scenario::Scenario;

use super::prelude::*;

impl StartableCommand for command::runtime::SGET {
    type StateInfo = ();

    fn apply_state(&self, _state: &mut VmState) {
        // nothing to do
    }

    fn start(
        self,
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        vm_state: &VmState,
        _state_info: (),
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        let value = vm_state.persist.get(self.slot_number);
        self.token.finish(value).into()
    }
}
