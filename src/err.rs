use ordered_float::NotNan;

use crate::{guest::GID, node::NodeID};

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
    #[error("guest with GID::{0:?} not found in physical world")]
    NotExist(GID),

    #[error("energy is not enough for operation::{0}, {1} needed, {2} left")]
    EnergyNotEnough(&'static str, NotNan<f32>, NotNan<f32>),

}

#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("node with NodeID::{0:?} not found in physical world")]
    NotExist(NodeID),

    #[error(
        "node index::{0:?} overflow! node index is limited in i16, which should be in range [-32_768i16, 32_767i16]"
    )]
    IndexOverflow(NodeID),
}

#[derive(Debug, thiserror::Error)]
pub enum SoulError {
    #[error("Soul with uid::{0} not exists")]
    NotExist(String),

    #[error("GID::{0:?} is not recorded in soul's memory")]
    GuestNotConnected(GID),

    #[error("guest quota::{0} has been exceeded")]
    GuestQuotaExceeded(u64),
}
