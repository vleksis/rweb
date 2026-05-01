use std::env;

use anyhow::Context;
use rweb::loader::Url;

mod gui;

fn main() -> anyhow::Result<()> {
    let raw_url = env::args().nth(1).context("usage: rweb <url>")?;
    let url: Url = raw_url.parse()?;

    gui::show(&raw_url, url)
}
