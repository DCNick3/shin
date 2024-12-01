//! Support for decoding PIC format used by the game
//!
//! The picture format splits the picture in blocks that are first separately transformed by using a dictionary or a differential encoding and an optional lz77 compression on top.
//!
//! It also stores vertices for each block specifying which regions of the image have transparency and which don't. This potentially allows for a more efficient GPU rendering (this implementation doesn't do this yet).

use std::{borrow::Cow, io, sync::Mutex};

use anyhow::{bail, Context, Result};
use binrw::{prelude::*, Endian};
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use image::{ImageBuffer, RgbaImage};
use itertools::Itertools;
use shin_tasks::ParallelSlice;

#[derive(BinRead, BinWrite, Debug)]
#[brw(little, magic = b"PIC4")]
struct PicHeader {
    version: u32,
    file_size: u32,
    origin_x: i16,
    origin_y: i16,
    effective_width: u16,
    effective_height: u16,
    /// Some sort of flags. Varying bit (1 << 0) seen in files, the game has code to handle the (1 << 1) bit somehow
    field_20: u32,
    block_count: u32,
    picture_id: u32,
    /// Scale in units of 1/4096
    scale: u32,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
struct PicBlockDesc {
    x: u16,
    y: u16,
    // from the beginning of the pic file
    offset: u32,
    size: u32,
}

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    struct CompressionFlags: u16 {
        // ShinDataUtil has inverted bits...
        // ("Separate alpha" and "differential encoding")
        // the actual values strored in the file are inverted so I decided to invert them in this bitfield too
        const USE_INLINE_ALPHA = 0b00000001;
        const USE_DICT_ENCODING = 0b00000010;
    }
}

impl BinRead for CompressionFlags {
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        let flags = u16::read_options(reader, endian, ())?;
        CompressionFlags::from_bits(flags).ok_or_else(|| {
            binrw::Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid compression flags: {}", flags),
            ))
        })
    }
}
impl BinWrite for CompressionFlags {
    type Args<'a> = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: (),
    ) -> BinResult<()> {
        self.bits().write_options(writer, endian, ())
    }
}

#[derive(BinRead, BinWrite, Debug)]
#[br(little)]
struct PicBlockHeader {
    compression_flags: CompressionFlags,
    opaque_vertex_count: u16,
    transparent_vertex_count: u16,
    // specifies amount of padding before (possibly compressed) data in 2-byte words
    padding_before_data: u16,
    offset_x: u16,
    offset_y: u16,
    width: u16,
    height: u16,
    compressed_size: u16,
    unknown_bool: u16,
}

impl PicBlockHeader {
    pub fn use_inline_alpha(&self) -> bool {
        self.compression_flags
            .contains(CompressionFlags::USE_INLINE_ALPHA)
    }
    pub fn use_dict_encoding(&self) -> bool {
        self.compression_flags
            .contains(CompressionFlags::USE_DICT_ENCODING)
    }
}

#[derive(BinRead, BinWrite, Debug, Copy, Clone)]
#[br(little)]
pub struct PicVertexEntry {
    pub from_x: u16,
    pub from_y: u16,
    pub to_x: u16,
    pub to_y: u16,
}

#[derive(Zeroable, Pod, Copy, Clone, Default, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<Rgba8> for image::Rgba<u8> {
    #[inline(always)]
    fn from(v: Rgba8) -> Self {
        Self([v.r, v.g, v.b, v.a])
    }
}

pub trait PictureBuilder<'d>: Send {
    type Args;
    type Output: Send;

    fn new(
        args: Self::Args,
        effective_width: u32,
        effective_height: u32,
        origin_x: i32,
        origin_y: i32,
        picture_id: u32,
    ) -> Self;

    fn add_block(&mut self, position: (u32, u32), block: PictureBlock) -> Result<()>;

    fn build(self) -> Result<Self::Output>;
}

#[derive(Debug, Clone)]
pub struct PictureBlock {
    pub offset_x: u32,
    pub offset_y: u32,
    pub opaque_vertices: Vec<PicVertexEntry>,
    pub transparent_vertices: Vec<PicVertexEntry>,
    pub data: RgbaImage,
}

impl PictureBlock {
    pub fn new(
        offset_x: u32,
        offset_y: u32,
        width: u32,
        height: u32,
        opaque_vertices: Vec<PicVertexEntry>,
        transparent_vertices: Vec<PicVertexEntry>,
    ) -> Self {
        Self {
            offset_x,
            offset_y,
            opaque_vertices,
            transparent_vertices,
            data: ImageBuffer::new(width, height),
        }
    }

