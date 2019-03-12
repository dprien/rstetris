use crate::{input, piece, board, util, js_api};

const INPUT_GAME_START: (usize, usize) = (0, 32);
const INPUT_STOP_GAME: (usize, usize) = (0, 27);

const INPUT_HARD_DROP: (usize, usize) = (0, 87);
const INPUT_SOFT_DROP: (usize, usize) = (0, 83);

const INPUT_MOVE_LEFT: (usize, usize) = (0, 65);
const INPUT_MOVE_RIGHT: (usize, usize)= (0, 68);

const INPUT_ROTATE_CW: (usize, usize)= (0, 39);
const INPUT_ROTATE_CCW: (usize, usize) = (0, 37);

const INITIAL_DELAY_SOFT_DROP: f64 = 1000.0 / 60.0 * 3.0;
const REPEAT_DELAY_SOFT_DROP: f64 = 1000.0 / 60.0 * 3.0;

const INITIAL_DELAY_MOVE: f64 = 1000.0 / 60.0 * 12.0;
const REPEAT_DELAY_MOVE: f64 = 1000.0 / 60.0 * 3.0;

const ANIMATION_DURATION_HARD_DROP: f64 = 300.0;
const ANIMATION_DURATION_LINE_CLEAR: f64 = 600.0;

trait Animation {
    fn is_finished(&self) -> bool;
    fn tick(&mut self, timestamp: f64);
}

trait GameState {
    fn tick(&mut self, timestamp: f64, controller: &input::Controller) -> Option<Box<dyn GameState>>;
}

struct GameTitle {
    board_width: usize,
    board_height: usize,
}

struct ActivePiece {
    index: usize,
    position: util::Position,
    rotation: usize,
}

struct GameRunning {
    pieces: Vec<piece::Piece>,
    board: board::Board,
    active_piece: ActivePiece,
    animations: Vec<Box<dyn Animation>>,
}

struct Game {
    frame_index: u64,
    controller: input::Controller,
    game_state: Box<dyn GameState>,
}

struct LineClearAnimation {
    timestamp: f64,
    interp: util::LinearInterp,
    rows: Vec<usize>,
    width: usize,
}

struct WhooshAnimation {
    timestamp: f64,
    interp: util::LinearInterp,
    points: Vec<(usize, usize)>,
    y1: i32,
    y2: i32,
    color: u32,
}

impl LineClearAnimation {
    fn new(rows: Vec<usize>, width: usize, timestamp: f64, duration: f64) -> Self {
        Self {
            timestamp: timestamp,
            interp: util::LinearInterp::new(timestamp, timestamp + duration),
            rows: rows,
            width: width,
        }
    }
}

impl Animation for LineClearAnimation {
    fn is_finished(&self) -> bool {
        self.timestamp >= self.interp.end()
    }

    fn tick(&mut self, timestamp: f64) {
        self.timestamp = timestamp;

        let t = self.interp.t(self.timestamp);
        let component = ((1.0 - t) * 255.0) as u32;
        let color = (component << 16) | (component << 8) | component;

        for &y in self.rows.iter() {
            for x in 0..self.width {
                js_api::draw_block(x as u32, y as u32, color);
            }
        }
    }
}

impl WhooshAnimation {
    fn new(points: Vec<(usize, usize)>, y1: i32, y2: i32, color: u32, timestamp: f64, duration: f64) -> Self {
        Self {
            timestamp: timestamp,
            interp: util::LinearInterp::new(timestamp, timestamp + duration),
            points: points,
            y1: y1,
            y2: y2,
            color: color,
        }
    }

    fn draw_points(&self, y: i32, color: u32) {
        for &(bx, by) in self.points.iter() {
            js_api::draw_block(bx as u32, (by as i32 + y) as u32, color);
        }
    }
}

impl Animation for WhooshAnimation {
    fn is_finished(&self) -> bool {
        self.timestamp >= self.interp.end()
    }

