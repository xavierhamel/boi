use crate::img;
use crate::{U8_BITS, USIZE_BITS};
use macros::BoiTyp;

#[derive(BoiTyp)]
pub enum Typ<const CHANNELS: usize> {
    #[boi(code = 0b110, code_len = 3, size = 4, uses_channels = true)]
    Short,

    //#[boi(code = 0b100, code_len = 3, size = 6, uses_channels = true)]
    #[boi(code = 0b1110, code_len = 4, size = 6, uses_channels = true)]
    Medium,

    #[boi(code = 0b1111, code_len = 4, size = 9, uses_channels = true)]
    //#[boi(code = 0b101, code_len = 3, size = 9, uses_channels = true)]
    Long,

    #[boi(code = 0b100, code_len = 3, size = 6)]
    Repeating,

    #[boi(code = 0b01, code_len = 2, size = 4)]
    Color,

    #[boi(code = 0b101, code_len = 3, size = 2)]
    Gray,

    #[boi(code = 0b00, code_len = 2, size = 8)]
    Offset,
}

#[derive(Debug)]
pub struct Block {
    /// Value of the byte
    pub value: usize,
    /// The number of bits to represent the value
    pub bit_count: usize,
}

impl Block {
    #[inline]
    pub fn new_with_code(
        bit_count: usize,
        mut value: usize,
        code_len: usize,
        mut code: usize,
    ) -> Self {
        code <<= bit_count;
        value &= 2usize.pow(bit_count as u32) - 1;
        value |= code;
        Self {
            value,
            bit_count: bit_count + code_len,
        }
    }

    #[inline]
    pub fn new(bit_count: usize, value: usize) -> Self {
        Self { bit_count, value }
    }

    /// Turn the block into bytes with proper alignment based on the offset of the buffer. It also
    /// returns the new buffer offset. Return `(bytes, bytes_count, new_offset)`.
    #[inline]
    pub fn into_bytes(self, buffer_offset: usize) -> ([u8; 8], usize, usize) {
        let total_offset = self.bit_count + buffer_offset;
        let new_offset = total_offset % U8_BITS;
        let mut bytes_count = total_offset / U8_BITS;
        if new_offset > 0 {
            bytes_count += 1;
        }
        (
            (self.value << (USIZE_BITS - total_offset)).to_be_bytes(),
            bytes_count,
            new_offset,
        )
    }

    #[inline]
    pub fn block_len<const CHANNELS: usize>(code: usize) -> usize {
        if code == 15 {
            Repeating::BITS_COUNT
        } else if code == 14 {
            Pixel::<CHANNELS>::LONG_BITS as usize * CHANNELS
        } else {
            [
                Pixel::<CHANNELS>::SHORT_BITS * CHANNELS,           // 0b0000
                Offset::BITS_COUNT,                                 // 0b0001
                0,                                                  // 0b0010
                0,                                                  // 0b0011
                Pixel::<CHANNELS>::MEDIUM_BITS as usize * CHANNELS, // 0b0101
                Gray::<CHANNELS>::BITS_COUNT,                       // 0b0110
                Color::BITS_COUNT,                                  // 0b0111
            ][code]
        }
    }
}

/// A count of repeating pixels with the exact same value (all the channels). The value of the
/// repeating pixels are the same as the previous pixel. The value cannot be 0 (not repeating).
/// To use the value, add or remove 1. A stored value of 0 means 1 pixel are repeating. This is
/// done to store more data in less space.
pub struct Repeating;

impl Repeating {
    const BITS_COUNT: usize = Typ::<0>::REPEATING_SIZE;
    const CODE_LEN: usize = Typ::<0>::REPEATING_CODE_LEN;
    const CODE: usize = Typ::<0>::REPEATING_CODE;
    pub const MAX: usize = 2usize.pow(Self::BITS_COUNT as u32);

    // bits : 8

    #[inline]
    pub fn encode(value: usize) -> Block {
        Block::new_with_code(Self::BITS_COUNT, value - 1, Self::CODE_LEN, Self::CODE)
    }
}

