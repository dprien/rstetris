use std::collections::{HashMap};

use crate::{util};

pub struct ButtonInput {
    timestamp_curr: f64,
    timestamp_prev: f64,

    state_curr: HashMap<(usize, usize), f64>,
    state_prev: HashMap<(usize, usize), f64>,
}

#[derive(Clone, Debug)]
pub struct Touch {
    pub position: util::Position,
    pub timestamp: f64,
}

pub struct TouchInput {
    timestamp: f64,
    finished: HashMap<i32, (Touch, Touch)>,

    active_curr: HashMap<i32, (Touch, Touch)>,
    active_prev: HashMap<i32, (Touch, Touch)>,
}

impl ButtonInput {
    pub fn new() -> Self {
        Self {
            timestamp_curr: 0.0,
            timestamp_prev: 0.0,

            state_curr: HashMap::new(),
            state_prev: HashMap::new(),
        }
    }

    pub fn update(&mut self, timestamp: f64) {
        self.timestamp_prev = self.timestamp_curr;
        self.timestamp_curr = timestamp;

        self.state_prev = self.state_curr.clone();
    }

    pub fn button_press(&mut self, input_id: (usize, usize)) {
        if !self.state_curr.contains_key(&input_id) {
            self.state_curr.insert(input_id, self.timestamp_curr);
        }
    }

    pub fn button_release(&mut self, input_id: (usize, usize)) {
        self.state_curr.remove(&input_id);
    }

    pub fn get_button_press_timestamp(&self, input_id: (usize, usize)) -> Option<f64> {
        self.state_curr.get(&input_id).cloned()
    }

    pub fn is_pressed(&self, input_id: (usize, usize)) -> bool {
        self.state_curr.contains_key(&input_id)
    }

    pub fn is_triggered(&self, input_id: (usize, usize)) -> bool {
        self.state_curr.contains_key(&input_id) && !self.state_prev.contains_key(&input_id)
    }

    pub fn is_triggered_or_repeat(&self, input_id: (usize, usize), initial_delay: f64, repeat_delay: f64) -> bool {
        if let Some(press_timestamp) = self.get_button_press_timestamp(input_id) {
            if !self.state_prev.contains_key(&input_id) {
                return true;
            }

            let initial_prev = self.timestamp_prev - press_timestamp - initial_delay;
            let initial_curr = self.timestamp_curr - press_timestamp - initial_delay;

            if initial_curr >= 0.0 {
                if initial_prev < 0.0 {
                    return true;
                }

                let repeat_prev = (initial_prev / repeat_delay).floor();
                let repeat_curr = (initial_curr / repeat_delay).floor();
                if repeat_curr > repeat_prev {
                    return true;
                }
            }
        }

        false
    }
}

impl Touch {
    fn new(position: util::Position, timestamp: f64) -> Self {
        Self {
            position: position,
            timestamp: timestamp,
        }
    }
}

impl TouchInput {
    pub fn new() -> Self {
        Self {
            timestamp: 0.0,
            finished: HashMap::new(),

            active_curr: HashMap::new(),
            active_prev: HashMap::new(),
        }
    }

    fn get_distance(start: &Touch, end: &Touch) -> f64 {
        let (p1, p2) = (start.position, end.position);
        let (dx, dy) = (p2.x - p1.x, p2.y - p1.y);
        ((dx * dx + dy * dy) as f64).sqrt()
    }

    fn is_swipe(start: &Touch, end: &Touch, min_distance: f64) -> bool {
        Self::get_distance(&start, &end) >= min_distance
    }

    fn is_tap(start: &Touch, end: &Touch, max_distance: f64, max_period: f64) -> bool {
        let distance = Self::get_distance(&start, &end);
        let period = end.timestamp - start.timestamp;
        distance < max_distance && period < max_period
    }

    pub fn update(&mut self, timestamp: f64) {
        self.timestamp = timestamp;
        self.finished.clear();

        self.active_prev = self.active_curr.clone();
    }

    pub fn touch_start(&mut self, touch_id: i32, x: i32, y: i32) {
        let touch = Touch::new(util::Position::new(x, y), self.timestamp);
        self.active_curr.insert(touch_id, (touch.clone(), touch));
    }

    pub fn touch_end(&mut self, touch_id: i32, x: i32, y: i32) {
        if let Some((start, _)) = self.active_curr.remove(&touch_id) {
            let end = Touch::new(util::Position::new(x, y), self.timestamp);
            self.finished.insert(touch_id, (start, end));
        }
    }

    pub fn touch_cancel(&mut self, touch_id: i32, _: i32, _: i32) {
        self.active_curr.remove(&touch_id);
    }

    pub fn touch_move(&mut self, touch_id: i32, x: i32, y: i32) {
        if let Some((_, end)) = self.active_curr.get_mut(&touch_id) {
            end.position = util::Position::new(x, y);
            end.timestamp = self.timestamp;
        }
    }

    pub fn swipes(&self, min_distance: f64) -> impl Iterator<Item = (&i32, &(Touch, Touch))> {
        self.finished.iter().filter(move |(_, (start, end))| {
            Self::is_swipe(start, end, min_distance)
        })
    }

    fn swipes_filter<F>(&self, min_distance: f64, mut f: F) -> impl Iterator<Item = (&i32, &(Touch, Touch))>
        where F: FnMut(i32, i32) -> bool
    {
        self.swipes(min_distance)
            .filter(move |(_, (start, end))| {
                f(end.position.x - start.position.x, end.position.y - start.position.y)
            })
    }

    pub fn swipes_left(&self, min_distance: f64) -> impl Iterator<Item = (&i32, &(Touch, Touch))> {
        self.swipes_filter(min_distance, move |dx, dy| {
            dx < 0 && dx.abs() > dy.abs() && dy.abs() < (min_distance / 2.0) as i32
        })
    }

    pub fn swipes_right(&self, min_distance: f64) -> impl Iterator<Item = (&i32, &(Touch, Touch))> {
        self.swipes_filter(min_distance, move |dx, dy| {
            dx > 0 && dx.abs() > dy.abs() && dy.abs() < (min_distance / 2.0) as i32
        })
    }

    pub fn swipes_up(&self, min_distance: f64) -> impl Iterator<Item = (&i32, &(Touch, Touch))> {
        self.swipes_filter(min_distance, move |dx, dy| {
            dy < 0 && dy.abs() > dx.abs() && dx.abs() < (min_distance / 2.0) as i32
        })
    }

    pub fn swipes_down(&self, min_distance: f64) -> impl Iterator<Item = (&i32, &(Touch, Touch))> {
        self.swipes_filter(min_distance, move |dx, dy| {
            dy > 0 && dy.abs() > dx.abs() && dx.abs() < (min_distance / 2.0) as i32
        })
    }

    pub fn taps(&self, max_distance: f64, max_period: f64) -> impl Iterator<Item = (&i32, &(Touch, Touch))> {
        self.finished.iter().filter(move |(_, (start, end))| {
            Self::is_tap(start, end, max_distance, max_period)
        })
    }

    pub fn motions(&self) -> impl Iterator<Item = (&i32, (&Touch, &Touch, &Touch))> {
        self.active_curr.iter()
            .filter_map(move |(touch_id, (start, end_curr))| {
                self.active_prev.get(touch_id).map(|(_, end_prev)| {
                    (touch_id, (start, end_prev, end_curr))
                })
            })
    }
}
