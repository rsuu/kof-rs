use asefile::AsepriteFile;
use image;
use image::io::Reader as ImageReader;
use log::debug;
use std::io::Cursor;
use std::path::Path;

use minifb::{Key, Window};

const FPS: u64 = 60;
const SLEEP: u64 = 1000 / 60;

#[derive(Debug)]
struct Packet {
    tag: Movement,
    right: Vec<Frame>,
    left: Vec<Frame>,
    width: u32,
    //blocks:PlayerBlock,
    //checker: Dire,
}

#[derive(Debug)]
struct PlayerBlock {
    block_body: Block,
    block_head: Block,
    block_hand: Block,
    block_leg: Block,
}

type Frame = Vec<u32>;
type Stream = Vec<Packet>;

#[derive(Debug, Clone, Copy)]
enum Speed {
    Stop = 0,
    VeryFast = 1,
    Fast = 3,
    Norminal = 6,
    Slow = 12,
    VerySlow = 24,
}

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

    window.limit_update_rate(Some(std::time::Duration::from_millis(SLEEP)));

    // ==========================================
    // p1
    let mut p1 = Player::new(Dire::Right);
    p1.load_stream("stop");
    p1.load_stream("walk");
    p1.load_stream("run");
    p1.x_offset = 0;
    p1.y_offset = 0;

    // ==========================================
    // init
    window.update();
    let mut frame_timer = p1.speed as u8;
    // ==========================================
    // display
    'l1: while window.is_open() {
        match window.get_keys().as_slice() {
            &[Key::W] => {}
            &[Key::S] => {}

            &[Key::A] => {
                p1.dire = Dire::Left;

                match p1.movement {
                    Movement::Stop => {
                        p1.switch_to(Movement::Walk);
                    }

                    Movement::Walk => {
                        p1.move_walk();

                        if p1.is_dou {
                            p1.switch_to(Movement::Run);
                        } else {
                        }
                    }
                    Movement::Run => {
                        p1.move_run();
                        p1.is_dou = false;
                    }
                    _ => {}
                }
            }

            &[Key::D] => {
                p1.dire = Dire::Right;

                match p1.movement {
                    Movement::Stop => {
                        p1.switch_to(Movement::Walk);
                    }

                    Movement::Walk => {
                        p1.move_walk();

                        if p1.is_dou {
                            p1.switch_to(Movement::Run);
                        } else {
                        }
                    }
                    Movement::Run => {
                        p1.move_run();
                        p1.is_dou = false;
                    }
                    _ => {}
                }
            }

            &[Key::Q] => {
                std::process::exit(0);
            }

            _ => {
                if (p1.timer > 3 && p1.timer < 15) {
                    p1.is_dou = true;
                } else if p1.movement != Movement::Stop {
                    p1.switch_to(Movement::Stop);
                    p1.timer = 0;
                    p1.is_dou = false;
                } else {
                    p1.timer = 0;
                    p1.is_dou = false;
                }
            }
        }

        p1.timer += 1;

        // background
        let bg = &vec![0; 1000 * 1000];

        // frame
        if p1.frame_timer > 0 {
            p1.frame_timer -= 1;
        } else {
            p1.next_frame();
            p1.frame_timer = p1.speed as u8;
        }

        // p1
        let buffer = flush_buffer(
            &bg,
            p1.get_frame(),
            p1.x_offset,
            0,
            width as u32,
            p1.stream[p1.movement as usize].width,
        );

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
struct Player {
    // v_wait:FramesInfo,
    // v_run:FramesInfo,
    stream: Stream,
    ptr_frame: usize,
    ptr_packet: usize,

    hp: u32,
    ep: u32,

    block_body: Block,
    block_head: Block,
    block_hand: Block,
    block_leg: Block,

    x_offset: u32,
    y_offset: u32,
    movement: Movement,
    status: Status,

    speed: Speed,
    frame_timer: u8,
    timer: u32,

    is_dou: bool,

    dire: Dire,
}

#[derive(Debug)]
enum Status {
    Null,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
enum Movement {
    #[default]
    Stop = 0,

    Walk = 1,

    Run = 2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Dire {
    Up,
    Down,
    Left,
    Right,
    Unknown,
}

#[derive(Debug)]
struct Offset {
    x: u32,
    y: u32,
}

#[derive(Debug)]
struct Block {
    x1: u32,
    x2: u32,
    y1: u32,
    y2: u32,
}

impl Player {
    fn new(dire: Dire) -> Self {
        Self {
            timer: 0,
            ptr_frame: 0,
            ptr_packet: 0,
            stream: Vec::new(),
            hp: 1000,
            ep: 100,
            block_body: Block::new(0, 0, 100, 100),
            block_head: Block::new(10, 10, 10, 10),
            block_hand: Block::new(10, 10, 10, 10),
            block_leg: Block::new(10, 10, 10, 10),

            x_offset: 0,
            y_offset: 0,
            movement: Movement::default(),
            status: Status::Null,

            is_dou: false,

            speed: Speed::Norminal,
            frame_timer: Speed::Norminal as u8,

            dire,
        }
    }

