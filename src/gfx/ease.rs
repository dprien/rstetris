pub fn linear(t: f64) -> f64 {
    t
}

pub fn quadratic_in(t: f64) -> f64 {
    t * t
}

pub fn quadratic_out(t: f64) -> f64 {
    t * (2.0 - t)
}

pub fn cubic_in(t: f64) -> f64 {
    t * t * t
}

pub fn cubic_out(t: f64) -> f64 {
    (t - 1.0) * (t - 1.0) * (t - 1.0) + 1.0
}

pub fn exp_in(t: f64) -> f64 {
    2.0f64.powf(10.0 * (t - 1.0))
}

pub fn exp_out(t: f64) -> f64 {
    1.0 - 2.0f64.powf(-10.0 * t)
}
