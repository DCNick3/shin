use crate::vm::Vm;
use shin_core::vm::command;
use shin_core::vm::command::layer::VLayerIdRepr;
use shin_core::vm::command::CommandResult;

pub struct LAYERCTRL;

impl super::Command<command::runtime::LAYERCTRL> for LAYERCTRL {
    type Result = CommandResult;

    fn start(command: command::runtime::LAYERCTRL, vm: &mut Vm) -> Self::Result {
        // TODO: handle the delay?
        match command.layer_id.repr() {
            r @ VLayerIdRepr::Neg1
            | r @ VLayerIdRepr::Neg2
            | r @ VLayerIdRepr::Neg3
            | r @ VLayerIdRepr::Neg4 => {
                todo!("LAYERCTRL: {:?}", r);
            }
            VLayerIdRepr::Selected => {
                todo!("LAYERCTRL: selected");
            }
            VLayerIdRepr::Layer(id) => {
                // TODO: this is done in some different way in the game...
                // they have a function that can iterate over range of layer ids (if they are present) and they are using that?
                let layerbank = vm
                    .state
                    .layerbank_allocator
                    .get_layerbank_id(id)
                    .expect("failed to get layerbank id");
                // TODO: this might be useful as a function
                let info = &mut vm.state.layerbank_info[layerbank.raw() as usize];
                if let Some(old) = info {
                    old.properties[command.property_id as usize] = command.params[0];
                } else {
                    panic!("LAYERCTRL: layerbank not loaded");
                }
            }
        }
        command.token.finish()
    }
}
