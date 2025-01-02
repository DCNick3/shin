pub mod commands;
pub mod font;
pub mod mixins;

use commands::{Char, Command, Section, Voice, VoiceSync, VoiceWait, Wait};
use float_ord::FloatOrd;
use font::FontMetrics;
use glam::{vec2, Vec2};
use itertools::Itertools;
use shin_primitives::{char_set::CharSet, color::UnormColor};

use crate::{
    layout::text_layouter::TextLayouter,
    vm::command::types::{MessageTextLayout, MessageboxType},
};

#[derive(Debug, PartialEq)]
pub struct LineInfo {
    pub width: f32,
    pub y_position: f32,
    /// Distance from `y_position` to the `y_position` of the next line (minus `height3`)
    pub line_advance: f32,
    /// Distance from `y_position` to the baseline of the base text
    pub total_height: f32,
    pub rubi_height: f32,
}

pub struct LayoutParams {
    pub layout_width: f32,
    pub text_alignment: MessageTextLayout,

    // this is a mess..
    /// Space before the line
    pub line_spacing: f32,
    /// Space below the line
    pub another_line_height: f32,
    /// Space between the lines (?)
    pub line_height3: f32,

    pub rubi_size: f32,
    pub text_size: f32,
    pub base_font_horizontal_scale: f32,
    pub follow_kinsoku_shori_rules: bool,
    pub always_leave_space_for_rubi: bool,
    pub perform_soft_breaks: bool,
}

impl Default for LayoutParams {
    fn default() -> Self {
        Self {
            layout_width: 640.0,
            text_alignment: MessageTextLayout::Justify,
            line_spacing: 0.0,
            another_line_height: 0.0,
            line_height3: 0.0,
            rubi_size: 0.0,
            text_size: 20.0,
            base_font_horizontal_scale: 1.0,
            follow_kinsoku_shori_rules: true,
            always_leave_space_for_rubi: false,
            perform_soft_breaks: true,
        }
    }
}

pub struct MessageTextLayouterDefaults {
    // NOTE: unparsed values are stored here
    pub color: i32,
    pub draw_speed: i32,
    pub fade: i32,
}

fn parse_color(color: i32) -> UnormColor {
    UnormColor::from_decimal_rgb(color)
}

fn parse_draw_speed(draw_speed: i32) -> f32 {
    let draw_speed = draw_speed.clamp(0, 100);

    (100 - draw_speed) as f32 * 0.0001 * 0.25
}

fn parse_fade(fade: i32) -> f32 {
    fade.max(0) as f32 * 0.001
}

fn parse_font_scale(size: i32) -> f32 {
    size.clamp(10, 200) as f32 * 0.01
}

fn parse_voice_volume(volume: i32) -> f32 {
    volume.clamp(0, 100) as f32 * 0.01
}

pub struct MessageTextLayouterImpl<Font> {
    pub commands: Vec<Command>,
    pub lines: Vec<LineInfo>,
    pub font_normal: Font,
    pub font_bold: Font,
    pub params: LayoutParams,

    pub default_font_scale: f32,
    pub default_color_rgba: UnormColor,
    pub default_draw_speed: f32,
    pub default_fade: f32,

    pub finalized_command_count: usize,
    pub position: Vec2,

    pub current_time: f32,
    pub block_start_time: f32,
    pub block_max_time: f32,

    pub font_scale: f32,
    pub color_rgba: UnormColor,
    pub draw_speed: f32,
    pub fade: f32,

    pub is_inside_instant_text: bool,
    pub auto_click: bool,
    pub lipsync_enabled: bool,
    pub voice_volume: f32,

    pub last_voice_or_voicesync_index: usize,
    pub rubi_text: String,
    pub rubi_open: bool,
    pub rubi_start_cmd_index: usize,
    pub rubi_start_x: f32,
    pub rubi_start_time: f32,

    pub is_bold: bool,
    pub section_counter: u32,
    pub sync_counter: u32,

    pub size: Vec2,
}

