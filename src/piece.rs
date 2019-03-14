use std::{cmp, fmt};

use crate::{gfx, util, js_api};

struct BlockMatrix {
    stride: usize,
    data: Vec<bool>,
}

pub struct Piece {
    pub name: String,
    pub color: gfx::Color,
    rotations: [BlockMatrix; 4],
}

impl BlockMatrix {
    fn new<E, T>(data: T) -> Self
        where E: AsRef<[u8]>,
              T: AsRef<[E]>
    {
        let stride = data.as_ref().len();

        let mut v = Vec::with_capacity(stride * stride);
        for row in data.as_ref() {
            assert!(row.as_ref().len() == stride);
            v.extend(row.as_ref().into_iter().map(|&x| { x != 0 }));
        }

        Self {
            stride: stride,
            data: v,
        }
    }

    fn get_block(&self, x: usize, y: usize) -> bool {
        self.data[y * self.stride + x]
    }

    fn iter_coords(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.stride).flat_map(move |y| {
            (0..self.stride).map(move |x| {
                (x, y)
            })
        })
        .filter(move |&(x, y)| {
            self.get_block(x, y)
        })
    }

    fn rotate(&self) -> Self {
        let mut v = Vec::with_capacity(self.data.len());
        for x in 0..self.stride {
            for y in (0..self.stride).rev() {
                v.push(self.get_block(x, y));
            }
        }

        BlockMatrix {
            stride: self.stride,
            data: v,
        }
    }
}

impl fmt::Debug for BlockMatrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.stride {
            for x in 0..self.stride {
                write!(f, "{}", if self.get_block(x, y) { "X" } else { " " })?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}


impl Piece {
    fn new<E, T, S>(name: S, color: gfx::Color, data: T) -> Self
        where E: AsRef<[u8]>,
              T: AsRef<[E]>,
              S: Into<String>,
    {
        let a = BlockMatrix::new(data);
        let b = a.rotate();
        let c = b.rotate();
        let d = c.rotate();

        Self {
            name: name.into(),
            color: color,
            rotations: [a, b, c, d],
        }
    }

    #[allow(dead_code)]
    pub fn bounds(&self, rotation: usize) -> ((usize, usize), (usize, usize)) {
        let mut x1 = usize::max_value();
        let mut x2 = usize::min_value();
        let mut y1 = usize::max_value();
        let mut y2 = usize::min_value();

        for (x, y) in self.iter_coords(rotation) {
            x1 = cmp::min(x1, x);
            x2 = cmp::max(x2, x);
            y1 = cmp::min(y1, y);
            y2 = cmp::max(y2, y);
        }

        ((x1, y1), (x2, y2))
    }

    pub fn iter_coords(&self, rotation: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        let v = &self.rotations[rotation % self.rotations.len()];
        v.iter_coords()
    }

    pub fn draw(&self, position: &util::Position, rotation: usize, intensity: f64) {
        let color = self.color.fade(intensity);

        for (bx, by) in self.iter_coords(rotation) {
            let dx = position.x + (bx as i32);
            let dy = position.y + (by as i32);
            js_api::draw_block(dx as u32, dy as u32, color.to_argb32());
        }
    }
}

#[allow(dead_code)]
pub fn make_standard() -> Vec<Piece> {
    vec![
        Piece::new("I", gfx::Color::from_argb32(0x00ffff), [
                   [0, 0, 0, 0],
                   [1, 1, 1, 1],
                   [0, 0, 0, 0],
                   [0, 0, 0, 0],
        ]),
        Piece::new("O", gfx::Color::from_argb32(0xffff00), [
                   [1, 1],
                   [1, 1],
        ]),
        Piece::new("J", gfx::Color::from_argb32(0x0000ff), [
                   [1, 0, 0],
                   [1, 1, 1],
                   [0, 0, 0],
        ]),
        Piece::new("L", gfx::Color::from_argb32(0xffa500), [
                   [0, 0, 1],
                   [1, 1, 1],
                   [0, 0, 0],
        ]),
        Piece::new("S", gfx::Color::from_argb32(0x00ff00), [
                   [0, 1, 1],
                   [1, 1, 0],
                   [0, 0, 0],
        ]),
        Piece::new("Z", gfx::Color::from_argb32(0xff0000), [
                   [1, 1, 0],
                   [0, 1, 1],
                   [0, 0, 0],
        ]),
        Piece::new("T", gfx::Color::from_argb32(0xaa00ff), [
                   [0, 1, 0],
                   [1, 1, 1],
                   [0, 0, 0],
        ]),
    ]
}

#[allow(dead_code)]
pub fn make_ttc_original() -> Vec<Piece> {
    // See https://tetris.wiki/SRS#How_Guideline_SRS_Really_Works
    vec![
        Piece::new("I", gfx::Color::from_argb32(0x00ffff), [
                   [0, 0, 0, 0, 0],
                   [0, 0, 0, 0, 0],
                   [0, 1, 1, 1, 1],
                   [0, 0, 0, 0, 0],
                   [0, 0, 0, 0, 0],
        ]),
        Piece::new("O", gfx::Color::from_argb32(0xffff00), [
                   [0, 1, 1],
                   [0, 1, 1],
                   [0, 0, 0],
        ]),
        Piece::new("J", gfx::Color::from_argb32(0x0000ff), [
                   [1, 0, 0],
                   [1, 1, 1],
                   [0, 0, 0],
        ]),
        Piece::new("L", gfx::Color::from_argb32(0xffa500), [
                   [0, 0, 1],
                   [1, 1, 1],
                   [0, 0, 0],
        ]),
        Piece::new("S", gfx::Color::from_argb32(0x00ff00), [
                   [0, 1, 1],
                   [1, 1, 0],
                   [0, 0, 0],
        ]),
        Piece::new("Z", gfx::Color::from_argb32(0xff0000), [
                   [1, 1, 0],
                   [0, 1, 1],
                   [0, 0, 0],
        ]),
        Piece::new("T", gfx::Color::from_argb32(0xaa00ff), [
                   [0, 1, 0],
                   [1, 1, 1],
                   [0, 0, 0],
        ]),
    ]
}
