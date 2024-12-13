use shin_core::vm::command::types::PlaneId;
use tracing::debug;

use super::prelude::*;
use crate::adv::vm_state::layers::LayerOperationTargetList;

pub struct LayerLoadStateInfo {
    plane: PlaneId,
    affected_layers: LayerOperationTargetList,
}

impl StartableCommand for command::runtime::LAYERUNLOAD {
    type StateInfo = LayerLoadStateInfo;
    fn apply_state(&self, state: &mut VmState) -> LayerLoadStateInfo {
        let layers = &mut state.layers;
        let plane = layers.current_plane;

        let mut affected_layers = Default::default();

        match self.layer_id.repr() {
            VLayerIdRepr::RootLayerGroup
            | VLayerIdRepr::ScreenLayer
            | VLayerIdRepr::PageLayer
            | VLayerIdRepr::PlaneLayerGroup => {
                warn!(
                    "Attempt to unload a special layer ({:?})",
                    self.layer_id.repr()
                );
            }
            VLayerIdRepr::Selected => {
                affected_layers = layers.layerbank_allocator.layers_in_range(
                    layers.current_plane,
                    layers.layer_selection.from,
                    layers.layer_selection.to,
                )
            }
            VLayerIdRepr::Layer(layer_id) => {
                affected_layers = layers.layerbank_allocator.layers_in_range(
                    layers.current_plane,
                    layer_id,
                    layer_id,
                )
            }
        }

        for &target in &affected_layers {
            layers.layerbanks[target.layerbank].layer_type = None;
            layers
                .layerbank_allocator
                .free_layerbank(plane, target.layer);
        }

        LayerLoadStateInfo {
            plane,
            affected_layers,
        }
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        state_info: LayerLoadStateInfo,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        // TODO: create a ¿back layer group? if it doesn't exist yet

        let plane_layer_group = adv_state.plane_layer_group_mut(state_info.plane);

        for &target in &state_info.affected_layers {
            plane_layer_group.remove_layer(target.layerbank, self.delay_time);
        }

        self.token.finish().into()
    }
}
