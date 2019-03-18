use crate::{js_api};

mod ease;

pub trait Animation {
    fn is_active(&self) -> bool;
    fn is_blocking(&self) -> bool;

    fn tick(&mut self, timestamp: f64);
}

#[derive(Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

pub struct AnimationQueue {
    animations: Vec<Box<dyn Animation>>,
}

pub struct LineClearAnimation {
    start: f64,
    end: f64,
    timestamp: f64,
    rows: Vec<usize>,
    width: usize,
}

pub struct WhooshAnimation {
    start: f64,
    end: f64,

    timestamp: f64,

    points: Vec<(usize, usize)>,
    color: Color,

    x: i32,
    y1: i32,
    y2: i32,
}

pub struct TitleAnimation {
    timestamp_curr: f64,
    timestamp_prev: f64,

    width: usize,
    height: usize,
}

pub struct GameOverAnimation {
    start: f64,
    end: f64,
    timestamp: f64,
    width: usize,
    height: usize,
}

impl Color {
    pub fn black() -> Self {
        Self::rgb(0, 0, 0)
    }

    pub fn white() -> Self {
        Self::rgb(255, 255, 255)
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r,
            g: g,
            b: b,
        }
    }

    pub fn from_argb32(value: u32) -> Self {
        Self {
            r: ((value >> 16) & 0xff) as u8,
            g: ((value >>  8) & 0xff) as u8,
            b: ((value      ) & 0xff) as u8,
        }
    }

    pub fn to_argb32(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn mix(&self, other: &Color, t: f64) -> Self {
        let r1 = (self.r as f64) * (1.0 - t);
        let g1 = (self.g as f64) * (1.0 - t);
        let b1 = (self.b as f64) * (1.0 - t);

        let r2 = (other.r as f64) * t;
        let g2 = (other.g as f64) * t;
        let b2 = (other.b as f64) * t;

        Self {
            r: (r1 + r2).round() as u8,
            g: (g1 + g2).round() as u8,
            b: (b1 + b2).round() as u8,
        }
    }

    pub fn fade(&self, t: f64) -> Self {
        Self::black().mix(self, t)
    }
}

impl AnimationQueue {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
        }
    }

    pub fn should_block(&self) -> bool {
        self.animations.iter().any(|x| { x.is_blocking() })
    }

    pub fn add(&mut self, animation: Box<dyn Animation>) {
        self.animations.push(animation);
    }

    pub fn update(&mut self, timestamp: f64) {
        if !self.animations.is_empty() {
            for x in self.animations.iter_mut() {
                x.tick(timestamp);
            }

            self.animations.retain(|x| { x.is_active() });
        }
    }
}


impl LineClearAnimation {
    pub fn new(rows: Vec<usize>, width: usize, timestamp: f64, duration: f64) -> Self {
        Self {
            start: timestamp,
            end: timestamp + duration,
            timestamp,
            rows,
            width,
        }
    }
}

impl Animation for LineClearAnimation {
    fn is_active(&self) -> bool {
        self.timestamp < self.end
    }

    fn is_blocking(&self) -> bool {
        true
    }

    fn tick(&mut self, timestamp: f64) {
        self.timestamp = timestamp;
        let t_norm = (self.timestamp - self.start) / (self.end - self.start);

        let color = {
            if t_norm <= 0.7 {
                Color::white().fade(ease::quadratic_out(1.0 - t_norm / 0.7)).to_argb32()
            } else {
                0x000000
            }
        };

        for &y in self.rows.iter() {
            for x in 0..self.width {
                js_api::draw_block(x as u32, y as u32, color);
            }
        }
    }
}

impl WhooshAnimation {
    pub fn new<I>(points: I, color: Color, x: i32, y1: i32, y2: i32, timestamp: f64, duration: f64) -> Self
        where I: IntoIterator<Item = (usize, usize)>
    {
        Self {
            start: timestamp,
            end: timestamp + duration,
            timestamp,
            points: points.into_iter().collect(),
            color,
            x,
            y1,
            y2,
        }
    }

    fn draw_points(&self, y: i32, color: &Color) {
        for &(bx, by) in self.points.iter() {
            let bx = bx as i32 + self.x;
            let by = by as i32 + y;
            js_api::draw_block(bx as u32, by as u32, color.to_argb32());
        }
    }
}

impl Animation for WhooshAnimation {
    fn is_active(&self) -> bool {
        self.timestamp < self.end
    }

    fn is_blocking(&self) -> bool {
        true
    }

    fn tick(&mut self, timestamp: f64) {
        self.timestamp = timestamp;
        let t_norm = (self.timestamp - self.start) / (self.end - self.start);

        for y in self.y1..self.y2 {
            let intensity = {
                let y_norm = (y - self.y1 + 1) as f64 / (self.y2 - self.y1) as f64;
                if t_norm <= y_norm {
                    1.0 - t_norm / y_norm
                } else {
                    0.0
                }
            };

            let color = self.color.fade(intensity);
            self.draw_points(y, &color);
        }

        self.draw_points(self.y2, &self.color);
    }
}

impl TitleAnimation {
    pub fn new(width: usize, height: usize, timestamp: f64) -> Self {
        Self {
            timestamp_curr: timestamp,
            timestamp_prev: timestamp,

            width,
            height,
        }
    }
}

impl Animation for TitleAnimation {
    fn is_active(&self) -> bool {
        true
    }

    fn is_blocking(&self) -> bool {
        false
    }

    fn tick(&mut self, timestamp: f64) {
        self.timestamp_prev = self.timestamp_curr;
        self.timestamp_curr = timestamp;

        let prev = (self.timestamp_prev / 500.0).floor() as i32;
        let curr = (self.timestamp_curr / 500.0).floor() as i32;

        if curr <= prev {
            return;
        }

        static COLORS: &[u32] = &[0x00ffff, 0xffff00, 0x0000ff, 0xffa500, 0x00ff00, 0xff0000, 0xaa00ff];

        for y in 0..self.height {
            for x in 0..self.width {
                let index = (js_api::random() * COLORS.len() as f64).floor() as usize;
                let intensity = js_api::random();
                let color = Color::from_argb32(COLORS[index]).fade(intensity);

                js_api::draw_block(x as u32, y as i32 as u32, color.to_argb32());
            }
        }
    }
}

impl GameOverAnimation {
    pub fn new(width: usize, height: usize, timestamp: f64, duration: f64) -> Self {
        Self {
            start: timestamp,
            end: timestamp + duration,
            timestamp,
            width,
            height
        }
    }
}

impl Animation for GameOverAnimation {
    fn is_active(&self) -> bool {
        self.timestamp < self.end
    }

    fn is_blocking(&self) -> bool {
        false
    }

    fn tick(&mut self, timestamp: f64) {
        self.timestamp = timestamp;

        let t_norm = (self.timestamp - self.start) / (self.end - self.start);
        let start_y = ((1.0 - t_norm) * self.height as f64).ceil() as usize;

        let color = Color::rgb(127, 127, 127).to_argb32();

        for y in start_y..self.height {
            for x in 0..self.width {
                js_api::draw_block(x as u32, y as i32 as u32, color);
            }
        }
    }
}
