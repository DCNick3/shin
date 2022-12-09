use super::prelude::*;

impl super::StartableCommand for command::runtime::LAYERINIT {
    fn apply_state(&self, state: &mut VmState) {
        state
            .layers
            .get_vlayer_mut(self.layer_id)
            .for_each(|layer| layer.properties.init());
    }

    fn start(
        self,
        _context: &UpdateContext,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        adv_state
            .root_layer_group
            .get_vlayer_mut(self.layer_id)
            .for_each(|layer| layer.properties_mut().init());
        self.token.finish().into()
    }
}
