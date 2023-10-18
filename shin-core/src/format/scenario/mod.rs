//! Support for SNR file format, storing the game scenario.
//!
//! See also [crate::vm] for the VM that runs the scenario.

pub mod info;
pub mod instruction_elements;
pub mod instructions;
pub mod types;

use crate::format::scenario::info::ScenarioInfoTables;
use crate::format::scenario::instructions::{CodeAddress, Instruction};
use anyhow::{bail, Result};
use binrw::BinRead;
use bytes::Bytes;
use std::io::Cursor;

#[derive(BinRead)]
#[br(little, magic = b"SNR ")]
#[allow(dead_code)] // this stuff is declarative
struct ScenarioHeader {
    pub size: u32,
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u32,
    pub unk5: u32,
    pub unk6: u32,
    pub code_offset: u32,
    pub info_tables: ScenarioInfoTables,
}

#[allow(unused)]
pub struct Scenario {
    info_tables: ScenarioInfoTables,
    entrypoint_address: CodeAddress,
    raw_data: Bytes,
}

impl Scenario {
    pub fn new(data: Bytes) -> Result<Self> {
        let mut cur = Cursor::new(&data);
        let header = ScenarioHeader::read(&mut cur)?;

        if header.size as usize != data.len() {
            bail!("SNR file size mismatch");
        }

        Ok(Self {
            info_tables: header.info_tables,
            entrypoint_address: CodeAddress(header.code_offset),
            raw_data: data,
        })
    }

    pub fn info_tables(&self) -> &ScenarioInfoTables {
        &self.info_tables
    }

    pub fn raw(&self) -> &[u8] {
        &self.raw_data
    }

    pub fn entrypoint_address(&self) -> CodeAddress {
        self.entrypoint_address
    }

    pub fn instruction_reader(&self, offset: CodeAddress) -> InstructionReader {
        InstructionReader::new(self.raw_data.clone(), offset)
    }
}

pub struct InstructionReader {
    cur: Cursor<Bytes>,
}

impl InstructionReader {
    pub fn new(data: Bytes, offset: CodeAddress) -> Self {
        let mut cur = Cursor::new(data);
        cur.set_position(offset.0 as u64);
        Self { cur }
    }

    #[inline]
    pub fn read(&mut self) -> Result<Instruction> {
        let instruction = Instruction::read(&mut self.cur)?;
        Ok(instruction)
    }

    #[inline]
    pub fn position(&self) -> CodeAddress {
        CodeAddress(self.cur.position().try_into().unwrap())
    }

    pub fn set_position(&mut self, offset: CodeAddress) {
        assert!(offset.0 as u64 <= self.cur.get_ref().len() as u64);
        self.cur.set_position(offset.0 as u64);
    }
}
