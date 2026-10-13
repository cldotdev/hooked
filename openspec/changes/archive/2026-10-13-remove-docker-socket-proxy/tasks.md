## 1. Compose and Environment

- [x] 1.1 In `compose.yaml`, remove the `docker-socket-proxy` service, the top-level `networks:` block, and the `hooked` service's `depends_on`, `networks`, and `DOCKER_HOST`
- [x] 1.2 Mount `/var/run/docker.sock:/var/run/docker.sock` in the `hooked` service, add `group_add: ["${DOCKER_GID:?set DOCKER_GID in .env to the host docker group ID}"]`, and change the repository mount comment to `# - /path/to/repo/:/path/to/repo/`
- [x] 1.3 Add `.env.example` with `HOST_UID` and `HOST_GID` commented out and a `DOCKER_GID` placeholder, each with the command that gives its value

## 2. Examples

- [x] 2.1 Rewrite `example/build.sh`: drop the proxy permission header; show a secret read between `set +x` and `set -x`; replace the compose commands with `docker compose --progress plain up -d --build`, `docker builder prune -f --max-used-space <size>`, and `docker image prune -f --filter label=com.docker.compose.project=<name>`, with a short comment on why each is bounded or scoped
- [x] 2.2 Add `example/static-site/build.sh` following design.md, with the repository URL, Hugo image, base path, and `keep_releases` as variables at the top, and mark it executable in git
- [x] 2.3 Add `example/static-site/compose.yaml` with the `hooked` service (socket, `group_add`, same-path mounts of the build and site directories under the base path used in `build.sh`) and a `web` nginx service that mounts `./.srv/site/` read-only and `nginx.conf`
- [x] 2.4 Add `example/static-site/nginx.conf` serving `root /srv/site/current`

## 3. README and Version

- [x] 3.1 In the Docker section, document the socket mount and `DOCKER_GID` (pointing to `.env.example`), the same-path repository mount and why, the executable bit on the build script, and creating `.srv/hooked/` as the host user before the first start when `HOST_UID` is set
- [x] 3.2 Add a security note: the build script and anything it runs from the repository have host root, so whoever can push to the watched branch controls the host; point to the static site example
- [x] 3.3 Add a section describing `example/static-site/`: what the repository can and cannot affect, and how the files fit together
- [x] 3.4 Add "Upgrading from 0.4.x" to Migration: set `DOCKER_GID`, update `compose.yaml`, run `docker compose up -d --remove-orphans`, and, for hosts that built through the proxy, remove the `buildx_buildkit_default` builder state and the `moby/buildkit:buildx-stable-1` image
- [x] 3.5 Set `version = "0.5.0"` in `Cargo.toml` and run `cargo build` so `Cargo.lock` follows

## 4. Verification

- [x] 4.1 Run `docker compose config` for `compose.yaml` and `example/static-site/compose.yaml` with an env file in a temporary directory: with `DOCKER_GID` set, confirm the rendered `group_add` and socket mount; without it, confirm the command fails with a message that names `DOCKER_GID`
- [x] 4.2 Run `sh -n` on both build scripts, and `shellcheck` if it is installed
- [x] 4.3 Build the hooked image locally, run it as `1000:1000` with the socket mounted and the host docker group added, and confirm that `docker version` reaches the daemon and `docker buildx ls` shows the default builder with the `docker` driver
- [x] 4.4 In the locally built image, run `example/static-site/build.sh` against a minimal Hugo site in a temporary bare repository, with the variables pointed at temporary directories mounted at the same paths: confirm the release directory contains the rendered site, `current` points to it, a commit ID with non-hex characters is rejected, and only `keep_releases` releases remain after more builds than that
- [x] 4.5 Grep README, `compose.yaml`, `.env.example`, and `example/` for `proxy`, `DOCKER_HOST`, `docker-proxy`, `force-recreate`, and `/repo/`, and confirm that the only remaining matches are in the Migration entry
- [x] 4.6 Remove the test image, containers, and temporary directories
