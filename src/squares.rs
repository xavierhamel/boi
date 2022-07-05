use crate::img;

pub struct Blob<const CHANNELS: usize> {
    color: [u8; CHANNELS], //img::Pixel<CHANNELS>,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl<const CHANNELS: usize> Blob<CHANNELS> {
    pub fn new(color: img::Pixel<CHANNELS>, start_x: usize, end_x: usize, start_y: usize) -> Self {
        let mut color: [u8; CHANNELS] = [255; CHANNELS];
        color[0] += 93 * (start_x * end_x) as u8;
        color[1] += 23 * (start_x * end_x) as u8;
        color[2] += 219 * (start_x * end_x) as u8;
        Self {
            color,
            x: start_x,
            y: start_y,
            width: end_x - start_x,
            height: 1,
        }
    }

    pub fn size(&self) -> usize {
        self.width * self.height
    }

    pub fn is_big_enough(&self) -> bool {
        self.height * (self.width / 64 * 9) > 71
    }

    pub fn is_inside(&self, x: usize, y: usize) -> bool {
        if x >= self.x && x <= self.x + self.width {
            return y >= self.y && y <= self.y + self.height;
        }
        false
    }

    pub fn generate_map(&self) -> Vec<usize> {
        let mut map = Vec::with_capacity(self.width * self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                map.push((x + self.x) + (self.y + y) * self.width);
            }
        }
        map
    }
}

pub struct Blobs<const CHANNELS: usize> {
    img_width: usize,
    img_height: usize,
    pixels: Vec<img::Pixel<CHANNELS>>,
    pub blobs: Vec<Blob<CHANNELS>>,
    map: Vec<usize>,
}

impl<const CHANNELS: usize> Blobs<CHANNELS> {
    pub fn new(raw: &[u8], img_width: usize, img_height: usize) -> Self {
        let mut previous_chunk = &[0; CHANNELS][..];
        let mut pixels = Vec::with_capacity(raw.len() / CHANNELS);
        for current in raw.chunks_exact(CHANNELS) {
            pixels.push(img::Pixel::<CHANNELS>::compute_forward(
                previous_chunk,
                current,
            ));
            previous_chunk = current;
        }

        Self {
            img_width,
            img_height,
            map: vec![usize::MAX; pixels.len()],
            pixels,
            blobs: Vec::new(),
        }
    }

    pub fn compute(&mut self) {
        let mut closed_blobs = Vec::new();
        let mut growing_blobs: Vec<Blob<CHANNELS>> = Vec::new();

        for y in 0..self.img_height {
            let mut previous = self.pixels[y * self.img_width];
            let mut is_repeating = false;
            let mut starting_x = 0;
            for x in 1..self.img_width {
                let current = self.pixels[y * self.img_width + x];
                if previous == current {
                    if !is_repeating {
                        starting_x = x - 1;
                        is_repeating = true;
                    }
                } else {
                    if is_repeating {
                        if x - starting_x > 64 {
                            let mut is_blob_added = false;
                            for blob in growing_blobs.iter_mut() {
                                if blob.x == starting_x && blob.width == x - starting_x {
                                    blob.height += 1;
                                    is_blob_added = true;
                                    break;
                                }
                            }
                            if !is_blob_added {
                                growing_blobs.push(Blob::new(current, starting_x, x, y));
                            }
                        }
                        is_repeating = false;
                    }
                }
                previous = current;
            }
            //growing_blobs.sort_unstable_by(|a, b| (b.size()).cmp(&a.size()));
            //growing_blobs.truncate(10);
            let (mut newly_closed_blobs, still_growing_blobs): (Vec<_>, Vec<_>) = growing_blobs
                .into_iter()
                .partition(|blob| blob.y + blob.height == y);
            growing_blobs = still_growing_blobs;
            closed_blobs.append(&mut newly_closed_blobs);
        }
        //closed_blobs.append(&mut growing_blobs);
        //closed_blobs.sort_unstable_by(|a, b| (b.size()).cmp(&a.size()));
        //closed_blobs.truncate(512);
        closed_blobs
            .iter()
            .filter(|blob| blob.is_big_enough())
            .collect::<Vec<_>>();
        self.blobs = closed_blobs;
        self.generate_blobs_map();
    }

