use dblib::QueryParams;
use serde::{Deserialize, Serialize};
use std::{
    cell::Ref,
    collections::{hash_map::Values, HashMap},
    fmt::Debug,
    marker::PhantomData,
    ops::Index,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};
use uuid::Uuid;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct RID(Arc<str>);

impl RID {
    pub fn new(id: impl ToString) -> Self {
        Self(Arc::from(id.to_string()))
    }
}

impl From<&dyn ToString> for RID {
    fn from(value: &dyn ToString) -> Self {
        RID::new(value.to_string())
    }
}

struct Reference<'d, T> {
    uuid: RID,
    data: &'d HashMap<RID, T>,
    phantom: PhantomData<T>,
}

impl<'d, T: Debug> Reference<'d, T> {
    pub fn new(uuid: RID, data: &'d HashMap<RID, T>) -> Self {
        Self {
            uuid,
            data,
            phantom: PhantomData {},
        }
    }

    pub fn get(&self) -> Option<&'d T> {
        self.data.get(&self.uuid)
    }
}

//type Predicate<'a, T> = impl Fn(&'a T) -> bool;

pub struct Repository<T>
where
    T: Serialize,
{
    data: HashMap<RID, T>,
}

impl<T> Repository<T>
where
    T: Serialize,
{
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get_all(&self) -> Vec<&T> {
        self.data.values().collect()
    }

    pub fn filter_builder(&self) -> Values<RID, T> {
        self.data.values()
    }

    pub fn filter<'a>(&self, predicate: impl Fn(&T) -> bool) -> Vec<&T> {
        self.data
            .values()
            .filter(|value| predicate(value))
            .collect::<Vec<_>>()
    }
}

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

trait CompareProperty {}

struct OneToOne<'d, F: Debug, T: Debug> {
    from: Reference<'d, F>,
    to: Reference<'d, T>,
}

impl<'d, F: Debug, T: Debug> Debug for OneToOne<'d, F, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.from.get().unwrap();
        let to = self.to.get().unwrap();

        write!(f, "{from:?} -> {to:?}")
    }
}

impl<'d, F: Debug, T: Debug> OneToOne<'d, F, T> {
    pub fn new(
        from: RID,
        to: RID,
        from_data: &'d HashMap<RID, F>,
        to_data: &'d HashMap<RID, T>,
    ) -> Self {
        Self {
            from: Reference::new(from, from_data),
            to: Reference::new(to, to_data),
        }
    }
}

type Follows<'d> = OneToOne<'d, User, User>;

pub struct DB {
    id: RID,
    document_count: usize,
    pub users: Repository<User>,
    //follows: Repository<Follows<'f>>,
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
        self.users.data.insert(rid.clone(), user);
        let concatenated = self.id.0.to_string() + ":" + &rid.0;
        RID::new(concatenated)
    }
}
