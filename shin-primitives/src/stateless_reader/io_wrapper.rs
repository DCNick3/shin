use std::io::SeekFrom;

use crate::stateless_reader::StatelessReader;

pub struct StatelessIoWrapper<S> {
    inner: S,
    position: u64,
}

impl<S> StatelessIoWrapper<S> {
    pub fn new(inner: S) -> Self {
        Self { inner, position: 0 }
    }
}

impl<S: StatelessReader> std::io::Read for StatelessIoWrapper<S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remaining = self.inner.size() - self.position;
        let target_len = (buf.len() as u64).min(remaining) as usize;

        self.inner
            .read_at_exact(self.position, &mut buf[..target_len])?;
        self.position += target_len as u64;

        Ok(target_len)
    }
}

impl<S: StatelessReader> std::io::Seek for StatelessIoWrapper<S> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let size = self.inner.size();

        let new_position = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(end) => {
                let end = size as i64 + end;
                if end < 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "invalid seek to a negative position",
                    ));
                }
                if end > size as i64 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "invalid seek beyond the end of the file",
                    ));
                }

                end as u64
            }
            SeekFrom::Current(delta) => {
                let new_position = self.position as i64 + delta;
                if new_position < 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "invalid seek to a negative position",
                    ));
                }
                if new_position > size as i64 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "invalid seek beyond the end of the file",
                    ));
                }
                new_position as u64
            }
        };

        self.position = new_position;

        Ok(new_position)
    }
}
