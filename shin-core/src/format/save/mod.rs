//! Support for decrypting and decoding save files.

use anyhow::{anyhow, bail, Result};
use bitreader::BitReader;
use chrono::{NaiveDate, NaiveDateTime};
use num_integer::Integer;
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

/// Stores the persistent variables used by the VM.
/// They are independent of the save slots, used for stuff like global progression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistData(pub Vec<i16>);

impl PersistData {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    fn parse(reader: &mut BitReader) -> Result<Self> {
        let count = reader.read_u32(16)?;

        let mut vec = Vec::with_capacity(count as usize);
        for _ in 0..count {
            vec.push(reader.read_i16(16)?);
        }

        Ok(Self(vec))
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

/// Stores game settings
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

/// Stores minimal info necessary to load a save.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    date_time: NaiveDateTime,
    entry: GameDataEntry,
}

impl GameData {
    fn parse(reader: &mut BitReader) -> Result<Self> {
        let date = parse_date_time(reader)?;
        let v6_arr_count = reader.read_u32(1)?;
        assert_eq!(v6_arr_count, 0);

        let entry = GameDataEntry::parse(reader)?;

        Ok(Self {
            date_time: date,
            entry,
        })
    }
}

fn parse_date_time(reader: &mut BitReader) -> Result<NaiveDateTime> {
    let year = reader.read_u16(12)?;
    let month = reader.read_u8(4)?;
    let day = reader.read_u8(5)?;
    let hour = reader.read_u8(5)?;
    let minute = reader.read_u8(6)?;
    let second = reader.read_u8(6)?;

    NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
        .and_then(|date| date.and_hms_opt(hour as u32, minute as u32, second as u32))
        .ok_or_else(|| anyhow!("Invalid date"))
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