    fn tick(&mut self, timestamp: f64) {
        self.timestamp = timestamp;

        let t = self.interp.t(self.timestamp);

        for y in self.y1..self.y2 {
            let intensity = {
                let y_norm = (y - self.y1 + 1) as f64 / (self.y2 - self.y1) as f64;
                if t <= y_norm {
                    1.0 - t / y_norm
                } else {
                    0.0
                }
            };

            let color = util::color_intensity(self.color, intensity);
            self.draw_points(y, color);
        }

        self.draw_points(self.y2, self.color);
    }
}

impl GameTitle {
    fn new(board_width: usize, board_height: usize) -> Self {
        Self {
            board_width: board_width,
            board_height: board_height,
        }
    }
}

impl GameState for GameTitle {
    fn tick(&mut self, _timestamp: f64, controller: &input::Controller) -> Option<Box<dyn GameState>> {
        if controller.is_triggered(INPUT_GAME_START) {
            return Some(Box::new(GameRunning::new(self.board_width, self.board_height)))
        }

        None
    }
}

impl GameRunning {
    fn new(board_width: usize, board_height: usize) -> Self {
        let pieces = piece::make_standard();
        let board = board::Board::new(board_width, board_height);

        let index = 0;
        let rotation = 0;
        let position = board.initial_position(&pieces[index], rotation);

        Self {
            board: board,
            pieces: pieces,

            active_piece: ActivePiece {
                index: index,
                position: position,
                rotation: rotation,
            },

            animations: Vec::new(),
        }
    }

    fn place_new_piece(&mut self) {
        self.active_piece.index = (self.active_piece.index + 1) % self.pieces.len();
        self.active_piece.rotation = 0;

        let piece = &self.pieces[self.active_piece.index];
        self.active_piece.position = self.board.initial_position(piece, self.active_piece.rotation);
    }

    fn input_misc(&mut self, controller: &input::Controller) -> Option<Box<dyn GameState>> {
        if controller.is_triggered(INPUT_STOP_GAME) {
            self.board.clear();
            return Some(Box::new(GameTitle::new(self.board.width(), self.board.height())));
        }

        None
    }

    fn input_drop(&mut self, timestamp: f64, controller: &input::Controller) -> Option<Box<dyn GameState>> {
        if controller.is_triggered(INPUT_HARD_DROP) {
            let piece = &self.pieces[self.active_piece.index];

            let drop_pos = self.board.find_drop_position(piece, &self.active_piece.position, self.active_piece.rotation);
            self.board.put_piece(piece, &drop_pos, self.active_piece.rotation);

            if drop_pos != self.active_piece.position {
                let points = piece.iter_coords(self.active_piece.rotation)
                    .map(|(x, y)| { (x + drop_pos.x as usize, y) })
                    .collect::<Vec<_>>();

                let anim = WhooshAnimation::new(
                    points,
                    self.active_piece.position.y,
                    drop_pos.y,
                    piece.color,
                    timestamp,
                    ANIMATION_DURATION_HARD_DROP);

                self.animations.push(Box::new(anim));
            }

            let cleared_lines = self.board.clear_lines();
            if !cleared_lines.is_empty() {
                let anim = LineClearAnimation::new(
                    cleared_lines,
                    self.board.width(),
                    timestamp,
                    ANIMATION_DURATION_LINE_CLEAR,
                );
                self.animations.push(Box::new(anim));
            }

            self.place_new_piece();
        }

        None
    }

    fn input_move(&mut self, controller: &input::Controller) -> Option<Box<dyn GameState>> {
        let piece = &self.pieces[self.active_piece.index];

        let ts_left = controller.get_button_pressed_timestamp(INPUT_MOVE_LEFT);
        let ts_right = controller.get_button_pressed_timestamp(INPUT_MOVE_RIGHT);

        let is_left = controller.is_triggered_or_repeat(INPUT_MOVE_LEFT, INITIAL_DELAY_MOVE, REPEAT_DELAY_MOVE);
        let is_right = controller.is_triggered_or_repeat(INPUT_MOVE_RIGHT, INITIAL_DELAY_MOVE, REPEAT_DELAY_MOVE);

        let offset = {
            if ts_left > ts_right {
                if is_left { -1 } else { 0 }
            } else if ts_right > ts_left {
                if is_right { 1 } else { 0 }
            } else {
                0
            }
        };

        if offset != 0 {
            let new_position = self.active_piece.position.add_x(offset);
            if !self.board.collides(piece, &new_position, self.active_piece.rotation) {
                self.active_piece.position = new_position;
            }
        }

        let is_soft_drop = controller.is_triggered_or_repeat(INPUT_SOFT_DROP, INITIAL_DELAY_SOFT_DROP, REPEAT_DELAY_SOFT_DROP);

        let offset = {
            if is_soft_drop {
                1
            } else {
                0
            }
        };

        if offset != 0 {
            let new_position = self.active_piece.position.add_y(offset);
            if !self.board.collides(piece, &new_position, self.active_piece.rotation) {
                self.active_piece.position = new_position;
            }
        }

        None
    }

