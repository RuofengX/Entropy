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
        // soul api
        .route("/player", get(handler::get_player))
        .route("/player", post(handler::register))
        // // node api
        // .route("/node/:x/:y", get(api::node::get_json))
        // .route("/node/bytes/:x/:y", get(api::node::get_bytes))
        // // guest api
        // .route("/guest", get(api::guest::get))
        // .route("/guest/contain", get(api::guest::contain))
        // .route("/guest/walk", post(api::guest::walk))
        // .route("/guest/harvest", post(api::guest::harvest))
        // .route("/guest/heat", post(api::guest::heat))
        // other thing
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
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")).into_response() // change this into json
    }
}

#[derive(Clone)]
pub struct AppState {
    pub conn: DbConn,
}
