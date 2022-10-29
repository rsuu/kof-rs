use image;
use image::io::Reader as ImageReader;
use std::io::Cursor;

use minifb::{Key, Window};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (width, height) = (1000_usize, 1000_usize);

    let windowoptions = minifb::WindowOptions {
        borderless: false,
        transparency: false,
        title: true,
        resize: false,
        topmost: false,
        none: true,
        scale_mode: minifb::ScaleMode::Center,
        scale: minifb::Scale::X1,
    };

    let mut window = Window::new("rmg", width, height, windowoptions).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_millis(
        args[1].parse::<u64>().unwrap(),
    )));

    // ==========================================
    // background
    window.update();

    // ==========================================
    // p1
    let mut p1 = Hero::new(Vec::new(), 30, true);
    p1.load_frames();
    p1.x_offset = 0;
    p1.y_offset = 0;

    // ==========================================
    // display
    'l1: while window.is_open() {
        match window.get_keys().as_slice() {
            &[Key::W] => {}
            &[Key::S] => {}

            &[Key::A] => {}
            &[Key::D] => {
                p1.move_right();
            }

            &[Key::Q] => {
                std::process::exit(0);
            }
            _ => {}
        }

        // background
        let bg = &vec![0; 1000 * 1000];

        // p1
        let buffer = flush_buffer(&bg, p1.get_frame(), p1.x_offset, 0, width as u32, 864);
        p1.next_frame();

        // p2

        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}

#[derive(Debug)]
struct FramesInfo {
    data: Vec<Vec<u32>>,
    frame: u32,
    frame_group: u32,
}

#[derive(Debug)]
struct Hero {
    // v_wait:FramesInfo,
    // v_run:FramesInfo,
    frames: Vec<Vec<u32>>,
    frame: u32,
    frame_group: u32,
    hp: u32,
    ep: u32,

    block_body: Block,
    block_head: Block,
    block_hand: Block,
    block_leg: Block,

    x_offset: u32,
    y_offset: u32,
    is_p1: bool,
    state: HeroState,
}

#[derive(Debug, Default)]
enum HeroState {
    #[default]
    Wait,

    Walk,
    Run,
}

#[derive(Debug)]
enum BlockChecker {
    Up,
    Down,
    Left,
    Right,
    Unknown,
}

#[derive(Debug)]
struct Block {
    x1: u32,
    x2: u32,
    y1: u32,
    y2: u32,
}

impl Hero {
    fn new(frames: Vec<Vec<u32>>, frame_group: u32, is_p1: bool) -> Self {
        Self {
            frames,
            frame: 0,
            frame_group,
            hp: 10000,
            ep: 10000,
            block_body: Block::new(0, 0, 100, 100),
            block_head: Block::new(10, 10, 10, 10),
            block_hand: Block::new(10, 10, 10, 10),
            block_leg: Block::new(10, 10, 10, 10),

            x_offset: 0,
            y_offset: 0,
            is_p1,
            state: HeroState::default(),
        }
    }

    fn load_frames(&mut self) {
        for nums in 1..=self.frame_group as usize {
            let img = ImageReader::open(format!("./Athena/Wait/right{}.png", nums))
                .unwrap()
                .decode()
                .unwrap();
            let mut img = img.to_rgb8().to_vec();
            let mut buffer = Vec::new();

            for f in (0..img.len()).step_by(3) {
                buffer.push(rgb_as_u32(&img[f..f + 3].try_into().unwrap()));
            }

            self.frames.push(buffer);
        }
    }

    fn get_frame(&self) -> &[u32] {
        &self.frames[self.frame as usize]
    }

    fn next_frame(&mut self) {
        if self.frame + 1 == self.frame_group {
            self.frame = 0;
        } else {
            self.frame += 1;
        }
    }

    fn try_move(&mut self, block_checker: &BlockChecker) {
        match block_checker {
            _ => {
                // doing nothing
            }
        }
    }

    fn move_left(&mut self) {}
    fn move_right(&mut self) {
        if self.x_offset > 200 * 4 {
            self.x_offset = 0;
        } else {
            self.x_offset += 4;
        }
    }
    fn move_up(&mut self) {}
    fn move_down(&mut self) {}
}

impl Block {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x1: x,
            x2: x + width,
            y1: y,
            y2: y + height,
        }
    }

    fn gen_body(&self, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x1: self.x1 + x,
            x2: self.x2 + x + width,
            y1: self.y1 + y,
            y2: self.y2 + y + height,
        }
    }

    fn check(&self, block2: &Block) -> BlockChecker {
        // x1 --- x2    x1 --- x2    x1 --- x2
        // |       |    |       |    |       |
        // |       |    |       |    |       |
        // |       |    |       |    |       |
        // y1 --- y2    y1 --- y2    y1 --- y2
        //
        // x1 --- x2    x1 --- x2    x1 --- x2
        // |       |    |.......|    |       |
        // |       |    |.......|    |       |
        // |       |    |.......|    |       |
        // y1 --- y2    y1 --- y2    y1 --- y2
        //
        // x1 --- x2    x1 --- x2    x1 --- x2
        // |       |    |       |    |       |
        // |       |    |       |    |       |
        // |       |    |       |    |       |
        // y1 --- y2    y1 --- y2    y1 --- y2

        if self.x2 <= block2.x1 {
            BlockChecker::Right
        } else if self.x1 <= block2.x2 {
            BlockChecker::Left
        } else if self.x1 <= block2.y1 {
            BlockChecker::Up
        } else if self.y1 >= block2.x1 {
            BlockChecker::Down
        } else {
            BlockChecker::Unknown
        }
    }
}

#[inline(always)]
fn flush_buffer(buffer: &[u32], color: &[u32], x: u32, y: u32, width: u32, bw: u32) -> Vec<u32> {
    let mut buffer = buffer.to_vec();
    let mut pos = 0_usize;
    let mut start: usize = (y as usize * width as usize) + x as usize;
    let height: usize = buffer.len() / width as usize;
    let bh = color.len() / bw as usize;

    for h in 0..bh {
        for offset in 0..bw as usize {
            buffer[start] = color[pos];

            start += 1;
            pos += 1;
        }

        //start += width as usize - bw as usize - 1;
        start += width as usize - bw as usize;
    }

    buffer
}

struct Buffer {
    data: Vec<u32>,
    width: u32,
    height: u32,
}

impl Buffer {
    pub fn new(data: Vec<u32>, width: u32) -> Self {
        Self {
            height: (data.len() / width as usize) as u32,
            data,
            width,
        }
    }
}

#[inline(always)]
pub fn rgb_as_u32(rgb: &[u8; 3]) -> u32 {
    let r = (rgb[0] as u32) << 16;
    let g = (rgb[1] as u32) << 8;
    let b = (rgb[2] as u32) << 0;

    r + g + b
}