    pub fn empty() -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            opaque_vertices: Vec::new(),
            transparent_vertices: Vec::new(),
            data: ImageBuffer::new(0, 0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.width() == 0 && self.data.height() == 0
    }
}

pub struct SimpleMergedPicture {
    pub image: RgbaImage,
    pub origin_x: i32,
    pub origin_y: i32,
    pub picture_id: u32,
}

impl<'a> PictureBuilder<'a> for SimpleMergedPicture {
    type Args = ();
    type Output = SimpleMergedPicture;

    fn new(
        _: (),
        effective_width: u32,
        effective_height: u32,
        origin_x: i32,
        origin_y: i32,
        picture_id: u32,
    ) -> Self {
        SimpleMergedPicture {
            image: RgbaImage::new(effective_width, effective_height),
            origin_x,
            origin_y,
            picture_id,
        }
    }

    fn add_block(&mut self, (x, y): (u32, u32), block: PictureBlock) -> Result<()> {
        // I think those are used only in bustups
        // I am not sure how to handle them yet
        assert_eq!(block.offset_x, 0);
        assert_eq!(block.offset_y, 0);

        let block_image = block.data;
        image::imageops::replace(&mut self.image, &block_image, x as i64, y as i64);

        Ok(())
    }

    fn build(self) -> Result<Self::Output> {
        Ok(self)
    }
}

pub struct SimplePicture {
    pub blocks: Vec<((u32, u32), PictureBlock)>,
    pub effective_width: u32,
    pub effective_height: u32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub picture_id: u32,
}

impl<'a> PictureBuilder<'a> for SimplePicture {
    type Args = ();
    type Output = SimplePicture;

    fn new(
        _: (),
        effective_width: u32,
        effective_height: u32,
        origin_x: i32,
        origin_y: i32,
        picture_id: u32,
    ) -> Self {
        Self {
            blocks: vec![],
            effective_width,
            effective_height,
            origin_x,
            origin_y,
            picture_id,
        }
    }

    fn add_block(&mut self, position: (u32, u32), block: PictureBlock) -> Result<()> {
        self.blocks.push((position, block));
        Ok(())
    }

    fn build(self) -> Result<Self::Output> {
        Ok(self)
    }
}

fn decode_dict(
    image: &mut RgbaImage,
    dict: &[Rgba8; 0x100],
    encoded_data: &[u8],
    alpha_data: Option<&[u8]>,
    width: usize,
    stride: usize,
) {
    if let Some(alpha_data) = alpha_data {
        assert_eq!(alpha_data.len(), encoded_data.len());

        for ((row, alpha_row), dest_row) in encoded_data
            .chunks(stride)
            .zip(alpha_data.chunks(stride))
            .zip_eq(image.rows_mut())
        {
            for ((index, alpha), dest_pixel) in row[..width]
                .iter()
                .cloned()
                .zip(alpha_row[..width].iter().cloned())
                .zip_eq(dest_row)
            {
                let mut val = dict[index as usize];
                val.a = alpha;
                *dest_pixel = val.into();
            }
        }
    } else {
        for (row, dest_row) in encoded_data.chunks(stride).zip_eq(image.rows_mut()) {
            for (index, dest_pixel) in row[..width].iter().cloned().zip_eq(dest_row) {
                *dest_pixel = dict[index as usize].into();
            }
        }
    }
}

