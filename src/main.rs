#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use actix_web::{App, HttpMessage, HttpRequest, HttpResponse, HttpServer, web};
use actix_web::get;
use actix_web::http::{HeaderMap, HeaderName, HeaderValue};
use actix_web::http::header::ToStrError;
use actix_web::middleware::Logger;
use log::LevelFilter;
use simplelog::{Config, TerminalMode, TermLogger};

use crate::cache::{CachedRequest, to_cached_request, Cache};

mod cache;

struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

#[get("/push/*")]
async fn push_get(data: web::Data<Cache>, req: HttpRequest) -> HttpResponse {
    //store_data(data, req);

    HttpResponse::Ok().json(to_cached_request(req))
}

fn store_data(data: web::Data<Cache>, _req: HttpRequest) {
    let c = data.cache.lock().unwrap();

    // let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    // *counter += 1; // <- access counter inside MutexGuard
}

// time = 1596040068821
// header map = HeaderMap { inner: {"accept": One("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"), "sec-fetch-user": One("?1"), "accept-language": One("en-GB,en;q=0.9,hr-HR;q=0.8,hr;q=0.7,en-US;q=0.6,bs;q=0.5"), "sec-fetch-mode": One("navigate"), "sec-fetch-site": One("none"), "accept-encoding": One("gzip, deflate, br"), "cookie": One("_ga=GA1.1.833966493.1591460296"), "host": One("localhost:8088"), "user-agent": One("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/83.0.4103.116 Safari/537.36"), "connection": One("keep-alive"), "upgrade-insecure-requests": One("1"), "sec-fetch-dest": One("document")} }
// version = HTTP/1.1
// method = GET
// query_string = a=x&b=3
// ip = 127.0.0.1:52908
// scheme = http
// path = /push/1/1


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(AppStateWithCounter {
                counter: Mutex::new(0),
            }))
            .service(push_get)
    })
        .bind("127.0.0.1:8088")?
        .run()
        .await
}
