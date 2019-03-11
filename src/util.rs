use std::fmt;
use std::cell::{RefCell};

pub struct LinearInterp {
    start: f64,
    end: f64,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[allow(dead_code)]
impl LinearInterp {
    pub fn new(start: f64, end: f64) -> Self {
        Self {
            start: start,
            end: end,
        }
    }

    pub fn start(&self) -> f64 {
        self.start
    }

    pub fn end(&self) -> f64 {
        self.end
    }

    pub fn t(&self, v: f64) -> f64 {
        let t = (v - self.start) / (self.end - self.start);
        clamp(t, 0.0, 1.0)
    }

    pub fn value(&self, t: f64) -> f64 {
        let value = self.start + t * (self.end - self.start);
        clamp(value, self.start, self.end)
    }
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x: x,
            y: y,
        }
    }

    pub fn add_x(&self, offset: i32) -> Self {
        Self {
            x: self.x.saturating_add(offset),
            y: self.y,
        }
    }

    pub fn add_y(&self, offset: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.saturating_add(offset),
        }
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(x={:0.2},y={:0.2})", self.x, self.y)
    }
}

pub fn into_address<T>(obj: T) -> u32 {
    let obj = Box::new(RefCell::new(obj));
    Box::into_raw(obj) as u32
}

pub unsafe fn address_as_refcell<'a, T>(address: u32) -> &'a RefCell<T> {
    let ptr = address as *mut RefCell<T>;
    assert!(!ptr.is_null());
    &*ptr
}

#[allow(dead_code)]
pub fn with_address_as_ref<T, F, R>(address: u32, f: F) -> R
    where F: FnOnce(&T) -> R
{
    let rc = unsafe { address_as_refcell(address) };
    f(&rc.borrow())
}

#[allow(dead_code)]
pub fn with_address_as_mut<T, F, R>(address: u32, f: F) -> R
    where F: FnOnce(&mut T) -> R
{
    let rc = unsafe { address_as_refcell(address) };
    f(&mut rc.borrow_mut())
}

fn clamp<T: PartialOrd>(v: T, min: T, max: T) -> T {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}
