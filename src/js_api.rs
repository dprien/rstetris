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
