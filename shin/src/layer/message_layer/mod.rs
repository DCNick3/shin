mod font_atlas;

use crate::layer::message_layer::font_atlas::FontAtlas;
use crate::layer::{Layer, LayerProperties};
use crate::render::dynamic_atlas::AtlasImage;
use crate::render::{GpuCommonResources, Renderable, TextVertex, VertexBuffer};
use crate::update::{Updatable, UpdateContext};
use cgmath::{ElementWise, Matrix4, Vector2};
use shin_core::format::font::{GlyphTrait, LazyFont};
use shin_core::layout::Command;
use shin_core::vm::command::layer::{MessageTextLayout, MessageboxStyle};
use shin_core::vm::command::time::Ticks;
use std::sync::{Arc, Mutex};

enum State {
    Hidden,
    Running,
    Waiting,
    Finished,
}

struct Message {
    font_atlas: Arc<Mutex<FontAtlas>>,
    commands: Vec<Command>,
    vertex_buffer: VertexBuffer<TextVertex>,
}

impl Message {
    pub fn new(
        context: &UpdateContext,
        font_atlas: &Arc<Mutex<FontAtlas>>,
        base_position: Vector2<f32>,
        message: &str,
    ) -> Self {
        let mut font_atlas_guard = font_atlas.lock().unwrap();

        let layout_params = shin_core::layout::LayoutParams {
            font: font_atlas_guard.get_font(),
            layout_width: 1500.0,
            base_font_height: 50.0,
            font_horizontal_base_scale: 0.9696999788284302,
            text_layout: MessageTextLayout::Left,
            default_state: Default::default(),
            has_character_name: true,
        };

        let commands = shin_core::layout::layout_text(layout_params, message);

        let mut vertices = Vec::new();
        for command in commands.iter() {
            match command {
                Command::Char(char) => {
                    let glyph_info = font_atlas_guard
                        .get_font()
                        .get_glyph_for_character(char.codepoint)
                        .get_info();

                    let atlas_size = font_atlas_guard.texture_size();
                    let atlas_size = Vector2::new(atlas_size.0 as f32, atlas_size.1 as f32);

                    let AtlasImage {
                        position: tex_position,
                        size: tex_size,
                    } = font_atlas_guard.get_image(context.gpu_resources, char.codepoint);

                    // we don't actually want to use the full size of the glyph texture
                    //   because they are padded to be a power of 2
                    //   so we need to scale the texture coordinates to the actual size of the glyph
                    let tex_size = tex_size.mul_element_wise(glyph_info.actual_size_relative());

                    // scale texture coordinates to the size of the texture
                    let tex_position = tex_position.div_element_wise(atlas_size);
                    let tex_size = tex_size.div_element_wise(atlas_size);

                    let position = base_position
                        + char.position
                        + Vector2::new(
                            glyph_info.bearing_x as f32 * char.size.horizontal_scale,
                            -glyph_info.bearing_y as f32 * char.size.scale,
                        );
                    let size = char.size.size();

                    let time = char.time.0;
                    let fade = char.fade;

                    vertices.extend([
                        TextVertex {
                            position: position + Vector2::new(0.0, 0.0),
                            tex_position: tex_position + Vector2::new(0.0, 0.0),
                            time,
                            fade,
                        },
                        TextVertex {
                            position: position + Vector2::new(size.x, 0.0),
                            tex_position: tex_position + Vector2::new(tex_size.x, 0.0),
                            time,
                            fade,
                        },
                        TextVertex {
                            position: position + Vector2::new(0.0, size.y),
                            tex_position: tex_position + Vector2::new(0.0, tex_size.y),
                            time,
                            fade,
                        },
                        TextVertex {
                            position: position + Vector2::new(size.x, size.y),
                            tex_position: tex_position + Vector2::new(tex_size.x, tex_size.y),
                            time,
                            fade,
                        },
                        TextVertex {
                            position: position + Vector2::new(0.0, size.y),
                            tex_position: tex_position + Vector2::new(0.0, tex_size.y),
                            time,
                            fade,
                        },
                        TextVertex {
                            position: position + Vector2::new(size.x, 0.0),
                            tex_position: tex_position + Vector2::new(tex_size.x, 0.0),
                            time,
                            fade,
                        },
                    ]);
                }
            }
        }

        let vertex_buffer = VertexBuffer::new(
            context.gpu_resources,
            dbg!(&vertices),
            Some("Message VertexBuffer"),
        );

        Self {
            font_atlas: font_atlas.clone(),
            commands,
            vertex_buffer,
        }
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        let mut font_atlas = self.font_atlas.lock().unwrap();
        for command in self.commands.iter() {
            match command {
                Command::Char(char) => {
                    font_atlas.free_image(char.codepoint);
                }
            }
        }
    }
}

pub struct MessageLayer {
    props: LayerProperties,
    style: MessageboxStyle,
    running_time: Ticks,
    state: State,
    font_atlas: Arc<Mutex<FontAtlas>>,
}

impl MessageLayer {
    pub fn new(resources: &GpuCommonResources, font: LazyFont) -> Self {
        Self {
            props: LayerProperties::new(),
            style: MessageboxStyle::default(),
            running_time: Ticks::ZERO,
            state: State::Hidden,
            font_atlas: Arc::new(Mutex::new(FontAtlas::new(resources, font))),
        }
    }

    pub fn set_style(&mut self, style: MessageboxStyle) {
        self.style = style;
    }

    pub fn set_message(&mut self, context: &UpdateContext, message: &str) {
        self.state = State::Running;
        self.running_time = Ticks::ZERO;

        let _message = Message::new(
            context,
            &self.font_atlas,
            Vector2::new(-900.0, -300.0),
            message,
        );
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
