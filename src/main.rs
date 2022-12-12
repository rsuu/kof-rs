use asefile::AsepriteFile;
use image;
use image::io::Reader as ImageReader;
use log::debug;
use minifb::{Key, Window};
use std::io::Cursor;
use std::mem;
use std::path::Path;

// TODO:
//  resize frames: [ load_stream() ]

const FPS: u64 = 60;
const SLEEP: u64 = 1000 / 68;

type Stream = Vec<Packet>;
type Frame = Vec<u32>;

#[derive(Debug, Clone)]
struct Packet {
    tag: Movement,
    right: Vec<Frame>, // Vec<argb>
    left: Vec<Frame>,  // Vec<argb>
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

    // p2
    let mut p2 = p1.clone();
    p2.dire = Dire::Left;
    p2.x_offset = width as u32 - 300;

    // ==========================================
    // init
    window.update();
    let bg = &vec![0; width * height]; // background
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
        p2.check_keys_p2(&window);
        p2.next_frame();
        p2.flush_buffer(&mut buffer, 0, width as u32);

        window.update_with_buffer(&buffer, width, height).unwrap();
    }
    // ==========================================
}

#[inline(always)]
pub fn argb_u32(buffer: &mut Vec<u32>, bytes: &[u8]) {
    *buffer = vec![0; bytes.len() / 4];

    for (idx, f) in (0..bytes.len()).step_by(4).enumerate() {
        buffer[idx] = rgba_as_argb_u32(&bytes[f], &bytes[f + 1], &bytes[f + 2], &bytes[f + 3]);
    }
}

