use crate::{input, gfx, piece, board, util};

const BLOCK_SIZE_PX: i32 = 40;

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

const TOUCH_SWIPE_DISTANCE_THRESHOLD: f64 = BLOCK_SIZE_PX as f64 * 2.0;
const TOUCH_TAP_DISTANCE_THRESHOLD: f64 = BLOCK_SIZE_PX as f64 / 2.0;
const TOUCH_TAP_PERIOD_THRESHOLD: f64 = 500.0;

trait State {
    fn tick(&mut self, timestamp: f64, controller: &Controller) -> Option<Box<dyn State>>;
}

struct Controller {
    button_input: input::ButtonInput,
    touch_input: input::TouchInput,
}

struct TitleState {
    board_width: usize,
    board_height: usize,
}

struct RunningState {
    bag: piece::Bag,
    board: board::Board,
    position: util::Position,
    rotation: usize,
    animations: Vec<Box<dyn gfx::Animation>>,
}

struct Game {
    frame_index: u64,
    controller: Controller,
    state: Box<dyn State>,
}

impl Controller {
    fn new() -> Self {
        Self {
            button_input: input::ButtonInput::new(),
            touch_input: input::TouchInput::new(),
        }
    }
}

impl TitleState {
    fn new(board_width: usize, board_height: usize) -> Self {
        Self {
            board_width: board_width,
            board_height: board_height,
        }
    }
}

impl State for TitleState {
    fn tick(&mut self, _timestamp: f64, controller: &Controller) -> Option<Box<dyn State>> {
        if controller.button_input.is_triggered(INPUT_GAME_START) {
            return Some(Box::new(RunningState::new(self.board_width, self.board_height)))
        }

        let num_swipes = controller.touch_input.swipes_up(TOUCH_SWIPE_DISTANCE_THRESHOLD).count();
        if num_swipes > 0 {
            return Some(Box::new(RunningState::new(self.board_width, self.board_height)))
        }

        None
    }
}

impl RunningState {
    fn new(board_width: usize, board_height: usize) -> Self {
        let board = board::Board::new(board_width, board_height);
        let bag = piece::Bag::new(piece::make_standard());
        let rotation = 0;
        let position = board.initial_position(bag.current(), rotation);

        Self {
            board,
            bag,
            position,
            rotation,
            animations: Vec::new()
        }
    }

    fn place_new_piece(&mut self) -> bool {
        self.bag.advance();

        let rotation = 0;
        let position = self.board.initial_position(self.bag.current(), rotation);

        if self.board.collides(self.bag.current(), &position, rotation) {
            false
        } else {
            self.rotation = rotation;
            self.position = position;
            true
        }
    }

    fn move_piece_x(&mut self, offset: i32) {
        let step = {
            if offset > 0 {
                1
            } else if offset < 0 {
                -1
            } else {
                return;
            }
        };

        let piece = self.bag.current();
        for _ in 0..offset.abs() {
            let new_position = self.position.add_x(step);
            if self.board.collides(piece, &new_position, self.rotation) {
                break;
            }
            self.position = new_position;
        }
    }

    fn move_piece_y(&mut self, offset: i32) {
        if offset <= 0 {
            return;
        }

        let piece = self.bag.current();
        for _ in 0..offset.abs() {
            let new_position = self.position.add_y(1);
            if self.board.collides(piece, &new_position, self.rotation) {
                break;
            }
            self.position = new_position;
        }
    }

    fn rotate_piece(&mut self, offset: i32) {
        if offset == 0 {
            return;
        }

        let piece = self.bag.current();
        for _ in 0..offset.abs() {
            let new_rotation = {
                if offset > 0 {
                    self.rotation.wrapping_add(1)
                } else {
                    self.rotation.wrapping_sub(1)
                }
            };

            'outer:
            for &y in &[0, 1, 2] {
                for &x in &[0, 1, -1, 2, -2] {
                    let pos = util::Position::new(self.position.x + x, self.position.y + y);
                    if !self.board.collides(piece, &pos, new_rotation) {
                        self.position = pos;
                        self.rotation = new_rotation;
                        break 'outer;
                    }
                }
            }
        }
    }

