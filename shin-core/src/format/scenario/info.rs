//! Contains types for tables in scenario headers.
//!
//! These tables provide various kinds of metadata for different engine features. Most importantly, there is one table for each type of asset the game needs to load, serving as a reference to an asset file, together with some additional metadata that is always the same each time a particular asset is loaded. Each asset table is a [`Vec`] of *info items*, where an info item is a struct containing a filename and potentially additional data. The index into the [`Vec`] is known as the asset's **ID**: when the script needs to specify an asset to load, it will only specify this ID, and all other necessary information is obtained by looking up the info item stored at this index in the [`Vec`].
//!
//! For example, the list of pictures — static images used primarily as backgrounds — is represented by [`PictureInfo`], which is just a [`Vec<PictureInfoItem>`]. If the script needs a picture to be loaded, it will specify an ID, which is looked up in this [`Vec`]. Concretely, let us consider the example of the “This story is nothing but fiction” picture at the beginning of every Umineko episode. For the purpose of displaying this picture, the script ultimately contains a `LAYERLOAD` instruction with the ID `1012` as its argument. When that instruction is executed, the engine will look up the [`PictureInfoItem`] with index `1012` in the [`PictureInfo`] table. That [`PictureInfoItem`] will contain the information that the desired picture has the filename “TEXT001”. Consequently, the engine will know to load the asset file located at `picture/text001.pic`, and display its contents on the screen.
//!
//! In some cases, there are also cross-references between different asset tables, which are specified by ID as well. For example, a [`MovieInfoItem`] contains the ID of a picture to be shown after the movie finishes playing. In this case, the engine will first look up the [`MovieInfoItem`] in the [`MovieInfo`] table, then secondarily look up the [`PictureInfoItem`] in the [`PictureInfo`] table whose index corresponds to the `linked_picture_id` field in the [`MovieInfoItem`].
//!
//! Apart from the asset tables, there are also a few other data blocks for various game-specific features, such as the Picture Box (`cgmode`) and Music Box (`bgmmode`), or Umineko's character relationship grid (`chars`). These may be somewhat more freeform in structure than the simple tables listed above, and their corresponding entry structs often also contain IDs linking to other data tables, as explained above.

use crate::format::scenario::types::{U16List, U8List};
use crate::format::text::U16String;
use binrw::file_ptr::FilePtrArgs;
use binrw::{BinRead, BinResult, BinWrite, Endian, FilePtr32};
use std::io::{Read, Seek};

/// References a mask, a black and white image specifying a transition between two screens.
#[derive(Debug, BinRead, BinWrite)]
pub struct MaskInfoItem {
    /// The internal name of the mask. Corresponds to the base filename of the `.msk` file the engine will load from the `mask/` directory when a transition with this mask is to be performed.
    pub name: U16String,
}
pub type MaskInfo = Vec<MaskInfoItem>;

/// References a static picture (`.pic` file).
#[derive(Debug, BinRead, BinWrite)]
pub struct PictureInfoItem {
    /// The internal name of the picture. Corresponds to the base filename of the `.pic` file the engine will load from the `picture/` directory when the picture is to be displayed.
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

/// References a particular state of a bustup, that is, a sprite composed from multiple replaceable parts.
///
/// In Saku, those parts are a base (character + outfit, without a face), an emotion/expression (the face except for the mouth), and lips (for lipsync).
///
/// This struct specifically references a combination of (base + emotion); the lip state is determined and stored separately, by the lipsync system.
#[derive(Debug, BinRead, BinWrite)]
pub struct BustupInfoItem {
    /// The base filename of the bustup. When the bustup is shown, the engine will load the `.bup` file with this basename from the `bustup` directory, regardless of the referenced emotion.
    pub name: U16String,

    /// The internal name of the emotion, to be selected from the emotions present in the bustup file.
    pub emotion: U16String,

    /// The ID of the character referenced by this bustup, for lipsync purposes: if a voice file with a matching `character_id` in its corresponding [`VoiceMappingInfoItem`] is played, lipsync will be performed on this bustup.
    pub lipsync_character_id: u16,
}
pub type BustupInfo = Vec<BustupInfoItem>;

impl BustupInfoItem {
    pub fn path(&self) -> String {
        format!("/bustup/{}.bup", self.name.as_str().to_ascii_lowercase(),)
    }
}

/// References a background music (BGM) track.
#[derive(Debug, BinRead, BinWrite)]
pub struct BgmInfoItem {
    /// The internal name of the BGM track. Corresponds to the base filename of the `.nxa` file the engine will load from the `bgm/` directory when the BGM is to be played.
    pub name: U16String,

    /// The display name of the BGM track. This is the name the engine will show in the top left corner when BGM playback starts. It does not affect the title displayed in the Music Box (`bgmmode`).
    pub display_name: U16String,

