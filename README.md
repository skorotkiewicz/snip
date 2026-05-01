# Snip

A minimal snippet sharing service with API, CLI, and web frontend.

## Quick Start

```bash
# Build
cargo build --release

# Run server
./target/release/snip

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

### Post Snippet

```bash
# Using CLI
echo "Hello World" | ./target/release/snipped --desc "greeting" --api-key YOUR_API_KEY

# Or with curl
curl -X POST http://localhost:3000/api/snippets \
  -H "X-API-Key: YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"content": "Hello World", "description": "greeting"}'
```

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
| POST | `/api/snippets` | API Key | Create snippet |
| GET | `/api/snippets` | - | List all snippets |
| GET | `/api/users/{username}/snippets` | - | List user snippets |
| GET | `/` | - | Web frontend |
| GET | `/u/{username}` | - | User profile page |

## Architecture

- **Backend**: Axum (Rust) + SQLite
- **Frontend**: Vanilla HTML/JS (embedded in binary)
- **CLI**: Separate binary `snipped`

Data persists in `snip.db` (SQLite).

## Environment

```bash
# Custom database path
DATABASE_URL=sqlite:/path/to/snip.db ./target/release/snip

# CLI server URL
SNIP_API_KEY=xxx ./target/release/snipped --server http://localhost:3000
```
