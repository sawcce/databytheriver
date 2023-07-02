use dblib::{Repository, RID};

use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;

use shared::models::User;

pub struct DB {
    id: RID,
    document_count: usize,
    pub users: Repository<User>,
}

impl DB {
    pub fn new(id: impl ToString) -> Self {
        Self {
            id: RID::new(id),
            document_count: 0,
            users: Repository::new(),
        }
    }

    pub fn unlock<'a>(db: &'a Arc<Mutex<Self>>) -> MutexGuard<'a, DB> {
        db.lock().unwrap()
    }

    pub fn info_string(&self) -> String {
        format!("Shard: {}", self.id.to_string())
    }

    pub fn get_document_count(&self) -> usize {
        return self.document_count;
    }
}
