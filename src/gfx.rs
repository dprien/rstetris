use crate::{js_api};

mod ease;

pub trait Animation {
    fn is_active(&self) -> bool;
    fn tick(&mut self, timestamp: f64);
}

#[derive(Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
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
    y1: i32,
    y2: i32,
    color: Color,
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

impl LineClearAnimation {
    pub fn new(rows: Vec<usize>, width: usize, timestamp: f64, duration: f64) -> Self {
        Self {
            start: timestamp,
            end: timestamp + duration,
            timestamp: timestamp,
            rows: rows,
            width: width,
        }
    }
}

impl Animation for LineClearAnimation {
    fn is_active(&self) -> bool {
        self.timestamp < self.end
    }

    fn tick(&mut self, timestamp: f64) {
        self.timestamp = timestamp;
        let t_norm = (self.timestamp - self.start) / (self.end - self.start);

        let color = Color::white().fade(ease::cubic_out(1.0 - t_norm));

        for &y in self.rows.iter() {
            for x in 0..self.width {
                js_api::draw_block(x as u32, y as u32, color.to_argb32());
            }
        }
    }
}

impl WhooshAnimation {
    pub fn new(points: Vec<(usize, usize)>, y1: i32, y2: i32, color: Color, timestamp: f64, duration: f64) -> Self {
        Self {
            start: timestamp,
            end: timestamp + duration,
            timestamp: timestamp,
            points: points,
            y1: y1,
            y2: y2,
            color: color,
        }
    }

    fn draw_points(&self, y: i32, color: &Color) {
        for &(bx, by) in self.points.iter() {
            js_api::draw_block(bx as u32, (by as i32 + y) as u32, color.to_argb32());
        }
    }
}

impl Animation for WhooshAnimation {
    fn is_active(&self) -> bool {
        self.timestamp < self.end
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

            // let grey = Color::rgb(128, 128, 128);
            // let gradient = self.color.fade(intensity);
            // let color = grey.mix(&gradient, ease::cubic_out(t_norm));

            let color = self.color.fade(intensity);
            self.draw_points(y, &color);
        }

        self.draw_points(self.y2, &self.color);
    }
}
