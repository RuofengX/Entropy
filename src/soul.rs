use dashmap::DashSet;
use futures::{future, TryFutureExt};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

use crate::{
    db::SaveStorage,
    guest::{self, Guest, GID},
    node::{direction::Direction, NodeID},
    world::World,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Soul {
    pub uid: String,
    pub username: String,
    password: String,
    guest_quota: u64,
    guests: DashSet<GID, ahash::RandomState>,
}

#[derive(Error, Debug)]
pub enum SoulError {
    #[error("GID::{0:?} is not recorded in soul's memory")]
    GuestNotConnected(GID),
    #[error("guest with GID::{0:?} is recorded in soul's memory, but not found in physical world")]
    GuestNotExistInWorld(GID),
    #[error("guest quota::{0} has been exceeded")]
    GuestQuotaExceeded(u64),
    #[error(transparent)]
    GuestError(#[from] guest::GuestError),
}
