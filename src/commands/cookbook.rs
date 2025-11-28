// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

use crate::{commands::Runnable, config::Config, util::io::open};

#[derive(Args, Debug)]
pub struct CookbookCmd;

#[async_trait]
impl Runnable for CookbookCmd {
    fn needs_sudo(&self) -> bool {
        false
    }

    async fn run(&self, _: &Config) -> Result<()> {
        open("https://machlit.github.io/cutler").await
    }
}
