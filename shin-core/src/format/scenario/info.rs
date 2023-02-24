//! Contains types for tables in scenario headers

use crate::format::scenario::types::{U16List, U8List};
use crate::format::text::U16String;
use binrw::file_ptr::FilePtrArgs;
use binrw::{BinRead, BinResult, BinWrite, Endian, FilePtr32};
use std::io::{Read, Seek};

/// Represents a mask, a black and white image specifying a transition between two screens.
#[derive(Debug, BinRead, BinWrite)]
pub struct MaskInfoItem {
    /// The name of the mask, corresponding to the filename that will be loaded when a transition is performed with this mask.
    pub name: U16String,
}
pub type MaskInfo = Vec<MaskInfoItem>;

/// Represents a static picture (`.pic` file).
#[derive(Debug, BinRead, BinWrite)]
pub struct PictureInfoItem {
    /// The name of the picture, corresponding to the filename that will be loaded when the picture is displayed.
    pub name: U16String,

    /// The ID of a different picture that is to be unlocked in the Picture Box (`cgmode`) if this picture is displayed.
    ///
    /// This is needed because the game occasionally composes CGs dynamically out of multiple animated parts, but still wants the (static, pre-assembled) main CG picture to be unlocked in the Picture Box if its main part (i.e. this picture) is shown.
    ///
    /// If there is no linked picture, the value is `-1`.
    pub linked_cg_id: i16,
}
pub type PictureInfo = Vec<PictureInfoItem>;

impl PictureInfoItem {
    pub fn path(&self) -> String {
        format!("/picture/{}.pic", self.name.as_str().to_ascii_lowercase())
    }
}

/// Represents a particular state of a bustup, that is, a sprite composed from multiple replaceable parts.
///
/// In Saku, those parts are a base (character + outfit, without a face), an emotion/expression (the face except for the mouth), and lips (for lipsync).
///
/// This struct specifically represents a combination of (base + emotion); the lip state is determined and stored separately, by the lipsync system.
#[derive(Debug, BinRead, BinWrite)]
pub struct BustupInfoItem {
    /// The base filename of the bustup. This is the file that will be loaded when the bustup is shown.
    pub name: U16String,

    /// The name of the emotion, to be selected from the emotions present in the bustup file.
    pub emotion: U16String,

    /// The ID of the character represented by this bustup, for lipsync purposes: if a voice file with a matching `character_id` in its corresponding [`VoiceMappingInfoItem`] is played, lipsync will be performed on this bustup.
    pub lipsync_character_id: u16,
}
pub type BustupInfo = Vec<BustupInfoItem>;

impl BustupInfoItem {
    pub fn path(&self) -> String {
        format!("/bustup/{}.bup", self.name.as_str().to_ascii_lowercase(),)
    }
}

/// Represents the metadata for a background music (BGM) track.
#[derive(Debug, BinRead, BinWrite)]
pub struct BgmInfoItem {
    /// The internal name of the BGM track, corresponding to the filename the engine will look for when playing the BGM.
    pub name: U16String,

    /// The display name of the BGM track. This is the name the engine will show in the top left corner when the BGM is played. It does not affect the title displayed in the Music Box (`bgmmode`).
    pub display_name: U16String,

    pub unk1: u16, // BGM mode unlock id?
}
pub type BgmInfo = Vec<BgmInfoItem>;

impl BgmInfoItem {
    pub fn path(&self) -> String {
        format!("/bgm/{}.nxa", self.name.as_str().to_ascii_lowercase())
    }
}

/// Represents the metadata for a sound effect.
#[derive(Debug, BinRead, BinWrite)]
pub struct SeInfoItem {
    /// The name of this sound effect; simultaneously the filename the engine will look for when playing the sound effect.
    pub name: U16String,
}
pub type SeInfo = Vec<SeInfoItem>;

impl SeInfoItem {
    pub fn path(&self) -> String {
        format!("/se/{}.nxa", self.name.as_str().to_ascii_lowercase())
    }
}

