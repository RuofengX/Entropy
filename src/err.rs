use sea_orm::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("backend database error")]
    Database(#[from] DbErr),
    #[error("error while parse model <- {desc}")]
    Parse { desc: String },
}

#[derive(Error, Debug)]
pub enum OperationError {
    #[error(transparent)]
    Model(ModelError),
    #[error("energy not enough")]
    EnergyNotEnough {
        energy_reserve: u64,
        energy_required: u64,
        operation: &'static str,
    },
}
