#[derive(Debug)]
pub struct RingBuffer<const G: usize> {
    buffer: Box<[u8; G]>,
    write_cursor: usize,
    read_cursor: usize,
}

impl<const G: usize> std::io::Write for RingBuffer<G> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut written = 0;
        for byte in buf {
            if self.is_full() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "Buffer full",
                ))?;
            }
            self.buffer[self.write_cursor] = *byte;
            self.write_cursor = (self.write_cursor + 1) % G;
            written += 1;
        }
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<const G: usize> std::io::Read for RingBuffer<G> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut read = 0;
        while read < buf.len() {
            if self.read_cursor == self.write_cursor {
                break;
            }
            buf[read] = self.buffer[self.read_cursor];
            self.read_cursor = (self.read_cursor + 1) % G;
            read += 1;
        }
        Ok(read)
    }
}

impl<const G: usize> RingBuffer<G> {
    pub fn new() -> Self {
        Self {
            buffer: Box::new([0; G]),
            write_cursor: 0,
            read_cursor: 0,
        }
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        (self.write_cursor + 1) % G == self.read_cursor
    }
}
