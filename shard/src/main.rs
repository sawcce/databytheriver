use std::env;

use libloading::{Library, Symbol};
use once_cell::sync::OnceCell;

use actix_web::{web::ServiceConfig, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    static LIB: OnceCell<Library> = OnceCell::new();

    let mut args = env::args().skip(1);
    let path = args.next().unwrap();
    let id = args.next().unwrap();
    let port = args.next().unwrap();
    let lib = args.next().unwrap();

    println!("{} {} {} {}", path, id, port, lib);

    let configure = unsafe {
        let lib = LIB.get_or_init(|| Library::new(lib).unwrap());
        let setup_func: Symbol<unsafe extern "C" fn(&mut ServiceConfig)> =
            lib.get(b"setup_shard").unwrap();
        setup_func
    };

    HttpServer::new(move || {
        let app = App::new();

        app.configure(|sc| unsafe { configure(sc) })
    })
    .bind(format!("[::1]:{}", port))?
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
