use axum::{
    Router,
    response::Html,
    routing::{delete, get, post},
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tracing_subscriber::fmt;

use snip::config;

mod db;
mod handlers;
mod models;
mod redis_cache;
mod web;

use db::Database;
use handlers::*;
use redis_cache::RedisCache;

#[derive(Clone)]
struct AppState {
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
    // Create the file if it doesn't exist (SQLite needs this in some environments)
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
        redis: redis.clone(),
    };

    // Start background task to periodically flush view counters
    if redis.is_enabled() {
        let db_for_flush = state.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(
                config::rate_limit::VIEW_COUNTER_FLUSH_INTERVAL_SECS,
            ));
            loop {
                interval.tick().await;
                let counters = redis.flush_view_counters("views").await;
                if !counters.is_empty() {
                    tracing::info!("Flushing {} view counters to database", counters.len());
                    for (snippet_id, count) in counters {
                        if let Err(e) =
                            sqlx::query("UPDATE snippets SET views = views + ?1 WHERE id = ?2")
                                .bind(count)
                                .bind(snippet_id)
                                .execute(db_for_flush.pool())
                                .await
                        {
                            tracing::error!(
                                "Failed to update views for snippet {}: {}",
                                snippet_id,
                                e
                            );
                        }
                    }
                }
            }
        });
    }

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/u/{username}", get(serve_index))
        .route("/s/{id}", get(serve_index))
        .route("/raw/{id}", get(get_raw_snippet))
        .route("/api/register", post(register_user))
        .route("/api/login", post(login))
        .route("/api/revoke-key", post(revoke_api_key))
        .route("/api/change-password", post(change_password))
        .route("/api/snippets", post(create_snippet))
        .route("/api/snippets", get(list_snippets))
        .route("/api/snippets/{id}", get(get_snippet))
        .route("/api/snippets/{id}", delete(delete_snippet))
        .route("/api/snippets/{id}/star", post(star_snippet))
        .route("/api/snippets/{id}/star", delete(unstar_snippet))
        .route("/api/snippets/{id}/star", get(get_star_status))
        .route("/api/snippets/{id}/fork", post(fork_snippet))
        .route("/api/search", get(search_snippets))
        .route("/api/users/{username}/snippets", get(list_user_snippets))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Server running on http://{}:{}", host, port);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_index() -> Html<String> {
    Html(web::INDEX_HTML.to_string())
}

// async fn serve_index() -> Html<&'static str> {
//     Html(include_str!("index.html"))
// }
