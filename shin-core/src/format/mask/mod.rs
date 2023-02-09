//! Support for MSK format, storing 8-bit mask textures

use anyhow::Result;
use binrw::{BinRead, BinWrite};
use image::{GrayImage, Luma};
use itertools::Itertools;
use std::borrow::Cow;

#[derive(BinRead, BinWrite)]
#[brw(little, magic = b"MSK4")]
struct MskHeader {
    #[br(assert(version == 1))]
    #[bw(assert(*version == 1))]
    pub version: u32,
    pub file_size: u32,
    pub mask_id: u32,
    pub width: u16,
    pub height: u16,
    pub data_offset: u32,
    pub data_size: u32,
    pub vertices_data: u32,
    pub vertices_size: u32,
}

pub struct MaskTexture {
    pub id: u32,
    // TODO: vertices
    pub texels: GrayImage,
}

fn read_texels(texels_data: &[u8], width: u32, height: u32) -> Result<GrayImage> {
    let mut source = std::io::Cursor::new(texels_data);
    let source = &mut source;

    let compressed_size = u32::read_le(source)? as usize;

    let data = &source.get_ref()[(source.position() as usize)..];

    let stride = ((width + 0xf) & 0xfffffff0) as usize;
    let decompressed_size = stride * height as usize;

    let data = if compressed_size != 0 {
        // need to decompress...
        let mut out_buffer = Vec::with_capacity(decompressed_size);
        let compressed = &data[..compressed_size];
        super::lz77::decompress::<12>(compressed, &mut out_buffer);

        assert_eq!(out_buffer.len(), decompressed_size);

        Cow::Owned(out_buffer)
    } else {
        assert_eq!(data.len(), decompressed_size);
        Cow::Borrowed(data)
    };

    let mut result = GrayImage::new(width, height);

    for (row_data, result_row) in data.chunks_exact(stride).zip_eq(result.rows_mut()) {
        for (src, dst) in row_data
            .iter()
            .copied()
            .take(width as usize)
            .zip_eq(result_row)
        {
            *dst = Luma([src]);
        }
    }

    Ok(result)
}

pub fn read_mask(source: &[u8]) -> Result<MaskTexture> {
    let mut source = std::io::Cursor::new(source);
    let source = &mut source;

    let header = MskHeader::read(source)?;

    let data = &source.get_ref()[header.data_offset as usize..][..header.data_size as usize];
    let _vertices =
        &source.get_ref()[header.vertices_data as usize..][..header.vertices_size as usize];
    // vertices are not parsed here, as our engine does not use them

    let texels = read_texels(data, header.width as u32, header.height as u32)?;

    Ok(MaskTexture {
        id: header.mask_id,
        texels,
    })
}