pub struct Gray<const CHANNELS: usize>;

impl<const CHANNELS: usize> Gray<CHANNELS> {
    const BITS_COUNT: usize = Typ::<0>::GRAY_SIZE;
    const CODE_LEN: usize = Typ::<0>::GRAY_CODE_LEN;
    const CODE: usize = Typ::<0>::GRAY_CODE;
    // bits : 4

    const MIN: i16 = -2i16.pow(Self::BITS_COUNT as u32) / 2;
    const MAX: i16 = (2i16.pow(Self::BITS_COUNT as u32) / 2) - 1;

    #[inline]
    pub fn encode(pixel: &img::Pixel<CHANNELS>) -> Block {
        Block::new_with_code(
            Self::BITS_COUNT,
            pixel.0[0] as usize,
            Self::CODE_LEN,
            Self::CODE,
        )
    }

    #[inline]
    pub fn is_gray(pixel: &img::Pixel<CHANNELS>) -> bool {
        if pixel.0[0] == pixel.0[1] && pixel.0[1] == pixel.0[2] {
            pixel.0[0] <= Self::MAX && pixel.0[1] >= Self::MIN
        } else {
            false
        }
    }
}

/// A negative offset to a preivous pixel with the same value as the current one (all the
/// channels). Because an offset of 0 is not possible (cannot reference itself), all values are
/// shifted by 1. Therefore a value of 0 actually means an offset of 1. This is done to store more
/// data in less space.
pub struct Offset;

impl Offset {
    const BITS_COUNT: usize = Typ::<0>::OFFSET_SIZE;
    const CODE_LEN: usize = Typ::<0>::OFFSET_CODE_LEN;
    const CODE: usize = Typ::<0>::OFFSET_CODE;

    // const BITS_COUNT: usize = 8;
    pub const MASK: usize = 0b111111;
    pub const MAX: usize = 2usize.pow(Self::BITS_COUNT as u32);

    #[inline]
    pub fn encode(value: usize) -> Block {
        Block::new_with_code(Self::BITS_COUNT, value, Self::CODE_LEN, Self::CODE)
    }
}

pub struct Pixel<const CHANNELS: usize>;

impl<const CHANNELS: usize> Pixel<CHANNELS> {
    const SHORT_BITS: usize = Typ::<CHANNELS>::SHORT_SIZE;
    const SHORT_CODE_LEN: usize = Typ::<CHANNELS>::SHORT_CODE_LEN;
    const SHORT_CODE: usize = Typ::<CHANNELS>::SHORT_CODE;

    const MEDIUM_BITS: usize = Typ::<CHANNELS>::MEDIUM_SIZE;
    const MEDIUM_CODE_LEN: usize = Typ::<CHANNELS>::MEDIUM_CODE_LEN;
    const MEDIUM_CODE: usize = Typ::<CHANNELS>::MEDIUM_CODE;

    const LONG_BITS: usize = Typ::<CHANNELS>::LONG_SIZE;
    const LONG_CODE_LEN: usize = Typ::<CHANNELS>::LONG_CODE_LEN;
    const LONG_CODE: usize = Typ::<CHANNELS>::LONG_CODE;

    const SHORT_MIN: i16 = -2i16.pow(Self::SHORT_BITS as u32) / 2;
    const SHORT_MAX: i16 = (2i16.pow(Self::SHORT_BITS as u32) / 2) - 1;
    const MEDIUM_MIN: i16 = -2i16.pow(Self::MEDIUM_BITS as u32) / 2;
    const MEDIUM_MAX: i16 = (2i16.pow(Self::MEDIUM_BITS as u32) / 2) - 1;

