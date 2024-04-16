use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use ordered_float::NotNan;
use serde::Serialize;

use crate::{guest::GID, node::NodeID};

pub(crate) type Result<T> = std::result::Result<T, anyhow::Error>;

pub(crate) enum AxumResponse<T: Serialize> {
    Ok(T),
    Err(anyhow::Error),
}
impl<T: Serialize> From<Result<T>> for AxumResponse<T> {
    fn from(value: Result<T>) -> Self {
        match value {
            Ok(x) => AxumResponse::Ok(x),
            Err(e) => AxumResponse::Err(e),
        }
    }
}
impl<T: Serialize> IntoResponse for AxumResponse<T> {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Ok(x) => Response::builder().status(200).body(
                Json(x)
            )?.,
            _
        }
    }
}

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

    #[error("energy is not enough for operation::{op_name}, {require} needed, {left} left")]
    EnergyNotEnough {
        op_name: &'static str,
        require: NotNan<f32>,
        left: NotNan<f32>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("node with NodeID::{0:?} not found in physical world")]
    NotExist(NodeID),
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
