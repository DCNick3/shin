use super::prelude::*;
use crate::layer::UserLayer;

impl super::StartableCommand for command::runtime::LAYERLOAD {
    fn apply_state(&self, state: &mut VmState) {
        assert_eq!(self.leave_uninitialized, 0); // I __think__ this has to do with init props/leave them be, but I'm not sure

        match self.layer_id.repr() {
            VLayerIdRepr::RootLayerGroup
            | VLayerIdRepr::ScreenLayer
            | VLayerIdRepr::PageLayer
            | VLayerIdRepr::PlaneLayerGroup => {
                unreachable!("You can't load special layers")
            }
            VLayerIdRepr::Selected => {
                todo!("LAYERLOAD: selected");
            }
            VLayerIdRepr::Layer(id) => {
                // unwrap_or_else is unusable because of borrow checker
                let layer = match state.layers.get_layer_mut(id) {
                    None => state.layers.alloc(id),
                    Some(v) => v,
                };

                layer.layerinit_params = Some((self.layer_type, self.params));
            }
        }
    }

    fn start(
        self,
        context: &UpdateContext,
        scenario: &Scenario,
        vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        // TODO: loading should be done async
        let layer = UserLayer::load(
            context.gpu_resources,
            context.game_data,
            scenario,
            self.layer_type,
            self.params,
        );

        match self.layer_id.repr() {
            VLayerIdRepr::RootLayerGroup
            | VLayerIdRepr::ScreenLayer
            | VLayerIdRepr::PageLayer
            | VLayerIdRepr::PlaneLayerGroup => {
                unreachable!("You can't load special layers")
            }
            VLayerIdRepr::Selected => {
                todo!("LAYERLOAD: selected");
            }
            VLayerIdRepr::Layer(id) => adv_state
                .current_layer_group_mut(vm_state)
                .add_layer(id, layer),
        }

        self.token.finish().into()
    }
}
