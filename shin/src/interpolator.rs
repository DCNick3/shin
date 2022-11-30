use std::f32::consts::PI;

const HALF_PI: f32 = PI / 2.0;
const TAU: f32 = 2.0 * PI;

pub struct Interpolator {
    t_now: f32,
    t_final: f32,
    y_0: f32,
    y_1: f32,
    y_current: f32,
    easing: Easing,
    queue: Vec<InterpolatorEvent>,
}

#[derive(Debug, Clone, Copy)]
enum Easing {
    Identity,
    SineIn,
    SinePingPong,
    SineInOut,
    Jump,
    Power(i32),
}

#[inline]
fn ease(easing: Easing, p: f32) -> f32 {
    match easing {
        Easing::Identity => p,
        Easing::SineIn => 1.0 - (HALF_PI * p).cos(),
        Easing::SinePingPong => (HALF_PI * p).sin(),
        Easing::SineInOut => (1.0 - (PI * p).cos()) / 2.0,
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

enum InterpolatorEvent {}

impl Interpolator {
    pub fn new() -> Self {
        Self {
            t_now: 0.0,
            t_final: 5.0,
            y_0: 0.0,
            y_1: 400.0,
            y_current: 0.0,
            easing: Easing::Power(-2),
            queue: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.t_now += dt;
        if self.t_now >= self.t_final {
            if let Some(_event) = self.queue.pop() {
                todo!()
            } else {
                self.t_now = self.t_final;
            }
        }

        let x = self.t_now / self.t_final;

        let y = ease(self.easing, x);
        let y = self.y_0 + (self.y_1 - self.y_0) * y;

        self.y_current = y;
    }

    pub fn value(&self) -> f32 {
        self.y_current
    }
}
