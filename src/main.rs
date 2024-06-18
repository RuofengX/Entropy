pub mod api;
pub mod entity;
pub mod err;
pub mod grid;

use std::path::PathBuf;

use clap::Parser;
use err::RuntimeError;
use sea_orm::Database;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::{info_span, warn};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    // initiate event system
    tracing_subscriber::fmt()
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
        parse_config(cli.config).await?
    };

    let db = {
        let _db_span = info_span!("prepare_db").entered();
        let db = Database::connect(&config.db).await?;
        let screen_url = Url::parse(&config.db)?; // db conn uri that shows on screen
        warn!(
            "checking database <- {}:{}",
            screen_url.host_str().ok_or(url::ParseError::EmptyHost)?,
            screen_url.port().ok_or(url::ParseError::InvalidPort)?
        );
        db.ping().await?;
        warn!("database connected");
        entity::prelude::ensure_database_schema(&db).await?;
        db
    };

    api::http::start(format!("{}:{}", config.address, config.port,), &db).await?;
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

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    db: String,
    address: String,
    port: u16,
}

async fn parse_config(path: PathBuf) -> Result<Config, RuntimeError> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}
