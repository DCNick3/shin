use super::prelude::*;

impl super::StartableCommand for command::runtime::SEPLAY {
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: SEPLAY state: {:?}", self);
    }

    fn start(
        self,
        context: &UpdateContext,
        scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        if self.pan != 0 {
            warn!("TODO: SEPLAY: ignoring pan={}", self.pan);
        }
        if self.play_speed != 1000 {
            warn!("TODO: SEPLAY: ignoring play_speed={}", self.play_speed);
        }
        if self.no_repeat == 0 {
            warn!("TODO: SEPLAY: ignoring no_repeat={}", self.no_repeat);
        }

        let se_name = scenario.get_se_data(self.se_data_id);
        let se_path = format!("/se/{}.nxa", se_name);

        let audio = context
            .asset_server
            // TODO: sync - bad!!
            .load_sync(se_path)
            .expect("Failed to load BGM track");

        adv_state.se_player.play(
            self.se_slot,
            audio,
            (self.volume as f32 / 1000.0).clamp(0.0, 1.0),
        );

        self.token.finish().into()
    }
}
