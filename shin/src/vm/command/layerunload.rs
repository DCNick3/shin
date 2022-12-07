use super::prelude::*;

impl super::StartableCommand for command::runtime::LAYERUNLOAD {
    fn apply_state(&self, state: &mut VmState) {
        match self.layer_id.repr() {
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

    fn start(self, _vm: &mut Vm) -> CommandStartResult {
        todo!("LAYERUNLOAD")
    }
}
