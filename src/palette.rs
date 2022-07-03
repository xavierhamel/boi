use crate::blocks;
use crate::img;
use std::collections::HashMap;

/// A palette containing (almost) all the colors of an image. The palette can be created from a
/// random(ish) sample. `FullPalette` is used to create the palette with only the most common
/// colors.
pub struct ImagePalette<const CHANNELS: usize>(pub HashMap<img::Pixel<CHANNELS>, usize>);

impl<const CHANNELS: usize> ImagePalette<CHANNELS> {
    pub const SAMPLE_FRENQUENCY: usize = 100;

    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Increment the count of a color by 1.
    #[inline]
    pub fn increment_color(&mut self, pixel: img::Pixel<CHANNELS>) {
        *(self.0.entry(pixel).or_insert(0)) += 1;
    }
}

impl<const CHANNELS: usize> From<&[u8]> for ImagePalette<CHANNELS> {
    fn from(raw: &[u8]) -> Self {
        let mut image_palette = Self::new();
        let step = Self::SAMPLE_FRENQUENCY / 2;
        for chunk in raw.chunks_exact(CHANNELS * 2).step_by(step) {
            let pixel =
                img::Pixel::<CHANNELS>::compute_forward(&chunk[0..CHANNELS], &chunk[CHANNELS..]);
            if !blocks::Gray::is_gray(&pixel) {
                image_palette.increment_color(pixel);
            }
        }
        image_palette
    }
}

#[derive(Debug)]
pub struct Palette<const CHANNELS: usize>(pub Vec<img::Pixel<CHANNELS>>);

impl<const CHANNELS: usize> Palette<CHANNELS> {
    #[inline]
    pub fn get(&self, pixel: &img::Pixel<CHANNELS>) -> Option<usize> {
        for (idx, pixel_in_palette) in self.0.iter().enumerate() {
            if pixel_in_palette == pixel {
                return Some(idx);
            }
        }
        None
    }
}

impl<const CHANNELS: usize> From<ImagePalette<CHANNELS>> for Palette<CHANNELS> {
    fn from(image_palette: ImagePalette<CHANNELS>) -> Self {
        let mut colors = image_palette.0.into_iter().collect::<Vec<_>>();
        colors.sort_unstable_by_key(|color| usize::MAX - color.1);
        colors.truncate(blocks::Color::MAX);
        let palette = colors.into_iter().map(|color| color.0).collect::<Vec<_>>();
        Self(palette)
    }
}
