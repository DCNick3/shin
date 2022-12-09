use super::prelude::*;

impl super::StartableCommand for command::runtime::MSGINIT {
    fn apply_state(&self, state: &mut VmState) {
        state.messagebox_state.msginit = Some(self.messagebox_param);
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Scenario,
        _vm_state: &VmState,
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        self.token.finish().into()
    }
}
