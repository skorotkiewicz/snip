pub const INDEX_HTML: &str = r##"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>snip</title>
    <link id="hljs-theme" rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css">
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

        /* Light mode (default) */
        :root {
            --bg-color: #f5f5f5;
            --text-color: #333;
            --container-bg: #fff;
            --border-color: #ccc;
            --border-strong: #333;
            --code-bg: #eee;
            --meta-color: #666;
            --link-color: #333;
            --accent-bg: #f5f5f5;
            --star-btn-bg: #fff;
            --star-btn-border: #ccc;
            --star-btn-hover: #999;
            --delete-color: #c00;
            --delete-hover: #f00;
            --success-color: #0a0;
            --error-color: #c00;
        }

        /* Dark mode */
        [data-theme="dark"] {
            --bg-color: #1a1a1a;
            --text-color: #e0e0e0;
            --container-bg: #111;
            --border-color: #444;
            --border-strong: #666;
            --code-bg: #333;
            --meta-color: #999;
            --link-color: #e0e0e0;
            --accent-bg: #333;
            --star-btn-bg: #2a2a2a;
            --star-btn-border: #555;
            --star-btn-hover: #777;
            --delete-color: #f55;
            --delete-hover: #f77;
            --success-color: #5f5;
            --error-color: #f55;
        }

        body {
            font-family: "Courier New", "Liberation Mono", monospace;
            background: var(--bg-color);
            color: var(--text-color);
            line-height: 1.6;
            padding: 2rem 1rem;
            transition: background 0.3s, color 0.3s;
        }
        .container {
            max-width: 80ch;
            margin: 0 auto;
        }
        h1 {
            font-size: 1.5rem;
            margin-bottom: 1rem;
            padding-bottom: 0.5rem;
            border-bottom: 2px solid var(--border-strong);
        }
        h1 a {
            color: inherit;
            text-decoration: none;
        }
        h1 a:hover {
            text-decoration: underline;
        }
        .help {
            background: var(--container-bg);
            border: 1px solid var(--border-color);
            padding: 1rem;
            margin-bottom: 1.5rem;
        }
        .help code {
            background: var(--code-bg);
            padding: 0.1rem 0.3rem;
        }
        .auth-box {
            background: var(--container-bg);
            border: 1px solid var(--border-color);
            padding: 1rem;
            margin-bottom: 1.5rem;
        }
        .auth-box h3 {
            font-size: 1rem;
            margin-bottom: 0.75rem;
            padding-bottom: 0.25rem;
            border-bottom: 1px solid var(--border-color);
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
            border: 1px solid var(--border-color);
            background: var(--bg-color);
            color: var(--text-color);
            flex: 1;
            min-width: 120px;
        }
        .auth-form button {
            font-family: inherit;
            padding: 0.25rem 0.75rem;
            background: var(--border-strong);
            color: var(--bg-color);
            border: none;
            cursor: pointer;
        }
        .auth-form button:hover {
            background: var(--text-color);
        }
        .api-key-display {
            border: 1px solid var(--border-color);
            padding: 0.75rem;
            margin-top: 0.5rem;
        }
        .api-key-display pre {
            margin: 0.5rem 0;
            font-family: inherit;
            background: var(--container-bg);
            padding: 0.5rem;
            border: 1px solid var(--border-color);
            word-break: break-all;
        }
        .api-key-display button {
            font-family: inherit;
            padding: 0.25rem 0.5rem;
            background: var(--container-bg);
            border: 1px solid var(--border-strong);
            color: var(--text-color);
            cursor: pointer;
            margin-top: 0.5rem;
        }
        .api-key-display button:hover {
            background: var(--border-strong);
            color: var(--container-bg);
        }
        .error-msg {
            color: var(--error-color);
            margin-top: 0.5rem;
        }
        .success-msg {
            color: var(--success-color);
            margin-top: 0.5rem;
        }
        .snippet {
            margin-bottom: 2rem;
        }
        .snippet-desc {
            font-size: 1rem;
            font-weight: bold;
            color: var(--text-color);
            margin-bottom: 0.5rem;
            padding: 0.25rem 0;
        }
        .snippet-lang {
            font-size: 0.75rem;
            color: var(--meta-color);
            background: var(--code-bg);
            padding: 0.1rem 0.4rem;
            margin-left: 0.5rem;
            text-transform: lowercase;
        }
        .snippet-content {
            background: var(--container-bg);
            border: 1px solid var(--border-strong);
            margin-bottom: 0.5rem;
        }
        .snippet-content pre {
            margin: 0;
            font-family: inherit;
            white-space: pre-wrap;
            word-break: break-word;
            overflow-x: auto;
            color: var(--text-color);
            line-height: 1.5;
        }
        .snippet-meta {
            font-size: 0.875rem;
            color: var(--meta-color);
            text-align: right;
        }
        .snippet-meta a {
            color: var(--link-color);
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
            color: var(--delete-color);
            cursor: pointer;
            padding: 0;
            margin-left: 0.5rem;
        }
        .delete-btn:hover {
            color: var(--delete-hover);
            text-decoration: underline;
        }
        .star-btn {
            font-family: inherit;
            font-size: 0.875rem;
            background: var(--star-btn-bg);
            border: 1px solid var(--star-btn-border);
            color: var(--meta-color);
            cursor: pointer;
            padding: 0.1rem 0.4rem;
            margin-left: 0.5rem;
            border-radius: 3px;
            transition: all 0.2s;
        }
        .star-btn:hover {
            background: var(--bg-color);
            border-color: var(--star-btn-hover);
            color: var(--text-color);
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
            color: var(--meta-color);
            margin-left: 0.5rem;
        }
        .pagination {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 1rem;
            margin-top: 1.5rem;
            padding-top: 1rem;
            border-top: 1px solid var(--border-color);
        }
        .pagination button {
            font-family: inherit;
            padding: 0.25rem 0.75rem;
            background: var(--container-bg);
            border: 1px solid var(--border-strong);
            color: var(--text-color);
            cursor: pointer;
        }
        .pagination button:hover:not(:disabled) {
            background: var(--border-strong);
            color: var(--container-bg);
        }
        .pagination button:disabled {
            opacity: 0.3;
            cursor: default;
        }
        .loading, .empty {
            text-align: center;
            padding: 2rem;
            color: var(--meta-color);
        }
        .search-box {
            background: var(--container-bg);
            border: 1px solid var(--border-color);
            padding: 1rem;
            margin-bottom: 1.5rem;
        }
        .search-box h3 {
            font-size: 1rem;
            margin-bottom: 0.75rem;
            padding-bottom: 0.25rem;
            border-bottom: 1px solid var(--border-color);
        }
        .search-form {
            display: flex;
            gap: 0.5rem;
            flex-wrap: wrap;
        }
        .search-form input, .search-form select {
            font-family: inherit;
            padding: 0.25rem 0.5rem;
            border: 1px solid var(--border-color);
            background: var(--bg-color);
            color: var(--text-color);
        }
        .search-form input[type="text"] {
            flex: 2;
            min-width: 150px;
        }
        .search-form select {
            flex: 1;
            min-width: 100px;
        }
        .search-form button {
            font-family: inherit;
            padding: 0.25rem 0.75rem;
            background: var(--border-strong);
            color: var(--bg-color);
            border: none;
            cursor: pointer;
        }
        .search-form button:hover {
            background: var(--text-color);
        }
        .clear-search {
            font-size: 0.875rem;
            color: var(--meta-color);
            margin-top: 0.5rem;
        }
        .clear-search a {
            color: var(--link-color);
        }
        .theme-toggle {
            cursor: pointer;
            user-select: none;
            margin-left: 0.5rem;
        }
        .hljs {
          color: var(--text-color);
          background: var(--bg-color);
        }
    </style>
