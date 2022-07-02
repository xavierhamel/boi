use crate::blocks;
use crate::buffer;
use crate::img;

pub struct Encoder<const CHANNELS: usize>;

impl<const CHANNELS: usize> Encoder<CHANNELS> {
    pub fn encode(raw: &[u8], width: usize, height: usize) -> Vec<u8> {
        let image = img::Image::new(raw, width, height);
        let header = img::Header::new(width, height, &image.palette);
        let mut offsets = [img::Pixel::zeros(); blocks::Offset::MAX];
        let mut repeating = 0;
        let mut previous_hash = 0;
        let mut previous_chunk = &[0; CHANNELS][..];
        let mut buffer = buffer::Buffer::from(header);

        for current in raw.chunks_exact(CHANNELS) {
            let pixel = img::Pixel::<CHANNELS>::compute_forward(previous_chunk, current);
            let hashed = pixel.hash();

            if offsets[previous_hash] == pixel {
                if repeating < blocks::Repeating::MAX {
                    repeating += 1;
                } else {
                    buffer.push(blocks::Repeating::encode(repeating));
                    repeating = 1;
                }
            } else {
                if repeating > 0 {
                    buffer.push(blocks::Repeating::encode(repeating));
                    repeating = 0;
                }
                if let Some(color) = image.palette.get(&pixel) {
                    buffer.push(blocks::Color::encode(color));
                } else if offsets[hashed] == pixel {
                    buffer.push(blocks::Offset::encode(hashed));
                } else {
                    buffer.push(blocks::Pixel::encode(&pixel));
                }
            }
            offsets[hashed] = pixel;
            previous_hash = hashed;
            previous_chunk = current;
        }
        if repeating > 0 {
            buffer.push(blocks::Repeating::encode(repeating));
        }
        buffer.bytes
    }
}
