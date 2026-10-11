## Why

Each accepted push spawns a task that runs the build script with `std::process::Command` and posts to Discord with `reqwest::blocking`, so a long build holds a tokio worker thread for its whole duration. Pushes that arrive close together start builds that run concurrently against the same checkout, and an outdated build can finish last. Discord is also mandatory, so hooked cannot run without a Discord webhook.

## What Changes

- Run the build script and the Discord upload asynchronously, without blocking the runtime.
- Run builds one at a time through a single worker. While a build runs, keep only the newest accepted push; when the build finishes, run that one and drop any older pending pushes. **BREAKING** (behavior): pushes that are superseded while queued no longer trigger a build.
- Make `--discord-webhook-url` / `DISCORD_WEBHOOK_URL` optional. When it is unset or empty, hooked logs each build's exit status and sends nothing.
- Replace panics in the build task with logged errors, so a failing script launch or Discord request does not kill the worker.

## Capabilities

### New Capabilities

- `build-execution`: How accepted pushes turn into build script runs: ordering, coalescing, and the arguments passed to the script.
- `build-notification`: How the outcome of each build is reported, through Discord when configured and through the log otherwise.

### Modified Capabilities

None.

## Impact

- `src/web/handler/webhook.rs`, `src/web/app.rs`, `src/config.rs`, `src/main.rs`, plus a new worker module (`src/web/worker.rs`)
- `Cargo.toml`: drop reqwest's `blocking` feature and the `tempfile` dependency
- README: `--discord-webhook-url` becomes optional; the build ordering changes