</head>
<body>
    <div class="container">
        <h1><a href="/">snip</a><span id="header-suffix"> ~ code snippets</span> <span style="font-size: 0.875rem; font-weight: normal; float: right;"><span class="theme-toggle" onclick="toggleTheme()" title="Toggle dark mode">🌙</span> <a href="#" id="search-toggle" onclick="toggleSearch(); return false;">+search</a> <a href="#" id="auth-toggle" onclick="toggleAuth(); return false;">+auth</a></span></h1>

        <div class="help" id="help-box">
            <p>$ <strong>echo</strong> <span style="color: #888">"text"</span> | <strong>snip</strong> <em>--desc</em> <span style="color: #888">"note"</span> <em>--lang</em> <span style="color: #888">rust</span></p>
            <p style="font-size: 12px; margin-top: 0.5rem; color: #666;"># POST /api/register {username, password} to get API key</p>
        </div>

        <div class="search-box" id="search-box" style="display:none;">
            <h3>~ search</h3>
            <div class="search-form">
                <input type="text" id="search-input" placeholder="search in content or description...">
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
                <button onclick="doSearch()">search</button>
            </div>
            <div class="clear-search" id="clear-search" style="display:none;">
                <a href="#" onclick="clearSearch(); return false;">&lt; clear search</a>
            </div>
        </div>

        <div class="auth-box" id="auth-box" style="display:none;">
            <h3>~ auth</h3>
            <div id="login-form">
                <div class="auth-form">
                    <input type="text" id="login-user" placeholder="username">
                    <input type="password" id="login-pass" placeholder="password">
                    <button onclick="doLogin()">login</button>
                </div>
                <div style="margin-top: 0.5rem; font-size: 0.875rem;">
                    <a href="#" onclick="showRegister(); return false;" id="show-register-link">register</a>
                </div>
                <div id="login-msg"></div>
            </div>
            <div id="register-form" style="display:none;">
                <div class="auth-form">
                    <input type="text" id="register-user" placeholder="username (3-32 chars)">
                    <input type="password" id="register-pass" placeholder="password (6+ chars)">
                    <button onclick="doRegister()">register</button>
                </div>
                <div style="margin-top: 0.5rem; font-size: 0.875rem;">
                    <a href="#" onclick="showLogin(); return false;">&lt; back to login</a>
                </div>
                <div id="register-msg"></div>
            </div>
            <div id="api-key-box" style="display:none;">
                <div class="api-key-display">
                    <strong>API Key:</strong>
                    <pre id="api-key-value"></pre>
                    <button onclick="copyApiKey()">copy</button>
                    <button onclick="doRevoke()" style="margin-left: 0.5rem;">revoke & regenerate</button>
                    <button onclick="showChangePassword()" style="margin-left: 0.5rem;">change password</button>
                    <button onclick="doLogout()" style="margin-left: 0.5rem;">logout</button>
                    <div id="revoke-msg"></div>
                </div>
                <div id="change-password-form" style="display:none; margin-top: 1rem;">
                    <div class="auth-form">
                        <input type="password" id="old-password" placeholder="old password">
                        <input type="password" id="new-password" placeholder="new password (6+ chars)">
                        <button onclick="doChangePassword()">change</button>
                        <button onclick="hideChangePassword()" style="background: #fff; color: #333; border: 1px solid #333;">cancel</button>
                    </div>
                    <div id="change-password-msg"></div>
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
        let searchQuery = '';
        let searchLang = 'all';

        // Parse URL to determine view mode
        const pathParts = window.location.pathname.split('/');
        const profileUser = pathParts[1] === 'u' ? pathParts[2] : null;
        const singleSnippetId = pathParts[1] === 's' ? pathParts[2] : null;

        async function loadSingleSnippet() {
            if (!singleSnippetId) return;

            try {
                const response = await fetch(`/api/snippets/${singleSnippetId}`);
                if (response.ok) {
                    const snippet = await response.json();
                    document.getElementById('header-suffix').textContent = ` ~ snippet #${snippet.id}`;
                    document.getElementById('help-box').style.display = 'none';
                    document.getElementById('search-box').style.display = 'none';
                    document.getElementById('auth-box').style.display = 'none';
                    document.getElementById('pagination').style.display = 'none';

                    // Load star status for single snippet
                    const apiKey = localStorage.getItem('snip_api_key');
                    if (apiKey) {
                        try {
                            const starResp = await fetch(`/api/snippets/${snippet.id}/star`, {
                                headers: { 'X-API-Key': apiKey }
                            });
                            if (starResp.ok) {
                                const starData = await starResp.json();
                                snippet.starred = starData.starred;
                                snippet.stars = starData.total_stars;
                            }
                        } catch (e) {
                            // Ignore errors
                        }
                    }

                    renderSnippets([snippet]);
                } else if (response.status === 404) {
                    document.getElementById('snippets').innerHTML = '<div class="empty">Snippet not found</div>';
                } else {
                    document.getElementById('snippets').innerHTML = '<div class="empty">Error loading snippet</div>';
                }
            } catch (error) {
                document.getElementById('snippets').innerHTML = '<div class="empty">Error loading snippet</div>';
            }
        }

        async function loadSnippets(page = 1) {
            currentPage = page;
            let url;

            if (singleSnippetId) {
                // Single snippet view - handled separately
                return;
            } else if (searchQuery || (searchLang && searchLang !== 'all')) {
                // Use search endpoint
                const params = new URLSearchParams();
                params.append('page', page);
                params.append('limit', ITEMS_PER_PAGE);
                if (searchQuery) params.append('q', searchQuery);
                if (searchLang && searchLang !== 'all') params.append('lang', searchLang);
                url = `/api/search?${params.toString()}`;
            } else if (profileUser) {
                url = `/api/users/${profileUser}/snippets?page=${page}&limit=${ITEMS_PER_PAGE}`;
            } else {
                url = `/api/snippets?page=${page}&limit=${ITEMS_PER_PAGE}`;
            }

            try {
                const response = await fetch(url);
                const data = await response.json();

                totalPages = Math.ceil(data.total / ITEMS_PER_PAGE) || 1;

                // Load star status for each snippet if user is logged in
                const apiKey = localStorage.getItem('snip_api_key');
                if (apiKey && data.snippets) {
                    await Promise.all(data.snippets.map(async (s) => {
                        try {
                            const starResp = await fetch(`/api/snippets/${s.id}/star`, {
                                headers: { 'X-API-Key': apiKey }
                            });
                            if (starResp.ok) {
                                const starData = await starResp.json();
                                s.starred = starData.starred;
                                s.stars = starData.total_stars;
                            }
                        } catch (e) {
                            // Ignore errors
                        }
                    }));
                }

                renderSnippets(data.snippets);
                renderPagination();
            } catch (error) {
                document.getElementById('snippets').innerHTML =
                    '<div class="empty">Error loading snippets</div>';
            }
        }

        function doSearch() {
            searchQuery = document.getElementById('search-input').value.trim();
            searchLang = document.getElementById('search-lang').value;

            if (!searchQuery && searchLang === 'all') {
                clearSearch();
                return;
            }

            document.getElementById('clear-search').style.display = 'block';
            currentPage = 1;
            loadSnippets(1);
        }

        function clearSearch() {
            searchQuery = '';
            searchLang = 'all';
            document.getElementById('search-input').value = '';
            document.getElementById('search-lang').value = 'all';
            document.getElementById('clear-search').style.display = 'none';
            currentPage = 1;
            loadSnippets(1);
        }

        // Allow Enter key to trigger search
        document.getElementById('search-input').addEventListener('keypress', function(e) {
            if (e.key === 'Enter') doSearch();
        });

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
                const starCount = s.stars || 0;
                const starBtn = apiKey
                    ? ` <button class="star-btn ${s.starred ? 'starred' : ''}" onclick="toggleStar(${s.id}, this)" title="${s.starred ? 'Unstar' : 'Star'}">${s.starred ? '★' : '☆'} ${starCount}</button>`
                    : ` <span class="star-count">☆ ${starCount}</span>`;
                const langClass = s.language && s.language !== 'plaintext' ? ` class="language-${s.language}"` : '';
                const views = s.views || 0;
                return `
                <div class="snippet" id="snippet-${s.id}">
                    ${descHtml}
                    <div class="snippet-content">
                        <pre><code${langClass}>${escapeHtml(s.content)}</code></pre>
                    </div>
                    <div class="snippet-meta">${authorLink} · <a href="/s/${s.id}">${formatDate(s.created_at)}</a> · ${views} views${starBtn} · <a href="/raw/${s.id}" style="font-size: 0.75rem; color: #999;">raw</a>${deleteBtn}</div>
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

        async function toggleStar(id, btn) {
            const apiKey = localStorage.getItem('snip_api_key');
            if (!apiKey) {
                alert('Please login to star snippets');
                return;
            }

            const isStarred = btn.classList.contains('starred');
            const method = isStarred ? 'DELETE' : 'POST';

            try {
                const response = await fetch(`/api/snippets/${id}/star`, {
                    method: method,
                    headers: { 'X-API-Key': apiKey }
                });

                if (response.ok) {
                    const data = await response.json();
                    const count = data.total_stars || 0;
                    btn.textContent = data.starred ? `★ ${count}` : `☆ ${count}`;
                    btn.title = data.starred ? 'Unstar' : 'Star';
                    if (data.starred) {
                        btn.classList.add('starred');
                    } else {
                        btn.classList.remove('starred');
                    }
                } else if (response.status === 401) {
                    alert('Session expired. Please login again.');
                } else {
                    alert('Failed to update star');
                }
            } catch (e) {
                alert('Error starring snippet');
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

        function showRegister() {
            document.getElementById('login-form').style.display = 'none';
            document.getElementById('register-form').style.display = 'block';
            document.getElementById('login-msg').innerHTML = '';
        }

        function showLogin() {
            document.getElementById('register-form').style.display = 'none';
            document.getElementById('login-form').style.display = 'block';
            document.getElementById('register-msg').innerHTML = '';
        }

        async function doRegister() {
            const username = document.getElementById('register-user').value;
            const password = document.getElementById('register-pass').value;
            const msgDiv = document.getElementById('register-msg');

            if (!username || !password) {
                msgDiv.innerHTML = '<div class="error-msg">enter username and password</div>';
                return;
            }

            if (username.length < 3 || username.length > 32) {
                msgDiv.innerHTML = '<div class="error-msg">username must be 3-32 characters</div>';
                return;
            }

            if (password.length < 6) {
                msgDiv.innerHTML = '<div class="error-msg">password must be at least 6 characters</div>';
                return;
            }

            try {
                const response = await fetch('/api/register', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ username, password })
                });

                if (response.ok) {
                    const data = await response.json();
                    localStorage.setItem('snip_api_key', data.api_key);
                    localStorage.setItem('snip_username', data.username);
                    showApiKey(data.api_key);
                    msgDiv.innerHTML = '<div class="success-msg">registration successful</div>';

                    // Clear register form
                    document.getElementById('register-user').value = '';
                    document.getElementById('register-pass').value = '';
                } else if (response.status === 409) {
                    msgDiv.innerHTML = '<div class="error-msg">username already exists</div>';
                } else {
                    const error = await response.text();
                    msgDiv.innerHTML = `<div class="error-msg">${escapeHtml(error)}</div>`;
                }
            } catch (e) {
                msgDiv.innerHTML = '<div class="error-msg">registration failed</div>';
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
            document.getElementById('register-form').style.display = 'none';
            document.getElementById('api-key-box').style.display = 'none';
            document.getElementById('auth-box').style.display = 'none';
            document.getElementById('auth-toggle').textContent = '+auth';
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
            document.getElementById('change-password-form').style.display = 'block';
            document.getElementById('change-password-msg').innerHTML = '';
        }

        function hideChangePassword() {
            document.getElementById('change-password-form').style.display = 'none';
            document.getElementById('old-password').value = '';
            document.getElementById('new-password').value = '';
            document.getElementById('change-password-msg').innerHTML = '';
        }

        async function doChangePassword() {
            const apiKey = localStorage.getItem('snip_api_key');
            const oldPass = document.getElementById('old-password').value;
            const newPass = document.getElementById('new-password').value;
            const msgDiv = document.getElementById('change-password-msg');

            if (!oldPass || !newPass) {
                msgDiv.innerHTML = '<div class="error-msg">enter both old and new passwords</div>';
                return;
            }

            if (newPass.length < 6) {
                msgDiv.innerHTML = '<div class="error-msg">new password must be at least 6 characters</div>';
                return;
            }

            try {
                const response = await fetch('/api/change-password', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-API-Key': apiKey
                    },
                    body: JSON.stringify({ old_password: oldPass, new_password: newPass })
                });

                if (response.ok) {
                    msgDiv.innerHTML = '<div class="success-msg">password changed successfully</div>';
                    setTimeout(hideChangePassword, 2000);
                } else if (response.status === 401) {
                    msgDiv.innerHTML = '<div class="error-msg">invalid old password</div>';
                } else {
                    const error = await response.text();
                    msgDiv.innerHTML = `<div class="error-msg">${escapeHtml(error)}</div>`;
                }
            } catch (e) {
                msgDiv.innerHTML = '<div class="error-msg">failed to change password</div>';
            }
        }

        function toggleSearch() {
            const searchBox = document.getElementById('search-box');
            const toggleLink = document.getElementById('search-toggle');
            if (searchBox.style.display === 'none') {
                searchBox.style.display = 'block';
                toggleLink.textContent = '-search';
            } else {
                searchBox.style.display = 'none';
                toggleLink.textContent = '+search';
            }
        }

        function toggleAuth() {
            const authBox = document.getElementById('auth-box');
            const toggleLink = document.getElementById('auth-toggle');
            if (authBox.style.display === 'none') {
                authBox.style.display = 'block';
                toggleLink.textContent = '-auth';
            } else {
                authBox.style.display = 'none';
                toggleLink.textContent = '+auth';
            }
        }

        function showApiKey(key) {
            document.getElementById('login-form').style.display = 'none';
            document.getElementById('register-form').style.display = 'none';
            document.getElementById('api-key-box').style.display = 'block';
            document.getElementById('auth-box').style.display = 'block';
            document.getElementById('auth-toggle').textContent = '-auth';
            document.getElementById('api-key-value').textContent = key;
        }

        function copyApiKey() {
            const key = document.getElementById('api-key-value').textContent;
            navigator.clipboard.writeText(key).then(() => {
                const msgDiv = document.getElementById('revoke-msg');
                msgDiv.innerHTML = '<div class="success-msg">copied to clipboard</div>';
            });
        }

        // Check for existing session (stay logged in but auth box stays hidden)
        const savedKey = localStorage.getItem('snip_api_key');
        const savedUser = localStorage.getItem('snip_username');
        if (savedKey && savedUser) {
            // User is logged in, show API key in the hidden auth box
            document.getElementById('api-key-value').textContent = savedKey;
            document.getElementById('login-form').style.display = 'none';
            document.getElementById('register-form').style.display = 'none';
            document.getElementById('api-key-box').style.display = 'block';
        }

        // Load appropriate view
        if (singleSnippetId) {
            loadSingleSnippet();
        } else {
            loadSnippets();
        }

        // Theme toggle
        function initTheme() {
            const savedTheme = localStorage.getItem('snip_theme');
            if (savedTheme === 'dark') {
                document.documentElement.setAttribute('data-theme', 'dark');
                updateThemeIcon(true);
                updateHljsTheme(true);
            } else if (savedTheme === 'light') {
                document.documentElement.setAttribute('data-theme', 'light');
                updateThemeIcon(false);
                updateHljsTheme(false);
            } else {
                // Check system preference
                if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
                    document.documentElement.setAttribute('data-theme', 'dark');
                    updateThemeIcon(true);
                    updateHljsTheme(true);
                }
            }
        }

        function updateThemeIcon(isDark) {
            const toggle = document.querySelector('.theme-toggle');
            toggle.textContent = isDark ? '☀️' : '🌙';
            toggle.title = isDark ? 'Switch to light mode' : 'Switch to dark mode';
        }

        function updateHljsTheme(isDark) {
            const themeLink = document.getElementById('hljs-theme');
            const lightTheme = 'https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css';
            const darkTheme = 'https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css';
            themeLink.href = isDark ? darkTheme : lightTheme;
        }

        function toggleTheme() {
            const currentTheme = document.documentElement.getAttribute('data-theme');
            const isDark = currentTheme === 'dark';

            if (isDark) {
                document.documentElement.setAttribute('data-theme', 'light');
                localStorage.setItem('snip_theme', 'light');
                updateThemeIcon(false);
                updateHljsTheme(false);
            } else {
                document.documentElement.setAttribute('data-theme', 'dark');
                localStorage.setItem('snip_theme', 'dark');
                updateThemeIcon(true);
                updateHljsTheme(true);
            }
        }

        // Initialize theme on load
        initTheme();

        // Listen for system theme changes
        if (window.matchMedia) {
            window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
                const savedTheme = localStorage.getItem('snip_theme');
                if (!savedTheme) {
                    // Only auto-switch if user hasn't manually set theme
                    if (e.matches) {
                        document.documentElement.setAttribute('data-theme', 'dark');
                        updateThemeIcon(true);
                        updateHljsTheme(true);
                    } else {
                        document.documentElement.setAttribute('data-theme', 'light');
                        updateThemeIcon(false);
                        updateHljsTheme(false);
                    }
                }
            });
        }
    </script>
</body>
</html>
"##;
