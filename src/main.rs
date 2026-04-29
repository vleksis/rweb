use std::env;

use anyhow::Context;
use rweb::loader::Client;
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
        let resp = client.load_url(url).await?;
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
