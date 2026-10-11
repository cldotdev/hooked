## 1. Dependencies

- [x] 1.1 Remove `sqlx` from `Cargo.toml`, the `HttpError::Sqlx` variant, and its arms in `src/web/object/base.rs`
- [x] 1.2 Bump every remaining dependency in `Cargo.toml` to its newest stable release at the existing major.minor precision, and regenerate `Cargo.lock`
- [x] 1.3 Adapt the source to breaking API changes until `cargo build --release` succeeds with no new warnings

## 2. Builder Image

- [x] 2.1 Change the `Dockerfile` builder stage to the newest stable `rust:<x.y.z>-alpine` tag
- [x] 2.2 Build the image with `docker build .` and confirm the build succeeds
- [x] 2.3 Remove the now-unused `openssl-dev openssl-libs-static pkgconfig` packages from the builder stage and rebuild the image

## 3. Verification

- [x] 3.1 Run `cargo test` and confirm the `src/crypto.rs` tests pass
- [x] 3.2 Run `cargo clippy --all-targets` and `cargo fmt --check` and fix any new findings in touched code
