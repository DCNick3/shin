use crate::asset::Asset;
use shin_core::format::scenario::Scenario;

impl Asset for Scenario {
    fn load_from_bytes(data: Vec<u8>) -> anyhow::Result<Self> {
        Scenario::new(data.into())
    }
}
