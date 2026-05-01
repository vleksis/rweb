use crate::browser::LoadedPage;

#[derive(Debug)]
pub enum BrowserEvent {
    PageLoaded(Result<LoadedPage, String>),
}
