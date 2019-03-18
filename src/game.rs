use crate::{input, gfx, piece, board, util, js_api};

const BLOCK_SIZE_PX: i32 = 50;

const INPUT_GAME_START: (usize, usize) = (0, 32);
const INPUT_GAME_STOP: (usize, usize) = (0, 27);

const INPUT_HARD_DROP: (usize, usize) = (0, 87);
const INPUT_SOFT_DROP: (usize, usize) = (0, 83);

const INPUT_MOVE_LEFT: (usize, usize) = (0, 65);
const INPUT_MOVE_RIGHT: (usize, usize)= (0, 68);

const INPUT_ROTATE_CW: (usize, usize)= (0, 39);
const INPUT_ROTATE_CCW: (usize, usize) = (0, 37);

const INITIAL_DELAY_SOFT_DROP: f64 = 1000.0 / 60.0 * 4.0;
const REPEAT_DELAY_SOFT_DROP: f64 = 1000.0 / 60.0 * 4.0;

const INITIAL_DELAY_MOVE: f64 = 1000.0 / 60.0 * 12.0;
const REPEAT_DELAY_MOVE: f64 = 1000.0 / 60.0 * 4.0;

const TOUCH_SWIPE_DISTANCE_THRESHOLD: f64 = BLOCK_SIZE_PX as f64 * 2.0;
const TOUCH_TAP_DISTANCE_THRESHOLD: f64 = BLOCK_SIZE_PX as f64 / 2.0;
const TOUCH_TAP_PERIOD_THRESHOLD: f64 = 500.0;

const ANIMATION_DURATION_HARD_DROP: f64 = 200.0;
const ANIMATION_DURATION_LINE_CLEAR: f64 = 600.0;
const ANIMATION_DURATION_GAME_OVER: f64 = 2000.0;

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

    animations: gfx::AnimationQueue,
}

struct GameOverState {
    board_width: usize,
    board_height: usize,

    animations: gfx::AnimationQueue,
}

struct RunningState {
    timestamp_start: f64,
    timestamp_curr: f64,
    timestamp_prev: f64,

    frame_index: u64,

    bag: piece::Bag,
    board: board::Board,

    position: util::Position,
    rotation: usize,

    score: u64,
    num_cleared_lines: u64,
    level: u64,

    animations: gfx::AnimationQueue,
}

struct Game {
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
    fn new(timestamp: f64, board_width: usize, board_height: usize) -> Self {
        let mut animations = gfx::AnimationQueue::new();
        animations.add(Box::new(gfx::TitleAnimation::new(board_width, board_height, timestamp)));

        for y in 0..board_height {
            for x in 0..board_width {
                js_api::draw_block(x as u32, y as i32 as u32, 0x000000);
            }
        }

        js_api::html("top_bar", "<span class = \"title\">Press SPACE or swipe up to start</span>");
        js_api::html("stats", "");

        Self {
            board_width,
            board_height,
            animations,
        }
    }
}

impl State for TitleState {
    fn tick(&mut self, timestamp: f64, controller: &Controller) -> Option<Box<dyn State>> {
        self.animations.update(timestamp);
        if self.animations.should_block() {
            return None;
        }

        if controller.button_input.is_triggered(INPUT_GAME_START) {
            return Some(Box::new(RunningState::new(timestamp, self.board_width, self.board_height)))
        }

        let num_swipes = controller.touch_input.swipes_up(TOUCH_SWIPE_DISTANCE_THRESHOLD).count();
        if num_swipes > 0 {
            return Some(Box::new(RunningState::new(timestamp, self.board_width, self.board_height)))
        }

        None
    }
}

impl GameOverState {
    fn new(timestamp: f64, board_width: usize, board_height: usize) -> Self {
        let mut animations = gfx::AnimationQueue::new();
        animations.add(Box::new(gfx::GameOverAnimation::new(board_width, board_height, timestamp, ANIMATION_DURATION_GAME_OVER)));

        js_api::html("top_bar", "<span class = \"game-over\">GAME OVER</span>");

        Self {
            board_width,
            board_height,
            animations,
        }
    }
}

impl State for GameOverState {
    fn tick(&mut self, timestamp: f64, controller: &Controller) -> Option<Box<dyn State>> {
        self.animations.update(timestamp);
        if self.animations.should_block() {
            return None;
        }

        let is_start = controller.button_input.is_triggered(INPUT_GAME_START);
        let is_stop = controller.button_input.is_triggered(INPUT_GAME_STOP);

        if is_start || is_stop {
            return Some(Box::new(TitleState::new(timestamp, self.board_width, self.board_height)));
        }

        let num_swipes = controller.touch_input.swipes_up(TOUCH_SWIPE_DISTANCE_THRESHOLD).count();
        if num_swipes > 0 {
            return Some(Box::new(TitleState::new(timestamp, self.board_width, self.board_height)));
        }

        None
    }
}

impl RunningState {
    fn new(timestamp: f64, board_width: usize, board_height: usize) -> Self {
        js_api::html("top_bar", "");

        let board = board::Board::new(board_width, board_height);
        let bag = piece::Bag::new(piece::make_standard());
        let rotation = 0;
        let position = board.initial_position(bag.current(), rotation);

        Self {
            timestamp_start: timestamp,
            timestamp_curr: timestamp,
            timestamp_prev: timestamp,

            frame_index: 0,

            board,
            bag,

            position,
            rotation,

            score: 0,
            num_cleared_lines: 0,
            level: 0,

            animations: gfx::AnimationQueue::new(),
        }
    }

