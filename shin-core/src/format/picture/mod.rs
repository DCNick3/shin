use anyhow::{bail, Result};
use binrw::prelude::*;
use binrw::{ReadOptions, WriteOptions};
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use image::{ImageBuffer, RgbaImage};
use std::borrow::Cow;
use std::io;
use std::sync::Mutex;

#[derive(BinRead, BinWrite, Debug)]
#[br(little, magic = b"PIC4")]
struct PicHeader {
    version: u32,
    file_size: u32,
    origin_x: u16,
    origin_y: u16,
    effective_width: u16,
    effective_height: u16,
    field_20: u32,
    chunk_count: u32,
    picture_id: u32,
    field_32: u32,
}

#[derive(BinRead, BinWrite, Debug)]
#[br(little)]
struct PicChunkDesc {
    x: u16,
    y: u16,
    // from the beginning of the pic file
    offset: u32,
    size: u32,
}

bitflags! {
    struct CompressionFlags: u16 {
        // ShinDataUtil has inverted bits...
        // ("Separate alpha" and "differential encoding")
        // the actual values strored in the file are inverted so I decided to invert them in this bitfield too
        const USE_INLINE_ALPHA = 0b00000001;
        const USE_DICT_ENCODING = 0b00000010;
    }
}

impl BinRead for CompressionFlags {
    type Args = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let flags = u16::read_options(reader, options, ())?;
        CompressionFlags::from_bits(flags).ok_or_else(|| {
            binrw::Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid compression flags: {}", flags),
            ))
        })
    }
}
impl BinWrite for CompressionFlags {
    type Args = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        _: (),
    ) -> BinResult<()> {
        self.bits().write_options(writer, options, ())
    }
}

#[derive(BinRead, BinWrite, Debug)]
#[br(little)]
struct PicChunkHeader {
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

impl PicChunkHeader {
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

pub trait PictureChunkBuilder {
    type Output: Send;
    fn new(
        offset_x: u32,
        offset_y: u32,
        width: u32,
        height: u32,
        opaque_vertices: Vec<PicVertexEntry>,
        transparent_vertices: Vec<PicVertexEntry>,
    ) -> Self;

    // TODO: handle vertices

    fn on_pixel(&mut self, x: u32, y: u32, value: Rgba8);
    fn build(self) -> Result<Self::Output>;
}

pub trait PictureBuilder<'d>: Send {
    type Args;
    type Output: Send;
    type ChunkBuilder: PictureChunkBuilder;

    fn new(
        args: Self::Args,
        effective_width: u32,
        effective_height: u32,
        origin_x: u32,
        origin_y: u32,
        picture_id: u32,
    ) -> Self;

    fn add_chunk(
        &mut self,
        position: (u32, u32),
        chunk: <Self::ChunkBuilder as PictureChunkBuilder>::Output,
    ) -> Result<()>;

    fn build(self) -> Result<Self::Output>;
}

#[derive(Debug, Clone)]
pub struct SimplePictureChunk {
    pub offset_x: u32,
    pub offset_y: u32,
    pub opaque_vertices: Vec<PicVertexEntry>,
    pub transparent_vertices: Vec<PicVertexEntry>,
    pub data: RgbaImage,
}

impl PictureChunkBuilder for SimplePictureChunk {
    type Output = SimplePictureChunk;

    fn new(
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

    fn on_pixel(&mut self, x: u32, y: u32, value: Rgba8) {
        self.data.put_pixel(x, y, value.into());
    }

    fn build(self) -> Result<Self::Output> {
        Ok(self)
    }
}

pub struct SimpleMergedPicture {
    pub image: RgbaImage,
    pub origin_x: u32,
    pub origin_y: u32,
    pub picture_id: u32,
}

impl<'a> PictureBuilder<'a> for SimpleMergedPicture {
    type Args = ();
    type Output = SimpleMergedPicture;
    type ChunkBuilder = SimplePictureChunk;

    fn new(
        _: (),
        effective_width: u32,
        effective_height: u32,
        origin_x: u32,
        origin_y: u32,
        picture_id: u32,
    ) -> Self {
        SimpleMergedPicture {
            image: RgbaImage::new(effective_width, effective_height),
            origin_x,
            origin_y,
            picture_id,
        }
    }

    fn add_chunk(
        &mut self,
        (x, y): (u32, u32),
        chunk: <Self::ChunkBuilder as PictureChunkBuilder>::Output,
    ) -> Result<()> {
        // I think those are used only in bustups
        // I am not sure how to handle them yet
        assert_eq!(chunk.offset_x, 0);
        assert_eq!(chunk.offset_y, 0);

        let chunk_image = chunk.data;
        image::imageops::replace(&mut self.image, &chunk_image, x as i64, y as i64);

        Ok(())
    }

    fn build(self) -> Result<Self::Output> {
        Ok(self)
    }
}

pub struct SimplePicture {
    pub chunks: Vec<((u32, u32), SimplePictureChunk)>,
    pub effective_width: u32,
    pub effective_height: u32,
    pub origin_x: u32,
    pub origin_y: u32,
    pub picture_id: u32,
}

impl<'a> PictureBuilder<'a> for SimplePicture {
    type Args = ();
    type Output = SimplePicture;
    type ChunkBuilder = SimplePictureChunk;

    fn new(
        _: (),
        effective_width: u32,
        effective_height: u32,
        origin_x: u32,
        origin_y: u32,
        picture_id: u32,
    ) -> Self {
        Self {
            chunks: vec![],
            effective_width,
            effective_height,
            origin_x,
            origin_y,
            picture_id,
        }
    }

    fn add_chunk(
        &mut self,
        position: (u32, u32),
        chunk: <Self::ChunkBuilder as PictureChunkBuilder>::Output,
    ) -> Result<()> {
        self.chunks.push((position, chunk));
        Ok(())
    }

