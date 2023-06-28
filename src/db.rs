use dblib::{Repository, RID};

use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;

use crate::models::User;

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
        format!("Shard: {}", self.id.0.to_string())
    }

    pub fn get_document_count(&self) -> usize {
        return self.document_count;
    }

    pub fn insert_user(&mut self, user: User) -> RID {
        self.document_count += 1;
        let rid = RID::new(Uuid::new_v4());
        self.users.data.push(user);
        let concatenated = self.id.0.to_string() + ":" + &rid.0;
        RID::new(concatenated)
    }
}
