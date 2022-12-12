use crate::format::lz77;
use anyhow::Result;
use binrw::{BinRead, BinWrite};
use byteorder::{LittleEndian, ReadBytesExt};
use image::GrayImage;
use rayon::prelude::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io;
use std::io::Read;

#[derive(BinRead, BinWrite, Debug)]
#[brw(little, magic = b"FNT4")]
struct FontHeader {
    pub version: u32,
    pub size: u32,
    pub max_size: u16,
    pub min_size: u16,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
struct GlyphHeader {
    // terms are roughly based on https://freetype.org/freetype2/docs/glyphs/glyphs-3.html
    /// Distance between the current position of the pen and left of the glyph bitmap
    pub bearing_x: i8,
    /// Distance between the baseline and the top of the glyph bitmap
    pub bearing_y: i8,
    /// Width, without padding (glyph bitmap are padded to be a power of 2)
    pub actual_width: u8,
    /// Height, without padding (glyph bitmap are padded to be a power of 2)
    pub actual_height: u8,
    /// Amount of horizontal pen movements after drawing the glyph
    pub advance_width: u8,
    // might have been advance_height, but it's always 0
    // it's not like the engine can render text vertically, right?
    pub unused: u8,
    /// Width of the texture (should be a power of 2)
    pub texture_width: u8,
    /// Height of the texture (should be a power of 2)
    pub texture_height: u8,
    pub compressed_size: u16,
}

#[derive(Debug, Copy, Clone)]
pub struct GlyphInfo {
    /// Distance between the current position of the pen and left of the glyph bitmap
    pub bearing_x: i8,
    /// Distance between the baseline and the top of the glyph  bitmap
    pub bearing_y: i8,
    /// Amount of horizontal pen movements after drawing the glyph
    pub advance_width: u8,
    /// Width of the glyph bitmap (w/o padding)
    pub actual_width: u8,
    /// Height of the glyph bitmap (w/o padding)
    pub actual_height: u8,
}

impl GlyphInfo {
    pub fn size(&self) -> (u32, u32) {
        (self.actual_width as u32, self.actual_height as u32)
    }
}

pub enum GlyphMipLevel {
    Level0,
    Level1,
    Level2,
    Level3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlyphId(pub u32);

pub struct Glyph {
    info: GlyphInfo,
    mip_level_0: GrayImage,
    mip_level_1: GrayImage,
    mip_level_2: GrayImage,
    mip_level_3: GrayImage,
}

impl Glyph {
    pub fn get_image(&self, mip_level: GlyphMipLevel) -> &GrayImage {
        match mip_level {
            GlyphMipLevel::Level0 => &self.mip_level_0,
            GlyphMipLevel::Level1 => &self.mip_level_1,
            GlyphMipLevel::Level2 => &self.mip_level_2,
            GlyphMipLevel::Level3 => &self.mip_level_3,
        }
    }

