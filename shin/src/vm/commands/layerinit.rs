use super::prelude::*;

pub struct LAYERINIT;

impl super::Command<command::runtime::LAYERINIT> for LAYERINIT {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::LAYERINIT, state: &mut VmState) {
        todo!()
    }

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
