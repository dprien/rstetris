use std::cell::{RefCell};

use crate::{js_api};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x: x,
            y: y,
        }
    }

    pub fn origin() -> Self {
        Self::new(0, 0)
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

pub fn into_address<T>(obj: T) -> u32 {
    let obj = Box::new(RefCell::new(obj));
    Box::into_raw(obj) as u32
}

pub unsafe fn address_as_refcell<'a, T>(address: u32) -> &'a RefCell<T> {
    let ptr = address as *mut RefCell<T>;
    assert!(!ptr.is_null());
    &*ptr
}

pub fn with_address_as_ref<T, F, R>(address: u32, f: F) -> R
    where F: FnOnce(&T) -> R
{
    let rc = unsafe { address_as_refcell(address) };
    f(&rc.borrow())
}

pub fn with_address_as_mut<T, F, R>(address: u32, f: F) -> R
    where F: FnOnce(&mut T) -> R
{
    let rc = unsafe { address_as_refcell(address) };
    f(&mut rc.borrow_mut())
}

pub fn clamp<T: PartialOrd>(v: T, min: T, max: T) -> T {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

fn random_index_pairs(min: usize, max: usize) -> impl Iterator<Item = (usize, usize)> {
    (1..(max - min)).rev().map(move |i| {
        let j = (js_api::random() * (i + 1) as f64).floor() as usize;
        (i + min, j + min)
    })
}

pub fn shuffle<E, T>(min: usize, max: usize, seq: &mut T)
    where T: std::ops::IndexMut<usize, Output = E>,
          E: Clone
{
    for (i, j) in random_index_pairs(min, max) {
        assert!(i >= min && i < max);
        assert!(j >= min && j < max);

        let tmp = seq[i].clone();
        seq[i] = seq[j].clone();
        seq[j] = tmp;
    }
}

pub fn format_timestamp(timestamp: f64) -> String {
    let hh = (timestamp / 1000.0 / 60.0 / 60.0).floor() as u64;
    let mm = (timestamp / 1000.0 / 60.0).floor() as u64 % 60;
    let ss = (timestamp / 1000.0).floor() as u64 % 60;
    let cs = (timestamp / 10.0).floor() as u64 % 100;

    if hh == 0 {
        format!("{:02}:{:02}.{:02}", mm, ss, cs)
    } else {
        format!("{}:{:02}:{:02}.{:02}", hh, mm, ss, cs)
    }

}
