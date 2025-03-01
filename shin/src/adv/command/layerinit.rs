use shin_core::vm::command::types::{LayerId, LayerbankId};
use shin_render::shaders::types::RenderCloneCtx;

use super::prelude::*;
use crate::adv::vm_state::layers::LayerOperationTargetList;

impl StartableCommand for command::runtime::LAYERINIT {
    type StateInfo = LayerOperationTargetList;
    fn apply_state(&self, state: &mut VmState) -> Self::StateInfo {
        let layers = &mut state.layers;

        let mut affected_layers = Default::default();

        match self.layer_id.repr() {
            VLayerIdRepr::RootLayerGroup => {
                layers.root_layer_group.init();
            }
            VLayerIdRepr::ScreenLayer => {
                layers.screen_layer.init();
            }
            VLayerIdRepr::PageLayer => {
                layers.page_layer.init();
            }
            VLayerIdRepr::PlaneLayerGroup => {
                layers.plane_layergroups[layers.current_plane]
                    .properties
                    .init();
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
            layers.layerbanks[target.layerbank].properties.init();
        }

        affected_layers
    }

    fn start(
        self,
        context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        vm_state: &VmState,
        affected_layers: LayerOperationTargetList,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        adv_state.create_back_layer_group_if_needed(&mut context.pre_render.render_clone_ctx());

        match self.layer_id.repr() {
            VLayerIdRepr::RootLayerGroup => {
                adv_state.root_layer_group_mut().properties_mut().init()
            }
            VLayerIdRepr::ScreenLayer => {
                adv_state.screen_layer_mut().properties_mut().init();
            }
            VLayerIdRepr::PageLayer => {
                adv_state.page_layer_mut().properties_mut().init();
            }
            VLayerIdRepr::PlaneLayerGroup => {
                adv_state
                    .plane_layer_group_mut(vm_state.layers.current_plane)
                    .properties_mut()
                    .init();
            }
            VLayerIdRepr::Selected | VLayerIdRepr::Layer(_) => {
                for target in affected_layers {
                    adv_state
                        .plane_layer_group_mut(vm_state.layers.current_plane)
                        .get_layer_mut(target.layerbank)
                        .expect("BUG: layerbank not found")
                        .properties_mut()
                        .init();
                }
            }
        }

        self.token.finish().into()
    }
}
