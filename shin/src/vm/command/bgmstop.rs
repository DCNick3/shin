use super::prelude::*;

impl super::StartableCommand for command::runtime::BGMSTOP {
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: BGMSTOP state: {:?}", self);
    }

    fn start(self, _vm_state: &VmState, _adv_state: &mut AdvState) -> CommandStartResult {
        warn!("TODO: BGMSTOP: {:?}", self);
        self.token.finish().into()
    }
}