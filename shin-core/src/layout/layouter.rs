use cgmath::{Vector2, Vector3};
use shin_core::format::font::{Font, GlyphTrait};
use shin_core::layout::{LayouterCommand, LayouterParser};
use shin_core::vm::command::layer::MessageTextLayout;

pub enum CommandInner {
    Char {
        position: Vector2<f32>,
        color: Vector3<f32>,
        size: f32,
        fade_speed: f32,
        character: char,
    },
}

pub struct Command {
    pub time: f32, // TODO: ticks type
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

pub struct LayoutParams<'a, G: GlyphTrait> {
    pub font: &'a Font<G>,
    pub layout_width: f32,
    pub base_font_size: f32,
    pub text_layout: MessageTextLayout,
    pub default_state: LayouterState,
    pub has_character_name: bool,
}

impl<'a, G: GlyphTrait> LayoutParams<'a, G> {
    // fn
}

fn layout_text<G: GlyphTrait>(params: LayoutParams<G>, text: &str) -> Vec<Command> {
    let LayoutParams {
        font,
        layout_width,
        base_font_size,
        text_layout,
        default_state,
        has_character_name,
    } = params;

    let mut parser = LayouterParser::new(text).peekable();

    let mut commands = Vec::new();
    let mut state = default_state;
    let mut position = Vector2::new(0.0, 0.0);

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

    commands
}
