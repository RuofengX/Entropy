use axum::{http::StatusCode, response::{IntoResponse, Response}};
use utoipa::ToSchema;
use thiserror::Error;

#[derive(Debug, Error, ToSchema)]
pub enum ApiError {
    #[error("auth error, wrong uid::{0} or password")]
    #[schema(example = "Wrong uid or password.")]
    AuthError(String),

    #[error("no password found in request header")]
    #[schema(example = "No password were given.")]
    EmptyPassword,

    /// all the other error is from backend
    #[error(transparent)]
    #[schema(value_type = Object, example = "Error from internal.")]
    BackendError(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;
