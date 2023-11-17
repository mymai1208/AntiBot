use std::sync::Arc;

use askama::Template;
use poise::serenity_prelude::CacheAndHttp;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub cache_and_http: Arc<CacheAndHttp>,
    pub turnstile_secret: String,
}

#[derive(Template)]
#[template(path = "verify.html")]
pub struct VerifyPageTemplate {
    pub key: String,
}

#[derive(Deserialize)]
pub struct JsonBody {
    pub key: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub servers: Vec<ServerConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConfig {
    pub id: u64,
    pub grant_role_id: u64,
}

#[derive(Clone)]
pub struct Key {
    pub user_id: u64,
    pub server_id: u64,
    pub key: String
}

#[derive(Clone)]
pub struct KeyManager {
    pub keys: Vec<Key>,
}

pub struct HtmlTemplate<T>(pub T);
