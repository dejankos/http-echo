#[macro_use]
extern crate log;

use std::borrow::BorrowMut;

use std::sync::Mutex;

use actix_web::body::{Body, ResponseBody};
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::middleware::Logger;
use actix_web::web::Bytes;
use actix_web::{dev, get, http};
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

#[get("/poll/*")]
async fn poll_get(mut manager: web::Data<Mutex<CacheManager>>, req: HttpRequest) -> HttpResponse {
    let mut man = manager.borrow_mut().lock().unwrap();
    HttpResponse::Ok().json(man.retrieve(&req.path()))
}

async fn push(
    manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    HttpResponse::Ok().json(store_data(manager, req, body))
}

fn store_data(
    mut manager: web::Data<Mutex<CacheManager>>,
    req: HttpRequest,
    body: Bytes,
) -> CachedRequest {
    let mut man = manager.borrow_mut().lock().unwrap();
    man.store(req, body)
}

#[allow(clippy::unnecessary_wraps)]
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap();
    let args: ServerConfig = ServerConfig::from_args();

    let bind = format!("{}:{}", args.ip, args.port);
    let workers = args.workers.unwrap_or_else(num_cpus::get);
    let global_state = web::Data::new(Mutex::new(CacheManager::new(args.ttl)));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(ErrorHandlers::new().handler(http::StatusCode::NOT_FOUND, not_found))
            .app_data(global_state.clone())
            .service(web::resource("/push/*").to(push))
            .service(poll_get)
    })
    .bind(bind)?
    .workers(workers)
    .shutdown_timeout(10)
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::dev::ServiceResponse;
    use actix_web::http::StatusCode;
    use actix_web::rt as actix_rt;
    use actix_web::{test, web, App, Error};
    use serde::de::DeserializeOwned;

    use super::*;

    #[actix_rt::test]
    async fn should_cache_request_for_push_get() -> Result<(), Error> {
        std::env::set_var("RUST_BACKTRACE", "full");
        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(Mutex::new(CacheManager::new(1000 * 15))))
                .service(web::resource("/push/*").to(push))
                .service(poll_get),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/push/push_get?a=b&c=d")
            .to_request();
        let res = test::call_service(&mut app, req).await;
        let status = &res.status();
        let cr: CachedRequest = read_response_payload(res);

        assert_response(&status, "GET", "/push_get", "a=b&c=d", None, &cr);

        let req = test::TestRequest::get().uri("/poll/push_get").to_request();
        let res = test::call_service(&mut app, req).await;
        let status = &res.status();
        let cr: Vec<CachedRequest> = read_response_payload(res);

        assert_eq!(cr.len(), 1);
        assert_response(&status, "GET", "/push_get", "a=b&c=d", None, &cr[0]);

        Ok(())
    }

    #[actix_rt::test]
    async fn should_cache_multiple_request_for_push_post() -> Result<(), Error> {
        std::env::set_var("RUST_BACKTRACE", "full");
        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(Mutex::new(CacheManager::new(1000 * 15))))
                .service(web::resource("/push/*").to(push))
                .service(poll_get),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/push/push_post?a=b&c=d")
            .set_payload("test payload")
            .to_request();

        let res = test::call_service(&mut app, req).await;
        let status = &res.status();
        let cr: CachedRequest = read_response_payload(res);

        assert_response(
            &status,
            "POST",
            "/push_post",
            "a=b&c=d",
            Some("test payload"),
            &cr,
        );

        let req = test::TestRequest::post()
            .uri("/push/push_post?e=f&g=h")
            .set_payload("test payload 1")
            .to_request();

        let res = test::call_service(&mut app, req).await;
        let status = &res.status();
        let cr: CachedRequest = read_response_payload(res);

        assert_response(
            &status,
            "POST",
            "/push_post",
            "e=f&g=h",
            Some("test payload 1"),
            &cr,
        );

        let req = test::TestRequest::get().uri("/poll/push_post").to_request();
        let res = test::call_service(&mut app, req).await;
        let status = &res.status();
        let cr: Vec<CachedRequest> = read_response_payload(res);
        assert_eq!(cr.len(), 2);
        assert_response(
            &status,
            "POST",
            "/push_post",
            "a=b&c=d",
            Some("test payload"),
            &cr[0],
        );
        assert_response(
            &status,
            "POST",
            "/push_post",
            "e=f&g=h",
            Some("test payload 1"),
            &cr[1],
        );

        Ok(())
    }

    fn assert_response(
        status_code: &StatusCode,
        method: &str,
        path: &str,
        query_string: &str,
        body: Option<&str>,
        cr: &CachedRequest,
    ) {
        assert!(status_code.is_success());
        assert_eq!(method, cr.method);
        assert_eq!(path, cr.path);
        assert_eq!(query_string, cr.query_string);
        if let Some(b) = body {
            assert_eq!(b, cr.body);
        }
    }

    fn read_response_payload<T>(res: ServiceResponse<Body>) -> T
    where
        T: DeserializeOwned,
    {
        match res.response().body().as_ref() {
            Some(Body::Bytes(bytes)) => deserialize(bytes),
            _ => panic!("Can't read response body"),
        }
    }

    fn deserialize<T>(bytes: &Bytes) -> T
    where
        T: DeserializeOwned,
    {
        match serde_json::from_slice(bytes) {
            Ok(r) => r,
            _ => panic!("Can't deserialize response body"),
        }
    }
}
