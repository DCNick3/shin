use super::prelude::*;
use tracing::debug;

impl StartableCommand for command::runtime::DEBUGOUT {
    fn apply_state(&self, _state: &mut VmState) {}

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        debug!("DEBUGOUT: {} {:?}", self.format, self.args);
        self.token.finish().into()
    }
}
