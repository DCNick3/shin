use anyhow::Result;
use binrw::{BinRead, BinWrite};

#[derive(BinRead, BinWrite)]
#[br(little, magic = b"")]
struct ScenarioHeader {

}

pub struct ScenarioReader {

}

impl ScenarioReader {
    pub fn new(data: Vec<u8>) -> Result<Self> {
        todo!()
    }
}