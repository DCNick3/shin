mod font_atlas;
mod messagebox;

pub use messagebox::MessageboxTextures;
use std::sync::Arc;

use crate::adv::assets::AdvFonts;
use crate::layer::message_layer::font_atlas::FontAtlas;
use crate::layer::message_layer::messagebox::Messagebox;
use crate::layer::{Layer, LayerProperties};
use crate::render::dynamic_atlas::AtlasImage;
use crate::render::overlay::{OverlayCollector, OverlayVisitable};
use crate::render::{GpuCommonResources, Renderable, TextVertex, VertexBuffer};
use crate::update::{Updatable, UpdateContext};
use cgmath::{ElementWise, Matrix4, Vector2};
use shin_core::format::font::GlyphTrait;
use shin_core::layout::{Action, ActionType, Block, BlockExitCondition, LayoutedMessage};
use shin_core::vm::command::layer::{MessageTextLayout, MessageboxStyle};
use shin_core::vm::command::time::Ticks;
use tracing::warn;

struct Message {
    time: Ticks,
    font_atlas: Arc<FontAtlas>,
    used_codepoints: Vec<u16>,
    actions: Vec<Action>,
    blocks: Vec<Block>,
    vertex_buffer: VertexBuffer<TextVertex>,
    signalled: u32,
    completed_blocks: u32,
}

pub enum MessageStatus {
    Printing,
    ClickWaiting,
    SignalWaiting,
    Complete,
}

impl Message {
    pub fn new(
        context: &UpdateContext,
        font_atlas: Arc<FontAtlas>,
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

        let LayoutedMessage {
            chars,
            mut actions,
            mut blocks,
        } = shin_core::layout::layout_text(layout_params, message);

        // reverse the blocks & actions so that we can easily pop them off the end in order
        blocks.reverse();
        actions.reverse();

        let mut used_codepoints = Vec::new();
        let mut vertices = Vec::new();
        for char in chars.iter() {
            // TODO: support for BOLD font
            let glyph_info = font_atlas
                .get_font()
                .get_glyph_for_character(char.codepoint)
                .get_info();

            let atlas_size = font_atlas.texture_size();
            let atlas_size = Vector2::new(atlas_size.0 as f32, atlas_size.1 as f32);

            let AtlasImage {
                position: tex_position,
                size: _, // the atlas size is not to be trusted, as it can be larger than the actual texture (even larger than the power of 2 padded texture...)
            } = font_atlas.get_glyph(context.gpu_resources, char.codepoint);
            // save the codepoint to free it from the atlas later
            used_codepoints.push(char.codepoint);

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
                        position: position + Vector2::new($x, $y).mul_element_wise(size),
                        tex_position: tex_position
                            + Vector2::new($tex_x, $tex_y).mul_element_wise(tex_size),
                        color,
                        time,
                        fade,
                    }
                };
            }

            vertices.extend([
                // Top left triangle
                v!((0.0, 0.0), (0.0, 0.0)),
                v!((1.0, 0.0), (1.0, 0.0)),
                v!((0.0, 1.0), (0.0, 1.0)),
                // Bottom right triangle
                v!((1.0, 1.0), (1.0, 1.0)),
                v!((0.0, 1.0), (0.0, 1.0)),
                v!((1.0, 0.0), (1.0, 0.0)),
            ]);
        }

        let vertex_buffer = VertexBuffer::new(
            context.gpu_resources,
            &vertices,
            Some("Message VertexBuffer"),
        );

        Self {
            time: Ticks::ZERO,
            font_atlas,
            used_codepoints,
            actions,
            blocks,
            vertex_buffer,
            signalled: 0,
            completed_blocks: 0,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn status(&self) -> MessageStatus {
        match self.current_block() {
            None => MessageStatus::Complete,
            Some(block) => {
                if block.completed(self.time) {
                    match block.exit_condition {
                        BlockExitCondition::ClickWait => MessageStatus::ClickWaiting,
                        BlockExitCondition::Signal(_) => MessageStatus::SignalWaiting,
                        BlockExitCondition::None => unreachable!(
                            "If Block has None as exit condition it should be immediately removed"
                        ),
                    }
                } else {
                    MessageStatus::Printing
                }
            }
        }
    }

    fn current_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    fn next_block(&mut self) {
        // let old_block =
        self.blocks
            .pop()
            .expect("Message::next_block called when no blocks remain");
        self.completed_blocks += 1;

        // let overshoot_time = self.time - old_block.end_time;
        if let Some(block) = self.current_block() {
            self.time = block.start_time;
            self.execute_actions();
            // self.time += overshoot_time;
        }
    }

    pub fn advance(&mut self) {
        if let Some(block) = self.current_block() {
            if block.completed(self.time)
                && matches!(block.exit_condition, BlockExitCondition::ClickWait)
            {
                self.next_block();
            } else {
                // skip to the end of the current block
                // NOTE: we may want to have a separate control for that
                self.time = block.end_time;
            }
        }
    }

    fn execute_actions(&mut self) {
        while let Some(action) = self.actions.last() {
            if action.time > self.time {
                break;
            }
            let action = self.actions.pop().unwrap();
            match action.action_type {
                ActionType::SetLipSync(state) => warn!("Ignoring SetLipSync action: {:?}", state),
                ActionType::VoiceVolume(volume) => {
                    warn!("Ignoring voice volume change: {}", volume)
                }
                ActionType::Voice(filename) => warn!("Ignoring voice action: {}", filename),
                ActionType::Signal => self.signalled += 1,
            }
        }
    }

    pub fn completed_blocks(&self) -> u32 {
        self.completed_blocks
    }

    pub fn signalled(&self) -> u32 {
        self.signalled
    }
}

