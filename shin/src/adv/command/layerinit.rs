use super::prelude::*;

impl super::StartableCommand for command::runtime::LAYERINIT {
    fn apply_state(&self, state: &mut VmState) {
        state
            .layers
            .for_each_vlayer_mut(self.layer_id, |layer| layer.properties.init());
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Scenario,
        vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        adv_state.for_each_vlayer_mut(vm_state, self.layer_id, |mut layer| {
            layer.properties_mut().init()
        });
        self.token.finish().into()
    }
}
