use shin_core::time::Tween;

use super::prelude::*;

impl StartableCommand for command::runtime::SEVOL {
    type StateInfo = ();
    fn apply_state(&self, state: &mut VmState) {
        if let Some(state) = state.audio.se[self.se_slot as usize].as_mut() {
            state.volume = self.volume;
        }
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _state_info: (),
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        adv_state
            .se_player
            .set_volume(self.se_slot, self.volume, Tween::linear(self.fade_in_time));

        self.token.finish().into()
    }
}
