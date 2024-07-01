use pg_embed_alternative::pg_errors::PgEmbedError;
use sea_orm::DbErr;
use thiserror::Error;

use entropy_base::grid::{navi, NodeID};

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("backend database error <- {0}")]
    Database(#[from] DbErr),
    #[error("error while parse model <- {desc}")]
    Parse { desc: String },
    #[error("data out of limit::{limit_type} <- {desc}")]
    OutOfLimit {
        desc: String,
        limit_type: &'static str,
    },
}

#[derive(Error, Debug)]
pub enum OperationError {
    #[error(transparent)]
    Model(#[from] ModelError),
    #[error("energy not enough <- require:{require}, reserve:{reserve}")]
    EnergyNotEnough { require: i64, reserve: i64 },
    #[error("player already has guest <- only player with no guest can spawn free guest")]
    AlreadyHasGuest,
    #[error("player not exist or check your password <- request player id:{0}")]
    PlayerNotExist(i32),
    #[error("guest not exist <- request guest id:{0}")]
    GuestNotExist(i32),
    #[error("navi direction not allowed <- request direction:{0:?}")]
    DirectionNotAllowed(navi::Direction),
    #[error("cannot exhaust heat <- node:{0:?}")]
    NodeTemperatureTooHigh(NodeID),
    #[error("index longer than node:{node:?} index <- required:{require}, max length:{max}")]
    CellIndexOutOfRange {
        node: NodeID,
        require: usize,
        max: usize,
    },
    #[error("cannot exhaust heat <- index:{index}@node:{node:?}")]
    CellTemperatureTooHigh { node: NodeID, index: usize },
}

impl From<DbErr> for OperationError {
    fn from(value: DbErr) -> Self {
        OperationError::Model(ModelError::Database(value))
    }
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    Operation(#[from] OperationError),

    #[error("authorization error <- uid::{0} or password")]
    AuthError(i32),

    #[error("authorization header error")]
    AuthHeader,
}

impl From<DbErr> for ApiError {
    fn from(value: DbErr) -> Self {
        ApiError::Operation(OperationError::Model(ModelError::Database(value)))
    }
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    Database(#[from] DbErr),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Config(#[from] toml::de::Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    PgEmbed(#[from] PgEmbedError),
}