    /// The ID of another BGM track that should be unlocked in the Music Box (`bgmmode`) in addition to this track, when this track is played. `-1` if there is no linked BGM track.
    pub linked_bgm_id: i16,
}
pub type BgmInfo = Vec<BgmInfoItem>;

impl BgmInfoItem {
    pub fn path(&self) -> String {
        format!("/bgm/{}.nxa", self.name.as_str().to_ascii_lowercase())
    }
}

/// References a sound effect (SE).
#[derive(Debug, BinRead, BinWrite)]
pub struct SeInfoItem {
    /// The internal name of this sound effect. Corresponds to the base filename of the `.nxa` file the engine will load from the `se/` directory when the sound effect is to be played.
    pub name: U16String,
}
pub type SeInfo = Vec<SeInfoItem>;

impl SeInfoItem {
    pub fn path(&self) -> String {
        format!("/se/{}.nxa", self.name.as_str().to_ascii_lowercase())
    }
}

/// References a movie, i.e. a video that can be played back by the engine. The engine makes no fundamental distinction between movies used for cutscenes (e.g. openings) and movies used for animation purposes.
#[derive(Debug, BinRead, BinWrite)]
pub struct MovieInfoItem {
    /// The name of this movie. Corresponds to the base filename of the `.mp4` file the engine will load from the `movie/` directory when the movie is to be played.
    pub name: U16String,

    /// The ID of the picture (indexing into [`PictureInfo`]) that will be displayed instead of the movie after the movie has finished playing. This is only really relevant for movies used in animations; the movies used in cutscenes have this set to 0.
    pub linked_picture_id: u16,

    /// A bitfield controlling the movie's exact playback behavior.
    pub flags: u16, // todo: document meanings of bits

    /// The ID of the BGM (indexing into [`BgmInfo`]) that will be unlocked in the Music Box if this movie is played back. This is only really relevant for cutscene movies, where the Music Box entry for an opening theme needs to be unlocked. If there is no linked BGM track, the value is `-1`.
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

    /// List of picture IDs (indexing into [`PictureInfo`]) that will be shown in sequence as the player clicks through the entry.
    pub picture_ids: U16List<u16>,
}
pub type PictureBoxInfo = Vec<PictureBoxInfoItem>;

/// An entry in the Music Box (`bgmmode`).
#[derive(Debug, BinRead, BinWrite)]
pub struct MusicBoxInfoItem {
    /// The ID of the BGM track (indexing into [`BgmInfo`]) to be played if this entry is selected.
    pub bgm_id: u16,

    /// The index of the name to be displayed on the button for this entry, to be loaded from the `title*` textures in `bgmmode.txa`.
    pub name_index: u16,

    /// If this flag is set to 1, the BGM track will play only once instead of looping at the end. This is used e.g. for opening themes.
    pub once_flag: u16,
}
pub type MusicBoxInfo = Vec<MusicBoxInfoItem>;

/// An individual instruction for building the data underlying the Character Box (`bupmode`).
#[derive(Debug, BinRead, BinWrite)]
pub enum CharacterBoxSegment {
    /// Defines an individual background to be available for selection in the character box. The background will be shown behind the selected bustup.
    #[brw(magic = 0x0u8)]
    Background {
        /// The index of the picture (indexing into [`PictureInfo`]) that constitutes the primary background image (shown in front).
        primary_picture_id: u16,

        /// This value is added to primary_picture_id to get the index of the secondary background image, shown behind the primary image. If 0, no secondary image will be shown.
        secondary_picture_id_offset: u16,
    },

    /// Defines an individual bustup to be available for selection in the character box.
    #[brw(magic = 0x1u8)]
    Bustup {
        /// The ID of the bustup reference to be displayed. Indexes into [`BustupInfo`].
        bustup_id: u16,
    },

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

/// Defines how a `chars` grid portrait is displayed.
#[derive(Debug, BinRead, BinWrite)]
#[brw(repr = u8)]
pub enum CharsPortraitDisplayMode {
    /// Portrait will be shown in full color.
    Alive = 0,

    /// Portrait will be shown in red.
    Dead = 1,

    /// Portrait will be shown in grayscale.
    Missing = 2,

    /// The portrait will be divided diagonally; the upper left half will be shown in full color, while the bottom right half will be shown in red.
    HalfDead = 3,
}

/// An individual instruction for building the data underlying a character in the Characters screen (`chars`).
#[derive(Debug, BinRead, BinWrite)]
pub enum CharsSpriteSegment {
    /// Begins a new character state. A character state is a combination of (sprite variants + name/description); multiple character states can be switched between using the “Execute”/“Resurrect” buttons below the selection grid. A character can have 1 to 4 defined states, however the game can display at most 3 states.
    #[brw(magic = 0x1u8)]
    BeginState {
        /// The index of the new state. Should monotonically increment starting at 1 for the first state.
        index: u8,
    },