impl<Font> MessageTextLayouterImpl<Font> {
    pub fn new(
        font_normal: Font,
        font_bold: Font,
        mut params: LayoutParams,
        defaults: MessageTextLayouterDefaults,
    ) -> Self {
        Self {
            commands: vec![],
            lines: vec![],
            font_normal,
            font_bold,
            params: {
                if params.rubi_size == 0.0 {
                    params.rubi_size = params.text_size * 0.4;
                }

                params
            },
            default_font_scale: 1.0,
            default_color_rgba: parse_color(defaults.color),
            default_draw_speed: parse_draw_speed(defaults.draw_speed),
            default_fade: parse_fade(defaults.fade),
            finalized_command_count: 0,
            position: Default::default(),
            current_time: 0.0,
            block_start_time: 0.0,
            block_max_time: 0.0,
            font_scale: 0.0,
            color_rgba: UnormColor::BLACK,
            draw_speed: 0.0,
            fade: 0.0,
            is_inside_instant_text: false,
            auto_click: false,
            lipsync_enabled: false,
            voice_volume: 0.0,
            last_voice_or_voicesync_index: 0,
            rubi_text: "".to_string(),
            rubi_open: false,
            rubi_start_cmd_index: 0,
            rubi_start_x: 0.0,
            rubi_start_time: 0.0,
            is_bold: false,
            section_counter: 0,
            sync_counter: 0,
            size: Default::default(),
        }
    }
}

impl<Font: FontMetrics> MessageTextLayouterImpl<Font> {
    fn get_block_end_time(&self) -> f32 {
        self.current_time.max(self.block_max_time)
    }

    pub fn on_message_start(&mut self) {
        self.commands.clear();
        self.lines.clear();
        self.finalized_command_count = 0;
        self.position = Vec2::ZERO;

        self.current_time = 0.0;
        self.block_start_time = 0.0;
        self.block_max_time = 0.0;
        self.font_scale = 1.0;
        self.color_rgba = self.default_color_rgba;
        self.draw_speed = self.default_draw_speed;
        self.fade = self.default_fade;
        self.is_inside_instant_text = false;
        self.auto_click = false;
        self.lipsync_enabled = true;
        self.voice_volume = 1.0;
        self.last_voice_or_voicesync_index = 0;

        // NB: not touching rubi_text
        self.rubi_open = false;
        self.rubi_start_cmd_index = 0;
        self.rubi_start_x = 0.0;
        self.rubi_start_time = 0.0;
        self.is_bold = false;

        self.section_counter = 1; // sic! unlike sync counter, section counter is initialized to 1
        self.sync_counter = 0;
        self.size = Vec2::ZERO;
    }

    pub fn on_message_end<M: MessageTextLayouterMixin<Font>>(&mut self, mixin: &mut M) {
        self.on_rubi_base_end();

        let cmd = Wait {
            time: self.get_block_end_time(),
            line_index: 0,
            is_last_wait: true,
            is_auto_click: self.auto_click,
        };

        self.block_max_time = 0.0;
        self.current_time += 0.001;
        self.block_start_time = self.current_time;

        self.commands.push(Command::Wait(cmd));

        mixin.on_newline(self);

        self.commands.sort_by_key(|cmd| FloatOrd(cmd.time()));
    }

