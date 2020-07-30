use std::collections::HashMap;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use actix_web::http::{HeaderMap, HeaderName, HeaderValue};
use actix_web::HttpRequest;
use lru_time_cache::LruCache;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
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

pub struct CacheManager {
    cache: LruCache<String, Vec<CachedRequest>>,
}

impl CacheManager {
    pub(crate) fn new() -> Self {
        CacheManager {
            cache: LruCache::<String, Vec<CachedRequest>>::with_expiry_duration(
                Duration::from_secs(15 * 60),
            ),
        }
    }

    pub(crate) fn store(&mut self, req: HttpRequest) -> CachedRequest {
        let cached_req = to_cached_request(req);
        if let Some(r) = self.cache.get_mut(&cached_req.path) {
            r.push(cached_req.clone())
        } else {
            self.cache
                .insert(cached_req.path.clone(), vec![cached_req.clone()]);
        }

        cached_req
    }

    pub(crate) fn retrieve(&mut self, path: &String) -> Option<Vec<CachedRequest>> {
        let key = &remove_base_path(path.as_str());
        self.cache.remove(key)
    }
}

fn to_cached_request(req: HttpRequest) -> CachedRequest {
    let time = current_time_ms();
    let headers = headers_to_map(req.headers());
    let http_version = format!("{:?}", req.version());
    let method = format!("{:?}", req.method());
    let query_string = String::from(req.query_string());
    let conn_info = req.connection_info();
    let ip = String::from(conn_info.remote().unwrap_or("localhost"));
    let path = remove_base_path(req.path());
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

fn remove_base_path(path: &str) -> String {
    path[5..path.len()].to_string()
}

fn current_time_ms() -> u128 {
    if let Ok(dur) = SystemTime::now().duration_since(UNIX_EPOCH) {
        dur.as_millis()
    } else {
        0
    }
}

fn headers_to_map(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|(name, value)| header_as_str_tuple(name, value))
        .collect()
}

fn header_as_str_tuple(name: &HeaderName, value: &HeaderValue) -> Option<(String, String)> {
    match value.to_str() {
        Ok(s) => Some((name.to_string(), s.to_string())),
        Err(e) => {
            error!("Error converting header = {} to string, e = {} ", name, e);
            None
        }
    }
}
