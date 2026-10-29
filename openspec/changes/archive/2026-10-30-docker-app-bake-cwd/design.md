## Context

See proposal.md - Why. A reproduction in `ghcr.io/cldotdev/hooked:0.6.3` (docker CLI 29.9.1, Compose v5.6.0, buildx v0.38.0) against a 29.9.2 daemon, with a build secret at `<deploy>/.secrets/tok` and the compose file at `<deploy>/app/compose.yaml`, gave:

| Working directory or setting | Result |
|------------------------------|--------|
| `/app` | fails: `additional privileges requested: pass "--allow=fs.read=..."` |
| `<deploy>` | builds |
| `<deploy>/app` (the compose file's directory) | fails for `../.secrets/tok` |
| `/app` with `--project-directory <deploy>` | fails |
| `/app` with `COMPOSE_BAKE=false` | fails |
| `/app` with `BUILDX_BAKE_ENTITLEMENTS_FS=0` | builds |

The build context (`../src`) is not subject to the check; only files such as build secrets are. The static-site example runs `docker run`, not `docker compose build`, and is not affected.

## Goals / Non-Goals

**Goals:**
- The docker-app example builds with Compose v5 when `app/compose.yaml` declares build secrets under `deploy_dir`, with the entitlement check still on.
- Users with files outside `deploy_dir` find the escape hatch and its trade-off in the example README.

**Non-Goals:**
- Setting `BUILDX_BAKE_ENTITLEMENTS_FS=0` in the image or in the example `compose.yaml`.
- Pinning the `docker` base image or the Compose version.

## Decisions

- **`cd "$deploy_dir"` in `build.sh` instead of disabling the check.** The check is relative to the process's working directory, so running compose from `deploy_dir` admits exactly the files the README already requires to live there (bind mounts and secret files must have the same path on the host and in the container). Disabling the check would also work, since `app/compose.yaml` is host-owned and the repository cannot change the bake definition, but a default applies to every deployment, including layouts where it is not needed. The `cd` goes after the `git archive` step, because the earlier commands use the relative `repo.git` in hooked's working directory; the commands after it use absolute paths or none.
- **Escape hatch documented, not shipped.** `BUILDX_BAKE_ENTITLEMENTS_FS=0` in the `hooked` service's `environment:` is described in the example README for deployments that keep build secrets outside `deploy_dir`, with the reason it is acceptable there (the compose file is on the host) and what it gives up.
- **Migration entry keyed to Compose v5, not a hooked version.** The image is `FROM docker:latest`, so the Compose version an image carries depends on when it was built. The entry names the symptom and both fixes.

## Risks / Trade-offs

- [A deployment keeps build secrets outside `deploy_dir` and copies only the new `build.sh`] → The build still fails with the same `--allow=fs.read` message; the README entry quotes that message and names both fixes.
- [A later buildx release changes how the check resolves paths] → The base image floats, so the example could break again without a hooked change; the README states the observed behavior and the versions it was observed on.

## Migration Plan

Existing docker-app deployments add `cd "$deploy_dir"` before the `docker compose` call in their `.srv/hooked/build.sh` and move build secrets under `deploy_dir`, or set `BUILDX_BAKE_ENTITLEMENTS_FS=0` on the `hooked` service. Rolling back the example is reverting `build.sh`; builds under Compose v2 behave the same either way.
