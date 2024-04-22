mod alphabet;
pub(crate) mod api;
mod db;
mod err;
mod guest;
pub mod node;
pub mod soul;
pub mod world;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use db::Storage;
use world::World;

#[tokio::main]
async fn main() {
    let db = Storage::new("entropy.sled".into(), false).unwrap();
    let world = Arc::new(World::new(db));

    let router = Router::new()
        // soul api
        .route("/register", post(api::soul::register))
        .route("/soul", get(api::soul::get))
        // node api
        .route("/node/:x/:y", get(api::node::get_json))
        .route("/node/json/:x/:y", get(api::node::get_json))
        .route("/node/bytes/:x/:y", get(api::node::get_bytes))
        // guest api
        .route("/guest", get(api::guest::get))
        .route("/guest/exist", get(api::guest::contain))
        .route("/guest/walk", post(api::guest::walk))
        .route("/guest/harvest", post(api::guest::harvest))
        .route("/guest/heat", post(api::guest::heat))
        // other thing
        .with_state(world.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            println!("\nstop signal caught");
        })
        .await
        .unwrap();
}
