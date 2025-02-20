use glam::vec3;
use shin_core::{
    format::mask::{MaskRect, RegionData},
    primitives::color::UnormColor,
};
use shin_render::{
    gpu_texture::GpuTexture,
    shaders::types::{
        buffer::{OwnedIndexBuffer, OwnedVertexBuffer, VertexSource},
        vertices::PosColVertex,
    },
};

use crate::asset::system::{Asset, AssetDataAccessor, AssetLoadContext};

struct MaskTextureVertexOffsets {
    pub black_offset: usize,
    pub black_count: usize,
    pub white_offset: usize,
    pub white_count: usize,
    pub transparent_offset: usize,
    pub transparent_count: usize,
}

enum MaskVertexKind {
    Black,
    White,
    Transparent,
}

impl MaskTextureVertexOffsets {
    fn get_offset_and_count(&self, kind: MaskVertexKind) -> (usize, usize) {
        match kind {
            MaskVertexKind::Black => (self.black_offset, self.black_count),
            MaskVertexKind::White => (self.white_offset, self.white_count),
            MaskVertexKind::Transparent => (self.transparent_offset, self.transparent_count),
        }
    }

    pub fn slice_vertex_and_index<'a>(
        &self,
        vertex: &'a OwnedVertexBuffer<PosColVertex>,
        index: &'a OwnedIndexBuffer,
        kind: MaskVertexKind,
    ) -> VertexSource<'a, PosColVertex> {
        let (offset, size) = self.get_offset_and_count(kind);

        let vertices =
            vertex.as_sliced_buffer_ref(offset * VERTICES_PER_RECT, size * VERTICES_PER_RECT);
        let indices =
            index.as_sliced_buffer_ref(offset * INDICES_PER_RECT, size * INDICES_PER_RECT);

        VertexSource::VertexAndIndexBuffer { vertices, indices }
    }
}

pub struct MaskTexture {
    pub label: String,

    // NB: the original engine has `total_area`, `black_area`, `white_area` and `transparent_area` fields
    // they are not very useful though, so left unimplemented
    pub offsets: MaskTextureVertexOffsets,

    pub vertex_buffer: OwnedVertexBuffer<PosColVertex>,
    pub index_buffer: OwnedIndexBuffer,
    pub texture: GpuTexture,
}

const VERTICES_PER_RECT: usize = 4;
const INDICES_PER_RECT: usize = 6;

fn load_vertices(
    context: &AssetLoadContext,
    region_data: &RegionData,
    label: &str,
) -> (
    OwnedVertexBuffer<PosColVertex>,
    OwnedIndexBuffer,
    MaskTextureVertexOffsets,
) {
    let black_count = region_data.black_regions.rect_count as usize;
    let white_count = region_data.white_regions.rect_count as usize;
    let transparent_count = region_data.transparent_regions.rect_count as usize;

    let black_offset = 0;
    let white_offset = black_offset + black_count;
    let transparent_offset = white_offset + white_count;
    let rect_count = transparent_offset + transparent_count;

    assert_eq!(region_data.rects.len(), rect_count);

    let mut vertex_buffer = Vec::with_capacity(rect_count * VERTICES_PER_RECT);
    let mut index_buffer = Vec::with_capacity(rect_count * INDICES_PER_RECT);

    for &MaskRect {
        from_x,
        from_y,
        to_x,
        to_y,
    } in &region_data.rects
    {
        let from_x = from_x as f32;
        let to_x = to_x as f32 + 0.5;
        let from_y = from_y as f32;
        let to_y = to_y as f32 + 0.5;

        let vertices: [_; VERTICES_PER_RECT] = [
            (from_x, from_y),
            (to_x, from_y),
            (from_x, to_y),
            (to_x, to_y),
        ];

        let index_base = vertex_buffer.len() as u16;
        let indices: [u16; INDICES_PER_RECT] = [0, 1, 2, 3, 2, 1].map(|i| index_base + i);

        vertex_buffer.extend(vertices.map(|(x, y)| PosColVertex {
            position: vec3(x, y, 0.0),
            color: UnormColor::WHITE,
        }));
        index_buffer.extend(indices);
    }

    let vertex_buffer = OwnedVertexBuffer::allocate_vertex(
        &context.wgpu_device,
        &vertex_buffer,
        Some(&format!("{}/vertex", label)),
    );
    let index_buffer = OwnedIndexBuffer::allocate_index(
        &context.wgpu_device,
        &index_buffer,
        Some(&format!("{}/index", label)),
    );

    let offsets = MaskTextureVertexOffsets {
        black_offset,
        black_count,
        white_offset,
        white_count,
        transparent_offset,
        transparent_count,
    };

    (vertex_buffer, index_buffer, offsets)
}

impl Asset for MaskTexture {
    type Args = ();

    async fn load(
        context: &AssetLoadContext,
        _args: Self::Args,
        name: &str,
        data: AssetDataAccessor,
    ) -> anyhow::Result<Self> {
        let label = format!("Mask[{}]", name);

        let data = data.read_all().await;

        let mask = shin_core::format::mask::read_mask(&data)?;

        let (vertex_buffer, index_buffer, offsets) = load_vertices(context, &mask.regions, &label);

        let texture = GpuTexture::new_static_from_gray_image(
            &context.wgpu_device,
            &context.wgpu_queue,
            Some(&format!("{}/texture", label)),
            &mask.texels,
        );

        let result = MaskTexture {
            label,
            offsets,
            vertex_buffer,
            index_buffer,
            texture,
        };

        Ok(result)
    }
}
