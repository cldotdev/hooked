## Why

Images built on Docker Compose v5 (hooked 0.6.3 ships Compose v5.6.0 with buildx v0.38.0) build through `buildx bake`, which refuses to read a file outside the process's working directory unless the build passes `--allow=fs.read=<path>`. hooked starts the build script in `/app`, so `docker compose up --build` in the docker-app example fails as soon as `app/compose.yaml` declares a build secret with `file:` under `deploy_dir`. Compose v2 did not check this, so deployments that upgrade from older hooked images break on the first build.

## What Changes

- `example/docker-app/build.sh` changes into `deploy_dir` before running `docker compose`, so files under `deploy_dir` pass the check without disabling it.
- `example/docker-app/README.md` states the rule (build secrets and other files `app/compose.yaml` reads must be under `deploy_dir`), says that `COMPOSE_BAKE=false` and `--project-directory` do not help, and describes `BUILDX_BAKE_ENTITLEMENTS_FS=0` on the `hooked` service as the escape hatch for files kept elsewhere.
- Root README: a Migration entry for deployments whose build script runs `docker compose` from `/app`.
- The image does not set `BUILDX_BAKE_ENTITLEMENTS_FS`, and the base image stays `docker:latest`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. The change touches only the example and documentation; hooked and the image behave as before.

## Impact

- `example/docker-app/build.sh`, `example/docker-app/README.md`, `README.md`.
- Existing docker-app deployments must add the `cd` to their copy of `build.sh`, or set `BUILDX_BAKE_ENTITLEMENTS_FS=0`, before they build with Compose v5.
