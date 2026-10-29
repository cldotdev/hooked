## Context

See proposal.md - Why. The server accepts builds only through `POST /webhooks/{github,gitea}`, verified with HMAC-SHA256 over the body, and hands them to a single worker through a watch channel that keeps only the newest pending job. In the container, `WEBHOOK_SECRET`, `WATCH_PUSH_BRANCH`, `HOOKED_HOST`, `HOOKED_PORT`, and `BUILD_ENTRY_SCRIPT_PATH` are set on the service, so a process started with `docker exec` sees them; it does not go through the entrypoint, so it lacks the git setup, but the trigger does not need git.

## Goals / Non-Goals

**Goals:**
- A manual rebuild that obeys the one-at-a-time rule and sends the same notifications as a push.
- No new HTTP surface and no new dependencies.

**Non-Goals:**
- Resolving a branch name to a commit; hooked has no git access of its own, so the caller passes a commit ID.
- Triggering a branch other than the watched one.
- A remote trigger from outside the host; the trigger is meant for `docker exec` on the same host as the server.

## Decisions

- **Reuse the webhook endpoint instead of a new endpoint.** The trigger is a client of the existing `/webhooks/github` route: the signature check is the authentication, and the handler already filters the branch and enqueues the job. A dedicated endpoint would need its own authentication and tests for the same outcome. `/webhooks/github` is chosen over `/webhooks/gitea` only because both behave the same; the event header is `X-GitHub-Event: push`.
- **Synthetic payload.** `ref` is `refs/heads/<watched branch>`, `after` is the commit, `repository.full_name` is `-`, and `head_commit.committer.name` is `manual trigger`, so the Discord embed shows that a person triggered the build. `head_commit.url` is omitted, because hooked cannot build a correct commit URL: it does not know the repository, and `GIT_URL` is exported only by the entrypoint. Placeholder URLs were rejected because Discord can refuse an embed with an invalid URL, which would drop the notification.
- **`head_commit.url` becomes optional on the server.** The job carries the URL as an optional value and the embed omits `url` when it is absent. GitHub and Gitea always send it, so their deliveries are unaffected.
- **Commit validation in the trigger.** The trigger accepts only lowercase hex, so a typo such as a branch name fails locally instead of reaching the build script, which receives the value as its first argument.
- **Subcommand on the existing parser.** `Config` gains an optional subcommand, `trigger <commit>`. The existing settings stay top-level, so the trigger reads exactly what the server reads, from the same environment variables. They keep their current required status: inside the container the environment satisfies them, and making them optional only for the trigger would weaken the server's argument checking. The top-level `-?`/`--help` flag is global, because `Config` disables clap's own help flag (`-h` is `--host`) and that setting also removes it from the subcommand.
- **One signing function.** `crypto::sign` returns the `X-Hub-Signature-256` value; the trigger and the webhook tests use it, next to the existing `verify_signature`.
- **Target address.** The trigger connects to the configured host and port, replacing an unspecified address with loopback of the same family. IPv6 hosts are bracketed in the URL.
- **No file logger for the trigger.** `main` sets up the log file only for the server; the trigger reports to stdout and stderr, printing the full error chain so a connection error shows its cause (for example `Connection refused`), and the server logs the request as it does any webhook.

## Risks / Trade-offs

- [Running `hooked trigger` outside the container requires the server's settings, including `--build-entry-script-path`, which the trigger does not use] → The README documents the `docker exec` form, where the environment provides them.
- [Anyone with `docker exec` access to the container can trigger builds] → They can already read the webhook secret and run the build script directly, so the trigger grants nothing new.
- [A commit that is not reachable from the watched branch reaches the build script] → The example scripts fetch only the watched branch, so `git archive` fails and the build is reported as failed; the README says the commit must be on the watched branch for those scripts.

## Migration Plan

No migration: existing command lines start the server as before, and the handler accepts every payload it accepted before. Rolling back removes the subcommand.
