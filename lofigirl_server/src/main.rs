mod config;
mod server;
mod session;
mod worker;

use server::LofiServer;
use std::path::PathBuf;
use structopt::StructOpt;
use worker::ServerWorker;

use crate::config::ServerConfig;

/// Scrobble the tracks you listen on lofigirl streams.
#[derive(StructOpt, Debug)]
#[structopt(name = "lofigirl_server")]
struct Opt {
    /// Configuration toml file.
    #[structopt(short, long, default_value = "config.toml")]
    config: PathBuf,
    /// Only provide information for the first given link.
    #[structopt(short, long)]
    only_first: bool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();
    let config = ServerConfig::from_toml(&opt.config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let mut worker = ServerWorker::new(&config, opt.only_first)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let state = worker.state.clone();
    actix_rt::spawn(async move {
        worker.loop_work().await;
    });
    LofiServer::start(state, config.server_settings.port).await
}
