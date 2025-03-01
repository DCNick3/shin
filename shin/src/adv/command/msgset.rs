use std::fmt::{Debug, Formatter};

use super::prelude::*;
use crate::layer::message_layer::{MessageFlags, MsgsetParams};

pub struct MSGSET {
    #[allow(unused)]
    token: Option<command::token::MSGSET>,
}

impl StartableCommand for command::runtime::MSGSET {
    type StateInfo = ();
    fn apply_state(&self, state: &mut VmState) {
        // TODO: think about async messages (those where you would use MSGWAIT)
        state.messagebox_state.text = Some(self.text.clone());
        state.messagebox_state.messagebox_shown = true;
    }

    fn start(
        self,
        context: &mut UpdateContext,
        scenario: &Arc<Scenario>,
        vm_state: &VmState,
        _state_info: (),
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        adv_state.root_layer_group.message_layer_mut().on_msgset(
            context.pre_render,
            scenario,
            &self.text,
            MsgsetParams {
                flags: {
                    let mut flags = MessageFlags::empty();

                    // TODO: we currently don't track those in the vm state
                    // if vm_state.current_ev.is_some() {
                    //     flags |= MessageFlags::IGNORE_INPUT;
                    // }
                    // if vm_state.has_seen_current_message {
                    //     flags |= MessageFlags::UNUSED_FLAG;
                    // }

                    flags
                },
                messagebox_type: vm_state.messagebox_state.msginit.messagebox_type,
                text_layout: vm_state.messagebox_state.msginit.text_layout,
                message_id: self.msg_id,
            },
            true, // TODO: this needs to be adjusted for FF support
        );

        if self.auto_wait {
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
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
        // TODO: I think we __should__ somehow to react to fast-forwarding here
        // rn it's kludged in the ADV update function though
        _is_fast_forwarding: bool,
    ) -> Option<CommandResult> {
        if adv_state
            .root_layer_group
            .message_layer()
            .recv_sync_is_waiting(-1)
        {
            None
        } else {
            Some(self.token.take().unwrap().finish())
        }
    }
}

impl Debug for MSGSET {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MSGSET").finish()
    }
}
