mod config;
mod listener;
mod worker;

use anyhow::Result;
use config::Config;
use once_cell::sync::Lazy;
use std::{path::PathBuf, time::Duration};
use structopt::StructOpt;
use worker::Worker;

static REGULAR_INTERVAL: Lazy<Duration> = Lazy::new(|| Duration::from_secs(15));
static FAST_TRY_INTERVAL: Lazy<Duration> = Lazy::new(|| Duration::from_secs(5));

/// Now written in Rust
#[derive(StructOpt, Debug)]
#[structopt(name = "lofigirl")]
struct Opt {
    /// Configuration toml file.
    #[structopt(short, long, default_value = "config.toml")]
    config: PathBuf,
    /// Use second video link for listen info
    #[structopt(short, long)]
    second: bool,
}

fn main() -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(body())
}

async fn body() -> Result<()> {
    let opt = Opt::from_args();
    let config = Config::from_toml(&opt.config).await?;
    let mut worker = Worker::new(&config, opt.second)?;
    loop {
        let wait_duration = match worker.work().await {
            true => &REGULAR_INTERVAL,
            false => &FAST_TRY_INTERVAL
        };
        std::thread::sleep(**wait_duration);
    }
}
