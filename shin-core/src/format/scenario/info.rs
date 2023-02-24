//! Contains types for tables in scenario headers

use crate::format::scenario::types::{U16List, U8List};
use crate::format::text::U16String;
use binrw::file_ptr::FilePtrArgs;
use binrw::{BinRead, BinResult, BinWrite, Endian, FilePtr32};
use std::io::{Read, Seek};

#[derive(Debug, BinRead, BinWrite)]
pub struct MaskInfoItem {
    pub name: U16String,
}
pub type MaskInfo = Vec<MaskInfoItem>;

#[derive(Debug, BinRead, BinWrite)]
pub struct PictureInfoItem {
    pub name: U16String,
    pub unk1: i16, // CG mode unlock id?
}
pub type PictureInfo = Vec<PictureInfoItem>;

impl PictureInfoItem {
    pub fn path(&self) -> String {
        format!("/picture/{}.pic", self.name.as_str().to_ascii_lowercase())
    }
}

#[derive(Debug, BinRead, BinWrite)]
pub struct BustupInfoItem {
    pub name: U16String,
    pub emotion: U16String,
    pub unk1: u16, // character id for lipsync?
}
pub type BustupInfo = Vec<BustupInfoItem>;

impl BustupInfoItem {
    pub fn path(&self) -> String {
        format!("/bustup/{}.bup", self.name.as_str().to_ascii_lowercase(),)
    }
}

#[derive(Debug, BinRead, BinWrite)]
pub struct BgmInfoItem {
    pub name: U16String,
    pub display_name: U16String,
    pub unk1: u16, // BGM mode unlock id?
}
pub type BgmInfo = Vec<BgmInfoItem>;

impl BgmInfoItem {
    pub fn path(&self) -> String {
        format!("/bgm/{}.nxa", self.name.as_str().to_ascii_lowercase())
    }
}

#[derive(Debug, BinRead, BinWrite)]
pub struct SeInfoItem {
    pub name: U16String,
}
pub type SeInfo = Vec<SeInfoItem>;

impl SeInfoItem {
    pub fn path(&self) -> String {
        format!("/se/{}.nxa", self.name.as_str().to_ascii_lowercase())
    }
}

#[derive(Debug, BinRead, BinWrite)]
pub struct MovieInfoItem {
    pub name: U16String,
    pub unk1: u16,
    pub unk2: u16,
    pub unk3: i16,
}
pub type MovieInfo = Vec<MovieInfoItem>;

impl MovieInfoItem {
    pub fn path(&self) -> String {
        format!("/movie/{}.mp4", self.name.as_str().to_ascii_lowercase())
    }
}

#[derive(Debug, BinRead, BinWrite)]
pub struct VoiceMappingInfoItem {
    pub name_prefix: U16String,
    pub unk1: U8List<u8>, // character id list for lipsync?
}
pub type VoiceMappingInfo = Vec<VoiceMappingInfoItem>;

#[derive(Debug, BinRead, BinWrite)]
pub struct PictureBoxInfoItem {
    pub name: U16String,
    pub picture_ids: U16List<u16>,
}
pub type PictureBoxInfo = Vec<PictureBoxInfoItem>;

#[derive(Debug, BinRead, BinWrite)]
pub struct MusicBoxInfoItem {
    pub bgm_id: u16,
    pub name_index: u16,
    pub once_flag: u16,
}
pub type MusicBoxInfo = Vec<MusicBoxInfoItem>;

trait FinalSegment {
    fn is_final(&self) -> bool;
}

#[derive(Debug, BinRead, BinWrite)]
pub enum CharacterBoxSegment {
    /// Defines an individual background to be available for selection in the character box
    #[brw(magic = 0x0u8)]
    Background {
        /// The index of the picture that constitutes the primary background image (shown in front)
        primary_picture_id: u16,

        /// This value is added to primary_picture_id to get the index of the secondary background image (shown behind the primary image). If 0, no secondary image will be shown.
        secondary_picture_id_offset: u16,
    },

    /// Defines an individual bustup to be available for selection in the character box
    #[brw(magic = 0x1u8)]
    Bustup { bustup_id: u16 },

    /// Ends a group of facial expressions (表情)
    #[brw(magic = 0x2u8)]
    EndExpressionGroup,

