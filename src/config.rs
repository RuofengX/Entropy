use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};

use crate::err::RuntimeError;

#[derive(Debug, Deserialize)]
pub struct Root {
    pub db: Db,
    pub http: Http,
    // pub socket: Socket,
}

#[derive(Debug, Deserialize)]
pub struct Db {
    pub embed: EmbedDb,
    pub remote: RemoteDb,
}
#[derive(Debug, Deserialize)]
pub struct EmbedDb {
    pub enable: bool,
    pub dir: PathBuf,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub persistent: bool,
    pub timeout: u64,
}

#[derive(Debug, Deserialize)]
pub struct RemoteDb {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Http {
    pub enable: bool,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Socket {
    pub enable: bool,
    pub address: String,
    pub port: u16,
}

pub async fn read_from_file(path: PathBuf) -> Result<Root, RuntimeError> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    let config: Root = toml::from_str(&contents)?;
    Ok(config)
}
