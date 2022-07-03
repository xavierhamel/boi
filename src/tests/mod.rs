pub mod log;
use crate::encoder;
use colored::*;
use png;
use qoi;
use std::collections::HashMap;
use std::time::Instant;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Algo {
    Boi,
    Png,
    Qoi,
    Fas,
}

impl std::fmt::Display for Algo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Algo::Boi => write!(f, "BOI"),
            Algo::Fas => write!(f, "Fas"),
            Algo::Png => write!(f, "PNG"),
            Algo::Qoi => write!(f, "QOI"),
        }
    }
}

pub fn size_diff_to_string(algo: Algo, size_diff: f64) -> ColoredString {
    if algo == Algo::Png {
        return "---".white();
    }
    if size_diff >= 0.0 {
        format!("+{:.2}", size_diff).red()
    } else {
        format!("{:.2}", size_diff).green()
    }
}

pub fn time_diff_to_string(algo: Algo, time_diff: f64) -> ColoredString {
    if algo == Algo::Png {
        return "---".white();
    }
    if time_diff < 1.0 {
        format!("x{:.1}", time_diff).red()
    } else {
        format!("x{:.1}", time_diff).green()
    }
}

pub struct Test {
    algo: Algo,
    instant: Instant,
    time: usize,
    size: usize,
    ref_time: usize,
    ref_size: usize,
}

impl Test {
    pub fn start_with_ref(algo: Algo, ref_test: &Self) -> Self {
        match ref_test {
            Test {
                algo: ref_algo,
                time,
                size,
                ..
            } if ref_algo == &Algo::Png => Test {
                algo,
                instant: Instant::now(),
                time: 0,
                size: 0,
                ref_time: *time,
                ref_size: *size,
            },
            _ => Test::start(Algo::Boi),
        }
    }
    pub fn start(algo: Algo) -> Self {
        Self {
            algo,
            instant: Instant::now(),
            time: 0,
            size: 0,
            ref_time: 0,
            ref_size: 0,
        }
    }

    pub fn stop(&mut self, size: usize) {
        self.time = self.instant.elapsed().as_millis() as usize;
        self.size = size / 2usize.pow(10);
    }

    pub fn size_diff(&self) -> f64 {
        if self.ref_size != 0 {
            (self.size as f64 - self.ref_size as f64) / self.ref_size as f64 * 100.0
        } else {
            100.0
        }
    }

    pub fn time_diff(&self) -> f64 {
        if self.time != 0 {
            self.ref_time as f64 / self.time as f64
        } else {
            50.0
        }
    }
}

impl std::fmt::Display for Test {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // | BOI ||    000000 | -00.00 ||    000000 | -x00.0 |
        let sep = "|".to_string().yellow();
        let dsep = "||".to_string().yellow();
        write!(
            f,
            "{sep} {} {dsep} {: >9} {sep} {: >6} {dsep} {: >9} {sep} {: >6} {sep}",
            self.algo.to_string().cyan(),
            self.size,
            size_diff_to_string(self.algo, self.size_diff()),
            self.time,
            time_diff_to_string(self.algo, self.time_diff()),
        )
    }
}

pub struct TestImages {
    tests: Vec<TestImage>,
    pub agregator: log::Agregator,
}

impl TestImages {
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            agregator: log::Agregator::new(),
        }
    }

    pub fn add(&mut self, test: TestImage) {
        self.agregator.add(&test.logger);
        self.tests.push(test);
    }
}