    /// Ends a group of poses (ポーズ)
    #[brw(magic = 0x12u8)]
    EndPoseGroup,

    /// Ends either the list of background definitions at the beginning, or ends an individual character definition, corresponding to a group of outfits (衣装)
    #[brw(magic = 0x22u8)]
    EndDefinition,
}
pub type CharacterBoxInfo = Vec<CharacterBoxSegment>;

#[derive(Debug, BinRead, BinWrite)]
pub enum CharsSpriteSegment {
    #[brw(magic = 0x0u8)]
    End,

    #[brw(magic = 0x1u8)]
    Segment0x1 { unk1: u8 },

    #[brw(magic = 0x2u8)]
    Segment0x2 {
        unk1: u8,
        unk2: u8,
        unk3: U16String,
        unk4: U16String,
    },

    #[brw(magic = 0x3u8)]
    Segment0x3 { unk1: U16String, unk2: U16String },
}

impl FinalSegment for CharsSpriteSegment {
    fn is_final(&self) -> bool {
        match self {
            CharsSpriteSegment::End => true,
            _ => false,
        }
    }
}

#[derive(Debug, BinRead, BinWrite)]
pub struct CharsSpriteInfoItem {
    pub unk1: u8,

    #[br(parse_with = parse_terminated_segment_list)]
    pub segments: Vec<CharsSpriteSegment>,
}

pub type CharsSpriteInfo = Vec<CharsSpriteInfoItem>;

#[derive(Debug, BinRead, BinWrite)]
pub enum CharsGridSegment {
    #[brw(magic = 0x0u8)]
    End,

    #[brw(magic = 0x1u8)]
    Portrait {
        page: u8,
        grid_x: u8,
        grid_y: u8,
        character_id: u16,
        behaviour: u8,
        behaviour_modifier: u8,
    },

    #[brw(magic = 0x2u8)]
    Connector {
        page: u8,
        grid_x: u8,
        grid_y: u8,
        shape: u8,
        color: u8,
    },
}

impl FinalSegment for CharsGridSegment {
    fn is_final(&self) -> bool {
        match self {
            CharsGridSegment::End => true,
            _ => false,
        }
    }
}

#[derive(Debug, BinRead, BinWrite)]
pub struct CharsGridInfoItem {
    #[br(parse_with = parse_terminated_segment_list)]
    pub segments: Vec<CharsGridSegment>,
}

pub type CharsGridInfo = Vec<CharsGridInfoItem>;

#[derive(Debug, BinRead, BinWrite)]
pub struct TipsInfoItem {
    pub unk1: u8,
    pub unk2: u16,
    pub unk3: U16String,
    pub unk4: U16String,
}

// types to parse the info sections

#[derive(Debug, BinRead)]
#[allow(dead_code)] // this stuff is declarative
struct SimpleTable<T: for<'a> BinRead<Args<'a> = ()> + 'static> {
    element_count: u32,
    #[br(count = element_count)]
    elements: Vec<T>,
}

#[derive(Debug, BinRead)]
#[allow(dead_code)] // this stuff is declarative
struct SizedTable<T: for<'a> BinRead<Args<'a> = ()> + 'static> {
    byte_size: u32,
    element_count: u32,
    #[br(count = element_count)]
    elements: Vec<T>,
}

fn parse_sized_segment_list<R: Read + Seek, T: for<'a> BinRead<Args<'a> = ()> + 'static>(
    reader: &mut R,
    endian: Endian,
    (byte_size,): (u32,),
) -> BinResult<Vec<T>> {
    // can this be done more elegantly?
    let initial_pos = reader.stream_position()?;
    let mut result = Vec::new();
    while reader.stream_position()? < initial_pos + byte_size as u64 {
        match T::read_options(reader, endian, ()) {
            Ok(segment) => result.push(segment),
            Err(err) => return Err(err),
        };
    }
    Ok(result)
}

fn parse_terminated_segment_list<
    R: Read + Seek,
    T: for<'a> BinRead<Args<'a> = ()> + FinalSegment + 'static,
