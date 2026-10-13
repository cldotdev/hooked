## Why

The compose setup routes Docker access through `tecnativa/docker-socket-proxy`, but the proxy is not a security boundary: its HAProxy ACL matches only the path and method, so with the `CONTAINERS=1` and `POST=1` the example enables, `POST /containers/create` with `Privileged: true` and `Binds: ["/:/host"]` yields host root, and `EXEC=1` reaches any container. It also breaks buildx: `POST /grpc` returns 500 and `GET /v1.47/info` returns 403, so buildx falls back to the docker-container driver and creates a privileged `buildx_buildkit_default` container, its state volume, and a 249MB `moby/buildkit` image on the host.

## What Changes

- **BREAKING** `compose.yaml` drops the `docker-socket-proxy` service, the `docker-proxy` network, and `DOCKER_HOST`. The hooked service mounts `/var/run/docker.sock` and joins the host docker group through `group_add`, so `DOCKER_GID` must be set in `.env`, even when the container runs as root.
- New `.env.example` listing `HOST_UID`, `HOST_GID`, and `DOCKER_GID` with the command that gives each.
- The repository mount example uses the same absolute path inside the container as on the host, because compose resolves relative bind and secret paths on the client side.
- `example/build.sh` drops the proxy permission list and shows a secret read with tracing off, `docker compose up -d --build` without `--force-recreate`, a bounded `docker builder prune`, and an `image prune` scoped to the compose project.
- New `example/static-site/` with `build.sh`, `compose.yaml`, and `nginx.conf`: a deployment where the repository supplies only Hugo source, built in a throwaway container without network access, and served by a separate nginx service.
- README: a security note that the build script and anything it runs from the repository have host root, the same-path mount, the executable bit on the build script, pre-creating `.srv/hooked/` as the host user, the static site example, and a Migration entry for 0.4.x.
- Version 0.5.0.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `container-runtime-user`: The compose configuration gives hooked Docker access through the mounted host socket and the host docker group, and requires `DOCKER_GID`; the "not configured" scenario no longer starts without it.

## Impact

- `compose.yaml`, `.env.example`, `example/build.sh`, `example/static-site/`, README, `Cargo.toml`/`Cargo.lock` (version only). The image itself is unchanged.
- Existing deployments must set `DOCKER_GID`, remove the orphaned proxy container, and may remove buildx resources the proxy caused.
