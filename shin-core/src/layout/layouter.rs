use crate::format::font::{GlyphTrait, LazyFont};
use crate::layout::parser::{LayouterParser, ParsedCommand};
use crate::vm::command::layer::MessageTextLayout;
use crate::vm::command::time::Ticks;
use cgmath::{Vector2, Vector3};
use float_ord::FloatOrd;
use std::iter::Peekable;
use tracing::warn;

#[derive(Debug, Clone, Copy)]
pub struct CharCommand {
    pub time: Ticks,
    pub position: Vector2<f32>,
    pub color: Vector3<f32>,
    pub size: GlyphSize,
    pub fade: f32,
    pub codepoint: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Char(CharCommand),
}

impl Command {
    pub fn time(&self) -> Ticks {
        match self {
            Command::Char(c) => c.time,
        }
    }
}

/// Represents layouter state directly settable by the user.
#[derive(Debug, Copy, Clone)]
pub struct LayouterState {
    /// Font size, in relative units (0.1 - 2.0)
    pub font_size: f32,
    pub text_color: Vector3<f32>,
    /// Text draw speed (well, actually it's time to draw one pixel)
    pub text_draw_speed: f32,
    pub fade: f32,
}

impl Default for LayouterState {
    fn default() -> Self {
        Self {
            font_size: 1.0,
            text_color: Vector3::new(1.0, 1.0, 1.0),
            // TODO: those are not correct
            // TODO: make those into newtypes
            text_draw_speed: 0.1,
            fade: 0.01,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct GlyphSize {
    pub scale: f32,
    pub horizontal_scale: f32,
    pub advance_width: f32,
    pub line_height: f32,
    pub width: f32,
    pub height: f32,
}

impl GlyphSize {
    pub fn size(&self) -> Vector2<f32> {
        Vector2::new(self.width, self.height)
    }

    pub fn scale_horizontal(&mut self, scale: f32) {
        self.advance_width *= scale;
        self.width *= scale;
        self.horizontal_scale *= scale;
    }
}

#[derive(Copy, Clone)]
pub struct LayoutParams<'a> {
    pub font: &'a LazyFont,
    pub layout_width: f32,
    pub base_font_height: f32,
    pub furigana_font_height: f32,
    pub font_horizontal_base_scale: f32,
    pub text_layout: MessageTextLayout,
    pub default_state: LayouterState,
    pub has_character_name: bool,
}

impl<'a> LayoutParams<'a> {
    fn glyph_size(&self, font_size: f32, codepoint: u16) -> GlyphSize {
        let line_height = self.base_font_height * font_size;
        let scale = line_height / self.font.get_line_height() as f32;
        let horizontal_scale = scale * self.font_horizontal_base_scale;

        let glyph = self.font.get_glyph_for_character(codepoint).get_info();
        let height = glyph.actual_height as f32 * scale;
        let width = glyph.actual_width as f32 * horizontal_scale;
        let advance_width = glyph.advance_width as f32 * horizontal_scale;

        GlyphSize {
            scale,
            horizontal_scale,
            advance_width,
            line_height,
            width,
            height,
        }
    }
}

struct Layouter<'a> {
    parser: Peekable<LayouterParser<'a>>,
    params: LayoutParams<'a>,
    state: LayouterState,
    commands: Vec<Command>,
    pending_chars: Vec<CharCommand>,
    position: Vector2<f32>,
    time: Ticks,
}

impl<'a> Layouter<'a> {
    fn on_char(&mut self, c: char) {
        assert!((c as u32) < 0x10000);
        let codepoint = c as u16;
        let size = self.params.glyph_size(self.state.font_size, codepoint);
        let fade_time = self.state.text_draw_speed * size.width;

        // TODO: handle special cases for brackets
        // TODO: handle furigana

        self.pending_chars.push(CharCommand {
            time: self.time,
            position: Vector2::new(self.position.x, 0.0), // do not set y position yet, it will be set when we know which line this char is on
            color: self.state.text_color,
            size,
            fade: fade_time,
            codepoint,
        });

        self.position.x += size.advance_width;
        self.time += Ticks(self.state.text_draw_speed * size.advance_width);

        // TODO: handle full stops (they add more delay)

        // TODO: where are overflows handled? On the linefeed?
    }

