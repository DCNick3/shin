use shin_core::vm::command::types::{LayerId, PlaneId, LAYERS_COUNT};

use super::prelude::*;

impl StartableCommand for command::runtime::PLANECLEAR {
    type StateInfo = PlaneId;
    fn apply_state(&self, state: &mut VmState) -> PlaneId {
        let layers = &mut state.layers;
        let plane = layers.current_plane;

        let affected_layers = layers.layerbank_allocator.layers_in_range(
            plane,
            LayerId::new(0),
            LayerId::new(LAYERS_COUNT as u16 - 1),
        );

        for &target in &affected_layers {
            layers.layerbanks[target.layerbank].layer_type = None;
            layers
                .layerbank_allocator
                .free_layerbank(plane, target.layer);
        }

        layers.plane_layergroups[plane].mask_id = -1;

        plane
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        state_info: PlaneId,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        // TODO: create a ¿back layer group? if it doesn't exist yet

        let layer_group = adv_state.plane_layer_group_mut(state_info);

        layer_group.clear_layers();
        layer_group.clear_mask_texture();

        self.token.finish().into()
    }
}