    pub fn generate_blobs_map(&mut self) {
        for (blob_idx, blob) in self.blobs.iter().enumerate() {
            blob.generate_map()
                .iter()
                .for_each(|idx| self.map[*idx] = blob_idx);
        }
    }

    pub fn is_inside_blob(&self, idx: usize) -> bool {
        self.map[idx] != usize::MAX
    }

    pub fn size(&mut self) -> usize {
        self.blobs.len() * 9
    }

    // pub fn into_test(mut self) -> Vec<u8> {
    //     let mut bytes = Vec::with_capacity(self.pixels.len());
    //     let mut color = [255; CHANNELS];

    //     for idx in 0..self.pixels.len() {
    //         let x = idx % self.img_width;
    //         let y = idx / self.img_width;
    //         let mut is_inside_blob = false;
    //         for blob in self.blobs.iter() {
    //             if blob.is_inside(x, y) {
    //                 is_inside_blob = true;
    //                 bytes.append(&mut blob.color.to_vec());
    //                 break;
    //             }
    //         }
    //         if !is_inside_blob {
    //             let mut p = self.pixels[idx]
    //                 .0
    //                 .into_iter()
    //                 .map(|c| c as u8)
    //                 .collect::<Vec<_>>();
    //             p[CHANNELS - 1] = 255;
    //             bytes.append(&mut p);
    //         }
    //     }

    //     bytes
    // }
}

pub struct ColorBlobs<const CHANNELS: usize> {
    img_width: usize,
    pixels: Vec<img::Pixel<CHANNELS>>,
    is_visited: Vec<bool>,
    blobs: Vec<Vec<usize>>,
}

impl<const CHANNELS: usize> ColorBlobs<CHANNELS> {
    pub fn new(raw: &[u8], img_width: usize) -> Self {
        let mut previous_chunk = &[0; CHANNELS][..];
        let mut pixels = Vec::with_capacity(raw.len() / CHANNELS);
        for current in raw.chunks_exact(CHANNELS) {
            pixels.push(img::Pixel::<CHANNELS>::compute_forward(
                previous_chunk,
                current,
            ));
            previous_chunk = current;
        }

        Self {
            img_width,
            is_visited: vec![false; pixels.len()],
            pixels,
            blobs: Vec::new(),
        }
    }

    pub fn compute(&mut self) {
        for idx in 0..self.pixels.len() {
            self.visit(idx);
            //self.pixels[idx].0 = [255; CHANNELS];
        }
    }

    pub fn visit(&mut self, idx: usize) {
        let mut stack = vec![idx];
        let mut blob = vec![idx];

        while let Some(pixel_idx) = stack.pop() {
            if self.is_visited[pixel_idx] {
                continue;
            }
            self.is_visited[pixel_idx] = true;
            for neighbor_idx in self.neighbors(pixel_idx) {
                if !self.is_visited[neighbor_idx] && self.pixels[neighbor_idx] == self.pixels[idx] {
                    stack.push(neighbor_idx);
                    blob.push(neighbor_idx);
                }
            }
        }
        if blob.len() > 10 {
            self.blobs.push(blob);
        }
    }

    pub fn neighbors(&self, idx: usize) -> Vec<usize> {
        let mut neighbors = Vec::with_capacity(4);
        let x = idx % self.img_width;
        if x > 0 {
            neighbors.push(idx - 1);
        }
        if x < self.img_width - 1 {
            neighbors.push(idx + 1);
        }
        if idx > self.img_width {
            neighbors.push(idx - self.img_width);
        }
        if idx < self.pixels.len() - self.img_width - 1 {
            neighbors.push(idx + self.img_width);
        }
        neighbors
    }
}
