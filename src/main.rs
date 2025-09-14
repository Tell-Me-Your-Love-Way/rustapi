use std::{env, fs, sync::Arc};

use axum::{routing::post, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{handlers::{signin_handler, signup_handler}, repository::user_repository::SqlxUserRepository, services::user_service::UserService};

pub mod handlers;
pub mod services;
pub mod repository;
pub mod models;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info,rustapi=debug") // Set log levels: info for all, debug for my_api
        .init();
    let db_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => "postgres://user:password@localhost:5432/mydb".to_string(),
    };
    let addr = match env::var("APP_IP_PORT_LISTENER") {
        Ok(uri) => uri,
        Err(_) => "0.0.0.0:3000".to_string(),
    };
    
    let cache_url = match env::var("CACHE_URL") {
        Ok(uri) => uri,
        Err(_) => "redis://127.0.0.1:6379/".to_string(),
    };
    let privkey = fs::read("./critical_word").unwrap();
    let pubkey = fs::read("./low_word").unwrap();
    let pg_pool: PgPool = PgPoolOptions::new()
        .max_connections(300)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(60))
        .test_before_acquire(false)
        .connect(&db_url)
        .await.map_err(|e| format!("Database Error {}", e)).unwrap();
    let redis_client = redis::Client::open(cache_url).unwrap();
    let cache_con = redis_client.get_connection_manager().await.unwrap();
    let user_repo = SqlxUserRepository::new(pg_pool,cache_con);
    let user_service = UserService::new(user_repo, Arc::new(privkey), Arc::new(pubkey));
    let login_route: Router<UserService> = axum::Router::new()
        .route("/login", post(signin_handler::handler));
    let register_route: Router<UserService> = axum::Router::new()
        .route("/register", post(signup_handler::handler));
    let routes: Router<UserService> = Router::new().nest("/api", login_route).nest("/api", register_route);
    let app: Router = Router::new()
        .merge(routes)
        .with_state(user_service);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();
    axum::serve(listener, app)
    .await
    .unwrap()
}
