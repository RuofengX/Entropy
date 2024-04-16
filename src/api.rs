use std::sync::Arc;

use axum::{extract::State, response::Response};

use crate::{db::SaveStorage, err::Result, guest::GID, world::World};

pub async fn contains_guest<S: SaveStorage> (
    State(world): State<Arc<World<S>>>,
    uid: String,
    id: GID,
) -> Response<T> {
    let s = world.get_soul(uid).await?;
    Ok(s.contains_guest(id))
}
