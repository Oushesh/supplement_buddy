# Rust Backend (Axum + Tokio + SQLx)

A high-performance REST API for supplement search built with:

- **[Axum](https://github.com/tokio-rs/axum)** — ergonomic async web framework built on Tower/Hyper
- **[Tokio](https://tokio.rs)** — async runtime
- **[SQLx](https://github.com/launchbadge/sqlx)** — async SQLite driver with compile-time query checking
- **[Serde](https://serde.rs)** — JSON serialization

This backend mirrors the Django REST API surface and can be used as a drop-in performance alternative for high-throughput workloads.

## API Endpoints

| Method | URL | Description |
|--------|-----|-------------|
| GET | `/api/supplements` | List all supplements (paginated) |
| POST | `/api/supplements` | Create a supplement |
| GET | `/api/supplements/:id` | Get a supplement by ID |
| GET | `/api/supplements/search?q=<query>` | Text search (SPLADE fallback) |
| PUT | `/api/supplements/:id/splade-vector` | Update a supplement's SPLADE vector |
| GET | `/api/search-queries` | List past search queries |

### Pagination

`GET /api/supplements?page=2&page_size=10`

```json
{
  "count": 42,
  "page": 2,
  "page_size": 10,
  "results": [...]
}
```

## Quick Start

```bash
# 1. Copy and configure environment
cp .env.example .env

# 2. Run the server
cargo run

# Server starts on http://localhost:8001
```

## Running Tests

```bash
cargo test
```

All 12 integration tests run against an in-memory SQLite database — no external services required.

```
running 12 tests
test test_create_supplement_missing_name_returns_400 ... ok
test test_create_supplement_success              ... ok
test test_get_supplement_by_id                   ... ok
test test_get_supplement_not_found               ... ok
test test_list_search_queries_empty              ... ok
test test_list_supplements_empty                 ... ok
test test_list_supplements_returns_all           ... ok
test test_search_missing_query_returns_400       ... ok
test test_search_persists_query                  ... ok
test test_search_returns_matching_results        ... ok
test test_update_splade_vector                   ... ok
test test_update_splade_vector_not_found         ... ok

test result: ok. 12 passed; 0 failed; 0 ignored
```

## Docker

```bash
docker build -t supplement-buddy-rust .
docker run -p 8001:8001 supplement-buddy-rust
```

## Architecture Notes

- **Database**: SQLite by default (set `DATABASE_URL=sqlite:///path/to/db.sqlite3`). No migration tooling needed — migrations run inline at startup.
- **Search**: Text-based LIKE search fallback. SPLADE inference is performed by the Python fine-tuning scripts; the resulting sparse vectors are stored per-supplement via the `PUT /api/supplements/:id/splade-vector` endpoint.
- **CORS**: Configurable via `CORS_ALLOWED_ORIGINS`. If unset, all origins are allowed (development mode).
- **Logging**: Controlled via `RUST_LOG` env var (uses `tracing`/`tracing-subscriber`).

## Switching from Django

This backend exposes the same REST surface. To switch:
1. Point `NEXT_PUBLIC_API_URL` in the frontend to `http://localhost:8001`.
2. Re-run the `index_supplements.py` script pointing at the Rust backend's `PUT /api/supplements/:id/splade-vector` endpoint to repopulate SPLADE vectors.
