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

extern {
    #[link_name = "console_log"]
    fn _js_console_log(address: u32, length: u32);

    #[link_name = "draw_block"]
    fn _js_draw_block(x: u32, y: u32, color: u32);

    #[link_name = "random"]
    fn _js_random() -> f64;

    #[link_name = "html"]
    fn _js_html(id_address: u32, id_length: u32, html_address: u32, html_length: u32);
}

pub fn console_log<T>(s: T)
    where T: AsRef<str>
{
    let s = s.as_ref();
    let length = s.len();
    unsafe { _js_console_log(s.as_ptr() as u32, length as u32) };
}

pub fn draw_block(x: u32, y: u32, color: u32) {
    unsafe { _js_draw_block(x, y, color) };
}

pub fn random() -> f64 {
    unsafe { _js_random() }
}

pub fn html<T, U>(id: T, html: U)
    where T: AsRef<str>,
          U: AsRef<str>
{
    let id = id.as_ref();
    let id_address = id.as_ptr() as u32;
    let id_length = id.len() as u32;

    let html = html.as_ref();
    let html_address = html.as_ptr() as u32;
    let html_length = html.len() as u32;

    unsafe { _js_html(id_address, id_length, html_address, html_length) };
}