    pub fn on_char(&mut self, codepoint: char) {
        const SHOULD_NOT_START_A_LINE: CharSet<56> = CharSet::new(")>]―’”‥…─♪、。々〉》」』】〕〟ぁぃぅぇぉっゃゅょゎんゝゞァィゥェォッャュョヮヵヶ・ーヽヾ！）：；？｝～");
        const SHOULD_NOT_END_A_LINE: CharSet<14> = CharSet::new("(<[‘“〈《「『【〔〝（｛");

        let mut cant_be_at_line_start = SHOULD_NOT_START_A_LINE.contains(codepoint);
        let mut cant_be_at_line_end = SHOULD_NOT_END_A_LINE.contains(codepoint);

        if !self.params.follow_kinsoku_shori_rules {
            cant_be_at_line_start = false;
            cant_be_at_line_end = false;
        }

        let font = if self.is_bold {
            &self.font_bold
        } else {
            &self.font_normal
        };

        // TODO: what to do when a nonexistent codepoint is encountered?
        // The original impl seems to ignore it and use uninitialized values (bad)
        let glyph_info = font.get_glyph_info(codepoint).unwrap();

        let scale = self.params.text_size / (font.get_ascent() + font.get_descent()) as f32
            * self.font_scale;
        let horizontal_scale = scale * self.params.base_font_horizontal_scale;
        let width = horizontal_scale * glyph_info.advance_width as f32;
        let height = self.params.text_size * self.font_scale;

        if self.rubi_open {
            // if we have moved on from the start of the rubi base text - do not allow a line break
            cant_be_at_line_start = self.rubi_start_x != self.position.x;
        }

        let mut cmd = Char {
            time: self.current_time,
            line_index: 0,
            codepoint,
            is_rubi: false,
            cant_be_at_line_start,
            cant_be_at_line_end,
            has_rubi: self.rubi_open,
            width,
            height,
            position: self.position,
            horizontal_scale,
            scale,
            color: self.color_rgba,
            fade: self.fade,
        };

        if self.is_inside_instant_text {
            cmd.fade = 0.0;
        } else {
            self.current_time += cmd.width * self.draw_speed;

            let punct_delay = if codepoint == '。' {
                // U+3002 IDEOGRAPHIC FULL STOP
                4.0
            } else if codepoint == '、' {
                // U+3001 IDEOGRAPHIC COMMA
                2.0
            } else {
                0.0
            };

            // NB: IEEE floats are a bitch. Previously I have written (cmd.width + self.params.text_size * punct_delay) * self.draw_speed, but this is not the same thing
            self.current_time += (self.params.text_size * punct_delay) * self.draw_speed;
        }
        self.position.x += cmd.width;

        self.commands.push(Command::Char(cmd));
    }

    pub fn on_newline<M: MessageTextLayouterMixin<Font>>(&mut self, mixin: &mut M) {
        self.on_rubi_base_end();

        let new_commands_to_finalize = self.commands.len() - self.finalized_command_count;

        // NB: the original engine stores references to chars instead of indices
        // we can't do this because of borrow checker (without adding a bunch of RCs), so the shape of the code is very different
        let mut new_char_indices = Vec::with_capacity(new_commands_to_finalize);
        for (i, cmd) in self
            .commands
            .iter()
            .enumerate()
            .skip(self.finalized_command_count)
        {
            if let Command::Char(c) = cmd {
                // rubi characters do not participate in the newline logic
                if c.is_rubi {
                    continue;
                }
                new_char_indices.push(i);
            }
        }

        for (prev, current) in new_char_indices.iter().copied().tuple_windows() {
            // do some tricks in order to get two mutable references to the commands
            let (prev_slice, current_slice) = self.commands.split_at_mut(current);
            let Command::Char(prev) = &mut prev_slice[prev] else {
                unreachable!()
            };
            let Command::Char(current) = &mut current_slice[0] else {
                unreachable!()
            };

            // synchronize the prohibition rules
            if current.cant_be_at_line_start {
                prev.cant_be_at_line_end = true;
            } else if prev.cant_be_at_line_end {
                current.cant_be_at_line_start = true;
            }
        }

        if let Some(&first) = new_char_indices.first() {
            // the first char cannot be forbidden to be at the start of a line, it has no choice
            let Command::Char(char) = &mut self.commands[first] else {
                unreachable!();
            };
            char.cant_be_at_line_start = false;
        }
        if let Some(&last) = new_char_indices.last() {
            // the last char cannot be forbidden to be at the end of a line, it has no choice
            let Command::Char(char) = &mut self.commands[last] else {
                unreachable!();
            };
            char.cant_be_at_line_end = false;
        }

        if self.params.perform_soft_breaks {
            // this is so spaghetti...
            while !new_char_indices.is_empty() {
                let mut valid_line_end = new_char_indices.len();
                let mut char_it = new_char_indices.len();
                for (index_idx, &cmd_idx) in new_char_indices.iter().enumerate() {
                    let Command::Char(char) = &self.commands[cmd_idx] else {
                        unreachable!()
                    };
                    // NB: the layout width can change during `finalize_up_to` calls due to mixins
                    // need to specifically take the current value
                    let layout_width = self.params.layout_width;
                    if char.position.x >= layout_width
                        || char.right_border() > layout_width + layout_width * 0.05
                    // NB: x * 1.05 != x + x * 0.05, blame IEEE floats
                    {
                        // need to insert a soft line break
                        char_it = if valid_line_end != new_char_indices.len() {
                            valid_line_end
                        } else {
                            index_idx
                        };
                        break;
                    }

                    char_it = index_idx + 1;
                    if char.cant_be_at_line_end {
                        char_it = valid_line_end;
                    }
                    valid_line_end = char_it;
                }

                if char_it == new_char_indices.len() {
                    break;
                }

                mixin.finalize_up_to(self, new_char_indices[char_it], false);

                new_char_indices.drain(..char_it);
            }
        }

        mixin.finalize_up_to(self, self.commands.len(), true);
        self.position.x = 0.0;
    }

