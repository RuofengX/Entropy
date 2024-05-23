use std::sync::Arc;

use axum::{
    debug_handler,
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    }, response::{IntoResponse, Response},
};
use axum_auth::AuthBasic;

use super::verify;
use crate::world::World;

#[debug_handler]
pub async fn stream(
    AuthBasic((uid, pw_hash)): AuthBasic,
    ws: WebSocketUpgrade,
    State(world): State<Arc<World>>,
) -> Response {
    match verify::verify_soul(&world, &uid, pw_hash).await{
        Ok(_) => ws.on_upgrade(|socket| ws_main(socket, world)),
        Err(e) => e.into_response()
    }
}

pub async fn ws_main(mut socket: WebSocket, world: Arc<World>) {
    async {
        while let Some(msg) = socket.recv().await {
            let msg = msg?;
            todo!()
        }
        Ok::<(), anyhow::Error>(())
    }
    .await
    .expect("client closed connection") // a block with ? sugar
}
