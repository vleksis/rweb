pub const WINDOW_WIDTH: f64 = 800.0;
pub const WINDOW_HEIGHT: f64 = 600.0;
pub const HSTEP: i32 = 18;
pub const VSTEP: i32 = 22;
pub const MARGIN: i32 = 13;
pub const FONT_SIZE: f32 = 18.0;
pub const GLYPH_SIZE: i32 = 16;

#[derive(Debug, Clone)]
pub struct DisplayItem {
    pub x: i32,
    pub y: i32,
    pub c: char,
}
