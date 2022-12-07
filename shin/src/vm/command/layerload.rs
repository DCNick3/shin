use super::prelude::*;

impl super::StartableCommand for command::runtime::LAYERLOAD {
    fn apply_state(&self, state: &mut VmState) {
        assert_eq!(self.leave_uninitialized, 0); // I __think__ this has to do with init props/leave them be, but I'm not sure

        match self.layer_id.repr() {
            VLayerIdRepr::Neg1 | VLayerIdRepr::Neg2 | VLayerIdRepr::Neg3 | VLayerIdRepr::Neg4 => {
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

    fn start(self, vm: &mut Vm) -> CommandStartResult {
        todo!("LAYERLOAD")
        // command.token.finish().into()
    }
}
