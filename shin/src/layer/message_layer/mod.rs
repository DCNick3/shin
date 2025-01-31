mod blocks;
mod layout;
mod messagebox;

use std::sync::Arc;

use bitflags::bitflags;
use glam::{vec2, vec3, vec4, Mat4, Vec2};
use itertools::{Either, Itertools};
use shin_core::{
    format::scenario::{instruction_elements::MessageId, Scenario},
    layout::{
        commands::{CharFontType, Command},
        LayoutParams, MessageLayerLayouter, MessageTextLayouterDefaults,
    },
    primitives::color::FloatColor4,
    time::Ticks,
    vm::command::types::{AudioWaitStatus, MessageTextLayout, MessageboxType, Volume},
};
use shin_render::{
    gpu_texture::GpuTexture,
    render_pass::RenderPass,
    shaders::types::{
        buffer::{OwnedVertexBuffer, VertexSource},
        vertices::TextVertex,
    },
    ColorBlendType, DrawPrimitive, PassKind, RenderProgramWithArguments, RenderRequestBuilder,
};
use tracing::debug;

use crate::{
    adv::assets::AdvFonts,
    asset::{font::GpuFontLazy, texture_archive::TextureArchive},
    audio::{VoicePlayFlags, VoicePlayer},
    layer::{
        message_layer::{
            blocks::{Block, BlockType},
            messagebox::Messagebox,
        },
        properties::LayerProperties,
        render_params::TransformParams,
        DrawableLayer, Layer, PreRenderContext,
    },
    update::{AdvUpdatable, AdvUpdateContext},
};

#[derive(TextureArchive)]
pub struct MessageboxTextures {
    #[txa(name = "keywait")]
    pub keywait: GpuTexture,
    #[txa(name = "select")]
    pub select: GpuTexture,
    #[txa(name = "select_cur")]
    pub select_cursor: GpuTexture,

    #[txa(name = "msgwnd1")]
    pub message_window_1: GpuTexture,
    #[txa(name = "msgwnd2")]
    pub message_window_2: GpuTexture,
    #[txa(name = "msgwnd3")]
    pub message_window_3: GpuTexture,
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default)]
    pub struct MessageFlags: u32 {
        const UNUSED_FLAG = 0x1;
        const IGNORE_INPUT = 0x2;
    }
}

/// Messagebox, but it has an interpolator for sliding out
///
/// This is used to store additional fake messageboxes for properly animating messagebox type changes
#[derive(Debug, Copy, Clone)]
struct SlidingOutMessagebox {
    pub ty: MessageboxType,
    pub slide_out: SimpleInterpolator,
    pub height: f32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum WaitKind {
    Regular,
    Last,
    AutoClick,
}

#[derive(Debug, Copy, Clone)]
pub struct MsgsetParams {
    pub flags: MessageFlags,
    pub messagebox_type: MessageboxType,
    pub text_layout: MessageTextLayout,
    pub message_id: MessageId,
}

/// A simple utility interpolator class used to animate the message box sliding in and out & opacity changes when menu is opened
///
/// Clamps the value between 0.0 and 1.0 and stores the direction of the interpolation
#[derive(Debug, Copy, Clone)]
struct SimpleInterpolator {
    current_direction: SimpleInterpolatorDirection,
    value: f32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SimpleInterpolatorDirection {
    Increasing,
    Decreasing,
}

impl SimpleInterpolatorDirection {
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

impl SimpleInterpolator {
    const RATE_PER_TICK: f32 = 0.1;

    #[inline]
    pub fn new(value: f32, direction: SimpleInterpolatorDirection) -> Self {
        Self {
            value,
            current_direction: direction,
        }
    }

