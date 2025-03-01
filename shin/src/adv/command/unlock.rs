use super::prelude::*;

impl StartableCommand for command::runtime::UNLOCK {
    type StateInfo = ();
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: UNLOCK state: {:?}", self);
    }

    fn start(
        self,
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _state_info: (),
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        warn!("TODO: UNLOCK: {:?}", self);
        self.token.finish().into()
    }
}
