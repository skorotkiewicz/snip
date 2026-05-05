pub const INDEX_HTML: &str = r##"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>snip</title>
    <link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='4' fill='%23333'/%3E%3Ctext x='16' y='24' text-anchor='middle' font-family='monospace' font-size='20' font-weight='bold' fill='%23fff'%3ES%3C/text%3E%3C/svg%3E">
    <link id="hljs-theme" rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/scala.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/zig.min.js"></script>
    <style>
        /* ===== Reset & Variables ===== */
        *, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }

        :root {
            --bg: #f5f5f5;
            --fg: #333;
            --card: #fff;
            --border: #ccc;
            --border-strong: #333;
            --code-bg: #eee;
            --muted: #666;
            --link: #333;
            --star-bg: #fff;
            --star-border: #ccc;
            --star-hover: #999;
            --danger: #c00;
            --danger-hover: #f00;
            --success: #0a0;
            --error: #c00;
        }

        [data-theme="dark"] {
            --bg: #1a1a1a;
            --fg: #e0e0e0;
            --card: #111;
            --border: #444;
            --border-strong: #666;
            --code-bg: #333;
            --muted: #999;
            --link: #e0e0e0;
            --star-bg: #2a2a2a;
            --star-border: #555;
            --star-hover: #777;
            --danger: #f55;
            --danger-hover: #f77;
            --success: #5f5;
            --error: #f55;
        }

        /* ===== Base ===== */
        body {
            font-family: "Courier New", "Liberation Mono", monospace;
            background: var(--bg);
            color: var(--fg);
            line-height: 1.6;
            padding: 2rem 1rem;
            transition: background 0.3s, color 0.3s;
        }

        .container { max-width: 80ch; margin: 0 auto; }

        /* ===== Header ===== */
        .site-header {
            display: flex;
            align-items: baseline;
            justify-content: space-between;
            flex-wrap: wrap;
            margin-bottom: 1rem;
            padding-bottom: 0.5rem;
            border-bottom: 2px solid var(--border-strong);
        }
        .site-header h1 {
            font-size: 1.5rem;
        }
        .site-header h1 a { color: inherit; text-decoration: none; }
        .site-header h1 a:hover { text-decoration: underline; }
        .header-nav {
            font-size: 0.875rem;
            font-weight: normal;
            display: flex;
            gap: 0.5rem;
            align-items: center;
        }
        .header-nav a { color: var(--link); }
        .theme-toggle {
            cursor: pointer;
            user-select: none;
        }

        /* ===== Help Box ===== */
        .help {
            background: var(--card);
            border: 1px solid var(--border);
            padding: 1rem;
            margin-bottom: 1.5rem;
        }
        .help code {
            background: var(--code-bg);
            padding: 0.1rem 0.3rem;
        }
        .help .hint {
            font-size: 0.75rem;
            margin-top: 0.5rem;
            color: var(--muted);
        }
        .help .arg { color: var(--muted); }

        /* ===== Panels (shared for search & auth) ===== */
        .panel {
            background: var(--card);
            border: 1px solid var(--border);
            padding: 1rem;
            margin-bottom: 1.5rem;
        }
        .panel h3 {
            font-size: 1rem;
            margin-bottom: 0.75rem;
            padding-bottom: 0.25rem;
            border-bottom: 1px solid var(--border);
        }

        /* ===== Forms ===== */
        .form-row {
            display: flex;
            gap: 0.5rem;
            flex-wrap: wrap;
        }
        .form-row input,
        .form-row select {
            font-family: inherit;
            padding: 0.25rem 0.5rem;
            border: 1px solid var(--border);
            background: var(--bg);
            color: var(--fg);
        }
        .form-row input[type="text"],
        .form-row input[type="password"] {
            flex: 1;
            min-width: 120px;
        }
        .form-row select { flex: 1; min-width: 100px; }
        .btn {
            font-family: inherit;
            padding: 0.25rem 0.75rem;
            cursor: pointer;
            border: none;
            background: var(--border-strong);
            color: var(--bg);
        }
        .btn:hover { background: var(--fg); }
        .btn-secondary {
            background: var(--card);
            color: var(--fg);
            border: 1px solid var(--border-strong);
        }
        .btn-secondary:hover {
            background: var(--border-strong);
            color: var(--card);
        }
        .form-sub {
            margin-top: 0.5rem;
            font-size: 0.875rem;
        }
        .form-sub a { color: var(--link); }
        .msg { margin-top: 0.5rem; }
        .msg-error { color: var(--error); }
        .msg-success { color: var(--success); }

        /* ===== Search ===== */
        .search-input { flex: 2 !important; min-width: 150px !important; }
        .clear-search {
            font-size: 0.875rem;
            color: var(--muted);
            margin-top: 0.5rem;
        }
        .clear-search a { color: var(--link); }

        /* ===== API Key ===== */
        .api-key-display {
            border: 1px solid var(--border);
            padding: 0.75rem;
            margin-top: 0.5rem;
        }
        .api-key-display pre {
            margin: 0.5rem 0;
            font-family: inherit;
            background: var(--card);
            padding: 0.5rem;
            border: 1px solid var(--border);
            word-break: break-all;
        }
        .api-key-actions {
            display: flex;
            gap: 0.5rem;
            flex-wrap: wrap;
            margin-top: 0.5rem;
        }

        /* ===== Snippets ===== */
        .snippet { margin-bottom: 2rem; }
        .snippet-header {
            font-size: 1rem;
            font-weight: bold;
            margin-bottom: 0.5rem;
            padding: 0.25rem 0;
        }
        .snippet-lang {
            font-size: 0.75rem;
            color: var(--muted);
            background: var(--code-bg);
            padding: 0.1rem 0.4rem;
            margin-left: 0.5rem;
            text-transform: lowercase;
        }
        .snippet-content {
            background: var(--card);
            border: 1px solid var(--border-strong);
            margin-bottom: 0.5rem;
        }
        .snippet-content pre {
            margin: 0;
            font-family: inherit;
            white-space: pre-wrap;
            word-break: break-word;
            overflow-x: auto;
            line-height: 1.5;
        }
        .snippet-meta {
            font-size: 0.875rem;
            color: var(--muted);
            text-align: right;
        }
        .snippet-meta a { color: var(--link); text-decoration: none; }
        .snippet-meta a:hover { text-decoration: underline; }
        .raw-link { font-size: 0.75rem; color: var(--muted); }
        .delete-btn {
            font-family: inherit;
            font-size: 0.875rem;
            background: none;
            border: none;
            color: var(--danger);
            cursor: pointer;
            padding: 0;
            margin-left: 0.5rem;
        }
        .delete-btn:hover { color: var(--danger-hover); text-decoration: underline; }
        .star-btn {
            font-family: inherit;
            font-size: 0.875rem;
            background: var(--star-bg);
            border: 1px solid var(--star-border);
            color: var(--muted);
            cursor: pointer;
            padding: 0.1rem 0.4rem;
            margin-left: 0.5rem;
            border-radius: 3px;
            transition: all 0.2s;
        }
        .star-btn:hover {
            background: var(--bg);
            border-color: var(--star-hover);
            color: var(--fg);
        }
        .star-btn.starred {
            background: #fff8e1;
            border-color: #ffc107;
            color: #ff8f00;
        }
        [data-theme="dark"] .star-btn.starred {
            background: #5c4a00;
            border-color: #ffab00;
            color: #ffd54f;
        }
        .star-count {
            font-size: 0.875rem;
            color: var(--muted);
            margin-left: 0.5rem;
        }
        .fork-count {
            font-size: 0.875rem;
            color: var(--muted);
            margin-left: 0.5rem;
        }
        .comment-count {
            font-size: 0.875rem;
            color: var(--muted);
            margin-left: 0.5rem;
            text-decoration: none;
        }
        .comment-count:hover {
            color: var(--fg);
            text-decoration: underline;
        }
        .forked-from {
            font-size: 0.875rem;
            color: var(--muted);
        }
        .forked-from a {
            color: var(--link);
            text-decoration: underline;
        }

        /* Fork Button */
        .fork-btn {
            font-family: inherit;
            font-size: 0.875rem;
            padding: 0.125rem 0.375rem;
            background: var(--star-bg);
            border: 1px solid var(--star-border);
            color: var(--fg);
            cursor: pointer;
            margin-left: 0.5rem;
            transition: all 0.2s;
        }
        .fork-btn:hover {
            background: var(--code-bg);
            border-color: var(--star-hover);
        }
        [data-theme="dark"] .fork-btn:hover {
            background: #2a2a2a;
            border-color: #777;
        }

        /* ===== Pagination ===== */
        .pagination {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 1rem;
            margin-top: 1.5rem;
            padding-top: 1rem;
            border-top: 1px solid var(--border);
        }
        .pagination button {
            font-family: inherit;
            padding: 0.25rem 0.75rem;
            background: var(--card);
            border: 1px solid var(--border-strong);
            color: var(--fg);
            cursor: pointer;
        }
        .pagination button:hover:not(:disabled) {
            background: var(--border-strong);
            color: var(--card);
        }
        .pagination button:disabled {
            opacity: 0.3;
            cursor: default;
        }

        /* ===== Comments ===== */
        .comments-panel {
            margin-top: 1.5rem;
        }
        .comment-form textarea {
            width: 100%;
            min-height: 6rem;
            resize: vertical;
            font-family: inherit;
            padding: 0.5rem;
            border: 1px solid var(--border);
            background: var(--bg);
            color: var(--fg);
        }
        .comment-form-actions {
            display: flex;
            justify-content: space-between;
            align-items: center;
            gap: 0.75rem;
            margin-top: 0.75rem;
            flex-wrap: wrap;
        }
        .comment-replying {
            font-size: 0.875rem;
            color: var(--muted);
        }
        .comment-login-hint {
            font-size: 0.875rem;
            color: var(--muted);
        }
        .comment-login-hint a {
            color: var(--link);
        }
        .comments-list {
            display: flex;
            flex-direction: column;
            gap: 1rem;
            margin-top: 1rem;
        }
        .comment {
            border-left: 1px solid var(--border);
            padding-left: 1rem;
        }
        .comment-children {
            margin-top: 1rem;
            display: flex;
            flex-direction: column;
            gap: 1rem;
        }
        .comment-meta {
            font-size: 0.8rem;
            color: var(--muted);
            margin-bottom: 0.4rem;
        }
        .comment-body {
            white-space: pre-wrap;
            word-break: break-word;
        }
        .comment-actions {
            display: flex;
            flex-wrap: wrap;
            gap: 0.5rem;
            margin-top: 0.6rem;
        }
        .comment-btn {
            font-family: inherit;
            font-size: 0.8rem;
            background: var(--card);
            color: var(--fg);
            border: 1px solid var(--border);
            padding: 0.15rem 0.45rem;
            cursor: pointer;
        }
        .comment-btn:hover {
            background: var(--code-bg);
        }
        .comment-btn.liked {
            border-color: #ffc107;
            color: #ff8f00;
        }

        /* ===== Toast Notifications ===== */
        #toast-container {
            position: fixed;
            top: 1rem;
            right: 1rem;
            z-index: 1000;
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
            pointer-events: none;
        }
        .toast {
            padding: 0.5rem 1rem;
            font-family: inherit;
            font-size: 0.875rem;
            border: 1px solid var(--border);
            background: var(--card);
            color: var(--fg);
            animation: toast-in 0.2s ease;
            max-width: 30ch;
            pointer-events: auto;
        }
        .toast-success { border-color: var(--success); color: var(--success); }
        .toast-error { border-color: var(--error); color: var(--error); }
        .toast-exit { animation: toast-out 0.3s ease forwards; }
        @keyframes toast-in { from { opacity: 0; transform: translateX(1rem); } to { opacity: 1; transform: none; } }
        @keyframes toast-out { from { opacity: 1; } to { opacity: 0; transform: translateX(1rem); } }

        /* ===== Confirm Modal ===== */
        .confirm-overlay {
            position: fixed;
            inset: 0;
            background: rgba(0,0,0,0.5);
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 999;
        }
        .confirm-box {
            background: var(--card);
            border: 1px solid var(--border-strong);
            padding: 1.5rem;
            max-width: 40ch;
            font-family: inherit;
        }
        .confirm-box p { margin-bottom: 1rem; }
        .confirm-actions { display: flex; gap: 0.5rem; justify-content: flex-end; }
        .btn-danger { background: var(--danger); color: #fff; border: none; }
        .btn-danger:hover { background: var(--danger-hover); }

        /* ===== Misc ===== */
        .loading, .empty {
            text-align: center;
            padding: 2rem;
            color: var(--muted);
        }
        [hidden] { display: none !important; }
        .hljs { color: var(--fg); background: var(--bg); }
    </style>
</head>
<body>
    <div id="toast-container" aria-live="polite"></div>

    <div id="confirm-overlay" class="confirm-overlay" hidden>
        <div class="confirm-box" role="dialog" aria-modal="true">
            <p id="confirm-msg"></p>
            <div class="confirm-actions">
                <button class="btn btn-danger" id="confirm-yes">confirm</button>
                <button class="btn btn-secondary" id="confirm-no">cancel</button>
            </div>
        </div>
    </div>

    <div class="container">
        <header class="site-header">
            <h1><a href="/">snip</a><span id="header-suffix"> ~ code snippets</span></h1>
            <nav class="header-nav">
                <span class="theme-toggle" id="theme-toggle" tabindex="0" role="button" aria-label="Toggle theme">🌙</span>
                <a href="#" id="search-toggle">+search</a>
                <a href="#" id="auth-toggle">+auth</a>
            </nav>
        </header>

        <section class="help" id="help-box">
            <p>$ <strong>echo</strong> <span class="arg">"text"</span> | <strong>snip</strong> <em>--desc</em> <span class="arg">"note"</span> <em>--lang</em> <span class="arg">rust</span></p>
            <p class="hint"># POST /api/register {"username", "password"} to get API key</p>
        </section>

        <section class="panel" id="search-box" hidden>
            <h3>~ search</h3>
            <form class="form-row" id="search-form">
                <input type="text" id="search-input" class="search-input" placeholder="search in content or description...">
                <select id="search-lang">
                    <option value="all">all languages</option>
                    <option value="plaintext">plaintext</option>
                    <option value="bash">bash</option>
                    <option value="c">c</option>
                    <option value="cpp">cpp</option>
                    <option value="csharp">csharp</option>
                    <option value="css">css</option>
                    <option value="go">go</option>
                    <option value="html">html</option>
                    <option value="java">java</option>
                    <option value="javascript">javascript</option>
                    <option value="json">json</option>
                    <option value="kotlin">kotlin</option>
                    <option value="lua">lua</option>
                    <option value="markdown">markdown</option>
                    <option value="php">php</option>
                    <option value="python">python</option>
                    <option value="ruby">ruby</option>
                    <option value="rust">rust</option>
                    <option value="scala">scala</option>
                    <option value="shell">shell</option>
                    <option value="sql">sql</option>
                    <option value="swift">swift</option>
                    <option value="typescript">typescript</option>
                    <option value="yaml">yaml</option>
                    <option value="zig">zig</option>
                </select>
                <button type="submit" class="btn">search</button>
            </form>
            <div class="clear-search" id="clear-search" hidden>
                <a href="#" id="clear-search-link">&lt; clear search</a>
            </div>
        </section>

        <section class="panel" id="auth-box" hidden>
            <h3>~ auth</h3>

            <form id="login-form">
                <div class="form-row">
                    <input type="text" id="login-user" placeholder="username" autocomplete="username">
                    <input type="password" id="login-pass" placeholder="password" autocomplete="current-password">
                    <button type="submit" class="btn">login</button>
                </div>
                <div class="form-sub"><a href="#" id="show-register-link">register</a></div>
                <div id="login-msg" class="msg"></div>
            </form>

            <form id="register-form" hidden>
                <div class="form-row">
                    <input type="text" id="register-user" placeholder="username (3-32 chars)" autocomplete="username">
                    <input type="password" id="register-pass" placeholder="password (6+ chars)" autocomplete="new-password">
                    <button type="submit" class="btn">register</button>
                </div>
                <div class="form-sub"><a href="#" id="show-login-link">&lt; back to login</a></div>
                <div id="register-msg" class="msg"></div>
            </form>

            <div id="api-key-box" hidden>
                <div class="api-key-display">
                    <strong>API Key:</strong>
                    <pre id="api-key-value"></pre>
                    <div class="api-key-actions">
                        <button class="btn btn-secondary" id="copy-key-btn">copy</button>
                        <button class="btn btn-secondary" id="revoke-btn">revoke &amp; regenerate</button>
                        <button class="btn btn-secondary" id="change-pass-btn">change password</button>
                        <button class="btn btn-secondary" id="logout-btn">logout</button>
                    </div>
                    <div id="revoke-msg" class="msg"></div>
                </div>
                <form id="change-password-form" hidden style="margin-top: 1rem;">
                    <div class="form-row">
                        <input type="password" id="old-password" placeholder="old password" autocomplete="current-password">
                        <input type="password" id="new-password" placeholder="new password (6+ chars)" autocomplete="new-password">
                        <button type="submit" class="btn">change</button>
                        <button type="button" class="btn btn-secondary" id="cancel-change-pass">cancel</button>
                    </div>
                    <div id="change-password-msg" class="msg"></div>
                </form>
            </div>
        </section>

        <main id="snippets">
            <div class="loading">loading...</div>
        </main>

        <nav class="pagination" id="pagination"></nav>

        <section class="panel comments-panel" id="comments-panel" hidden>
            <h3>~ comments</h3>
            <form class="comment-form" id="comment-form" hidden>
                <textarea id="comment-input" placeholder="write a comment or reply..."></textarea>
                <div class="comment-form-actions">
                    <div class="comment-replying" id="comment-replying" hidden>
                        replying to <span id="comment-reply-author"></span>
                        <button type="button" class="btn btn-secondary" id="comment-reply-cancel">cancel reply</button>
                    </div>
                    <button type="submit" class="btn">post comment</button>
                </div>
                <div id="comment-msg" class="msg"></div>
            </form>
            <div class="comment-login-hint" id="comment-login-hint" hidden>
                login from `+auth` to post comments, reply, and like.
            </div>
            <div class="comments-list" id="comments-list">
                <div class="loading">loading comments...</div>
            </div>
        </section>
    </div>

    <script>
    /* ==========================================================
       snip — Code Snippet Viewer
       ========================================================== */

    // ===== Configuration =====
    const PER_PAGE = 10;

    // ===== State =====
    const state = {
        page: 1,
        totalPages: 1,
        searchQuery: '',
        searchLang: 'all',
        profileUser: null,
        snippetId: null,
        replyParentId: null,
        replyAuthor: null,
    };

    // Parse URL routing
    const pathParts = window.location.pathname.split('/');
    if (pathParts[1] === 'u' && pathParts[2]) state.profileUser = pathParts[2];
    if (pathParts[1] === 's' && pathParts[2]) state.snippetId = pathParts[2];

    // ===== Utilities =====
    function escapeHtml(text) {
        const d = document.createElement('div');
        d.textContent = text;
        return d.innerHTML;
    }

    function formatDate(dateStr) {
        const normalized = dateStr.includes('T')
            ? dateStr
            : dateStr.includes(' ')
                ? dateStr.replace(' ', 'T') + 'Z'
                : dateStr;
        return new Date(normalized).toISOString().slice(0, 16).replace('T', ' ');
    }

    function getAuth() {
        return {
            apiKey: localStorage.getItem('snip_api_key'),
            username: localStorage.getItem('snip_username'),
        };
    }

    function setAuth(apiKey, username) {
        localStorage.setItem('snip_api_key', apiKey);
        localStorage.setItem('snip_username', username);
    }

    function clearAuth() {
        localStorage.removeItem('snip_api_key');
        localStorage.removeItem('snip_username');
    }

    // ===== Toast =====
    function toast(message, type = 'info') {
        const el = document.createElement('div');
        el.className = `toast toast-${type}`;
        el.textContent = message;
        document.getElementById('toast-container').appendChild(el);
        setTimeout(() => {
            el.classList.add('toast-exit');
            el.addEventListener('animationend', () => el.remove());
        }, 2500);
    }

    // ===== Confirm Modal =====
    let confirmResolve = null;

    function showConfirm(message) {
        return new Promise(resolve => {
            confirmResolve = resolve;
            document.getElementById('confirm-msg').textContent = message;
            document.getElementById('confirm-overlay').hidden = false;
        });
    }

    function resolveConfirm(result) {
        document.getElementById('confirm-overlay').hidden = true;
        if (confirmResolve) {
            confirmResolve(result);
            confirmResolve = null;
        }
    }

    // ===== API Helper =====
    async function apiFetch(url, options = {}) {
        const { apiKey, ...opts } = options;
        if (apiKey) {
            opts.headers = { ...(opts.headers || {}), 'X-API-Key': apiKey };
        }
        return fetch(url, opts);
    }

    function updateCommentComposer() {
        const panel = document.getElementById('comments-panel');
        const form = document.getElementById('comment-form');
        const loginHint = document.getElementById('comment-login-hint');
        const replying = document.getElementById('comment-replying');
        const replyAuthor = document.getElementById('comment-reply-author');
        const { apiKey } = getAuth();

        panel.hidden = !state.snippetId;
        form.hidden = !apiKey || !state.snippetId;
        loginHint.hidden = !!apiKey || !state.snippetId;
        replying.hidden = !state.replyParentId;
        replyAuthor.textContent = state.replyAuthor || '';
    }

    function clearReplyTarget() {
        state.replyParentId = null;
        state.replyAuthor = null;
        updateCommentComposer();
    }

    function startReply(parentId, author) {
        state.replyParentId = parseInt(parentId, 10);
        state.replyAuthor = author;
        updateCommentComposer();
        document.getElementById('comment-input').focus();
    }

    function renderCommentTree(comments) {
        const { apiKey } = getAuth();

        if (!comments.length) {
            return '<div class="empty">No comments yet.</div>';
        }

        function renderNodes(nodes) {
            return nodes.map(comment => {
                const likeButton = apiKey
                    ? `<button class="comment-btn${comment.liked ? ' liked' : ''}" data-comment-action="like" data-id="${comment.id}">${comment.liked ? '♥' : '♡'} ${comment.likes || 0}</button>`
                    : `<span class="star-count">♡ ${comment.likes || 0}</span>`;
                const replyButton = apiKey
                    ? `<button class="comment-btn" data-comment-action="reply" data-id="${comment.id}" data-author="${escapeHtml(comment.author)}">reply</button>`
                    : '';
                const deleteButton = comment.can_delete
                    ? `<button class="comment-btn" data-comment-action="delete" data-id="${comment.id}">delete</button>`
                    : '';
                const children = comment.children && comment.children.length
                    ? `<div class="comment-children">${renderNodes(comment.children)}</div>`
                    : '';

                return `<article class="comment" id="comment-${comment.id}">
                    <div class="comment-meta">${escapeHtml(comment.author)} · ${formatDate(comment.created_at)}</div>
                    <div class="comment-body">${escapeHtml(comment.content)}</div>
                    <div class="comment-actions">${likeButton}${replyButton}${deleteButton}</div>
                    ${children}
                </article>`;
            }).join('');
        }

        return renderNodes(comments);
    }

    function renderComments(comments) {
        document.getElementById('comments-list').innerHTML = renderCommentTree(comments);
    }

    // ===== Rendering =====
    function renderSnippets(snippets) {
        const container = document.getElementById('snippets');
        if (!snippets.length) {
            container.innerHTML = '<div class="empty">No snippets yet.</div>';
            return;
        }
        const { apiKey, username } = getAuth();

        container.innerHTML = snippets.map(s => {
            const author = state.profileUser
                ? escapeHtml(s.author)
                : `<a href="/u/${escapeHtml(s.author)}">${escapeHtml(s.author)}</a>`;
            const langTag = s.language && s.language !== 'plaintext'
                ? `<span class="snippet-lang">${escapeHtml(s.language)}</span>` : '';
            const desc = (s.description || langTag)
                ? `<div class="snippet-header">${s.description ? escapeHtml(s.description) : ''}${langTag}</div>` : '';
            const isOwner = apiKey && s.author === username;
            const deleteBtn = isOwner
                ? ` <button class="delete-btn" data-action="delete" data-id="${s.id}">[x]</button>` : '';
            const starCount = s.stars || 0;
            const starBtn = apiKey
                ? ` <button class="star-btn${s.starred ? ' starred' : ''}" data-action="star" data-id="${s.id}" title="${s.starred ? 'Unstar' : 'Star'}">${s.starred ? '★' : '☆'} ${starCount}</button>`
                : ` <span class="star-count">☆ ${starCount}</span>`;
            const forkCount = s.forks || 0;
            const forkBtn = apiKey
                ? ` <button class="fork-btn" data-action="fork" data-id="${s.id}" title="Fork">🍴 ${forkCount}</button>`
                : (forkCount > 0 ? ` <span class="fork-count">🍴 ${forkCount}</span>` : '');
            const commentCount = s.comments || 0;
            const commentBtn = commentCount > 0
                ? ` <a href="/s/${s.id}" class="comment-count" title="View comments">// ${commentCount}</a>`
                : '';
            const forkedFrom = s.forked_from
                ? ` · <span class="forked-from">forked from <a href="/s/${s.forked_from}">/s/${s.forked_from}</a></span>`
                : '';
            const langClass = s.language && s.language !== 'plaintext' ? ` class="language-${s.language}"` : '';
            const views = s.views || 0;
            return `<div class="snippet" id="snippet-${s.id}">
                    ${desc}
                    <div class="snippet-content"><pre><code${langClass}>${escapeHtml(s.content)}</code></pre></div>
                    <div class="snippet-meta">${author} · <a href="/s/${s.id}">${formatDate(s.created_at)}</a> · ${views} views${starBtn}${forkBtn}${commentBtn}${forkedFrom} · <a href="/raw/${s.id}" class="raw-link">raw</a>${deleteBtn}</div>
                </div>`;
        }).join('');

        container.querySelectorAll('pre code').forEach(block => hljs.highlightElement(block));
    }

    function renderPagination() {
        const container = document.getElementById('pagination');
        if (state.totalPages <= 1) { container.innerHTML = ''; return; }
        container.innerHTML = `
            <button ${state.page === 1 ? 'disabled' : ''} data-page="${state.page - 1}">&lt; prev</button>
            <span>${state.page}/${state.totalPages}</span>
            <button ${state.page === state.totalPages ? 'disabled' : ''} data-page="${state.page + 1}">next &gt;</button>`;
    }

    // ===== Data Loading =====
    async function loadSingleSnippet() {
        const container = document.getElementById('snippets');
        const { apiKey } = getAuth();
        try {
            const resp = await apiFetch(`/api/snippets/${state.snippetId}`, { apiKey });
            if (!resp.ok) {
                container.innerHTML = resp.status === 404
                    ? '<div class="empty">Snippet not found</div>'
                    : '<div class="empty">Error loading snippet</div>';
                return;
            }
            const snippet = await resp.json();
            document.getElementById('header-suffix').textContent = ` ~ snippet #${snippet.id}`;
            document.getElementById('help-box').hidden = true;
            document.getElementById('search-box').hidden = true;
            document.getElementById('auth-box').hidden = true;
            document.getElementById('pagination').innerHTML = '';
            renderSnippets([snippet]);
            updateCommentComposer();
            await loadComments();
        } catch (e) {
            container.innerHTML = '<div class="empty">Error loading snippet</div>';
        }
    }

    async function loadSnippets(page = 1) {
        state.page = page;
        const container = document.getElementById('snippets');
        const { apiKey } = getAuth();
        clearReplyTarget();
        document.getElementById('comments-panel').hidden = true;
        let url;
        if (state.searchQuery || (state.searchLang && state.searchLang !== 'all')) {
            const params = new URLSearchParams({ page, limit: PER_PAGE });
            if (state.searchQuery) params.append('q', state.searchQuery);
            if (state.searchLang !== 'all') params.append('lang', state.searchLang);
            url = `/api/search?${params}`;
        } else if (state.profileUser) {
            url = `/api/users/${state.profileUser}/snippets?page=${page}&limit=${PER_PAGE}`;
        } else {
            url = `/api/snippets?page=${page}&limit=${PER_PAGE}`;
        }
        try {
            const resp = await apiFetch(url, { apiKey });
            const data = await resp.json();
            state.totalPages = Math.ceil(data.total / PER_PAGE) || 1;
            renderSnippets(data.snippets);
            renderPagination();
        } catch (e) {
            container.innerHTML = '<div class="empty">Error loading snippets</div>';
        }
    }

    function refreshSnippets() {
        if (state.snippetId) loadSingleSnippet();
        else loadSnippets(state.page);
    }

    async function loadComments() {
        if (!state.snippetId) return;
        const { apiKey } = getAuth();
        const container = document.getElementById('comments-list');
        updateCommentComposer();
        container.innerHTML = '<div class="loading">loading comments...</div>';
        try {
            const resp = await apiFetch(`/api/snippets/${state.snippetId}/comments`, { apiKey });
            if (!resp.ok) {
                container.innerHTML = '<div class="empty">Error loading comments</div>';
                return;
            }
            const data = await resp.json();
            renderComments(data.comments || []);
        } catch (e) {
            container.innerHTML = '<div class="empty">Error loading comments</div>';
        }
    }

    async function submitComment(e) {
        e.preventDefault();
        const { apiKey } = getAuth();
        const content = document.getElementById('comment-input').value.trim();
        const msgDiv = document.getElementById('comment-msg');
        if (!apiKey) {
            msgDiv.innerHTML = '<div class="msg-error">login required</div>';
            return;
        }
        if (!content) {
            msgDiv.innerHTML = '<div class="msg-error">comment cannot be empty</div>';
            return;
        }
        try {
            const resp = await apiFetch(`/api/snippets/${state.snippetId}/comments`, {
                method: 'POST',
                apiKey,
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ content, parent_id: state.replyParentId }),
            });
            if (resp.ok) {
                document.getElementById('comment-input').value = '';
                msgDiv.innerHTML = '';
                clearReplyTarget();
                await loadComments();
            } else {
                const error = await resp.text();
                msgDiv.innerHTML = `<div class="msg-error">${escapeHtml(error)}</div>`;
            }
        } catch (e) {
            msgDiv.innerHTML = '<div class="msg-error">failed to post comment</div>';
        }
    }

    async function toggleCommentLike(commentId) {
        const { apiKey } = getAuth();
        if (!apiKey) { toast('Please login to like comments', 'error'); return; }
        const button = document.querySelector(`[data-comment-action="like"][data-id="${commentId}"]`);
        const isLiked = button?.classList.contains('liked');
        try {
            const resp = await apiFetch(`/api/comments/${commentId}/like`, {
                method: isLiked ? 'DELETE' : 'POST',
                apiKey,
            });
            if (resp.ok) {
                await loadComments();
            } else {
                toast('Failed to update comment like', 'error');
            }
        } catch (e) {
            toast('Error liking comment', 'error');
        }
    }

    async function deleteComment(commentId) {
        const { apiKey } = getAuth();
        if (!apiKey) { toast('Please login to delete comments', 'error'); return; }
        if (!(await showConfirm('Delete this comment and its replies?'))) return;
        try {
            const resp = await apiFetch(`/api/comments/${commentId}`, {
                method: 'DELETE',
                apiKey,
            });
            if (resp.ok) {
                await loadComments();
                toast('Comment deleted', 'success');
            } else if (resp.status === 403) {
                toast('Not allowed to delete this comment', 'error');
            } else {
                toast('Failed to delete comment', 'error');
            }
        } catch (e) {
            toast('Error deleting comment', 'error');
        }
    }

    // ===== Snippet Actions =====
    async function deleteSnippet(id) {
        const { apiKey } = getAuth();
        if (!apiKey) { toast('Not authenticated', 'error'); return; }
        if (!(await showConfirm('Delete this snippet?'))) return;
        try {
            const resp = await apiFetch(`/api/snippets/${id}`, { method: 'DELETE', apiKey });
            if (resp.ok) {
                document.getElementById(`snippet-${id}`)?.remove();
                toast('Snippet deleted', 'success');
            } else if (resp.status === 403) {
                toast('Can only delete your own snippets', 'error');
            } else {
                toast('Failed to delete', 'error');
            }
        } catch (e) { toast('Error deleting snippet', 'error'); }
    }

    async function toggleStar(id, btn) {
        const { apiKey } = getAuth();
        if (!apiKey) { toast('Please login to star snippets', 'error'); return; }
        const isStarred = btn.classList.contains('starred');
        try {
            const resp = await apiFetch(`/api/snippets/${id}/star`, {
                method: isStarred ? 'DELETE' : 'POST',
                apiKey,
            });
            if (resp.ok) {
                const data = await resp.json();
                const count = data.total_stars || 0;
                btn.textContent = data.starred ? `★ ${count}` : `☆ ${count}`;
                btn.title = data.starred ? 'Unstar' : 'Star';
                btn.classList.toggle('starred', data.starred);
            } else if (resp.status === 401) {
                toast('Session expired. Please login again.', 'error');
            } else {
                toast('Failed to update star', 'error');
            }
        } catch (e) { toast('Error starring snippet', 'error'); }
    }

    async function forkSnippet(id) {
        const { apiKey } = getAuth();
        if (!apiKey) { toast('Please login to fork snippets', 'error'); return; }
        try {
            const resp = await apiFetch(`/api/snippets/${id}/fork`, {
                method: 'POST',
                apiKey,
            });
            if (resp.ok) {
                const data = await resp.json();
                toast(`Forked as snippet #${data.forked_id}`, 'success');
                // Refresh to show updated fork count and the new snippet
                setTimeout(() => {
                    window.location.href = `/s/${data.forked_id}`;
                }, 800);
            } else if (resp.status === 400) {
                toast('Cannot fork your own snippet', 'error');
            } else if (resp.status === 404) {
                toast('Snippet not found', 'error');
            } else if (resp.status === 401) {
                toast('Session expired. Please login again.', 'error');
            } else if (resp.status === 429) {
                toast('Rate limit: 10 forks per minute', 'error');
            } else {
                toast('Failed to fork snippet', 'error');
            }
        } catch (e) { toast('Error forking snippet', 'error'); }
    }

    // ===== Auth =====
    function showApiKey(key) {
        document.getElementById('login-form').hidden = true;
        document.getElementById('register-form').hidden = true;
        document.getElementById('api-key-box').hidden = false;
        document.getElementById('auth-box').hidden = false;
        document.getElementById('auth-toggle').textContent = '-auth';
        document.getElementById('api-key-value').textContent = key;
    }

    function resetAuthForms() {
        document.getElementById('login-form').hidden = false;
        document.getElementById('register-form').hidden = true;
        document.getElementById('api-key-box').hidden = true;
        document.getElementById('login-user').value = '';
        document.getElementById('login-pass').value = '';
        document.getElementById('register-user').value = '';
        document.getElementById('register-pass').value = '';
        document.getElementById('login-msg').innerHTML = '';
        document.getElementById('register-msg').innerHTML = '';
        document.getElementById('revoke-msg').innerHTML = '';
        hideChangePassword();
    }

    function showChangePassword() {
        document.getElementById('change-password-form').hidden = false;
        document.getElementById('change-password-msg').innerHTML = '';
    }

    function hideChangePassword() {
        document.getElementById('change-password-form').hidden = true;
        document.getElementById('old-password').value = '';
        document.getElementById('new-password').value = '';
        document.getElementById('change-password-msg').innerHTML = '';
    }

    async function doLogin(e) {
        e.preventDefault();
        const username = document.getElementById('login-user').value;
        const password = document.getElementById('login-pass').value;
        const msgDiv = document.getElementById('login-msg');
        if (!username || !password) {
            msgDiv.innerHTML = '<div class="msg-error">enter username and password</div>';
            return;
        }
        try {
            const resp = await fetch('/api/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password }),
            });
            if (resp.ok) {
                const data = await resp.json();
                setAuth(data.api_key, data.username);
                showApiKey(data.api_key);
                msgDiv.innerHTML = '<div class="msg-success">login successful</div>';
                refreshSnippets();
            } else {
                const error = await resp.text();
                msgDiv.innerHTML = `<div class="msg-error">${escapeHtml(error)}</div>`;
            }
        } catch (e) {
            msgDiv.innerHTML = '<div class="msg-error">login failed</div>';
        }
    }

    async function doRegister(e) {
        e.preventDefault();
        const username = document.getElementById('register-user').value;
        const password = document.getElementById('register-pass').value;
        const msgDiv = document.getElementById('register-msg');
        if (!username || !password) {
            msgDiv.innerHTML = '<div class="msg-error">enter username and password</div>'; return;
        }
        if (username.length < 3 || username.length > 32) {
            msgDiv.innerHTML = '<div class="msg-error">username must be 3-32 characters</div>'; return;
        }
        if (password.length < 6) {
            msgDiv.innerHTML = '<div class="msg-error">password must be at least 6 characters</div>'; return;
        }
        try {
            const resp = await fetch('/api/register', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password }),
            });
            if (resp.ok) {
                const data = await resp.json();
                setAuth(data.api_key, data.username);
                showApiKey(data.api_key);
                msgDiv.innerHTML = '<div class="msg-success">registration successful</div>';
                document.getElementById('register-user').value = '';
                document.getElementById('register-pass').value = '';
                refreshSnippets();
            } else if (resp.status === 409) {
                msgDiv.innerHTML = '<div class="msg-error">username already exists</div>';
            } else {
                const error = await resp.text();
                msgDiv.innerHTML = `<div class="msg-error">${escapeHtml(error)}</div>`;
            }
        } catch (e) {
            msgDiv.innerHTML = '<div class="msg-error">registration failed</div>';
        }
    }

    async function doRevoke() {
        const { apiKey, username } = getAuth();
        const msgDiv = document.getElementById('revoke-msg');
        if (!apiKey) return;
        if (!(await showConfirm('Are you sure? This will invalidate your old API key.'))) return;
        try {
            const resp = await apiFetch('/api/revoke-key', { method: 'POST', apiKey });
            if (resp.ok) {
                const data = await resp.json();
                setAuth(data.new_api_key, username);
                showApiKey(data.new_api_key);
                msgDiv.innerHTML = '<div class="msg-success">API key revoked and regenerated</div>';
            } else {
                const error = await resp.text();
                msgDiv.innerHTML = `<div class="msg-error">${escapeHtml(error)}</div>`;
            }
        } catch (e) {
            msgDiv.innerHTML = '<div class="msg-error">revoke failed</div>';
        }
    }

    async function doChangePassword(e) {
        e.preventDefault();
        const { apiKey } = getAuth();
        const oldPass = document.getElementById('old-password').value;
        const newPass = document.getElementById('new-password').value;
        const msgDiv = document.getElementById('change-password-msg');
        if (!oldPass || !newPass) {
            msgDiv.innerHTML = '<div class="msg-error">enter both old and new passwords</div>'; return;
        }
        if (newPass.length < 6) {
            msgDiv.innerHTML = '<div class="msg-error">new password must be at least 6 characters</div>'; return;
        }
        try {
            const resp = await apiFetch('/api/change-password', {
                method: 'POST',
                apiKey,
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ old_password: oldPass, new_password: newPass }),
            });
            if (resp.ok) {
                msgDiv.innerHTML = '<div class="msg-success">password changed successfully</div>';
                setTimeout(hideChangePassword, 2000);
            } else if (resp.status === 401) {
                msgDiv.innerHTML = '<div class="msg-error">invalid old password</div>';
            } else {
                const error = await resp.text();
                msgDiv.innerHTML = `<div class="msg-error">${escapeHtml(error)}</div>`;
            }
        } catch (e) {
            msgDiv.innerHTML = '<div class="msg-error">failed to change password</div>';
        }
    }

    function doLogout() {
        clearAuth();
        resetAuthForms();
        document.getElementById('auth-box').hidden = true;
        document.getElementById('auth-toggle').textContent = '+auth';
        toast('Logged out', 'success');
        refreshSnippets();
    }

    async function copyApiKey() {
        const key = document.getElementById('api-key-value').textContent;
        try {
            await navigator.clipboard.writeText(key);
            toast('Copied to clipboard', 'success');
        } catch (e) {
            toast('Failed to copy', 'error');
        }
    }

    // ===== Search =====
    function doSearch(e) {
        e.preventDefault();
        state.searchQuery = document.getElementById('search-input').value.trim();
        state.searchLang = document.getElementById('search-lang').value;
        if (!state.searchQuery && state.searchLang === 'all') { clearSearch(); return; }
        document.getElementById('clear-search').hidden = false;
        loadSnippets(1);
    }

    function clearSearch() {
        state.searchQuery = '';
        state.searchLang = 'all';
        document.getElementById('search-input').value = '';
        document.getElementById('search-lang').value = 'all';
        document.getElementById('clear-search').hidden = true;
        loadSnippets(1);
    }

    // ===== Theme =====
    function initTheme() {
        const saved = localStorage.getItem('snip_theme');
        const prefersDark = window.matchMedia?.('(prefers-color-scheme: dark)').matches;
        const isDark = saved === 'dark' || (!saved && prefersDark);
        applyTheme(isDark);
    }

    function applyTheme(isDark) {
        document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
        const toggle = document.getElementById('theme-toggle');
        toggle.textContent = isDark ? '☀️' : '🌙';
        toggle.title = isDark ? 'Switch to light mode' : 'Switch to dark mode';
        document.getElementById('hljs-theme').href = isDark
            ? 'https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css'
            : 'https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css';
    }

    function toggleTheme() {
        const isDark = document.documentElement.getAttribute('data-theme') === 'dark';
        const next = !isDark;
        localStorage.setItem('snip_theme', next ? 'dark' : 'light');
        applyTheme(next);
    }

    // ===== Event Binding =====
    function bindEvents() {
        // Theme
        document.getElementById('theme-toggle').addEventListener('click', toggleTheme);
        document.getElementById('theme-toggle').addEventListener('keydown', e => {
            if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); toggleTheme(); }
        });

        // Panel toggles
        document.getElementById('search-toggle').addEventListener('click', e => {
            e.preventDefault();
            const box = document.getElementById('search-box');
            const hidden = !box.hidden;
            box.hidden = hidden;
            e.currentTarget.textContent = hidden ? '+search' : '-search';
        });

        document.getElementById('auth-toggle').addEventListener('click', e => {
            e.preventDefault();
            const box = document.getElementById('auth-box');
            const hidden = !box.hidden;
            box.hidden = hidden;
            e.currentTarget.textContent = hidden ? '+auth' : '-auth';
        });

        // Search
        document.getElementById('search-form').addEventListener('submit', doSearch);
        document.getElementById('clear-search-link').addEventListener('click', e => {
            e.preventDefault(); clearSearch();
        });

        // Auth forms
        document.getElementById('login-form').addEventListener('submit', doLogin);
        document.getElementById('register-form').addEventListener('submit', doRegister);
        document.getElementById('show-register-link').addEventListener('click', e => {
            e.preventDefault();
            document.getElementById('login-form').hidden = true;
            document.getElementById('register-form').hidden = false;
            document.getElementById('login-msg').innerHTML = '';
        });
        document.getElementById('show-login-link').addEventListener('click', e => {
            e.preventDefault();
            document.getElementById('register-form').hidden = true;
            document.getElementById('login-form').hidden = false;
            document.getElementById('register-msg').innerHTML = '';
        });

        // API key actions
        document.getElementById('copy-key-btn').addEventListener('click', copyApiKey);
        document.getElementById('revoke-btn').addEventListener('click', doRevoke);
        document.getElementById('change-pass-btn').addEventListener('click', showChangePassword);
        document.getElementById('logout-btn').addEventListener('click', doLogout);
        document.getElementById('cancel-change-pass').addEventListener('click', hideChangePassword);
        document.getElementById('change-password-form').addEventListener('submit', doChangePassword);
        document.getElementById('comment-form').addEventListener('submit', submitComment);
        document.getElementById('comment-reply-cancel').addEventListener('click', clearReplyTarget);

        // Confirm modal
        document.getElementById('confirm-yes').addEventListener('click', () => resolveConfirm(true));
        document.getElementById('confirm-no').addEventListener('click', () => resolveConfirm(false));
        document.getElementById('confirm-overlay').addEventListener('click', e => {
            if (e.target === e.currentTarget) resolveConfirm(false);
        });
        document.addEventListener('keydown', e => {
            if (e.key === 'Escape' && !document.getElementById('confirm-overlay').hidden) {
                resolveConfirm(false);
            }
        });

        // Snippet actions (event delegation)
        document.getElementById('snippets').addEventListener('click', e => {
            const btn = e.target.closest('[data-action]');
            if (!btn) return;
            const { action, id } = btn.dataset;
            if (action === 'delete') deleteSnippet(id);
            if (action === 'star') toggleStar(id, btn);
            if (action === 'fork') forkSnippet(id);
        });

        // Pagination (event delegation)
        document.getElementById('pagination').addEventListener('click', e => {
            const btn = e.target.closest('[data-page]');
            if (!btn || btn.disabled) return;
            loadSnippets(parseInt(btn.dataset.page));
        });

        document.getElementById('comments-panel').addEventListener('click', e => {
            const btn = e.target.closest('[data-comment-action]');
            if (!btn) return;
            const { commentAction, id, author } = btn.dataset;
            if (commentAction === 'reply') startReply(id, author);
            if (commentAction === 'like') toggleCommentLike(id);
            if (commentAction === 'delete') deleteComment(id);
        });

        // System theme change
        window.matchMedia?.('(prefers-color-scheme: dark)').addEventListener('change', e => {
            if (!localStorage.getItem('snip_theme')) applyTheme(e.matches);
        });
    }

    // ===== Init =====
    function init() {
        initTheme();
        bindEvents();

        // Restore logged-in state (keep auth box hidden until toggled)
        const { apiKey } = getAuth();
        if (apiKey) {
            document.getElementById('api-key-value').textContent = apiKey;
            document.getElementById('login-form').hidden = true;
            document.getElementById('register-form').hidden = true;
            document.getElementById('api-key-box').hidden = false;
        }

        // Update header for profile view
        if (state.profileUser) {
            document.getElementById('header-suffix').textContent = ` ~ ${state.profileUser}`;
        }

        updateCommentComposer();

        // Load appropriate view
        if (state.snippetId) loadSingleSnippet();
        else loadSnippets();
    }

    init();
    </script>
</body>
</html>
"##;
