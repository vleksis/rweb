use skia_safe::Canvas;
use skia_safe::Color;
use skia_safe::Paint;
use skia_safe::Rect as SkRect;
use skia_safe::colors::BLACK;

use crate::browser::display::CssPx;
use crate::browser::display::DisplayItem;
use crate::browser::font;

const SCROLLBAR_TRACK: Color = Color::from_rgb(0xf0, 0xf0, 0xf0);
const SCROLLBAR_THUMB: Color = Color::from_rgb(0x88, 0x88, 0x88);
const SCROLLBAR_WIDTH: CssPx = 8.0;
const MIN_THUMB_HEIGHT: CssPx = 20.0;

const BACKGROUND: Color = Color::from_rgb(0xff, 0xff, 0xff);
const FOREGROUND: Color = Color::from_rgb(0x11, 0x11, 0x11);

struct Rect {
    x: CssPx,
    y: CssPx,
    width: CssPx,
    height: CssPx,
}

pub struct Renderer;

impl Renderer {
    pub fn draw(
        buffer: &mut [u32],
        width: u32,
        height: u32,
        display_list: &[DisplayItem],
        scroll_y: CssPx,
        content_height: CssPx,
    ) {
        let Ok(width_i32) = i32::try_from(width) else {
            return;
        };
        let Ok(height_i32) = i32::try_from(height) else {
            return;
        };
        let Some(canvas) = Canvas::from_raster_direct_n32((width_i32, height_i32), buffer, None)
        else {
            return;
        };
        canvas.clear(BACKGROUND);

        let mut paint = Paint::new(BLACK, None);
        paint.set_anti_alias(true);
        paint.set_color(FOREGROUND);
        for item in display_list {
            let y = item.y - scroll_y;
            if y > height as CssPx {
                continue;
            }
            let metrics = font::font_metrics(item.style);
            if y + metrics.ascent + metrics.descent < 0.0 {
                continue;
            }

            let font = font::font_for_text(&item.text, item.style);
            canvas.draw_str(&item.text, (item.x, y + metrics.ascent), &font, &paint);
        }

        Self::draw_scrollbar(&canvas, width, height, scroll_y, content_height);
    }

    fn draw_scrollbar(
        canvas: &Canvas,
        width: u32,
        height: u32,
        scroll_y: CssPx,
        content_height: CssPx,
    ) {
        let viewport_height = height as CssPx;
        if content_height <= viewport_height {
            return;
        }

        let track_x = width as CssPx - SCROLLBAR_WIDTH;
        let thumb_height =
            (viewport_height * viewport_height / content_height).max(MIN_THUMB_HEIGHT);
        let max_scroll = content_height - viewport_height;
        let max_thumb_y = viewport_height - thumb_height;
        let thumb_y = scroll_y * max_thumb_y / max_scroll;

        let mut paint = Paint::new(BLACK, None);
        paint.set_anti_alias(false);

        Self::draw_rect(
            canvas,
            Rect {
                x: track_x,
                y: 0.0,
                width: SCROLLBAR_WIDTH,
                height: viewport_height,
            },
            SCROLLBAR_TRACK,
            &mut paint,
        );
        Self::draw_rect(
            canvas,
            Rect {
                x: track_x,
                y: thumb_y,
                width: SCROLLBAR_WIDTH,
                height: thumb_height,
            },
            SCROLLBAR_THUMB,
            &mut paint,
        );
    }

    fn draw_rect(canvas: &Canvas, rect: Rect, fill: Color, paint: &mut Paint) {
        paint.set_color(fill);
        canvas.draw_rect(
            SkRect::from_xywh(rect.x, rect.y, rect.width, rect.height),
            paint,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_are_opaque() {
        assert_eq!(BACKGROUND.a(), 0xff);
        assert_eq!(FOREGROUND.a(), 0xff);
        assert_eq!(SCROLLBAR_TRACK.a(), 0xff);
        assert_eq!(SCROLLBAR_THUMB.a(), 0xff);
    }
}
