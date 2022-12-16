use super::prelude::*;

impl super::StartableCommand for command::runtime::MSGWAIT {
    fn apply_state(&self, _state: &mut VmState) {
        // nothing to do
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Scenario,
        _vm_state: &VmState,
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        warn!("TODO: MSGWAIT {:?}", self);
        self.token.finish().into()
    }
}
