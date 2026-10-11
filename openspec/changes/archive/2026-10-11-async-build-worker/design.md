## Context

`post_webhooks_github` validates the push and then `tokio::spawn`s a task that runs the script with `std::process::Command::output()`, writes stdout/stderr into temp files, and uploads them with `reqwest::blocking`. Every step `expect`s, so any failure panics the task. `Config.discord_webhook_url` is a required `String`. The Docker deployment passes `DISCORD_WEBHOOK_URL` through compose, where an unset variable can arrive as an empty string.

## Goals / Non-Goals

**Goals:**
- No blocking calls on the tokio runtime.
- One worker that owns build execution, with latest-wins coalescing as specified in `specs/build-execution`.

**Non-Goals:**
- Persisting pending pushes across restarts; a pending push is lost if hooked stops.
- Cancelling a running build when a newer push arrives; the running build always finishes.
- Graceful shutdown that waits for an in-flight build.
- Changing webhook validation, routes, or response bodies.

## Decisions

- **Coalesce with `tokio::sync::watch<Option<BuildJob>>`.** The handler sends the job into the watch channel; a single worker task spawned at startup loops on `changed().await` and takes the current value. A watch channel stores only the latest value, so pushes sent while a build runs overwrite each other and the worker sees just the newest one when it next checks. Alternatives: a bounded `mpsc` that the worker drains to its last item adds draining code for the same result; a `Mutex<Option<_>>` plus `Notify` reimplements what `watch` already provides.
- **The worker owns the receiver; `Context` holds the sender** (behind an `Arc`), so the handler stays a thin validator that enqueues and returns.
- **Run the script with `tokio::process::Command::output().await`.** A spawn error becomes a failed build whose stderr text is the error message, so both the log and Discord report it like any other failure.
- **Upload output from memory.** Build the multipart form with `reqwest::multipart::Part::bytes(..).file_name("stdout.txt")` (and `stderr.txt`) using the async client. This removes the temp files, the `tempfile` dependency, and the need for valid UTF-8, and lets the `blocking` feature go.
- **`discord_webhook_url: Option<String>`, normalized after parsing.** clap passes an empty flag or env value through as `Some("")`, so `Config::load()` parses and then maps an empty URL to `None`; `DISCORD_WEBHOOK_URL=` therefore disables Discord. `parse_secret_or_env` still resolves `/run/secrets/...` paths, and delegates to `parse_secret_in(value, dir)` so tests can point it at a temporary directory.
- **Short commit ID by characters.** The embed takes the first seven characters of the commit ID instead of slicing seven bytes, so a short or non-ASCII ID no longer panics.
- **Treat a Discord HTTP error status as a failure.** The response goes through `error_for_status()`, so a rejected upload is logged like a network error.
- **Log level by outcome.** `info!` for exit status 0, `warn!` for non-zero or spawn failure, `error!` for a failed Discord request. Each line carries the commit ID and exit status, never the script output. The handler also logs each accepted push with its commit ID at `info`.

## Risks / Trade-offs

- [Superseded pushes are skipped silently] → The log records each accepted push with its commit ID, so skipped commits can be found by comparing accepted and built IDs.
- [A hung build script blocks all later builds] → Accepted; adding a timeout would be a new decision. The log shows which commit is running.
- [Tests write a script and execute it right away, which can hit `ETXTBSY` if another test thread forks while the file is open] → Not observed so far; the tests do not retry.
- [Large script output held in memory] → Same as today, where `output()` already buffers it; build logs are small compared with the host's memory.
