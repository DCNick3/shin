use glam::Vec2;
use shin_core::primitives::color::UnormColor;
use shin_derive::RenderClone;

use crate::asset::font::GpuFontGlyphHandle;

#[derive(Debug, Clone, RenderClone)]
pub struct LineInfo {
    pub y_position: f32,
    pub baseline_ascent: f32,
    pub line_height: f32,
    pub rubi_height: f32,
    /// Set to 1.0 when any of the chars on the line are visible
    ///
    /// This field is seemingly unused
    pub is_visible: f32,
}

#[derive(Debug, Clone, RenderClone)]
pub struct Char {
    pub time: f32,
    pub line_index: usize,
    pub is_rubi: bool,
    pub position: Vec2,
    pub width: f32,
    pub height: f32,
    pub horizontal_scale: f32,
    pub vertical_scale: f32,
    pub color_rgba: UnormColor,
    pub progress_rate: f32,
    pub current_progress: f32,
    pub block_index: usize,
    pub vertex_buffer_offset: usize,
    pub border_distances: [Vec2; 8],
    pub glyph: GpuFontGlyphHandle,
}

impl Char {
    pub fn scale(&self) -> Vec2 {
        Vec2::new(self.horizontal_scale, self.vertical_scale)
    }
}
