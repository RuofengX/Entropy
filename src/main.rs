pub mod err;
mod model;
pub mod node;

use sea_orm::{prelude::*, Database, Schema};

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    println!("Hello, world!");
    // Connecting SQLite
    let db = Database::connect("postgres://postgres:1677@localhost:5432/entropy_test").await?;

    // Setup database schema
    setup_schema(&db).await?;
    Ok(())
}

async fn setup_schema(db: &DbConn) -> Result<(), DbErr> {
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
        db.execute(db.get_database_backend().build(i.if_not_exists())).await?;
    }
    // for i in index_stmts {
    //     for j in i {
    //         db.execute(db.get_database_backend().build(&j)).await?;
    //     }
    // }
    Ok(())
}
