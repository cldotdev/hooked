## 1. Configuration

- [x] 1.1 Make `discord_webhook_url` an `Option<String>` in `src/config.rs`, mapping an empty value to `None` while keeping `/run/secrets/...` resolution
- [x] 1.2 Add unit tests for the flag unset, the env var empty, a literal URL, and a `/run/secrets/`-style path read from a temp file

## 2. Build Worker

- [x] 2.1 Add a worker module that owns a `tokio::sync::watch` receiver of `Option<BuildJob>` and runs builds one at a time with `tokio::process::Command`
- [x] 2.2 Spawn the worker at startup in `src/web/app.rs` and put the sender in `Context`
- [x] 2.3 Change `post_webhooks_github` to send a `BuildJob` and return `push event accepted` without spawning its own task
- [x] 2.4 Treat a script spawn error as a failed build with the error text as stderr, and replace every `expect` in the build path with logged errors

## 3. Notification

- [x] 3.1 Send the Discord embed with stdout/stderr attached from memory using the async `reqwest` client, only when a URL is configured
- [x] 3.2 Log each build's commit ID and exit status at `info`/`warn`, and log Discord request failures at `error`
- [x] 3.3 Remove reqwest's `blocking` feature, and remove `tempfile` from `Cargo.toml` if nothing else uses it

## 4. Tests

- [x] 4.1 Test coalescing: with a script that records its argument and sleeps, enqueue a job, then three more while it runs, and assert that exactly the first and the last commit IDs were built, in that order
- [x] 4.2 Test that a missing script path is reported as a failed build and the next job still runs
- [x] 4.3 Test the embed payload for success and failure (title, color, fields, short commit ID)
- [x] 4.4 Test that non-UTF-8 script output is accepted
- [x] 4.5 Test that a failing Discord request is followed by the next build
- [x] 4.6 Run `cargo build --release`, `cargo test`, `cargo clippy --all-targets`, and `cargo fmt --check` with no new warnings in touched code
