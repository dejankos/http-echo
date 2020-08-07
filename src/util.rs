use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use actix_web::http::{HeaderMap, HeaderName, HeaderValue};
use actix_web::web::Bytes;

pub fn current_time_ms() -> u128 {
    if let Ok(dur) = SystemTime::now().duration_since(UNIX_EPOCH) {
        dur.as_millis()
    } else {
        0
    }
}

pub fn headers_to_map(headers: &HeaderMap) -> HashMap<String, String> {
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

pub fn bytes_to_str(body: Bytes) -> String {
    if body.is_empty() {
        String::new()
    } else {
        match String::from_utf8(body.to_vec()) {
            Ok(s) => s,
            Err(e) => {
                error!("Error converting payload to str, e = {} ", e);
                String::new()
            }
        }
    }
}

pub fn remove_base_path(path: &str) -> String {
    path[5..path.len()].to_string()
}
