use std::time::Duration;

use enum_map::{enum_map, Enum, EnumMap};
use tracing::warn;

use crate::RawInputState;

#[derive(Default, Copy, Clone)]
struct ActionData60Fps {
    previous_state: bool,
    held_time: u8,
}

#[derive(Default, Copy, Clone)]
enum ActionDataDynamic {
    #[default]
    Released,
    Clicked {
        duration: Duration,
    },
    Repeating {
        duration_since_rapid_repeat: Duration,
        rapid_repeats: u32,
    },
}

pub enum ActionSignal {
    Held,
    Clicked,
    ClickedOrRepeated,
    ClickedOrRapidRepeated,
}

#[derive(Copy, Clone, Default)]
pub struct ActionState {
    /// The button is currently held down
    pub is_held: bool,
    /// The button was started being held down this frame
    pub is_clicked: bool,
    pub is_clicked_or_repeated: bool,
    pub is_clicked_or_rapid_repeated: bool,
}

impl ActionState {
    fn released() -> Self {
        Self {
            is_held: false,
            is_clicked: false,
            is_clicked_or_repeated: false,
            is_clicked_or_rapid_repeated: false,
        }
    }
    fn clicked() -> Self {
        Self {
            is_held: true,
            is_clicked: true,
            is_clicked_or_repeated: true,
            is_clicked_or_rapid_repeated: true,
        }
    }
    fn held() -> Self {
        Self {
            is_held: true,
            is_clicked: false,
            is_clicked_or_repeated: false,
            is_clicked_or_rapid_repeated: false,
        }
    }
    fn repeated() -> Self {
        Self {
            is_held: true,
            is_clicked: false,
            is_clicked_or_repeated: true,
            is_clicked_or_rapid_repeated: true,
        }
    }
    fn rapid_repeated() -> Self {
        Self {
            is_held: true,
            is_clicked: false,
            is_clicked_or_repeated: false,
            is_clicked_or_rapid_repeated: true,
        }
    }
}

impl ActionData60Fps {
    // this is tick-perfect reimplementation of shin's input handling algorithm
    // unfortunately, it won't work well with any tick rate different from 60/s
    // TODO: maybe we should guess when display is running at close to 60Hz and use this algorithm?
    // this will prevent time aliasing that the dynamic algo is prone to
    #[allow(unused)]
    pub fn update(&mut self, current_state: bool) -> ActionState {
        let is_held = current_state;
        let is_clicked = self.previous_state == false && current_state == true;

        let (is_clicked_or_repeated, is_clicked_or_rapid_repeated) = if !current_state {
            self.held_time = 0;

            (false, false)
        } else {
            let prev_held_time = self.held_time;
            // normal repeats starts after holding the button for 24 frames, triggering every 4th frame
            let is_clicked_or_repeated =
                prev_held_time == 0 || prev_held_time >= 24 && prev_held_time % 4 == 0;
            // rapid repeats starts after holding the button for 64 frames, triggering every other frame
            let is_clicked_or_rapid_repeated =
                is_clicked_or_repeated || prev_held_time >= 64 && prev_held_time % 2 == 0;

            if prev_held_time >= 68 {
                self.held_time = 64;
            } else {
                self.held_time += 1;
            }

            (is_clicked_or_repeated, is_clicked_or_rapid_repeated)
        };

        self.previous_state = current_state;

        ActionState {
            is_held,
            is_clicked,
            is_clicked_or_repeated,
            is_clicked_or_rapid_repeated,
        }
    }
}

impl ActionDataDynamic {
    // 24 ticks @ 60 tps
    const CLICKED_TO_REPEATING: Duration = Duration::from_nanos(24 * 1000000000 / 60);
    const RAPID_REPEAT_PERIOD: Duration = Duration::from_nanos(2 * 1000000000 / 60);
    const RAPID_REPEAT_START: u32 = 20;

    // TODO: duration's precision is probably excessive
    pub fn update(&mut self, current_state: bool, elapsed: Duration) -> ActionState {
        if !current_state {
            *self = ActionDataDynamic::Released;
            ActionState::released()
        } else {
            match self {
                ActionDataDynamic::Released => {
                    *self = ActionDataDynamic::Clicked {
                        duration: Duration::from_secs(0),
                    };
                    ActionState::clicked()
                }
                ActionDataDynamic::Clicked { duration } => {
                    *duration += elapsed;

                    if *duration >= Self::CLICKED_TO_REPEATING {
                        *self = ActionDataDynamic::Repeating {
                            duration_since_rapid_repeat: Duration::from_secs(0),
                            rapid_repeats: 0,
                        };

                        ActionState::repeated()
                    } else {
                        ActionState::held()
                    }
                }
                ActionDataDynamic::Repeating {
                    duration_since_rapid_repeat,
                    rapid_repeats,
                } => {
                    *duration_since_rapid_repeat += elapsed;

                    if *duration_since_rapid_repeat >= Self::RAPID_REPEAT_PERIOD {
                        // skip repeats if we are running too slow
                        let mut new_repeats = 0;
                        while *duration_since_rapid_repeat >= Self::RAPID_REPEAT_PERIOD {
                            *duration_since_rapid_repeat -= Self::RAPID_REPEAT_PERIOD;
                            new_repeats += 1;
                        }
                        if new_repeats > 1 {
                            warn!(
                                "Running too slow, skipped {} rapid repeats",
                                new_repeats - 1
                            );
                        }
                        *rapid_repeats += 1;

                        if *rapid_repeats % 2 == 0 {
                            ActionState::repeated()
                        } else if *rapid_repeats >= Self::RAPID_REPEAT_START {
                            ActionState::rapid_repeated()
                        } else {
                            ActionState::held()
                        }
                    } else {
                        ActionState::held()
                    }
                }
            }
        }
    }
}

pub struct ActionsState<A: Enum> {
    actions_data: EnumMap<A, ActionDataDynamic>,
}

impl<A: Enum> ActionsState<A> {
    pub fn new() -> Self {
        Self {
            actions_data: enum_map! { _ => ActionDataDynamic::default() },
        }
    }

    pub fn update(
        &mut self,
        current_state: EnumMap<A, bool>,
        elapsed: Duration,
    ) -> EnumMap<A, ActionState> {
        self.actions_data
            .iter_mut()
            .zip(current_state.iter())
            .map(|((action, action_data), (_, current_state))| {
                (action, action_data.update(*current_state, elapsed))
            })
            .collect()
    }
}

pub trait Action: Enum {
    fn lower(raw_input_state: &RawInputState) -> EnumMap<Self, bool>;
}
