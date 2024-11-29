use std::{io, io::Read as _, path::PathBuf, sync::Arc};

use shin_core::{
    format::rom::{RomFileReader, RomReader},
    primitives::stateless_reader::StatelessFile,
};

#[derive(Clone)]
enum AssetDataAccessorInner {
    File(PathBuf),
    RomFile(RomFileReader<StatelessFile, Arc<RomReader<StatelessFile>>>),
}

#[derive(Clone)]
pub struct AssetDataAccessor {
    inner: AssetDataAccessorInner,
}

impl AssetDataAccessor {
    pub(super) fn from_file(path: PathBuf) -> AssetDataAccessor {
        AssetDataAccessor {
            inner: AssetDataAccessorInner::File(path),
        }
    }

    pub(super) fn from_rom_file(
        reader: RomFileReader<StatelessFile, Arc<RomReader<StatelessFile>>>,
    ) -> AssetDataAccessor {
        AssetDataAccessor {
            inner: AssetDataAccessorInner::RomFile(reader),
        }
    }
}

impl AssetDataAccessor {
    pub fn cursor(&self) -> AssetDataCursor {
        match &self.inner {
            AssetDataAccessorInner::File(file) => {
                let file = std::fs::File::open(file).expect("Opening file");
                AssetDataCursor {
                    inner: AssetDataCursorInner::File(file),
                }
            }
            AssetDataAccessorInner::RomFile(reader) => {
                let reader = reader.clone();
                AssetDataCursor {
                    inner: AssetDataCursorInner::RomFile(reader),
                }
            }
        }
    }

    pub async fn read_all(&self) -> Vec<u8> {
        match &self.inner {
            AssetDataAccessorInner::File(path) => std::fs::read(path).expect("Reading file"),
            AssetDataAccessorInner::RomFile(reader) => {
                let mut result = Vec::new();
                reader
                    .clone()
                    .read_to_end(&mut result)
                    .expect("Reading rom file");
                result
            }
        }
    }
}

enum AssetDataCursorInner {
    File(std::fs::File),
    RomFile(RomFileReader<StatelessFile, Arc<RomReader<StatelessFile>>>),
}
pub struct AssetDataCursor {
    inner: AssetDataCursorInner,
}

impl io::Read for AssetDataCursor {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.inner {
            AssetDataCursorInner::File(file) => file.read(buf),
            AssetDataCursorInner::RomFile(reader) => reader.read(buf),
        }
    }
}

impl io::Seek for AssetDataCursor {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match &mut self.inner {
            AssetDataCursorInner::File(file) => file.seek(pos),
            AssetDataCursorInner::RomFile(reader) => reader.seek(pos),
        }
    }
}
