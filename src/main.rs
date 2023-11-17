use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router, Server,
};
use config::ConfigManager;
use once_cell::sync::Lazy;
use poise::serenity_prelude::CacheHttp;
use rand::distributions::{Alphanumeric, DistString};
use serde_json::Value;
use std::{net::SocketAddr, sync::Mutex};
use structs::{AppState, HtmlTemplate, JsonBody, VerifyPageTemplate, KeyManager, Key};

mod bot;
mod structs;
mod config;

pub static KEY_MANAGER: Lazy<Mutex<KeyManager>> = Lazy::new(|| Mutex::new(KeyManager::new()));
pub static CONFIG: Lazy<Mutex<ConfigManager>> = Lazy::new(|| Mutex::new(ConfigManager::new("config.json").unwrap()));

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").unwrap();
    let turnstile_secret = std::env::var("TURNSTILE_SECRET").unwrap();

    let bot = bot::create_bot(&token).await.unwrap();

    let cache_and_http = bot.client().cache_and_http.clone();

    let app = Router::new().route("/verify/:key", get(verify_page)).route(
        "/complete_verify",
        post(complete_verify_page).with_state(AppState { cache_and_http, turnstile_secret }),
    );

    let address = SocketAddr::from(([127, 0, 0, 1], 3000));

    tokio::task::spawn(async {
        bot.start().await.unwrap();
    });

    Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

impl KeyManager {
    pub fn new() -> Self {
        Self { keys: vec![] }
    }

    pub fn add_key(&mut self, user_id: u64, server_id:u64, key: String) {
        self.keys.push(Key {
            user_id,
            server_id,
            key,
        });
    }

    pub fn remove_key(&mut self, key: String) {        
        self.keys.retain(|k| k.key != key);
    }

    pub fn contains_key(&self, key: String) -> bool {
        self.keys.iter().any(|k| k.key == key)
    }

    pub fn create_key(&mut self, server_id: u64, user_id: u64) -> String {
        let mut rng = rand::thread_rng();
        let key = Alphanumeric.sample_string(&mut rng, 16);

        self.add_key(
            user_id,
            server_id,
            key.clone()
        );

        key
    }

    pub fn get_key(&self, key: String) -> Option<&Key> {
        self.keys.iter().find(|k| k.key == key)
    }
}

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        let body = self.0.render().unwrap();

        Html(body).into_response()
    }
}

async fn complete_verify_page(
    State(state): State<AppState>,
    Json(payload): Json<JsonBody>,
) -> impl IntoResponse {
    let client = reqwest::Client::new();
    let response = client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&[("secret", &state.turnstile_secret), ("response", &payload.token)])
        .send()
        .await
        .unwrap();

    let json = response.json::<Value>().await.unwrap();

    let is_success = json["success"].as_bool().unwrap();

    if !is_success {
        return "failure";
    }

    if !KEY_MANAGER.lock().unwrap().contains_key(payload.key.clone()) {
        return "failure";
    }

    let key = KEY_MANAGER.lock().unwrap().get_key(payload.key).unwrap().clone();
    let config = CONFIG.lock().unwrap().get_server_config(key.server_id).unwrap().clone();

    let result = state.cache_and_http.http().add_member_role(key.server_id, key.user_id, config.grant_role_id, None).await;

    if result.is_err() {
        return "failure";
    }

    KEY_MANAGER.lock().unwrap().remove_key(key.key);

    "success"

}

async fn verify_page(Path(key): Path<String>) -> impl IntoResponse {
    let template = VerifyPageTemplate { key };

    HtmlTemplate(template)
}
