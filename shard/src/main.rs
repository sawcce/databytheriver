use std::{
    env,
    fmt::format,
    ops::DerefMut,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::db::DB;

use dblib::macros::data_shard;
use shared::models::{User, UserQueryParams};

use actix_web::{dev::HttpServiceFactory, get, web, App, HttpServer, Responder};
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

data_shard!(User);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut args = env::args().skip(1);
    let path = args.next().unwrap();
    let id = args.next().unwrap();
    let port = args.next().unwrap();

    println!("{} {} {}", path, id, port);

    let mut rdr = csv::Reader::from_path(path).unwrap();

    let mut shard = DataShard::new(id.clone());

    for result in rdr.deserialize().skip(0) {
        let user: User = result?;
        shard.insert_user(user.clone());
        println!("{:?}: {user:?}", user.id);
    }

    /* let queryResult = db.users.filter(|user| user.first_name == "Leota");
    println!("{queryResult:?}"); */
    //let services = shard.get_services();
    let db = Arc::new(Mutex::new(shard));

    HttpServer::new(move || {
        let mut app = App::new()
            .service(info)
            .service(get_users)
            .service(canadians);

        for service in db.clone().lock().unwrap().get_services() {
            app = app.service(service);
        }

        app.app_data(web::Data::new(db.clone()))
    })
    .bind(format!("[::1]:{}", port))?
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
