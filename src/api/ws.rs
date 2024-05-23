use std::sync::Arc;

use axum::{
    debug_handler,
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
};
use axum_auth::AuthBasic;

use super::verify;
use super::error::Result;
use crate::world::World;

#[debug_handler]
pub async fn stream(
    AuthBasic((uid, pw_hash)): AuthBasic,
    ws: WebSocketUpgrade,
    State(world): State<Arc<World>>,
) -> Result<()> {
    verify::verify_soul(&world, &uid, pw_hash).await?;
    ws.on_upgrade(|socket| ws_main(socket, world));
    Ok(())
}
pub struct Command {
    name: String,
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