    pub fn on_click_wait(&mut self) {
        self.on_rubi_base_end();

        let cmd = Wait {
            time: self.get_block_end_time(),
            line_index: 0,
            is_last_wait: false,
            is_auto_click: false,
        };

        self.block_max_time = 0.0;
        self.current_time += 0.001;
        self.block_start_time = self.current_time;

        self.commands.push(Command::Wait(cmd));
    }

    pub fn on_auto_click(&mut self) {
        self.auto_click = true;
    }

    pub fn on_set_font_scale(&mut self, scale: i32) {
        if scale < 0 {
            self.font_scale = self.default_font_scale;
        } else {
            self.font_scale = parse_font_scale(scale);
        }
    }

    pub fn on_set_color(&mut self, color: i32) {
        if color < 0 {
            self.color_rgba = self.default_color_rgba;
        } else {
            self.color_rgba = parse_color(color);
        }
    }

    pub fn on_set_draw_speed(&mut self, speed: i32) {
        if speed < 0 {
            self.draw_speed = self.default_draw_speed;
        } else {
            self.draw_speed = parse_draw_speed(speed);
        }
    }

    pub fn on_set_fade(&mut self, fade: i32) {
        if fade < 0 {
            self.fade = self.default_fade;
        } else {
            self.fade = parse_fade(fade);
        }
    }

    pub fn on_wait(&mut self, delay: i32) {
        self.current_time += delay as f32 * 0.001;
    }

    pub fn on_start_parallel(&mut self) {
        if self.block_max_time < self.current_time {
            self.block_max_time = self.current_time;
        }
        self.current_time = self.block_start_time;
    }

    pub fn on_section(&mut self) {
        let cmd = Section {
            time: self.get_block_end_time(),
            line_index: 0,
            index: self.section_counter,
        };

        self.section_counter += 1;

        self.current_time = cmd.time;
        self.block_start_time = self.current_time;
        self.block_max_time = 0.0;

        self.commands.push(Command::Section(cmd));
    }

    pub fn on_sync(&mut self) {
        self.on_rubi_base_end();

        let cmd = commands::Sync {
            time: self.get_block_end_time(),
            line_index: 0,
            index: self.sync_counter,
        };

        self.sync_counter += 1;

        self.current_time = cmd.time + 0.001;
        self.block_start_time = self.current_time;
        self.block_max_time = 0.0;

        self.commands.push(Command::Sync(cmd));
    }

    pub fn on_instant_start(&mut self) {
        self.on_rubi_base_end();
        self.is_inside_instant_text = true;
    }

    pub fn on_instant_end(&mut self) {
        self.on_rubi_base_end();
        self.is_inside_instant_text = false;
    }

    pub fn on_lipsync_enabled(&mut self) {
        self.lipsync_enabled = true;
    }

    pub fn on_lipsync_disabled(&mut self) {
        self.lipsync_enabled = false;
    }

    pub fn on_set_voice_volume(&mut self, volume: i32) {
        if volume < 0 {
            self.voice_volume = 1.0
        } else {
            self.voice_volume = parse_voice_volume(volume);
        }
    }

