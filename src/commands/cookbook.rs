// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

use crate::{commands::Runnable, util::io::open};

#[derive(Args, Debug)]
pub struct CookbookCmd;

#[async_trait]
impl Runnable for CookbookCmd {
    async fn run(&self) -> Result<()> {
        open("https://cutlercli.github.io/cookbook").await
    }
}
