use crate::render::dynamic_atlas::{AtlasImage, DynamicAtlas, ImageProvider};
use crate::render::{GpuCommonResources, TextureBindGroup};
use shin_core::format::font::{GlyphId, GlyphMipLevel, GlyphTrait, LazyFont};
use std::sync::Arc;
use strum::IntoEnumIterator;
use wgpu::TextureFormat;

struct FontImageProvider {
    font: Arc<LazyFont>,
}

impl ImageProvider for FontImageProvider {
    const IMAGE_FORMAT: TextureFormat = TextureFormat::R8Unorm;
    const MIPMAP_LEVELS: u32 = 4;
    type Id = GlyphId;

    fn get_image(&self, id: Self::Id) -> (Vec<Vec<u8>>, (u32, u32)) {
        let glyph = self.font.get_glyph(id).unwrap();
        let size = glyph.get_info().texture_size();
        let glyph = glyph.decompress();

        let mut result = Vec::new();
        for mip_level in GlyphMipLevel::iter() {
            let image = glyph.get_image(mip_level);
            result.push(image.to_vec());
        }

        (result, size)
    }
}

const TEXTURE_SIZE: (u32, u32) = (2048, 2048);

// TODO: later this should migrate away from the MessageLayer and ideally should be shared with all the game
pub struct FontAtlas {
    atlas: DynamicAtlas<FontImageProvider>,
}

impl FontAtlas {
    pub fn new(resources: &GpuCommonResources, font: Arc<LazyFont>) -> Self {
        let provider = FontImageProvider { font };
        let atlas = DynamicAtlas::new(resources, provider, TEXTURE_SIZE, Some("FontAtlas"));

        Self { atlas }
    }

    pub fn get_font(&self) -> &LazyFont {
        &self.atlas.provider().font
    }

    pub fn texture_bind_group(&self) -> &TextureBindGroup {
        self.atlas.texture_bind_group()
    }

    pub fn texture_size(&self) -> (u32, u32) {
        self.atlas.texture_size()
    }

    // TODO: implement internal locking
    pub fn get_image(&mut self, resources: &GpuCommonResources, charcode: u16) -> AtlasImage {
        let glyph_id = self.get_font().get_character_mapping()[charcode as usize];
        self.atlas
            .get_image(resources, glyph_id)
            .expect("Could not fit image in atlas")
    }

    pub fn free_image(&mut self, charcode: u16) {
        let glyph_id = self.get_font().get_character_mapping()[charcode as usize];
        self.atlas.free_image(glyph_id);
    }
}
