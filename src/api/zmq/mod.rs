pub mod handler;
pub mod connect;

use sea_orm::DatabaseConnection;

use crate::{config, err::RuntimeError};

pub async fn socket_daemon(
    config::Socket { address, port, .. }: config::Socket,
    db: &DatabaseConnection,
) -> Result<(), RuntimeError> {
    Ok(())
}
