mod alphabet;
pub(crate) mod api;
mod db;
mod err;
mod guest;
pub mod node;
pub mod soul;
pub mod world;

use std::sync::Arc;

use axum::{routing::get, Router};
use db::Storage;
use world::World;

#[tokio::main]
async fn main() {
    let db = Storage::new("entropy.sled".into(), false).unwrap();
    let world = Arc::new(World::new(db));

    let router = Router::new()
        .route("/register", get(api::register_soul))
        .route("/exist_guest", get(api::contain_guest))
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
