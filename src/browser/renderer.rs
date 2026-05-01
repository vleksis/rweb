use font8x8::UnicodeFonts;

use crate::browser::display::BACKGROUND;
use crate::browser::display::DisplayItem;
use crate::browser::display::FOREGROUND;
use crate::browser::display::GLYPH_SCALE;
use crate::browser::display::VSTEP;

const SCROLLBAR_TRACK: u32 = 0xf0f0f0;
const SCROLLBAR_THUMB: u32 = 0x888888;
const SCROLLBAR_WIDTH: i32 = 8;
const MIN_THUMB_HEIGHT: i32 = 20;

struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

pub struct Renderer;

impl Renderer {
    pub fn draw(
        buffer: &mut [u32],
        width: u32,
        height: u32,
        display_list: &[DisplayItem],
        scroll_y: i32,
        content_height: i32,
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

        Self::draw_scrollbar(buffer, width, height, scroll_y, content_height);
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

    fn draw_scrollbar(
        buffer: &mut [u32],
        width: u32,
        height: u32,
        scroll_y: i32,
        content_height: i32,
    ) {
        let viewport_height = height as i32;
        if content_height <= viewport_height {
            return;
        }

        let track_x = width as i32 - SCROLLBAR_WIDTH;
        let thumb_height =
            (viewport_height * viewport_height / content_height).max(MIN_THUMB_HEIGHT);
        let max_scroll = content_height - viewport_height;
        let max_thumb_y = viewport_height - thumb_height;
        let thumb_y = scroll_y * max_thumb_y / max_scroll;

        Self::draw_rect(
            buffer,
            width,
            height,
            Rect {
                x: track_x,
                y: 0,
                width: SCROLLBAR_WIDTH,
                height: viewport_height,
            },
            SCROLLBAR_TRACK,
        );
        Self::draw_rect(
            buffer,
            width,
            height,
            Rect {
                x: track_x,
                y: thumb_y,
                width: SCROLLBAR_WIDTH,
                height: thumb_height,
            },
            SCROLLBAR_THUMB,
        );
    }

    fn draw_rect(buffer: &mut [u32], width: u32, height: u32, rect: Rect, color: u32) {
        for yy in rect.y..rect.y + rect.height {
            for xx in rect.x..rect.x + rect.width {
                Self::set_pixel(buffer, width, height, xx, yy, color);
            }
        }
    }
}