    fn hard_drop_piece(&mut self, timestamp: f64) {
        let piece = self.bag.current();

        let drop_pos = self.board.find_drop_position(piece, &self.position, self.rotation);
        self.board.put_piece(piece, &drop_pos, self.rotation);

        if drop_pos != self.position {
            let points = piece.iter_coords(self.rotation)
                .map(|(x, y)| { (x + drop_pos.x as usize, y) })
                .collect::<Vec<_>>();

            let anim = gfx::WhooshAnimation::new(
                points,
                self.position.y,
                drop_pos.y,
                piece.color.clone(),
                timestamp,
                ANIMATION_DURATION_HARD_DROP);

            self.animations.push(Box::new(anim));
        }

        let cleared_lines = self.board.clear_lines();
        if !cleared_lines.is_empty() {
            let anim = gfx::LineClearAnimation::new(
                cleared_lines,
                self.board.width(),
                timestamp,
                ANIMATION_DURATION_LINE_CLEAR,
            );
            self.animations.push(Box::new(anim));
        }

        self.place_new_piece();
    }

    fn handle_input_misc(&mut self, _timestamp: f64, controller: &Controller) -> Option<Box<dyn State>> {
        if controller.button_input.is_triggered(INPUT_STOP_GAME) {
            self.board.clear();
            return Some(Box::new(TitleState::new(self.board.width(), self.board.height())));
        }

        None
    }

    fn handle_input_drop(&mut self, timestamp: f64, controller: &Controller) -> Option<Box<dyn State>> {
        if controller.button_input.is_triggered(INPUT_HARD_DROP) {
            self.hard_drop_piece(timestamp);
        }

        let num_swipes = controller.touch_input.swipes_up(TOUCH_SWIPE_DISTANCE_THRESHOLD).count();
        if num_swipes > 0 {
            self.hard_drop_piece(timestamp);
        }

        None
    }

    fn handle_input_move(&mut self, _timestamp: f64, controller: &Controller) -> Option<Box<dyn State>> {
        let ts_left = controller.button_input.get_button_press_timestamp(INPUT_MOVE_LEFT);
        let ts_right = controller.button_input.get_button_press_timestamp(INPUT_MOVE_RIGHT);

        let is_left = controller.button_input.is_triggered_or_repeat(INPUT_MOVE_LEFT, INITIAL_DELAY_MOVE, REPEAT_DELAY_MOVE);
        let is_right = controller.button_input.is_triggered_or_repeat(INPUT_MOVE_RIGHT, INITIAL_DELAY_MOVE, REPEAT_DELAY_MOVE);

        if ts_left > ts_right {
            if is_left {
                self.move_piece_x(-1)
            }
        } else if ts_right > ts_left {
            if is_right {
                self.move_piece_x(1)
            }
        }

        let is_soft_drop = controller.button_input.is_triggered_or_repeat(INPUT_SOFT_DROP, INITIAL_DELAY_SOFT_DROP, REPEAT_DELAY_SOFT_DROP);
        if is_soft_drop {
            self.move_piece_y(1);
        }

        for (_, (start, prev, curr)) in controller.touch_input.motions() {
            let x_offset = ((curr.position.x - start.position.x) / BLOCK_SIZE_PX) - ((prev.position.x - start.position.x) / BLOCK_SIZE_PX);
            let y_offset = ((curr.position.y - start.position.y) / BLOCK_SIZE_PX) - ((prev.position.y - start.position.y) / BLOCK_SIZE_PX);

            if x_offset != 0 || y_offset != 0 {
                self.move_piece_x(x_offset);
                self.move_piece_y(y_offset);
            }
        }

        None
    }

    fn handle_input_rotate(&mut self, _timestamp: f64, controller: &Controller) -> Option<Box<dyn State>> {
        let is_cw = controller.button_input.is_triggered(INPUT_ROTATE_CW);
        let is_ccw = controller.button_input.is_triggered(INPUT_ROTATE_CCW);

        if is_cw && !is_ccw {
            self.rotate_piece(1);
        } else if is_ccw && !is_cw {
            self.rotate_piece(-1);
        }

        let num_taps = controller.touch_input.taps(TOUCH_TAP_DISTANCE_THRESHOLD, TOUCH_TAP_PERIOD_THRESHOLD).count();
        if num_taps > 0 {
            self.rotate_piece(1);
        }

        None
    }