/// Represents a movie, i.e. a video that can be played back by the engine. The engine makes no fundamental distinction between movies used for cutscenes (e.g. openings) and movies used for animation purposes.
#[derive(Debug, BinRead, BinWrite)]
pub struct MovieInfoItem {
    /// The name of this movie; simultaneously the filename the engine will look for when playing back the movie.
    pub name: U16String,

    /// The ID of the picture that will be displayed instead of the movie after the movie has finished playing. This is only really relevant for movies used in animations; the movies used in cutscenes have this set to 0.
    pub linked_picture_id: u16,

    /// A bitfield controlling the movie's exact playback behavior.
    pub flags: u16, // todo: document meanings of bits

    /// The ID of the BGM that will be unlocked in the Music Box if this movie is played back. This is only really relevant for cutscene movies, where the Music Box entry for an opening theme needs to be unlocked. If there is no linked BGM track, the value is `-1`.
    pub linked_bgm_id: i16,
}
pub type MovieInfo = Vec<MovieInfoItem>;

impl MovieInfoItem {
    pub fn path(&self) -> String {
        format!("/movie/{}.mp4", self.name.as_str().to_ascii_lowercase())
    }
}

/// Matches a voice file to the lipsync character IDs for the characters speaking in the voice file, for lipsync purposes.
#[derive(Debug, BinRead, BinWrite)]
pub struct VoiceMappingInfoItem {
    /// A pattern of voice file paths to be matched to the list of character IDs; either an individual path or a wildcard pattern specified using `*`. Does not include the `voice/` prefix or the file extension.
    pub name_pattern: U16String,

    /// List of character IDs for which a bustup with a matching lipsync character ID in its [`BustupInfoItem`] should have its lips animated if it is currently being displayed while a voice file matching the pattern is being played back.
    pub lipsync_character_ids: U8List<u8>,
}
pub type VoiceMappingInfo = Vec<VoiceMappingInfoItem>;

/// An entry in the Picture Box (`cgmode`).
#[derive(Debug, BinRead, BinWrite)]
pub struct PictureBoxInfoItem {
    /// Internal name of the entry; defines the name of the texture to be loaded from `cgmode.txa` as the thumbnail for this entry.
    pub name: U16String,

    /// List of picture IDs that will be shown in sequence as the player clicks through the entry.
    pub picture_ids: U16List<u16>,
}
pub type PictureBoxInfo = Vec<PictureBoxInfoItem>;

/// An entry in the Music Box (`bgmmode`).
#[derive(Debug, BinRead, BinWrite)]
pub struct MusicBoxInfoItem {
    /// The ID of the BGM track to be played if this entry is selected.
    pub bgm_id: u16,

    /// The index of the name to be displayed on the button for this entry, to be loaded from the `title*` textures in `bgmmode.txa`.
    pub name_index: u16,

    /// If this flag is set to 1, the BGM track will play only once instead of looping at the end. This is used e.g. for opening themes.
    pub once_flag: u16,
}
pub type MusicBoxInfo = Vec<MusicBoxInfoItem>;

trait FinalSegment {
    fn is_final(&self) -> bool;
}

/// An individual instruction for building the data underlying the Character Box (`bupmode`).
#[derive(Debug, BinRead, BinWrite)]
pub enum CharacterBoxSegment {
    /// Defines an individual background to be available for selection in the character box.
    #[brw(magic = 0x0u8)]
    Background {
        /// The index of the picture that constitutes the primary background image (shown in front).
        primary_picture_id: u16,

        /// This value is added to primary_picture_id to get the index of the secondary background image (shown behind the primary image). If 0, no secondary image will be shown.
        secondary_picture_id_offset: u16,
    },

    /// Defines an individual bustup to be available for selection in the character box.
    #[brw(magic = 0x1u8)]
    Bustup { bustup_id: u16 },

    /// Ends a group of facial expressions (表情).
    #[brw(magic = 0x2u8)]
    EndExpressionGroup,

    /// Ends a group of poses (ポーズ).
    #[brw(magic = 0x12u8)]
    EndPoseGroup,

