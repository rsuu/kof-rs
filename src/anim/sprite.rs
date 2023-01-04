//!

// TODO:
//  resize frames: [ load_stream() ]
//  jump
//  cd
//  hp

use asefile::AsepriteFile;
use minifb::{Key, Window};
use std::mem;
use std::path::Path;

type Stream = Vec<Packet>;
type Frame = Vec<u32>; // image buffer

#[derive(Debug, Clone)]
pub struct Packet {
    tag: Movement,
    right: Vec<Frame>, // Vec<argb>
    left: Vec<Frame>,  // Vec<argb>
    width: u32,
    //blocks:PlayerBlock,
    //checker: Dire,
}

#[derive(Debug)]
pub struct PlayerBlock {
    block_body: Block,
    block_head: Block,
    block_hand: Block,
    block_leg: Block,
}

pub struct Buffer {
    data: Vec<u32>,
    width: u32,
    height: u32,
}

#[derive(Debug)]
pub struct FramesInfo {
    data: Vec<Vec<u32>>,
    frame: u32,
    frame_group: u32,
}

#[derive(Debug, Clone)]
pub struct Player {
    // v_wait:FramesInfo,
    // v_run:FramesInfo,
    pub stream: Stream,
    pub ptr_frame: usize,
    pub ptr_packet: usize,

    pub hp: u32,
    pub ep: u32,

    pub block_body: Block,
    pub block_head: Block,
    pub block_hand: Block,
    pub block_leg: Block,

    pub x_offset: u32,
    pub y_offset: u32,
    pub movement: Movement,
    pub status: Status,

    pub speed: Speed,
    pub frame_timer: u8,
    pub timer: u32,

    pub is_dou: bool,

    pub dire: Dire,

    pub is_p1: bool,
    pub key_list: Vec<KeyCount>,
}

