mod db;
mod err;
mod guest;
pub mod node;
pub mod soul;
pub mod world;
pub mod api;

use std::sync::Arc;

use axum::{extract::State, routing::get, Router};
use db::SledStorage;
use soul::WonderingSoul;
use world::World;

#[tokio::main]
async fn main() {
    let sled_db = SledStorage::new("entropy.sled".into(), false).unwrap();
    let shared_world = Arc::new(World::new(sled_db));
    let app = Router::new()
        .route("/contains", get(api::contains_guest))
        .with_state(shared_world);
}