    fn finalize_line(&mut self, chars: &[CharCommand], last_line: bool, x_pos: f32) {
        if chars.is_empty() {
            return;
        }

        // TODO: there are flags.... I think they have to do with difference between text alignment 0 & 1

        let max_line_height = chars
            .iter()
            .map(|c| FloatOrd(c.size.line_height))
            .max()
            .unwrap()
            .0;
        let furigana_height = self.params.furigana_font_height; // TODO: there is an "always leave space for furigana" flag
        let width = chars
            .iter()
            .map(|c| FloatOrd(c.position.x + c.size.advance_width))
            .max()
            .unwrap()
            .0
            - x_pos;
        // let start_x = chars
        //     .iter()
        //     .map(|c| FloatOrd(c.position.x))
        //     .min()
        //     .unwrap()
        //     .0;

        // if we are not the last line, we haven't overflowed yet
        let should_stretch = !last_line
            && self.params.layout_width > width
            && self.params.text_layout == MessageTextLayout::Left
            && self.params.layout_width - width < self.params.layout_width * 0.05;

        let fit_scale = if !last_line {
            // if we are not at the last line, the line should be full
            // and usually this means that it has overflowed
            // squish text a bit to make it fit (probably more visually pleasing?)
            self.params.layout_width / width
        } else {
            1.0
        };

        let font = self.params.font;

        let line_ascent =
            (max_line_height / font.get_line_height() as f32) * font.get_ascent() as f32;

        // TODO: handle hiragana
        // TODO: handle special cases for brackets

        let x_offset = match self.params.text_layout {
            MessageTextLayout::Left => 0.0,
            MessageTextLayout::Layout1 => 0.0,
            MessageTextLayout::Center => (self.params.layout_width - width) / 2.0,
            MessageTextLayout::Right => self.params.layout_width - width,
        };

        self.commands.extend(
            chars
                .iter()
                .cloned()
                .map(|mut c| {
                    // move the text to the beginning of the real line
                    // x might be larger than we want if an overflow happened
                    c.position.x -= x_pos;

                    // align the text according to the layout params
                    c.position.x += x_offset;

                    // move the glyph on its line y coordinate (previously it was zero)
                    c.position.y += self.position.y;
                    // make sure that the glyph is on the baseline (doing it here because font size might change on the line)
                    c.position.y += line_ascent;
                    // leave space for furigana
                    // TODO: we, obviously, should not do this when there is no furigana
                    c.position.y += furigana_height;

                    // if we are overflowing - make it fit by squishing the text
                    c.position.x *= fit_scale;
                    c.size.scale_horizontal(fit_scale);

                    // if needed - make the text fit by stretching it
                    if should_stretch {
                        // I don't get this formula...
                        // also it seems to do something strange
                        // TODO: figure this stuff out
                        // c.position.x = (self.params.layout_width - c.size.width)
                        //     * (self.position.x
                        //         / (self.position.x + (width - (self.position.x + c.size.width))));
                    }
                    c
                })
                .map(Command::Char),
        );

        self.position.x = 0.0;
        self.position.y += max_line_height + furigana_height + 4.0 /* TODO: this is one of the many obscure line height-type parameters */;
    }

    fn on_newline(&mut self) {
        let chars = std::mem::take(&mut self.pending_chars);

        // split into lines on overflows
        // TODO: implement word wrapping?
        let mut start = 0;
        let mut x_pos = 0.0;
        for (i, c) in chars.iter().enumerate() {
            // if the start of the character is outside of the layout width
            if c.position.x - x_pos > self.params.layout_width
                // or if the end of the character is outside of the layout width * 1.05
                || c.position.x + c.size.width - x_pos > self.params.layout_width * 1.05
            /* allow a bit of overflow, the chars will be rescaled */
            {
                self.finalize_line(&chars[start..i], false, x_pos);
                x_pos = c.position.x;
                start = i;
            }
        }

        // TODO: handle overflows
        self.finalize_line(&chars[start..], true, x_pos);
        self.pending_chars.clear();
    }

