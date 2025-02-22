use std::{io::Cursor, sync::Arc};

use anyhow::Context;
use indexmap::{IndexMap, IndexSet};
use itertools::{Either, Itertools};
use rayon::prelude::*;
use shin_core::{
    format::font::{
        FontLazy, Glyph, GlyphId, GlyphInfo, GlyphMipLevel, GlyphTrait, read_lazy_font,
    },
    layout::font::FontMetrics,
};
use shin_render::{gpu_texture::GpuTexture, shaders::types::texture::TextureSource};

use crate::asset::system::{
    Asset, AssetDataAccessor, AssetLoadContext,
    cache::{AssetCache, CacheHandle},
};

pub struct GpuFontLazy {
    font: FontLazy,
    glyph_cache: AssetCache<GlyphId, GpuFontGlyph>,
}

impl GpuFontLazy {
    pub fn new(font: FontLazy) -> Self {
        Self {
            font,
            glyph_cache: AssetCache::new(),
        }
    }

    /// Returns handles to an array of glyphs in bulk.
    ///
    /// The actual loading will happen in the compute task pool, with [`GpuFontGlyphHandle`] giving access to the results asynchronously.
    pub fn load_glyphs(
        self: Arc<Self>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        characters: &[char],
    ) -> Vec<GpuFontGlyphHandle> {
        if characters.is_empty() {
            return Vec::new();
        }

        let glyph_map = self.font.get_character_mapping();

        let mut glyphs = Vec::with_capacity(characters.len());
        let mut glyphs_dedup = IndexSet::new();
        for &char in characters {
            let glyph = glyph_map[char as usize];
            glyphs.push(glyph);
            glyphs_dedup.insert(glyph);
        }

        let (need_loading_list, mut loading_list): (Vec<_>, Vec<_>) = glyphs_dedup
            .iter()
            .map(|&id| (id, self.glyph_cache.lookup(id)))
            .partition_map(|(id, result)| {
                Either::from(result.try_into_cache_handle()).map_either(
                    |l| (id, l),
                    |handle| {
                        let info = self.font.get_glyph(id).unwrap().get_info();

                        GpuFontGlyphHandle { id, info, handle }
                    },
                )
            });

        if !need_loading_list.is_empty() {
            for &(id, ref need_loading) in &need_loading_list {
                let info = self.font.get_glyph(id).unwrap().get_info();

                let handle = need_loading.get_handle();
                loading_list.push(GpuFontGlyphHandle { id, info, handle });
            }

            // actually load the stuff
            // let this = self.clone();
            shin_tasks::compute::spawn_and_forget(move || {
                // spawn a rayon task because `for_each` will block otherwise
                need_loading_list
                    .into_par_iter()
                    .for_each(|(id, need_loading)| {
                        let glyph = self.font.get_glyph(id).unwrap();
                        let glyph = glyph.decompress();
                        let glyph = Arc::new(GpuFontGlyph::load(&device, &queue, id, glyph));

                        self.glyph_cache.finish_load(need_loading, glyph);
                    });
            });
        }

        let mut glyph_handles = IndexMap::new();
        for glyph_handle in loading_list {
            glyph_handles.insert(glyph_handle.id, glyph_handle);
        }

        let mut result = Vec::new();

        for glyph_id in glyphs {
            let handle = glyph_handles.get(&glyph_id).unwrap();
            result.push(handle.clone());
        }

        result
    }
}

impl FontMetrics for GpuFontLazy {
    fn get_ascent(&self) -> u32 {
        self.font.get_ascent() as u32
    }

    fn get_descent(&self) -> u32 {
        self.font.get_descent() as u32
    }

    fn get_glyph_info(&self, codepoint: char) -> Option<GlyphInfo> {
        self.font.get_glyph_info(codepoint)
    }
}

#[derive(Debug)]
pub struct GpuFontGlyph {
    texture: GpuTexture,
}

impl GpuFontGlyph {
    pub fn load(device: &wgpu::Device, queue: &wgpu::Queue, id: GlyphId, glyph: Glyph) -> Self {
        let texture = GpuTexture::new_static_from_gray_mipped_image(
            device,
            queue,
            Some(&format!("Glyph({:?})", id.0)),
            &[
                glyph.get_image(GlyphMipLevel::Level0),
                glyph.get_image(GlyphMipLevel::Level1),
                glyph.get_image(GlyphMipLevel::Level2),
                glyph.get_image(GlyphMipLevel::Level3),
            ],
        );

        Self { texture }
    }
}

#[derive(Debug, Clone)]
pub struct GpuFontGlyphHandle {
    id: GlyphId,
    info: GlyphInfo,
    handle: CacheHandle<GpuFontGlyph>,
}

impl GpuFontGlyphHandle {
    pub fn info(&self) -> &GlyphInfo {
        &self.info
    }

    pub fn as_texture_source(&self) -> TextureSource {
        self.handle.wait_ref().texture.as_source()
    }
}

impl Asset for GpuFontLazy {
    type Args = ();

    async fn load(
        _context: &Arc<AssetLoadContext>,
        _args: (),
        _name: &str,
        data: AssetDataAccessor,
    ) -> anyhow::Result<Self> {
        let lazy_font =
            read_lazy_font(&mut Cursor::new(data.read_all().await)).context("Reading font")?;

        Ok(Self::new(lazy_font))
    }
}
