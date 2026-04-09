# Agent Guidance

## Commands
- Start server: `cargo run -- --port 10086` (default port 10086, configurable via CLI)
- Binary name: `brain-overflow` (built with `cargo build --release`)
- Override config: `--config-path ./brain-overflow.toml`

## Testing
- Run all tests: `cargo test`
- Test auth crate only: `cargo test -p auth`
- Integration test for auth: `crates/auth/tests/integration_test.rs`

## Linting & Formatting
- Format code: `cargo fmt`
- Lint: `cargo clippy`
- Existing clippy warnings may need to be addressed before committing

## Configuration
- Default config file: `brain-overflow.toml` in workspace root
- CLI options override config values (e.g., `--port`)
- Config structure: see `src/app_config.rs` modules
- `.env` file may be used for local development (not automatically loaded)

## Database
- Requires PostgreSQL with credentials matching config
- Database URL format: `postgres://user:password@host:port/database`
- Migrations: none currently; schema likely managed separately
- Ensure PostgreSQL is running before starting server

## S3 Storage
- Configure S3-compatible storage (AWS S3, MinIO, LocalStack) via `[s3]` section in `brain-overflow.toml`
- Required fields: `bucket`, `region`
- Optional: `access_key_id`, `secret_access_key`, `endpoint`, `force_path_style`, `cdn_domain`, `key_prefix`
- For MinIO/local development, set `endpoint = "http://localhost:9000"` and `force_path_style = true`
- Asset uploads use pre-signed URLs (PUT) with 15-minute expiry
- Asset public URLs are generated based on CDN, custom endpoint, or AWS S3 format

## Workspace Structure
- Root crate: binary `brain-overflow` (main entrypoint `src/main.rs`)
- Library crate: `auth` (`crates/auth/`) – JWT encoding/decoding utilities
- Dependencies: workspace-wide in root `Cargo.toml`

## Notes
- The auth crate is a standalone JWT library with its own tests
- Server uses Axum with tower-http middleware (CORS, timeout, tracing)
- Request IDs are base64-encoded UUID v7
- Configuration uses `config` crate with TOML files
