use std::collections::VecDeque;
use std::f32::consts::PI;
use tracing::debug;

const HALF_PI: f32 = PI / 2.0;

pub struct Interpolator {
    t_now: f32,
    t_final: f32,
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

#[derive(Debug)]
enum InterpolatorEvent {
    QueueNext {
        value: f32,
        time: f32,
        easing: Easing,
    },
}

impl Interpolator {
    pub fn new(value: f32) -> Self {
        Self {
            t_now: 0.0,
            t_final: 0.0,
            y_0: value,
            y_1: value,
            y_current: value,
            easing: Easing::Identity,
            queue: VecDeque::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.t_now += dt;
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
                        self.t_now = 0.0;
                        self.t_final = time;
                        self.y_0 = self.y_1;
                        self.y_1 = value;
                    }
                }
            } else {
                self.t_now = self.t_final;
            }
        }

        let x = if self.t_final != 0.0 {
            self.t_now / self.t_final
        } else {
            0.0
        };

        let y = ease(self.easing, x);
        let y = self.y_0 + (self.y_1 - self.y_0) * y;

        self.y_current = y;
    }

    pub fn enqueue(&mut self, value: f32, time: f32, easing: Easing) {
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
