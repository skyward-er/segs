

#[derive(Debug)]
pub struct RingBuffer<const G: usize> {
    buffer: Box<[u8; G]>,
    write_cursor: usize,
    read_cursor: usize,
}

impl<const G: usize> RingBuffer<G> {
    pub fn new() -> Self {
        Self {
            buffer: Box::new([0; G]),
            write_cursor: 0,
            read_cursor: 0,
        }
    }

    pub fn len(&self) -> usize {
        if self.write_cursor >= self.read_cursor {
            self.write_cursor - self.read_cursor
        } else {
            G - self.read_cursor + self.write_cursor
        }
    }

    #[inline]
    pub fn remaining_capacity(&self) -> usize {
        G - self.len() - 1
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        (self.write_cursor + 1) % G == self.read_cursor
    }
}

impl<const G: usize> std::io::Write for RingBuffer<G> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.remaining_capacity() < buf.len() {
            Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "Buffer full",
            ))?;
        }
        let mut written = 0;
        for byte in buf {
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

pub struct OverwritingRingBuffer<const G: usize> {
    buffer: Box<[u8; G]>,
    write_cursor: usize,
    read_cursor: usize,
    full: bool, // new flag to indicate if buffer is full
}

impl<const G: usize> OverwritingRingBuffer<G> {
    pub fn new() -> Self {
        Self {
            buffer: Box::new([0; G]),
            write_cursor: 0,
            read_cursor: 0,
            full: false, // initialize full flag to false
        }
    }

    // Updated len() using full flag
    pub fn len(&self) -> usize {
        if self.full {
            G
        } else if self.write_cursor >= self.read_cursor {
            self.write_cursor - self.read_cursor
        } else {
            G - self.read_cursor + self.write_cursor
        }
    }
}

impl<const G: usize> std::io::Write for OverwritingRingBuffer<G> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut written = 0;
        for &byte in buf {
            self.buffer[self.write_cursor] = byte;
            self.write_cursor = (self.write_cursor + 1) % G;
            if self.full {
                // Buffer full; advance read_cursor to overwrite oldest
                self.read_cursor = (self.read_cursor + 1) % G;
            }
            // Set full flag if write_cursor catches up to read_cursor
            self.full = self.write_cursor == self.read_cursor;
            written += 1;
        }
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<const G: usize> std::io::Read for OverwritingRingBuffer<G> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut read = 0;
        // If full, then buffer is not empty and we will clear full once reading starts.
        while read < buf.len() {
            // Buffer empty condition: not full and cursors equal
            if !self.full && (self.read_cursor == self.write_cursor) {
                break;
            }
            buf[read] = self.buffer[self.read_cursor];
            self.read_cursor = (self.read_cursor + 1) % G;
            self.full = false; // once reading, clear full flag
            read += 1;
        }
        Ok(read)
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    // Tests for RingBuffer
    #[test]
    fn test_ring_buffer_empty_read() {
        let mut rb = super::RingBuffer::<8>::new();
        let mut buf = [0; 4];
        let n = rb.read(&mut buf).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn test_ring_buffer_write_then_read() {
        let mut rb = super::RingBuffer::<8>::new();
        let data = [10, 20, 30, 40];
        let n = rb.write(&data).unwrap();
        assert_eq!(n, data.len());
        let mut buf = [0u8; 4];
        let n = rb.read(&mut buf).unwrap();
        assert_eq!(n, data.len());
        assert_eq!(buf, data);
    }

    #[test]
    fn test_ring_buffer_wraparound() {
        let mut rb = super::RingBuffer::<8>::new();
        // Write initial data.
        let data1 = [1, 2, 3, 4, 5, 6];
        let n = rb.write(&data1).unwrap();
        assert_eq!(n, data1.len());
        // Read a few bytes to free space.
        let mut buf1 = [0u8; 3];
        let n = rb.read(&mut buf1).unwrap();
        assert_eq!(n, 3);
        assert_eq!(buf1, [1, 2, 3]);
        // Write additional data to wrap-around.
        let data2 = [7, 8, 9];
        let n = rb.write(&data2).unwrap();
        assert_eq!(n, data2.len());
        // Read remaining data.
        let mut buf2 = [0u8; 6];
        let n = rb.read(&mut buf2).unwrap();
        // Expected order: remaining from data1 ([4,5,6]) then data2 ([7,8,9])
        assert_eq!(n, 6);
        assert_eq!(buf2, [4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_ring_buffer_full_error() {
        let mut rb = super::RingBuffer::<4>::new();
        // Usable capacity is G - 1: 3 bytes.
        let _ = rb.write(&[10, 20, 30]).unwrap();
        let res = rb.write(&[40]);
        assert!(res.is_err());
    }

    #[test]
    fn test_ring_buffer_partial_write_when_full() {
        let mut rb = super::RingBuffer::<6>::new();
        // Usable capacity: 5 bytes.
        let n = rb.write(&[1, 2, 3, 4]).unwrap();
        assert_eq!(n, 4);
        // Only one byte can be written before buffer is full.
        // The write should error on the second byte.
        let res = rb.write(&[5, 6]);
        assert!(res.is_err());

        // Read the first 5 bytes.
        let mut buf = [0u8; 5];
        let n = rb.read(&mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(buf, [1, 2, 3, 4, 0]);
    }

    // Tests for OverwritingRingBuffer
    #[test]
    fn test_overwriting_ring_buffer_empty_read() {
        let mut orb = super::OverwritingRingBuffer::<8>::new();
        let mut buf = [0; 4];
        let n = orb.read(&mut buf).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn test_overwriting_ring_buffer_write_then_read() {
        let mut orb = super::OverwritingRingBuffer::<8>::new();
        let data = [10, 20, 30, 40];
        let n = orb.write(&data).unwrap();
        assert_eq!(n, data.len());
        let mut buf = [0u8; 4];
        let n = orb.read(&mut buf).unwrap();
        assert_eq!(n, data.len());
        assert_eq!(buf, data);
    }

    #[test]
    fn test_overwriting_ring_buffer_overwrite() {
        let mut orb = super::OverwritingRingBuffer::<6>::new();
        // Write initial 5 elements (usable capacity = 5).
        let _ = orb.write(&[1, 2, 3, 4, 5]).unwrap();
        // Write two more elements to force overwrite.
        let _ = orb.write(&[6, 7]).unwrap();
        // After overwrite, expected order starts from index advanced by one overwrite
        // Expected stored order: [2,3,4,5,6,7]
        let mut buf = [0u8; 6];
        let n = orb.read(&mut buf).unwrap();
        assert_eq!(n, 6);
        assert_eq!(buf, [2, 3, 4, 5, 6, 7]);
    }
}
