//! Support for BUP files, storing the character bustup sprites.
//!
//! The BUP format is re-using the machinery from the picture format, but it has some additions on top.
//!
//! The character sprite is composed of up to five layers:
//! - the base image, which is the character's body
//! - the expression common face, which displays the character's facial expression, potentially shared with other expressions
//! - the expression face, which displays the character's facial expression for the emotion
//! - the mouth, which displays the character's mouth, animated for lipsync
//! - the eyes, which display the character's eyes for the blinking animation
//!
//! The layers are separate because one base image can have multiple facial expressions layered on top, using storage more efficiently.
//!
//! All of the layers besides the base image are optional, used as the encoder sees to be more efficient.

mod builder;
pub mod default_builder;

use anyhow::{Result, bail};
use binrw::{BinRead, BinWrite};
use indexmap::IndexMap;
use rayon::prelude::*;

pub use self::builder::{
    BustupBlockPromise, BustupBlockPromiseToken, BustupBuilder, BustupExpressionSkeleton,
    BustupSkeleton,
};
use crate::format::{
    bustup::builder::BustupBlockPromisesOwner, picture::read_picture_block, text::ZeroString,
};

#[derive(Copy, Clone, Hash, PartialEq, Eq, BinRead, BinWrite)]
pub struct BustupBlockId(u32);

impl BustupBlockId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl std::fmt::Debug for BustupBlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BustupBlockId({:08x})", self.0)
    }
}

impl std::fmt::Display for BustupBlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BupBlk{:08x}", self.0)
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, BinRead, BinWrite)]
pub struct BustupId(u32);

impl BustupId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl std::fmt::Debug for BustupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BustupId({:08x})", self.0)
    }
}

impl std::fmt::Display for BustupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bup{:08x}", self.0)
    }
}

#[derive(BinRead, BinWrite, Debug)]
#[br(little, magic = b"BUP4")]
#[br(assert(version == 4))]
#[bw(assert(*version == 4))]
struct BustupHeader {
    version: u32,
    file_size: u32,
    // offset 12
    origin_x: i16,
    origin_y: i16,
    effective_width: u16,
    effective_height: u16,
    /// always seems to be 0x1, meaning unknown
    f_14: u32,

    bustup_id: BustupId,
    /// always seems to be 0x0, meaning unknown, probably related to the base block descriptors
    f_1c: u32,
    base_block_descriptors_offset: u32,
    base_block_descriptors_size: u32,

    /// always seems to be 0x0, meaning unknown, probably related to the expression descriptors
    f_28: u32,
    expression_descriptors_offset: u32,
    expression_descriptors_size: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct BustupHeaderInfo {
    pub origin_x: i16,
    pub origin_y: i16,
    pub effective_width: u16,
    pub effective_height: u16,
    pub bustup_id: BustupId,
}

#[derive(BinRead, BinWrite, Debug)]
#[br(little)]
struct BaseBlockDescriptors {
    count: u32,
    #[br(count = count)]
    descriptors: Vec<BustupBlockDesc>,
}

#[derive(BinRead, BinWrite, Debug)]
#[br(little)]
struct ExpressionDescriptors {
    count: u32,
    #[br(count = count)]
    descriptors: Vec<BustupExpressionDesc>,
}

impl ExpressionDescriptors {
    pub fn iter_additional_block_descs(&self) -> impl Iterator<Item = &BustupBlockDesc> {
        self.descriptors.iter().flat_map(|c| {
            std::iter::once(&c.face1)
                .chain(std::iter::once(&c.face2))
                .chain(c.mouth_blocks.iter())
                .chain(c.eye_blocks.iter())
        })
    }
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq)]
struct BustupBlockDesc {
    offset: u32,
    size: u32,
    block_id: BustupBlockId,
}

impl BustupBlockDesc {
    pub fn is_null(&self) -> bool {
        self.offset == 0
    }
}

#[derive(BinRead, BinWrite, Debug)]
struct BustupExpressionDesc {
    header_length: u32,
    // as far as I can tell from looking at files of sakahari, face1 can be sometimes shared between multiple emotions, hence the separation
    // I really doubt that it's actually saving anything, and it's not used in umineko, but it doesn't hurt that much to implement it
    face1: BustupBlockDesc,
    face2: BustupBlockDesc,
    mouth_block_count: u16,
    eye_block_count: u16,

    expression_name: ZeroString,

    #[brw(align_before = 0x4)]
    #[br(count = mouth_block_count)]
    mouth_blocks: Vec<BustupBlockDesc>,

    // eye textures with various stages of blinking
    // used to make bustup sprites look more lively by making them blink at random intervals
    // not used by umineko
    #[brw(align_before = 0x4)]
    #[br(count = eye_block_count)]
    eye_blocks: Vec<BustupBlockDesc>,
}

pub fn dump_header<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
    prefix: &str,
) -> Result<String> {
    let header = BustupHeader::read(reader)?;

    let s = format!(
        "{}{:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}",
        prefix,
        header.origin_x,
        header.origin_y,
        header.effective_width,
        header.effective_height,
        header.f_14,
        header.bustup_id,
        header.f_1c,
        header.base_block_descriptors_offset,
        header.base_block_descriptors_size,
        header.f_28,
        header.expression_descriptors_offset,
        header.expression_descriptors_size,
    );

    Ok(s)
}

pub fn dump_expression_descriptors<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
    prefix: &str,
) -> Result<String> {
    let header = BustupHeader::read(reader)?;
    reader.seek(std::io::SeekFrom::Start(
        header.expression_descriptors_offset as u64,
    ))?;

    let descriptors = ExpressionDescriptors::read(reader)?;

    let mut result = String::new();

    for (i, descriptor) in (0..).zip(descriptors.descriptors) {
        result.push_str(&format!(
            "{}{}, {}, {}, {:#x}, {:#x}, {:?}\n",
            prefix,
            i,
            descriptor.face1.block_id,
            descriptor.face2.block_id,
            descriptor.mouth_block_count,
            descriptor.mouth_block_count,
            descriptor.expression_name.0
        ));
    }

    Ok(result)
}

