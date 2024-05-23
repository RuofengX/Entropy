use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::{IntoResponse, Response},
};
use axum_auth::AuthBasic;

use super::verify;
use crate::{soul::WonderingSoul, world::World};

pub async fn stream(
    AuthBasic((uid, pw_hash)): AuthBasic,
    ws: WebSocketUpgrade,
    State(world): State<Arc<World>>,
) -> Response {
    match verify::verify_soul(&world, &uid, pw_hash).await {
        // WonderingSoul<'w> cannot move between threads.
        // So pass world here.
        Ok(_) => ws.on_upgrade(|socket| ws_main(socket, world, uid)),
        Err(e) => e.into_response(),
    }
}

pub async fn ws_main<'w>(mut socket: WebSocket, world: Arc<World>, uid: String) {
    let w_soul = world.get_wondering_soul(&uid).await.unwrap().unwrap();
    while let Some(Ok(msg)) = socket.recv().await {
        let rtn = match msg {
            Message::Binary(b) => binary_handler(&w_soul, b).await,
            Message::Text(t) => text_handler(&w_soul, t).await,
            Message::Ping(_) => Some(Message::Pong(vec![])),
            Message::Pong(_) => None,
            Message::Close(_) => {
                graceful_close_socket(socket);
                return;
            }
        };
        if let Some(rtn) = rtn {
            socket.send(rtn).await;
        } else {
            continue;
        };
        todo!()
    }
    graceful_close_socket(socket).await;
}

async fn binary_handler<'w>(w_soul: &WonderingSoul<'w>, b: Vec<u8>) -> Option<Message> {
    todo!()
}

async fn text_handler<'w>(w_soul: &WonderingSoul<'w>, t: String) -> Option<Message> {
    todo!()
}

async fn struct_handler<'w>(w_soul: &WonderingSoul<'w>, command: String) -> Option<Message> {
    todo!()
}

async fn graceful_close_socket(mut socket: WebSocket) {
    socket.send(Message::Close(None)).await;
    socket.close().await;
}
