use crate::update::Ticks;
use cgmath::{Vector2, Vector3};

pub enum CommandInner {
    Char {
        position: Vector2<f32>,
        color: Vector3<f32>,
        size: f32,
        fade_speed: f32,
        glyph: char,
    },
}

pub struct Command {
    pub time: Ticks,
    pub inner: CommandInner,
}
