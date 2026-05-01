use std::cell::RefCell;
use std::collections::HashMap;

use skia_safe::Font;
use skia_safe::FontMgr;
use skia_safe::FontStyle as SkiaFontStyle;
use skia_safe::Paint;
use skia_safe::Unichar;
use skia_safe::font_style::Slant as SkiaSlant;
use skia_safe::font_style::Weight as SkiaWeight;
use skia_safe::font_style::Width as SkiaWidth;

use crate::browser::display::CssPx;
use crate::browser::display::FontSlant;
use crate::browser::display::FontWeight;
use crate::browser::display::TextStyle;

#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    pub ascent: CssPx,
    pub descent: CssPx,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FontKey {
    size_bits: u32,
    weight: FontWeight,
    slant: FontSlant,
}

impl From<TextStyle> for FontKey {
    fn from(style: TextStyle) -> Self {
        Self {
            size_bits: style.size.to_bits(),
            weight: style.weight,
            slant: style.slant,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FallbackKey {
    font: FontKey,
    c: char,
}

struct Fonts {
    manager: FontMgr,
    default_family: String,
    primary: HashMap<FontKey, Font>,
    fallback: HashMap<FallbackKey, Font>,
    widths: HashMap<(FontKey, String), CssPx>,
}

impl Fonts {
    fn new() -> Self {
        let manager = FontMgr::new();
        let default = make_font(
            manager
                .legacy_make_typeface(None, SkiaFontStyle::normal())
                .map(|typeface| Font::from_typeface(typeface, None))
                .unwrap_or_default(),
            TextStyle::default(),
        );
        let default_family = default.typeface().family_name();

        Self {
            manager,
            default_family,
            primary: HashMap::new(),
            fallback: HashMap::new(),
            widths: HashMap::new(),
        }
    }

    fn primary(&mut self, style: TextStyle) -> Font {
        let key = FontKey::from(style);
        self.primary
            .entry(key)
            .or_insert_with(|| {
                let typeface = self
                    .manager
                    .match_family_style(&self.default_family, skia_style(style))
                    .or_else(|| self.manager.legacy_make_typeface(None, skia_style(style)));
                let font = typeface
                    .map(|typeface| Font::from_typeface(typeface, None))
                    .unwrap_or_default();
                make_font(font, style)
            })
            .clone()
    }

    fn for_text(&mut self, text: &str, style: TextStyle) -> Font {
        let primary = self.primary(style);
        let missing = text
            .chars()
            .find(|c| primary.unichar_to_glyph(*c as Unichar) == 0);

        let Some(c) = missing else {
            return primary;
        };

        let key = FallbackKey {
            font: FontKey::from(style),
            c,
        };
        self.fallback
            .entry(key)
            .or_insert_with(|| {
                self.manager
                    .match_family_style_character(
                        &self.default_family,
                        skia_style(style),
                        &[],
                        c as Unichar,
                    )
                    .map(|typeface| make_font(Font::from_typeface(typeface, None), style))
                    .unwrap_or(primary)
            })
            .clone()
    }

    fn measure(&mut self, text: &str, style: TextStyle) -> CssPx {
        let key = (FontKey::from(style), text.to_string());
        if let Some(width) = self.widths.get(&key) {
            return *width;
        }

        let font = self.for_text(text, style);
        let width = font.measure_str(text, None::<&Paint>).0;
        self.widths.insert(key, width);
        width
    }

    fn metrics(&mut self, style: TextStyle) -> FontMetrics {
        let font = self.primary(style);
        let (_, metrics) = font.metrics();

        FontMetrics {
            ascent: -metrics.ascent,
            descent: metrics.descent,
        }
    }
}

thread_local! {
    static FONTS: RefCell<Fonts> = RefCell::new(Fonts::new());
}

pub fn font_for_text(text: &str, style: TextStyle) -> Font {
    FONTS.with(|fonts| {
        let mut fonts = fonts.borrow_mut();
        fonts.for_text(text, style)
    })
}

pub fn measure_text(text: &str, style: TextStyle) -> CssPx {
    FONTS.with(|fonts| {
        let mut fonts = fonts.borrow_mut();
        fonts.measure(text, style)
    })
}

pub fn font_metrics(style: TextStyle) -> FontMetrics {
    FONTS.with(|fonts| {
        let mut fonts = fonts.borrow_mut();
        fonts.metrics(style)
    })
}

fn make_font(mut font: Font, style: TextStyle) -> Font {
    font.set_size(style.size);
    font.set_subpixel(true);
    font
}

fn skia_style(style: TextStyle) -> SkiaFontStyle {
    let weight = match style.weight {
        FontWeight::Normal => SkiaWeight::NORMAL,
        FontWeight::Bold => SkiaWeight::BOLD,
    };
    let slant = match style.slant {
        FontSlant::Roman => SkiaSlant::Upright,
        FontSlant::Italic => SkiaSlant::Italic,
    };

    SkiaFontStyle::new(weight, SkiaWidth::NORMAL, slant)
}
