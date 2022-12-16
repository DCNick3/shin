use super::prelude::*;

impl super::StartableCommand for command::runtime::SEPLAY {
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: SEPLAY state: {:?}", self);
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Scenario,
        _vm_state: &VmState,
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        warn!("TODO: SEPLAY: {:?}", self);
        self.token.finish().into()
    }
}
