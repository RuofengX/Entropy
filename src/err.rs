use sea_orm::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Data out of range. Detail:{desc}")]
    OutOfRange { desc: &'static str },
    #[error("Backend database error.")]
    Database(#[from] DbErr),
}

pub enum GuestError {
    EnergyNotEnough {
        energy_reserve: u64,
        energy_required: u64,
        operation: &'static str,
    },
}

pub enum PlayerError {}
