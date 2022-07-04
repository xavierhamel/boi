use crate::blocks;
use crate::buffer;
use crate::img;
use crate::tests::log;

pub struct Encoder<const CHANNELS: usize>;

impl<const CHANNELS: usize> Encoder<CHANNELS> {
    pub fn encode_with_logger(raw: &[u8], width: usize, height: usize) -> (log::Logger, Vec<u8>) {
        let image = img::Image::<CHANNELS>::new(raw, width, height);
        let header = img::Header::new(width, height, &image.palette);
        let mut offsets = [img::Pixel::zeros(); blocks::Offset::MAX];
        let mut repeating = 0;
        let mut previous_hash = 0;
        let mut previous_chunk = &[0; CHANNELS][..];
        let mut logger = log::Logger::new();
        let mut buffer = buffer::Buffer::from(header);

        for current in raw.chunks_exact(CHANNELS) {
            let pixel = img::Pixel::<CHANNELS>::compute_forward(previous_chunk, current);
            let hashed = pixel.hash();

            if offsets[previous_hash] == pixel {
                if repeating < blocks::Repeating::MAX {
                    repeating += 1;
                } else {
                    buffer.push(blocks::Repeating::encode(repeating));
                    logger.repeating += 1;
                    repeating = 1;
                }
            } else {
                if repeating > 0 {
                    buffer.push(blocks::Repeating::encode(repeating));
                    logger.repeating += 1;
                    repeating = 0;
                }
                if blocks::Gray::is_gray(&pixel) {
                    buffer.push(blocks::Gray::encode(&pixel));
                    logger.gray += 1;
                } else if let Some(color) = image.palette.get(&pixel) {
                    buffer.push(blocks::Color::encode(color));
                    logger.palette += 1;
                } else if offsets[hashed] == pixel {
                    buffer.push(blocks::Offset::encode(hashed));
                    logger.offset += 1;
                } else {
                    let block = blocks::Pixel::encode(&pixel);
                    if block.bit_count == blocks::Typ::<CHANNELS>::Short.size() {
                        logger.short += 1;
                    } else if block.bit_count == blocks::Typ::<CHANNELS>::Medium.size() {
                        logger.medium += 1;
                    } else if block.bit_count == blocks::Typ::<CHANNELS>::Long.size() {
                        logger.long += 1;
                    }
                    buffer.push(block);
                }
            }
            offsets[hashed] = pixel;
            previous_hash = hashed;
            previous_chunk = current;
        }
        if repeating > 0 {
            buffer.push(blocks::Repeating::encode(repeating));
            logger.repeating += 1;
        }
        (logger, buffer.bytes)
    }

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
