use sea_orm::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("backend database error <- {0}")]
    Database(#[from] DbErr),
    #[error("error while parse model <- {desc}")]
    Parse { desc: String },
}

#[derive(Error, Debug)]
pub enum OperationError {
    #[error(transparent)]
    Model(#[from] ModelError),
    #[error("energy not enough")]
    EnergyNotEnough {
        energy_reserve: u64,
        energy_required: u64,
        operation: &'static str,
    },
}

impl From<DbErr> for OperationError {
    fn from(value: DbErr) -> Self {
        OperationError::Model(ModelError::Database(value))
    }
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    Operation(#[from] OperationError),

    #[error("authorization error <- uid::{0} or password")]
    AuthError(i32),

    #[error("authorization header error")]
    AuthHeader,
}


#[derive(Debug, Error)]
pub enum RuntimeError{
    #[error(transparent)]
    Database(#[from] DbErr),
    #[error(transparent)]
    IO(#[from] std::io::Error)
}

