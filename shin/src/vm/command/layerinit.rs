use super::prelude::*;

impl super::StartableCommand for command::runtime::LAYERINIT {
    fn apply_state(&self, state: &mut VmState) {
        state
            .layers
            .get_vlayer_mut(self.layer_id)
            .for_each(|layer| layer.properties.init());
    }

    fn start(self, _vm_state: &VmState, _adv_state: &mut AdvState) -> CommandStartResult {
        self.token.finish().into()
    }
}
