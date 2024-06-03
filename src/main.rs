pub mod err;
mod model;
pub mod node;

use sea_orm::{prelude::*, Database, Schema};

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    println!("Hello, world!");
    // Connecting SQLite
    let db = Database::connect(
        "postgres://postgres:1677@localhost:5432/entropy_test?currentSchema=my_schema",
    )
    .await?;

    // Setup database schema
    setup_schema(&db).await?;
    Ok(())
}

async fn setup_schema(db: &DbConn) -> Result<(), DbErr> {
    // Setup Schema helper
    let schema = Schema::new(db.get_database_backend());

    // Derive from Entity
    let stmts = vec![
        schema.create_table_from_entity(model::node::Entity),
        schema.create_table_from_entity(model::player::Entity),
        schema.create_table_from_entity(model::guest::Entity),
    ];

    for i in stmts {
        db.execute(db.get_database_backend().build(&i)).await?;
    }
    Ok(())
}
