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
    <title>Snip - Code Snippets</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 2rem;
        }
        .container {
            max-width: 900px;
            margin: 0 auto;
        }
        h1 {
            color: white;
            text-align: center;
            margin-bottom: 2rem;
            font-size: 2.5rem;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.2);
        }
        .snippet {
            background: white;
            border-radius: 12px;
            padding: 1.5rem;
            margin-bottom: 1rem;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
            transition: transform 0.2s;
        }
        .snippet:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 12px rgba(0,0,0,0.15);
        }
        .snippet-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1rem;
            padding-bottom: 0.5rem;
            border-bottom: 2px solid #e0e0e0;
        }
        .snippet-desc {
            font-weight: 600;
            color: #333;
            font-size: 1.1rem;
        }
        .snippet-meta {
            color: #666;
            font-size: 0.85rem;
        }
        .snippet-content {
            background: #f8f9fa;
            border-radius: 8px;
            padding: 1rem;
            overflow-x: auto;
        }
        .snippet-content pre {
            margin: 0;
            font-family: 'Fira Code', 'Monaco', 'Menlo', monospace;
            font-size: 0.9rem;
            line-height: 1.5;
            color: #333;
            white-space: pre-wrap;
            word-break: break-word;
        }
        .pagination {
            display: flex;
            justify-content: center;
            gap: 0.5rem;
            margin-top: 2rem;
        }
        .pagination button {
            background: white;
            border: none;
            padding: 0.75rem 1.5rem;
            border-radius: 8px;
            cursor: pointer;
            font-size: 1rem;
            transition: all 0.2s;
        }
        .pagination button:hover:not(:disabled) {
            background: #667eea;
            color: white;
        }
        .pagination button:disabled {
            opacity: 0.5;
            cursor: not-allowed;
        }
        .pagination span {
            color: white;
            align-self: center;
            font-weight: 500;
        }
        .loading {
            text-align: center;
            color: white;
            font-size: 1.2rem;
            padding: 2rem;
        }
        .empty {
            text-align: center;
            color: white;
            font-size: 1.2rem;
            padding: 3rem;
            background: rgba(255,255,255,0.1);
            border-radius: 12px;
        }
        .api-info {
            background: rgba(255,255,255,0.95);
            border-radius: 12px;
            padding: 1.5rem;
            margin-bottom: 2rem;
            text-align: center;
        }
        .api-info code {
            background: #e9ecef;
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
            font-family: monospace;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>📋 Snip</h1>
        
        <div class="api-info">
            <p><strong>CLI Usage:</strong> <code>cat file.txt | snipped --desc "my snippet"</code></p>
            <p style="margin-top: 0.5rem; color: #666;">Register at <code>POST /api/register</code> with <code>{"username": "...", "password": "..."}</code></p>
        </div>
        
        <div id="snippets">
            <div class="loading">Loading snippets...</div>
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
                <button ${currentPage === 1 ? 'disabled' : ''} onclick="loadSnippets(${currentPage - 1})">← Prev</button>
                <span>Page ${currentPage} of ${totalPages}</span>
                <button ${currentPage === totalPages ? 'disabled' : ''} onclick="loadSnippets(${currentPage + 1})">Next →</button>
            `;
        }

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }

        function formatDate(dateStr) {
            const date = new Date(dateStr);
            return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'});
        }

        loadSnippets();
    </script>
</body>
</html>
"#;
