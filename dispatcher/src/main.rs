use serde::{de::DeserializeOwned, Deserialize, Serialize};
use shared::models::{User, UserQueryParams};
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use actix_web::{self, get, web, App, HttpServer, Responder};
use reqwest::{self, Client};

#[derive(Debug, Clone)]
pub struct Dispatcher {
    instances: Vec<Arc<str>>,
}

#[derive(Serialize, Deserialize)]
struct DispatchResult<T>(Vec<T>);

impl Dispatcher {
    pub async fn execute_query<'de, T: DeserializeOwned>(&self, url: &String) -> DispatchResult<T> {
        let mut result = Vec::new();
        let client = Client::new();

        for instance in self.instances.iter().clone() {
            let mut a = client
                .get(format!("http://{}/{}", instance, url.clone()))
                .send()
                .await
                .unwrap()
                .json::<DispatchResult<T>>()
                .await
                .unwrap();

            result.append(&mut a.0);
        }

        DispatchResult(result)
    }
}

#[get("/get_users")]
pub async fn get_users(
    dispatcher: web::Data<Arc<Mutex<Dispatcher>>>,
    query: web::Query<UserQueryParams>,
) -> impl Responder {
    let query = serde_urlencoded::to_string(query.0).unwrap();
    println!("{query}");

    let res = dispatcher
        .clone()
        .lock()
        .unwrap()
        .execute_query::<User>(&format!("get_users?{}", query))
        .await;

    serde_json::to_string(&res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let dispatcher = Dispatcher {
        instances: vec!["127.0.0.1:8080".into(), "127.0.0.1:8081".into()],
    };
    let dispatcher = Arc::new(Mutex::new(dispatcher));

    HttpServer::new(move || {
        App::new()
            .service(get_users)
            .app_data(web::Data::new(dispatcher.clone()))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