pub fn read_bustup_header(source: &[u8]) -> Result<BustupHeaderInfo> {
    let mut source = std::io::Cursor::new(source);
    let header: BustupHeader = BinRead::read(&mut source)?;

    if header.file_size != source.get_ref().len() as u32 {
        bail!("File size mismatch");
    }

    Ok(BustupHeaderInfo {
        origin_x: header.origin_x,
        origin_y: header.origin_y,
        effective_width: header.effective_width,
        effective_height: header.effective_height,
        bustup_id: header.bustup_id,
    })
}

// TODO: maybe create a reader wrapper that will actually limit the read size instead of just checking in the end?
fn with_offset_and_size<
    R: std::io::Read + std::io::Seek,
    F: FnOnce(&mut R) -> Result<T, E>,
    T,
    E: Into<anyhow::Error> + Send,
>(
    reader: &mut R,
    offset: u32,
    length: u32,
    f: F,
) -> Result<T> {
    let before = reader.stream_position()?;
    reader.seek(std::io::SeekFrom::Start(offset as u64))?;

    let result = f(reader).map_err(|e| e.into())?;

    let after = reader.stream_position()?;
    if after - offset as u64 != length as u64 {
        bail!(
            "Expected to read {} bytes, but read {} bytes",
            length,
            after - before
        );
    }

    reader.seek(std::io::SeekFrom::Start(before))?;

    Ok(result)
}

/// Reads and decodes a bustup.
///
/// NOTE: this will spawn rayon tasks and block waiting for them. If you don't want blocking wrap it with [`shin_tasks::compute::spawn`].
pub fn read_bustup<B: BustupBuilder>(source: &[u8], builder_args: B::Args) -> Result<B::Output> {
    let mut source = std::io::Cursor::new(source);
    let source = &mut source;

    let header = BustupHeader::read(source)?;

    if header.file_size != source.get_ref().len() as u32 {
        bail!("File size mismatch");
    }

    let base_block_descriptors = with_offset_and_size(
        source,
        header.base_block_descriptors_offset,
        header.base_block_descriptors_size,
        BaseBlockDescriptors::read,
    )?;

    let expression_descriptors = with_offset_and_size(
        source,
        header.expression_descriptors_offset,
        header.expression_descriptors_size,
        ExpressionDescriptors::read,
    )?;

    // collect all the non-null blocks
    let mut block_descriptors = Vec::new();
    for &desc in base_block_descriptors.descriptors.iter() {
        if !desc.is_null() {
            block_descriptors.push(desc);
        }
    }
    for &desc in expression_descriptors.iter_additional_block_descs() {
        if !desc.is_null() {
            block_descriptors.push(desc);
        }
    }

    let owner = BustupBlockPromisesOwner::new(block_descriptors.len());
    let mut issuer = owner.bind(block_descriptors.iter().map(|d| d.block_id));

    // build the skeleton
    let skeleton = {
        let base_blocks = base_block_descriptors
            .descriptors
            .iter()
            .map(|d| issuer.visit(d))
            .collect();

        let mut expressions = IndexMap::new();
        for expression in expression_descriptors.descriptors.iter() {
            let face1 = issuer.visit_opt(&expression.face1);
            let face2 = issuer.visit_opt(&expression.face2);
            let mouth_blocks = expression
                .mouth_blocks
                .iter()
                .map(|d| issuer.visit(d))
                .collect();
            let eye_blocks = expression
                .eye_blocks
                .iter()
                .map(|d| issuer.visit(d))
                .collect();

            expressions.insert(
                expression.expression_name.0.clone(),
                BustupExpressionSkeleton {
                    face1,
                    face2,
                    mouth_blocks,
                    eye_blocks,
                },
            );
        }

        BustupSkeleton {
            origin_x: header.origin_x,
            origin_y: header.origin_y,
            effective_width: header.effective_width,
            effective_height: header.effective_height,
            bustup_id: header.bustup_id,
            base_blocks,
            expressions,
        }
    };

    // let the builder decide which blocks to decode. if they are not interested in a certain block, they should drop the promise
    let skeleton = B::new(&builder_args, skeleton);

    let required_blocks = (0..)
        .zip(&block_descriptors)
        .zip(&owner.counters)
        .filter(|&(_, c)| *c.borrow() > 0)
        .map(|((index, &desc), _)| (index, desc))
        .collect::<Vec<_>>();

    let decoded_blocks = required_blocks
        .into_par_iter()
        .map(|(index, desc)| {
            let data = &source.get_ref()[desc.offset as usize..(desc.offset + desc.size) as usize];
            read_picture_block(data)
                .and_then(|block| B::new_block(&builder_args, desc.offset, block))
                .map(|block| (index, block))
        })
        .collect_vec_list();

    // prepare an array of `Option<PictureBlock>` for easier access by index by the promise
    let mut blocks_array = {
        let mut blocks_array = Vec::with_capacity(block_descriptors.len());
        // can't use vec![init; count] syntax here because it requires Clone
        for _ in 0..block_descriptors.len() {
            blocks_array.push(None);
        }
        for result in decoded_blocks.into_iter().flatten() {
            let (index, block) = result?;
            blocks_array[index] = Some(block);
        }
        blocks_array
    };
    let token = BustupBlockPromiseToken {
        decoded_blocks: &mut blocks_array,
    };

    let output = B::build(&builder_args, skeleton, token)?;

    Ok(output)
}
