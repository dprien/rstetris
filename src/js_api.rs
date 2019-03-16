extern {
    #[link_name = "console_log"]
    fn _js_console_log(address: u32, length: u32);

    #[link_name = "draw_block"]
    fn _js_draw_block(x: u32, y: u32, color: u32);

    #[link_name = "random"]
    fn _js_random() -> f64;
}

#[allow(dead_code)]
pub fn console_log<T>(s: T)
    where T: AsRef<str>
{
    let s = s.as_ref();
    let length = s.len();
    unsafe { _js_console_log(s.as_ptr() as u32, length as u32) };
}

#[allow(dead_code)]
pub fn draw_block(x: u32, y: u32, color: u32) {
    unsafe { _js_draw_block(x, y, color) };
}

pub fn random() -> f64 {
    unsafe { _js_random() }
}