    /// Defines one sprite variant, i.e. a combination of a portrait texture, a full texture, and the portrait display mode. Different sprite variants can be switched between using the “Change” button below the selection grid.
    #[brw(magic = 0x2u8)]
    SpriteVariant {
        /// The index of this variant. Should monotonically increment starting at 0 for the first variant.
        variant_index: u8,

        /// How the portrait is to be displayed (alive/dead/missing/half-dead). Does not affect the display of the full sprite; different textures are used to display sprites in different states of aliveness.
        portrait_display_mode: CharsPortraitDisplayMode,

        /// The texture to use for the portrait on the grid. Loaded from `chars.txa`.
        portrait_texture_name: U16String,

        /// The texture to use for the full sprite displayed on the right side. Corresponds to the basename of a `.txa` file in the `chars/` directory; the equivalently named texture in that file will be used for the full sprite.
        full_texture_name: U16String,
    },

    /// Defines the name and description to be displayed for the current character state.
    #[brw(magic = 0x3u8)]
    Texts {
        /// The character name that will be displayed above the description.
        name: U16String,

        /// The full description of the character at their current state.
        description: U16String,
    },
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

/// The shape of an individual connector between portraits in the `chars` grid.
#[derive(Debug, BinRead, BinWrite)]
#[brw(repr = u8)]
pub enum CharsGridConnectorShape {
    /// No connector is displayed.
    None = 0,

    /// `╴`: a line segment only on the left side.
    LeftDeadEnd = 1,

    /// `╵`: a line segment only on the top.
    TopDeadEnd = 2,

    /// `┘`: an elbow with lines facing towards the left side and the top.
    LeftTopElbow = 3,

    /// `╶`: a line segment only on the right side.
    RightDeadEnd = 4,

    /// `─`: a line from left to right.
    HorizontalLine = 5,

    /// `└`: an elbow with lines facing towards the top and the right side.
    TopRightElbow = 6,

    /// `┴`: a T with its orthogonal segment pointing towards the top.
    TopPointingT = 7,

    /// `╷`: a line segment only on the bottom.
    BottomDeadEnd = 8,

    /// `┐`: an elbow with lines facing towards the bottom and the left side.
    BottomLeftElbow = 9,

    /// `│`: a line from top to bottom.
    VerticalLine = 10,

    /// `┤`: a T with its orthogonal segment pointing towards the left side.
    LeftPointingT = 11,

    /// `┌`: an elbow with lines facing towards the right side and the bottom.
    RightBottomElbow = 12,

    /// `┬`: a T with its orthogonal segment pointing towards the bottom.
    BottomPointingT = 13,

    /// `├`: a T with its orthogonal segment pointing towards the right side.
    RightPointingT = 14,

    /// `┼`: a cross with line segments pointing to all four sides.
    Cross = 15,

    /// A vertical line, but displayed darker than usual. (This is never used in the game and might just be a failure mode of the engine for out-of-bounds shapes)
    DarkVerticalLine = 16,
}

/// The color of an individual connector between portraits in the `chars` grid.
#[derive(Debug, BinRead, BinWrite)]
#[brw(repr = u8)]
pub enum CharsGridConnectorColor {
    Red = 1,
    Blue = 2,
    Yellow = 3,
}

/// An individual instruction for building the data underlying the grid in the Characters screen (`chars`).
#[derive(Debug, BinRead, BinWrite)]
pub enum CharsGridSegment {
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

        /// The index of the character state (see [`CharsSpriteSegment`]) to display initially. If 0, the portrait will display as an unselectable empty frame.
        default_state: u8,

        /// The index of the character sprite variant (see [`CharsSpriteSegment`]) to display initially.
        default_variant: u8,
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

        /// The shape of this connector.
        shape: CharsGridConnectorShape,

        /// The color of this connector.
        color: CharsGridConnectorColor,
    },
}

/// A grid for the Characters screen (`chars`). Contains portraits which can be selected to reveal additional information about the character, and connectors making up lines between the portraits to show relationships between the characters.
///
/// The script can select a particular grid by ID to set it as the one that will be shown when opening `chars` from in-game. In addition, the first 8 grids are respectively the Episode 1-8 ones selectable from the main menu.
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
struct SizedSegmentList<T: for<'a> BinRead<Args<'a> = ()> + 'static> {
    byte_size: u32,
    #[br(parse_with = parse_sized_segment_list, args(byte_size))]
    segments: Vec<T>,
}

#[derive(Debug, BinRead)]
#[allow(dead_code)] // this stuff is declarative
enum EndableSegment<T: for<'a> BinRead<Args<'a> = ()> + 'static> {
    #[brw(magic = 0x0u8)]
    End,
    Some(T),
}

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

fn parse_terminated_segment_list<R: Read + Seek, T: for<'a> BinRead<Args<'a> = ()> + 'static>(
    reader: &mut R,
    endian: Endian,
    _: (),
) -> BinResult<Vec<T>> {
    let mut result = Vec::new();
    loop {
        match EndableSegment::read_options(reader, endian, ()) {
            Ok(EndableSegment::Some(segment)) => result.push(segment),
            Ok(EndableSegment::End) => return Ok(result),
            Err(err) => return Err(err),
        };
    }
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