#[derive(Debug)]
struct FramesInfo {
    data: Vec<Vec<u32>>,
    frame: u32,
    frame_group: u32,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy)]
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

        let head = 0;
        let tail = ase.num_frames();

        let mut frames = Vec::with_capacity(tail as usize);
        //let mut pts_list = Vec::with_capacity(tail as usize);

        for idx in head..tail {
            //pts_list.push(FPS as u32 + frame.duration());

            frames.push(mem::take(&mut ase.frame(idx).image().to_vec()));
            *frames.last_mut().unwrap().last_mut().unwrap() = 0;
        }

        //dbg!(&frames[0][0..40]);

        let mut tmp: Vec<u32> = Vec::new();

        for idx in 0..frames.len() {
            argb_u32(&mut tmp, &mem::take(&mut frames[idx]));

            packet.right.push(tmp.clone());

            turn(&mut tmp, ase.width(), ase.height());
            packet.left.push(tmp.clone());
        }

        self.stream.push(packet);
    }

    #[inline(always)]
    fn get_frame(&self) -> &[u32] {
        let ptr = self.movement as usize;

        if self.dire == Dire::Left {
            &self.stream[ptr].left[self.ptr_frame]
        } else {
            &self.stream[ptr].right[self.ptr_frame]
        }
    }

    #[inline(always)]
    fn next_frame(&mut self) {
        if self.frame_timer > 0 {
            self.frame_timer -= 1;
        } else {
            let ptr = self.movement as usize;

            if self.ptr_frame + 1 < self.stream[ptr].right.len() {
                self.ptr_frame += 1;
            } else {
                self.ptr_frame = 0;
            }
            self.frame_timer = self.speed as u8;
        }
    }

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    fn move_run(&mut self) {
        if self.dire == Dire::Right {
            // TODO:
            if self.x_offset + 4 * 4 >= 200 * 2 {
                // background OR stop
                //self.x_offset = 0;
            } else {
                // player
                self.x_offset += 4 * 4;
            }
        } else if self.dire == Dire::Left {
            // TODO:
            if self.x_offset >= 4 * 4 {
                self.x_offset -= 4 * 4;
            } else {
            }
        }
    }
    fn move_up(&mut self) {}
    fn move_down(&mut self) {}

    #[inline(always)]
    fn check_keys(&mut self, window: &Window) {
        match window.get_keys().as_slice() {
            &[Key::W] => {}
            &[Key::S] => {}

            &[Key::A] => {
                self.dire = Dire::Left;

                match self.movement {
                    Movement::Stop => {
                        self.switch_to(Movement::Walk);
                    }

                    Movement::Walk => {
                        self.move_walk();

                        if self.is_dou {
                            self.switch_to(Movement::Run);
                        } else {
                        }
                    }
                    Movement::Run => {
                        self.move_run();
                        self.is_dou = false;
                    }
                    _ => {}
                }
            }

            &[Key::D] => {
                self.dire = Dire::Right;

                match self.movement {
                    Movement::Stop => {
                        self.switch_to(Movement::Walk);
                    }

                    Movement::Walk => {
                        self.move_walk();

                        if self.is_dou {
                            self.switch_to(Movement::Run);
                        } else {
                        }
                    }
                    Movement::Run => {
                        self.move_run();
                        self.is_dou = false;
                    }
                    _ => {}
                }
            }

            &[Key::Q] => {
                std::process::exit(0);
            }

            _ => {
                if (self.timer > 3 && self.timer < 15) {
                    self.is_dou = true;
                } else if self.movement != Movement::Stop {
                    self.switch_to(Movement::Stop);
                    self.timer = 0;
                    self.is_dou = false;
                } else {
                    self.timer = 0;
                    self.is_dou = false;
                }
            }
        }

        self.timer += 1;
    }

    #[inline(always)]
    fn check_keys_p2(&mut self, window: &Window) {
        match window.get_keys().as_slice() {
            &[Key::Up] => {}
            &[Key::Down] => {}

            &[Key::Left] => {
                self.dire = Dire::Left;

                match self.movement {
                    Movement::Stop => {
                        self.switch_to(Movement::Walk);
                    }

                    Movement::Walk => {
                        self.move_walk();

                        if self.is_dou {
                            self.switch_to(Movement::Run);
                        } else {
                        }
                    }
                    Movement::Run => {
                        self.move_run();
                        self.is_dou = false;
                    }
                    _ => {}
                }
            }

            &[Key::Right] => {
                self.dire = Dire::Right;

                match self.movement {
                    Movement::Stop => {
                        self.switch_to(Movement::Walk);
                    }

                    Movement::Walk => {
                        self.move_walk();

                        if self.is_dou {
                            self.switch_to(Movement::Run);
                        } else {
                        }
                    }
                    Movement::Run => {
                        self.move_run();
                        self.is_dou = false;
                    }
                    _ => {}
                }
            }

            _ => {
                if (self.timer > 3 && self.timer < 15) {
                    self.is_dou = true;
                } else if self.movement != Movement::Stop {
                    self.switch_to(Movement::Stop);
                    self.timer = 0;
                    self.is_dou = false;
                } else {
                    self.timer = 0;
                    self.is_dou = false;
                }
            }
        }

        self.timer += 1;
    }

    #[inline(always)]
    fn flush_buffer(&self, buffer: &mut Vec<u32>, y: u32, width: u32) {
        let color = self.get_frame();
        let mut pos = 0_usize;
        let mut idx: usize = (y as usize * width as usize) + self.x_offset as usize;

        let bw = self.stream[self.movement as usize].width;
        let bh = color.len() / bw as usize;
        let height: usize = buffer.len() / width as usize;

        for h in 0..bh {
            for offset in 0..bw as usize {
                //dbg!(&buffer[idx] >> 24);

                // FIXME:
                //buffer[idx] = *merge_bg_if(&buffer[idx], &color[pos]);
                buffer[idx] = color[pos];

                idx += 1;
                pos += 1;
            }

            //idx += width as usize - bw as usize - 1;
            idx += width as usize - bw as usize;
        }
    }
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
pub fn rgba_as_argb_u32(r: &u8, g: &u8, b: &u8, a: &u8) -> u32 {
    // (r, g, b, a) -> (a, r, g, b) -> u32
    //  3  2  1  0      3  2  1  0
    u32::from_be_bytes([*a, *r, *g, *b])
}

#[inline(always)]
pub fn rgba_as_u32(arr: &[u8; 4]) -> u32 {
    let r = (arr[0] as u32) << 8 * 3;
    let g = (arr[1] as u32) << 8 * 2;
    let b = (arr[2] as u32) << 8 * 1;

    let a = (arr[3] as u32) << 8 * 0;

    r + g + b + a
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

#[inline]
fn turn(img: &mut Vec<u32>, iw: usize, ih: usize) {
    let mut left: usize = 0;
    let mut right: usize = 0;
    let split = iw / 2;

    // TODO:
    for y in 1..=ih {
        right = iw * y - 1;
        left = iw * y - iw;

        for x in 0..split {
            img.swap(left, right);

            left += 1;
            right -= 1;
        }
    }
}

#[inline(always)]
fn merge_bg_if<'a>(bg: &'a u32, sp: &'a u32) -> &'a u32 {
    //dbg!(sp & 0xff);

    // 0xff = 0b1111_1111
    if sp & 0xff != 0 {
        sp
    } else {
        bg
    }
}