    fn handle_input(&mut self, timestamp: f64, controller: &Controller) -> Option<Box<dyn State>> {
        None
            .or_else(|| { self.handle_input_misc(timestamp, &controller) })
            .or_else(|| { self.handle_input_drop(timestamp, &controller) })
            .or_else(|| { self.handle_input_move(timestamp, &controller) })
            .or_else(|| { self.handle_input_rotate(timestamp, &controller) })
    }
}

impl State for RunningState {
    fn tick(&mut self, timestamp: f64, controller: &Controller) -> Option<Box<dyn State>> {
        if !self.animations.is_empty() {
            for x in self.animations.iter_mut() {
                x.tick(timestamp);
            }
            self.animations.retain(|x| { x.is_active() });

            return None;
        }

        let new_state = self.handle_input(timestamp, controller);
        if new_state.is_some() {
            return new_state;
        }

        // HACK: Skip drawing of everything else if an animation has just been added.
        if !self.animations.is_empty() {
            return None;
        }

        self.board.draw();

        let piece = self.bag.current();
        let drop_pos = self.board.find_drop_position(piece, &self.position, self.rotation);

        piece.draw(&drop_pos, self.rotation, 0.3);
        piece.draw(&self.position, self.rotation, 1.0);

        None
    }
}

impl Game {
    fn new(board_width: usize, board_height: usize) -> Self {
        Self {
            frame_index: 0,
            controller: Controller::new(),
            state: Box::new(TitleState::new(board_width, board_height)),
        }
    }

    fn key_handler(&mut self, key_code: i32, state: i32) {
        if state != 0 {
            self.controller.button_input.button_press((0, key_code as usize));
        } else {
            self.controller.button_input.button_release((0, key_code as usize));
        }
    }

    fn touch_start_handler(&mut self, touch_id: i32, x: i32, y: i32) {
        self.controller.touch_input.touch_start(touch_id, x, y);
    }

    fn touch_end_handler(&mut self, touch_id: i32, x: i32, y: i32) {
        self.controller.touch_input.touch_end(touch_id, x, y);
    }

    fn touch_cancel_handler(&mut self, touch_id: i32, x: i32, y: i32) {
        self.controller.touch_input.touch_cancel(touch_id, x, y);
    }

    fn touch_move_handler(&mut self, touch_id: i32, x: i32, y: i32) {
        self.controller.touch_input.touch_move(touch_id, x, y);
    }

    fn tick(&mut self, timestamp: f64) {
        if let Some(new_state) = self.state.tick(timestamp, &self.controller) {
            self.state = new_state;
        }

        self.controller.button_input.update(timestamp);
        self.controller.touch_input.update(timestamp);

        self.frame_index += 1;
    }
}

#[no_mangle]
pub extern fn Game_new(board_width: usize, board_height: usize) -> u32 {
    util::into_address(Game::new(board_width, board_height))
}

#[no_mangle]
pub extern fn Game_key_handler(address: u32, key_code: i32, state: i32) {
    util::with_address_as_mut(address, |o: &mut Game| { o.key_handler(key_code, state) })
}

#[no_mangle]
pub extern fn Game_touch_start_handler(address: u32, touch_id: i32, x: i32, y: i32) {
    util::with_address_as_mut(address, |o: &mut Game| { o.touch_start_handler(touch_id, x, y) })
}

#[no_mangle]
pub extern fn Game_touch_end_handler(address: u32, touch_id: i32, x: i32, y: i32) {
    util::with_address_as_mut(address, |o: &mut Game| { o.touch_end_handler(touch_id, x, y) })
}

#[no_mangle]
pub extern fn Game_touch_cancel_handler(address: u32, touch_id: i32, x: i32, y: i32) {
    util::with_address_as_mut(address, |o: &mut Game| { o.touch_cancel_handler(touch_id, x, y) })
}

#[no_mangle]
pub extern fn Game_touch_move_handler(address: u32, touch_id: i32, x: i32, y: i32) {
    util::with_address_as_mut(address, |o: &mut Game| { o.touch_move_handler(touch_id, x, y) })
}

#[no_mangle]
pub extern fn Game_tick(address: u32, timestamp: f64) {
    util::with_address_as_mut(address, |o: &mut Game| { o.tick(timestamp) })
}
