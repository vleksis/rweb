use crate::browser::page::LoadedPage;
use crate::browser::page::Page;
use crate::browser::page::PageStatus;
use crate::loader::Url;

#[derive(Debug)]
pub struct Tab {
    current_url: Option<Url>,
    page: PageStatus,
}

impl Tab {
    pub fn new(initial_url: Url) -> Self {
        Self {
            current_url: Some(initial_url.clone()),
            page: PageStatus::Loading(Page::loading(initial_url)),
        }
    }

    pub fn start_navigation(&mut self, url: Url) {
        self.current_url = Some(url.clone());
        self.page = PageStatus::Loading(Page::loading(url));
    }

    pub fn finish_navigation(&mut self, result: Result<LoadedPage, String>) {
        match result {
            Ok(loaded) => {
                self.current_url = Some(loaded.url.clone());
                self.page = PageStatus::Loaded(Page::new(loaded));
            }
            Err(message) => {
                let url = self
                    .current_url
                    .clone()
                    .expect("failed navigation should have an active URL");
                self.page = PageStatus::Failed(Page::failed(url, message));
            }
        }
    }

    pub fn page(&self) -> &PageStatus {
        &self.page
    }

    pub fn page_mut(&mut self) -> &mut PageStatus {
        &mut self.page
    }
}
