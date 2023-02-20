use kuma::{
    res::sprite::{Dire, Player},
    FPS,
};
use minifb::Window;
use std::thread::sleep_ms;

fn main() {
    let (width, height) = (1000_usize, 1000_usize);

    let windowoptions = minifb::WindowOptions {
        borderless: false,
        transparency: false,
        resize: false,
        topmost: false,
        title: true,
        none: true,
        scale_mode: minifb::ScaleMode::Center,
        scale: minifb::Scale::X1,
    };

    let mut window = Window::new("kuma", width, height, windowoptions).unwrap();

    // ==========================================
    // p1
    let mut p1 = Player::new(Dire::Right, true);
    p1.load_stream("stop");
    p1.load_stream("walk");
    p1.load_stream("run");
    p1.x_offset = 0;
    p1.y_offset = 0;

    // p2
    let mut p2 = p1.clone();
    p1.is_p1 = false;
    p2.dire = Dire::Left;
    p2.x_offset = width as u32 - 300;

    // ==========================================
    // init
    window.update();
    let bg = &vec![123 * 123 * 123; width * height]; // background
    let mut buffer = bg.clone();
    // ==========================================
    // display
    'l1: while window.is_open() {
        buffer = bg.clone();

        // p1
        p1.check_keys(&window);
        p1.next_frame();
        p1.flush_buffer(&mut buffer, 0, width as u32);

        // p2
        p2.check_keys(&window);
        p2.next_frame();
        p2.flush_buffer(&mut buffer, 0, width as u32);

        window.update_with_buffer(&buffer, width, height).unwrap();

        sleep_ms(FPS);
    }
    // ==========================================
}

#[derive(Debug)]
struct Rect {}

#[derive(Debug)]
struct Buffer {
    timer: u32,
}
