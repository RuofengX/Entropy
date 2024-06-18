use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use sea_orm::DbConn;
use tracing::{instrument, warn};

use crate::err::{ApiError, RuntimeError};
pub mod handler;

#[instrument(skip(db))]
pub async fn http_service(address: String, db: &DbConn) -> Result<(), RuntimeError> {
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
        .route("/guest/detect/:id", get(handler::detect))
        // .route("/guest/heat", post(todo!()))
        .with_state(state);

    warn!("http server listening at {}", address);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    Ok(axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            warn!("stop signal caught");
        })
        .await?)
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::NOT_ACCEPTABLE, format!("{self}")).into_response() // change this into json
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub conn: DbConn,
}
