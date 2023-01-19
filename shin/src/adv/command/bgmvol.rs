use super::prelude::*;
use shin_core::time::Tween;

impl StartableCommand for command::runtime::BGMVOL {
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: BGMVOL state: {:?}", self);
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        adv_state.bgm_player.set_volume(
            self.volume as f32 / 1000.0,
            Tween::linear(self.fade_in_time),
        );
        self.token.finish().into()
    }
}
