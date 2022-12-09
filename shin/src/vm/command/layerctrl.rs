use super::prelude::*;

impl super::StartableCommand for command::runtime::LAYERCTRL {
    fn apply_state(&self, state: &mut VmState) {
        let [target_value, _time, _flags, _, _, _, _, _] = self.params;

        state
            .layers
            .get_vlayer_mut(self.layer_id)
            .for_each(|layer| {
                layer
                    .properties
                    .set_property(self.property_id, target_value);
            });
    }

    fn start(
        self,
        _context: &UpdateContext,
        _vm_state: &VmState,
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        todo!()
        // command.token.finish().into()
    }
}
