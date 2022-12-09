use super::prelude::*;

pub struct MSGSET {
    #[allow(unused)]
    token: Option<command::token::MSGSET>,
}

impl super::StartableCommand for command::runtime::MSGSET {
    fn apply_state(&self, _state: &mut VmState) {
        warn!("TODO: MSGSET state: {:?}", self)
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Scenario,
        _vm_state: &VmState,
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        Yield(
            MSGSET {
                token: Some(self.token),
            }
            .into(),
        )
    }
}

impl super::UpdatableCommand for MSGSET {
    fn update(
        &mut self,
        _context: &UpdateContext,
        _scenario: &Scenario,
        _vm_state: &VmState,
        _adv_state: &mut AdvState,
    ) -> Option<CommandResult> {
        // TODO: do something
        // Some(self.token.take().unwrap().finish())
        None
    }
}

// pub fn system(mut _commands: Commands, mut query: Query<(Entity, &mut MSGSET)>) {
//     for (_entity, mut _wait) in query.iter_mut() {
//         // TODO: here we do not finish the command, making the VM wait forever
//     }
// }