    fn game_over(&self) -> Box<dyn State> {
        return Box::new(GameOverState::new(self.timestamp_curr, self.board.width(), self.board.height()));
    }

    fn new_piece(&mut self) -> bool {
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

    fn place_piece(&mut self) {
        let piece = self.bag.current();
        self.board.put_piece(piece, &self.position, self.rotation);

        let cleared_lines = self.board.clear_lines();
        if !cleared_lines.is_empty() {
            self.score += 100 * (1 << (cleared_lines.len() - 1));
            self.num_cleared_lines += cleared_lines.len() as u64;

            let anim = gfx::LineClearAnimation::new(
                cleared_lines,
                self.board.width(),
                self.timestamp_curr,
                ANIMATION_DURATION_LINE_CLEAR,
            );
            self.animations.add(Box::new(anim));
        }

        self.new_piece();
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

    fn hard_drop_piece(&mut self) {
        let piece = self.bag.current();

        let drop_pos = self.board.find_drop_position(piece, &self.position, self.rotation);
        if drop_pos != self.position {
            let anim = gfx::WhooshAnimation::new(
                piece.iter_coords(self.rotation),
                piece.color.clone(),
                self.position.x,
                self.position.y,
                drop_pos.y,
                self.timestamp_curr,
                ANIMATION_DURATION_HARD_DROP);

            self.animations.add(Box::new(anim));
        }

        self.position = drop_pos;
        self.place_piece();
    }

    fn handle_input_misc(&mut self, controller: &Controller) -> Option<Box<dyn State>> {
        if controller.button_input.is_triggered(INPUT_GAME_STOP) {
            self.board.clear();
            return Some(self.game_over());
        }

        None
    }

    fn handle_input_drop(&mut self, controller: &Controller) -> Option<Box<dyn State>> {
        if controller.button_input.is_triggered(INPUT_HARD_DROP) {
            self.hard_drop_piece();
        }

        let num_swipes = controller.touch_input.swipes_up(TOUCH_SWIPE_DISTANCE_THRESHOLD).count();
        if num_swipes > 0 {
            self.hard_drop_piece();
        }

        None
    }

    fn handle_input_move(&mut self, controller: &Controller) -> Option<Box<dyn State>> {
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

    fn handle_input_rotate(&mut self, controller: &Controller) -> Option<Box<dyn State>> {
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

    fn handle_input(&mut self, controller: &Controller) -> Option<Box<dyn State>> {
        None
            .or_else(|| { self.handle_input_misc(&controller) })
            .or_else(|| { self.handle_input_drop(&controller) })
            .or_else(|| { self.handle_input_move(&controller) })
            .or_else(|| { self.handle_input_rotate(&controller) })
    }

    fn output_stats(&self) {
        let text = format!(
            r#"
            <div>
                <span class = "name">TIME</span>
                <span class = "value">{}</span>
            </div>
            <div>
                <span class = "name">SCORE</span>
                <span class = "value">{}</span>
            </div>
            <div>
                <span class = "name">LINES</span>
                <span class = "value">{}</span>
            </div>
            <div>
                <span class = "name">LEVEL</span>
                <span class = "value">{}</span>
            </div>
            "#,
            util::format_timestamp(self.timestamp_curr - self.timestamp_start),
            self.score,
            self.num_cleared_lines,
            self.level,
        );

        js_api::html("stats", text);
    }

    fn update(&mut self, controller: &Controller) -> Option<Box<dyn State>> {
        self.level = 1 + ((self.timestamp_curr - self.timestamp_start) / 60000.0) as u64;

        if self.frame_index % 2 == 0 {
            self.output_stats();
        }

        self.animations.update(self.timestamp_curr);
        if self.animations.should_block() {
            return None;
        }

        let new_state = self.handle_input(controller);
        if new_state.is_some() {
            return new_state;
        }

        if self.animations.should_block() {
            return None;
        }

        self.board.draw();

        let piece = self.bag.current();
        let drop_pos = self.board.find_drop_position(piece, &self.position, self.rotation);

        piece.draw(&drop_pos, self.rotation, 0.4);
        piece.draw(&self.position, self.rotation, 1.0);

        None
    }
}

impl State for RunningState {
    fn tick(&mut self, timestamp: f64, controller: &Controller) -> Option<Box<dyn State>> {
        self.timestamp_prev = self.timestamp_curr;
        self.timestamp_curr = timestamp;

        let new_state = self.update(controller);
        self.frame_index += 1;

        new_state
    }
}

impl Game {
    fn new(timestamp: f64, board_width: usize, board_height: usize) -> Self {
        Self {
            controller: Controller::new(),
            state: Box::new(TitleState::new(timestamp, board_width, board_height)),
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
    }
}

#[no_mangle]
pub extern fn Game_new(timestamp: f64, board_width: usize, board_height: usize) -> u32 {
    util::into_address(Game::new(timestamp, board_width, board_height))
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
