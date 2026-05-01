use font8x8::UnicodeFonts;

use crate::browser::display::BACKGROUND;
use crate::browser::display::DisplayItem;
use crate::browser::display::FOREGROUND;
use crate::browser::display::GLYPH_SCALE;
use crate::browser::display::VSTEP;

pub struct Renderer;

impl Renderer {
    pub fn draw(
        buffer: &mut [u32],
        width: u32,
        height: u32,
        display_list: &[DisplayItem],
        scroll_y: i32,
    ) {
        buffer.fill(BACKGROUND);

        for item in display_list {
            let y = item.y - scroll_y;
            if y > height as i32 {
                continue;
            }
            if y + VSTEP < 0 {
                continue;
            }

            Self::draw_char(buffer, width, height, item.x, y, item.c);
        }
    }

    fn draw_char(buffer: &mut [u32], width: u32, height: u32, x: i32, y: i32, c: char) {
        let glyph = font8x8::BASIC_FONTS
            .get(c)
            .or_else(|| font8x8::BASIC_FONTS.get('?'));
        let Some(glyph) = glyph else {
            return;
        };

        for (row, bits) in glyph.iter().enumerate() {
            let bits = *bits;
            for col in 0..8 {
                if bits & (1u8 << col) == 0 {
                    continue;
                }

                for dy in 0..GLYPH_SCALE {
                    for dx in 0..GLYPH_SCALE {
                        Self::set_pixel(
                            buffer,
                            width,
                            height,
                            x + col * GLYPH_SCALE + dx,
                            y + row as i32 * GLYPH_SCALE + dy,
                            FOREGROUND,
                        );
                    }
                }
            }
        }
    }

    fn set_pixel(buffer: &mut [u32], width: u32, height: u32, x: i32, y: i32, color: u32) {
        if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
            return;
        }

        buffer[y as usize * width as usize + x as usize] = color;
    }
}
