## Context

The crate has one route (`POST /webhooks/github`), HMAC verification in `src/crypto.rs` with unit tests, and a spawned task in `src/web/handler/webhook.rs` that uses `reqwest::blocking`. A follow-up change replaces the blocking calls; this one only moves versions forward.

## Goals / Non-Goals

**Goals:**
- Every dependency and the builder image on its newest stable release, with the existing unit tests still passing.

**Non-Goals:**
- Replacing `reqwest::blocking` or `std::process::Command`; the follow-up change owns that, so the `blocking` and `multipart` reqwest features stay.
- Any refactor beyond what the new APIs require.

## Decisions

- **Resolve versions at implementation time.** Take each crate's newest stable release from crates.io (`cargo search` / `cargo update`) and the newest `rust:<x.y.z>-alpine` tag from Docker Hub, rather than fixing numbers here, so the change lands on what is current when it is applied. Pre-releases are excluded.
- **Pin the image to a full version tag** (`rust:<x.y.z>-alpine`) rather than a floating `1-alpine`, keeping builds reproducible as today's `1.90` pin does.
- **Keep version requirements at the same precision the manifest uses today** (major.minor, e.g. `"0.8"`), with `Cargo.lock` recording the exact resolution.
- **Drop the OpenSSL build packages.** reqwest 0.13 defaults to rustls (aws-lc-rs), so `openssl-sys` leaves the dependency tree and the builder stage installs only `musl-dev`.
- **Remove `sqlx` instead of upgrading it.** Its only reference is the `HttpError::Sqlx` variant and the matching log branch; no code constructs it.

## Risks / Trade-offs

- [reqwest 0.13 changes default TLS or feature names] → Select features so the blocking multipart upload to Discord still compiles and uses HTTPS; confirm with `cargo build` in both the local toolchain and the Docker builder.
- [hmac 0.13 / sha2 0.11 move to digest 0.11 with renamed APIs] → The existing `src/crypto.rs` tests, which use GitHub's published test vector, guard the signature check.
- [aws-lc-sys needs build tools the slimmer builder lacks] → Build the Docker image locally to confirm the release binary still builds with only `musl-dev`.