impl std::fmt::Display for TestImages {
    // +-----++----------+----------+
    // |     || Size (%) | Time (%) |
    // +-----++----------+----------+
    // | BOI ||   -00.00 |   -x00.0 |
    // | QOI ||   +00.00 |   +x00.0 |
    // +-----++----------+----------+
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hsep = "+-----++----------+----------+\n".yellow();
        let title = "|     || Size (%) | Time (%) |\n".yellow();
        let mut count = 0;
        let mut sizes: HashMap<Algo, f64> = HashMap::new();
        let mut times: HashMap<Algo, f64> = HashMap::new();
        for tests in self.tests.iter() {
            count += 1;
            for test in tests.tests.iter() {
                *sizes.entry(test.algo).or_insert(0.0) += test.size_diff();
                *times.entry(test.algo).or_insert(0.0) += test.time_diff();
            }
        }
        let tests = sizes
            .iter()
            .map(|(key, size)| {
                let time = times.get(key).unwrap();
                let sep = "|".to_string().yellow();
                let dsep = "||".to_string().yellow();
                format!(
                    "{sep} {} {dsep} {: >8} {sep} {: >8} {sep}",
                    key.to_string().cyan(),
                    size_diff_to_string(*key, size / count as f64),
                    time_diff_to_string(*key, time / count as f64),
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let logger = &self.agregator;
        write!(f, "\n{hsep}{title}{hsep}{}\n{hsep}\n{logger}", tests)
    }
}

pub struct TestImage {
    path: String,
    bytes: Vec<u8>,
    height: usize,
    width: usize,
    is_alpha: bool,
    tests: Vec<Test>,
    logger: log::Logger,
}

impl TestImage {
    pub fn test(path: String) -> Self {
        let decoder = png::Decoder::new(std::fs::File::open(path.clone()).unwrap());
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let next_frame = reader.next_frame(&mut buf).unwrap();
        let bytes = buf[..next_frame.buffer_size()].to_vec();
        let info = reader.info();
        let is_alpha = info.color_type == png::ColorType::Rgba;
        let mut test = Self {
            width: info.width as usize,
            height: info.height as usize,
            is_alpha,
            bytes,
            path,
            tests: Vec::new(),
            logger: log::Logger::new(),
        };
        test.test_png();
        test.test_boi_with_log();
        test.test_qoi();
        test
    }

    pub fn test_boi_with_log(&mut self) {
        let mut test = Test::start_with_ref(Algo::Boi, &self.tests[0]);
        let (logger, encoded) = if !self.is_alpha {
            encoder::Encoder::<3>::encode_with_logger(&self.bytes, self.width, self.height)
        } else {
            encoder::Encoder::<4>::encode_with_logger(&self.bytes, self.width, self.height)
        };
        let len = encoded.len();
        let _ = std::fs::write("./img/out.boi", encoded);
        test.stop(len);
        self.tests.push(test);
        self.logger = logger;
    }

    pub fn test_boi(&mut self) {
        let mut test = Test::start_with_ref(Algo::Boi, &self.tests[0]);
        let encoded = if !self.is_alpha {
            encoder::Encoder::<3>::encode(&self.bytes, self.width, self.height)
        } else {
            encoder::Encoder::<4>::encode(&self.bytes, self.width, self.height)
        };
        let len = encoded.len();
        let _ = std::fs::write("./img/out.boi", encoded);
        test.stop(len);
        self.tests.push(test);
    }

    pub fn test_fas(&mut self) {
        //     let mut test = Test::start_with_ref(Algo::Fas, &self.tests[0]);
        //     let encoded = if !self.is_alpha {
        //         encoder::Encoder::<3>::encode_fast(&self.bytes, self.width, self.height)
        //     } else {
        //         encoder::Encoder::<4>::encode_fast(&self.bytes, self.width, self.height)
        //     };
        //     let len = encoded.len();
        //     let _ = std::fs::write("./img/out.boi", encoded);
        //     test.stop(len);
        //     self.tests.push(test);
    }

    pub fn test_qoi(&mut self) {
        let mut test = Test::start_with_ref(Algo::Qoi, &self.tests[0]);
        let encoded =
            qoi::encode_to_vec(&self.bytes, self.width as u32, self.height as u32).unwrap();
        let _ = std::fs::write("./img/out.qoi", encoded);
        test.stop(std::fs::metadata("./img/out.qoi").unwrap().len() as usize);
        self.tests.push(test);
    }

    pub fn test_png(&mut self) {
        let mut test = Test::start(Algo::Png);
        let path = std::path::Path::new("./img/out.png");
        let file = std::fs::File::create(path).unwrap();
        let ref mut w = std::io::BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
        match self.is_alpha {
            true => encoder.set_color(png::ColorType::Rgba),
            false => encoder.set_color(png::ColorType::Rgb),
        }
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Default);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&self.bytes).unwrap();
        test.stop(std::fs::metadata("./img/out.png").unwrap().len() as usize);
        self.tests.push(test);
    }
}

impl std::fmt::Display for TestImage {
    // +-----++-----------+--------++-----------+--------+
    // |     || Size (Ko) |      % || Time (ms) |        |
    // +-----++-----------+--------++-----------+--------+
    // | PNG ||    000000 |     -- ||    000000 |     -- |
    // | BOI ||    000000 | -00.00 ||    000000 | -x00.0 |
    // | QOI ||    000000 | +00.00 ||    000000 | +x00.0 |
    // +-----++-----------+--------++-----------+--------+
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hsep = "+-----++-----------+--------++-----------+--------+\n".yellow();
        let title = "|     || Size (Ko) |      % || Time (ms) |        |\n".yellow();
        let tests = self
            .tests
            .iter()
            .map(|test| test.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        write!(
            f,
            "{}\n{hsep}{title}{hsep}{}\n{hsep}",
            self.path.yellow(),
            tests
        )
    }
}
