use super::prelude::*;

impl super::StartableCommand for command::runtime::SAVEINFO {
    fn apply_state(&self, state: &mut VmState) {
        state.save_info.set_save_info(self.level, self.info.clone());
    }

    fn start(self, _vm: &mut Vm) -> CommandStartResult {
        self.token.finish().into()
    }
}