    /// Ends either the list of background definitions at the beginning, or ends an individual character definition, corresponding to a group of outfits (衣装).
    #[brw(magic = 0x22u8)]
    EndDefinition,
}
pub type CharacterBoxInfo = Vec<CharacterBoxSegment>;

/// An individual instruction for building the data underlying a character in the Characters screen (`chars`).
#[derive(Debug, BinRead, BinWrite)]
pub enum CharsSpriteSegment {
    /// Ends the current character definition.
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

/// The data for a character in the Characters screen (`chars`)
#[derive(Debug, BinRead, BinWrite)]
pub struct CharsSpriteInfoItem {
    /// The episode for which the character sprite and description is valid.
    pub episode: u8,

    /// The segments defining the sprites and description for this character.
    #[br(parse_with = parse_terminated_segment_list)]
    pub segments: Vec<CharsSpriteSegment>,
}

pub type CharsSpriteInfo = Vec<CharsSpriteInfoItem>;

/// An individual instruction for building the data underlying the grid in the Characters screen (`chars`).
#[derive(Debug, BinRead, BinWrite)]
pub enum CharsGridSegment {
    /// Ends the current character grid definition.
    #[brw(magic = 0x0u8)]
    End,

    /// Defines a portrait on the grid, with a corresponding character to have its full sprite, name, and description shown when the portrait is selected.
    #[brw(magic = 0x1u8)]
    Portrait {
        /// The page to show this portrait on, between 0 and 3 (both inclusive). The game will automatically generate as many pages as necessary for a specific grid.
        page: u8,

        /// The portrait's X position on the grid.
        grid_x: u8,

        /// The portrait's Y position on the grid.
        grid_y: u8,

        /// The ID of the character this portrait is for, indexing into [`CharsSpriteInfo`].
        character_id: u16,

        /// Defines how the portrait and the corresponding sprite is displayed, and how it behaves on Execute/Resurrect:
        ///  - **in game**: `0` = empty portrait, `1` = alive, `2` = dead (small blood splotch on sprite), `3` = very dead (large blood splotch on sprite)
        ///  - **out of game** (Characters screen selected from main menu): `0` = empty portrait, `1` = two phases (alive/dead), `2` = three phases (alive/missing/dead)
        behaviour: u8,

        /// Modifies the `behaviour` in certain circumstances.
        behaviour_modifier: u8, // todo: find out and document what this does
    },

    /// Defines an individual line segment to be placed on the grid, to connect character portraits.
    #[brw(magic = 0x2u8)]
    Connector {
        /// The page to show this portrait on, between 0 and 3 (both inclusive). The game will automatically generate as many pages as necessary for a specific grid.
        page: u8,

        /// The connector's X position on the grid.
        grid_x: u8,

        /// The connector's Y position on the grid.
        grid_y: u8,

        /// The shape of this connector:
        ///  - `0` = none
        ///  - `1` = left dead end
        ///  - `2` = top dead end
        ///  - `3` = left-top elbow
        ///  - `4` = right dead end
        ///  - `5` = horizontal line
        ///  - `6` = top-right elbow
        ///  - `7` = T to the top
        ///  - `8` = bottom dead end
        ///  - `9` = bottom-left elbow
        ///  - `10` = vertical line
        ///  - `11` = T to the left
        ///  - `12` = right-bottom elbow
        ///  - `13` = T to the bottom
        ///  - `14` = T to the right
        ///  - `15` = cross
        ///  - `16` = dark vertical line
        shape: u8,

        /// The color of this connector: `1` = red, `2` = blue, `3` = yellow.
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

/// An entry on the Tips screen (`tips`).
#[derive(Debug, BinRead, BinWrite)]
pub struct TipsInfoItem {
    /// The episode this tip is for.
    pub episode: u8,

    /// The index of the title to be shown on the tip's selection button. Indexes into the `items` texture in `tips.txa`.
    pub title_index: u16,

    /// The textual title of this tip, to be shown in the headline above the content.
    pub title: U16String,

    /// The main content text.
    pub content: U16String,
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
