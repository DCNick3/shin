use crate::update::UpdateContext;
use shin_core::time::Ticks;
use std::collections::VecDeque;
use std::f32::consts::PI;
use tracing::debug;

const HALF_PI: f32 = PI / 2.0;

// TODO: custom Debug
#[derive(Debug, Clone)]
pub struct Interpolator {
    t_now: Ticks,
    t_final: Ticks,
    y_0: f32,
    y_1: f32,
    y_current: f32,
    easing: Easing,
    queue: VecDeque<InterpolatorEvent>,
}

#[derive(Debug, Clone, Copy)]
pub enum Easing {
    Identity,
    EaseIn,
    EaseOut,
    EaseInOut,
    Jump,
    Power(i32),
}

#[inline]
fn ease(easing: Easing, p: f32) -> f32 {
    match easing {
        Easing::Identity => p,
        Easing::EaseIn => 1.0 - (HALF_PI * p).cos(),
        Easing::EaseOut => (HALF_PI * p).sin(),
        Easing::EaseInOut => (1.0 - (PI * p).cos()) / 2.0,
        Easing::Jump => {
            if p <= 1.0 {
                0.0
            } else {
                1.0
            }
        }
        Easing::Power(power) => {
            if power > 0 {
                p.powi(power)
            } else if power != 0 {
                1.0 - (1.0 - p).powi(-power)
            } else {
                p
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum InterpolatorEvent {
    QueueNext {
        value: f32,
        time: Ticks,
        easing: Easing,
    },
}

impl Interpolator {
    pub fn new(value: f32) -> Self {
        Self {
            t_now: Ticks::ZERO,
            t_final: Ticks::ZERO,
            y_0: value,
            y_1: value,
            y_current: value,
            easing: Easing::Identity,
            queue: VecDeque::new(),
        }
    }

    pub fn update(&mut self, context: &UpdateContext) {
        self.t_now += context.time_delta_ticks();
        if self.t_now >= self.t_final {
            if let Some(event) = self.queue.pop_back() {
                debug!("Switch: {:?}", event);
                match event {
                    InterpolatorEvent::QueueNext {
                        value,
                        time,
                        easing,
                    } => {
                        self.easing = easing;
                        self.t_now = Ticks::ZERO;
                        self.t_final = time;
                        self.y_0 = self.y_1;
                        self.y_1 = value;
                    }
                }
            } else {
                self.t_now = self.t_final;
            }
        }

        let x = if self.t_final != Ticks::ZERO {
            self.t_now / self.t_final
        } else {
            1.0
        };

        let y = ease(self.easing, x);
        let y = self.y_0 + (self.y_1 - self.y_0) * y;

        self.y_current = y;
    }

    pub fn enqueue_force(&mut self, value: f32) {
        self.queue.clear();
        self.y_0 = value;
        self.y_1 = value;
        self.y_current = value;
        self.t_now = Ticks::ZERO;
        self.t_final = Ticks::ZERO;
    }

    pub fn enqueue(&mut self, value: f32, time: Ticks, easing: Easing) {
        self.queue.push_front(InterpolatorEvent::QueueNext {
            value,
            time,
            easing,
        })
    }

    pub fn value(&self) -> f32 {
        self.y_current
    }
}
