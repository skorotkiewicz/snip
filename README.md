# Snip

A minimal snippet sharing service with API, CLI, and web frontend.

## Quick Start

### Local Build

```bash
# Build
cargo build --release

# Run server
DATABASE_URL="sqlite:./snip.db" ./target/release/snipped

# Open http://localhost:3000
```

### Docker

```bash
# Build and run with Docker Compose
docker-compose up -d

# Or build manually
docker build -t snip .
docker run -d -p 3000:3000 -v snip_data:/data snip
# docker run -d -p 3000:3000 -v ./data:/data snip
```

### systemd

```bash
# Copy binary and service file
sudo cp target/release/snipped /usr/bin/
sudo cp systemd/snip.service /etc/systemd/system/
sudo cp systemd/snip.tmpfiles /usr/lib/tmpfiles.d/snip.conf

# Create data directory
sudo systemd-tmpfiles --create /usr/lib/tmpfiles.d/snip.conf

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable --now snip

# Check status
sudo systemctl status snip
```

## Usage

### Register

```bash
curl -X POST http://localhost:3000/api/register \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "secret123"}'
```

Response:
```json
{"id": 1, "username": "alice", "api_key": "..."}
```

### Login

```bash
curl -X POST http://localhost:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "secret123"}'
```

Response:
```json
{"username": "alice", "api_key": "..."}
```

### Revoke API Key

Invalidate the current API key and generate a new one:

```bash
curl -X POST http://localhost:3000/api/revoke-key \
  -H "X-API-Key: YOUR_OLD_API_KEY"
```

Response:
```json
{"username": "alice", "old_api_key": "...", "new_api_key": "..."}
```

### Post Snippet

```bash
# Using CLI
echo "Hello World" | ./target/release/snip --desc "greeting" --lang javascript

# Or pipe from file
cat file.rs | ./target/release/snip --desc "my rust code" --lang rust

# Or with curl
curl -X POST http://localhost:3000/api/snippets \
  -H "X-API-Key: YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"content": "Hello World", "description": "greeting", "language": "javascript"}'
```

**Limits:** Content max 5000 chars, Description max 255 chars.

**Languages:** `plaintext` (default), `bash`, `c`, `cpp`, `csharp`, `css`, `go`, `html`, `java`, `javascript`, `json`, `kotlin`, `lua`, `markdown`, `php`, `python`, `ruby`, `rust`, `scala`, `shell`, `sql`, `swift`, `typescript`, `yaml`, `zig`

### Get Snippet

```bash
# Get snippet by ID
./target/release/snip get 123

# Output includes metadata and full content
```

### Login / Register (CLI)

The CLI can save credentials to `~/.config/snip/config.json`:

```bash
# Login
./target/release/snip login myusername
# (will prompt for password)

# Register new account
./target/release/snip register newusername
# (will prompt for password)

# Check who you are
./target/release/snip whoami

# Logout (clear credentials)
./target/release/snip logout
```

### Search Snippets

```bash
# Search all snippets
./target/release/snip search "function"

# Search with language filter
./target/release/snip search "main" --lang rust

# Limit results
./target/release/snip search "hello" --lang python -n 5
```

### Delete Snippet

Delete your own snippets (requires API key):

**CLI:**
```bash
# Delete by ID (requires login or --api-key)
./target/release/snip delete 123
```

**curl:**
```bash
curl -X DELETE http://localhost:3000/api/snippets/123 \
  -H "X-API-Key: YOUR_API_KEY"
```

On the web UI, a `[x]` button appears next to your own snippets when logged in.

### Search Snippets

Search in content and description, filter by language:

```bash
# Search by keyword
curl "http://localhost:3000/api/search?q=hello&page=1&limit=10"

# Filter by language
curl "http://localhost:3000/api/search?lang=rust&page=1&limit=10"

# Combined search
curl "http://localhost:3000/api/search?q=function&lang=javascript&page=1&limit=10"
```

Web UI has a search box with language filter dropdown.

### View Single Snippet

View a specific snippet by ID:

```
http://localhost:3000/s/123
```

API:
```bash
curl http://localhost:3000/api/snippets/123
```

Response:
```json
{"id": 123, "content": "...", "description": "...", "language": "rust", "created_at": "...", "author": "alice"}
```

### Raw Snippet Content

Get raw snippet content (useful for piping or embedding):

```bash
curl http://localhost:3000/raw/123
```

Returns the raw content as `text/plain`:
```
console.log("hello world");
```

### User Profiles

Click any username to view their profile, or visit directly:

```
http://localhost:3000/u/alice
```

API:
```bash
curl http://localhost:3000/api/users/alice/snippets?page=1&limit=10
```

## API

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| POST | `/api/register` | - | Create account, returns API key |
| POST | `/api/login` | - | Login with password, returns API key |
| POST | `/api/revoke-key` | API Key | Revoke old key, generate new API key |
| POST | `/api/snippets` | API Key | Create snippet |
| GET | `/api/snippets/{id}` | - | Get single snippet by ID |
| DELETE | `/api/snippets/{id}` | API Key | Delete your own snippet |
| GET | `/api/snippets` | - | List all snippets |
| GET | `/api/search` | - | Search snippets by content/description/language |
| GET | `/api/users/{username}/snippets` | - | List user snippets |
| GET | `/` | - | Web frontend |
| GET | `/s/{id}` | - | View single snippet page |
| GET | `/raw/{id}` | - | Get raw snippet content (plain text) |
| GET | `/u/{username}` | - | User profile page |

## Architecture

- **Backend**: Axum (Rust) + SQLite
- **Frontend**: Vanilla HTML/JS (embedded in binary)
- **CLI**: Separate binary `snip`

Data persists in `snip.db` (SQLite).

## Environment

```bash
# Custom database path
DATABASE_URL=sqlite:/path/to/snip.db ./target/release/snipped

# CLI environment variables
SNIP_API_KEY=xxx SNIP_URL_SERVER=http://localhost:3000 ./target/release/snip

# Or with flags
SNIP_API_KEY=xxx ./target/release/snip --server http://localhost:3000
```
