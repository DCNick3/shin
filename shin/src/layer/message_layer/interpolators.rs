//! MessageLayer-specific interpolators.

use shin_core::time::Ticks;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SlideInterpolatorDirection {
    Increasing,
    Decreasing,
}

impl SlideInterpolatorDirection {
    pub fn from_is_increasing(is_increasing: bool) -> Self {
        if is_increasing {
            Self::Increasing
        } else {
            Self::Decreasing
        }
    }

    pub fn as_f32(&self) -> f32 {
        match self {
            Self::Increasing => 1.0,
            Self::Decreasing => -1.0,
        }
    }
}

/// An interpolator that is used to animate messagebox sliding in/out and its opacity.
///
/// Clamps the value between 0.0 and 1.0 and stores the direction of the interpolation
#[derive(Debug, Copy, Clone)]
pub struct SlideInterpolator {
    current_direction: SlideInterpolatorDirection,
    value: f32,
}

impl SlideInterpolator {
    const RATE_PER_TICK: f32 = 0.1;

    #[inline]
    pub fn new(value: f32, direction: SlideInterpolatorDirection) -> Self {
        Self {
            value,
            current_direction: direction,
        }
    }

    pub fn set_direction(&mut self, direction: SlideInterpolatorDirection) {
        self.current_direction = direction;
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    #[inline]
    pub fn update(&mut self, delta_ticks: Ticks) -> f32 {
        let delta = delta_ticks.as_f32() * Self::RATE_PER_TICK;
        self.value = (self.value + delta * self.current_direction.as_f32()).clamp(0.0, 1.0);

        self.value
    }

    pub fn direction(&self) -> SlideInterpolatorDirection {
        self.current_direction
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn is_fully_at(&self, direction: SlideInterpolatorDirection) -> bool {
        if self.current_direction != direction {
            return false;
        }
        match direction {
            SlideInterpolatorDirection::Increasing => self.value >= 1.0,
            SlideInterpolatorDirection::Decreasing => self.value <= 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct HeightInterpolator {
    target: f32,
    value: f32,
}

impl HeightInterpolator {
    const RATE_PER_TICK: f32 = 18.0;

    pub fn new(value: f32) -> Self {
        Self {
            target: value,
            value,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn set_min_target(&mut self, target: f32) {
        self.target = self.target.max(target);
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    #[inline]
    pub fn update(&mut self, delta_ticks: Ticks) {
        let delta = delta_ticks.as_f32() * Self::RATE_PER_TICK;

        match self.target.partial_cmp(&self.value).unwrap() {
            std::cmp::Ordering::Less => {
                self.value = (self.value - delta).max(self.target);
            }
            std::cmp::Ordering::Greater => {
                self.value = (self.value + delta).min(self.target);
            }
            std::cmp::Ordering::Equal => {}
        }
    }

    pub fn is_interpolating(&self) -> bool {
        self.value != self.target
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Countdown {
    time_left: f32,
}

impl Countdown {
    const RATE_PER_TICK: f32 = Ticks::SECONDS_PER_TICK;

    pub fn new(time_left: f32) -> Self {
        Self { time_left }
    }

    pub fn set_time_left(&mut self, time_left: f32) {
        self.time_left = time_left;
    }

    pub fn is_done(&self) -> bool {
        self.time_left <= 0.0
    }

    pub fn update(&mut self, delta_ticks: Ticks) -> bool {
        if self.time_left > 0.0 {
            self.time_left -= delta_ticks.as_f32() * Self::RATE_PER_TICK;
        }
        self.is_done()
    }
}
