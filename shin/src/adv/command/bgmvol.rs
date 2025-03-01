use shin_core::time::Tween;

use super::prelude::*;

impl StartableCommand for command::runtime::BGMVOL {
    type StateInfo = ();
    fn apply_state(&self, state: &mut VmState) {
        if let Some(state) = state.audio.bgm.as_mut() {
            state.volume = self.volume;
        }
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
            .bgm_player
            .set_volume(self.volume, Tween::linear(self.fade_in_time));
        self.token.finish().into()
    }
}
