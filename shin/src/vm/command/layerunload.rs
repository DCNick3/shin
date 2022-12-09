use super::prelude::*;

impl super::StartableCommand for command::runtime::LAYERUNLOAD {
    fn apply_state(&self, state: &mut VmState) {
        // TODO: make another utility function for this
        match self.layer_id.repr() {
            VLayerIdRepr::RootLayerGroup
            | VLayerIdRepr::ScreenLayer
            | VLayerIdRepr::PageLayer
            | VLayerIdRepr::PlaneLayerGroup => {
                unreachable!("You can't unload special layers")
            }
            VLayerIdRepr::Selected => {
                todo!("LAYERUNLOAD: selected");
            }
            VLayerIdRepr::Layer(id) => {
                state.layers.free(id);
            }
        }
    }

    fn start(
        self,
        _context: &UpdateContext,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        match self.layer_id.repr() {
            VLayerIdRepr::RootLayerGroup
            | VLayerIdRepr::ScreenLayer
            | VLayerIdRepr::PageLayer
            | VLayerIdRepr::PlaneLayerGroup => {
                unreachable!("You can't unload special layers")
            }
            VLayerIdRepr::Selected => {
                todo!("LAYERUNLOAD: selected");
            }
            VLayerIdRepr::Layer(id) => {
                adv_state.root_layer_group.remove_layer(id);
            }
        }
        self.token.finish().into()
    }
}
