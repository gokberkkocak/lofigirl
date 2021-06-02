mod config;
mod worker;

use anyhow::Result;
use config::Config;
use lofigirl_shared_common::{FAST_TRY_INTERVAL, REGULAR_INTERVAL};
use std::path::PathBuf;
use structopt::StructOpt;
use worker::Worker;

#[cfg(not(feature = "standalone"))]
const APP_NAME: &str = "lofigirl";

#[cfg(feature = "standalone")]
const APP_NAME: &str = "lofigirl_standalone";

/// Scrobble the tracks you listen on lofigirl streams.
#[derive(StructOpt, Debug)]
#[structopt(name = APP_NAME)]
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
    let mut worker = Worker::new(&config, opt.second).await?;
    loop {
        let wait_duration = match worker.work().await {
            true => &REGULAR_INTERVAL,
            false => &FAST_TRY_INTERVAL,
        };
        std::thread::sleep(**wait_duration);
    }
}
