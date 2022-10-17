use anyhow::Result;
use std::fs::File;
use crate::format::rom::{IndexEntry, RomReader};

mod format;

fn main() -> Result<()> {
    let path = "/home/dcnick3/trash/switch-games/Umineko When They Cry (JAP + ENG + RUS Mod.) [NSZ]/01006A300BA2C000/b3248e2805a10a33debfdb64c4ef4cd3.1/data.rom";
    let rom = File::open(path)?;

    let reader = RomReader::new(rom)?;

    for (name, entry) in reader.traverse() {
        let ty = match entry {
            IndexEntry::File(_) => "FILE",
            IndexEntry::Directory(_) => "DIR ",
        };
        println!("{} {}", ty, name);
    }

    Ok(())
}
