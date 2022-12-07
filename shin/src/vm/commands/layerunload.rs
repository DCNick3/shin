use super::prelude::*;

pub struct LAYERUNLOAD;

impl super::Command<command::runtime::LAYERUNLOAD> for LAYERUNLOAD {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::LAYERUNLOAD, state: &mut VmState) {
        match command.layer_id.repr() {
            VLayerIdRepr::Neg1 | VLayerIdRepr::Neg2 | VLayerIdRepr::Neg3 | VLayerIdRepr::Neg4 => {
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

    fn start(command: command::runtime::LAYERUNLOAD, vm: &mut Vm) -> Self::Result {
        todo!("LAYERUNLOAD")
    }
}