    pub fn set_direction(&mut self, direction: SimpleInterpolatorDirection) {
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

    pub fn direction(&self) -> SimpleInterpolatorDirection {
        self.current_direction
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Debug, Copy, Clone)]
struct HeightInterpolator {
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
struct Countdown {
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

const VERTICES_PER_CHARACTER: usize = 4;

pub struct MessageLayer {
    messagebox_textures: Arc<MessageboxTextures>,
    props: LayerProperties,
    // TODO: how should we handle the ownership for the listener?
    message_layer_listener: (),
    voice_player: VoicePlayer,
    adv_fonts: AdvFonts,
    // TODO: maybe split it into smaller structs to reduce complexity somewhat
    slide_in: SimpleInterpolator,
    /// Annoyingly, this not only affects opacity, but also the position at the same time
    ///
    /// But it's independent of the `slide_in` interpolator and is used to hide the messagebox when modal dialogues are shown
    ///
    // TODO: I am thinking of renaming slide_in to natural_slide, and opacity to modal_slide (and SimpleInterpolator to SlideInterpolator)
    opacity: SimpleInterpolator,

    autoplay_requested: bool,
    // NB: there is another `_requested`-like field in the original engine, but it's never set, so don't bother
    scenario: Option<Arc<Scenario>>,

    current_block_index: usize,
    current_time: f32,

    height: HeightInterpolator,

    wait_kind: Option<WaitKind>,
    time_to_skip_wait: Countdown,
    autoplay_voice_delay: Countdown,
    is_voice_playing: bool,
    disable_voice: bool,
    // NB: there was a `disable_voice` field, but it's never set to true
    //
    completed_sections: u32,
    received_syncs: u32,
    ticks_since_last_wait: Ticks,

    cursor_position: Vec2,

    current_line_index: usize,
    total_voices_count: u32,
    voice_block_index: usize,
    voice_counter: i32,
    // NB: "some_unused_mode"
    //
    message_flags: MessageFlags,

    messagebox_type: MessageboxType,
    text_layout: MessageTextLayout,
    message_id: MessageId,
    chars: Vec<layout::Char>,
    lines: Vec<layout::LineInfo>,
    blocks: Vec<Block>,

    char_name_width: f32,
    message_size: Vec2,

    vertex_buffer: Option<OwnedVertexBuffer<TextVertex>>,
    sliding_out_messageboxes: Vec<SlidingOutMessagebox>,
    transform: Mat4,
    // font_atlas: Arc<FontAtlas>,
    // message: Option<Message>,
    // messagebox: Messagebox,
}

impl MessageLayer {
    pub fn new(
        adv_fonts: AdvFonts,
        messagebox_textures: Arc<MessageboxTextures>,
        voice_player: VoicePlayer,
    ) -> Self {
        Self {
            messagebox_textures,
            props: LayerProperties::new(),
            message_layer_listener: (), // TODO
            voice_player,
            adv_fonts,

            slide_in: SimpleInterpolator::new(0.0, SimpleInterpolatorDirection::Decreasing),
            opacity: SimpleInterpolator::new(1.0, SimpleInterpolatorDirection::Increasing),

            autoplay_requested: false,
            scenario: None,
            current_block_index: 0,
            current_time: 0.0,
            height: HeightInterpolator::new(357.0),
            wait_kind: None,
            time_to_skip_wait: Countdown::new(0.0),
            autoplay_voice_delay: Countdown::new(0.0),
            is_voice_playing: false,
            disable_voice: false,
            completed_sections: 0,
            received_syncs: 0,
            ticks_since_last_wait: Ticks::ZERO,
            cursor_position: Vec2::ZERO,
            current_line_index: 0,
            total_voices_count: 0,
            voice_block_index: 0,
            voice_counter: 0,
            message_flags: MessageFlags::default(),
            messagebox_type: MessageboxType::Neutral,
            text_layout: MessageTextLayout::Justify,
            message_id: MessageId(0),
            chars: vec![],
            lines: vec![],
            blocks: vec![],
            char_name_width: 0.0,
            message_size: Vec2::ZERO,
            vertex_buffer: None,
            sliding_out_messageboxes: vec![],
            transform: Mat4::from_translation(vec3(-960.0, -540.0, 0.0)),
        }
    }

    fn reset_message(&mut self) {
        self.vertex_buffer = None;
        self.blocks.clear();
        self.lines.clear();
        self.chars.clear();
        self.total_voices_count = 0;
    }

