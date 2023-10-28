//! Support for SNR file format, storing the game scenario.
//!
//! See also [crate::vm] for the VM that runs the scenario.

pub mod info;
pub mod instruction_elements;
pub mod instructions;
pub mod types;

use std::io::Cursor;

use anyhow::{bail, Result};
use binrw::{BinRead, BinWrite};
use bytes::Bytes;
use instruction_elements::CodeAddress;

use crate::format::scenario::{info::ScenarioInfoTables, instructions::Instruction};

#[derive(Debug, Copy, Clone, BinRead, BinWrite)]
#[brw(little, magic = b"SNR ")]
#[allow(dead_code)] // this stuff is declarative
pub struct ScenarioHeader {
    pub size: u32,
    /// Basically `max(msg_id)` amongst all MSGSETs. This counts all the dialogue lines in the scenario, ignoring the ones that do not get a message ID allocated.
    pub dialogue_line_count: u32,
    /// Meaning unknown.
    ///
    /// Known values (kudos to @Neurochitin):
    /// - umineko -> 6
    /// - higurashi -> 63
    /// - kaleido -> 1
    /// - konosuba -> 127
    /// - sugar style -> 2
    /// - D.C.4 -> 24
    pub unk2: u32,
    /// Meaning unknown.
    ///
    /// Known values (kudos to @Neurochitin):
    /// - umineko -> 19
    /// - higurashi -> 129
    /// - kaleido -> 1
    /// - konosuba -> 408
    /// - sugar style -> 3
    /// - D.C.4 -> 62
    pub unk3: u32,
    /// Seems to always be zero. Padding?
    pub unk4_zero: u32,
    /// Seems to always be zero. Padding?
    pub unk5_zero: u32,
    /// Seems to always be zero. Padding?
    pub unk6_zero: u32,
    pub code_offset: u32,
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
        let info_tables = ScenarioInfoTables::read(&mut cur)?;

        if header.size as usize != data.len() {
            bail!("SNR file size mismatch");
        }

        Ok(Self {
            info_tables,
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
