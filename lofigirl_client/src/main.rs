mod config;
mod worker;

use anyhow::Result;
use clap::Parser;
use config::Config;
use std::path::PathBuf;
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
    /// LofiGirl Youtube stream URL.
    #[clap(short, long, value_parser)]
    url: url::Url,
}

fn main() -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(body())
}

async fn body() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::parse();
    let mut config = Config::from_toml(&opt.config).await?;
    let requested_url = opt.url;
    let (mut worker, changed) = Worker::new(&mut config, requested_url).await?;
    if changed {
        // modify config file so that we can store token and/or session_key
        config.to_toml(&opt.config).await?;
    }
    worker.work().await;
    Ok(())
}
