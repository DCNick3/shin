use crate::vm::Vm;
use shin_core::vm::command;
use shin_core::vm::command::layer::VLayerIdRepr;
use shin_core::vm::command::CommandResult;

pub struct LAYERUNLOAD;

impl super::Command<command::runtime::LAYERUNLOAD> for LAYERUNLOAD {
    type Result = CommandResult;

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
