## Why

The build image (`rust:1.90-alpine`) and most crates are several releases behind, and the next changes (async build execution, Gitea support) would otherwise be written against APIs that are already superseded. `sqlx` is compiled into every build although nothing uses a database.

## What Changes

- Bump the Rust builder image in `Dockerfile` to the newest stable `rust:<version>-alpine` tag.
- Bump every dependency in `Cargo.toml` to its newest stable release and regenerate `Cargo.lock`, adapting the code to breaking API changes (notably axum 0.8, tower 0.5, tower-http 0.7, thiserror 2, reqwest 0.13, hmac 0.13 / sha2 0.11).
- Remove the OpenSSL build packages from the builder stage; reqwest's switch to rustls leaves them unused.
- Remove the `sqlx` dependency and the `HttpError::Sqlx` variant that is its only use.
- No change to HTTP routes, CLI arguments, environment variables, webhook handling, or Discord notifications.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This change does not alter observable behavior, so it sets `skip_specs: true`.

## Impact

- `Cargo.toml`, `Cargo.lock`, `Dockerfile` (builder image tag and apk packages)
- `src/web/object/base.rs` (drop the `Sqlx` variant) and `src/crypto.rs` (hmac 0.13 API change)
- Smaller dependency tree after dropping `sqlx` and OpenSSL
