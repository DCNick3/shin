use super::prelude::*;

pub struct MSGSET {
    #[allow(unused)]
    token: Option<command::token::MSGSET>,
}

impl super::StartableCommand for command::runtime::MSGSET {
    fn apply_state(&self, state: &mut VmState) {
        // TODO: think about async messages (those where you would use MSGWAIT)
        state.messagebox_state.text = Some(self.text.clone());
        state.messagebox_state.messagebox_shown = true;
    }

    fn start(
        self,
        context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        assert!(matches!(self.auto_wait, 0 | 1));

        adv_state
            .root_layer_group
            .message_layer_mut()
            .set_message(context, &self.text);

        if self.auto_wait == 1 {
            Yield(
                MSGSET {
                    token: Some(self.token),
                }
                .into(),
            )
        } else {
            self.token.finish().into()
        }
    }
}

impl UpdatableCommand for MSGSET {
    fn update(
        &mut self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> Option<CommandResult> {
        if adv_state.root_layer_group.message_layer().is_finished() {
            Some(self.token.take().unwrap().finish())
        } else {
            None
        }
    }
}

// pub fn system(mut _commands: Commands, mut query: Query<(Entity, &mut MSGSET)>) {
//     for (_entity, mut _wait) in query.iter_mut() {
//         // TODO: here we do not finish the command, making the VM wait forever
//     }
// }