    fn rebuild_vertices(&mut self, ctx: &PreRenderContext) {
        self.vertex_buffer = None;

        let mut vertices = Vec::with_capacity(VERTICES_PER_CHARACTER * self.chars.len());

        for char in &self.chars {
            let glyph = char.glyph.info();

            let scaled_size = char.scale() * glyph.actual_size_f32();

            // this adds an overdraw of 2 pixels on all sides of the character
            let pos_to_top_left = char.scale() * glyph.bearing_screenspace_f32() - 2.0;
            let pos_to_bottom_right = pos_to_top_left + scaled_size + 4.0;

            let [screen_left, screen_top] = (char.position + pos_to_top_left).to_array();
            let [screen_right, screen_bottom] = (char.position + pos_to_bottom_right).to_array();

            let tex_overdraw_ratio =
                ((pos_to_bottom_right - pos_to_top_left) / scaled_size - 1.0) / 2.0;

            let [tex_left, tex_top] =
                (-glyph.actual_size_normalized() * tex_overdraw_ratio).to_array();
            let [tex_right, tex_bottom] =
                (glyph.actual_size_normalized() * (tex_overdraw_ratio + 1.0)).to_array();

            let (color_top, color_bottom) = if char.is_rubi {
                (0.0, 0.0)
            } else {
                let line = &self.lines[char.line_index];
                // I don't think the formulas in the original engine are right (it doesn't take scale into account),
                // but umineko doesn't rely on this feature (both tints are set to the same color),
                // so I ain't gonna fix it

                // NB: this bearing is NOT in screenspace, so we need to flip the Y
                let effective_ascent = line.baseline_ascent - glyph.bearing_y as f32;
                let effective_descent = effective_ascent + glyph.actual_height as f32;

                (
                    effective_ascent / line.line_height,
                    effective_descent / line.line_height,
                )
            };

            let char_vertices: [_; VERTICES_PER_CHARACTER] = [
                TextVertex {
                    position: vec4(screen_left, screen_top, tex_left, tex_top),
                    color: color_top,
                },
                TextVertex {
                    position: vec4(screen_right, screen_top, tex_right, tex_top),
                    color: color_top,
                },
                TextVertex {
                    position: vec4(screen_left, screen_bottom, tex_left, tex_bottom),
                    color: color_bottom,
                },
                TextVertex {
                    position: vec4(screen_right, screen_bottom, tex_right, tex_bottom),
                    color: color_bottom,
                },
            ];

            vertices.extend(char_vertices);
        }

        self.vertex_buffer = Some(OwnedVertexBuffer::allocate_vertex(
            ctx.device,
            &vertices,
            Some("MessageLayer/vtxbuf"),
        ));
    }

    pub fn is_interested_in_input(&self) -> bool {
        // messagebox is in the process of being hidden or is not fully shown yet
        // TODO: can this be a function on SimpleInterpolator?
        if self.slide_in.direction() == SimpleInterpolatorDirection::Decreasing {
            return false;
        }
        if self.slide_in.value() < 1.0 {
            return false;
        }

        // All the blocks that we can wait were already executed
        if self.current_block_index >= self.blocks.len() {
            return false;
        }

        if self.message_flags.contains(MessageFlags::IGNORE_INPUT) {
            return false;
        }
        if self.messagebox_type == MessageboxType::NoText
            && self.wait_kind == Some(WaitKind::AutoClick)
        {
            return false;
        }

        true
    }

    pub fn is_waiting(&self, signal_index: i32) -> bool {
        // all blocks are done, so nothing should be waiting
        if self.current_block_index >= self.blocks.len() {
            return false;
        }
        // negative `section_index` -> wait for the full message
        if signal_index < 0 {
            return true;
        }
        let signal_index = signal_index as u32;

        self.completed_sections <= signal_index
    }

