pub struct Logger {
    pub repeating: usize,
    pub short: usize,
    pub medium: usize,
    pub long: usize,
    pub offset: usize,
    pub palette: usize,
    pub gray: usize,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            repeating: 0,
            short: 0,
            medium: 0,
            long: 0,
            offset: 0,
            palette: 0,
            gray: 0,
        }
    }

    fn total(&self) -> usize {
        self.repeating
            + self.short
            + self.medium
            + self.long
            + self.offset
            + self.palette
            + self.gray
    }
}

pub struct Agregator {
    pub repeating: f64,
    pub short: f64,
    pub medium: f64,
    pub long: f64,
    pub offset: f64,
    pub palette: f64,
    pub gray: f64,
}

impl Agregator {
    pub fn new() -> Self {
        Self {
            repeating: 0.0,
            short: 0.0,
            medium: 0.0,
            long: 0.0,
            offset: 0.0,
            palette: 0.0,
            gray: 0.0,
        }
    }

    fn total(&self) -> f64 {
        self.repeating
            + self.short
            + self.medium
            + self.long
            + self.offset
            + self.palette
            + self.gray
    }

    pub fn add(&mut self, logger: &Logger) {
        let total = logger.total() as f64;
        self.repeating += logger.repeating as f64 / total;
        self.short += logger.short as f64 / total;
        self.medium += logger.medium as f64 / total;
        self.long += logger.long as f64 / total;
        self.offset += logger.offset as f64 / total;
        self.palette += logger.palette as f64 / total;
        self.gray += logger.gray as f64 / total;
    }

    pub fn average(&mut self) {
        let total = self.total();
        self.repeating = self.repeating / total;
        self.short = self.short / total;
        self.medium = self.medium / total;
        self.long = self.long / total;
        self.offset = self.offset / total;
        self.palette = self.palette / total;
        self.gray = self.gray / total;
    }
}

impl std::fmt::Display for Agregator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "repeating: {}\nshort: {}\nmedium: {}\nlong: {}\noffset: {}\npalette: {}\ngray: {}",
            self.repeating,
            self.short,
            self.medium,
            self.long,
            self.offset,
            self.palette,
            self.gray
        )
    }
}
