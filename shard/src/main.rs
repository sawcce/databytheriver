use std::{env, sync::Arc};

use dblib::macros::data_shard;
use futures::lock::Mutex;
use shared::models::{User, UserQueryParams};

use actix_web::{
    web::{self, ServiceConfig},
    App, HttpServer,
};

data_shard!(User);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut args = env::args().skip(1);
    let path = args.next().unwrap();
    let id = args.next().unwrap();
    let port = args.next().unwrap();
    let lib = args.next().unwrap();

    println!("{} {} {} {}", path, id, port, lib);

    let configure = unsafe {
        let lib = libloading::Library::new(lib).unwrap();
        let setup_func: libloading::Symbol<unsafe fn() -> fn(&mut ServiceConfig)> =
            lib.get(b"setup_shard").unwrap();
        setup_func()
    };

    HttpServer::new(move || {
        let app = App::new();

        app.configure(|a| configure(a))
    })
    .bind(format!("[::1]:{}", port))?
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
