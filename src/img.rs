use crate::blocks;
use crate::buffer;
use crate::img;
use crate::palette;

/// An `Pixel` is computed from the previous and current pixel of the actual image. An
/// `Pixel` is actually the preivous minus pixel value minus the current pixel value.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Pixel<const CHANNELS: usize>(pub [i16; CHANNELS]);

impl<const CHANNELS: usize> Pixel<CHANNELS> {
    #[inline]
    pub fn zeros() -> Self {
        Self([0; CHANNELS])
    }

    /// Compute the value of a pixel that is going to be encoded.
    #[inline]
    pub fn compute_forward(previous: &[u8], current: &[u8]) -> Self {
        let mut pixel = [0; CHANNELS];
        for idx in 0..pixel.len() {
            pixel[idx] = previous[idx] as i16 - current[idx] as i16;
        }
        Self(pixel)
    }

    /// Compute the value of a pixel that is being decoded
    #[inline]
    pub fn compute_backward(previous: &[u8], current: &Self) -> Vec<u8> {
        let mut pixel = vec![0; CHANNELS];
        for idx in 0..pixel.len() {
            pixel[idx] = (previous[idx] as i16 - current.0[idx]) as u8;
        }
        pixel
    }

    /// Return a hash for the color of the pixel. This is used to compute the offset. This uses the
    /// exact same function as qoi uses for the hash algorithm. Some research maybe needed to find
    /// a better or faster one.
    ///
    /// SPIKE: A slower algorithm could be used for better compression depending on the options
    /// choosen.
    #[inline]
    pub fn hash(&self) -> usize {
        let mut hash = 0;
        let test = [3, 5, 7, 11];
        for idx in 0..self.0.len() {
            hash += self.0[idx] as usize * test[idx];
        }
        hash % blocks::Offset::MAX
    }

    /// Return a vec of the underlying array
    pub fn as_vec(self) -> Vec<i16> {
        self.0.to_vec()
    }
}

/// Here `Image` is the data needed by the encoder to correctly encode the image.
pub struct Image<const CHANNELS: usize> {
    width: usize,
    height: usize,
    pub palette: palette::Palette<CHANNELS>,
}

impl<const CHANNELS: usize> Image<CHANNELS> {
    pub fn new(raw: &[u8], width: usize, height: usize) -> Self {
        let image_palette = palette::ImagePalette::from(raw);
        let palette = palette::Palette::from(image_palette);
        Self {
            width,
            height,
            palette,
        }
    }
}

/// An image header containing informations about the image to be decoded.
#[derive(Debug)]
pub struct Header<const CHANNELS: usize> {
    /// With of the image
    pub width: u32,
    /// Height of the image
    pub height: u32,
    /// The color palette used in the image of the most present colors.
    pub encoded_palette: Vec<blocks::Block>,
    /// The color palette
    pub palette: Vec<img::Pixel<CHANNELS>>,
}

impl<const CHANNELS: usize> Header<CHANNELS> {
    pub fn new(width: usize, height: usize, palette: &palette::Palette<CHANNELS>) -> Self {
        let encoded_palette = palette
            .0
            .iter()
            .map(|color| blocks::Pixel::encode(color))
            .collect::<Vec<_>>();
        Self {
            width: width as u32,
            height: height as u32,
            encoded_palette,
            palette: Vec::new(),
        }
    }
}

impl<const CHANNELS: usize> From<&mut buffer::BufferDecoder> for Header<CHANNELS> {
    fn from(buffer: &mut buffer::BufferDecoder) -> Self {
        let _is_alpha = buffer.next_n_bits(1).unwrap() == 1;
        let width = buffer.next_n_bits(32).unwrap();
        let height = buffer.next_n_bits(32).unwrap();
        let palette = (0..blocks::Color::MAX)
            .map(|idx| {
                if let Some((code, encoded_value)) = buffer.next_block::<CHANNELS>() {
                    blocks::Pixel::<CHANNELS>::decode(encoded_value, code)
                } else {
                    unreachable!();
                }
            })
            .collect::<Vec<_>>();
        Self {
            width: width as u32,
            height: height as u32,
            encoded_palette: Vec::new(),
            palette,
        }
    }
}