pub fn read_texture(
    data: &[u8],
    compressed_size: usize,
    target_image: &mut RgbaImage,
    use_dict_encoding: bool,
    use_inline_alpha: bool,
) {
    let width = target_image.width();
    let height = target_image.height();

    // TODO: maybe replace this bit alignment magic with easier to understand operations?
    let differential_stride = ((width * 4 + 0xf) & 0xfffffff0) as usize;
    let dictionary_stride = ((width + 3) & 0xfffffffc) as usize;

    let data = if compressed_size != 0 {
        // need to decompress...
        let decompressed_size = if use_dict_encoding {
            let mut out_size = dictionary_stride * height as usize;
            if !use_inline_alpha {
                out_size *= 2;
            }
            out_size += 0x400; // for the dictionary
            out_size
        } else {
            differential_stride * height as usize
        };
        let mut out_buffer = Vec::with_capacity(decompressed_size);
        let compressed = &data[..compressed_size];
        super::lz77::decompress::<12>(compressed, &mut out_buffer);

        assert_eq!(decompressed_size, out_buffer.len());

        Cow::Owned(out_buffer)
    } else {
        Cow::Borrowed(data)
    };

    if use_dict_encoding {
        let stride = dictionary_stride;
        let dictionary = &data[..0x400];
        let encoded_data = &data[0x400..0x400 + stride * height as usize];
        let alpha_data = if !use_inline_alpha {
            Some(&data[0x400 + stride * height as usize..])
        } else {
            None
        };

        let dictionary = bytemuck::pod_read_unaligned::<[Rgba8; 0x100]>(dictionary);

        if !use_inline_alpha {
            debug_assert!(dictionary
                .iter()
                // if we have inline alpha we can't have any transparent pixels
                // (the second case is for empty dictionary entries, where all the components are 0)
                .all(|v| v.a == 0xff || v == &Rgba8::default()));
        }

        decode_dict(
            target_image,
            &dictionary,
            encoded_data,
            alpha_data,
            width as usize,
            stride,
        )
    } else {
        todo!("decode differential")
    }
}

/// Read a picture block from the data
///
/// If the block data is an empty slice, the function will return an empry image block
/// (this is used in some bustups)
pub fn read_picture_block(block_data: &[u8]) -> Result<PictureBlock> {
    use io::Seek;

    if block_data.is_empty() {
        // the game actually supports "empty" picture blocks...
        // handle them specially, since they are not really structured the same way
        return Ok(PictureBlock::empty());
    }

    let mut reader = io::Cursor::new(block_data);
    let header: PicBlockHeader = reader.read_le().context("Reading block header")?;

    let opaque_vertices = (0..header.opaque_vertex_count)
        .map(|_| reader.read_le())
        .collect::<BinResult<Vec<PicVertexEntry>>>()?;
    let transparent_vertices = (0..header.transparent_vertex_count)
        .map(|_| reader.read_le())
        .collect::<BinResult<Vec<PicVertexEntry>>>()?;

    // skip padding
    reader.seek(io::SeekFrom::Current(header.padding_before_data as i64 * 2))?;

    let width = header.width as u32;
    let height = header.height as u32;

    let mut block = PictureBlock::new(
        header.offset_x as u32,
        header.offset_y as u32,
        width,
        height,
        opaque_vertices,
        transparent_vertices,
    );

    read_texture(
        &block_data[reader.position() as usize..],
        header.compressed_size as usize,
        &mut block.data,
        header.use_dict_encoding(),
        header.use_inline_alpha(),
    );

    Ok(block)
}

pub fn read_picture<'a, B: PictureBuilder<'a>>(
    source: &'a [u8],
    builder_args: B::Args,
) -> Result<B::Output> {
    let mut source = io::Cursor::new(source);
    let header = PicHeader::read(&mut source)?;

    if header.version != 3 {
        bail!("Unsupported picture format version {}", header.version);
    }

    if header.file_size != source.get_ref().len() as u32 {
        bail!("File size mismatch");
    }

    if !matches!(header.field_20, 0 | 1) {
        bail!("Unknown field_20 value {}", header.field_20);
    }

    if header.scale != 4096 {
        bail!("Unsupported scale value {}/4096", header.scale);
    }

    let mut blocks = Vec::new();
    for _ in 0..header.block_count {
        let block_desc = PicBlockDesc::read(&mut source)?;
        let block_data =
            &source.get_ref()[block_desc.offset as usize..][..block_desc.size as usize];
        blocks.push(((block_desc.x as usize, block_desc.y as usize), block_data));
    }

    let builder = B::new(
        builder_args,
        header.effective_width as u32,
        header.effective_height as u32,
        header.origin_x as i32,
        header.origin_y as i32,
        header.picture_id,
    );

    let builder = Mutex::new(builder);
    blocks
        .par_map(shin_tasks::AsyncComputeTaskPool::get(), |&(pos, data)| {
            (pos, read_picture_block(data))
        })
        .into_iter()
        .try_for_each(|(pos, block)| {
            builder
                .lock()
                .unwrap()
                .add_block((pos.0 as u32, pos.1 as u32), block?)
        })?;

    let listener = builder.into_inner().unwrap();

    listener.build()
}