    fn play_voice(&mut self, voice_index: usize, segment_start: u32, segment_duration: u32) {
        // NB: original game passes the actual voice block reference in here,
        // but it's annoying to do with borrow checker, as we need &mut for VoicePlayer::play
        let Block {
            ty: BlockType::Voice(voice),
            ..
        } = &self.blocks[voice_index]
        else {
            panic!(
                "Expected a voice block, but got {:?}",
                self.blocks[voice_index].ty
            )
        };
        // TODO: need a settings handle here
        let voicevol = 90;
        if voicevol == 0 {
            return;
        }

        let flags = if voice.lipsync_enabled {
            VoicePlayFlags::ENABLE_CHARACTER_LIPSYNC | VoicePlayFlags::ENABLE_CHARACTER_MUTING
        } else {
            VoicePlayFlags::ENABLE_CHARACTER_MUTING
        };

        debug!(
            "Playing voice: {:?} (segment_start: {}, segment_duration: {})",
            voice.filename, segment_start, segment_duration
        );

        self.is_voice_playing = self.voice_player.play(
            self.scenario.as_ref().unwrap(),
            &voice.filename,
            segment_start,
            segment_duration,
            flags,
            voice.volume,
        );

        self.autoplay_voice_delay
            .set_time_left(if self.is_voice_playing { 0.5 } else { 0.0 });
    }

    pub fn on_msgset(
        &mut self,
        ctx: &PreRenderContext,
        scenario: &Arc<Scenario>,
        // NB: the original engine accepted a number here,
        // but it's always set to -1 when parsing MSGSET and the (clamped) number is never used later, so unimplemented
        message: &str,
        params: MsgsetParams,
        dont_ff_slide: bool,
    ) {
        // if the user can currently see a messagebox of a different type, take care to slide it out before (visibly) changing the type
        if params.messagebox_type != self.messagebox_type && self.slide_in.value() > 0.0 {
            self.sliding_out_messageboxes.push(SlidingOutMessagebox {
                ty: self.messagebox_type,
                // it should always be sliding out
                slide_out: SimpleInterpolator::new(
                    self.slide_in.value(),
                    SimpleInterpolatorDirection::Decreasing,
                ),
                height: self.height.value(),
            });
            self.slide_in.set_value(0.0);
        }
        self.reset_message();

        self.message_flags = params.flags;
        self.messagebox_type = params.messagebox_type;
        self.text_layout = params.text_layout;
        self.message_id = params.message_id;

        self.scenario = Some(scenario.clone());

        self.slide_in
            .set_direction(SimpleInterpolatorDirection::Increasing);

        self.current_block_index = 0;
        self.current_time = 0.0;

        self.wait_kind = None;
        self.time_to_skip_wait.set_time_left(0.0);
        self.autoplay_voice_delay.set_time_left(0.0);
        self.is_voice_playing = false;
        self.disable_voice = false;

        self.completed_sections = 0;
        self.received_syncs = 0;
        self.ticks_since_last_wait = Ticks::ZERO;

        self.cursor_position = Vec2::ZERO;

        self.current_line_index = 0;
        self.total_voices_count = 0;
        self.voice_block_index = 0;
        self.voice_counter = -1;

        if self.messagebox_type == MessageboxType::Novel {
            self.height.set_target(1080.0);
            self.height.set_value(1080.0);
        } else {
            if self.slide_in.value() == 0.0 {
                self.height.set_value(357.0);
            }
            self.height.set_target(357.0);
        }

        self.set_message(ctx, message);
        self.rebuild_vertices(ctx);

        if !dont_ff_slide {
            self.slide_in.set_value(1.0);
        }
    }

