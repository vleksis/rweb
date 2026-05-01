pub type CssPx = f32;

pub const WINDOW_WIDTH: f64 = 800.0;
pub const WINDOW_HEIGHT: f64 = 600.0;
pub const HSTEP: CssPx = 16.0;
pub const VSTEP: CssPx = 20.0;
pub const MARGIN: CssPx = 13.0;
pub const FONT_SIZE: CssPx = 18.0;
pub const GLYPH_SIZE: CssPx = 16.0;

#[derive(Debug, Clone)]
pub struct DisplayItem {
    pub x: CssPx,
    pub y: CssPx,
    pub c: char,
}