    pub fn on_voice(&mut self, voice_path: String) {
        self.on_rubi_base_end();

        let cmd = Voice {
            time: self.current_time,
            line_index: 0,
            filename: voice_path,
            volume: self.voice_volume,
            lipsync_enabled: self.lipsync_enabled,
            time_to_first_sync: 0,
        };

        self.current_time = cmd.time;
        self.block_start_time = self.current_time;
        self.block_max_time = 0.0;

        self.last_voice_or_voicesync_index = self.commands.len();

        self.commands.push(Command::Voice(cmd));
    }

    pub fn on_voice_sync(&mut self, target_instant: i32) {
        self.on_rubi_base_end();

        let cmd = VoiceSync {
            time: self.get_block_end_time(),
            line_index: 0,
            target_instant,
            time_to_next_sync: 0,
        };

        if self.last_voice_or_voicesync_index < self.commands.len() {
            match &mut self.commands[self.last_voice_or_voicesync_index] {
                Command::Voice(voice) => {
                    voice.time_to_first_sync = target_instant;
                }
                Command::VoiceSync(sync) => {
                    sync.time_to_next_sync = target_instant - sync.target_instant;
                }
                _ => {}
            }
        }

        self.current_time = cmd.time;
        self.block_start_time = self.current_time;
        self.block_max_time = 0.0;

        self.last_voice_or_voicesync_index = self.commands.len();

        self.commands.push(Command::VoiceSync(cmd));
    }

    pub fn on_voice_wait(&mut self) {
        let cmd = VoiceWait {
            time: self.get_block_end_time(),
            line_index: 0,
        };

        self.current_time = cmd.time + 0.001;
        self.block_start_time = self.current_time;
        self.block_max_time = 0.0;

        self.commands.push(Command::VoiceWait(cmd));
    }

    pub fn on_rubi_content(&mut self, content: String) {
        // NOTE: the original engine converts the rubi content to UTF-32 and stores it this way
        // I don't see the point, and it's non-trivial in rust (need a crate), so it's not done
        self.rubi_text = content;
    }

    pub fn on_rubi_base_start(&mut self) {
        if self.rubi_open {
            return;
        }

        self.rubi_open = true;
        self.rubi_start_cmd_index = self.commands.len();
        self.rubi_start_x = self.position.x;
        self.rubi_start_time = self.current_time;
    }

    pub fn on_rubi_base_end(&mut self) {
        if !self.rubi_open {
            return;
        }

        if self.rubi_text.is_empty() || self.rubi_start_x == self.position.x {
            self.rubi_open = false;
            return;
        }

        let rubi_base_commands = self.commands[self.rubi_start_cmd_index..]
            .iter_mut()
            .map(|command| {
                let Command::Char(char) = command else {
                    unreachable!();
                };
                char
            })
            .collect::<Vec<_>>();
        let mut rubi_commands = Vec::with_capacity(self.rubi_text.chars().count());

        let font = &self.font_normal;

        // first, lay out rubi commands "normally"
        let scale = self.params.rubi_size / (font.get_ascent() + font.get_descent()) as f32;
        let horizontal_scale = scale * self.params.base_font_horizontal_scale;
        let mut rubi_pos_x = 0.0;
        let mut rubi_time = 0.0;

        for codepoint in self.rubi_text.chars() {
            let glyph_info = font.get_glyph_info(codepoint).unwrap();

            let width = horizontal_scale * glyph_info.advance_width as f32;

            let cmd = Char {
                time: self.rubi_start_time + rubi_time,
                line_index: 0,
                codepoint,
                is_rubi: true,
                cant_be_at_line_start: false,
                cant_be_at_line_end: false,
                has_rubi: false,
                width,
                height: self.params.rubi_size,
                position: vec2(self.rubi_start_x + rubi_pos_x, self.position.y),
                horizontal_scale,
                scale,
                color: self.color_rgba,
                // NB: possible bug in the original engine - instant rubi text still has the fade, but the normal text doesn't
                fade: self.fade,
            };

            rubi_commands.push(cmd);

            if !self.is_inside_instant_text {
                rubi_time += width * self.draw_speed;
            }
            rubi_pos_x += width;
        }

        // now, check whether the base text or rubi text is wider and reflow the smaller one to match
        #[inline]
        fn reflow<'a>(
            mut iter: impl ExactSizeIterator<Item = &'a mut Char>,
            extra_width: f32,
            extra_time: f32,
        ) {
            let width_for_each = extra_width / (iter.len() + 1) as f32;
            let time_for_each = extra_time / (iter.len() + 1) as f32;
            let mut pos_x = width_for_each;
            let mut time = time_for_each;

            for item in iter {
                item.position.x += pos_x;
                item.time += time;

                pos_x += width_for_each;
                time += time_for_each;
            }
        }

