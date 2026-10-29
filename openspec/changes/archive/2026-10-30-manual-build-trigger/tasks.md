## 1. Server

- [x] 1.1 Make the job's head commit URL optional: the push handler reads `head_commit.url` without requiring it (absent or empty becomes no URL), and the Discord embed omits `url` when there is none
- [x] 1.2 Add handler tests: a push without `head_commit.url` is accepted and enqueued with no URL; a push without `head_commit` still answers 400 naming `head_commit.committer.name`
- [x] 1.3 Add an embed test: a job without a URL produces an embed with no `url` key

## 2. Trigger

- [x] 2.1 Add an optional `trigger <commit>` subcommand to `Config`, keeping every existing setting top-level and unchanged; `hooked` without a subcommand still starts the server; make the top-level help flag global so `hooked trigger --help` works
- [x] 2.2 Implement the trigger in its own module: validate the commit as non-empty lowercase hex, build the payload described in design.md, sign it with the webhook secret in the `X-Hub-Signature-256` format, and POST it with `X-GitHub-Event: push` to `/webhooks/github` on the configured host and port, mapping an unspecified address to loopback and bracketing IPv6
- [x] 2.3 In `main`, dispatch on the subcommand: the server path is unchanged (logger, then server); the trigger path sets up no file logger, prints the server's message or the full error chain, and exits 0 only on 200 with `push event accepted`
- [x] 2.4 Add tests: config parsing with and without the subcommand; commit validation; target URL for `0.0.0.0`, `::`, `127.0.0.1`, and a hostname; a trigger against the router served on an ephemeral port is accepted and enqueues a job with the given commit, the watched branch, and no URL; a trigger against a closed port returns an error

- [x] 2.5 Add `crypto::sign` for the `X-Hub-Signature-256` value and use it in the trigger and the webhook test helper

## 3. Documentation

- [x] 3.1 In README Usage, add how to rebuild a commit: `docker exec <container> hooked trigger <commit>` (and `hooked trigger <commit>` with the server's settings outside Docker), that it goes through the same queue and notifications as a push, and that the example scripts can build only commits on the watched branch; document the subcommand next to the other arguments

## 4. Checks

- [x] 4.1 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`
