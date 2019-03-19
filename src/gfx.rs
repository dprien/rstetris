use crate::{gfx, util, js_api};

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
    clock: util::Clock,
    duration: f64,

    rows: Vec<usize>,
    width: usize,
}

pub struct WhooshAnimation {
    clock: util::Clock,
    duration: f64,

    points: Vec<(usize, usize)>,
    color: Color,

    x: i32,
    y1: i32,
    y2: i32,
}

pub struct TitleAnimation {
    clock: util::Clock,

    width: usize,
    height: usize,
}

pub struct GameOverAnimation {
    clock: util::Clock,
    duration: f64,

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

    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
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
    pub fn new(rows: Vec<usize>, width: usize, duration: f64) -> Self {
        Self {
            clock: util::Clock::new(),
            duration,

            rows,
            width,
        }
    }
}

impl Animation for LineClearAnimation {
    fn is_active(&self) -> bool {
        self.clock.elapsed() < self.duration
    }

    fn is_blocking(&self) -> bool {
        true
    }

    fn tick(&mut self, timestamp: f64) {
        self.clock.update(timestamp);

        let color = {
            let t = util::clamp(self.clock.elapsed() / self.duration, 0.0, 1.0);
            if t <= 0.7 {
                Color::white().fade(ease::quadratic_out(1.0 - t / 0.7)).to_argb32()
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
    pub fn new<I>(points: I, color: Color, x: i32, y1: i32, y2: i32, duration: f64) -> Self
        where I: IntoIterator<Item = (usize, usize)>
    {
        Self {
            clock: util::Clock::new(),
            duration,

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
        self.clock.elapsed() < self.duration
    }

    fn is_blocking(&self) -> bool {
        true
    }

    fn tick(&mut self, timestamp: f64) {
        self.clock.update(timestamp);
        let t = util::clamp(self.clock.elapsed() / self.duration, 0.0, 1.0);

        for y in self.y1..self.y2 {
            let intensity = {
                let yt = (y - self.y1 + 1) as f64 / (self.y2 - self.y1) as f64;
                if t <= yt {
                    1.0 - t / yt
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
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            clock: util::Clock::new(),

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
        self.clock.update(timestamp);
        if !self.clock.has_passed_multiple_of(500.0, 0.0) {
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
    pub fn new(width: usize, height: usize, duration: f64) -> Self {
        Self {
            clock: util::Clock::new(),
            duration,

            width,
            height
        }
    }
}

impl Animation for GameOverAnimation {
    fn is_active(&self) -> bool {
        self.clock.elapsed() < self.duration
    }

    fn is_blocking(&self) -> bool {
        false
    }

    fn tick(&mut self, timestamp: f64) {
        self.clock.update(timestamp);

        let white = gfx::Color::white();

        let t = 1.0 - util::clamp(self.clock.elapsed() / self.duration, 0.0, 1.0);
        let tick_height = t * self.height as f64;
        let tick_y = tick_height.trunc() as usize;
        let tick_t = tick_height.fract();

        for y in 0..self.height {
            let color = {
                if y < tick_y {
                    continue;
                } else if y > tick_y {
                    0x000000
                } else {
                    white.fade(tick_t).to_argb32()
                }
            };

            for x in 0..self.width {
                js_api::draw_block(x as u32, y as u32, color);
            }
        }
    }
}
