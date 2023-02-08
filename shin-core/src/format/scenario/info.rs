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

#[derive(Debug, BinRead, BinWrite)]
pub struct VoiceMappingInfoItem {
    pub name_prefix: U16String,
    pub unk1: U8List<u8>, // character id list for lipsync?
}
pub type VoiceMappingInfo = Vec<VoiceMappingInfoItem>;

#[derive(Debug, BinRead, BinWrite)]
pub struct Section64InfoItem {
    pub unk1: U16String,
    pub unk2: U16List<u16>,
}
pub type Section64Info = Vec<Section64InfoItem>;

#[derive(Debug, BinRead, BinWrite)]
pub struct Section68InfoItem {
    pub unk1: u16,
    pub unk2: u16,
    pub unk3: u16,
}
pub type Section68Info = Vec<Section68InfoItem>;

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
    pub section64_info: Section64Info,
    #[br(parse_with = parse_simple_section_ptr)]
    pub section68_info: Section68Info,
    // I don't know how to parse these sections yet
    pub offset_72: u32,
    pub offset_76: u32,
    pub offset_80: u32,

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