    #[inline]
    pub fn encode(pixel: &img::Pixel<CHANNELS>) -> Block {
        let min = *pixel.0.iter().min().unwrap();
        let max = *pixel.0.iter().max().unwrap();
        let channel = 3;
        let (channel_size, code, code_len) = if min >= Self::SHORT_MIN && max <= Self::SHORT_MAX {
            (Self::SHORT_BITS, Self::SHORT_CODE, Self::SHORT_CODE_LEN)
        } else if min >= Self::MEDIUM_MIN && max <= Self::MEDIUM_MAX {
            (Self::MEDIUM_BITS, Self::MEDIUM_CODE, Self::MEDIUM_CODE_LEN)
        } else {
            (Self::LONG_BITS, Self::LONG_CODE, Self::LONG_CODE_LEN)
        };
        let value = Self::encode_channels(pixel, channel_size);
        Block::new_with_code(channel_size * channel, value, code_len, code)
    }

    #[inline]
    fn encode_channels(pixel: &img::Pixel<CHANNELS>, channel_size: usize) -> usize {
        let mask = 2usize.pow(channel_size as u32) - 1;
        let channel = 3; // TODO: Check for the alpha channel.
        (0..channel).fold(0, |mut value, index| {
            value = (value as usize) << channel_size;
            value | ((pixel.0[index] as usize) & mask)
        })
    }

    #[inline]
    pub fn decode(value: usize, code: usize) -> img::Pixel<CHANNELS> {
        let channel_size = match code {
            Self::SHORT_CODE => Self::SHORT_BITS,
            Self::MEDIUM_CODE => Self::MEDIUM_BITS,
            Self::LONG_CODE => Self::LONG_BITS,
            _ => panic!("Code: {}", code),
        };
        let mut pixel = [0; CHANNELS];
        let mask = 2usize.pow(channel_size as u32) - 1;
        for idx in 0..CHANNELS {
            pixel[idx] = Self::extend_sign(
                (value >> (CHANNELS - idx - 1) * channel_size) & mask,
                channel_size,
            ) as i16;
        }
        img::Pixel(pixel)
    }

    /// Resize a binary complement's 2 number from a size to another size (the size is the number of
    /// bit that represent the number).
    /// 4 bits number to 6 bits would look like this:
    /// 0b1000 -> 0b111000
    /// 0b0100 -> 0b000100
    pub fn extend_sign(raw: usize, src_size: usize) -> usize {
        (raw as isize)
            .wrapping_shl(usize::BITS - src_size as u32)
            .wrapping_shr(usize::BITS - src_size as u32) as usize
    }
}

/// Represent a color of a palette. The `usize` is the index of the color in the palette of the
/// image. See [`Palette`].
pub struct Color;

impl Color {
    const BITS_COUNT: usize = Typ::<0>::COLOR_SIZE;
    const CODE_LEN: usize = Typ::<0>::COLOR_CODE_LEN;
    const CODE: usize = Typ::<0>::COLOR_CODE;

    pub const MAX: usize = 2usize.pow(Self::BITS_COUNT as u32);

    #[inline]
    pub fn encode(value: usize) -> Block {
        Block::new_with_code(Self::BITS_COUNT, value, Self::CODE_LEN, Self::CODE)
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_to_bytes() {
        let offset = 5;
        let block = Block::new(8, 0b11110001);
        let (bytes, _, _) = block.into_bytes(offset);
        assert_eq!(bytes[0], 7);
        assert_eq!(bytes[1], 136);
    }

    #[test]
    fn encode_repeating() {
        let block = Repeating::encode(8);
        assert_eq!(0b100111, block.value);
        let (bytes, count, _) = block.into_bytes(0);
        assert_eq!(0b10011100, bytes[0]);
        assert_eq!(1, count);
    }

    #[test]
    fn encode_pixel() {
        const CHANNELS: usize = 3;
        let block = Pixel::encode(&img::Pixel::<CHANNELS>([0; CHANNELS]));
        assert_eq!(0b101000000000000, block.value);
        let block = Pixel::encode(&img::Pixel::<CHANNELS>([15; CHANNELS]));
        assert_eq!(0b00001111001111001111, block.value);
    }

    #[test]
    fn encode_color() {}

    #[test]
    fn encode_offset() {}
}*/
