use std::cell::{RefCell};

const GLOBAL_STACK_SIZE: usize = 16;

static mut GLOBAL_STACK: Stack = Stack { index: 0, data: StackData([0; GLOBAL_STACK_SIZE]) };

#[repr(align(8))]
struct StackData([u32; GLOBAL_STACK_SIZE]);

pub struct Stack {
    index: usize,
    data: StackData,
}

impl Stack {
    fn push(&mut self, value: u32) {
        assert!(self.index < self.data.0.len());
        self.data.0[self.index] = value;
        self.index += 1;
    }

    fn pop(&mut self) -> u32 {
        assert!(self.index > 0);
        self.index -= 1;

        self.data.0[self.index]
    }
}

#[no_mangle]
pub unsafe extern fn stack_push(value: u32) {
    GLOBAL_STACK.push(value);
}

#[no_mangle]
pub unsafe extern fn stack_pop() -> u32 {
    GLOBAL_STACK.pop()
}

pub unsafe fn stack_pop_vec() -> Vec<u8> {
    let (size, address) = (stack_pop(), stack_pop());
    Vec::from_raw_parts(address as *mut u8, size as usize, size as usize)
}

pub unsafe fn stack_pop_string() -> String {
    String::from_utf8_unchecked(stack_pop_vec())
}

#[no_mangle]
pub extern fn alloc(size: u32) -> u32 {
    assert!(size > 0);
    let mut v: Vec<u8> = Vec::with_capacity(size as usize);
    let address = v.as_mut_ptr() as u32;
    std::mem::forget(v);

    address
}

#[no_mangle]
pub unsafe extern fn dealloc(address: u32, size: u32) {
    assert!(size > 0);
    let v = Vec::from_raw_parts(address as *mut u8, size as usize, size as usize);
    std::mem::drop(v);
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
