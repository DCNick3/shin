use shin_core::vm::command::types::WipeFlags;
use shin_tasks::AsyncTask;

use super::prelude::*;
use crate::wiper::AnyWiper;

#[derive(Debug)]
enum WipeState {
    WaitingForWipeLoader {
        load_task: Option<AsyncTask<AnyWiper>>,
    },
    WaitingForWipe,
    Finished,
}

#[derive(Debug)]
pub struct WIPE {
    token: Option<command::token::WIPE>,
    flags: WipeFlags,
    state: WipeState,
}

impl StartableCommand for command::runtime::WIPE {
    type StateInfo = bool;
    fn apply_state(&self, state: &mut VmState) -> bool {
        if state.layers.is_page_back_started {
            state.layers.is_page_back_started = false;

            true
        } else {
            false
        }
    }

    fn start(
        self,
        context: &mut UpdateContext,
        scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        needs_wipe: bool,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        if !needs_wipe {
            return self.token.finish().into();
        }

        let load_task = if self.duration > Ticks::ZERO {
            let asset_server = context.asset_server.clone();
            let scenario = scenario.clone();
            Some(shin_tasks::async_io::spawn(async move {
                AnyWiper::load(
                    &asset_server,
                    &scenario,
                    self.ty,
                    self.duration,
                    self.params,
                )
                .await
            }))
        } else {
            None
        };

        if !self.flags.contains(WipeFlags::DONT_BLOCK_ANIMATIONS) {
            adv_state.allow_running_animations = false;
        }

        // TODO: optimistically block for 5ms as the game does

        Yield(
            WIPE {
                token: Some(self.token),
                flags: self.flags,
                state: WipeState::WaitingForWipeLoader { load_task },
            }
            .into(),
        )
    }
}

impl UpdatableCommand for WIPE {
    fn update(
        &mut self,
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
        _is_fast_forwarding: bool,
    ) -> Option<CommandResult> {
        // TODO: state machines like this can be useful in multiple commands
        // it would be nice to have a generic abstraction for this
        let mut break_the_loop = false;

        while !break_the_loop {
            replace_with::replace_with(
                &mut self.state,
                || WipeState::Finished,
                |state| match state {
                    WipeState::WaitingForWipeLoader { load_task } => match load_task {
                        Some(mut load_task) => {
                            if let Some(wiper) = load_task.poll_naive() {
                                adv_state.screen_layer_mut().apply_transition(Some(wiper));
                                adv_state.allow_running_animations = true;

                                WipeState::WaitingForWipe
                            } else {
                                break_the_loop = true;
                                WipeState::WaitingForWipeLoader {
                                    load_task: Some(load_task),
                                }
                            }
                        }
                        None => {
                            adv_state.screen_layer_mut().apply_transition(None);
                            adv_state.allow_running_animations = true;

                            WipeState::WaitingForWipe
                        }
                        load_task @ Some(_) => {
                            break_the_loop = true;
                            WipeState::WaitingForWipeLoader { load_task }
                        }
                    },
                    WipeState::WaitingForWipe => {
                        if self.flags.contains(WipeFlags::DONT_WAIT)
                            || !adv_state.screen_layer().is_transition_active()
                        {
                            WipeState::Finished
                        } else {
                            break_the_loop = true;
                            WipeState::WaitingForWipe
                        }
                    }
                    WipeState::Finished => unreachable!(),
                },
            );

            if let WipeState::Finished = &self.state {
                return self.token.take().unwrap().finish().into();
            }
        }

        None
    }
}
