use super::prelude::*;

impl super::StartableCommand for command::runtime::SESTOPALL {
    fn apply_state(&self, state: &mut VmState) {
        warn!("TODO: SESTOPALL state: {:?}", self);
    }

    fn start(self, _vm_state: &VmState, _adv_state: &mut AdvState) -> CommandStartResult {
        warn!("TODO: SESTOPALL: {:?}", self);
        self.token.finish().into()
    }
}
