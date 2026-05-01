use crate::browser::display::CssPx;
use crate::browser::display::DisplayItem;
use crate::browser::display::MARGIN;
use crate::browser::display::TextStyle;
use crate::browser::display::VSTEP;
use crate::browser::font;
use crate::browser::html::Token;
use crate::browser::html::lex;
use crate::loader::Url;

#[derive(Debug)]
pub enum PageStatus {
    Loading(Page),
    Loaded(Page),
    Failed(Page),
}

impl PageStatus {
    pub fn page(&self) -> &Page {
        match self {
            Self::Loading(page) | Self::Loaded(page) | Self::Failed(page) => page,
        }
    }

    pub fn page_mut(&mut self) -> &mut Page {
        match self {
            Self::Loading(page) | Self::Loaded(page) | Self::Failed(page) => page,
        }
    }
}

#[derive(Debug)]
pub struct LoadedPage {
    pub url: Url,
    pub source: String,
}

#[derive(Debug)]
pub struct Page {
    url: Url,
    source: String,
    layout: Layout,
    scroll_y: CssPx,
}

impl Page {
    pub fn new(loaded: LoadedPage) -> Self {
        Self {
            url: loaded.url,
            source: loaded.source,
            layout: Layout::empty(),
            scroll_y: 0.0,
        }
    }

    pub fn loading(url: Url) -> Self {
        Self {
            url,
            source: "Loading...".to_string(),
            layout: Layout::empty(),
            scroll_y: 0.0,
        }
    }

    pub fn failed(url: Url, message: String) -> Self {
        Self {
            url,
            source: format!("Failed to load page:\n{message}"),
            layout: Layout::empty(),
            scroll_y: 0.0,
        }
    }

    pub fn layout(&mut self, viewport_width: CssPx) {
        if self.layout.width() == viewport_width {
            return;
        }

        let mut builder = LayoutBuilder::new(viewport_width);
        for token in lex(&self.source) {
            builder.token(token);
        }

        self.layout = builder.finish();
    }

    pub fn scroll(&mut self, delta: CssPx) {
        self.scroll_y = (self.scroll_y + delta).max(0.0);
    }

    pub fn clamp_scroll(&mut self, viewport_height: CssPx) {
        self.scroll_y = self
            .scroll_y
            .min((self.height() - viewport_height).max(0.0));
    }

    pub fn height(&self) -> CssPx {
        self.layout.height()
    }

    pub fn display_list(&self) -> &[DisplayItem] {
        self.layout.display_list()
    }

    pub fn scroll_y(&self) -> CssPx {
        self.scroll_y
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}

#[derive(Debug)]
pub struct Layout {
    width: CssPx,
    display_list: Vec<DisplayItem>,
    height: CssPx,
}

impl Layout {
    fn empty() -> Self {
        Self {
            width: 0.0,
            display_list: Vec::new(),
            height: VSTEP,
        }
    }

    pub fn new(width: CssPx, display_list: Vec<DisplayItem>, height: CssPx) -> Self {
        Self {
            width,
            display_list,
            height,
        }
    }

    pub fn width(&self) -> CssPx {
        self.width
    }

    pub fn display_list(&self) -> &[DisplayItem] {
        &self.display_list
    }

    pub fn height(&self) -> CssPx {
        self.height
    }
}

struct LayoutBuilder {
    display_list: Vec<DisplayItem>,
    line: Vec<LineItem>,
    style: TextStyle,
    viewport_width: CssPx,
    x: CssPx,
    y: CssPx,
    max_x: CssPx,
    pending_space: bool,
}

impl LayoutBuilder {
    fn new(viewport_width: CssPx) -> Self {
        Self {
            display_list: Vec::new(),
            line: Vec::new(),
            style: TextStyle::default(),
            viewport_width,
            x: MARGIN,
            y: MARGIN,
            max_x: viewport_width - MARGIN,
            pending_space: false,
        }
    }

    fn token(&mut self, token: Token) {
        match token {
            Token::Text(text) => self.text(&text),
            Token::Tag(tag) => self.tag(&tag),
        }
    }

    fn text(&mut self, text: &str) {
        let mut word = String::new();

        for c in text.chars() {
            if c.is_whitespace() {
                if !word.is_empty() {
                    self.word(&word);
                    word.clear();
                }
                self.pending_space = true;
            } else {
                word.push(c);
            }
        }

        if !word.is_empty() {
            self.word(&word);
        }
    }

