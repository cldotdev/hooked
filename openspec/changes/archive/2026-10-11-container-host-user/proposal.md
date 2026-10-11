## Why

The container runs as root, so every file hooked and the build script create in bind mounts is owned by root on the host. `.srv/hooked/log` is already `root:root`, and a `git fetch` or `git checkout` in the repository mount leaves root-owned files that the host user cannot edit.

## What Changes

- `compose.yaml` runs the hooked service as `"${HOST_UID:-0}:${HOST_GID:-0}"`, so setting `HOST_UID` and `HOST_GID` in `.env` makes it run as the host user. Without them it stays root.
- The image sets `HOME` to a world-writable directory (`/home/hooked`, mode 1777), so an arbitrary UID without a passwd entry can still write `~/.git-credentials` and the Docker CLI's `~/.docker`.
- README Migration section: a one-time `chown` of existing root-owned files before switching to the host user.

## Capabilities

### New Capabilities

- `container-runtime-user`: Which user the hooked container runs as and what it needs to work under an arbitrary UID.

### Modified Capabilities

None.

## Impact

- `Dockerfile`, `compose.yaml`
- README: setup for `HOST_UID`/`HOST_GID` and the Migration entry
- Deployments that switch to a non-root UID must `chown` the existing `.srv/hooked` files (notably `log/`) first, or hooked fails to open its log file and exits
