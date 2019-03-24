use crate::{gfx, piece, util, js_api};

mod ease;

pub trait Animation {
    fn should_block(&self) -> bool;
    fn draw(&self, t: f64);
}

#[derive(Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

pub struct AnimationQueue {
    animations: Vec<(f64, Option<f64>, Box<dyn Animation>)>,
}

pub struct LineClearAnimation {
    rows: Vec<usize>,
    width: usize,
}

pub struct WhooshAnimation {
    points: Vec<(usize, usize)>,
    color: Color,

    x: i32,
    y1: i32,
    y2: i32,
}

pub struct TitleAnimation {
    width: usize,
    height: usize,

    pieces: Vec<piece::Piece>,
}

pub struct GameOverAnimation {
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
        self.animations.iter().any(|(_start, _end, anim)| { anim.should_block() })
    }

    pub fn schedule(&mut self, start: f64, duration: f64, anim: Box<dyn Animation>) {
        self.animations.push((start, Some(start + duration), anim));
    }

    pub fn endless(&mut self, anim: Box<dyn Animation>) {
        self.animations.push((0.0, None, anim));
    }

    pub fn tick(&mut self, timestamp: f64) {
        if self.animations.is_empty() {
            return;
        }

        for (start, maybe_end, anim) in self.animations.iter() {
            if timestamp >= *start {
                let divisor = maybe_end.map(|end| { end - start }).unwrap_or(1000.0);
                let t = (timestamp - start) / divisor;
                anim.draw(t);
            }
        }

        self.animations.retain(|(_start, maybe_end, _anim)| {
            maybe_end.map(|end| { timestamp < end }).unwrap_or(true)
        });
    }
}

impl LineClearAnimation {
    pub fn new(rows: Vec<usize>, width: usize) -> Self {
        Self {
            rows,
            width,
        }
    }
}

impl Animation for LineClearAnimation {
    fn should_block(&self) -> bool {
        true
    }

    fn draw(&self, t: f64) {
        let t = util::clamp(t, 0.0, 1.0);

        let color = {
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
    pub fn new<I>(points: I, color: Color, x: i32, y1: i32, y2: i32) -> Self
        where I: IntoIterator<Item = (usize, usize)>
    {
        Self {
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
    fn should_block(&self) -> bool {
        true
    }

    fn draw(&self, t: f64) {
        let t = util::clamp(t, 0.0, 1.0);

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
            width,
            height,

            pieces: piece::make_standard(),
        }
    }

    fn draw_piece(&self, rng: &util::LinearCongruentialGenerator) {
        let index = (rng.next() as usize) % self.pieces.len();
        let rotation = (rng.next() as usize) % 4;
        let piece = &self.pieces[index];
        let ((x1, y1), (x2, y2)) = piece.bounds(rotation);

        let cx = rng.next() as usize % (self.width - (x2 - x1)) - x1;
        let cy = rng.next() as usize % (self.height - (y2 - y1)) - y1;

        for (bx, by) in piece.iter_coords(rotation) {
            let bx = bx as i32 + cx as i32;
            let by = by as i32 + cy as i32;
            js_api::draw_block(bx as u32, by as u32, piece.color.to_argb32());
        }
    }
}

impl Animation for TitleAnimation {
    fn should_block(&self) -> bool {
        false
    }

    fn draw(&self, t: f64) {
        let t = t * 2.0;

        for y in 0..self.height {
            for x in 0..self.width {
                js_api::draw_block(x as u32, y as u32, 0x000000);
            }
        }

        if t.fract() >= 0.8 {
            return;
        }

        let rng = util::LinearCongruentialGenerator::new(t.floor() as u32);
        self.draw_piece(&rng);
    }
}

impl GameOverAnimation {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height
        }
    }
}

impl Animation for GameOverAnimation {
    fn should_block(&self) -> bool {
        false
    }

    fn draw(&self, t: f64) {
        let white = gfx::Color::white();

        let t = 1.0 - util::clamp(t, 0.0, 1.0);
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
