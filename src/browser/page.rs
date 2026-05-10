use crate::browser::display::CssPx;
use crate::browser::display::DisplayItem;
use crate::browser::layout::Layout;
use crate::html;
use crate::html::Document;
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
    document: Document,
    layout: Layout,
    scroll_y: CssPx,
}

impl Page {
    pub fn new(loaded: LoadedPage) -> Self {
        Self::from_source(loaded.url, loaded.source)
    }

    fn from_source(url: Url, source: String) -> Self {
        Self {
            url,
            document: html::parse(source).expect("failed to parse page html"),
            layout: Layout::empty(),
            scroll_y: 0.0,
        }
    }

    pub fn loading(url: Url) -> Self {
        Self::from_source(url, "Loading...".to_string())
    }

    pub fn failed(url: Url, message: String) -> Self {
        Self::from_source(url, format!("Failed to load page:\n{message}"))
    }

    pub fn layout(&mut self, viewport_width: CssPx) {
        if self.layout.width() == viewport_width {
            return;
        }

        self.layout = Layout::build(&self.document, viewport_width);
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
