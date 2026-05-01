use axum::{
    Router,
    response::Html,
    routing::{get, post},
};
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::cors::CorsLayer;
use tracing_subscriber::fmt;

mod db;
mod handlers;
mod models;

use db::Database;
use handlers::*;

#[derive(Clone)]
struct AppState {
    db: Database,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:snip.db".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState {
        db: Database::new(pool),
    };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/register", post(register_user))
        .route("/api/snippets", post(create_snippet))
        .route("/api/snippets", get(list_snippets))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server running on http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_index() -> Html<String> {
    Html(INDEX_HTML.to_string())
}

const INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>snip</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: "Courier New", "Liberation Mono", monospace;
            background: #f5f5f5;
            color: #333;
            line-height: 1.6;
            padding: 2rem 1rem;
        }
        .container {
            max-width: 80ch;
            margin: 0 auto;
        }
        h1 {
            font-size: 1.5rem;
            margin-bottom: 1rem;
            padding-bottom: 0.5rem;
            border-bottom: 2px solid #333;
        }
        .help {
            background: #fff;
            border: 1px solid #ccc;
            padding: 1rem;
            margin-bottom: 1.5rem;
        }
        .help code {
            background: #eee;
            padding: 0.1rem 0.3rem;
        }
        .snippet {
            background: #fff;
            border: 1px solid #ccc;
            margin-bottom: 1rem;
        }
        .snippet-header {
            border-bottom: 1px solid #ccc;
            padding: 0.5rem 1rem;
            background: #fafafa;
            display: flex;
            justify-content: space-between;
            font-size: 0.875rem;
        }
        .snippet-desc {
            font-weight: bold;
        }
        .snippet-meta {
            color: #666;
        }
        .snippet-content {
            padding: 1rem;
        }
        .snippet-content pre {
            margin: 0;
            font-family: inherit;
            white-space: pre-wrap;
            word-break: break-word;
            overflow-x: auto;
        }
        .pagination {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 1rem;
            margin-top: 1.5rem;
            padding-top: 1rem;
            border-top: 1px solid #ccc;
        }
        .pagination button {
            font-family: inherit;
            padding: 0.25rem 0.75rem;
            background: #fff;
            border: 1px solid #333;
            cursor: pointer;
        }
        .pagination button:hover:not(:disabled) {
            background: #333;
            color: #fff;
        }
        .pagination button:disabled {
            opacity: 0.3;
            cursor: default;
        }
        .loading, .empty {
            text-align: center;
            padding: 2rem;
            color: #666;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>snip ~ code snippets</h1>
        
        <div class="help">
            <p>$ echo "text" | snipped --desc "note"</p>
            <p style="margin-top: 0.5rem; color: #666;"># POST /api/register {username, password} to get API key</p>
        </div>
        
        <div id="snippets">
            <div class="loading">loading...</div>
        </div>
        
        <div class="pagination" id="pagination"></div>
    </div>

    <script>
        const ITEMS_PER_PAGE = 10;
        let currentPage = 1;
        let totalPages = 1;

        async function loadSnippets(page = 1) {
            try {
                const response = await fetch(`/api/snippets?page=${page}&limit=${ITEMS_PER_PAGE}`);
                const data = await response.json();
                
                totalPages = Math.ceil(data.total / ITEMS_PER_PAGE) || 1;
                currentPage = page;
                
                renderSnippets(data.snippets);
                renderPagination();
            } catch (error) {
                document.getElementById('snippets').innerHTML = 
                    '<div class="empty">Error loading snippets</div>';
            }
        }

        function renderSnippets(snippets) {
            const container = document.getElementById('snippets');
            
            if (snippets.length === 0) {
                container.innerHTML = '<div class="empty">No snippets yet. Be the first to share!</div>';
                return;
            }
            
            container.innerHTML = snippets.map(s => `
                <div class="snippet">
                    <div class="snippet-header">
                        <span class="snippet-desc">${escapeHtml(s.description || 'Untitled')}</span>
                        <span class="snippet-meta">${escapeHtml(s.author)} · ${formatDate(s.created_at)}</span>
                    </div>
                    <div class="snippet-content">
                        <pre>${escapeHtml(s.content)}</pre>
                    </div>
                </div>
            `).join('');
        }

        function renderPagination() {
            const container = document.getElementById('pagination');
            
            if (totalPages <= 1) {
                container.innerHTML = '';
                return;
            }
            
            container.innerHTML = `
                <button ${currentPage === 1 ? 'disabled' : ''} onclick="loadSnippets(${currentPage - 1})">&lt; prev</button>
                <span>${currentPage}/${totalPages}</span>
                <button ${currentPage === totalPages ? 'disabled' : ''} onclick="loadSnippets(${currentPage + 1})">next &gt;</button>
            `;
        }

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }

        function formatDate(dateStr) {
            const d = new Date(dateStr);
            return d.toISOString().slice(0,16).replace('T',' ');
        }

        loadSnippets();
    </script>
</body>
</html>
"#;
