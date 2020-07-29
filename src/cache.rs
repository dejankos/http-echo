use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use actix_web::http::{HeaderMap, HeaderName, HeaderValue};
use actix_web::HttpRequest;
use serde::{Deserialize, Serialize};
use lru_time_cache::LruCache;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct CachedRequest {
    http_version: String,
    method: String,
    headers: HashMap<String, String>,
    query_string: String,
    path: String,
    body: String,
    time: u128,
    ip: String,
}

pub struct Cache {
    pub cache: Mutex<LruCache<String, Vec<CachedRequest>>>
}



pub fn to_cached_request(req: HttpRequest) -> CachedRequest {
    let time = current_time_ms();
    let headers = to_hash_map(req.headers());
    let http_version = format!("{:?}", req.version());
    let method = format!("{:?}", req.method());
    let query_string = String::from(req.query_string());
    let conn_info = req.connection_info();
    let ip = String::from(conn_info.remote().unwrap_or("localhost"));
    let path = String::from(req.path());
    let body = String::from("");

    CachedRequest {
        http_version,
        method,
        headers,
        query_string,
        path,
        body,
        time,
        ip,
    }
}

fn current_time_ms() -> u128 {
    if let Ok(dur) = SystemTime::now().duration_since(UNIX_EPOCH) {
        dur.as_millis()
    } else {
        0
    }
}

fn to_hash_map(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(name, value)| header_as_str_tuple(name, value))
        .collect()
}

fn header_as_str_tuple(
    name: &HeaderName,
    value: &HeaderValue,
) -> (String, String) {
    (name.to_string(),
     match value.to_str() {
         Ok(s) => s.to_string(),
         Err(e) => {
             error!("Error converting header = {} to string, e = {} ", name, e);
             format!("{}", e)
         }
     }
    )
}

