use crate::browser::display::DisplayItem;
use crate::browser::page::LoadedPage;
use crate::browser::tab::Tab;
use crate::browser::window::BrowserWindow;
use crate::loader::Url;

#[derive(Debug)]
pub struct Browser {
    windows: Vec<BrowserWindow>,
    active_window: usize,
}

impl Browser {
    pub fn new(initial_url: Url) -> Self {
        Self {
            windows: vec![BrowserWindow::new(initial_url)],
            active_window: 0,
        }
    }

    pub fn start_navigation(&mut self, url: Url) {
        self.active_tab_mut().start_navigation(url);
    }

    pub fn finish_navigation(&mut self, result: Result<LoadedPage, String>) {
        self.active_tab_mut().finish_navigation(result);
    }

    pub fn layout_active_page(&mut self, viewport_width: i32, viewport_height: i32) {
        let page = self.active_tab_mut().page_mut().page_mut();
        page.layout(viewport_width);
        page.clamp_scroll(viewport_height);
    }

    pub fn scroll_active_page(&mut self, delta: i32) {
        self.active_tab_mut().page_mut().page_mut().scroll(delta);
    }

    pub fn active_tab(&self) -> &Tab {
        self.active_window().active_tab()
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        self.active_window_mut().active_tab_mut()
    }

    pub fn active_scroll_y(&self) -> i32 {
        self.active_tab().page().page().scroll_y()
    }

    pub fn active_display_list(&self) -> &[DisplayItem] {
        self.active_tab().page().page().display_list()
    }

    pub fn active_page_height(&self) -> i32 {
        self.active_tab().page().page().height()
    }

    fn active_window(&self) -> &BrowserWindow {
        &self.windows[self.active_window]
    }

    fn active_window_mut(&mut self) -> &mut BrowserWindow {
        &mut self.windows[self.active_window]
    }
}
