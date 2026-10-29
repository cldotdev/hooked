## Why

hooked builds only when a push webhook arrives, so there is no supported way to rebuild a commit after a failed deploy or a host-side change. Redelivering the webhook needs repository admin rights, and running the build script with `docker exec` bypasses the worker: it can run at the same time as a webhook-triggered build, which the example scripts assume never happens (they replace a fixed source directory).

## What Changes

- New `hooked trigger <commit>` subcommand. It signs a push event for the watched branch and the given commit with the webhook secret and sends it to the running server's `/webhooks/github`, so the build goes through the same queue, one-at-a-time rule, and notifications as a push. `hooked` without a subcommand starts the server as before.
- The push handler no longer requires `head_commit.url`. When it is absent or empty, the Discord embed has no link.
- README: how to rebuild a commit with `docker exec <container> hooked trigger <commit>`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `webhook-receiver`: `head_commit.url` is no longer a required push field.
- `build-notification`: the Discord embed links to the head commit only when the push carries its URL.
- `build-execution`: adds the manual trigger, which enqueues a build through the running server.

## Impact

- `src/config.rs` (subcommand), `src/main.rs` (dispatch), a new client module for the trigger, `src/web/handler/webhook.rs` and `src/web/worker.rs` (optional URL), README.
- No new dependencies: the trigger uses `reqwest`, `hmac`, and `sha2`, which the server already uses.
