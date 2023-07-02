use std::{env, fmt::format, ops::DerefMut, rc::Rc, sync::Arc};

use dblib::macros::data_shard;
use futures::lock::Mutex;
use shared::models::{User, UserQueryParams};

use actix_web::{dev::HttpServiceFactory, get, web, App, HttpServer, Responder};

data_shard!(User);

#[get("/get_users")]
async fn get_users(
    db: web::Data<Arc<Mutex<DataShard>>>,
    query: web::Query<UserQueryParams>,
) -> impl Responder {
    let db = db.clone();
    let db = db.lock().await;

    let builder = db
        .user_repo
        .filter_builder()
        .filter(|user| user.matches_criteria(&query));

    let users: Vec<_> = builder.collect();
    println!("{users:?} {query:?}");

    serde_json::to_string(&users)
}

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
    }

    let db = Arc::new(Mutex::new(shard));

    HttpServer::new(move || {
        let mut app = App::new().service(get_users);

        for service in db.clone().try_lock().unwrap().get_services() {
            app = app.service(service);
        }

        app.app_data(web::Data::new(db.clone()))
    })
    .bind(format!("[::1]:{}", port))?
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
