use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{
    db::DB,
};

use shared::models::{User, UserQueryParams};

use actix_web::{get, web, App, HttpServer, Responder};
mod db;

#[get("/")]
async fn info(db: web::Data<Arc<Mutex<DB>>>) -> impl Responder {
    db.clone().lock().unwrap().info_string()
}

#[get("/canadians")]
async fn canadians(db: web::Data<Arc<Mutex<DB>>>) -> impl Responder {
    let db = DB::unlock(&db);
    let query = UserQueryParams::builder().country("Canada").wrap();

    let users = db.users.filter(|user| user.matches_criteria(&query));
    serde_json::to_string(&users)
}

#[get("/get_users")]
async fn get_users(
    db: web::Data<Arc<Mutex<DB>>>,
    query: web::Query<UserQueryParams>,
) -> impl Responder {
    let db = DB::unlock(&db);

    let builder = db
        .users
        .filter_builder()
        .filter(|user| user.matches_criteria(&query));

    let users: Vec<_> = builder.collect();
    println!("{users:?} {query:?}");

    serde_json::to_string(&users)
}

struct Dispatcher {
    instances: Vec<Rc<str>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut rdr = csv::Reader::from_path("./test.csv").unwrap();
    let mut db = DB::new("test-a0c");

    for result in rdr.deserialize().skip(0) {
        let user: User = result?;
        let id = db.insert_user(user.clone());
        println!("{id:?}: {user:?}, {}", db.get_document_count());
    }

    /* let queryResult = db.users.filter(|user| user.first_name == "Leota");
    println!("{queryResult:?}"); */

    let db = Arc::new(Mutex::new(db));

    HttpServer::new(move || {
        App::new()
            .service(info)
            .service(get_users)
            .service(canadians)
            .app_data(web::Data::new(db.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}