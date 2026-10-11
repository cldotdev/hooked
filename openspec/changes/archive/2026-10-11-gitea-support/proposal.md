## Why

hooked only advertises GitHub, while self-hosted Gitea sends push webhooks in a GitHub-compatible format (`X-GitHub-Event`, `X-Hub-Signature-256`, same payload fields). The remaining obstacles are the GitHub-specific names in the CLI, the single `/webhooks/github` route, and container git access that is hardcoded to `github.com`.

## What Changes

- Add `POST /webhooks/gitea`, handled by the same code as `POST /webhooks/github`.
- **BREAKING**: Rename `--github-webhook-secret` / `GITHUB_WEBHOOK_SECRET` to `--webhook-secret` / `WEBHOOK_SECRET`, and `--github-watch-push-branch` / `GITHUB_WATCH_PUSH_BRANCH` to `--watch-push-branch` / `WATCH_PUSH_BRANCH`. The old names are removed, not aliased.
- Container git access: the entrypoint configures the credential helper and SSH-to-HTTPS URL rewriting for the host in `GIT_HOST` (default `github.com`) through `GIT_CONFIG_*` environment variables, replacing the `github.com` rules baked into the `Dockerfile`.
- **BREAKING**: The git token secret moves from `/run/secrets/github_pat` to `/run/secrets/git_token`, with no fallback.
- `compose.yaml` switches to the new environment variable and secret names (`webhook_secret`, `git_token`).
- README gains a Migration section listing the renames.

## Capabilities

### New Capabilities

- `webhook-receiver`: Which webhook endpoints hooked serves, how requests are authenticated and filtered, and the configuration that controls them.
- `container-git-access`: How the Docker image gives build scripts authenticated git access to the configured host.

### Modified Capabilities

None.

## Impact

- `src/config.rs`, `src/web/app.rs`, `src/web/handler/webhook.rs`
- `entrypoint.sh`, `Dockerfile`, `compose.yaml`
- `Cargo.toml`: tower's `util` feature as a dev-dependency for router tests
- README: usage, Gitea setup, a Docker subsection for `GIT_HOST` and `git_token`, and a new Migration section
- Existing deployments must rename their flags or environment variables and the `github_pat` secret before upgrading
