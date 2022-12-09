use super::prelude::*;

impl super::StartableCommand for command::runtime::BGMPLAY {
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: BGMPLAY state: {:?}", self);
    }

    fn start(
        self,
        _context: &UpdateContext,
        _vm_state: &VmState,
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        warn!("TODO: BGMPLAY: {:?}", self);
        self.token.finish().into()
    }
}