    fn finalize(mut self) -> Vec<Command> {
        // TODO: close furigana
        self.on_newline();
        self.commands
    }
}

pub enum BlockExitCondition {
    /// Wait for user to press "Advance" button
    ClickWait,
    /// Wait for the VM to signal us with MSGSIGNAL command
    /// The number specified the signal number (they are counted consecutively, from 0)
    Signal(u32),
    /// Do not wait for anything to leave this block, just go to the next one (or finish the message)
    None,
}

/// One message is split into multiple blocks.
/// _Usually_ blocks are separated by click-wait commands (@k)
/// but sometimes they are separated by sync commands (@y)
///
/// Blocks are also kind of a single skip-able unit:
///  by pressing "Advance" button you skip the whole block.
///
/// For... reasons, blocks live separately from the characters,
///  their boundaries instead defined by the time they start and end.
///
/// They also contain exit conditions, which are used to determine
///  when the next block (or message) should be executed.
pub struct Block {
    pub exit_condition: BlockExitCondition,
    pub start_time: Ticks,
    pub end_time: Ticks,
}

impl Block {
    pub fn completed(&self, time: Ticks) -> bool {
        time >= self.end_time
    }
}

struct BlockBuilder {
    final_wait: bool,
    time_start: Ticks,
    signal_number: u32,
    blocks: Vec<Block>,
}

impl BlockBuilder {
    /// An artificial gap between blocks
    /// Needed because we don't want the fade-in of the next block to interfere   
    const TIME_GAP: Ticks = Ticks(1000.0);

    fn new() -> Self {
        Self {
            final_wait: true,
            time_start: Ticks(0.0),
            signal_number: 0,
            blocks: Vec::new(),
        }
    }

    fn click_wait(&mut self, time: &mut Ticks) {
        self.blocks.push(Block {
            exit_condition: BlockExitCondition::ClickWait,
            start_time: self.time_start,
            end_time: *time,
        });
        *time += Self::TIME_GAP;
        self.time_start = *time;
    }

    fn sync(&mut self, time: &mut Ticks) {
        self.blocks.push(Block {
            exit_condition: BlockExitCondition::Signal(self.signal_number),
            start_time: self.time_start,
            end_time: *time,
        });
        self.signal_number += 1;
        *time += Self::TIME_GAP;
        self.time_start = *time;
    }

    fn no_final_wait(&mut self) {
        self.final_wait = false;
    }

