use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};

use crate::err::RuntimeError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    pub db: String,
    pub http: Http,
}

pub async fn read_from_file(path: PathBuf) -> Result<Root, RuntimeError> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    let config: Root = toml::from_str(&contents)?;
    Ok(config)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Http {
    pub address: String,
    pub port: u16,
}
