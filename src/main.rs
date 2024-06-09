pub mod err;
pub mod grid;
pub mod entity;
pub mod api;

use api::http::start_http_service;
use err::RuntimeError;
use entity::{check_database, node};
use sea_orm::{prelude::*, Database, Schema, TransactionTrait};

#[tokio::main]
async fn main() -> Result<(), RuntimeError>{
    let db = Database::connect("postgres://postgres@localhost:5432/entropy").await?;
    create_schema_test(&db).await?;
    println!("main >> checking database");
    check_database(&db).await?;
    println!("main >> database connected");
    start_http_service("0.0.0.0:3333", &db).await?;
    Ok(())
}

pub async fn create_schema_test(db: &DbConn) -> Result<(), RuntimeError> {
    // Setup Schema helper
    let schema = Schema::new(db.get_database_backend());

    // Derive from Entity
    let table_stmts = vec![
        schema.create_table_from_entity(entity::node::Entity),
        schema.create_table_from_entity(entity::player::Entity),
        schema.create_table_from_entity(entity::guest::Entity),
    ];
    let index_stmts = vec![
        schema.create_index_from_entity(entity::node::Entity),
        schema.create_index_from_entity(entity::player::Entity),
        schema.create_index_from_entity(entity::guest::Entity),
    ];

    for mut i in table_stmts {
        db.execute(db.get_database_backend().build(i.if_not_exists()))
            .await?;
    }
    for mut i in index_stmts.into_iter().flatten() {
        db.execute(db.get_database_backend().build(i.if_not_exists()))
            .await?;
    }
    let txn = db.begin().await?;
    node::Model::prepare_origin(&txn).await?;
    txn.commit().await?;
    Ok(())
}