        let rubi_width = rubi_pos_x;
        let rubi_time = rubi_time;

        let base_width = self.position.x - self.rubi_start_x;
        let base_time = self.current_time - self.rubi_start_time;

        if rubi_width <= base_width {
            // rubi text is smaller
            reflow(
                rubi_commands.iter_mut(),
                base_width - rubi_width,
                base_time - rubi_time,
            );

            // we've reflown the rubi text, this doesn't affect the cursor position and time
        } else {
            // base text is smaller
            reflow(
                rubi_base_commands.into_iter(),
                rubi_width - base_width,
                rubi_time - base_time,
            );

            // reflect the changes in position and time
            self.position.x = self.rubi_start_x + rubi_width;
            self.current_time = self.rubi_start_time + rubi_time;
        }

        // append the rubi commands to the main list
        self.commands
            .extend(rubi_commands.into_iter().map(Command::Char));

        self.rubi_open = false;
    }

    pub fn on_bold_start(&mut self) {
        self.is_bold = true;
    }

    pub fn on_bold_end(&mut self) {
        self.is_bold = false;
    }

    pub fn finalize_up_to(&mut self, finalize_index: usize, is_hard_break: bool) {
        let new_commands = &mut self.commands[self.finalized_command_count..finalize_index];

        // useful for debugging & comparing with the actual game
        // eprintln!(
        //     "finalize_up_to({}, {}) [{}] [\n{}\n]",
        //     finalize_index,
        //     is_hard_break,
        //     self.lines.len(),
        //     new_commands.iter().map(|v| format!("{:?}", v)).join(",\n")
        // );

        let mut max_width = 0.0f32;
        let mut line_height = 0.0f32;
        let mut rubi_height = 0.0f32;
        let mut char_count = 0;
        if !new_commands.is_empty() {
            for cmd in new_commands.iter() {
                if let Command::Char(char) = cmd {
                    if char.is_rubi {
                        rubi_height = self.params.rubi_size;
                        max_width = max_width.max(char.position.x + char.width);
                    } else {
                        line_height = line_height.max(char.height);
                        max_width = max_width.max(char.position.x + char.width);

                        if self.params.always_leave_space_for_rubi {
                            rubi_height = self.params.rubi_size;
                        }

                        char_count += 1;
                    };
                }
            }
        }

        if char_count == 0 {
            line_height = self.params.text_size * self.font_scale;
            if self.params.always_leave_space_for_rubi {
                // NB: rubi height is not scaled
                rubi_height = self.params.rubi_size;
            }
        }

        // These variables are no longer going to change
        let layout_width = self.params.layout_width;

        let max_width = max_width;
        let mut line_width = max_width; // can change due to overflow or justification
        let line_height = line_height;
        let rubi_height = rubi_height;
        let char_count = char_count;

        let ascent = self.font_normal.get_ascent() as f32;
        let descent = self.font_normal.get_descent() as f32;

        let ascent_scaled = if char_count == 0 {
            // baseline calculation seems wrong if there are no characters
            // weird...
            self.params.text_size
        } else {
            line_height
        } / (ascent + descent)
            * ascent;
        let rubi_ascent_scaled = rubi_height / (ascent + descent) * ascent;

        if char_count > 0 {
            if max_width >= layout_width {
                // light overflow (the code in `on_newline` we won't allow more than 5%)
                // squish the line to fit
                let squish = layout_width / max_width;
                // eprintln!(
                //     "Squishing line to fit: {} -> {}; squish={}",
                //     max_width, layout_width, squish
                // );
                for cmd in new_commands.iter_mut() {
                    if let Command::Char(char) = cmd {
                        char.position.x *= squish;
                        char.width *= squish;
                        char.horizontal_scale *= squish;
                    }
                }

                // we've used the full line, override the line width
                line_width = layout_width;
            } else if !is_hard_break
                && self.params.text_alignment == MessageTextLayout::Justify
                && layout_width - max_width < layout_width * 0.05
            {
                // eprintln!("Justifying line to fit: {} -> {}", max_width, layout_width);
                // justify the non-last line characters if requested
                for cmd in new_commands.iter_mut() {
                    if let Command::Char(char) = cmd {
                        let x_pos = char.position.x;
                        char.position.x = (self.params.layout_width - char.width)
                            * (x_pos / (x_pos + (max_width - (x_pos + char.width))));
                    }
                }

                // we've used the full line, override the line width
                line_width = layout_width;
            }

            let x_offset = match self.params.text_alignment {
                MessageTextLayout::Center => (layout_width - line_width) / 2.0,
                MessageTextLayout::Right => layout_width - line_width,
                _ => 0.0,
            };

            for cmd in new_commands.iter_mut() {
                if let Command::Char(char) = cmd {
                    char.position.x += x_offset;
                    char.position.y += self.params.line_spacing;
                    char.position.y += if char.is_rubi {
                        rubi_ascent_scaled
                    } else {
                        rubi_height + ascent_scaled
                    };
                }
            }
        }

        for cmd in new_commands.iter_mut() {
            cmd.set_line_index(self.lines.len());
        }

        self.lines.push(LineInfo {
            width: line_width,
            y_position: self.position.y,
            line_advance: self.params.line_spacing
                + rubi_height
                + line_height
                + self.params.another_line_height,
            total_height: self.params.line_spacing + rubi_height + ascent_scaled,
            rubi_height,
        });

        self.size.x = self.size.x.max(line_width);

        let line_advance =
            self.params.line_spacing + rubi_height + line_height + self.params.another_line_height;
        let line_advance_final = line_advance + self.params.line_height3;

        self.size.y = self.position.y + line_advance;

        // move the characters after the finalized ones to the next line
        {
            let mut is_first_character = true;
            let mut negative_offset = max_width; // we are interested in the virtual line width before the overflow/justification, not the actual size
            for cmd in &mut self.commands[finalize_index..] {
                if let Command::Char(char) = cmd {
                    // eat space at the start of the newline
                    if is_first_character && !char.has_rubi && char.codepoint == '　' {
                        // U+3000 IDEOGRAPHIC SPACE
                        negative_offset += char.width;
                        char.width = 0.0;
                        char.horizontal_scale = 0.0;
                    }
                    is_first_character = false;

                    char.position.x -= negative_offset;
                    char.position.y += line_advance_final;
                }
            }
        }

        // NB: it's weird that the width of eaten space does not get subtracted here
        self.position.x -= max_width;
        self.position.y += line_advance_final;
        self.finalized_command_count = finalize_index;
    }
}

