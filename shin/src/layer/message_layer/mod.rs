mod font_atlas;
mod messagebox;

pub use messagebox::MessageboxTextures;
use std::sync::Arc;

use crate::adv::assets::AdvFonts;
use crate::layer::message_layer::font_atlas::FontAtlas;
use crate::layer::message_layer::messagebox::Messagebox;
use crate::layer::{Layer, LayerProperties};
use crate::render::dynamic_atlas::AtlasImage;
use crate::render::{GpuCommonResources, Renderable, TextVertex, VertexBuffer};
use crate::update::{Updatable, UpdateContext};
use cgmath::{ElementWise, Matrix4, Vector2};
use shin_core::format::font::GlyphTrait;
use shin_core::layout::Command;
use shin_core::vm::command::layer::{MessageTextLayout, MessageboxStyle};
use shin_core::vm::command::time::Ticks;

struct Message {
    time: Ticks,
    complete_time: Ticks,
    font_atlas: FontAtlas,
    commands: Vec<Command>,
    vertex_buffer: VertexBuffer<TextVertex>,
}

impl Message {
    pub fn new(
        context: &UpdateContext,
        mut font_atlas: FontAtlas,
        base_position: Vector2<f32>,
        message: &str,
    ) -> Self {
        // let mut font_atlas_guard = font_atlas.lock().unwrap();

        let layout_params = shin_core::layout::LayoutParams {
            font: font_atlas.get_font(),
            layout_width: 1500.0,
            base_font_height: 50.0,
            furigana_font_height: 20.0,
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
                    let glyph_info = font_atlas
                        .get_font()
                        .get_glyph_for_character(char.codepoint)
                        .get_info();

                    let atlas_size = font_atlas.texture_size();
                    let atlas_size = Vector2::new(atlas_size.0 as f32, atlas_size.1 as f32);

                    let AtlasImage {
                        position: tex_position,
                        size: _, // the atlas size is not to be trusted, as it can be larger than the actual texture (even larger than the power of 2 padded texture...)
                    } = font_atlas.get_image(context.gpu_resources, char.codepoint);

                    // just use the actual size of the glyph
                    let tex_size = glyph_info.actual_size();
                    let tex_size = Vector2::new(tex_size.0 as f32, tex_size.1 as f32);

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
                    let color = char.color;

                    // TODO: do the fade calculation here

                    // helper macro to reduce vertex creation boilerplate
                    macro_rules! v {
                        (($x:expr, $y:expr), ($tex_x:expr, $tex_y:expr)) => {
                            TextVertex {
                                position: position + Vector2::new($x, $y),
                                tex_position: tex_position + Vector2::new($tex_x, $tex_y),
                                color,
                                time,
                                fade,
                            }
                        };
                    }

                    vertices.extend([
                        // Top left triangle
                        v!((0.0, 0.0), (0.0, 0.0)),
                        v!((size.x, 0.0), (tex_size.x, 0.0)),
                        v!((0.0, size.y), (0.0, tex_size.y)),
                        // Bottom right triangle
                        v!((size.x, size.y), (tex_size.x, tex_size.y)),
                        v!((0.0, size.y), (0.0, tex_size.y)),
                        v!((size.x, 0.0), (tex_size.x, 0.0)),
                    ]);
                }
            }
        }

        let vertex_buffer = VertexBuffer::new(
            context.gpu_resources,
            &vertices,
            Some("Message VertexBuffer"),
        );

        let complete_time = commands
            .iter()
            .map(|v| v.time())
            .max()
            .unwrap_or(Ticks::ZERO)
            + Ticks::from_seconds(2.0);

        Self {
            time: Ticks::ZERO,
            complete_time,
            font_atlas,
            commands,
            vertex_buffer,
        }
    }

    pub fn complete(&self) -> bool {
        self.time >= self.complete_time
    }
}

impl Updatable for Message {
    fn update(&mut self, context: &UpdateContext) {
        self.time += context.time_delta_ticks();
    }
}

impl Renderable for Message {
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
        transform: Matrix4<f32>,
    ) {
        resources.draw_text(
            render_pass,
            self.vertex_buffer.vertex_source(),
            self.font_atlas.texture_bind_group(),
            transform,
            self.time.0,
        );
    }

    fn resize(&mut self, _resources: &GpuCommonResources) {}
}

impl Drop for Message {
    fn drop(&mut self) {
        for command in self.commands.iter() {
            match command {
                Command::Char(char) => {
                    self.font_atlas.free_image(char.codepoint);
                }
            }
        }
    }
}

pub struct MessageLayer {
    props: LayerProperties,
    style: MessageboxStyle,
    running_time: Ticks,
    fonts: AdvFonts,
    message: Option<Message>,
    messagebox: Messagebox,
}

impl MessageLayer {
    pub fn new(
        resources: &GpuCommonResources,
        fonts: AdvFonts,
        textures: Arc<MessageboxTextures>,
    ) -> Self {
        Self {
            props: LayerProperties::new(),
            style: MessageboxStyle::default(),
            running_time: Ticks::ZERO,
            fonts,
            message: None,
            messagebox: Messagebox::new(textures, resources),
        }
    }

    pub fn set_style(&mut self, style: MessageboxStyle) {
        self.style = style;
    }

    pub fn set_message(&mut self, context: &UpdateContext, message: &str) {
        self.running_time = Ticks::ZERO;

        self.message = Some(Message::new(
            context,
            // TODO: actually reuse the atlas
            FontAtlas::new(context.gpu_resources, self.fonts.medium_font.clone()),
            Vector2::new(-740.0 - 9.0, 300.0 - 83.0),
            message,
        ));
    }

    pub fn is_finished(&self) -> bool {
        self.message.as_ref().map(|m| m.complete()).unwrap_or(true)
    }
}

impl Renderable for MessageLayer {
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
        transform: Matrix4<f32>,
    ) {
        let transform = self.props.compute_transform(transform);
        self.messagebox.render(resources, render_pass, transform);
        if let Some(message) = &self.message {
            message.render(resources, render_pass, transform);
        }
    }

    fn resize(&mut self, _resources: &GpuCommonResources) {
        // no internal buffers to resize
    }
}

impl Updatable for MessageLayer {
    fn update(&mut self, ctx: &UpdateContext) {
        self.messagebox.update(ctx);
        if let Some(message) = &mut self.message {
            message.update(ctx);
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
