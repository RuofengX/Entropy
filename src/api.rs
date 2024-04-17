use std::{convert::Infallible, sync::Arc};

use axum::{
    async_trait, debug_handler,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_login::{AuthUser, AuthnBackend, UserId};
use futures::TryFutureExt;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    db::{SaveStorage, SledStorage},
    err::Error,
    guest::GID,
    soul::Soul,
    world::World,
};

pub(crate) struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

type Result<T> = std::result::Result<T, ApiError>;

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
impl<S: SaveStorage + Clone> AuthnBackend for World<S> {
    /// Authenticating user type.
    type User = Soul;

    /// Credential type used for authentication.
    type Credentials = SoulCred; // uid

    ///" An error which can occur during authentication and authorization."
    type Error = Infallible;

    /// Authenticates the given credentials with the backend.
    async fn authenticate(
        &self,
        cred: Self::Credentials,
    ) -> std::result::Result<Option<Self::User>, Self::Error> {
        std::result::Result::Ok(if self.varify_soul(&cred).await.unwrap_or(false) {
            self.get_soul(&cred.uid).await.ok()
        } else {
            None
        })
    }

    /// Gets the user by provided ID from the backend.
    async fn get_user(
        &self,
        user_id: &UserId<Self>,
    ) -> std::result::Result<Option<Self::User>, Self::Error> {
        std::result::Result::Ok(self.get_soul(user_id).await.ok())
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
    State(world): State<Arc<World<SledStorage>>>,
) -> Result<Json<Value>> {
    Ok(Json(serde_json::to_value(
        world.register_soul(soul.uid, soul.pw_hash).await?,
    )?))
}

#[debug_handler]
pub(crate) async fn contains_guest(
    Query(guest): Query<SoulGuestIndex>,
    State(world): State<Arc<World<SledStorage>>>,
) -> Result<Json<Value>> {
    let s = world.get_wondering_soul(&guest.uid).await?;
    Ok(Json(serde_json::to_value(s.contains_guest(guest.gid))?))
}
