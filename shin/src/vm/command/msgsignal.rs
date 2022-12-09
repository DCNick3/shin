use super::prelude::*;

impl super::StartableCommand for command::runtime::MSGSIGNAL {
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
        warn!("TODO: MSGSIGNAL {:?}", self);
        self.token.finish().into()
    }
}
