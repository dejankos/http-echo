use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CachedRequest {
    pub http_version: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub query_string: String,
    pub path: String,
    pub body: String,
    pub time: u128,
    pub ip: String,
}
