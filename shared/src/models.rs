use dblib_macros::QueryParams;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, QueryParams)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub country: String,
    pub address: String,
    pub city: String,
}

impl User {
    pub fn new(
        first_name: String,
        last_name: String,
        country: String,
        address: String,
        city: String,
    ) -> Self {
        Self {
            first_name,
            last_name,
            country,
            address,
            city,
        }
    }
}
