use shin_core::time::{Easing, Tween};
use shin_render::shaders::types::RenderCloneCtx;

use super::prelude::*;
use crate::{adv::vm_state::layers::LayerOperationTargetList, layer::LayerProperties};

impl StartableCommand for command::runtime::LAYERCTRL {
    type StateInfo = LayerOperationTargetList;
    fn apply_state(&self, state: &mut VmState) -> LayerOperationTargetList {
        let layers = &mut state.layers;
        let (target_value, _time, _flags, _easing_param, ..) = self.params;

        let mut affected_layers = Default::default();

        match self.layer_id.repr() {
            VLayerIdRepr::RootLayerGroup => {
                layers.root_layer_group.get_property(self.property_id);
            }
            VLayerIdRepr::ScreenLayer => {
                layers.screen_layer.get_property(self.property_id);
            }
            VLayerIdRepr::PageLayer => {
                layers.page_layer.get_property(self.property_id);
            }
            VLayerIdRepr::PlaneLayerGroup => {
                layers.plane_layergroups[layers.current_plane]
                    .properties
                    .get_property(self.property_id);
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
            layers.layerbanks[target.layerbank]
                .properties
                .set_property(self.property_id, target_value);
        }

        affected_layers
    }

    fn start(
        self,
        context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        vm_state: &VmState,
        affected_layers: LayerOperationTargetList,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        let (target_value, duration, flags, easing_param, ..) = self.params;

        let mut clone_ctx = RenderCloneCtx::new(context.pre_render.device);
        adv_state.create_back_layer_group_if_needed(&mut clone_ctx);
        clone_ctx.finish(context.pre_render.queue);

        if flags.unused_1() != 0 || flags.unused_2() != 0 || flags.unused_3() != 0 {
            panic!("LAYERCTRL: unused flags are set: {:?}", flags);
        }

        if flags.scale_time() {
            warn!("LAYERCTRL: scale_time is set, but not supported");
        }
        if flags.delta() {
            // note: delta flag has a non-trivial interaction with queue clear flags
            warn!("LAYERCTRL: delta is set, but not supported");
        }
        if flags.ff_to_current() && flags.ff_to_target() {
            panic!("LAYERCTRL: both ff_to_current and ff_to_target flags are set");
        }
        if flags.prohibit_fast_forward() {
            warn!("LAYERCTRL: prohibit_fast_forward is set, but not supported");
        }
        if flags.ignore_wait() {
            warn!("LAYERCTRL: ignore_wait is set, but not supported");
        }

        let easing = match flags.easing() {
            0 => Easing::Linear,
            1 => Easing::SineIn,
            2 => Easing::SineOut,
            3 => Easing::SineInOut,
            4 => Easing::Jump,
            5 => Easing::Power(easing_param),
            _ => panic!("LAYERCTRL: unknown easing function: {}", flags.easing()),
        };

        let mut changed = false;
        let mut apply_to_properties = |properties: &mut LayerProperties| {
            let tweener = properties.property_tweener_mut(self.property_id);

            let from_value = tweener.target_value();
            let to_value = target_value as f32;
            let mut duration = duration;

            if tweener.value() != to_value {
                changed = true;
            }

            if flags.scale_time() {
                // this flag makes "duration" actually mean change rate (in value per tick)
                let change = (to_value - from_value).abs();
                duration = Ticks::from_f32(change / duration.as_f32());
            }

            if flags.ff_to_current() {
                if flags.delta() {
                    todo!(
                        "LAYERCTRL: ff_to_current and delta flags have an interaction that is not yet implemented"
                    );
                }

                let current = tweener.value();
                tweener.fast_forward_to(current);
            }
            if flags.ff_to_target() {
                tweener.fast_forward();
            }

            tweener.enqueue(target_value as f32, Tween { duration, easing })
        };

        match self.layer_id.repr() {
            VLayerIdRepr::RootLayerGroup => {
                apply_to_properties(adv_state.root_layer_group_mut().properties_mut());
            }
            VLayerIdRepr::ScreenLayer => {
                apply_to_properties(adv_state.screen_layer_mut().properties_mut());
            }
            VLayerIdRepr::PageLayer => {
                apply_to_properties(adv_state.page_layer_mut().properties_mut());
            }
            VLayerIdRepr::PlaneLayerGroup => {
                apply_to_properties(
                    adv_state
                        .plane_layer_group_mut(vm_state.layers.current_plane)
                        .properties_mut(),
                );
            }
            VLayerIdRepr::Selected | VLayerIdRepr::Layer(_) => {
                for &target in &affected_layers {
                    apply_to_properties(
                        adv_state
                            .plane_layer_group_mut(vm_state.layers.current_plane)
                            .get_layer_mut(target.layerbank)
                            .expect("BUG: layerbank not found")
                            .properties_mut(),
                    );
                }
            }
        }

        self.token.finish().into()
    }
}
