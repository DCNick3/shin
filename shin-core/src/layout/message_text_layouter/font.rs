use std::sync::Arc;

use crate::format::font::{Font, GlyphInfo, GlyphTrait};

pub trait FontMetrics {
    fn get_ascent(&self) -> u32;

    fn get_descent(&self) -> u32;

    fn get_glyph_info(&self, codepoint: char) -> Option<GlyphInfo>;
}

impl<T: FontMetrics> FontMetrics for Arc<T> {
    fn get_ascent(&self) -> u32 {
        (**self).get_ascent()
    }

    fn get_descent(&self) -> u32 {
        (**self).get_descent()
    }

    fn get_glyph_info(&self, codepoint: char) -> Option<GlyphInfo> {
        (**self).get_glyph_info(codepoint)
    }
}

impl<G: GlyphTrait> FontMetrics for Font<G> {
    fn get_ascent(&self) -> u32 {
        Font::get_ascent(self) as u32
    }

    fn get_descent(&self) -> u32 {
        Font::get_descent(self) as u32
    }

    fn get_glyph_info(&self, codepoint: char) -> Option<GlyphInfo> {
        Font::try_get_glyph_for_character(self, codepoint.try_into().unwrap()).map(|v| v.get_info())
    }
}
