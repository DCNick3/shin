use super::prelude::*;

pub struct LAYERUNLOAD;

impl super::Command<command::runtime::LAYERUNLOAD> for LAYERUNLOAD {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::LAYERUNLOAD, state: &mut VmState) {
        todo!()
    }

    fn start(command: command::runtime::LAYERUNLOAD, vm: &mut Vm) -> Self::Result {
        // TODO: handle the delay?
        match command.layer_id.repr() {
            VLayerIdRepr::Neg1 | VLayerIdRepr::Neg2 | VLayerIdRepr::Neg3 | VLayerIdRepr::Neg4 => {
                unreachable!()
            }
            VLayerIdRepr::Selected => {
                todo!("LAYERUNLOAD: selected");
            }
            VLayerIdRepr::Layer(id) => {
                vm.state.layerbank_allocator.free_layerbank(id);
                // TODO: handle all the on-screen stuff
            }
        }
        command.token.finish()
    }
}
