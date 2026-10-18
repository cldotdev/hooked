# Examples

This directory has two compose projects that run hooked behind a Cloudflare Tunnel:

- [static-site](static-site/README.md): deploys a Hugo site, built in a throwaway container with no network.
- [docker-app](docker-app/README.md): deploys a long-running app from a repository Dockerfile.

Each example's README has its deployment steps. This page covers the setup both share. For the image interface (`GIT_URL`, `GIT_HOST`, `GIT_SSH_HOST`, and the `git_token` secret) and the security implications of mounting the Docker socket, see the [root README](../README.md#docker).

## Docker Socket

The container talks to the host Docker daemon through the mounted `/var/run/docker.sock`, and each `compose.yaml` adds the host docker group to the container with `group_add`. Copy `.env.example` from the example directory to `.env` next to your `compose.yaml` and set `DOCKER_GID` to the host docker group ID (`getent group docker | cut -d: -f3`). `DOCKER_GID` is required even when the container runs as root, and `docker compose` refuses to start without it.

## Running as Your Host User

The container runs as root by default, so files that hooked and the build script write into bind mounts, such as the log file and the repository checkout, end up owned by root on the host. To run it as your host user instead, uncomment `HOST_UID` and `HOST_GID` in `.env` (the output of `id -u` and `id -g`); the `hooked` service passes them to `user:`. Bash's own `UID` variable is not exported, so compose cannot read it. Before the first start, create `.srv/hooked/` as that user. Otherwise Docker creates it as root, and hooked cannot open its log file and exits:

```txt
mkdir -p .srv/hooked
```

hooked runs the build script directly, so it needs the executable bit: `chmod +x .srv/hooked/build.sh`.

## Project Name

Each example's compose file sets `name:` so that the project keeps its name when the directory changes; give every deployment on a host its own name.

## Cloudflare Tunnel

Both examples include a `cloudflared` service that connects a Cloudflare Tunnel using the token in `.secrets/cf_tunnel_token`, with its metrics server bound to the container's loopback.

The `cloudflared` service uses the `latest` image with `pull_policy: always`, so each `docker compose up -d` pulls the newest release; a container restart does not. The replaced image stays on the host without a tag, so remove it after the update:

```txt
docker compose up -d
docker image prune -f --filter label=org.opencontainers.image.source=https://github.com/cloudflare/cloudflared
```