    pub fn get_info(&self) -> GlyphInfo {
        self.info
    }
}

fn read_glyph(source: &mut io::Cursor<&[u8]>) -> Result<Glyph> {
    let header = GlyphHeader::read(source)?;

    assert_eq!(header.unused, 0);

    let uncompressed_size = header.texture_width as usize * header.texture_height as usize;
    // factor in the mip levels
    let uncompressed_size =
        uncompressed_size + uncompressed_size / 4 + uncompressed_size / 16 + uncompressed_size / 64;

    let data = if header.compressed_size == 0 {
        let mut data = vec![0; uncompressed_size];
        source.read_exact(&mut data)?;
        data
    } else {
        let mut compressed_data = vec![0; header.compressed_size as usize];
        source.read_exact(&mut compressed_data)?;
        let mut data = Vec::with_capacity(uncompressed_size);
        lz77::decompress::<10>(&compressed_data, &mut data);
        data
    };

    let mip_size_0 = header.texture_width as usize * header.texture_height as usize;
    let mip_size_1 = mip_size_0 / 4;
    let mip_size_2 = mip_size_1 / 4;
    let mip_size_3 = mip_size_2 / 4;

    let mip_level_0 = GrayImage::from_raw(
        header.texture_width as u32,
        header.texture_height as u32,
        data[0..mip_size_0].to_vec(),
    )
    .unwrap();
    let mip_level_1 = GrayImage::from_raw(
        header.texture_width as u32 / 2,
        header.texture_height as u32 / 2,
        data[mip_size_0..mip_size_0 + mip_size_1].to_vec(),
    )
    .unwrap();
    let mip_level_2 = GrayImage::from_raw(
        header.texture_width as u32 / 4,
        header.texture_height as u32 / 4,
        data[mip_size_0 + mip_size_1..mip_size_0 + mip_size_1 + mip_size_2].to_vec(),
    )
    .unwrap();
    let mip_level_3 = GrayImage::from_raw(
        header.texture_width as u32 / 8,
        header.texture_height as u32 / 8,
        data[mip_size_0 + mip_size_1 + mip_size_2
            ..mip_size_0 + mip_size_1 + mip_size_2 + mip_size_3]
            .to_vec(),
    )
    .unwrap();

    let info = GlyphInfo {
        bearing_x: header.bearing_x,
        bearing_y: header.bearing_y,
        advance_width: header.advance_width,
        actual_width: header.actual_width,
        actual_height: header.actual_height,
    };

    Ok(Glyph {
        info,
        mip_level_0,
        mip_level_1,
        mip_level_2,
        mip_level_3,
    })
}

pub struct Font {
    min_size: u16,
    max_size: u16,
    graphemes: [GlyphId; 0x10000],
    glyphs: HashMap<GlyphId, Glyph>,
}

impl Font {
    pub fn get_size_range(&self) -> (u16, u16) {
        (self.min_size, self.max_size)
    }

    pub fn get_for_grapheme(&self, grapheme: u16) -> &Glyph {
        self.glyphs.get(&self.graphemes[grapheme as usize]).unwrap()
    }

    pub fn get_grapheme_mapping(&self) -> &[GlyphId; 0x10000] {
        &self.graphemes
    }

    pub fn get_glyphs(&self) -> &HashMap<GlyphId, Glyph> {
        &self.glyphs
    }
}

// TODO: add a font struct variant that loads the graphemes lazily
// otherwise the memory usage is... not ideal
pub fn read_font<S: AsRef<[u8]>>(source: S) -> Result<Font> {
    let source_data = source.as_ref();
    let size = source_data.len();
    let mut source = io::Cursor::new(source_data);
    let header = FontHeader::read(&mut source)?;

    if header.version != 0x00000001 {
        anyhow::bail!("Unsupported font version: {}", header.version);
    }
    if header.size != size as u32 {
        anyhow::bail!("Unsupported font size: {}", header.size);
    }

    let mut grapheme_table = [0u32; 0x10000];
    source.read_u32_into::<LittleEndian>(&mut grapheme_table)?;

    let mut known_glyph_offsets = HashMap::new();
    let mut graphemes = [GlyphId(0); 0x10000];
    let mut glyphs = HashMap::new();

    for (grapheme_index, glyph_offset) in grapheme_table.into_iter().enumerate() {
        let next_glyph_id = GlyphId(known_glyph_offsets.len() as u32);
        let glyph_id = *known_glyph_offsets
            .entry(glyph_offset)
            .or_insert(next_glyph_id);
        graphemes[grapheme_index] = glyph_id;

        match glyphs.entry(glyph_id) {
            Entry::Occupied(_) => continue,
            Entry::Vacant(entry) => {
                entry.insert(glyph_offset);
            }
        }
    }

    // decode glyphs found in the glyph table in parallel
    let glyphs = glyphs
        .par_iter()
        .map(|(&glyph_id, &glyph_offset)| {
            let glyph = read_glyph(&mut io::Cursor::new(&source_data[glyph_offset as usize..]))?;
            Ok((glyph_id, glyph))
        })
        .collect::<Result<HashMap<GlyphId, Glyph>>>()?;

    Ok(Font {
        min_size: header.min_size,
        max_size: header.max_size,
        graphemes,
        glyphs,
    })
}
