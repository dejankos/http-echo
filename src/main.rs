#[macro_use]
extern crate log;

use std::borrow::BorrowMut;

use std::sync::Mutex;

use actix_web::{get, HttpMessage};

use actix_web::middleware::Logger;
use actix_web::post;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use log::LevelFilter;
use simplelog::{Config, TermLogger, TerminalMode};

use crate::cache::{CacheManager, CachedRequest};

mod cache;

#[get("/push/*")]
async fn push_get(manager: web::Data<Mutex<CacheManager>>, req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req))
}

#[post("/push/*")]
async fn push_post(manager: web::Data<Mutex<CacheManager>>, req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req))
}

#[get("/poll/*")]
async fn poll_get(mut manager: web::Data<Mutex<CacheManager>>, req: HttpRequest) -> HttpResponse {
    let mut man = manager.borrow_mut().lock().unwrap();
    HttpResponse::Ok().json(man.retrieve(&req.path().to_string()))
}

fn store_data(mut manager: web::Data<Mutex<CacheManager>>, req: HttpRequest) -> CachedRequest {
    let mut man = manager.borrow_mut().lock().unwrap();
    man.store(req)
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
            .service(poll_get)
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
