use shin_core::time::Ticks;
use shin_render::{
    render_pass::RenderPass, shaders::types::texture::TextureSource, RenderRequestBuilder,
};

use crate::{
    update::{AdvUpdatable, AdvUpdateContext},
    wiper::Wiper,
};

pub trait TimedWiper {
    fn render(
        &self,
        pass: &mut RenderPass,
        render_request_builder: RenderRequestBuilder,
        texture_target: TextureSource,
        texture_source: TextureSource,
        progress: f32,
    );
}

#[derive(Clone)]
struct TimedWiperState {
    current_time: Ticks,
    total_time: Ticks,
}

impl TimedWiperState {
    pub fn new(total_time: Ticks) -> Self {
        Self {
            current_time: Ticks::ZERO,
            total_time,
        }
    }

    pub fn update(&mut self, allow_running_animations: bool, delta_time: Ticks) {
        if allow_running_animations {
            self.current_time += delta_time;
            if self.current_time >= self.total_time {
                self.current_time = self.total_time;
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.current_time < self.total_time
    }

    pub fn fast_forward(&mut self) {
        self.current_time = self.total_time;
    }

    pub fn get_progress(&self) -> f32 {
        self.current_time / self.total_time
    }
}

#[derive(Clone)]
pub struct TimedWiperWrapper<T> {
    state: TimedWiperState,
    inner: T,
}

impl<T> TimedWiperWrapper<T> {
    pub fn from_inner(inner: T, total_time: Ticks) -> Self {
        Self {
            state: TimedWiperState::new(total_time),
            inner,
        }
    }
}

impl<T: AdvUpdatable> AdvUpdatable for TimedWiperWrapper<T> {
    fn update(&mut self, context: &AdvUpdateContext) {
        self.state
            .update(context.are_animations_allowed, context.delta_ticks);
        self.inner.update(context);
    }
}

impl<T: TimedWiper + AdvUpdatable> Wiper for TimedWiperWrapper<T> {
    fn is_running(&self) -> bool {
        self.state.is_running()
    }

    fn fast_forward(&mut self) {
        self.state.fast_forward()
    }

    fn render(
        &self,
        pass: &mut RenderPass,
        render_request_builder: RenderRequestBuilder,
        texture_target: TextureSource,
        texture_source: TextureSource,
    ) {
        self.inner.render(
            pass,
            render_request_builder,
            texture_target,
            texture_source,
            self.state.get_progress(),
        );
    }
}