    fn set_message(&mut self, ctx: &PreRenderContext, message: &str) {
        let layout_params = LayoutParams {
            layout_width: 1500.0,
            text_alignment: self.text_layout,
            line_padding_above: 0.0,
            line_padding_below: 0.0,
            line_padding_between: 4.0,
            rubi_size: 20.0,
            text_size: 50.0,
            base_font_horizontal_scale: 0.9697,
            follow_kinsoku_shori_rules: true,
            always_leave_space_for_rubi: true, // < I am not sure if this should be true
            perform_soft_breaks: true,
        };
        let defaults = MessageTextLayouterDefaults {
            color: 999,
            draw_speed: if self.messagebox_type == MessageboxType::NoText {
                100
            } else {
                80 // TODO: this comes from settings
            },
            fade: 200,
        };

        let (commands, lines, size) = MessageLayerLayouter::<&GpuFontLazy>::new(
            self.adv_fonts.medium_font.as_ref(),
            self.adv_fonts.bold_font.as_ref(),
            self.messagebox_type,
            layout_params,
            defaults,
        )
        .parse(message);

        let (regular_chars, bold_chars) = commands
            .iter()
            .filter_map(|c| {
                if let Command::Char(c) = c {
                    Some((c.codepoint, c.font))
                } else {
                    None
                }
            })
            .partition_map::<Vec<_>, Vec<_>, _, _, _>(|(c, font)| match font {
                CharFontType::Regular => Either::Left(c),
                CharFontType::Bold => Either::Right(c),
            });
        let mut regular_glyphs = self
            .adv_fonts
            .medium_font
            .clone()
            .load_glyphs(ctx.device, ctx.queue, &regular_chars)
            .into_iter();
        let mut bold_glyphs = self
            .adv_fonts
            .bold_font
            .clone()
            .load_glyphs(ctx.device, ctx.queue, &bold_chars)
            .into_iter();

        let mut wait_auto_delay = 0.0;
        for command in commands {
            match command {
                Command::Char(char) => {
                    let glyph = match char.font {
                        // rely on the order of the glyphs returned from `load_glyphs`
                        CharFontType::Regular => regular_glyphs.next().unwrap(),
                        CharFontType::Bold => bold_glyphs.next().unwrap(),
                    };
                    let info = glyph.info();

                    // TODO: maybe support v8=1 setting, which changes the shape of the outline

                    let v_distance = 1.5 / char.scale / info.actual_height as f32
                        * info.actual_size_normalized().y;
                    let h_distance = 1.5 / char.horizontal_scale / info.actual_width as f32
                        * info.actual_size_normalized().x;

                    // prepare a set of 8 displacements for font outline border shader
                    // they go in a circle, just in a weird order:
                    //
                    //         |
                    //         2
                    //      1  |  3
                    // ---4---------5---> x
                    //      6  |  8
                    //         7
                    //         |
                    //        \/  y
                    #[rustfmt::skip]
                    let border_distances = [
                        /* 1 */ vec2(-1.0, -1.0),
                        /* 2 */ vec2( 0.0, -1.0),
                        /* 3 */ vec2( 1.0, -1.0),
                        /* 4 */ vec2(-1.0,  0.0),
                        /* 5 */ vec2( 1.0,  0.0),
                        /* 6 */ vec2(-1.0,  1.0),
                        /* 7 */ vec2( 0.0,  1.0),
                        /* 8 */ vec2( 1.0,  1.0),
                    ]
                    .map(|v| v / v.length()) // normalize the length
                    .map(|v| v * vec2(h_distance, v_distance)); // apply the scaling for each axis

                    if !char.is_rubi {
                        wait_auto_delay += 0.05;
                    }

                    self.chars.push(layout::Char {
                        time: char.time,
                        line_index: char.line_index,
                        is_rubi: char.is_rubi,
                        position: char.position,
                        width: char.width,
                        height: char.height,
                        horizontal_scale: char.horizontal_scale,
                        vertical_scale: char.scale,
                        color_rgba: char.color,
                        progress_rate: {
                            if char.fade == 0.0 {
                                1.0
                            } else {
                                1.0 / char.fade / 60.0
                            }
                        },
                        current_progress: 0.0,
                        block_index: self.blocks.len(),
                        vertex_buffer_offset: self.chars.len() * VERTICES_PER_CHARACTER,
                        border_distances,
                        glyph,
                    });
                }
                Command::Section(section) => self.blocks.push(Block {
                    time: section.time,
                    ty: BlockType::Section(blocks::Section {
                        index: section.index,
                    }),
                }),
                Command::Sync(sync) => self.blocks.push(Block {
                    time: sync.time,
                    ty: BlockType::Sync(blocks::Sync { index: sync.index }),
                }),
                Command::Voice(voice) => self.blocks.push(Block {
                    time: voice.time,
                    ty: BlockType::Voice(blocks::Voice {
                        filename: voice.filename,
                        volume: Volume(voice.volume),
                        lipsync_enabled: voice.lipsync_enabled,
                        segment_duration: voice.segment_duration,
                    }),
                }),
                Command::VoiceSync(voice_sync) => self.blocks.push(Block {
                    time: voice_sync.time,
                    ty: BlockType::VoiceSync(blocks::VoiceSync {
                        segment_start: voice_sync.segment_start,
                        segment_duration: voice_sync.segment_duration,
                    }),
                }),
                Command::VoiceWait(voice_wait) => self.blocks.push(Block {
                    time: voice_wait.time,
                    ty: BlockType::VoiceWait(blocks::VoiceWait),
                }),
                Command::Wait(wait) => {
                    self.blocks.push(Block {
                        time: wait.time,
                        ty: BlockType::Wait(blocks::Wait {
                            wait_auto_delay,
                            is_last_wait: wait.is_last_wait,
                            is_auto_click: wait.is_auto_click,
                        }),
                    });

                    wait_auto_delay = 0.0;
                }
            }
        }

        for line in &lines {
            self.lines.push(layout::LineInfo {
                y_position: line.y_position,
                baseline_ascent: line.baseline_ascent,
                line_height: line.line_height,
                rubi_height: line.rubi_height,
                is_visible: 0.0,
            });
        }

        self.char_name_width = lines[0].width;
        self.message_size = size;
    }