impl Updatable for Message {
    fn update(&mut self, context: &UpdateContext) {
        if let Some(block) = self.current_block() {
            if !block.completed(self.time) {
                self.time += context.time_delta_ticks();
                self.execute_actions();
            } else if matches!(block.exit_condition, BlockExitCondition::None) {
                self.next_block();
            }
        }
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
        for &codepoint in self.used_codepoints.iter() {
            self.font_atlas.free_glyph(codepoint);
        }
    }
}

pub struct MessageLayer {
    props: LayerProperties,
    style: MessageboxStyle,
    running_time: Ticks,
    font_atlas: Arc<FontAtlas>,
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
            font_atlas: Arc::new(FontAtlas::new(resources, fonts.medium_font)),
            message: None,
            messagebox: Messagebox::new(textures, resources),
        }
    }

    pub fn set_style(&mut self, style: MessageboxStyle) {
        self.style = style;

        self.messagebox.set_messagebox_type(style.messagebox_type);
    }

    pub fn set_message(&mut self, context: &UpdateContext, message: &str) {
        self.running_time = Ticks::ZERO;

        self.messagebox.set_visible(true);
        self.message = Some(Message::new(
            context,
            // TODO: actually reuse the atlas
            self.font_atlas.clone(),
            Vector2::new(-740.0 - 9.0, 300.0 - 83.0),
            message,
        ));
    }

    pub fn is_finished(&self) -> bool {
        self.message
            .as_ref()
            .map(|m| m.is_complete())
            .unwrap_or(true)
    }

    pub fn advance(&mut self) {
        if let Some(m) = self.message.as_mut() {
            m.advance()
        }
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

impl OverlayVisitable for MessageLayer {
    fn visit_overlay(&self, collector: &mut OverlayCollector) {
        collector.subgroup(
            "Message Layer",
            |collector| {
                collector.overlay(
                    "Status",
                    |_ctx, top_left| {
                        let status = match self.message {
                            None => "N",
                            Some(ref m) => match m.status() {
                                MessageStatus::Printing => "P",
                                MessageStatus::ClickWaiting => "K",
                                MessageStatus::SignalWaiting => "Y",
                                MessageStatus::Complete => "C",
                            },
                        };
                        let blocks = self
                            .message
                            .as_ref()
                            .map(|m| m.completed_blocks())
                            .unwrap_or(0);
                        let signalled = self.message.as_ref().map(|m| m.signalled()).unwrap_or(0);
                        let time = self.message.as_ref().map(|v| v.time.0).unwrap_or(0.0);

                        top_left.label(format!(
                            "MessageLayer: {} B={} S={} T={:06.1} AF={:04.1}%",
                            status,
                            blocks,
                            signalled,
                            time,
                            self.font_atlas.free_space() * 100.0,
                        ));
                    },
                    true,
                );
                self.font_atlas.visit_overlay(collector);
            },
            true,
        );
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
