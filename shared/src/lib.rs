use crate::models::UserQueryParams;
use dblib::macros::data_shard;
use models::User;

pub mod models;

data_shard!(User);

#[no_mangle]
pub fn test_1() -> u32 {
    10
}
