use axum::{
    Router,
    response::Html,
    routing::{get, post},
};
use snip::config;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::{net::SocketAddr, str::FromStr};
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{compression::CompressionLayer, cors::CorsLayer};

use crate::{AppState, database::Database, handlers, middleware, web};

/// Run the snippet server until an interrupt or termination signal arrives.
#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && matches!(args[1].as_str(), "-V" | "--version") {
        println!("snipd {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    tracing_subscriber::fmt::init();
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| config::DEFAULT_DATABASE_URL.to_owned());
    let options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);
    let filename = options.get_filename();
    if let Some(parent) = filename.parent()
        && !parent.as_os_str().is_empty()
    {
        tokio::fs::create_dir_all(parent).await?;
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(config::MAX_CONNECTIONS)
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let state = AppState {
        db: Database::new(pool),
    };
    let host = std::env::var("SNIP_HOST").unwrap_or_else(|_| config::DEFAULT_HOST.to_owned());
    let port: u16 = std::env::var("SNIP_PORT")
        .unwrap_or_else(|_| config::DEFAULT_PORT.to_owned())
        .parse()?;
    let listener = tokio::net::TcpListener::bind((host.as_str(), port)).await?;
    tracing::info!(address = %listener.local_addr()?, "Server listening");
    axum::serve(
        listener,
        router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    Ok(())
}

pub(crate) fn router(state: AppState) -> Router {
    // Replenish one request every 100 ms: ten requests per second per peer IP.
    let governor = GovernorConfigBuilder::default()
        .per_millisecond(100)
        .burst_size(60)
        .finish()
        .expect("the rate interval and burst are both nonzero");
    Router::new()
        .route("/", get(serve_index))
        .route("/u/{username}", get(serve_index))
        .route("/s/{id}", get(serve_index))
        .route("/raw/{id}", get(handlers::get_raw_snippet))
        .route("/api/register", post(handlers::register_user))
        .route("/api/login", post(handlers::login))
        .route("/api/revoke-key", post(handlers::revoke_api_key))
        .route("/api/change-password", post(handlers::change_password))
        .route(
            "/api/snippets",
            get(handlers::list_snippets).post(handlers::create_snippet),
        )
        .route("/api/search", get(handlers::search_snippets))
        .route(
            "/api/users/{username}/snippets",
            get(handlers::list_user_snippets),
        )
        .route(
            "/api/snippets/{id}",
            get(handlers::get_snippet).delete(handlers::delete_snippet),
        )
        .route(
            "/api/snippets/{id}/star",
            get(handlers::get_star_status)
                .post(handlers::star_snippet)
                .delete(handlers::unstar_snippet),
        )
        .route("/api/snippets/{id}/fork", post(handlers::fork_snippet))
        .route(
            "/api/snippets/{id}/comments",
            get(handlers::list_comments).post(handlers::create_comment),
        )
        .route(
            "/api/comments/{id}",
            axum::routing::delete(handlers::delete_comment),
        )
        .route(
            "/api/comments/{id}/like",
            post(handlers::like_comment).delete(handlers::unlike_comment),
        )
        .layer(CompressionLayer::new())
        .layer(GovernorLayer::new(governor))
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn(middleware::response_headers))
        .with_state(state)
}

async fn serve_index() -> Html<&'static str> {
    Html(web::INDEX_HTML)
}

async fn shutdown_signal() {
    let interrupt = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "Failed to install interrupt handler");
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "Failed to install termination handler");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = interrupt => {}, _ = terminate => {} }
}
