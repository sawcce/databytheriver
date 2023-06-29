use std::sync::Arc;

use serde::{Deserialize, Serialize};
pub mod macros;

#[derive(Eq, PartialEq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct RID(Arc<str>);

impl RID {
    pub fn new(id: impl ToString) -> Self {
        Self(Arc::from(id.to_string()))
    }
}

impl ToString for RID {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<&dyn ToString> for RID {
    fn from(value: &dyn ToString) -> Self {
        RID::new(value.to_string())
    }
}

pub struct Repository<T>
where
    T: Serialize,
{
    data: Vec<T>,
}

impl<T> Repository<T>
where
    T: Serialize,
{
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn get_all(&self) -> &Vec<T> {
        &self.data
    }

    pub fn filter_builder(&self) -> core::slice::Iter<T> {
        self.data.iter()
    }

    pub fn filter<'a>(&self, predicate: impl Fn(&T) -> bool) -> Vec<&T> {
        self.data
            .iter()
            .filter(|value| predicate(value))
            .collect::<Vec<_>>()
    }

    pub fn push(&mut self, data: T) {
        self.data.push(data)
    }
}

pub struct DataShard<T> {
    id: RID,
    db: T,
}
