use super::prelude::*;
use shin_core::time::Tween;

impl StartableCommand for command::runtime::BGMPLAY {
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: BGMPLAY state: {:?}", self);
    }

    fn start(
        self,
        context: &UpdateContext,
        scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        let (bgm_filename, _bgm_name, _idk) = scenario.get_bgm_data(self.bgm_data_id);

        let bgm_path = format!("/bgm/{}.nxa", bgm_filename);

        let audio = context
            .asset_server
            // TODO: sync - bad!!
            .load_sync(bgm_path)
            .expect("Failed to load BGM track");

        adv_state.bgm_player.play(
            audio,
            !self.no_repeat,
            self.volume as f32 / 1000.0,
            Tween::linear(self.fade_in_time),
        );

        self.token.finish().into()
    }
}
