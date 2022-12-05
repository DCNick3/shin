use crate::vm::Vm;
use shin_core::vm::command;
use shin_core::vm::command::layer::VLayerIdRepr;
use shin_core::vm::command::CommandResult;

pub struct LAYERINIT;

impl super::Command<command::runtime::LAYERINIT> for LAYERINIT {
    type Result = CommandResult;

    fn start(command: command::runtime::LAYERINIT, _vm: &mut Vm) -> Self::Result {
        match command.layer_id.repr() {
            VLayerIdRepr::Layer(id) => {
                todo!("LAYERINIT: layer {:?}", id);
            }
            VLayerIdRepr::Neg1 | VLayerIdRepr::Neg2 | VLayerIdRepr::Neg3 | VLayerIdRepr::Neg4 => {
                // ignore for now
            }
            VLayerIdRepr::Selected => {
                todo!("LAYERINIT: selected");
            }
        }
        command.token.finish()
    }
}
