use axum::{
    Router,
    response::Html,
    routing::{delete, get, post},
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use tracing_subscriber::fmt;

use snip::config;

mod background;
mod db;
mod handlers;
mod middleware;
mod models;
mod redis_cache;
mod web;

use db::Database;
use handlers::*;
use redis_cache::RedisCache;

#[derive(Clone)]
pub struct AppState {
    db: Database,
    redis: RedisCache,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Handle version flag
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && (args[1] == "-V" || args[1] == "--version") {
        println!("snipped {}", VERSION);
        return Ok(());
    }

    fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| config::server::DEFAULT_DATABASE_URL.to_string());

    // Parse connection options
    let opts = SqliteConnectOptions::from_str(&database_url)?;

    // Ensure database directory and file exist
    let filename = opts.get_filename();
    if let Some(parent) = filename.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    if !filename.exists() {
        std::fs::File::create(filename)?;
    }

    let host =
        std::env::var("SNIP_HOST").unwrap_or_else(|_| config::server::DEFAULT_HOST.to_string());
    let port =
        std::env::var("SNIP_PORT").unwrap_or_else(|_| config::server::DEFAULT_PORT.to_string());
    let bind_addr = format!("{}:{}", host, port);

    let pool = SqlitePoolOptions::new()
        .max_connections(config::database::MAX_CONNECTIONS)
        .connect_with(opts)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    // Initialize Redis
    let redis_url = std::env::var("REDIS_URL")
        .ok()
        .or_else(|| config::server::DEFAULT_REDIS_URL.map(|s| s.to_string()));
    let redis = RedisCache::new(redis_url).await;

    let state = AppState {
        db: Database::new(pool),
        redis,
    };

    // Start background task to periodically flush view counters
    background::spawn_view_counter_flush(state.db.clone(), state.redis.clone());

    // Configure global rate limiting: 10 requests per second per IP with burst of 60
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(60)
        .finish()
        .unwrap();

    // Create separate routers for view and API routes
    let app_view = Router::new()
        .route("/", get(serve_index))
        .route("/u/{username}", get(serve_index))
        .route("/s/{id}", get(serve_index));

    let app_api = Router::new()
        .route("/api/register", post(register_user))
        .route("/api/login", post(login))
        .route("/api/revoke-key", post(revoke_api_key))
        .route("/api/change-password", post(change_password))
        .route("/api/snippets", post(create_snippet))
        .route("/api/snippets/{id}", delete(delete_snippet))
        .route("/api/snippets/{id}/star", post(star_snippet))
        .route("/api/snippets/{id}/fork", post(fork_snippet))
        .route("/api/snippets/{id}/star", delete(unstar_snippet))
        .route("/api/snippets/{id}/comments", post(create_comment))
        .route("/api/comments/{id}", delete(delete_comment))
        .route(
            "/api/comments/{id}/like",
            post(like_comment).delete(unlike_comment),
        )
        .route("/api/snippets/{id}/comments", get(list_comments))
        .route("/api/snippets/{id}", get(get_snippet))
        .route("/api/snippets/{id}/star", get(get_star_status));

    let app_api_cache = Router::new()
        .route("/raw/{id}", get(get_raw_snippet))
        .route("/api/snippets", get(list_snippets))
        .route("/api/search", get(search_snippets))
        .route("/api/users/{username}/snippets", get(list_user_snippets))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::cache_middleware,
        ));

    // Merge routers and add shared middleware
    let app = app_view
        .merge(app_api)
        .merge(app_api_cache)
        .layer(CompressionLayer::new())
        .layer(GovernorLayer::new(governor_conf))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Server running on http://{}:{}", host, port);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn serve_index() -> Html<String> {
    Html(web::INDEX_HTML.to_string())
}
