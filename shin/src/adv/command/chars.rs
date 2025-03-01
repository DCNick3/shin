use super::prelude::*;

impl StartableCommand for command::runtime::CHARS {
    type StateInfo = ();
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: CHARS state: {:?}", self);
    }

    fn start(
        self,
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _state_info: (),
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        warn!("TODO: CHARS: {:?}", self);
        self.token.finish().into()
    }
}
