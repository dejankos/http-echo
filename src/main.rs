use std::collections::HashMap;


use std::sync::{Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use actix_web::get;
use actix_web::http::header::ToStrError;
use actix_web::http::{HeaderMap, HeaderName, HeaderValue};

use actix_web::{web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer};

use crate::cache::CachedRequest;

mod cache;

struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

#[get("/push/*")]
async fn push_get(_data: web::Data<AppStateWithCounter>, req: HttpRequest) -> HttpResponse {
    //store_data(data, req);

    if let Ok(stored) = to_cached_request(req) {
        HttpResponse::Ok().json(stored)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

fn store_data(data: web::Data<AppStateWithCounter>, _req: HttpRequest) {
    let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard
}

// time = 1596040068821
// header map = HeaderMap { inner: {"accept": One("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"), "sec-fetch-user": One("?1"), "accept-language": One("en-GB,en;q=0.9,hr-HR;q=0.8,hr;q=0.7,en-US;q=0.6,bs;q=0.5"), "sec-fetch-mode": One("navigate"), "sec-fetch-site": One("none"), "accept-encoding": One("gzip, deflate, br"), "cookie": One("_ga=GA1.1.833966493.1591460296"), "host": One("localhost:8088"), "user-agent": One("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/83.0.4103.116 Safari/537.36"), "connection": One("keep-alive"), "upgrade-insecure-requests": One("1"), "sec-fetch-dest": One("document")} }
// version = HTTP/1.1
// method = GET
// query_string = a=x&b=3
// ip = 127.0.0.1:52908
// scheme = http
// path = /push/1/1

fn to_cached_request(req: HttpRequest) -> Result<CachedRequest, ToStrError> {
    let time = current_time_ms();
    let header_map = req.headers(); // convert to h_map
    let version = req.version(); // version to string
    let method = req.method();
    let query_string = String::from(req.query_string());

    let conn_info = req.connection_info();

    let ip = String::from(conn_info.remote().unwrap_or("localhost"));
    let scheme = String::from(conn_info.scheme());
    let path = String::from(req.path());
    let body = String::from("");

    println!("time = {}", time);
    println!("header map = {:?}", header_map);
    println!("version = {:?}", version);
    println!("method = {:?}", method);
    println!("query_string = {}", query_string);
    println!("ip = {}", ip);
    println!("scheme = {}", scheme);
    println!("path = {}", path);
    println!("header map new  = {:?}", to_hash_map(header_map).unwrap());

    Ok(CachedRequest {
        http_version: format!("{:?}", version),
        method: format!("{:?}", method),
        headers: to_hash_map(header_map)?,
        query_string,
        path,
        body,
        time,
        ip,
    })
}

fn current_time_ms() -> u128 {
    if let Ok(dur) = SystemTime::now().duration_since(UNIX_EPOCH) {
        dur.as_millis()
    } else {
        0
    }
}

fn to_hash_map(headers: &HeaderMap) -> Result<HashMap<String, String>, ToStrError> {
    headers
        .iter()
        .map(|(name, value)| header_as_str_tuple(name, value))
        .collect()
}

fn header_as_str_tuple(
    name: &HeaderName,
    value: &HeaderValue,
) -> Result<(String, String), ToStrError> {
    Ok((name.to_string(), value.to_str()?.to_string()))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppStateWithCounter {
                counter: Mutex::new(0),
            }))
            .service(push_get)
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
