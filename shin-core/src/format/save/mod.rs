//! Support for decrypting and decoding save files.

use anyhow::{bail, Result};
use bitreader::BitReader;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

mod crc32;
mod obfuscation;

static GAME_KEY: Lazy<u32> = Lazy::new(|| crc32::crc32("うみねこのなく頃に咲".as_bytes(), 0));

fn parse_u8(reader: &mut BitReader) -> Result<u8> {
    Ok(reader.read_u8(8)?)
}

fn parse_u16(reader: &mut BitReader) -> Result<u16> {
    Ok(reader.read_u16(16)?)
}

fn parse_u32(reader: &mut BitReader) -> Result<u32> {
    Ok(reader.read_u32(32)?)
}

fn parse_vec<T, L: TryInto<usize>, E1: Into<anyhow::Error>, E2: Into<anyhow::Error>>(
    reader: &mut BitReader,
    parse_len: impl Fn(&mut BitReader) -> Result<L, E1>,
    parse: impl Fn(&mut BitReader) -> Result<T, E2>,
) -> Result<Vec<T>> {
    let len = parse_len(reader)
        .map_err(|e| e.into())?
        .try_into()
        .map_err(|_| ())
        .unwrap();
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(parse(reader).map_err(|e| e.into())?);
    }
    Ok(vec)
}

fn parse_array<T, E: Into<anyhow::Error>, const N: usize>(
    reader: &mut BitReader,
    parse: impl Fn(&mut BitReader) -> Result<T, E>,
) -> Result<[T; N]> {
    let mut res = [(); N].map(|_| None);

    for res in res.iter_mut() {
        *res = Some(parse(reader).map_err(|e| e.into())?);
    }

    Ok(res.map(|v| v.unwrap()))
}

fn parse_opt<T, E: Into<anyhow::Error>>(
    reader: &mut BitReader,
    parse: impl Fn(&mut BitReader) -> Result<T, E>,
) -> Result<Option<T>> {
    let is_some = reader.read_bool()?;
    if is_some {
        Ok(Some(parse(reader).map_err(|e| e.into())?))
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
        Self::deobfuscate_with_key(data, *GAME_KEY)
    }

    /// Decrypts the game data, returning the raw decrypted bytes.
    /// Can fail if the CRC check fails.
    pub fn deobfuscate_with_key(data: &[u8], key: u32) -> Result<Vec<u8>> {
        obfuscation::decode(data, key)
    }

    /// Same as [Savedata::obfuscate_with_key], but with fixed game key.
    pub fn obfuscate(data: &[u8]) -> Vec<u8> {
        Self::obfuscate_with_key(data, *GAME_KEY)
    }

    /// Encrypts the game data, returning the raw encrypted bytes.
    pub fn obfuscate_with_key(data: &[u8], key: u32) -> Vec<u8> {
        obfuscation::encode(data, key)
    }

    /// Same as [Savedata::decode_with_key], but with fixed game key.
    pub fn decode(data: &[u8]) -> Result<Self> {
        Self::decode_with_key(data, *GAME_KEY)
    }

    /// Decrypts & decodes the game data, returning the parsed data.
    /// Can fail if the CRC check fails or the decoding fails.
    pub fn decode_with_key(data: &[u8], key: u32) -> Result<Self> {
        let data = Self::deobfuscate_with_key(data, key)?;
        let mut reader = BitReader::new(&data);
        Self::parse(&mut reader)
    }
}

