pub use dblib_macros::*;

#[macro_export]
macro_rules! model {
    ($name:ident, $($field:ident:$type:tt),*) => {
        #[derive(Debug, Clone, Serialize, Deserialize, QueryParams)]
        pub struct $name {
            id: dblib::RID,
            $($field: $type),*
        }
    };
}