    pub fn close(&mut self, dont_ff_slide: bool) {
        self.slide_in
            .set_direction(SimpleInterpolatorDirection::Decreasing);
        if !dont_ff_slide {
            self.slide_in.set_value(0.0);
        }
        self.reset_message();
    }

    pub fn signal(&mut self) {
        todo!()
        // if let Some(message) = self.message.as_mut() {
        //     message.signal();
        // }
    }

    pub fn advance(&mut self) {
        todo!()
        // if let Some(m) = self.message.as_mut() {
        //     m.advance()
        // }
    }

    pub fn fast_forward(&mut self) {
        todo!()
        // if let Some(m) = self.message.as_mut() {
        //     m.fast_forward()
        // }
    }
}

impl Clone for MessageLayer {
    fn clone(&self) -> Self {
        // while some parts of MessageLayer would be okay to be cloned, `VoicePlayer` is not
        // the ownership will end up being all weird
        // It's not like we would actually _need_ to clone it though, as transitions can't be applied to root layer group
        unimplemented!("MessageLayer is not supposed to be cloned")
    }
}

impl AdvUpdatable for MessageLayer {
    fn update(&mut self, context: &AdvUpdateContext) {
        let dt = context.delta_ticks;

        let autoplay_requested = self.autoplay_requested;
        self.autoplay_requested = false;

        self.props.update(context);

        // if we are (semi)-transparent, don't update anything else
        // this is used for pausing
        if self.opacity.update(dt) < 1.0 {
            return;
        }

        self.slide_in.update(dt);

        self.sliding_out_messageboxes.retain_mut(|messagebox| {
            // NB: technically not the same as the original game (it doesn't clamp and checks < 0.0),
            // but it's not a big difference
            messagebox.slide_out.update(dt) > 0.0
        });

        if self.slide_in.value() < 1.0 {
            return;
        }

        if self.is_voice_playing {
            if self
                .voice_player
                .get_wait_status()
                .contains(AudioWaitStatus::PLAYING)
            {
                // TODO: need to access the settings here
                // if voice volume is set to 0, stop the player & reset all the voice stuff
            } else {
                // voice player no longer playing, auto mode can proceed
                self.is_voice_playing = false;
            }
        }

        // TODO: this condition structure is kinda ugly...
        if let Some(wait_kind) = self.wait_kind {
            self.time_to_skip_wait.update(dt);

            let autoplay_effective = wait_kind == WaitKind::AutoClick
                || autoplay_requested && !self.message_flags.contains(MessageFlags::IGNORE_INPUT);

            if !autoplay_effective {
                if !self.is_voice_playing {
                    self.autoplay_voice_delay.set_time_left(0.5);
                }
            } else if self.time_to_skip_wait.is_done()
                && self.is_voice_playing == false
                && self.autoplay_voice_delay.update(dt)
            {
                if matches!(wait_kind, WaitKind::Last | WaitKind::AutoClick) {
                    // TODO: notify the listener that the message is done
                }
                self.wait_kind = None;
                self.current_block_index += 1;
            }
        };

        if self.wait_kind.is_none() {
            // using a while loop, as for is annoying for borrow checker reasons:
            // we want to call &mut self methods while iterating
            while self.current_block_index < self.blocks.len() {
                let block = &self.blocks[self.current_block_index];
                if block.time > self.current_time {
                    break;
                }
                match &block.ty {
                    BlockType::Voice(voice) => {
                        if self.is_voice_playing {
                            break;
                        }
                        // NB: the original game checks !unknown_requested here, but it's never requested
                        // this is probably an older way to do fast forwarding
                        self.play_voice(self.current_block_index, 0, voice.segment_duration);
                        self.voice_block_index = self.current_block_index;
                        self.voice_counter += 1;
                    }
                    BlockType::Wait(wait) => {
                        if self.height.is_interpolating() {
                            break;
                        }

                        let has_incomplete_char = self.chars.iter().any(|c| {
                            // char is not complete
                            c.current_progress < 1.0
                                    // and is in our block or earlier
                                    && c.block_index <= self.current_block_index
                        });
                        if has_incomplete_char || self.wait_kind.is_some() {
                            break;
                        }

                        if !self.message_flags.contains(MessageFlags::IGNORE_INPUT) {
                            self.wait_kind = Some(match (wait.is_last_wait, wait.is_auto_click) {
                                (_, true) => WaitKind::AutoClick,
                                (true, _) => WaitKind::Last,
                                _ => WaitKind::Regular,
                            });

                            self.ticks_since_last_wait = Ticks::ZERO;
                            // TODO: need to access the settings here to get v10_skipspeed
                            let skip_speed = 80;
                            self.time_to_skip_wait.set_time_left(
                                wait.wait_auto_delay * (100 - skip_speed) as f32 * 0.01,
                            );
                            debug!("Waiting on block {}", self.current_block_index);
                            break;
                        }
                        if self
                            .voice_player
                            .get_wait_status()
                            .contains(AudioWaitStatus::PLAYING)
                        {
                            break;
                        }
                        if wait.is_last_wait {
                            // TODO: notify the listener that the message is done
                        }
                    }
                    BlockType::Section(section) => {
                        self.completed_sections = section.index;
                    }
                    BlockType::Sync(sync) => {
                        if self.received_syncs <= sync.index {
                            break;
                        }
                    }
                    BlockType::VoiceSync(voice_sync) => {
                        self.play_voice(
                            self.voice_block_index,
                            voice_sync.segment_start,
                            voice_sync.segment_duration,
                        );
                    }
                    BlockType::VoiceWait(_) => {
                        if self.is_voice_playing {
                            break;
                        }
                    }
                }
                self.current_block_index += 1;
                debug!("Advanced to block {}", self.current_block_index);
            }
        }

        // surely nobody would need more than 64 lines, right?
        // NB: the original game uses an std::vector<bool> here, but I don't wanna
        let mut line_mask = 0u64;

        for char in &mut self.chars {
            if char.block_index > self.current_block_index || char.time > self.current_time {
                continue;
            }

            char.current_progress =
                1.0f32.min(char.current_progress + char.progress_rate * dt.as_f32());

            let line = &self.lines[char.line_index];
            self.height
                .set_min_target(char.position.y + line.line_height);

            let candidate_cursor_position = char.position + vec2(char.width, 0.0);
            if self.cursor_position.y == candidate_cursor_position.y {
                if self.cursor_position.x < candidate_cursor_position.x {
                    self.cursor_position = candidate_cursor_position;
                }
            } else if self.cursor_position.y < candidate_cursor_position.y {
                self.cursor_position = candidate_cursor_position;
            }

            assert!(char.line_index < 64);
            line_mask |= 1 << char.line_index as u64;
        }

        for (i, line) in (0..).zip(&mut self.lines) {
            if line_mask & (1 << i as u64) == 0 {
                continue;
            }

            line.is_visible = 1.0;
        }

        self.height.update(dt);

        if self.current_block_index < self.blocks.len() {
            let block_time = self.blocks[self.current_block_index].time;

            self.current_time = block_time.min(self.current_time + dt.as_seconds());
        }

        self.ticks_since_last_wait += dt;
    }
}

impl Layer for MessageLayer {
    fn get_stencil_bump(&self) -> u8 {
        3
    }

    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        if pass_kind != PassKind::Transparent {
            return;
        }

