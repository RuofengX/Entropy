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
        // soul
        .route("/register", post(api::soul::register))
        // node
        .route("/node/:x/:y", get(api::node::get_json))
        .route("/node/json/:x/:y", get(api::node::get_json))
        .route("/node/bytes/:x/:y", get(api::node::get_bytes))
        // guest
        .route("/guest", get(api::guest::get))
        .route("/guest/exist", get(api::guest::contain))
        .route("/guest/walk", post(api::guest::walk))
        .route("/guest/harvest", post(api::guest::harvest))
        .with_state(world.clone());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            println!("stop signal caught");
        })
        .await
        .unwrap();
}
