# Snip

A minimal snippet sharing service with API, CLI, and web frontend.

## Quick Start

```bash
# Build
cargo build --release

# Run server
./target/release/snipped

# Open http://localhost:3000
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
echo "Hello World" | ./target/release/snip --desc "greeting" --lang javascript --api-key YOUR_API_KEY

# Or with curl
curl -X POST http://localhost:3000/api/snippets \
  -H "X-API-Key: YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"content": "Hello World", "description": "greeting", "language": "javascript"}'
```

**Limits:** Content max 5000 chars, Description max 255 chars.

**Languages:** `plaintext` (default), `bash`, `c`, `cpp`, `csharp`, `css`, `go`, `html`, `java`, `javascript`, `json`, `kotlin`, `lua`, `markdown`, `php`, `python`, `ruby`, `rust`, `scala`, `shell`, `sql`, `swift`, `typescript`, `yaml`, `zig`

### Delete Snippet

Delete your own snippets (requires API key):

```bash
curl -X DELETE http://localhost:3000/api/snippets/123 \
  -H "X-API-Key: YOUR_API_KEY"
```

On the web UI, a `[x]` button appears next to your own snippets when logged in.

### View Snippets

Open http://localhost:3000 or use the API:

```bash
curl http://localhost:3000/api/snippets?page=1&limit=10
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
| DELETE | `/api/snippets/{id}` | API Key | Delete your own snippet |
| GET | `/api/snippets` | - | List all snippets |
| GET | `/api/users/{username}/snippets` | - | List user snippets |
| GET | `/` | - | Web frontend |
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

# CLI server URL
SNIP_API_KEY=xxx ./target/release/snip --server http://localhost:3000
```