        pass.push_debug("MessageLayer");

        let transform = transform.compute_final_transform() * self.transform;

        pass.push_debug("MessageLayer/messageboxes");

        let builder = RenderRequestBuilder::new().depth_stencil_shorthand(stencil_ref, true, false);
        for &messagebox in &self.sliding_out_messageboxes {
            self.render_messagebox(pass, builder, transform, messagebox.into());
        }
        self.render_messagebox(
            pass,
            builder,
            transform,
            Messagebox {
                ty: self.messagebox_type,
                slide_in_progress: self.slide_in.value(),
                height: self.height.value(),
            },
        );
        pass.pop_debug();

        if self.slide_in.value() * self.opacity.value() < 1.0 {
            pass.pop_debug();
            return;
        }

        let Some(vertex_buffer) = &self.vertex_buffer else {
            pass.pop_debug();
            return;
        };

        let position_y = match self.messagebox_type {
            MessageboxType::Neutral
            | MessageboxType::WitchSpace
            | MessageboxType::Ushiromiya
            | MessageboxType::Transparent
            | MessageboxType::NoText => {
                (1.0 - self.slide_in.value()) * 64.0 + (1080.0 - self.height.value()) - 32.0
            }

            MessageboxType::Novel => 32.0f32.max((1080.0 - self.message_size.y) * 0.35),
        };

