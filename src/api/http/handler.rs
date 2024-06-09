use axum::extract::State;
use axum::Json;
use axum_auth::AuthBasic;
use serde::Deserialize;

use crate::err::ApiError;
use crate::entities;

use super::AppState;
use crate::entities::guest::Model as Guest;
use crate::entities::node::Model as Node;
use crate::entities::player::Model as Player;

#[derive(Debug, Deserialize)]
pub struct PlayerRegister {
    name: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct PlayerAuth {
    id: i32,
    password: String,
}

fn verify_header(auth: (String, Option<String>)) -> Result<PlayerAuth, ApiError> {
    let (id, password) = auth;
    let password = password.ok_or(ApiError::AuthHeader)?;
    if let Ok(id) = id.parse::<i32>() {
        Ok(PlayerAuth { id, password })
    } else {
        Err(ApiError::AuthHeader)
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(PlayerRegister { name, password }): Json<PlayerRegister>,
) -> Result<Json<Player>, ApiError> {
    Ok(Json(
        entities::register_player(&state.conn, name, password).await?,
    ))
}

pub async fn get_player(
    State(state): State<AppState>,
    AuthBasic(auth): AuthBasic,
) -> Result<Json<Player>, ApiError> {
    let PlayerAuth { id, password } = verify_header(auth)?;

    Ok(Json(
        entities::get_player(&state.conn, id, password)
            .await?
            .ok_or(ApiError::AuthError(id))?,
    ))
}