>(
    reader: &mut R,
    endian: Endian,
    _: (),
) -> BinResult<Vec<T>> {
    let mut result = Vec::new();
    loop {
        match T::read_options(reader, endian, ()) {
            Ok(segment) => {
                let is_final = segment.is_final();
                result.push(segment);
                if is_final {
                    return Ok(result);
                }
            }
            Err(err) => return Err(err),
        };
    }
}

#[derive(Debug, BinRead)]
#[allow(dead_code)] // this stuff is declarative
struct SizedSegmentList<T: for<'a> BinRead<Args<'a> = ()> + 'static> {
    byte_size: u32,
    #[br(parse_with = parse_sized_segment_list, args(byte_size))]
    segments: Vec<T>,
}

fn parse_simple_section_ptr<R: Read + Seek, T: for<'a> BinRead<Args<'a> = ()> + 'static>(
    reader: &mut R,
    endian: Endian,
    args: FilePtrArgs<()>,
) -> BinResult<Vec<T>> {
    FilePtr32::<SimpleTable<T>>::parse(reader, endian, args).map(|x| x.elements)
}

fn parse_sized_section_ptr<R: Read + Seek, T: for<'a> BinRead<Args<'a> = ()> + 'static>(
    reader: &mut R,
    endian: Endian,
    args: FilePtrArgs<()>,
) -> BinResult<Vec<T>> {
    // maybe check that the size matches for our own sanity?
    FilePtr32::<SizedTable<T>>::parse(reader, endian, args).map(|x| x.elements)
}

fn parse_sized_segment_list_ptr<R: Read + Seek, T: for<'a> BinRead<Args<'a> = ()> + 'static>(
    reader: &mut R,
    endian: Endian,
    args: FilePtrArgs<()>,
) -> BinResult<Vec<T>> {
    FilePtr32::<SizedSegmentList<T>>::parse(reader, endian, args).map(|x| x.segments)
}

// parses the sections from offsets
#[derive(Debug, BinRead)]
pub struct ScenarioInfoTables {
    #[br(parse_with = parse_sized_section_ptr)]
    pub mask_info: MaskInfo,
    #[br(parse_with = parse_sized_section_ptr)]
    pub picture_info: PictureInfo,
    #[br(parse_with = parse_sized_section_ptr)]
    pub bustup_info: BustupInfo,
    #[br(parse_with = parse_sized_section_ptr)]
    pub bgm_info: BgmInfo,
    #[br(parse_with = parse_sized_section_ptr)]
    pub se_info: SeInfo,
    #[br(parse_with = parse_sized_section_ptr)]
    pub movie_info: MovieInfo,
    #[br(parse_with = parse_sized_section_ptr)]
    pub voice_mapping_info: VoiceMappingInfo,
    #[br(parse_with = parse_simple_section_ptr)]
    pub picture_box_info: PictureBoxInfo,
    #[br(parse_with = parse_simple_section_ptr)]
    pub music_box_info: MusicBoxInfo,
    #[br(parse_with = parse_sized_segment_list_ptr)]
    pub character_box_info: CharacterBoxInfo,
    #[br(parse_with = parse_sized_section_ptr)]
    pub chars_sprite_info: CharsSpriteInfo,
    #[br(parse_with = parse_sized_section_ptr)]
    pub chars_grid_info: CharsGridInfo,
    #[br(parse_with = parse_sized_section_ptr)]
    pub tips_info: Vec<TipsInfoItem>,
}

impl ScenarioInfoTables {
    pub fn mask_info(&self, msk_id: i32) -> &MaskInfoItem {
        &self.mask_info[msk_id as usize]
    }
    pub fn picture_info(&self, pic_id: i32) -> &PictureInfoItem {
        &self.picture_info[pic_id as usize]
    }
    pub fn bustup_info(&self, bup_id: i32) -> &BustupInfoItem {
        &self.bustup_info[bup_id as usize]
    }
    pub fn bgm_info(&self, bgm_id: i32) -> &BgmInfoItem {
        &self.bgm_info[bgm_id as usize]
    }
    pub fn se_info(&self, se_id: i32) -> &SeInfoItem {
        &self.se_info[se_id as usize]
    }
    pub fn movie_info(&self, movie_id: i32) -> &MovieInfoItem {
        &self.movie_info[movie_id as usize]
    }
}
