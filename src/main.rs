pub mod err;
pub mod grid;
pub mod model;

use sea_orm::{prelude::*, Database, Schema};

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let db = Database::connect("postgres://postgres:1677@localhost:5432/entropy").await?;

    match create_schema_test(&db).await {
        Ok(_) => {}
        Err(e) => println!("create schema failed, {:?}", e),
    };
    Ok(())
}

async fn create_schema_test(db: &DbConn) -> Result<(), DbErr> {
    // Setup Schema helper
    let schema = Schema::new(db.get_database_backend());

    // Derive from Entity
    let table_stmts = vec![
        schema.create_table_from_entity(model::node::Entity),
        schema.create_table_from_entity(model::player::Entity),
        schema.create_table_from_entity(model::guest::Entity),
    ];
    let index_stmts = vec![
        schema.create_index_from_entity(model::node::Entity),
        schema.create_index_from_entity(model::player::Entity),
        schema.create_index_from_entity(model::guest::Entity),
    ];

    for mut i in table_stmts {
        db.execute(db.get_database_backend().build(i.if_not_exists()))
            .await?;
    }
    for mut i in index_stmts.into_iter().flatten() {
        db.execute(db.get_database_backend().build(i.if_not_exists()))
            .await?;
    }
    Ok(())
}