    fn input_rotate(&mut self, controller: &input::Controller) -> Option<Box<dyn GameState>> {
        let piece = &self.pieces[self.active_piece.index];

        let is_cw = controller.is_triggered(INPUT_ROTATE_CW);
        let is_ccw = controller.is_triggered(INPUT_ROTATE_CCW);

        let maybe_value = {
            if is_cw && !is_ccw {
                Some(self.active_piece.rotation.wrapping_add(1))
            } else if is_ccw && !is_cw {
                Some(self.active_piece.rotation.wrapping_sub(1))
            } else {
                None
            }
        };

        if let Some(value) = maybe_value {
            for y in 0..2 {
                let pos = self.active_piece.position.add_y(y);
                if !self.board.collides(piece, &pos, value) {
                    self.active_piece.position = pos;
                    self.active_piece.rotation = value;
                    break;
                }
            }
        }

        None
    }
}

impl GameState for GameRunning {
    fn tick(&mut self, timestamp: f64, controller: &input::Controller) -> Option<Box<dyn GameState>> {
        if !self.animations.is_empty() {
            for x in self.animations.iter_mut() {
                x.tick(timestamp);
            }
            self.animations.retain(|x| { !x.is_finished() });

            return None;
        }

        let new_state = None
            .or_else(|| { self.input_misc(controller) })
            .or_else(|| { self.input_drop(timestamp, controller) })
            .or_else(|| { self.input_move(controller) })
            .or_else(|| { self.input_rotate(controller) })
            ;

        if new_state.is_some() {
            return new_state;
        }

        // HACK: Skip drawing of everything else if an animation has just been added.
        if !self.animations.is_empty() {
            return None;
        }

        self.board.draw();

        let piece = &self.pieces[self.active_piece.index];
        let drop_pos = self.board.find_drop_position(piece, &self.active_piece.position, self.active_piece.rotation);

        piece.draw(&drop_pos, self.active_piece.rotation, 0.3);
        piece.draw(&self.active_piece.position, self.active_piece.rotation, 1.0);

        None
    }
}

impl Game {
    fn new(board_width: usize, board_height: usize) -> Self {
        Self {
            frame_index: 0,
            controller: input::Controller::new(),
            game_state: Box::new(GameTitle {
                board_width: board_width,
                board_height: board_height,
            }),
        }
    }

    fn key_handler(&mut self, key_code: i32, state: i32) {
        if state != 0 {
            self.controller.set_button_pressed((0, key_code as usize));
        } else {
            self.controller.set_button_released((0, key_code as usize));
        }
    }

    fn tick(&mut self, timestamp: f64) {
        if let Some(new_state) = self.game_state.tick(timestamp, &self.controller) {
            self.game_state = new_state;
        }

        self.controller.update(timestamp);
        self.frame_index += 1;
    }
}

#[no_mangle]
pub extern fn Game_new(board_width: usize, board_height: usize) -> u32 {
    util::into_address(Game::new(board_width, board_height))
}

#[no_mangle]
pub extern fn Game_key_handler(address: u32, key_code: i32, state: i32) {
    util::with_address_as_mut(address, |x: &mut Game| { x.key_handler(key_code, state) })
}

#[no_mangle]
pub extern fn Game_tick(address: u32, timestamp: f64) {
    util::with_address_as_mut(address, |x: &mut Game| { x.tick(timestamp) })
}
