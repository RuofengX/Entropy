use std::sync::Arc;

use axum::{
    async_trait, debug_handler,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

use crate::{guest::GID, soul::Soul, world::World};

type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("error when auth::{:?}", .0)]
    AuthError(SoulCred),

    #[error("error when serialize/deserialize json data")]
    JsonParseError(#[from] serde_json::Error),

    /// all the other error is from backend
    #[error(transparent)]
    BackendError(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")).into_response()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SoulCred {
    pub uid: String,
    pub pw_hash: Vec<u8>,
}

impl AuthUser for Soul {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.name.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.pw_hash
    }
}

#[async_trait]
impl AuthnBackend for World {
    /// Authenticating user type.
    type User = Soul;

    /// Credential type used for authentication.
    type Credentials = SoulCred; // uid

    ///" An error which can occur during authentication and authorization."
    type Error = ApiError;

    /// Authenticates the given credentials with the backend.
    async fn authenticate(&self, cred: Self::Credentials) -> Result<Option<Self::User>> {
        Ok(if self.varify_soul(&cred).await.unwrap_or(false) {
            self.get_soul(&cred.uid).await?
        } else {
            None
        })
    }

    /// Gets the user by provided ID from the backend.
    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>> {
        Ok(self.get_soul(user_id).await?)
    }
}

#[derive(Debug, Deserialize)]
pub struct SoulGuestIndex {
    uid: String,
    gid: GID,
}

#[debug_handler]
pub(crate) async fn register_soul(
    Query(soul): Query<SoulCred>,
    State(world): State<Arc<World>>,
) -> Result<Json<Value>> {
    Ok(Json(serde_json::to_value(
        world.register_soul(soul.uid, soul.pw_hash).await?,
    )?))
}

#[debug_handler]
pub(crate) async fn contains_guest(
    Query(guest): Query<SoulGuestIndex>,
    State(world): State<Arc<World>>,
) -> Result<Json<Option<Value>>> {
    Ok(
        if let Some(wondering_soul) = world.get_wondering_soul(&guest.uid).await? {
            Json(Some(serde_json::to_value(
                wondering_soul.contains_guest(guest.gid),
            )?))
        } else {
            Json(None)
        },
    )
}
