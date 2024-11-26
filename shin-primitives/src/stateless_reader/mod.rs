use std::fs::File;

pub use self::io_wrapper::StatelessIoWrapper;
use self::sys::StatelessFileImpl;

mod io_wrapper;
mod sys;

pub trait StatelessReader {
    fn size(&self) -> u64;
    fn read_at_exact(&self, offset: u64, buf: &mut [u8]) -> std::io::Result<()>;
}

impl<T> StatelessReader for &T
where
    T: StatelessReader,
{
    fn size(&self) -> u64 {
        T::size(*self)
    }

    fn read_at_exact(&self, offset: u64, buf: &mut [u8]) -> std::io::Result<()> {
        T::read_at_exact(*self, offset, buf)
    }
}

/// Implements a stateless reading for a file
///
/// <div class="warning">Does not function correctly if the file is modified after being open</div>
#[derive(Debug)]
pub struct StatelessFile {
    impl_: StatelessFileImpl,
    size: u64,
}

impl StatelessFile {
    pub fn new(file: File) -> std::io::Result<Self> {
        let size = file.metadata()?.len();

        Ok(Self {
            impl_: StatelessFileImpl::new(file),
            size,
        })
    }
}

impl StatelessReader for StatelessFile {
    fn size(&self) -> u64 {
        self.size
    }

    fn read_at_exact(&self, mut offset: u64, mut buf: &mut [u8]) -> std::io::Result<()> {
        while !buf.is_empty() {
            let read = self.impl_.read_at(offset, buf)?;
            if read == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "failed to read whole buffer",
                ));
            }
            offset += read as u64;
            buf = &mut buf[read..];
        }

        Ok(())
    }
}

pub struct StatelessCursor<B> {
    inner: B,
}

impl<B> StatelessCursor<B> {
    pub fn new(inner: B) -> Self {
        Self { inner }
    }
}

impl<B: AsRef<[u8]>> StatelessReader for StatelessCursor<B> {
    fn size(&self) -> u64 {
        self.inner.as_ref().len() as u64
    }

    fn read_at_exact(&self, offset: u64, buf: &mut [u8]) -> std::io::Result<()> {
        let buffer = self.inner.as_ref();
        if offset + buf.len() as u64 > buffer.len() as u64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "read out of bounds of StatelessCursor",
            ));
        }
        buf.copy_from_slice(&buffer[offset as usize..offset as usize + buf.len()]);

        Ok(())
    }
}
