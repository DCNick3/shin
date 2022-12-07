use super::prelude::*;

pub struct LAYERCTRL;

impl super::Command<command::runtime::LAYERCTRL> for LAYERCTRL {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::LAYERCTRL, state: &mut VmState) {
        let [target_value, time, flags, _, _, _, _, _] = command.params;

        state
            .layers
            .get_vlayer_mut(command.layer_id)
            .for_each(|layer| {
                layer
                    .properties
                    .set_property(command.property_id, target_value);
            });
    }

    fn start(_command: command::runtime::LAYERCTRL, _vm: &mut Vm) -> Self::Result {
        todo!()
        // command.token.finish()
    }
}
