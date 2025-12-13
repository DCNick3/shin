//! Support for decrypting and decoding save files.

use anyhow::Result;
use bitbuffer::{BitRead, BitWrite, BitWriteStream, Endianness};
use chrono::{NaiveDate, NaiveDateTime};
use num_integer::Integer;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::crc32;

mod obfuscation;

type Endian = bitbuffer::BigEndian;
const ENDIAN: Endian = bitbuffer::BigEndian;
type BitReadStream<'a, E = Endian> = bitbuffer::BitReadStream<'a, E>;

const GAME_KEY: u32 = crc32::crc32("うみねこのなく頃に咲".as_bytes(), 0);

fn read_u8<E: Endianness>(reader: &mut BitReadStream<E>) -> bitbuffer::Result<u8> {
    reader.read_int(8)
}

fn read_u16<E: Endianness>(reader: &mut BitReadStream<E>) -> bitbuffer::Result<u16> {
    reader.read_int(16)
}

fn read_u32<E: Endianness>(reader: &mut BitReadStream<E>) -> bitbuffer::Result<u32> {
    reader.read_int(32)
}

fn read_vec<'a, T, E: Endianness, L: TryInto<usize>>(
    reader: &mut BitReadStream<'a, E>,
    parse_len: impl Fn(&mut BitReadStream<'a, E>) -> bitbuffer::Result<L>,
    parse: impl Fn(&mut BitReadStream<'a, E>) -> bitbuffer::Result<T>,
) -> bitbuffer::Result<Vec<T>> {
    let len = parse_len(reader)?.try_into().map_err(|_| ()).unwrap();
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(parse(reader)?);
    }
    Ok(vec)
}

fn read_array<'a, T, E: Endianness, const N: usize>(
    reader: &mut BitReadStream<'a, E>,
    parse: impl Fn(&mut BitReadStream<'a, E>) -> bitbuffer::Result<T>,
) -> bitbuffer::Result<[T; N]> {
    let mut res = [(); N].map(|_| None);

    for res in res.iter_mut() {
        *res = Some(parse(reader)?);
    }

    Ok(res.map(|v| v.unwrap()))
}

