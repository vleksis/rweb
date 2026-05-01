use crate::browser::tab::Tab;
use crate::loader::Url;

#[derive(Debug)]
pub struct BrowserWindow {
    tabs: Vec<Tab>,
    active_tab: usize,
}

impl BrowserWindow {
    pub fn new(initial_url: Url) -> Self {
        Self {
            tabs: vec![Tab::new(initial_url)],
            active_tab: 0,
        }
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    pub fn new_tab(&mut self, url: Url) {
        self.tabs.push(Tab::new(url));
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self, index: usize) {
        if self.tabs.len() == 1 {
            return;
        }

        self.tabs.remove(index);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
    }

    pub fn select_tab(&mut self, index: usize) {
        assert!(index < self.tabs.len(), "tab index is out of bounds");
        self.active_tab = index;
    }
}
