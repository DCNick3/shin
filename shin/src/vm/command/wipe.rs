use super::prelude::*;

impl super::StartableCommand for command::runtime::WIPE {
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: WIPE state: {:?}", self);
        // we don't track wipes yet
    }

    fn start(self, _vm: &mut Vm) -> CommandStartResult {
        warn!("TODO: WIPE: {:?}", self);
        self.token.finish().into()
    }
}