// Rust doesn't have 1:1 correspondence to inheritance with virtual methods, so we use the next best thing: mixins.
// This works because the inheritance hierarchy is only 2 classes high. This approach is not scalable to deeper hierarchies as-is.
pub trait MessageTextLayouterMixin<Font> {
    fn on_char(&mut self, layouter: &mut MessageTextLayouterImpl<Font>, codepoint: char);
    fn on_newline(&mut self, layouter: &mut MessageTextLayouterImpl<Font>);
    fn on_voice(&mut self, layouter: &mut MessageTextLayouterImpl<Font>, voice_path: String);
    fn finalize_up_to(
        &mut self,
        layouter: &mut MessageTextLayouterImpl<Font>,
        finalize_index: usize,
        is_hard_break: bool,
    );
}

pub struct MessageTextLayouterWithMixin<Font, M> {
    layouter: MessageTextLayouterImpl<Font>,
    mixin: M,
}

impl<Font: FontMetrics, M: MessageTextLayouterMixin<Font>> TextLayouter
    for MessageTextLayouterWithMixin<Font, M>
{
    fn on_message_start(&mut self) {
        self.layouter.on_message_start()
    }

    fn on_message_end(&mut self) {
        self.layouter.on_message_end(&mut self.mixin)
    }

    fn on_char(&mut self, codepoint: char) {
        self.mixin.on_char(&mut self.layouter, codepoint)
    }

    fn on_newline(&mut self) {
        self.mixin.on_newline(&mut self.layouter)
    }

    fn on_click_wait(&mut self) {
        self.layouter.on_click_wait()
    }

    fn on_auto_click(&mut self) {
        self.layouter.on_auto_click()
    }

    fn on_set_font_scale(&mut self, scale: i32) {
        self.layouter.on_set_font_scale(scale)
    }

    fn on_set_color(&mut self, color: i32) {
        self.layouter.on_set_color(color)
    }

    fn on_set_draw_speed(&mut self, speed: i32) {
        self.layouter.on_set_draw_speed(speed)
    }

    fn on_set_fade(&mut self, fade: i32) {
        self.layouter.on_set_fade(fade)
    }

    fn on_wait(&mut self, delay: i32) {
        self.layouter.on_wait(delay)
    }

    fn on_start_parallel(&mut self) {
        self.layouter.on_start_parallel()
    }

    fn on_section(&mut self) {
        self.layouter.on_section()
    }

    fn on_sync(&mut self) {
        self.layouter.on_sync()
    }

    fn on_instant_start(&mut self) {
        self.layouter.on_instant_start()
    }

    fn on_instant_end(&mut self) {
        self.layouter.on_instant_end()
    }

    fn on_lipsync_enabled(&mut self) {
        self.layouter.on_lipsync_enabled()
    }

    fn on_lipsync_disabled(&mut self) {
        self.layouter.on_lipsync_disabled()
    }

    fn on_set_voice_volume(&mut self, volume: i32) {
        self.layouter.on_set_voice_volume(volume)
    }

    fn on_voice(&mut self, voice_path: String) {
        self.mixin.on_voice(&mut self.layouter, voice_path)
    }

    fn on_voice_sync(&mut self, target_instant: i32) {
        self.layouter.on_voice_sync(target_instant)
    }

    fn on_voice_wait(&mut self) {
        self.layouter.on_voice_wait()
    }

    fn on_rubi_content(&mut self, content: String) {
        self.layouter.on_rubi_content(content)
    }

    fn on_rubi_base_start(&mut self) {
        self.layouter.on_rubi_base_start()
    }

    fn on_rubi_base_end(&mut self) {
        self.layouter.on_rubi_base_end()
    }

    fn on_bold_start(&mut self) {
        self.layouter.on_bold_start()
    }

    fn on_bold_end(&mut self) {
        self.layouter.on_bold_end()
    }
}

pub type MessageTextLayouter<Font> = MessageTextLayouterWithMixin<Font, mixins::NoMixin>;

impl<Font> MessageTextLayouter<Font> {
    pub fn new(
        font_normal: Font,
        font_bold: Font,
        layout_params: LayoutParams,
        defaults: MessageTextLayouterDefaults,
    ) -> Self {
        Self {
            layouter: MessageTextLayouterImpl::new(font_normal, font_bold, layout_params, defaults),
            mixin: mixins::NoMixin,
        }
    }
}

pub type MessageLayerLayouter<Font> =
    MessageTextLayouterWithMixin<Font, mixins::MessageLayerLayouterMixin>;

impl<Font> MessageLayerLayouter<Font> {
    pub fn new(
        font_normal: Font,
        font_bold: Font,
        messagebox_type: MessageboxType,
        layout_params: LayoutParams,
        defaults: MessageTextLayouterDefaults,
    ) -> Self {
        Self {
            layouter: MessageTextLayouterImpl::new(font_normal, font_bold, layout_params, defaults),
            mixin: mixins::MessageLayerLayouterMixin::new(messagebox_type),
        }
    }
}
#[cfg(test)]
mod tests;
