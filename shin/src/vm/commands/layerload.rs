use super::prelude::*;

pub struct LAYERLOAD;

impl super::Command<command::runtime::LAYERLOAD> for LAYERLOAD {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::LAYERLOAD, state: &mut VmState) {
        assert_eq!(command.leave_uninitialized, 0); // I __think__ this has to do with init props/leave them be, but I'm not sure

        match command.layer_id.repr() {
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

                layer.layerinit_params = Some((command.layer_type, command.params));
            }
        }
    }

    fn start(command: command::runtime::LAYERLOAD, vm: &mut Vm) -> Self::Result {
        todo!("LAYERLOAD")
        // command.token.finish()
    }
}
