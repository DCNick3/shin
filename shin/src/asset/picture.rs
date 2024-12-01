use std::collections::BTreeMap;

use anyhow::Result;
use bevy_utils::HashMap;
use glam::{vec2, Vec4};
use shin_core::format::picture::{PicBlock, PicBlockRect, PictureBuilder, SimpleMergedPicture};
use shin_render::{
    gpu_texture::GpuTexture,
    shaders::types::{
        buffer::{Buffer, OwnedIndexBuffer, OwnedVertexBuffer},
        vertices::LayerVertex,
    },
};

use crate::asset::system::{Asset, AssetDataAccessor, AssetLoadContext};

pub struct GpuPictureBlock {
    vertex_buffer: OwnedVertexBuffer<LayerVertex>,
    index_buffer: OwnedIndexBuffer,
    opaque_rect_count: u32,
    transparent_rect_count: u32,
    texture: GpuTexture,
}

impl GpuPictureBlock {
    const VERTICES_PER_RECT: usize = 4;
    const INDICES_PER_RECT: usize = 6;

    pub fn new(
        context: GpuTextureBuilderContext,
        position: (u32, u32),
        block: PicBlock,
        label: &str,
    ) -> Self {
        let rect_count = block.opaque_rects.len() + block.transparent_rects.len();

        let mut vertex_buffer = Vec::with_capacity(rect_count * Self::VERTICES_PER_RECT);
        let mut index_buffer = Vec::with_capacity(rect_count * Self::INDICES_PER_RECT);

        let offset_x = block.offset_x as f32;
        let offset_y = block.offset_y as f32;

        let width = block.data.width() as f32;
        let height = block.data.height() as f32;

        let mut emit_rect = |PicBlockRect {
                                 from_x,
                                 from_y,
                                 to_x,
                                 to_y,
                             }: PicBlockRect| {
            let from_x = from_x as f32;
            let to_x = to_x as f32;
            let from_y = from_y as f32;
            let to_y = to_y as f32;

            let x_left = (offset_x + from_x, (from_x + 1.0) / width);
            let x_right = (offset_x + to_x + 0.5, (to_x + 0.5 + 1.0) / width);
            let y_top = (offset_y + from_y, (from_y + 1.0) / height);
            let y_bottom = (offset_y + to_y + 0.5, (to_y + 0.5 + 1.0) / height);

            let vertices: [_; Self::VERTICES_PER_RECT] = [
                (x_left, y_top),
                (x_right, y_top),
                (x_left, y_bottom),
                (x_right, y_bottom),
            ];

            let index_base = vertex_buffer.len() as u16;
            let indices: [u16; Self::INDICES_PER_RECT] = [0, 1, 2, 3, 2, 1].map(|i| index_base + i);

            vertex_buffer.extend(vertices.map(|((px, tx), (py, ty))| LayerVertex {
                position: Vec4::new(px, py, tx, ty),
            }));
            index_buffer.extend(indices);
        };

        block.opaque_rects.iter().cloned().for_each(&mut emit_rect);
        block
            .transparent_rects
            .iter()
            .cloned()
            .for_each(&mut emit_rect);

        let vertex_buffer = Buffer::allocate_vertex(
            context.wgpu_device,
            &vertex_buffer,
            Some(&format!("{}/vertex", label)),
        );
        let index_buffer = Buffer::allocate_index(
            context.wgpu_device,
            &index_buffer,
            Some(&format!("{}/index", label)),
        );

        let texture = GpuTexture::new_static_from_image(
            context.wgpu_device,
            context.wgpu_queue,
            Some(label),
            &block.data,
        );

        GpuPictureBlock {
            vertex_buffer,
            index_buffer,
            opaque_rect_count: block.opaque_rects.len() as u32,
            transparent_rect_count: block.transparent_rects.len() as u32,
            texture,
        }
    }
}

#[derive(Copy, Clone)]
pub struct GpuTextureBuilderContext<'a> {
    pub wgpu_device: &'a wgpu::Device,
    pub wgpu_queue: &'a wgpu::Queue,
}

struct GpuPictureBuilder<'a> {
    context: GpuTextureBuilderContext<'a>,
    label: String,
    effective_width: u32,
    effective_height: u32,
    origin_x: i32,
    origin_y: i32,
    blocks: BTreeMap<u32, GpuPictureBlock>,
}

impl<'a> PictureBuilder for GpuPictureBuilder<'a> {
    type Args = (GpuTextureBuilderContext<'a>, String);
    type Output = Picture;

    fn new(
        (context, label): Self::Args,
        effective_width: u32,
        effective_height: u32,
        origin_x: i32,
        origin_y: i32,
        _picture_id: u32,
    ) -> Self {
        GpuPictureBuilder {
            context,
            label,
            effective_width,
            effective_height,
            origin_x,
            origin_y,
            blocks: BTreeMap::new(),
        }
    }

    fn add_block(&mut self, data_offset: u32, position: (u32, u32), block: PicBlock) -> Result<()> {
        let block = GpuPictureBlock::new(
            self.context,
            position,
            block,
            &format!("{}/{}", self.label, data_offset),
        );

        self.blocks.insert(data_offset, block);

        Ok(())
    }

    fn build(self) -> Result<Self::Output> {
        let Self {
            context: _,
            label,
            effective_width,
            effective_height,
            origin_x,
            origin_y,
            blocks,
        } = self;
        Ok(Picture {
            label,
            effective_width,
            effective_height,
            origin_x,
            origin_y,
            blocks,
        })
    }
}

/// A Picture, uploaded to GPU on demand (because doing it in the asset loading context is awkward)
pub struct Picture {
    label: String,
    effective_width: u32,
    effective_height: u32,
    origin_x: i32,
    origin_y: i32,
    blocks: BTreeMap<u32, GpuPictureBlock>,
}

impl Picture {
    // pub fn gpu_image(&self, resources: &GpuCommonResources) -> &GpuImage {
    //     self.picture.gpu_image(resources)
    // }
}

impl Asset for Picture {
    async fn load(context: &AssetLoadContext, data: AssetDataAccessor) -> Result<Self> {
        let data = data.read_all().await;

        // extract the picture id before the call to read_picture
        let info = shin_core::format::picture::read_picture_header(&data)?;

        // TODO: lookup if there's already a picture with this ID in the cache
        // Not sure if it makes sense to do so though: we are already caching pictures by their name
        // I don't think it's possible to have two pictures with the same ID but different names (why would they do it?)

        // Move the read_picture to io task pool, since most of the time it's going to be waiting on spawned tasks to complete
        shin_core::format::picture::read_picture::<GpuPictureBuilder>(
            &data,
            (
                GpuTextureBuilderContext {
                    wgpu_device: &context.wgpu_device,
                    wgpu_queue: &context.wgpu_queue,
                },
                // TODO: maybe use asset path as a label?
                format!("Pic{:08x}", info.picture_id),
            ),
        )
    }
}
