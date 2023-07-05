use dblib::macros::{model, QueryParams};
use serde::{Deserialize, Serialize};

model! {
    User:
        first_name: String,
        last_name: String,
        country: String,
        address: String,
        city: String
}
