use std::env;

use libloading::{Library, Symbol};
use once_cell::sync::OnceCell;

use actix_web::{
    get,
    web::{self, ServiceConfig},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};

use dblib::{
    actix::{Actor, StreamHandler},
    actix_web_actors::ws,
};

struct Ws;

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Ws {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(Ws {}, &req, stream);
    println!("{:?}", resp);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

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
        let app = App::new().route("/ws/", web::get().to(index));

        app.configure(|sc| unsafe { configure(sc) })
    })
    .bind(format!("[::1]:{}", port))?
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
