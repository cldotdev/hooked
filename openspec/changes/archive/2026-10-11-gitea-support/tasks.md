## 1. Settings

- [x] 1.1 Rename `github_webhook_secret` to `webhook_secret` and `github_watch_push_branch` to `watch_push_branch` in `src/config.rs`, keeping `/run/secrets/` resolution for the secret, and update every reference
- [x] 1.2 Update the doc comments on both fields so they no longer mention GitHub only
- [x] 1.3 Add config tests: the new flag names parse, and `--github-webhook-secret` fails parsing

## 2. Routes

- [x] 2.1 Rename the handler to a provider-neutral name and register it for both `/webhooks/github` and `/webhooks/gitea` in `src/web/app.rs`
- [x] 2.2 Add sanitized GitHub and Gitea push payloads for the tests (example.com hosts and fictitious names)
- [x] 2.3 Add handler tests that drive the router: a signed Gitea push to `/webhooks/gitea` and a signed GitHub push to `/webhooks/github` return `push event accepted` and enqueue the job; a missing or wrong signature returns 401; another branch returns `unhandled branch`; a tag ref returns `unhandled ref`; a push without `head_commit` returns 400; `ping` returns `ping event ok`
- [x] 2.4 Return 401 for a malformed signature instead of a server error, with a test

## 3. Container Git Access

- [x] 3.1 Rewrite `entrypoint.sh` to default `GIT_HOST` to `github.com`, export `credential.helper=store` and the three `insteadOf` rules through `GIT_CONFIG_*` starting after any existing `GIT_CONFIG_COUNT`, write `~/.git-credentials` from `/run/secrets/git_token` with mode 600, and warn when the token file is missing
- [x] 3.2 Remove the `git config` lines from the `Dockerfile`, keeping `apk add git`
- [x] 3.3 Update `compose.yaml` to `WEBHOOK_SECRET`, `WATCH_PUSH_BRANCH`, a `GIT_HOST` entry, and the secrets `webhook_secret` and `git_token`
- [x] 3.4 Build the image and verify in a container with `GIT_HOST=git.example.com` that `git ls-remote --get-url git@git.example.com:o/r.git` prints `https://git.example.com/o/r.git`, that `git config --get credential.helper` prints `store`, that a fake token mounted at `/run/secrets/git_token` ends up in `~/.git-credentials` for `git.example.com`, that without the token file the container prints a warning and still runs its command, and that the default host is `github.com`; remove the image afterwards

## 4. Verification

- [x] 4.1 Run `cargo build --release`, `cargo test`, `cargo clippy --all-targets`, and `cargo fmt --check` with no warnings in touched code
