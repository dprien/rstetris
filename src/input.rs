use std::collections::{HashMap};

pub struct Controller {
    timestamp_curr: f64,
    timestamp_prev: f64,

    button_map_curr: HashMap<(usize, usize), f64>,
    button_map_prev: HashMap<(usize, usize), f64>,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            timestamp_curr: 0.0,
            timestamp_prev: 0.0,

            button_map_curr: HashMap::new(),
            button_map_prev: HashMap::new(),
        }
    }

    pub fn update(&mut self, timestamp: f64) {
        self.timestamp_prev = self.timestamp_curr;
        self.timestamp_curr = timestamp;

        self.button_map_prev = self.button_map_curr.clone();
    }

    pub fn set_button_pressed(&mut self, input_id: (usize, usize)) {
        if !self.button_map_curr.contains_key(&input_id) {
            self.button_map_curr.insert(input_id, self.timestamp_curr);
        }
    }

    pub fn set_button_released(&mut self, input_id: (usize, usize)) {
        self.button_map_curr.remove(&input_id);
    }

    pub fn get_button_pressed_timestamp(&self, input_id: (usize, usize)) -> Option<f64> {
        self.button_map_curr.get(&input_id).cloned()
    }

    #[allow(dead_code)]
    pub fn is_pressed(&self, input_id: (usize, usize)) -> bool {
        self.button_map_curr.contains_key(&input_id)
    }

    #[allow(dead_code)]
    pub fn is_triggered(&self, input_id: (usize, usize)) -> bool {
        self.button_map_curr.contains_key(&input_id) && !self.button_map_prev.contains_key(&input_id)
    }

    #[allow(dead_code)]
    pub fn is_triggered_or_repeat(&self, input_id: (usize, usize), initial_delay: f64, repeat_delay: f64) -> bool {
        if let Some(press_timestamp) = self.get_button_pressed_timestamp(input_id) {
            if !self.button_map_prev.contains_key(&input_id) {
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
