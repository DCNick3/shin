use super::prelude::*;

impl StartableCommand for command::runtime::SAVEINFO {
    type StateInfo = ();
    fn apply_state(&self, state: &mut VmState) {
        state.save_info.set_save_info(self.level, self.info.clone());
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
