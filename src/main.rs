use std::env;

use anyhow::Context;
use rweb::http::HeaderName;
use rweb::http::Method;
use rweb::http::Version;
use rweb::loader::Client;
use rweb::loader::Request;
use rweb::loader::Url;

mod browser {
    use super::*;

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

    pub async fn load(client: &mut Client, url: &Url) -> anyhow::Result<()> {
        let req: Request = match url {
            Url::Http(url) => Request::builder()
                .http()
                .url(url.clone())
                .method(Method::GET)
                .version(Version::HTTP11)
                .header(HeaderName::HOST, &url.host_header())
                .header(HeaderName::CONNECTION, "keep-alive")
                .header(HeaderName::USER_AGENT, "RwebBrowser/0.1")
                .build()?
                .into(),

            Url::File(url) => Request::builder().file().url(url).build()?.into(),
            Url::Data(url) => Request::builder().data().url(url).build()?.into(),
        };

        let resp = client.load(req).await?;
        let body = resp.body_as_str()?;
        show(body);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = env::args().nth(1).context("usage: rweb <url>")?;
    let url = url.parse()?;

    let mut client = Client::builder().build();

    browser::load(&mut client, &url).await?;

    Ok(())
}
