## Context

Gitea sends `X-GitHub-Event` and `X-Hub-Signature-256` (with the `sha256=` prefix) alongside its own headers, and its push payload carries the fields the handler reads (`ref`, `after`, `repository.full_name`, `head_commit.url`, `head_commit.committer.name`). The handler can therefore serve Gitea unchanged. Git access in the image is split today: the `Dockerfile` bakes three `insteadOf` rules and `credential.helper store` for `github.com` into `/root/.gitconfig`, and `entrypoint.sh` writes `~/.git-credentials` from `/run/secrets/github_pat`.

## Goals / Non-Goals

**Goals:**
- One handler serving both routes, with names that no longer mention GitHub.
- Git host chosen at container start, so the published image works for any host.

**Non-Goals:**
- Reading Gitea-only headers (`X-Gitea-Event`, `X-Gitea-Signature`); Gitea already sends the GitHub-compatible ones.
- Several git hosts in one container.
- Gitea's form-encoded webhook content type; deliveries must use `application/json`, as for GitHub.
- Running as a non-root user (the next change).

## Decisions

- **Two routes, one handler.** Register `/webhooks/github` and `/webhooks/gitea` on the same handler function, renamed to something provider-neutral. A provider parameter would carry no information, because both providers send identical headers.
- **A malformed signature is a mismatch.** A missing `sha256=` prefix or invalid hex used to surface as a 500 through `HttpError::Anyhow`; it now returns 401 like any other failed verification.
- **Clean rename with no aliases.** `webhook_secret` / `watch_push_branch` replace the `github_` fields; clap derives `--webhook-secret`, `WEBHOOK_SECRET`, `--watch-push-branch`, and `WATCH_PUSH_BRANCH` from them. The old names are dropped, so an old flag fails parsing. An old environment variable is ignored, and hooked then fails because the required argument is missing.
- **Git configuration through `GIT_CONFIG_COUNT` / `GIT_CONFIG_KEY_<n>` / `GIT_CONFIG_VALUE_<n>`.** The entrypoint exports `credential.helper=store` and the three `url.https://$GIT_HOST/.insteadOf` rules, then `exec`s hooked, so the build script and every git it runs inherit them. Nothing is written to a git config file, so restarts cannot accumulate duplicate rules, and no write access to `/etc` or `$HOME/.gitconfig` is needed. Indexes start after any `GIT_CONFIG_COUNT` the caller already set, so user-supplied entries survive. Alternatives: `git config --global --add` in the entrypoint duplicates rules on every restart of the same container; a build-time `ARG` cannot be changed by users of the published image.
- **The `Dockerfile` keeps only `apk add git`.** Its `github.com` rules would duplicate the entrypoint's when `GIT_HOST` is the default.
- **Token file `/run/secrets/git_token` only.** The entrypoint writes `https://token:<token>@$GIT_HOST` to `~/.git-credentials` with mode 600, as today; no fallback to `github_pat`.
- **Router tests build payloads inline.** The GitHub- and Gitea-shaped push payloads are `json!` values in the handler tests rather than separate fixture files, and tests reach the router through `new_router()` (now `pub(crate)`) with tower's `util` feature enabled as a dev-dependency.
- **`compose.yaml` uses the new names throughout**: `WEBHOOK_SECRET`, `WATCH_PUSH_BRANCH`, and the secrets `webhook_secret` and `git_token`, plus a `GIT_HOST` entry showing the default.

## Risks / Trade-offs

- [Existing deployments break on upgrade] → The README Migration section lists every renamed flag, variable, and secret file.
- [`GIT_CONFIG_*` applies only to processes started from the entrypoint; a `docker exec` shell does not see it] → Acceptable, because builds always run under hooked. The README says so.
- [Gitea's payload format could drift from GitHub's] → A test feeds a Gitea-shaped push payload through the `/webhooks/gitea` route.

## Migration Plan

Deploy by renaming the flags or environment variables and the secret files as the README Migration section lists, then pulling the new image. Roll back by restoring the previous image tag together with the old names.
