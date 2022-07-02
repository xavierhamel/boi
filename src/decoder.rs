use crate::blocks;
use crate::buffer;
use crate::img;

pub fn decode(raw: Vec<u8>) -> (Vec<u8>, u32, u32) {
    let is_alpha = (raw[0] & 0b10000000) == 1;
    if is_alpha {
        Decoder::<4>::decode(raw)
    } else {
        Decoder::<3>::decode(raw)
    }
}

pub struct Decoder<const CHANNELS: usize>;

impl<const CHANNELS: usize> Decoder<CHANNELS> {
    pub fn decode(raw: Vec<u8>) -> (Vec<u8>, u32, u32) {
        let mut buffer = buffer::BufferDecoder::from(raw);
        let header = img::Header::<CHANNELS>::from(&mut buffer);
        let mut pixels: Vec<img::Pixel<CHANNELS>> =
            Vec::with_capacity((header.width * header.height) as usize);
        let mut offsets = [img::Pixel::<CHANNELS>::zeros(); blocks::Offset::MAX];
        let mut previous = img::Pixel::<CHANNELS>::zeros();

        // -1 -2
        // -1 -(-2) = -1 + 2 = 1
        // -1 - 1
        while let Some((code, encoded_value)) = buffer.next_block::<CHANNELS>() {
            match code {
                0b00 | 0b101 | 0b110 => {
                    let pixel = blocks::Pixel::<CHANNELS>::decode(encoded_value, code);
                    offsets[pixel.hash()] = pixel;
                    pixels.push(pixel);
                    previous = pixel;
                }
                0b01 => {
                    let pixel = header.palette[encoded_value];
                    offsets[pixel.hash()] = pixel;
                    pixels.push(pixel);
                    previous = pixel;
                }
                0b100 => {
                    for _ in 0..(encoded_value + 1) {
                        pixels.push(previous);
                    }
                }
                0b111 => {
                    let pixel = offsets[encoded_value];
                    pixels.push(pixel);
                    previous = pixel;
                }
                _ => panic!("The code does not exists"),
            }
        }
        let mut bytes = Vec::with_capacity((header.width * header.height) as usize * CHANNELS);
        let mut prev = vec![0; CHANNELS];
        for pixel in pixels.into_iter() {
            let mut current = img::Pixel::<CHANNELS>::compute_backward(&prev, &pixel);
            prev = current.clone();
            bytes.append(&mut current);
        }
        assert_eq!(
            (header.width * header.height) as usize * CHANNELS,
            bytes.len(),
        );
        (bytes, header.width, header.height)
    }
}