    fn word(&mut self, word: &str) {
        let space_width = font::measure_text(" ", self.style);
        let word_width = font::measure_text(word, self.style);
        let word_x = if self.pending_space && !self.line.is_empty() {
            self.x + space_width
        } else {
            self.x
        };

        if word_x + word_width > self.max_x {
            self.flush();
        }

        let word_x = if self.pending_space && !self.line.is_empty() {
            self.x + space_width
        } else {
            self.x
        };
        self.line.push(LineItem {
            x: word_x,
            text: word.to_string(),
            style: self.style,
        });
        self.x = word_x + word_width;
        self.pending_space = false;
    }

    fn tag(&mut self, tag: &str) {
        let tag = tag
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();

        match tag.as_str() {
            "br" => self.flush(),
            "/p" => {
                self.flush();
                self.y += VSTEP;
            }
            "b" | "strong" => self.style.bold(),
            "/b" | "/strong" => self.style.normal_weight(),
            "i" | "em" => self.style.italic(),
            "/i" | "/em" => self.style.roman(),
            "small" => self.style.smaller(),
            "/small" => self.style.larger(),
            "big" => self.style.larger(),
            "/big" => self.style.smaller(),
            _ => {}
        }
    }

    fn flush(&mut self) {
        if self.line.is_empty() {
            return;
        }

        let max_ascent = self
            .line
            .iter()
            .map(|item| font::font_metrics(item.style).ascent)
            .fold(0.0, CssPx::max);
        let baseline = self.y + 1.25 * max_ascent;
        let max_descent = self
            .line
            .iter()
            .map(|item| font::font_metrics(item.style).descent)
            .fold(0.0, CssPx::max);

        for item in self.line.drain(..) {
            let metrics = font::font_metrics(item.style);
            self.display_list.push(DisplayItem {
                x: item.x,
                y: baseline - metrics.ascent,
                text: item.text,
                style: item.style,
            });
        }

        self.x = MARGIN;
        self.y = baseline + 1.25 * max_descent;
        self.pending_space = false;
    }

    fn finish(mut self) -> Layout {
        self.flush();
        Layout::new(self.viewport_width, self.display_list, self.y.max(VSTEP))
    }
}

struct LineItem {
    x: CssPx,
    text: String,
    style: TextStyle,
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::browser::FontSlant;
    use crate::browser::FontWeight;

    #[test]
    fn lexes_text_and_tags() {
        assert_eq!(
            lex("a <b>bold</b>"),
            vec![
                Token::Text("a ".to_string()),
                Token::Tag("b".to_string()),
                Token::Text("bold".to_string()),
                Token::Tag("/b".to_string()),
            ]
        );
    }

    #[test]
    fn layout_emits_words_not_characters() {
        let mut builder = LayoutBuilder::new(400.0);
        builder.text("hello world");
        let layout = builder.finish();

        assert_eq!(layout.display_list.len(), 2);
        assert_eq!(layout.display_list[0].text, "hello");
        assert_eq!(layout.display_list[1].text, "world");
    }

    #[test]
    fn layout_applies_bold_tag() {
        let mut builder = LayoutBuilder::new(400.0);
        builder.token(Token::Tag("b".to_string()));
        builder.token(Token::Text("hello".to_string()));
        let layout = builder.finish();

        assert_eq!(layout.display_list[0].style.weight, FontWeight::Bold);
    }

    #[test]
    fn punctuation_after_inline_tag_does_not_get_extra_space() {
        let mut builder = LayoutBuilder::new(800.0);
        builder.token(Token::Text("What is a ".to_string()));
        builder.token(Token::Tag("em".to_string()));
        builder.token(Token::Text("font".to_string()));
        builder.token(Token::Tag("/em".to_string()));
        builder.token(Token::Text(", exactly?".to_string()));
        let layout = builder.finish();

        let font_item = layout
            .display_list
            .iter()
            .find(|item| item.text == "font")
            .unwrap();
        let comma = layout
            .display_list
            .iter()
            .find(|item| item.text == ",")
            .unwrap();
        let font_end = font_item.x + font::measure_text(&font_item.text, font_item.style);

        assert_eq!(font_item.style.slant, FontSlant::Italic);
        assert!((comma.x - font_end).abs() < 0.5);
    }
}
