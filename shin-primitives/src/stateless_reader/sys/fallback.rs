use std::{
    fs::File,
    io::{Read, Seek as _},
};

use parking_lot::Mutex;

#[derive(Debug)]
pub struct StatelessFileImpl {
    inner: Mutex<File>,
}

impl StatelessFileImpl {
    pub fn new(file: File) -> Self {
        Self {
            inner: Mutex::new(file),
        }
    }

    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut inner = self.inner.lock();
        inner.seek(std::io::SeekFrom::Start(offset))?;
        inner.read(buf)
    }
}
