pub mod api;
mod config;
pub mod entity;
pub mod err;
pub mod grid;

use std::path::PathBuf;

use clap::Parser;
use err::RuntimeError;
use sea_orm::{ConnectOptions, Database};
use tracing::{info_span, warn, Level};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    // initiate event system
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_file(false)
        .with_line_number(false)
        .init();

    let _main_span = info_span!("root").entered();

    let cli = {
        let _cli_span = info_span!("command_line_parse").entered();
        Cli::parse()
    };
    let config = {
        let _config_span = info_span!("read_config").entered();
        config::read_from_file(cli.config).await?
    };

    let db = {
        let _db_span = info_span!("prepare_db").entered();
        let db_opt = ConnectOptions::new(&config.db)
            .sqlx_logging(false) // Disable SQLx log
            .to_owned();
        let db = Database::connect(db_opt).await?;
        let screen_url = Url::parse(&config.db)?; // db conn uri that shows on screen
        warn!(
            "checking database <- {}:{}",
            screen_url.host_str().ok_or(url::ParseError::EmptyHost)?,
            screen_url.port().ok_or(url::ParseError::InvalidPort)?
        );
        db.ping().await?;
        warn!("database connected");
        entity::prelude::ensure_schema(&db).await?;
        entity::node::Model::prepare_origin(&db).await?;
        db
    };

    if config.http.enable{
        api::http::http_daemon(config.http, &db).await?;
    };
    if config.socket.enable{
        api::socket::socket_daemon(config.socket, &db).await?;
    }
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