impl Savedata {
    fn parse(reader: &mut BitReader) -> Result<Self> {
        let some_ctr = reader.read_u32(8)?;
        if some_ctr == 0 {
            todo!("Construct default Savedata")
        }
        if some_ctr > 1 {
            bail!("Invalid Savedata: some_ctr > 1")
        }

        let save_menu_position = reader.read_u8(7)?;
        let play_seconds = reader.read_u32(32)?;
        reader.align(1)?;

        let persist_data = PersistData::parse(reader)?;
        let save_vectors = SaveVectors::parse(reader)?;
        let settings = Settings::parse(reader)?;
        let auto_save_slot = parse_opt(reader, GameData::parse)?;
        let manual_save_slots = parse_array(reader, |r| parse_opt(r, GameData::parse))?;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistData(pub Vec<i16>);

impl PersistData {
    fn parse(reader: &mut BitReader) -> Result<Self> {
        let count = reader.read_u32(16)?;

        let mut vec = Vec::with_capacity(count as usize);
        for _ in 0..count {
            vec.push(reader.read_i16(16)?);
        }

        Ok(Self(vec))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveVectors {
    pub seen_messages_mask: Vec<u32>,
    // seen choices?
    pub vec2: Vec<u32>,
    // chosen variants?
    pub vec3: Vec<u8>,
    // unlocked CGs
    pub vec4: Vec<u32>,
    // unlocked BGMs
    pub vec5: Vec<u32>,
    // unlocked tips?
    pub vec6: Vec<u32>,
}

impl SaveVectors {
    pub fn parse(reader: &mut BitReader) -> Result<Self> {
        reader.align(1)?;

        Ok(Self {
            seen_messages_mask: parse_vec(reader, parse_u16, |reader| reader.read_u32(32))?,
            vec2: parse_vec(reader, parse_u16, |reader| reader.read_u32(32))?,
            vec3: parse_vec(reader, parse_u16, |reader| reader.read_u8(4))?,
            vec4: parse_vec(reader, parse_u16, |reader| reader.read_u32(32))?,
            vec5: parse_vec(reader, parse_u16, |reader| reader.read_u32(32))?,
            vec6: parse_vec(reader, parse_u16, |reader| reader.read_u32(32))?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
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

impl Settings {
    pub fn parse(reader: &mut BitReader) -> Result<Self> {
        reader.align(1)?;

        Ok(Self {
            v0_bgmvol: reader.read_u8(7)?,
            v1_sfxvol: reader.read_u8(7)?,
            v2_voicevol: reader.read_u8(7)?,
            v3_sysvol: reader.read_u8(7)?,
            v4_voicefocus: reader.read_bool()?,
            v5_voicepanapot: reader.read_bool()?,
            v6: reader.read_bool()?,
            v7: reader.read_u8(2)?,
            v8: reader.read_u8(2)?,
            v9_msgspeed: reader.read_u8(7)?,
            v10_skipspeed: reader.read_u8(7)?,
            v11_disallowskipunread: reader.read_bool()?,
            v12: reader.read_bool()?,
            v13_msgwinalpha: reader.read_u8(7)?,
            v14_showroutenavi: reader.read_bool()?,
            v15: reader.read_bool()?,
            v16_showtoucheffect: reader.read_bool()?,
            v17_showscenetitle: reader.read_bool()?,
            v18_showsongtitle: reader.read_bool()?,
            v19: reader.read_u32(32)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    date_time: DateTime,
    entry: GameDataEntry,
}

impl GameData {
    fn parse(reader: &mut BitReader) -> Result<Self> {
        let date = DateTime::parse(reader)?;
        let v6_arr_count = reader.read_u32(1)?;
        assert_eq!(v6_arr_count, 0);

        let entry = GameDataEntry::parse(reader)?;

        Ok(Self {
            date_time: date,
            entry,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl DateTime {
    fn parse(reader: &mut BitReader) -> Result<Self> {
        Ok(Self {
            year: reader.read_u16(12)?,
            month: reader.read_u8(4)?,
            day: reader.read_u8(5)?,
            hour: reader.read_u8(5)?,
            minute: reader.read_u8(6)?,
            second: reader.read_u8(6)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDataEntry {
    pub scenario_id: i32,
    pub random_seed: u32,
    pub save_position: u32,
    pub selection_data: Vec<u8>,
}

impl GameDataEntry {
    fn parse(reader: &mut BitReader) -> Result<Self> {
        reader.align(1)?;

        Ok(Self {
            scenario_id: reader.read_i32(32)?,
            random_seed: reader.read_u32(32)?,
            save_position: reader.read_u32(32)?,
            selection_data: parse_vec(reader, parse_u32, parse_u8)?,
        })
    }
}
