//!

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxAABB {
    pub x_min: u32,
    pub x_max: u32,
    pub y_min: u32,
    pub y_max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    x: u32,
    y: u32,
}

///////////////////////////////////////
impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl BoxAABB {
    fn new(x_min: u32, x_max: u32, y_min: u32, y_max: u32) -> Self {
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }
}

///////////////////////////////////////
impl Default for Point {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl Default for BoxAABB {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}
///////////////////////////////////////
