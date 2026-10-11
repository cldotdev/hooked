## 1. Lint Baseline

- [x] 1.1 Replace the `assert_eq!(result, true)` / `assert_eq!(result, false)` assertions in `src/crypto.rs` with `assert!(result)` / `assert!(!result)`, and confirm `cargo clippy --all-targets -- -D warnings` passes

## 2. Workflows

- [x] 2.1 Add `.github/workflows/ci.yml` for pushes to `master` and pull requests, reading the Rust version from the `Dockerfile`'s `FROM rust:<x.y.z>-alpine` line, installing it with `rustup` (clippy and rustfmt components), caching with `Swatinem/rust-cache@v2`, and running `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`
- [x] 2.2 Add `.github/workflows/release.yml` modeled on `../feedgen/.github/workflows/release.yml`: trigger on `[0-9]+.[0-9]+.[0-9]+` tags, fail when the tag differs from the `Cargo.toml` version, set up Buildx, build and push `linux/amd64` to `ghcr.io/${{ github.repository }}` with the semver tag and `latest` and the GitHub Actions cache, then create the release from the tag annotation
- [x] 2.3 Parse both workflow files as YAML locally, and run the version-check and Rust-version extraction shell snippets against the current `Cargo.toml` and `Dockerfile` to confirm they print `0.3.0` and `1.99.0`

## 3. Image Location

- [x] 3.1 Change `compose.yaml` to `ghcr.io/cldotdev/hooked:latest`
