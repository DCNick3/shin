mod command;
mod layouter;

use crate::layer::{Layer, LayerProperties};
use crate::render::{GpuCommonResources, Renderable};
use crate::update::{Updatable, UpdateContext};
use cgmath::Matrix4;
use shin_core::format::font::LazyFont;
use shin_core::vm::command::layer::{MessageTextLayout, MessageboxStyle};
use shin_core::vm::command::time::Ticks;

enum State {
    Hidden,
    Running,
    Waiting,
    Finished,
}

pub struct MessageLayer {
    props: LayerProperties,
    style: MessageboxStyle,
    running_time: Ticks,
    state: State,
    font: LazyFont,
}

impl MessageLayer {
    pub fn new(_resources: &GpuCommonResources, font: LazyFont) -> Self {
        Self {
            props: LayerProperties::new(),
            style: MessageboxStyle::default(),
            running_time: Ticks::ZERO,
            state: State::Hidden,
            font,
        }
    }

    pub fn set_style(&mut self, style: MessageboxStyle) {
        self.style = style;
    }

    pub fn set_message(&mut self, message: &str) {
        self.state = State::Running;
        self.running_time = Ticks::ZERO;

        let layout_params = shin_core::layout::LayoutParams {
            font: &self.font,
            layout_width: 1500.0,
            base_font_height: 50.0,
            font_horizontal_base_scale: 0.9696999788284302,
            text_layout: MessageTextLayout::Left,
            default_state: Default::default(),
            has_character_name: true,
        };

        let _layouted = shin_core::layout::layout_text(layout_params, message);
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.state, State::Finished)
    }
}

impl Renderable for MessageLayer {
    fn render<'enc>(
        &'enc self,
        _resources: &'enc GpuCommonResources,
        _render_pass: &mut wgpu::RenderPass<'enc>,
        _transform: Matrix4<f32>,
    ) {
    }

    fn resize(&mut self, _resources: &GpuCommonResources) {
        // no internal buffers to resize
    }
}

impl Updatable for MessageLayer {
    fn update(&mut self, ctx: &UpdateContext) {
        match self.state {
            State::Hidden => {}
            State::Running => {
                self.running_time += ctx.time_delta_ticks();
                if self.running_time >= Ticks::from_seconds(1.0) {
                    self.state = State::Finished;
                }
            }
            State::Waiting => {}
            State::Finished => {}
        }
    }
}

impl Layer for MessageLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
