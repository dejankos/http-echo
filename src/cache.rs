use std::collections::HashMap;
use std::time::Duration;

use actix_web::HttpRequest;
use lru_time_cache::LruCache;
use serde::{Deserialize, Serialize};

use crate::util::{bytes_to_str, current_time_ms, headers_to_map, remove_base_path};
use actix_web::web::Bytes;

#[derive(Serialize, Deserialize, Clone)]
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

    pub(crate) fn store(&mut self, req: HttpRequest, body: Bytes) -> CachedRequest {
        let cached_req = to_cached_request(req, body);
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

fn to_cached_request(req: HttpRequest, body: Bytes) -> CachedRequest {
    CachedRequest {
        http_version: format!("{:?}", req.version()),
        method: format!("{:?}", req.method()),
        headers: headers_to_map(req.headers()),
        query_string: req.query_string().to_string(),
        path: remove_base_path(req.path()),
        body: bytes_to_str(body),
        time: current_time_ms(),
        ip: String::from(req.connection_info().remote().unwrap_or("")),
    }
}
