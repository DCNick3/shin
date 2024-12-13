use super::prelude::*;
use crate::adv::vm_state::layers::LayerSelection;

impl StartableCommand for command::runtime::LAYERSELECT {
    type StateInfo = ();
    fn apply_state(&self, state: &mut VmState) {
        let mut from = self.selection_start_id;
        let mut to = self.selection_end_id;

        if from > to {
            warn!(
                "LAYERSELECT: invalid selection range order: {:?} > {:?}",
                from, to
            );
            std::mem::swap(&mut from, &mut to);
        }

        state.layers.layer_selection = LayerSelection { from, to }
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
