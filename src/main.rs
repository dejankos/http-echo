#[macro_use]
extern crate log;

use std::borrow::BorrowMut;
use std::sync::Mutex;

use actix_web::{connect, delete, get, head, options, patch, post, put, trace};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use actix_web::middleware::Logger;
use log::LevelFilter;
use simplelog::{Config, TerminalMode, TermLogger};

use crate::cache::{CachedRequest, CacheManager};

mod util;

mod cache;

#[get("/push/*")]
async fn push_get(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[post("/push/*")]
async fn push_post(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[put("/push/*")]
async fn push_put(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[delete("/push/*")]
async fn push_delete(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[head("/push/*")]
async fn push_head(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[connect("/push/*")]
async fn push_connect(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[options("/push/*")]
async fn push_options(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[trace("/push/*")]
async fn push_trace(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

#[patch("/push/*")]
async fn push_patch(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: web::Bytes,
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
    body: web::Bytes,
) -> CachedRequest {
    let mut man = manager.borrow_mut().lock().unwrap();
    man.store(req, body)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap();

    let global_state = web::Data::new(Mutex::new(CacheManager::new()));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
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
        .bind("127.0.0.1:8088")?
        .shutdown_timeout(5)
        .run()
        .await
}
