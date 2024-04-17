use ordered_float::NotNan;

use crate::guest::GID;

pub(crate) type Result<T> = std::result::Result<T, anyhow::Error>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    DatabaseError(#[from] DatabaseError),

    #[error(transparent)]
    GuestError(#[from] GuestError),

    #[error(transparent)]
    NodeError(#[from] NodeError),

    #[error(transparent)]
    SoulError(#[from] SoulError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum DatabaseError {
    #[error(transparent)]
    SledError(#[from] sled::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum GuestError {
    // #[error("guest with GID::{0:?} not found in physical world")]
    // NotExist(GID),
    #[error("energy is not enough for operation::{op_name}, {require} needed, {left} left")]
    EnergyNotEnough {
        op_name: &'static str,
        require: NotNan<f32>,
        left: NotNan<f32>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    // #[error("node with NodeID::{0:?} not found in physical world")]
    // NotExist(NodeID),
}

#[derive(Debug, thiserror::Error)]
pub enum SoulError {
    // #[error("Soul with uid::{0} not exists")]
    // NotExist(String),
    #[error("GID::{0:?} is not recorded in soul's memory")]
    GuestNotConnected(GID),

    #[error("guest quota::{0} has been exceeded")]
    GuestQuotaExceeded(u64),
}