    fn build(self) -> Result<Self::Output> {
        Ok(self)
    }
}

fn decode_dict<B: PictureChunkBuilder>(
    builder: &mut B,
    dict: &[Rgba8; 0x100],
    encoded_data: &[u8],
    alpha_data: Option<&[u8]>,
    width: usize,
    stride: usize,
) {
    if let Some(alpha_data) = alpha_data {
        assert_eq!(alpha_data.len(), encoded_data.len());

        for (y, (row, alpha_row)) in encoded_data
            .chunks(stride)
            .zip(alpha_data.chunks(stride))
            .enumerate()
        {
            for (x, (index, alpha)) in row[..width]
                .iter()
                .cloned()
                .zip(alpha_row[..width].iter().cloned())
                .enumerate()
            {
                let mut val = dict[index as usize];
                val.a = alpha;
                builder.on_pixel(x as u32, y as u32, val);
            }
        }
    } else {
        for (y, row) in encoded_data.chunks(stride).enumerate() {
            for (x, index) in row[..width].iter().cloned().enumerate() {
                let val = dict[index as usize];
                builder.on_pixel(x as u32, y as u32, val);
            }
        }
    }
}

fn read_picture_chunk<'a, L: PictureBuilder<'a>>(
    chunk_data: &'a [u8],
) -> Result<<L::ChunkBuilder as PictureChunkBuilder>::Output> {
    use io::Seek;

    let mut reader = io::Cursor::new(chunk_data);
    let header: PicChunkHeader = reader.read_le()?;

    let opaque_vertices = (0..header.opaque_vertex_count)
        .into_iter()
        .map(|_| reader.read_le())
        .collect::<BinResult<Vec<PicVertexEntry>>>()?;
    let transparent_vertices = (0..header.transparent_vertex_count)
        .into_iter()
        .map(|_| reader.read_le())
        .collect::<BinResult<Vec<PicVertexEntry>>>()?;

    // skip padding
    reader.seek(io::SeekFrom::Current(header.padding_before_data as i64 * 2))?;

    // TODO: maybe replace this bit alignment magic with easier to understand operations?
    let differential_stride = ((header.width as u32 * 4 + 0xf) & 0xfffffff0) as usize;
    let dictionary_stride = ((header.width as u32 + 3) & 0xfffffffc) as usize;

    let width = header.width as usize;
    let height = header.height as usize;

    let data = if header.compressed_size != 0 {
        // need to decompress...
        // first calculate size of required output buffer (for perf reasons)
        let out_size = if header.use_dict_encoding() {
            let mut out_size = dictionary_stride * height;
            if !header.use_inline_alpha() {
                out_size *= 2;
            }
            out_size += 0x400; // for the dictionary
            out_size
        } else {
            differential_stride * height
        };

        let mut out_buffer = Vec::with_capacity(out_size);
        let compressed =
            &chunk_data[reader.position() as usize..][..header.compressed_size as usize];
        reader.seek(io::SeekFrom::Current(header.compressed_size as i64))?;
        super::lz77::decompress::<12>(compressed, &mut out_buffer);

        assert_eq!(out_size, out_buffer.len());

        Cow::Owned(out_buffer)
    } else {
        Cow::Borrowed(&chunk_data[reader.position() as usize..])
    };

    let mut builder = L::ChunkBuilder::new(
        header.offset_x as u32,
        header.offset_y as u32,
        width as u32,
        height as u32,
        opaque_vertices,
        transparent_vertices,
    );

    if header.use_dict_encoding() {
        let stride = dictionary_stride;
        let dictionary = &data[..0x400];
        let encoded_data = &data[0x400..0x400 + stride * height];
        let alpha_data = if !header.use_inline_alpha() {
            Some(&data[0x400 + stride * height..])
        } else {
            None
        };

        let dictionary = bytemuck::pod_read_unaligned::<[Rgba8; 0x100]>(dictionary);

        if !header.use_inline_alpha() {
            debug_assert!(dictionary
                .iter()
                // if we have inline alpha we can't have any transparent pixels
                // (the second case is for empty dictionary entries, where all the components are 0)
                .all(|v| v.a == 0xff || v == &Rgba8::default()));
        }

        decode_dict(
            &mut builder,
            &dictionary,
            encoded_data,
            alpha_data,
            width,
            stride,
        )
    } else {
        todo!("decode differential")
    }

    builder.build()
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

    if header.field_32 != 0x1000 {
        bail!("Unknown field_32 value {}", header.field_32);
    }

    let mut chunks = Vec::new();
    for _ in 0..header.chunk_count {
        let chunk_desc = PicChunkDesc::read(&mut source)?;
        let chunk_data =
            &source.get_ref()[chunk_desc.offset as usize..][..chunk_desc.size as usize];
        chunks.push(((chunk_desc.x as usize, chunk_desc.y as usize), chunk_data));
    }

    use rayon::prelude::*;

    let builder = B::new(
        builder_args,
        header.effective_width as u32,
        header.effective_height as u32,
        header.origin_x as u32,
        header.origin_y as u32,
        header.picture_id,
    );
    // TODO: how should be parallelize it in bevy?
    // bevy doesn't use rayon, so using it here may be suboptimal
    // ideally we want to be generic over the parallelization strategy
    let builder = Mutex::new(builder);
    chunks
        .par_iter()
        .cloned()
        .map(|(pos, data)| (pos, read_picture_chunk::<B>(data)))
        .try_for_each(|(pos, chunk)| {
            builder
                .lock()
                .unwrap()
                .add_chunk((pos.0 as u32, pos.1 as u32), chunk?)
        })?;

    let listener = builder.into_inner().unwrap();

    listener.build()
}
