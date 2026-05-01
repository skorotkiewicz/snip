use axum::{
    Router,
    response::Html,
    routing::{delete, get, post},
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
        .route("/u/{username}", get(serve_index))
        .route("/api/register", post(register_user))
        .route("/api/login", post(login))
        .route("/api/revoke-key", post(revoke_api_key))
        .route("/api/snippets", post(create_snippet))
        .route("/api/snippets", get(list_snippets))
        .route("/api/snippets/{id}", delete(delete_snippet))
        .route("/api/users/{username}/snippets", get(list_user_snippets))
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
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/bash.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/c.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/cpp.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/csharp.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/css.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/go.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/java.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/javascript.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/json.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/kotlin.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/lua.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/php.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/python.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/ruby.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/rust.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/scala.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/sql.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/swift.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/typescript.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/yaml.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/zig.min.js"></script>
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
        h1 a {
            color: inherit;
            text-decoration: none;
        }
        h1 a:hover {
            text-decoration: underline;
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
        .auth-box {
            background: #fff;
            border: 1px solid #ccc;
            padding: 1rem;
            margin-bottom: 1.5rem;
        }
        .auth-box h3 {
            font-size: 1rem;
            margin-bottom: 0.75rem;
            padding-bottom: 0.25rem;
            border-bottom: 1px solid #ccc;
        }
        .auth-form {
            display: flex;
            gap: 0.5rem;
            flex-wrap: wrap;
            margin-bottom: 0.5rem;
        }
        .auth-form input {
            font-family: inherit;
            padding: 0.25rem 0.5rem;
            border: 1px solid #ccc;
            flex: 1;
            min-width: 120px;
        }
        .auth-form button {
            font-family: inherit;
            padding: 0.25rem 0.75rem;
            background: #333;
            color: #fff;
            border: none;
            cursor: pointer;
        }
        .auth-form button:hover {
            background: #555;
        }
        .api-key-display {
            background: #f5f5f5;
            border: 1px solid #ccc;
            padding: 0.75rem;
            margin-top: 0.5rem;
        }
        .api-key-display pre {
            margin: 0.5rem 0;
            font-family: inherit;
            background: #fff;
            padding: 0.5rem;
            border: 1px solid #ccc;
            word-break: break-all;
        }
        .api-key-display button {
            font-family: inherit;
            padding: 0.25rem 0.5rem;
            background: #fff;
            border: 1px solid #333;
            cursor: pointer;
            margin-top: 0.5rem;
        }
        .api-key-display button:hover {
            background: #333;
            color: #fff;
        }
        .error-msg {
            color: #c00;
            margin-top: 0.5rem;
        }
        .success-msg {
            color: #0a0;
            margin-top: 0.5rem;
        }
        .snippet {
            margin-bottom: 2rem;
        }
        .snippet-desc {
            font-size: 1rem;
            font-weight: bold;
            color: #333;
            margin-bottom: 0.5rem;
            padding: 0.25rem 0;
        }
        .snippet-lang {
            font-size: 0.75rem;
            color: #666;
            background: #eee;
            padding: 0.1rem 0.4rem;
            margin-left: 0.5rem;
            text-transform: lowercase;
        }
        .snippet-content {
            background: #fff;
            border: 1px solid #333;
            padding: 1rem;
            margin-bottom: 0.5rem;
        }
        .snippet-content pre {
            margin: 0;
            font-family: inherit;
            white-space: pre-wrap;
            word-break: break-word;
            overflow-x: auto;
            color: #222;
            line-height: 1.5;
        }
        .snippet-meta {
            font-size: 0.875rem;
            color: #666;
            text-align: right;
        }
        .snippet-meta a {
            color: #333;
            text-decoration: none;
        }
        .snippet-meta a:hover {
            text-decoration: underline;
        }
        .delete-btn {
            font-family: inherit;
            font-size: 0.875rem;
            background: none;
            border: none;
            color: #c00;
            cursor: pointer;
            padding: 0;
            margin-left: 0.5rem;
        }
        .delete-btn:hover {
            color: #f00;
            text-decoration: underline;
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
        <h1><a href="/">snip</a><span id="header-suffix"> ~ code snippets</span></h1>
        
        <div class="help" id="help-box">
            <p>$ echo "text" | snip --desc "note" --lang rust</p>
            <p style="margin-top: 0.5rem; color: #666;"># POST /api/register {username, password} to get API key</p>
        </div>

        <div class="auth-box" id="auth-box">
            <h3>~ auth</h3>
            <div id="login-form">
                <div class="auth-form">
                    <input type="text" id="login-user" placeholder="username">
                    <input type="password" id="login-pass" placeholder="password">
                    <button onclick="doLogin()">login</button>
                </div>
                <div id="login-msg"></div>
            </div>
            <div id="api-key-box" style="display:none;">
                <div class="api-key-display">
                    <strong>API Key:</strong>
                    <pre id="api-key-value"></pre>
                    <button onclick="copyApiKey()">copy</button>
                    <button onclick="doRevoke()" style="margin-left: 0.5rem;">revoke & regenerate</button>
                    <button onclick="doLogout()" style="margin-left: 0.5rem;">logout</button>
                    <div id="revoke-msg"></div>
                </div>
            </div>
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
        
        // Parse URL to determine if we're viewing a user profile
        const pathParts = window.location.pathname.split('/');
        const profileUser = pathParts[1] === 'u' ? pathParts[2] : null;

        async function loadSnippets(page = 1) {
            currentPage = page;
            const url = profileUser 
                ? `/api/users/${profileUser}/snippets?page=${page}&limit=${ITEMS_PER_PAGE}`
                : `/api/snippets?page=${page}&limit=${ITEMS_PER_PAGE}`;
            
            try {
                const response = await fetch(url);
                const data = await response.json();
                
                totalPages = Math.ceil(data.total / ITEMS_PER_PAGE) || 1;
                
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
                container.innerHTML = '<div class="empty">No snippets yet.</div>';
                return;
            }
            
            const currentUser = localStorage.getItem('snip_username');
            const apiKey = localStorage.getItem('snip_api_key');
            
            container.innerHTML = snippets.map(s => {
                const authorLink = profileUser
                    ? escapeHtml(s.author)
                    : `<a href="/u/${escapeHtml(s.author)}">${escapeHtml(s.author)}</a>`;
                const langTag = s.language && s.language !== 'plaintext' ? `<span class="snippet-lang">${escapeHtml(s.language)}</span>` : '';
                const descHtml = s.description ? `<div class="snippet-desc">${escapeHtml(s.description)}${langTag}</div>` : langTag ? `<div class="snippet-desc">${langTag}</div>` : '';
                const isOwner = apiKey && s.author === currentUser;
                const deleteBtn = isOwner ? ` <button class="delete-btn" onclick="deleteSnippet(${s.id})">[x]</button>` : '';
                const langClass = s.language && s.language !== 'plaintext' ? ` class="language-${s.language}"` : '';
                return `
                <div class="snippet" id="snippet-${s.id}">
                    ${descHtml}
                    <div class="snippet-content">
                        <pre><code${langClass}>${escapeHtml(s.content)}</code></pre>
                    </div>
                    <div class="snippet-meta">${authorLink} · ${formatDate(s.created_at)}${deleteBtn}</div>
                </div>
            `}).join('');
            
            // Apply syntax highlighting
            container.querySelectorAll('pre code').forEach((block) => {
                hljs.highlightElement(block);
            });
        }
        
        async function deleteSnippet(id) {
            if (!confirm('Delete this snippet?')) return;
            
            const apiKey = localStorage.getItem('snip_api_key');
            if (!apiKey) {
                alert('Not authenticated');
                return;
            }
            
            try {
                const response = await fetch(`/api/snippets/${id}`, {
                    method: 'DELETE',
                    headers: { 'X-API-Key': apiKey }
                });
                
                if (response.ok) {
                    document.getElementById(`snippet-${id}`).remove();
                } else if (response.status === 403) {
                    alert('Can only delete your own snippets');
                } else {
                    alert('Failed to delete');
                }
            } catch (e) {
                alert('Error deleting snippet');
            }
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
        
        // Update UI for profile view
        if (profileUser) {
            document.getElementById('header-suffix').textContent = ` ~ ${profileUser}`;
            document.getElementById('help-box').innerHTML = 
                `<p><a href="/">&lt; back to all snippets</a></p>`;
        }

        // Auth functions
        async function doLogin() {
            const username = document.getElementById('login-user').value;
            const password = document.getElementById('login-pass').value;
            const msgDiv = document.getElementById('login-msg');
            
            if (!username || !password) {
                msgDiv.innerHTML = '<div class="error-msg">enter username and password</div>';
                return;
            }
            
            try {
                const response = await fetch('/api/login', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ username, password })
                });
                
                if (response.ok) {
                    const data = await response.json();
                    localStorage.setItem('snip_api_key', data.api_key);
                    localStorage.setItem('snip_username', data.username);
                    showApiKey(data.api_key);
                    msgDiv.innerHTML = '<div class="success-msg">login successful</div>';
                } else {
                    const error = await response.text();
                    msgDiv.innerHTML = `<div class="error-msg">${escapeHtml(error)}</div>`;
                }
            } catch (e) {
                msgDiv.innerHTML = '<div class="error-msg">login failed</div>';
            }
        }
        
        async function doRevoke() {
            const apiKey = localStorage.getItem('snip_api_key');
            const msgDiv = document.getElementById('revoke-msg');
            
            if (!apiKey) return;
            
            if (!confirm('Are you sure? This will invalidate your old API key.')) return;
            
            try {
                const response = await fetch('/api/revoke-key', {
                    method: 'POST',
                    headers: { 'X-API-Key': apiKey }
                });
                
                if (response.ok) {
                    const data = await response.json();
                    localStorage.setItem('snip_api_key', data.new_api_key);
                    showApiKey(data.new_api_key);
                    msgDiv.innerHTML = '<div class="success-msg">API key revoked and regenerated</div>';
                } else {
                    const error = await response.text();
                    msgDiv.innerHTML = `<div class="error-msg">${escapeHtml(error)}</div>`;
                }
            } catch (e) {
                msgDiv.innerHTML = '<div class="error-msg">revoke failed</div>';
            }
        }
        
        function doLogout() {
            localStorage.removeItem('snip_api_key');
            localStorage.removeItem('snip_username');
            document.getElementById('login-form').style.display = 'block';
            document.getElementById('api-key-box').style.display = 'none';
            document.getElementById('login-user').value = '';
            document.getElementById('login-pass').value = '';
            document.getElementById('login-msg').innerHTML = '';
            document.getElementById('revoke-msg').innerHTML = '';
        }
        
        function showApiKey(key) {
            document.getElementById('login-form').style.display = 'none';
            document.getElementById('api-key-box').style.display = 'block';
            document.getElementById('api-key-value').textContent = key;
        }
        
        function copyApiKey() {
            const key = document.getElementById('api-key-value').textContent;
            navigator.clipboard.writeText(key).then(() => {
                const msgDiv = document.getElementById('revoke-msg');
                msgDiv.innerHTML = '<div class="success-msg">copied to clipboard</div>';
            });
        }
        
        // Check for existing session
        const savedKey = localStorage.getItem('snip_api_key');
        if (savedKey) {
            showApiKey(savedKey);
        }

        loadSnippets();
    </script>
</body>
</html>
"#;
