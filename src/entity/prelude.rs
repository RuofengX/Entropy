use sea_orm::{ConnectionTrait, DbConn, Schema, TransactionTrait};
use tracing::instrument;

use crate::err::RuntimeError;

#[instrument(skip(db), ret, err)]
pub async fn ensure_database_schema(db: &DbConn) -> Result<(), RuntimeError> {
    // Setup Schema helper
    let schema = Schema::new(db.get_database_backend());

    // Derive from Entity
    let table_stmts = vec![
        schema.create_table_from_entity(super::node::Entity),
        schema.create_table_from_entity(super::player::Entity),
        schema.create_table_from_entity(super::guest::Entity),
    ];
    let index_stmts = vec![
        schema.create_index_from_entity(super::node::Entity),
        schema.create_index_from_entity(super::player::Entity),
        schema.create_index_from_entity(super::guest::Entity),
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
    super::node::Model::prepare_origin(&txn).await?;
    txn.commit().await?;
    Ok(())
}
