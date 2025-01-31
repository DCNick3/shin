use std::{io::Cursor, sync::Arc};

use anyhow::Context;
use indexmap::{IndexMap, IndexSet};
use itertools::{Either, Itertools};
use shin_core::{
    format::font::{
        read_lazy_font, FontLazy, Glyph, GlyphId, GlyphInfo, GlyphMipLevel, GlyphTrait,
    },
    layout::font::FontMetrics,
};
use shin_render::{gpu_texture::GpuTexture, shaders::types::texture::TextureSource};
use shin_tasks::AsyncComputeTaskPool;

use crate::asset::system::{
    cache::{AssetCache, CacheHandle},
    Asset, AssetDataAccessor, AssetLoadContext,
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

    pub fn load_glyphs(
        self: Arc<Self>,
        device: &Arc<wgpu::Device>,
        queue: &Arc<wgpu::Queue>,
        characters: &[char],
    ) -> Vec<GpuFontGlyphHandle> {
        let glyph_map = self.font.get_character_mapping();

        let mut glyphs_dedup = IndexSet::new();
        for &char in characters {
            glyphs_dedup.insert(glyph_map[char as usize]);
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
            let task_pool = AsyncComputeTaskPool::get();
            let chunk_size = std::cmp::min(16, need_loading_list.len() / task_pool.thread_num());

            for chunk in &need_loading_list.into_iter().chunks(chunk_size) {
                let chunk = chunk.collect::<Vec<_>>();
                let device = device.clone();
                let queue = queue.clone();
                let this = self.clone();

                // load the glyphs in background
                // maybe by the time they're needed they'll be ready
                AsyncComputeTaskPool::get()
                    .spawn(async move {
                        for (id, need_loading) in chunk {
                            let glyph = this.font.get_glyph(id).unwrap();
                            let glyph = glyph.decompress();
                            let glyph = Arc::new(GpuFontGlyph::load(&device, &queue, id, glyph));

                            this.glyph_cache.finish_load(need_loading, glyph);
                        }
                    })
                    .detach();
            }
        }

        let mut glyph_handles = IndexMap::new();
        for glyph_handle in loading_list {
            glyph_handles.insert(glyph_handle.id, glyph_handle);
        }

        let mut result = Vec::new();

        for char in characters {
            let id = glyph_map[*char as usize];
            let handle = glyph_handles.get(&id).unwrap();
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
        _context: &AssetLoadContext,
        _args: (),
        data: AssetDataAccessor,
    ) -> anyhow::Result<Self> {
        let lazy_font =
            read_lazy_font(&mut Cursor::new(data.read_all().await)).context("Reading font")?;

        Ok(Self::new(lazy_font))
    }
}
