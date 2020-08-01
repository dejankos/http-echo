#[macro_use]
extern crate log;

use std::borrow::BorrowMut;
use std::sync::Mutex;

use actix_web::body::{Body, ResponseBody};
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::middleware::Logger;
use actix_web::web::Bytes;
use actix_web::{connect, delete, dev, get, head, http, options, patch, post, put, trace};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use simplelog::{Config, TermLogger, TerminalMode};
use structopt::StructOpt;

use crate::cache::{CacheManager, CachedRequest};

mod util;

mod cache;

#[derive(Serialize, Deserialize)]
struct NotFound {
    msg: String,
}

#[derive(StructOpt)]
pub struct ServerConfig {
    #[structopt(short, long, help = "Server ip", default_value = "127.0.0.1")]
    ip: String,
    #[structopt(short, long, help = "Server port", default_value = "8080")]
    port: u16,
    #[structopt(
        short,
        long,
        help = "Server workers - default value is number of logical CPUs"
    )]
    workers: Option<usize>,
    #[structopt(short, long, help = "Cache TTL", default_value = "900000")]
    ttl: u64,
}

#[get("/push/*")]
async fn push_get(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[post("/push/*")]
async fn push_post(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[put("/push/*")]
async fn push_put(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[delete("/push/*")]
async fn push_delete(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[head("/push/*")]
async fn push_head(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[connect("/push/*")]
async fn push_connect(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[options("/push/*")]
async fn push_options(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[trace("/push/*")]
async fn push_trace(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[patch("/push/*")]
async fn push_patch(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[get("/poll/*")]
async fn poll_get(mut manager: web::Data<Mutex<CacheManager>>, req: HttpRequest) -> HttpResponse {
    let mut man = manager.borrow_mut().lock().unwrap();
    HttpResponse::Ok().json(man.retrieve(&req.path().to_string()))
}

fn store_data(
    mut manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: Bytes,
) -> CachedRequest {
    let mut man = manager.borrow_mut().lock().unwrap();
    man.store(req, body)
}

fn not_found<B>(mut res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    res.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/json"),
    );

    let path = res.request().path().to_string();
    let r = res.map_body(|_h, _b| {
        ResponseBody::Other(Body::from(
            if let Ok(json) = serde_json::to_string(&NotFound {
                msg: format!(
                    "This is not the path you're looking for, path = \"{}\".",
                    path
                ),
            }) {
                json
            } else {
                "Not Found".to_string()
            },
        ))
    });

    Ok(ErrorHandlerResponse::Response(r))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap();
    let args: ServerConfig = ServerConfig::from_args();

    let bind = format!("{}:{}", args.ip, args.port);
    let workers = args.workers.unwrap_or(num_cpus::get());
    let global_state = web::Data::new(Mutex::new(CacheManager::new(args.ttl)));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(ErrorHandlers::new().handler(http::StatusCode::NOT_FOUND, not_found))
            .app_data(global_state.clone())
            .service(push_get)
            .service(push_post)
            .service(push_put)
            .service(push_delete)
            .service(push_head)
            .service(push_connect)
            .service(push_options)
            .service(push_trace)
            .service(push_patch)
            .service(poll_get)
    })
    .bind(bind)?
    .workers(workers)
    .shutdown_timeout(10)
    .run()
    .await
}
