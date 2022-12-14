use super::prelude::*;

impl super::StartableCommand for command::runtime::SESTOP {
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: SESTOP state: {:?}", self);
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        adv_state.se_player.stop(self.se_slot, self.fade_out_time);

        self.token.finish().into()
    }
}
