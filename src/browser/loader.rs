use winit::event_loop::EventLoopProxy;

use crate::browser::BrowserEvent;
use crate::browser::LoadedPage;
use crate::loader::Client;
use crate::loader::Url;

pub struct Loader {
    runtime: tokio::runtime::Runtime,
    proxy: EventLoopProxy<BrowserEvent>,
}

impl Loader {
    pub fn new(proxy: EventLoopProxy<BrowserEvent>) -> anyhow::Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        Ok(Self { runtime, proxy })
    }

    pub fn load(&self, url: Url) {
        let proxy = self.proxy.clone();

        self.runtime.spawn(async move {
            let result = load_page(url).await.map_err(|err| err.to_string());
            let _ = proxy.send_event(BrowserEvent::PageLoaded(result));
        });
    }
}

async fn load_page(url: Url) -> anyhow::Result<LoadedPage> {
    let mut client = Client::builder().build();
    let response = client.load_url(&url).await?;
    let body = response.body_as_str()?;
    let source = body.to_string();

    Ok(LoadedPage { url, source })
}
