use std::{fs::File, os::windows::fs::FileExt};

#[derive(Debug)]
pub struct StatelessFileImpl {
    inner: File,
}

impl StatelessFileImpl {
    pub fn new(file: File) -> Self {
        Self { inner: file }
    }

    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.seek_read(buf, offset)
    }
}
