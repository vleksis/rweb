pub const WINDOW_WIDTH: f64 = 800.0;
pub const WINDOW_HEIGHT: f64 = 600.0;
pub const HSTEP: i32 = 18;
pub const VSTEP: i32 = 22;
pub const MARGIN: i32 = 13;
pub const GLYPH_SCALE: i32 = 2;
pub const GLYPH_SIZE: i32 = 8 * GLYPH_SCALE;
pub const BACKGROUND: u32 = 0xffffff;
pub const FOREGROUND: u32 = 0x111111;

#[derive(Debug, Clone)]
pub struct DisplayItem {
    pub x: i32,
    pub y: i32,
    pub c: char,
}
