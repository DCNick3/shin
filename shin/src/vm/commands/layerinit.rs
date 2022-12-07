use super::prelude::*;

pub struct LAYERINIT;

impl super::Command<command::runtime::LAYERINIT> for LAYERINIT {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::LAYERINIT, state: &mut VmState) {
        state
            .layers
            .get_vlayer_mut(command.layer_id)
            .for_each(|layer| layer.properties.init());
    }

    fn start(command: command::runtime::LAYERINIT, _vm: &mut Vm) -> Self::Result {
        command.token.finish()
    }
}
