pub struct Block {
    a: i32,
    b: i32,
    c: i32,
    d: i32,
    //  a-------b
    //  |       |
    //  c-------d
}

#[derive(Debug)]
pub struct Hero {
    pub block_head: Block,
    pub block_body: Block,
    pub block_leg: Block,
    pub block_hand: Block,

    pub step_walk: u32,
    pub step_run: u32,
}
