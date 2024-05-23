use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("auth error, wrong uid::{0} or password")]
    AuthError(String),

    #[error("no password found in request header")]
    EmptyPassword,

    /// all the other error is from backend
    #[error(transparent)]
    BackendError(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;
