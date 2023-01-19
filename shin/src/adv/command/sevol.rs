use super::prelude::*;
use shin_core::time::Tween;

impl StartableCommand for command::runtime::SEVOL {
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: SEVOL state: {:?}", self);
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        adv_state.se_player.set_volume(
            self.se_slot,
            self.volume as f32 / 1000.0,
            Tween::linear(self.fade_in_time),
        );

        self.token.finish().into()
    }
}
