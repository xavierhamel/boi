#![allow(dead_code)]
mod blocks;
mod buffer;
mod decoder;
mod encoder;
mod img;
mod palette;
mod squares;
mod tests;

pub const U8_BITS: usize = u8::BITS as usize;
pub const USIZE_BITS: usize = usize::BITS as usize;

#[cfg(test)]
mod test {
    use super::*;

    fn open_image(path: String) -> (Vec<u8>, usize, usize, bool) {
        let decoder = png::Decoder::new(std::fs::File::open(path.clone()).unwrap());
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let next_frame = reader.next_frame(&mut buf).unwrap();
        let bytes = buf[..next_frame.buffer_size()].to_vec();
        let info = reader.info();
        let is_alpha = info.color_type == png::ColorType::Rgba;
        (bytes, info.width as usize, info.height as usize, is_alpha)
    }

    fn save_image(path: &str, width: u32, height: u32, bytes: &[u8]) {
        let path = std::path::Path::new(path);
        let file = std::fs::File::create(path).unwrap();
        let ref mut w = std::io::BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, width, height);
        match true {
            true => encoder.set_color(png::ColorType::Rgba),
            false => encoder.set_color(png::ColorType::Rgb),
        }
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Default);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(bytes).unwrap();
    }

    #[test]
    fn test_encoder() {
        let mut tests = tests::TestImages::new();
        for dir in [
            "./img/wallpaper",
            "./img/kodak",
            "./img/screenshots",
            "./img/textures",
        ] {
            let paths = std::fs::read_dir(dir).unwrap();
            for path in paths {
                let path = path.unwrap().path().display().to_string();
                let test = tests::TestImage::test(path);
                println!("{}", test);
                tests.add(test);
            }
        }
        tests.agregator.average();
        println!("{}", tests);
        assert!(true);
    }

    //#[test]
    fn test_encoder_single() {
        let path = "./img/screenshots/en.wikipedia.org.png".to_string();
        //let path = "./img/wallpaper/1492858.png".to_string();
        let (bytes, width, height, is_alpha) = open_image(path);
        let (_, encoded) = if !is_alpha {
            encoder::Encoder::<3>::encode_with_logger(&bytes, width, height)
        } else {
            encoder::Encoder::<4>::encode_with_logger(&bytes, width, height)
        };
        save_image("./img/out.png", width as u32, height as u32, &encoded);
    }

    //#[test]
    fn test_decoder() {
        let path = "./img/wallpaper/1492858.png".to_string();
        let (bytes, width, height, is_alpha) = open_image(path);
        let encoded = if !is_alpha {
            encoder::Encoder::<3>::encode(&bytes, width, height)
        } else {
            encoder::Encoder::<4>::encode(&bytes, width, height)
        };
        let (boi_bytes, width, height) = decoder::decode(encoded);
        save_image("./img/out.png", width, height, &boi_bytes);
    }
}
