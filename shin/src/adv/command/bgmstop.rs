use shin_core::time::Tween;

use super::prelude::*;

impl StartableCommand for command::runtime::BGMSTOP {
    type StateInfo = ();
    fn apply_state(&self, state: &mut VmState) {
        state.audio.bgm = None;
    }

    fn start(
        self,
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _state_info: (),
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        adv_state.bgm_player.stop(Tween::linear(self.fade_out_time));
        self.token.finish().into()
    }
}
