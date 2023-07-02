use log::info;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use shared::models::{User, UserQueryParams};
use std::{ops::Deref, sync::Arc};

use futures::{future::join_all, lock::Mutex};

use actix_web::{self, get, middleware::Logger, web, App, HttpServer, Responder};
use env_logger::Env;
use reqwest::{self, Client};

#[derive(Debug, Clone)]
pub struct Dispatcher {
    instances: Vec<Arc<str>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct DispatchResult<T>(Vec<T>);

impl<T> IntoIterator for DispatchResult<T> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Deref for DispatchResult<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.0[..]
    }
}

impl Dispatcher {
    pub async fn execute_query<'de, T: DeserializeOwned + Clone>(&self, url: &String) -> Vec<T> {
        let mut futures = Vec::with_capacity(self.instances.len());

        for instance in self.instances.iter().clone() {
            futures.push(async {
                let client = Client::new();

                client
                    .get(format!("http://{}/{}", instance.clone(), &url))
                    .send()
                    .await
                    .unwrap()
                    .json::<DispatchResult<T>>()
                    .await
                    .unwrap()
                    .0
            });
        }

        let result: Vec<T> = join_all(futures.into_iter())
            .await
            .into_iter()
            .flatten()
            .collect();
        result.clone()
    }
}

#[get("/get_user")]
pub async fn get_users(
    dispatcher: web::Data<Arc<Mutex<Dispatcher>>>,
    query: web::Query<UserQueryParams>,
) -> actix_web::Result<impl Responder> {
    let query = serde_urlencoded::to_string(query.0).unwrap();
    println!("{query}");

    let res = dispatcher
        .clone()
        .lock()
        .await
        .execute_query::<User>(&format!("get_user?{}", query))
        .await;

    Ok(serde_json::to_string(&res))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let instances = std::env::var("INSTANCES")
        .unwrap()
        .split(";")
        .map(|addr| Arc::from(addr))
        .collect::<Vec<_>>();

    let dispatcher = Dispatcher {
        instances: instances.clone(),
    };
    let dispatcher = Arc::new(Mutex::new(dispatcher));

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    info!("Started with {} instance(s):", instances.len());

    for instance in instances {
        info!("Instance: {instance}");
    }

    HttpServer::new(move || {
        App::new()
            .service(get_users)
            .app_data(web::Data::new(dispatcher.clone()))
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
