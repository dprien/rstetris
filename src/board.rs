use std::fmt;

use crate::{gfx, piece, util, js_api};

pub struct Board {
    width: usize,
    height: usize,
    data: Vec<Option<gfx::Color>>,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width: width,
            height: height,
            data: vec![None; width * height],
        }
    }

    fn get_block(&self, x: usize, y: usize) -> &Option<gfx::Color> {
        &self.data[y * self.width + x]
    }

    fn put_block(&mut self, x: usize, y: usize, color: gfx::Color) {
        self.data[y * self.width + x] = Some(color);
    }

    fn is_line(&self, y: usize) -> bool {
        (0..self.width).all(|x| { self.get_block(x, y).is_some() })
    }

    fn clear_row(&mut self, y: usize) {
        for x in 0..self.width {
            self.data[y * self.width + x] = None
        }
    }

    fn clone_row(&mut self, y_top: usize, y_bottom: usize) {
        if y_top < y_bottom {
            let (top, bottom) = self.data.split_at_mut(y_bottom * self.width);

            let src = &top[(y_top * self.width)..(y_top * self.width + self.width)];
            bottom[..self.width].clone_from_slice(src);
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.data.resize(self.width * self.height, None);
    }

    pub fn draw(&self) {
        for by in 0..self.height {
            for bx in 0..self.width {
                let color = self.get_block(bx, by)
                    .as_ref()
                    .map(|x| { x.to_argb32() })
                    .unwrap_or(0);

                js_api::draw_block(bx as u32, by as u32, color)
            }
        }
    }

    pub fn initial_position(&self, piece: &piece::Piece, rotation: usize) -> util::Position {
        let ((x1, y1), (x2, _)) = piece.bounds(rotation);
        util::Position::new(
            (self.width / 2) as i32 - ((x1 + x2 + 1) / 2) as i32,
            -(y1 as i32),
        )
    }

    pub fn put_piece(&mut self, piece: &piece::Piece, position: &util::Position, rotation: usize) {
        for (bx, by) in piece.iter_coords(rotation) {
            let dx = position.x + (bx as i32);
            let dy = position.y + (by as i32);

            assert!(dx >= 0 && dx < self.width as i32);
            assert!(dy >= 0 && dy < self.height as i32);

            self.put_block(dx as usize, dy as usize, piece.color.clone());
        }
    }

    pub fn collides(&self, piece: &piece::Piece, position: &util::Position, rotation: usize) -> bool {
        for (bx, by) in piece.iter_coords(rotation) {
            let dx = position.x + (bx as i32);
            let dy = position.y + (by as i32);

            if dx < 0 || dx >= self.width as i32 {
                return true;
            }

            if dy < 0 || dy >= self.height as i32 {
                return true;
            }

            if self.get_block(dx as usize, dy as usize).is_some() {
                return true;
            }
        }

        false
    }

    pub fn find_drop_position(&self, piece: &piece::Piece, position: &util::Position, rotation: usize) -> util::Position {
        (position.y..)
            .filter(|y| {
                let maybe_drop_position = util::Position::new(position.x, y + 1);
                self.collides(piece, &maybe_drop_position, rotation)
            })
            .next()
            .map(|y| { util::Position::new(position.x, y) })
            .unwrap()
    }

    pub fn clear_lines(&mut self) -> Vec<usize> {
        let mut cleared_lines = Vec::new();

        let mut y1_iter = (0..self.height).rev();
        let mut y2_iter = (0..self.height).rev().peekable();

        while let (Some(y1), Some(&y2)) = (y1_iter.next(), y2_iter.peek()) {
            if self.is_line(y1) {
                cleared_lines.push(y1);
            } else {
                y2_iter.next();
                self.clone_row(y1, y2);
            }
        }

        for y2 in y2_iter {
            self.clear_row(y2);
        }

        cleared_lines
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{}", if self.get_block(x, y).is_some() { "X" } else { " " })?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