fn parse_opt<'a, T, E: Endianness>(
    reader: &mut BitReadStream<'a, E>,
    parse: impl Fn(&mut BitReadStream<'a, E>) -> bitbuffer::Result<T>,
) -> bitbuffer::Result<Option<T>> {
    let is_some = reader.read_bool()?;
    if is_some {
        Ok(Some(parse(reader)?))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Savedata {
    pub save_menu_position: u8,
    pub play_seconds: u32,
    pub persist_data: PersistData,
    pub save_vectors: SaveVectors,
    pub settings: Settings,
    pub auto_save_slot: Option<GameData>,
    #[serde(with = "BigArray")]
    pub manual_save_slots: [Option<GameData>; 100],
}

impl Savedata {
    pub fn obfuscation_key_from_seed(seed: &str) -> u32 {
        crc32::crc32(seed.as_bytes(), 0)
    }

    /// Same as [Savedata::deobfuscate_with_key], but with fixed game key.
    pub fn deobfuscate(data: &[u8]) -> Result<Vec<u8>> {
        Self::deobfuscate_with_key(data, GAME_KEY)
    }

    /// Decrypts the game data, returning the raw decrypted bytes.
    /// Can fail if the CRC check fails.
    pub fn deobfuscate_with_key(data: &[u8], key: u32) -> Result<Vec<u8>> {
        obfuscation::decode(data, key)
    }

    /// Same as [Savedata::obfuscate_with_key], but with fixed game key.
    pub fn obfuscate(data: &[u8]) -> Vec<u8> {
        Self::obfuscate_with_key(data, GAME_KEY)
    }

    /// Encrypts the game data, returning the raw encrypted bytes.
    pub fn obfuscate_with_key(data: &[u8], key: u32) -> Vec<u8> {
        obfuscation::encode(data, key)
    }

    /// Same as [Savedata::decode_with_key], but with fixed game key.
    pub fn decode(data: &[u8]) -> Result<Self> {
        Self::decode_with_key(data, GAME_KEY)
    }

    /// Decrypts & decodes the game data, returning the parsed data.
    /// Can fail if the CRC check fails or the decoding fails.
    pub fn decode_with_key(data: &[u8], key: u32) -> Result<Self> {
        let data = Self::deobfuscate_with_key(data, key)?;
        let buffer = bitbuffer::BitReadBuffer::new(&data, ENDIAN);
        let mut reader = BitReadStream::new(buffer);
        Ok(Self::read(&mut reader)?)
    }
}

impl<'a, E: Endianness> BitRead<'a, E> for Savedata {
    fn read(reader: &mut BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        let some_ctr: u32 = reader.read_int(8)?;
        if some_ctr == 0 {
            todo!("Construct default Savedata")
        }
        if some_ctr > 1 {
            panic!("Invalid Savedata: some_ctr > 1") // TODO: bitbuffer doesn't have a way to pass a custom error =(
        }

        let save_menu_position = reader.read_int(7)?;
        let play_seconds = reader.read_int(32)?;
        reader.align()?;

        let persist_data = PersistData::read(reader)?;
        let save_vectors = SaveVectors::read(reader)?;
        let settings = Settings::read(reader)?;
        let auto_save_slot = parse_opt(reader, GameData::read)?;
        let manual_save_slots = read_array(reader, |r| parse_opt(r, GameData::read))?;

        Ok(Self {
            save_menu_position,
            play_seconds,
            persist_data,
            save_vectors,
            settings,
            auto_save_slot,
            manual_save_slots,
        })
    }
}

/// Stores the persistent variables used by the VM.
/// They are independent of the save slots, used for stuff like global progression.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistData(pub Vec<i16>);

impl PersistData {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get(&self, index: i32) -> i32 {
        if index < 0 || index >= self.0.len() as i32 {
            0
        } else {
            self.0[index as usize] as i32
        }
    }

    pub fn set(&mut self, index: i32, value: i32) {
        if self.0.len() <= index as usize {
            // allocate more space, round up to 64
            let new_len = Integer::div_ceil(&(index as usize), &64) * 64;
            self.0.resize(new_len, 0);
        }
        self.0[index as usize] = value.try_into().expect("value too large");
    }
}

impl<'a, E: Endianness> BitRead<'a, E> for PersistData {
    fn read(stream: &mut bitbuffer::BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        Ok(Self(read_vec(stream, read_u16, |r| r.read_int(16))?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveVectors {
    /// This stores a mask of all seen messages to not skip unseen messages (if the setting is enabled)
    pub seen_messages: Vec<u32>,
    /// This stores a mask of all seen choice variants to change how they are displayed
    pub select_seen_choices: Vec<u32>,
    /// This stores
    pub select_chosen_variants: Vec<u8>,
    pub unlocked_cgs: Vec<u32>,
    pub unlocked_bgms: Vec<u32>,
    pub unlocked_movies: Vec<u32>,
}

impl<'a, E: Endianness> BitRead<'a, E> for SaveVectors {
    fn read(stream: &mut BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        stream.align()?;

        Ok(Self {
            seen_messages: read_vec(stream, read_u16, |stream| stream.read_int(32))?,
            select_seen_choices: read_vec(stream, read_u16, |stream| stream.read_int(32))?,
            select_chosen_variants: read_vec(stream, read_u16, |stream| stream.read_int(4))?,
            unlocked_cgs: read_vec(stream, read_u16, |stream| stream.read_int(32))?,
            unlocked_bgms: read_vec(stream, read_u16, |stream| stream.read_int(32))?,
            unlocked_movies: read_vec(stream, read_u16, |stream| stream.read_int(32))?,
        })
    }
}

/// Stores game settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // it appears that some settings do not have an in-game toggle
    pub v0_bgmvol: u8,
    pub v1_sfxvol: u8,
    pub v2_voicevol: u8,
    pub v3_sysvol: u8,
    pub v4_voicefocus: bool,
    pub v5_voicepanapot: bool,
    pub v6: bool,
    pub v7: u8,
    pub v8: u8,
    pub v9_msgspeed: u8,
    pub v10_skipspeed: u8,
    pub v11_disallowskipunread: bool,
    pub v12: bool,
    pub v13_msgwinalpha: u8,
    pub v14_showroutenavi: bool,
    pub v15: bool,
    pub v16_showtoucheffect: bool,
    pub v17_showscenetitle: bool,
    pub v18_showsongtitle: bool,
    pub v19: u32,
}

impl<'a, E: Endianness> BitRead<'a, E> for Settings {
    fn read(reader: &mut bitbuffer::BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        reader.align()?;

        let v0_bgmvol = reader.read_int(7)?;
        let v1_sfxvol = reader.read_int(7)?;
        let v2_voicevol = reader.read_int(7)?;
        let v3_sysvol = reader.read_int(7)?;
        let v4_voicefocus = reader.read_bool()?;
        let v5_voicepanapot = reader.read_bool()?;
        let v6 = reader.read_bool()?;
        let v7 = reader.read_int(2)?;
        let v8 = reader.read_int(2)?;
        let v9_msgspeed = reader.read_int(7)?;
        let v10_skipspeed = reader.read_int(7)?;
        let v11_disallowskipunread = reader.read_bool()?;
        let v12 = reader.read_bool()?;
        let v13_msgwinalpha = reader.read_int(7)?;
        let v14_showroutenavi = reader.read_bool()?;
        let v15 = reader.read_bool()?;
        let v16_showtoucheffect = reader.read_bool()?;
        let v17_showscenetitle = reader.read_bool()?;
        let v18_showsongtitle = reader.read_bool()?;
        let v19 = reader.read_int(32)?;

        Ok(Self {
            v0_bgmvol,
            v1_sfxvol,
            v2_voicevol,
            v3_sysvol,
            v4_voicefocus,
            v5_voicepanapot,
            v6,
            v7,
            v8,
            v9_msgspeed,
            v10_skipspeed,
            v11_disallowskipunread,
            v12,
            v13_msgwinalpha,
            v14_showroutenavi,
            v15,
            v16_showtoucheffect,
            v17_showscenetitle,
            v18_showsongtitle,
            v19,
        })
    }
}

/// Stores minimal info necessary to load a save.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    date_time: NaiveDateTime,
    entry: GameDataEntry,
}

impl<'a, E: Endianness> BitRead<'a, E> for GameData {
    fn read(reader: &mut BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        let date = parse_date_time(reader)?;
        let v6_arr_count: u32 = reader.read_int(1)?;
        assert_eq!(v6_arr_count, 0);

        let entry = GameDataEntry::read(reader)?;

        Ok(Self {
            date_time: date,
            entry,
        })
    }
}

fn parse_date_time<E: Endianness>(
    reader: &mut BitReadStream<E>,
) -> bitbuffer::Result<NaiveDateTime> {
    let year: u32 = reader.read_int(12)?;
    let month = reader.read_int(4)?;
    let day = reader.read_int(5)?;
    let hour = reader.read_int(5)?;
    let minute = reader.read_int(6)?;
    let second = reader.read_int(6)?;

    let datetime = NaiveDate::from_ymd_opt(year as i32, month, day)
        .and_then(|date| date.and_hms_opt(hour, minute, second))
        .expect("invalid date");

    Ok(datetime)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDataEntry {
    pub scenario_id: i32,
    pub random_seed: u32,
    pub save_position: u32,
    pub selection_data: SelectionData,
}

impl<'a, E: Endianness> BitRead<'a, E> for GameDataEntry {
    fn read(reader: &mut BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        reader.align()?;

        let scenario_id = reader.read_int(32)?;
        let random_seed = reader.read_int(32)?;
        let save_position = reader.read_int(32)?;
        let selection_data = SelectionData::read(reader)?;

        Ok(Self {
            scenario_id,
            random_seed,
            save_position,
            selection_data,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionData(Vec<u8>);

impl<'a, E: Endianness> BitRead<'a, E> for SelectionData {
    fn read(reader: &mut BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        Ok(Self(read_vec(reader, read_u32, read_u8)?))
    }
}

impl<E: Endianness> BitWrite<E> for SelectionData {
    fn write(&self, _stream: &mut BitWriteStream<E>) -> bitbuffer::Result<()> {
        todo!()
    }
}
