
use crate::guest::GID;

pub struct Soul{
    pub id: u64,
    pub username: String,
    password: String,
    connected_guest: Vec<GID>,
}