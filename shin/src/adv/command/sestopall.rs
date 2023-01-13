use super::prelude::*;

impl StartableCommand for command::runtime::SESTOPALL {
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: SESTOPALL state: {:?}", self);
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        adv_state.se_player.stop_all(self.fade_out_time);
        self.token.finish().into()
    }
}
