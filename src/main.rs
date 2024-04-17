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
    let sled_db = Storage::new("entropy.sled".into(), false).unwrap();
    let shared_world = Arc::new(World::new(sled_db));
    let router = Router::new()
        .route("/register", get(api::register_soul))
        .route("/contains", get(api::contains_guest))
        .with_state(shared_world);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn create_default() {
        let sled_db = Storage::new("entropy.sled".into(), false).unwrap();
        let shared_world = Arc::new(World::new(sled_db));
        if shared_world.get_guest(guest::GID(1)).await.is_ok() {
            println!("guest with id::1 exist");
        } else {
            let id = shared_world.spawn().await;
            println!("spawn guest with id::{:?}", id);
        }
    }
}
