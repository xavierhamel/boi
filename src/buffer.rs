use crate::blocks;
use crate::img;
use crate::U8_BITS;

/// A buffer that handle that without proper alignment. It's just a lots of bit manipulation.
pub struct Buffer {
    pub bytes: Vec<u8>,
    offset: usize,
}

impl Buffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity),
            offset: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Push an unaligned data payload to an unaligned buffer.
    pub fn push(&mut self, byte: blocks::Block) {
        let (bytes, count, new_offset) = byte.into_bytes(self.offset);
        let bytes = &bytes[..count];
        let bytes_count = self.bytes.len();
        if self.offset == 0 {
            self.bytes.push(bytes[0]);
        } else {
            self.bytes[bytes_count - 1] |= bytes[0];
        }
        for byte in bytes[1..].iter() {
            self.bytes.push(*byte);
        }
        self.offset = new_offset;
    }
}

impl<const CHANNELS: usize> From<img::Header<CHANNELS>> for Buffer {
    fn from(header: img::Header<CHANNELS>) -> Self {
        let mut buffer = Buffer::new((header.width * header.height) as usize);
        let alpha_code = if CHANNELS == 3 { 0 } else { 1 };
        buffer.push(blocks::Block::new(1, alpha_code));
        buffer.push(blocks::Block::new(
            u32::BITS as usize,
            header.width as usize,
        ));
        buffer.push(blocks::Block::new(
            u32::BITS as usize,
            header.height as usize,
        ));
        header
            .encoded_palette
            .into_iter()
            .for_each(|block| buffer.push(block));
        buffer
    }
}

pub struct BufferDecoder {
    pub bytes: Vec<u8>,
    offset: usize,
    index: usize,
}

impl BufferDecoder {
    pub fn next_block<const CHANNELS: usize>(&mut self) -> Option<(usize, usize)> {
        let code = self.next_code()?;
        let value = self.next_n_bits(blocks::Block::block_len::<CHANNELS>(code))?;
        Some((code, value))
    }

    fn next_code(&mut self) -> Option<usize> {
        let start = self.next_n_bits(1)?;
        if start == 0 {
            self.next_n_bits(1)
        } else {
            Some(0b100 | self.next_n_bits(2)?)
        }
    }

    pub fn next_n_bits(&mut self, n: usize) -> Option<usize> {
        let len = n + self.offset;
        let mut bytes_count = len / U8_BITS;
        let new_offset = len % U8_BITS;
        if new_offset > 0 {
            bytes_count += 1;
        }
        if self.index + bytes_count > self.bytes.len() {
            return None;
        }
        let mut out: usize = 0;
        let bytes = &self.bytes[self.index..self.index + bytes_count];
        for byte in bytes.iter() {
            out = (out << U8_BITS) | *byte as usize;
        }
        out = out >> (bytes_count * U8_BITS - len);
        out &= 2usize.pow(n as u32) - 1;
        self.index += bytes_count - 1;
        self.offset = new_offset;
        if self.offset == 0 {
            self.index += 1;
        }
        Some(out)
    }
}

impl From<Vec<u8>> for BufferDecoder {
    fn from(raw: Vec<u8>) -> Self {
        Self {
            bytes: raw,
            offset: 0,
            index: 0,
        }
    }
}

/*#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_buffer_encoder() {
        let mut buffer = Buffer::new(0);
        buffer.push(blocks::Block::new(5, 0b10001));
        buffer.push(blocks::Block::new(5, 0b10001));
        buffer.push(blocks::Block::new(5, 0b10001));
        assert_eq!(buffer.bytes[0], 0b10001100);
        assert_eq!(buffer.bytes[1], 0b01100010);
    }

    #[test]
    fn test_buffer_decoder() {
        let mut buffer = BufferDecoder::from(vec![0b10100001, 0b01101101, 0b101000]);
        assert_eq!(0b1, buffer.next_n_bits(1));
        assert_eq!(0b01000010, buffer.next_n_bits(8));
        assert_eq!(0b1, buffer.next_n_bits(1));
        assert_eq!(0b10, buffer.next_n_bits(2));
    }
}*/