    fn load_stream(&mut self, id: &str) {
        let ase = AsepriteFile::read_file(Path::new(&format!("./tests/{}.ase", id))).unwrap();
        log::debug!("Size: {}x{}", ase.width(), ase.height());
        log::debug!("Frames: {}", ase.num_frames());
        log::debug!("Layers: {}", ase.num_layers());
        log::debug!("Tags: {}", ase.num_tags());

        let mut packet = Packet {
            tag: Movement::from(id),
            right: vec![],
            left: vec![],
            width: ase.width() as u32,
        };

        // TODO:
        let tag = ase.tag_by_name("p1").unwrap();
        let start = tag.from_frame();
        let end = tag.to_frame();

        for idx in start..end {
            let img = ase.frame(idx).layer(0).image();
            let img = img.as_raw();

            let mut frame = Vec::with_capacity(img.len() / 4);

            for f in (0..img.len()).step_by(4) {
                frame.push(argb_as_u32(&img[f..f + 4].try_into().unwrap()));
            }

            packet.right.push(frame.clone());

            turn(&mut frame, ase.width(), ase.height());
            packet.left.push(frame);
        }

        self.stream.push(packet);
    }

    fn get_frame(&self) -> &[u32] {
        let ptr = self.movement as usize;

        if self.dire == Dire::Left {
            &self.stream[ptr].left[self.ptr_frame]
        } else {
            &self.stream[ptr].right[self.ptr_frame]
        }
    }

    fn next_frame(&mut self) {
        let ptr = self.movement as usize;

        if self.ptr_frame + 1 < self.stream[ptr].right.len() {
            self.ptr_frame += 1;
        } else {
            self.ptr_frame = 0;
        }
    }

    fn switch_to(&mut self, movement: Movement) {
        self.movement = movement;
        self.ptr_frame = 0;
    }

    fn try_move(&mut self, block_checker: &Dire) {
        match block_checker {
            _ => {
                // doing nothing
            }
        }
    }

    fn move_walk(&mut self) {
        if self.dire == Dire::Right {
            // TODO:
            if self.x_offset + 4 >= 200 * 2 {
                // background OR stop
                //self.x_offset = 0;
            } else {
                // player
                self.x_offset += 4;
            }
        } else if self.dire == Dire::Left {
            // TODO:
            if self.x_offset >= 4 {
                self.x_offset -= 4;
            } else {
            }
        }
    }

    fn move_run(&mut self) {
        if self.dire == Dire::Right {
            // TODO:
            if self.x_offset + 4 >= 200 * 2 {
                // background OR stop
                //self.x_offset = 0;
            } else {
                // player
                self.x_offset += 4 * 4;
            }
        } else if self.dire == Dire::Left {
            // TODO:
            if self.x_offset >= 4 {
                self.x_offset -= 4 * 4;
            } else {
            }
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

    fn check(&self, block2: &Block) -> Dire {
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
            Dire::Right
        } else if self.x1 <= block2.x2 {
            Dire::Left
        } else if self.x1 <= block2.y1 {
            Dire::Up
        } else if self.y1 >= block2.x1 {
            Dire::Down
        } else {
            Dire::Unknown
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
pub fn argb_as_u32(arr: &[u8; 4]) -> u32 {
    let a = (arr[3] as u32) << 8 * 3;

    let r = (arr[0] as u32) << 8 * 2;
    let g = (arr[1] as u32) << 8 * 1;
    let b = (arr[2] as u32) << 8 * 0;

    a + r + g + b
}

impl From<&Speed> for u8 {
    fn from(value: &Speed) -> Self {
        match *value {
            Speed::Stop => 0,
            Speed::VeryFast => 1,
            Speed::Fast => 3,
            Speed::Norminal => 6,
            Speed::Slow => 12,
            Speed::VerySlow => 24,

            _ => {
                unreachable!()
            }
        }
    }
}

impl From<&str> for Movement {
    fn from(value: &str) -> Self {
        match value {
            "stop" => Movement::Stop,
            "walk" => Movement::Walk,
            "run" => Movement::Run,
            _ => {
                todo!()
            }
        }
    }
}

impl From<Movement> for usize {
    fn from(value: Movement) -> Self {
        match value {
            Movement::Stop => 0,
            Movement::Walk => 1,
            Movement::Run => 2,
            _ => {
                todo!()
            }
        }
    }
}

fn turn(img: &mut Vec<u32>, iw: usize, ih: usize) {
    let mut left: usize = 0;
    let mut right: usize = 0;
    let mut ptr = 0;

    // TODO:

    for y in 1..=ih {
        right = iw * y - 1;
        left = iw * y - iw;

        for x in 0..iw / 2 {
            img.swap(left, right);

            left += 1;
            right -= 1;
        }
    }
}
