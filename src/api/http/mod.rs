use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use sea_orm::DbConn;

use crate::err::{ApiError, RuntimeError};
pub mod handler;

pub async fn start_http_service(address: &'static str, db: &DbConn) -> Result<(), RuntimeError> {
    let state = AppState { conn: db.clone() };

    let router = Router::new()
        .route("/player", post(handler::register))
        .route("/player", get(handler::get_player))
        .route("/player/guest", get(handler::list_guest))
        .route("/player/guest/spawn", get(handler::spawn_guest))
        .route("/node/:x/:y", get(handler::get_node))
        .route("/node/bytes/:x/:y", get(handler::get_node_bytes))
        .route("/guest/:id", get(handler::get_guest))
        .route("/guest/walk/:id", post(handler::walk))
        .route("/guest/harvest/:id", post(handler::harvest))
        .route("/guest/arrange/:id", post(handler::arrange))
        // .route("/guest/heat", post(todo!()))
        .with_state(state);

    println!("apt::http >> http server listening at {}", address);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    Ok(axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            println!("api::http >> \n");
            println!("api::http >> stop signal caught");
        })
        .await?)
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::NOT_ACCEPTABLE, format!("{self}")).into_response() // change this into json
    }
}

#[derive(Clone)]
pub struct AppState {
    pub conn: DbConn,
}
