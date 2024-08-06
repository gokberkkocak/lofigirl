mod config;
mod server;
mod session;
mod worker;

use crate::config::ServerConfig;
use clap::Parser;
use server::LofiServer;
use std::path::PathBuf;
use worker::ServerWorker;

const APP_NAME: &str = "lofigirl_server";

/// Scrobble the tracks you listen on lofigirl streams.
#[derive(Parser, Debug)]
#[clap(name = APP_NAME, author, version, about, long_about = None)]
struct Opt {
    /// Configuration toml file.
    #[clap(short, long, value_parser, default_value = "config.toml")]
    config: PathBuf,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::parse();
    let config = ServerConfig::from_toml(&opt.config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let mut worker = ServerWorker::new(&config)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let state = worker.state.clone();
    actix_rt::spawn(async move {
        worker
            .work()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    });
    LofiServer::start(state, config.server_settings.port).await
}
