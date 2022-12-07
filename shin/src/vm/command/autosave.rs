use super::prelude::*;

impl super::StartableCommand for command::runtime::AUTOSAVE {
    fn apply_state(&self, _state: &mut VmState) {
        // nothing to do
    }

    fn start(self, _vm: &mut Vm) -> CommandStartResult {
        warn!("TODO: AUTOSAVE: {:?}", self);
        self.token.finish().into()
    }
}