#[derive(Debug)]
pub struct Offset {
    x: u32,
    y: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyCount {
    key: Key,
    count: u8,
    timeout: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    x1: u32,
    x2: u32,
    y1: u32,
    y2: u32,
}

////////////////////////////////////////
#[derive(Debug, Clone, Copy)]
pub enum Speed {
    Stop = 0,
    VeryFast = 1,
    Fast = 3,
    Norminal = 6,
    Slow = 12,
    VerySlow = 24,
}

#[derive(Debug, Clone)]
pub enum Status {
    Null,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Movement {
    #[default]
    Stop = 0,

    Walk = 1,

    Run = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dire {
    Up,
    Down,
    Left,
    Right,
    Unknown,
}

////////////////////////////////////////
impl KeyCount {
    pub fn new(key: Key, count: u8, timeout: u8) -> Self {
        Self {
            key,
            count,
            timeout,
        }
    }

    pub fn null() -> Self {
        Self::new(Key::Unknown, 0, 0)
    }

    pub fn matched(&self, map: &KeyCount) -> bool {
        self.key == map.key && self.count >= map.count
    }
}

impl Player {
    pub fn new(dire: Dire, is_p1: bool) -> Self {
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

            speed: Speed::Norminal,
            frame_timer: Speed::Norminal as u8,
            dire,
            is_dou: false,
            is_p1,
            key_list: vec![KeyCount::null()],
        }
    }

    pub fn load_stream(&mut self, id: &str) {
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
    pub fn get_frame(&self) -> &[u32] {
        let ptr = self.movement as usize;

        if self.dire == Dire::Left {
            &self.stream[ptr].left[self.ptr_frame]
        } else {
            &self.stream[ptr].right[self.ptr_frame]
        }
    }

    #[inline(always)]
    pub fn next_frame(&mut self) {
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
    pub fn switch_to(&mut self, movement: Movement) {
        self.movement = movement;
        self.ptr_frame = 0;
    }

    pub fn try_move(&mut self, _block_checker: &Dire) {
        {
            // doing nothing
        }
    }

    #[inline(always)]
    pub fn move_walk(&mut self) {
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
        } else {
        }
    }

    #[inline(always)]
    pub fn move_run(&mut self) {
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
    pub fn move_up(&mut self) {}
    pub fn move_down(&mut self) {}

    #[inline]
    pub fn timeout(&self) -> bool {
        // reset timer
        if self.timer >= 1 && self.timer >= 1000 / 30 {
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn keys_for(&mut self, keys: &[Key], window: &Window) {
        for key in keys.iter() {
            let last = self.key_list.last_mut().unwrap();

            if last.key == *key && last.count <= 200 {
                if self.movement == Movement::Walk && self.is_dou == false {
                    self.is_dou = true;
                    last.count = 1;
                } else {
                    last.count += 1;
                }
            } else {
                self.key_list.push(KeyCount::new(*key, 0, 0));
            }
        }
    }

    #[inline(always)]
    pub fn check_keys_p1(&mut self, window: &Window) {
        // dbg!(&self.movement);

        // TODO:
        match (self.movement, self.key_list.len(), self.key_list.as_slice()) {
            // stop -> Walk -> run
            (Movement::Walk, len, keys) if (len >= 4) => {
                if keys[1].matched(&KeyCount::new(Key::A, 0, 0))
                    && keys[2].matched(&KeyCount::new(Key::Unknown, 0, 0))
                    && keys[3].matched(&KeyCount::new(Key::A, 0, 0))
                {
                    self.dire = Dire::Left;
                    self.switch_to(Movement::Run);
                } else if keys[1].matched(&KeyCount::new(Key::D, 0, 0))
                    && keys[2].matched(&KeyCount::new(Key::Unknown, 0, 0))
                    && keys[3].matched(&KeyCount::new(Key::D, 0, 0))
                {
                    self.dire = Dire::Right;
                    self.switch_to(Movement::Run);
                }
            }

            // stop -> walk
            (Movement::Stop, len, keys) if (len >= 2) => {
                if keys[0].matched(&KeyCount::new(Key::Unknown, 0, 0)) {
                    if keys[1].matched(&KeyCount::new(Key::A, 0, 0)) {
                        self.dire = Dire::Left;
                        self.switch_to(Movement::Walk);
                    } else if keys[1].matched(&KeyCount::new(Key::D, 0, 0)) {
                        self.dire = Dire::Right;
                        self.switch_to(Movement::Walk);
                    } else {
                    }
                } else {
                }
            }

            _ => {}
        }
    }

    #[inline(always)]
    pub fn check_keys_p2(&mut self, window: &Window) {
        // TODO:
        match (self.movement, self.key_list.len(), self.key_list.as_slice()) {
            // stop -> Walk -> run
            (Movement::Walk, len, keys) if (len >= 4) => {
                if keys[1].matched(&KeyCount::new(Key::Left, 0, 0))
                    && keys[2].matched(&KeyCount::new(Key::Unknown, 0, 0))
                    && keys[3].matched(&KeyCount::new(Key::Left, 0, 0))
                {
                    self.dire = Dire::Left;
                    self.switch_to(Movement::Run);
                } else if keys[1].matched(&KeyCount::new(Key::Right, 0, 0))
                    && keys[2].matched(&KeyCount::new(Key::Unknown, 0, 0))
                    && keys[3].matched(&KeyCount::new(Key::Right, 0, 0))
                {
                    self.dire = Dire::Right;
                    self.switch_to(Movement::Run);
                }
            }

            // stop -> walk
            (Movement::Stop, len, keys) if (len >= 2) => {
                if keys[0].matched(&KeyCount::new(Key::Unknown, 0, 0)) {
                    if keys[1].matched(&KeyCount::new(Key::Left, 0, 0)) {
                        self.dire = Dire::Left;
                        self.switch_to(Movement::Walk);
                    } else if keys[1].matched(&KeyCount::new(Key::Right, 0, 0)) {
                        self.dire = Dire::Right;
                        self.switch_to(Movement::Walk);
                    } else {
                    }
                } else {
                }
            }

            _ => {}
        }
    }

    #[inline(always)]
    pub fn check_keys(&mut self, window: &Window) {
        let keys = window.get_keys();
        let mut tmp: Vec<Key> = vec![];

        if self.is_p1 {
            for key in keys.iter() {
                match *key as u32 {
                    10..=35 => {
                        tmp.push(*key);
                    }

                    _ => {}
                }
            }

            self.keys_for(tmp.as_slice(), window);
            self.check_keys_p1(window);
        } else {
            for key in keys.iter() {
                match *key as u32 {
                    0..=9 => {
                        tmp.push(*key);
                    }

                    _ if (*key == Key::Up
                        || *key == Key::Down
                        || *key == Key::Left
                        || *key == Key::Right) =>
                    {
                        tmp.push(*key);
                    }

                    _ => {}
                }
            }

            self.keys_for(tmp.as_slice(), window);
            self.check_keys_p2(window);
        }

        // e.g.
        //     Walk + D + (miss) + D = Run
        //     Walk + D + D = Walk
        if window.is_key_down(self.key_list.last().unwrap().key) {
        } else {
            let last = self.key_list.last_mut().unwrap();
            if last.key == Key::Unknown && last.count <= 200 {
                last.count += 1;
            } else {
                self.key_list.push(KeyCount::null());
            }
        }

        if self.key_list.len() > 2 {
            //dbg!(self.key_list.as_slice());
        }

        self.check_keys_misc(&keys);

        self.move_to();

        let mut need = false;

        // true:
        //   Walk + A + A
        //   Walk + D + D
        // false:
        //   Walk + A + D
        //   Walk + D + A
        if (self.movement == Movement::Walk || self.movement == Movement::Run) {
            if (window.is_key_down(Key::A) || window.is_key_down(Key::Left))
                && self.dire == Dire::Left
            {
            } else if (window.is_key_down(Key::D) || window.is_key_down(Key::Right))
                && self.dire == Dire::Right
            {
            } else {
                need = true;
            }
        } else {
            need = true;
        }

        if self.timeout() && need {
            self.timer = 1;
            self.key_list = vec![KeyCount::null()];
            self.switch_to(Movement::Stop);
            self.is_dou = false;
        } else {
        }

        self.timer += 1;
    }

    #[inline(always)]
    pub fn move_to(&mut self) {
        if self.movement == Movement::Run {
            //dbg!(self.movement);
        }

        match self.movement {
            Movement::Walk => {
                self.move_walk();
            }

            Movement::Run => {
                self.move_run();
            }

            _ => {}
        }
    }

    #[inline(always)]
    pub fn check_keys_misc(&mut self, keys: &[Key]) {
        match *keys {
            // exit
            [Key::Q] => {
                std::process::exit(0);
            }

            _ => {}
        }
    }

    #[inline(always)]
    pub fn flush_buffer(&self, buffer: &mut Vec<u32>, y: u32, width: u32) {
        let color = self.get_frame();
        let mut pos = 0_usize;
        let mut idx: usize = (y as usize * width as usize) + self.x_offset as usize;

        let bw = self.stream[self.movement as usize].width;
        let bh = color.len() / bw as usize;
        let _height: usize = buffer.len() / width as usize;

        for _h in 0..bh {
            for _offset in 0..bw as usize {
                // (123,123,123,123)
                if color[pos] == 8092539 {
                } else {
                    buffer[idx] = color[pos]
                };

                idx += 1;
                pos += 1;
            }

            //idx += width as usize - bw as usize - 1;
            idx += width as usize - bw as usize;
        }
    }
}

// TODO: remove
impl Block {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x1: x,
            x2: x + width,
            y1: y,
            y2: y + height,
        }
    }

    pub fn gen_body(&self, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x1: self.x1 + x,
            x2: self.x2 + x + width,
            y1: self.y1 + y,
            y2: self.y2 + y + height,
        }
    }

    pub fn check(&self, block2: &Block) -> Dire {
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

impl Buffer {
    pub fn new(data: Vec<u32>, width: u32) -> Self {
        Self {
            height: (data.len() / width as usize) as u32,
            data,
            width,
        }
    }
}

////////////////////////////////////////
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

////////////////////////////////////////
#[inline(always)]
fn rgba_as_argb_u32(r: &u8, g: &u8, b: &u8, a: &u8) -> u32 {
    // (r, g, b, a) -> (a, r, g, b) -> u32
    //  3  2  1  0      3  2  1  0
    u32::from_be_bytes([*a, *r, *g, *b])
}

#[inline(always)]
fn rgba_as_u32(arr: &[u8; 4]) -> u32 {
    let r = (arr[0] as u32) << (8 * 3);
    let g = (arr[1] as u32) << (8 * 2);
    let b = (arr[2] as u32) << 8;

    let a = arr[3] as u32;

    r + g + b + a
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

        for _x in 0..split {
            img.swap(left, right);

            left += 1;
            right -= 1;
        }
    }
}

#[inline(always)]
pub fn argb_u32(buffer: &mut Vec<u32>, bytes: &[u8]) {
    *buffer = vec![0; bytes.len() / 4];

    for (idx, f) in (0..bytes.len()).step_by(4).enumerate() {
        buffer[idx] = rgba_as_argb_u32(&bytes[f], &bytes[f + 1], &bytes[f + 2], &bytes[f + 3]);
    }
}

// TODO:
trait KeyTrans {}
