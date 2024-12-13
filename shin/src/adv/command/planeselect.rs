use shin_core::vm::command::types::{PlaneId, PLANES_COUNT};

use super::prelude::*;

impl StartableCommand for command::runtime::PLANESELECT {
    type StateInfo = ();
    fn apply_state(&self, state: &mut VmState) {
        assert!(
            self.plane_id >= 0 && self.plane_id < PLANES_COUNT as _,
            "invalid plane id: {}",
            self.plane_id
        );
        state.layers.current_plane = PlaneId::new(self.plane_id as _);
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _state_info: (),
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        self.token.finish().into()
    }
}
