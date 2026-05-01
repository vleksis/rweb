use crate::browser::display::DisplayItem;
use crate::browser::display::GLYPH_SIZE;
use crate::browser::display::HSTEP;
use crate::browser::display::MARGIN;
use crate::browser::display::VSTEP;
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
    pub text: String,
}

#[derive(Debug)]
pub struct Page {
    url: Url,
    text: String,
    display_list: Vec<DisplayItem>,
    layout_width: Option<i32>,
    scroll_y: i32,
}

impl Page {
    pub fn new(loaded: LoadedPage) -> Self {
        Self {
            url: loaded.url,
            text: loaded.text,
            display_list: Vec::new(),
            layout_width: None,
            scroll_y: 0,
        }
    }

    pub fn loading(url: Url) -> Self {
        Self {
            url,
            text: "Loading...".to_string(),
            display_list: Vec::new(),
            layout_width: None,
            scroll_y: 0,
        }
    }

    pub fn failed(url: Url, message: String) -> Self {
        Self {
            url,
            text: format!("Failed to load page:\n{message}"),
            display_list: Vec::new(),
            layout_width: None,
            scroll_y: 0,
        }
    }

    pub fn layout(&mut self, viewport_width: i32) {
        if self.layout_width == Some(viewport_width) {
            return;
        }

        let mut display_list = Vec::new();
        let mut x = MARGIN;
        let mut y = MARGIN;
        let max_x = viewport_width - MARGIN;

        for ch in self.text.chars() {
            if ch == '\r' {
                continue;
            }

            if ch == '\n' {
                x = MARGIN;
                y += (1.2 * VSTEP as f64) as i32;
                continue;
            }

            if x + GLYPH_SIZE > max_x {
                x = MARGIN;
                y += VSTEP;
            }

            display_list.push(DisplayItem { x, y, c: ch });
            x += HSTEP;
        }

        self.display_list = display_list;
        self.layout_width = Some(viewport_width);
    }

    pub fn scroll(&mut self, delta: i32) {
        self.scroll_y = (self.scroll_y + delta).max(0);
    }

    pub fn clamp_scroll(&mut self, viewport_height: i32) {
        self.scroll_y = self.scroll_y.min((self.height() - viewport_height).max(0));
    }

    pub fn height(&self) -> i32 {
        self.display_list
            .last()
            .map(|item| item.y + VSTEP)
            .unwrap_or(VSTEP)
    }

    pub fn display_list(&self) -> &[DisplayItem] {
        &self.display_list
    }

    pub fn scroll_y(&self) -> i32 {
        self.scroll_y
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}
