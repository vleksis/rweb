use std::env;

use anyhow::Context;
use rweb::loader::HeaderName;
use rweb::loader::Method;
use rweb::loader::Request;
use rweb::loader::Url;
use rweb::loader::Version;
use rweb::loader::send;

pub fn show(html: &str) {
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            print!("{}", c);
        }
    }
}

async fn load(url: &Url) -> anyhow::Result<()> {
    let req = Request::builder()
        .method(Method::GET)
        .version(Version::HTTP10)
        .url(url.clone())
        .header(HeaderName::HOST, url.host())
        .header(HeaderName::CONNECTION, "close")
        .header(HeaderName::USER_AGENT, "RwebBrowser/0.1")
        .build()?;

    let resp = send(req).await?;
    show(&resp);

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = env::args().nth(1).context("usage: rweb <url>")?;
    let url = Url::parse(&url)?;

    load(&url).await?;

    Ok(())
}
