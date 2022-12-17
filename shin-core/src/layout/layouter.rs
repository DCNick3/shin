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
}

#[derive(Copy, Clone)]
pub struct LayoutParams<'a> {
    pub font: &'a LazyFont,
    pub layout_width: f32,
    pub base_font_height: f32,
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
            position: self.position,
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

    fn finalize_line(&mut self, chars: &[CharCommand]) {
        if chars.is_empty() {
            return;
        }

        // TODO: there are flags.... I think they have to do with difference between text alignment 0 & 1

        // TODO: handle text alignment
        assert_eq!(self.params.text_layout, MessageTextLayout::Left);

        let max_line_height = chars
            .iter()
            .map(|c| FloatOrd(c.size.line_height))
            .max()
            .unwrap()
            .0;
        let _width = chars
            .iter()
            .map(|c| FloatOrd(c.position.x + c.size.width))
            .max()
            .unwrap()
            .0;

        let font = self.params.font;

        let line_ascent =
            (max_line_height / font.get_line_height() as f32) * font.get_ascent() as f32;

        // TODO: adjust vertical scale if the overflow is small
        // TODO: handle hiragana
        // TODO: handle special cases for brackets

        self.commands.extend(
            chars
                .iter()
                .cloned()
                .map(|mut c| {
                    c.position.y += line_ascent;
                    c
                })
                .map(Command::Char),
        );

        self.position.x = 0.0;
        self.position.y += max_line_height;
    }

    fn on_newline(&mut self) {
        let chars = std::mem::take(&mut self.pending_chars);
        // TODO: handle overflows
        self.finalize_line(&chars);
        self.pending_chars.clear();
    }

    fn finalize(mut self) -> Vec<Command> {
        // TODO: close furigana
        self.on_newline();
        self.commands
    }
}

pub fn layout_text(params: LayoutParams, text: &str) -> Vec<Command> {
    let mut layouter = Layouter {
        parser: LayouterParser::new(text).peekable(),
        params,
        state: params.default_state,
        commands: Vec::new(),
        pending_chars: Vec::new(),
        position: Vector2::new(0.0, 0.0),
        time: Ticks(0.0),
    };

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
    if layouter.parser.peek().is_none() {
        return layouter.finalize();
    }

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
            ParsedCommand::FuriganaEnd => warn!("FuriganaEnd layout command is not implemented"),
            ParsedCommand::SetFade(_) => todo!(),
            ParsedCommand::SetColor(_) => todo!(),
            ParsedCommand::AutoClick => todo!(),
            ParsedCommand::WaitClick => warn!("WaitClick layout command not implemented"),
            ParsedCommand::VoiceVolume(_) => todo!(),
            ParsedCommand::Newline => layouter.on_newline(),
            ParsedCommand::TextSpeed(_) => todo!(),
            ParsedCommand::SimultaneousStart => todo!(),
            ParsedCommand::Voice(_) => warn!("Voice layout command not implemented"),
            ParsedCommand::Wait(_) => todo!(),
            ParsedCommand::Sync => todo!(),
            ParsedCommand::FontSize(_) => todo!(),
            ParsedCommand::Signal => todo!(),
            ParsedCommand::InstantTextStart => todo!(),
            ParsedCommand::InstantTextEnd => todo!(),
            ParsedCommand::BoldTextStart => todo!(),
            ParsedCommand::BoldTextEnd => todo!(),
        }
    }

    layouter.finalize()
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
        let font = File::open("../shin/assets/data/system.fnt").unwrap();
        let mut font = BufReader::new(font);
        let font = shin_core::format::font::read_lazy_font(&mut font).unwrap();

        let params = LayoutParams {
            font: &font,
            layout_width: 1050.0,
            base_font_height: 50.0,
            font_horizontal_base_scale: 0.9696999788284302,
            text_layout: MessageTextLayout::Left,
            default_state: LayouterState::default(),
            has_character_name: true,
        };

        layout_text(params, text)
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
            assert!(ratio < 1.05);
            todo!()
        } else {
            panic!("Expected a char command");
        }
    }
}
