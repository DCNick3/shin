use super::prelude::*;

impl super::StartableCommand for command::runtime::SEPLAY {
    fn apply_state(&self, state: &mut VmState) {
        warn!("TODO: SEPLAY state: {:?}", self);
    }

    fn start(self, _vm: &mut Vm) -> CommandStartResult {
        warn!("TODO: SEPLAY: {:?}", self);
        self.token.finish().into()
    }
}
