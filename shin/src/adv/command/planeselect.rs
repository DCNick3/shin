use shin_core::vm::command::types::{PLANES_COUNT, PlaneId};

use super::prelude::*;

impl StartableCommand for command::runtime::PLANESELECT {
    type StateInfo = ();
    fn apply_state(&self, state: &mut VmState) {
        state.layers.current_plane = self.plane_id;
    }

    fn start(
        self,
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _state_info: (),
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        self.token.finish().into()
    }
}
