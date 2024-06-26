pub mod api;
mod config;
pub mod entity;
pub mod err;
pub mod grid;

use std::{path::PathBuf, thread, time::Duration};

use clap::Parser;
use err::RuntimeError;
use pg_embed::{
    pg_enums::PgAuthMethod,
    pg_fetch::{PgFetchSettings, PG_V13},
    postgres::{PgEmbed, PgSettings},
};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbConn};
use tracing::{info, info_span, instrument, warn, Level};
use url::Url;

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

    let db = prepare_db(config.db).await?;

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

enum DatabaseInstance {
    Embed(PgEmbed, DatabaseConnection),
    Remote(DatabaseConnection),
}
impl AsRef<DbConn> for DatabaseInstance {
    fn as_ref(&self) -> &DbConn {
        match self {
            Self::Embed(_db, conn) => conn,
            Self::Remote(conn) => conn,
        }
    }
}

#[instrument(skip(config), err(level = Level::ERROR))]
async fn prepare_db(config: config::Db) -> Result<DatabaseInstance, RuntimeError> {
    let mut embed_db: Option<PgEmbed> = None;
    let url = if config.embed.enable {
        let _setup_span = info_span!("setup_embed").entered();

        warn!("using embed database");
        let pg_settings = PgSettings {
            database_dir: config.embed.dir,
            port: config.embed.port,
            user: config.embed.user,
            password: config.embed.password,
            auth_method: PgAuthMethod::Plain,
            persistent: config.embed.persistent,
            timeout: Some(Duration::from_secs(config.embed.timeout)),
            migration_dir: None,
        };

        let fetch_settings = PgFetchSettings {
            version: PG_V13,
            ..Default::default()
        };
        let mut pg = PgEmbed::new(pg_settings, fetch_settings).await?;

        info!("downloading");
        pg.setup().await?;

        info!("starting");
        pg.start_db().await?;

        if !pg.database_exists("entropy").await? {
            info!("create database `entropy`");
            pg.create_database("entropy").await?;
        }

        thread::sleep(Duration::from_secs(3));
        let url = pg.full_db_uri("entropy");

        info!("done");
        embed_db = Some(pg);
        url
    } else {
        config.remote.url
    };
    let screen_url = Url::parse(&url)?; // db conn uri that shows on screen
    warn!(
        "connecting to database <- {}:{}",
        screen_url.host_str().ok_or(url::ParseError::EmptyHost)?,
        screen_url.port().ok_or(url::ParseError::InvalidPort)?
    );

    let db_opt = ConnectOptions::new(&url)
        .sqlx_logging(false) // Disable SQLx log
        .to_owned();
    let conn = Database::connect(db_opt).await?;
    conn.ping().await?;
    warn!("database connected");

    entity::prelude::ensure_schema(&conn).await?;
    entity::node::Model::prepare_origin(&conn).await?;

    if let Some(embed_db) = embed_db {
        Ok(DatabaseInstance::Embed(embed_db, conn))
    } else {
        Ok(DatabaseInstance::Remote(conn))
    }
}