    fn finalize(mut self, time: Ticks) -> Vec<Block> {
        self.blocks.push(Block {
            exit_condition: if self.final_wait {
                BlockExitCondition::ClickWait
            } else {
                BlockExitCondition::None
            },
            start_time: self.time_start,
            end_time: time,
        });
        self.blocks
    }
}

pub fn layout_text(params: LayoutParams, text: &str) -> (Vec<Command>, Vec<Block>) {
    let mut layouter = Layouter {
        parser: LayouterParser::new(text).peekable(),
        params,
        state: params.default_state,
        commands: Vec::new(),
        pending_chars: Vec::new(),
        position: Vector2::new(0.0, 0.0),
        time: Ticks(0.0),
    };

    let mut block_builder = BlockBuilder::new();

    // NOTE: the first line is always the character name, even if the message box does not show it
    // (it's ignored for that case)
    // TODO: actually parse anything in the character name
    // TODO: how are state changes handled in the character name? Are they preserved after the name is layouted?
    // font size for the character name is 0.9
    for command in layouter.parser.by_ref() {
        if matches!(command, ParsedCommand::Newline) {
            break;
        }
    }
    if layouter.parser.peek().is_some() {
        // Not using a for loop because of borrow checker
        while let Some(command) = layouter.parser.next() {
            match command {
                ParsedCommand::Char(c) => layouter.on_char(c),
                ParsedCommand::EnableLipsync => todo!(),
                ParsedCommand::DisableLipsync => todo!(),
                ParsedCommand::Furigana(_) => warn!("Furigana layout command is not implemented"),
                ParsedCommand::FuriganaStart => {
                    warn!("FuriganaStart layout command is not implemented")
                }
                ParsedCommand::FuriganaEnd => {
                    warn!("FuriganaEnd layout command is not implemented")
                }
                ParsedCommand::SetFade(_) => todo!(),
                ParsedCommand::SetColor(_) => todo!(),
                ParsedCommand::NoFinalClickWait => block_builder.no_final_wait(),
                ParsedCommand::ClickWait => block_builder.click_wait(&mut layouter.time),
                ParsedCommand::VoiceVolume(_) => todo!(),
                ParsedCommand::Newline => layouter.on_newline(),
                ParsedCommand::TextSpeed(_) => todo!(),
                ParsedCommand::SimultaneousStart => todo!(),
                ParsedCommand::Voice(_) => warn!("Voice layout command not implemented"),
                ParsedCommand::Wait(_) => todo!(),
                ParsedCommand::Sync => block_builder.sync(&mut layouter.time),
                ParsedCommand::FontSize(_) => todo!(),
                ParsedCommand::Signal => todo!(),
                ParsedCommand::InstantTextStart => todo!(),
                ParsedCommand::InstantTextEnd => todo!(),
                ParsedCommand::BoldTextStart => todo!(),
                ParsedCommand::BoldTextEnd => todo!(),
            }
        }
    }

    let blocks = block_builder.finalize(layouter.time);
    let commands = layouter.finalize();
    // TODO: separate Chars, Actions and Blocks
    return (commands, blocks);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;

    fn test_layout(text: &str) -> Vec<Command> {
        // NOTICE: here we need to use a font
        // it is an asset, so we need to load it from __somewhere__
        // having tests that depend on assets is not ideal
        // maybe I can create my own font for testing purposes?
        // use the one from assets for now
        let font = File::open("../shin/assets/data/newrodin-medium.fnt").unwrap();
        let mut font = BufReader::new(font);
        let font = shin_core::format::font::read_lazy_font(&mut font).unwrap();

        let params = LayoutParams {
            font: &font,
            layout_width: 1500.0,
            base_font_height: 50.0,
            furigana_font_height: 20.0,
            font_horizontal_base_scale: 0.9696999788284302,
            text_layout: MessageTextLayout::Left,
            default_state: LayouterState::default(),
            has_character_name: true,
        };

        // TODO: test the blocks
        let (commands, _blocks) = layout_text(params, text);

        commands
    }

    #[test]
    fn test_simple() {
        let result = test_layout("@rHello, world!");
        println!("{:#?}", result);
    }

    #[test]
    fn test_tsu() {
        let result = test_layout(
            "@r埃と甘ったるい異臭の入り混じった薄暗い書斎に、年輩の男たちの姿はあった。",
        );

        let tsu = result[3];
        if let Command::Char(c) = tsu {
            assert_eq!(c.codepoint, 'っ' as u16);

            const EXPECTED_ASPECT_RATIO: f32 = 104.0 / 80.0;
            let aspect_ratio = c.size.width / c.size.height;

            // divide max by min to get the ratio between aspect ratios
            let ratio =
                aspect_ratio.max(EXPECTED_ASPECT_RATIO) / aspect_ratio.min(EXPECTED_ASPECT_RATIO);
            // the ratio will always be larger than 1 and should be close to 1
            assert!(ratio < 1.09);
        } else {
            panic!("Expected a char command");
        }

        // the の should still be on the first line
        if let Command::Char(c) = result[29] {
            assert_eq!(c.codepoint, 'の' as u16);
            assert_eq!(c.position.y, 40.625);
        } else {
            panic!("Expected a char command");
        }

        // while the 姿 should be on the second line
        if let Command::Char(c) = result[30] {
            assert_eq!(c.codepoint, '姿' as u16);
            assert!(c.position.y > 40.625);
        } else {
            panic!("Expected a char command");
        }
    }
}
