use crate::vm::state::LayerbankInfo;
use crate::vm::Vm;
use shin_core::vm::command;
use shin_core::vm::command::layer::VLayerIdRepr;
use shin_core::vm::command::CommandResult;

pub struct LAYERLOAD;

impl super::Command<command::runtime::LAYERLOAD> for LAYERLOAD {
    type Result = CommandResult;

    fn start(command: command::runtime::LAYERLOAD, vm: &mut Vm) -> Self::Result {
        // TODO: handle the delay?
        match command.layer_id.repr() {
            VLayerIdRepr::Neg1 | VLayerIdRepr::Neg2 | VLayerIdRepr::Neg3 | VLayerIdRepr::Neg4 => {
                unreachable!()
            }
            VLayerIdRepr::Selected => {
                todo!("LAYERLOAD: selected");
            }
            VLayerIdRepr::Layer(id) => {
                let layerbank = vm
                    .state
                    .layerbank_allocator
                    .get_or_allocate_layerbank_id(id)
                    .expect("layerbank allocation failed");
                let info = &mut vm.state.layerbank_info[layerbank.raw() as usize];
                if let Some(old) = info {
                    if old.ty == command.layer_type && old.layerinit_params == command.params {
                        // nothing to do (yet)
                        // TODO: the game has slightly different logic, setting some flags? resetting the properties?
                    } else {
                        *old = LayerbankInfo {
                            ty: command.layer_type,
                            layer_id: id,
                            layerinit_params: command.params,
                            properties: [0; 90],
                        };
                    }
                } else {
                    *info = Some(LayerbankInfo {
                        ty: command.layer_type,
                        layer_id: id,
                        layerinit_params: command.params,
                        properties: [0; 90], // TODO: use proper values
                    });
                }
            }
        }
        command.token.finish()
    }
}
