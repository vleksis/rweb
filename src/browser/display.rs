pub type CssPx = f32;

pub const WINDOW_WIDTH: f64 = 800.0;
pub const WINDOW_HEIGHT: f64 = 600.0;
pub const FONT_SIZE: CssPx = 24.0;
pub const VSTEP: CssPx = 1.25 * FONT_SIZE;
pub const MARGIN: CssPx = 13.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontWeight {
    Normal,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontSlant {
    Roman,
    Italic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub size: CssPx,
    pub weight: FontWeight,
    pub slant: FontSlant,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: FONT_SIZE,
            weight: FontWeight::Normal,
            slant: FontSlant::Roman,
        }
    }
}

impl TextStyle {
    pub fn bold(&mut self) {
        self.weight = FontWeight::Bold;
    }

    pub fn normal_weight(&mut self) {
        self.weight = FontWeight::Normal;
    }

    pub fn italic(&mut self) {
        self.slant = FontSlant::Italic;
    }

    pub fn roman(&mut self) {
        self.slant = FontSlant::Roman;
    }

    pub fn smaller(&mut self) {
        self.size -= 2.0;
    }

    pub fn larger(&mut self) {
        self.size += 4.0;
    }
}

#[derive(Debug, Clone)]
pub struct DisplayItem {
    pub x: CssPx,
    pub y: CssPx,
    pub text: String,
    pub style: TextStyle,
}
