use crate::format::font::{Font, GlyphTrait};
use crate::layout::{LayouterCommand, LayouterParser};
use crate::vm::command::layer::MessageTextLayout;
use crate::vm::command::time::Ticks;
use cgmath::{Vector2, Vector3};

#[derive(Debug, Clone, Copy)]
pub enum CommandInner {
    Char {
        position: Vector2<f32>,
        color: Vector3<f32>,
        size: GlyphSize,
        fade_time: f32,
        codepoint: u16,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Command {
    pub time: Ticks,
    pub inner: CommandInner,
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

#[derive(Debug, Copy, Clone)]
pub struct GlyphSize {
    pub scale: f32,
    pub vertical_scale: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Copy, Clone)]
pub struct LayoutParams<'a, G: GlyphTrait> {
    pub font: &'a Font<G>,
    pub layout_width: f32,
    pub base_font_height: f32,
    pub font_horizontal_base_scale: f32,
    pub text_layout: MessageTextLayout,
    pub default_state: LayouterState,
    pub has_character_name: bool,
}

impl<'a, G: GlyphTrait> LayoutParams<'a, G> {
    fn glyph_size(&self, font_size: f32, codepoint: u16) -> GlyphSize {
        let height = self.base_font_height * font_size;
        let scale = height / self.font.get_line_height() as f32;
        let vertical_scale = scale * self.font_horizontal_base_scale;
        let glyph = self.font.get_glyph_for_character(codepoint).get_info();
        let width = glyph.advance_width as f32 * vertical_scale;

        GlyphSize {
            scale,
            vertical_scale,
            width,
            height,
        }
    }
}

fn layout_text<G: GlyphTrait>(params: LayoutParams<G>, text: &str) -> Vec<Command> {
    let LayoutParams {
        font,
        layout_width,
        base_font_height,
        font_horizontal_base_scale,
        text_layout,
        default_state,
        has_character_name,
    } = params;

    let mut parser = LayouterParser::new(text).peekable();

    let mut commands = Vec::new();
    let mut state = default_state;
    let mut position = Vector2::new(0.0, 0.0);
    let mut time = Ticks::ZERO;

    // NOTE: the first line is always the character name, even if the message box does not show it
    // (it's ignored for that case)
    // TODO: actually parse anything in the character name
    // TODO: how are state changes handled in the character name? Are they preserved after the name is layouted?
    // font size for the character name is 0.9
    for command in parser.by_ref() {
        if matches!(command, LayouterCommand::Newline) {
            break;
        }
    }
    if parser.peek().is_none() {
        return commands;
    }

    for command in parser.by_ref() {
        match command {
            LayouterCommand::Char(c) => {
                assert!((c as u32) < 0x10000);
                let codepoint = c as u16;
                let size = params.glyph_size(state.font_size, codepoint);
                let fade_time = state.text_draw_speed * size.width;

                // TODO: handle Table1 and Table2 (smth related to parenthesis and their different forms)

                commands.push(Command {
                    time,
                    inner: CommandInner::Char {
                        position,
                        color: state.text_color,
                        size,
                        fade_time,
                        codepoint,
                    },
                });

                position.x += size.width;
                time += Ticks(state.text_draw_speed * size.width);

                // TODO: handle full stops (they add more delay)

                // TODO: where are overflows handled? On the linefeed?
            }
            LayouterCommand::EnableLipsync => todo!(),
            LayouterCommand::DisableLipsync => todo!(),
            LayouterCommand::Furigana(_) => todo!(),
            LayouterCommand::FuriganaStart => todo!(),
            LayouterCommand::FuriganaEnd => todo!(),
            LayouterCommand::SetFade(_) => todo!(),
            LayouterCommand::SetColor(_) => todo!(),
            LayouterCommand::AutoClick => todo!(),
            LayouterCommand::WaitClick => todo!(),
            LayouterCommand::VoiceVolume(_) => todo!(),
            LayouterCommand::Newline => todo!(),
            LayouterCommand::TextSpeed(_) => todo!(),
            LayouterCommand::SimultaneousStart => todo!(),
            LayouterCommand::Voice(_) => todo!(),
            LayouterCommand::Wait(_) => todo!(),
            LayouterCommand::Sync => todo!(),
            LayouterCommand::FontSize(_) => todo!(),
            LayouterCommand::Signal => todo!(),
            LayouterCommand::InstantTextStart => todo!(),
            LayouterCommand::InstantTextEnd => todo!(),
            LayouterCommand::BoldTextStart => todo!(),
            LayouterCommand::BoldTextEnd => todo!(),
        }
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_simple() {
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
            default_state: LayouterState {
                font_size: 1.0,
                text_color: Vector3::new(1.0, 1.0, 1.0),
                text_draw_speed: 1.0,
                fade: 1.0,
            },
            has_character_name: true,
        };

        let result = layout_text(params, "@rHello, world!");
        println!("{:#?}", result);
    }
}
