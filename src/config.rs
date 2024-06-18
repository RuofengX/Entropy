use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};

use crate::err::RuntimeError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub db: String,
    pub address: String,
    pub port: u16,
}

pub async fn read_from_file(path: PathBuf) -> Result<Config, RuntimeError> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}
