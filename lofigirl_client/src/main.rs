mod config;
mod worker;

use anyhow::Result;
use config::Config;
use lofigirl_shared_common::{FAST_TRY_INTERVAL, REGULAR_INTERVAL};
use std::path::PathBuf;
use clap::Parser;
use worker::Worker;

#[cfg(not(feature = "standalone"))]
const APP_NAME: &str = "lofigirl_client";

#[cfg(feature = "standalone")]
const APP_NAME: &str = "lofigirl_client_standalone";

/// Scrobble the tracks you listen on lofigirl streams.
#[derive(Parser, Debug)]
#[clap(name = APP_NAME, author, version, about, long_about = None)]
struct Opt {
    /// Configuration toml file.
    #[clap(short, long, value_parser, default_value = "config.toml")]
    config: PathBuf,
    /// Use second video link for listen info
    #[clap(short, long, value_parser)]
    second: bool,
}

fn main() -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(body())
}

async fn body() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();
    let mut config = Config::from_toml(&opt.config).await?;
    let (mut worker, changed) = Worker::new(&mut config, opt.second).await?;
    if changed {
        // modify config file so that we can store token and/or session_key
        config.to_toml(&opt.config).await?;
    }
    loop {
        let wait_duration = match worker.work().await {
            true => &REGULAR_INTERVAL,
            false => &FAST_TRY_INTERVAL,
        };
        std::thread::sleep(**wait_duration);
    }
}
