use crate::models::UserQueryParams;
use dblib::macros::data_shard;
use models::*;

pub mod models;

data_shard!(User);