        match self.wait_kind {
            Some(WaitKind::Regular | WaitKind::Last) => {
                pass.push_debug("MessageLayer/keywait");
                // TODO: render the keywait sprite from msgtex txa
                // need SpriteVertices
                pass.pop_debug();
            }
            Some(WaitKind::AutoClick) | None => {
                // do not render the keywait
            }
        }

        if self.messagebox_type == MessageboxType::NoText {
            pass.pop_debug();
            return;
        }

        let transform = transform * Mat4::from_translation(vec3(210.0, position_y, 0.0));

        let builder = RenderRequestBuilder::new()
            .color_blend_type(ColorBlendType::Layer1)
            .depth_stencil_shorthand(stencil_ref + 2, true, true);

        pass.push_debug("MessageLayer/text");
        pass.push_debug("MessageLayer/text/border");
        // draw the borders...
        for char in &self.chars {
            pass.run(builder.build(
                RenderProgramWithArguments::FontBorder {
                    vertices: VertexSource::VertexBuffer {
                        vertices: vertex_buffer.as_sliced_buffer_ref(
                            char.vertex_buffer_offset,
                            VERTICES_PER_CHARACTER,
                        ),
                    },
                    glyph: char.glyph.as_texture_source(),
                    transform,
                    distances: char.border_distances,
                    color: FloatColor4::from_rgba(0.0, 0.0, 0.0, char.current_progress),
                },
                DrawPrimitive::TrianglesStrip,
            ));
        }
        pass.pop_debug();

        pass.push_debug("MessageLayer/text/normal");
        // and the characters themselves
        for char in &self.chars {
            pass.run(builder.build(
                RenderProgramWithArguments::Font {
                    vertices: VertexSource::VertexBuffer {
                        vertices: vertex_buffer.as_sliced_buffer_ref(
                            char.vertex_buffer_offset,
                            VERTICES_PER_CHARACTER,
                        ),
                    },
                    glyph: char.glyph.as_texture_source(),
                    transform,
                    color1:
                        FloatColor4::from_unorm(char.color_rgba).with_alpha(char.current_progress),
                    color2:
                        FloatColor4::from_unorm(char.color_rgba).with_alpha(char.current_progress),
                },
                DrawPrimitive::TrianglesStrip,
            ));
        }
        pass.pop_debug();

        pass.pop_debug();
    }
}

impl DrawableLayer for MessageLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
