use super::prelude::*;

impl super::StartableCommand for command::runtime::MSGCLOSE {
    fn apply_state(&self, state: &mut VmState) {
        state.messagebox_state.messagebox_shown = false;
        state.messagebox_state.text = None;
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Scenario,
        _vm_state: &VmState,
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        assert_eq!(self.wait_for_close, 0);
        self.token.finish().into()
    }
}
