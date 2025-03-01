use shin_core::time::Tween;

use super::prelude::*;

impl StartableCommand for command::runtime::SESTOPALL {
    type StateInfo = ();
    fn apply_state(&self, state: &mut VmState) {
        state.audio.se.iter_mut().for_each(|v| *v = None);
    }

    fn start(
        self,
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _state_info: (),
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        adv_state
            .se_player
            .stop_all(Tween::linear(self.fade_out_time));
        self.token.finish().into()
    }
}
