pub mod api;
mod config;
mod db;
pub mod entity;
pub mod err;

use std::path::PathBuf;

use clap::Parser;
use err::RuntimeError;
use tracing::{info_span, Level};

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    // initiate event system
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_file(false)
        .with_line_number(false)
        .init();

    let _main_span = info_span!("entropy").entered();

    let cli = {
        let _cli_span = info_span!("command_line_parse").entered();
        Cli::parse()
    };
    let config = {
        let _config_span = info_span!("read_config").entered();
        config::read_from_file(cli.config).await?
    };

    let db = db::prepare_db(config.db).await?;

    if config.http.enable {
        api::http::http_daemon(config.http, &db).await?;
    };
    // if config.socket.enable{
    // api::zmq::socket_daemon(config.socket, &db).await?;
    // };
    Ok(())
}

#[derive(Parser)]
#[command(name = "entropy")]
#[command(about = "a rust game server")]
struct Cli {
    /// Config file
    #[arg(short, long)]
    #[arg(help = "path to config file")]
    #[arg(default_value = "entropy.toml")]
    config: PathBuf,
}
