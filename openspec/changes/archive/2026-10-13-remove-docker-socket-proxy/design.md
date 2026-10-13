## Context

See proposal.md - Why for the problems with the proxy. hooked's image ships the Docker CLI (`docker:latest`) and git, and the build script talks to the host daemon, so any compose file, Dockerfile, or script that the build script runs from the repository runs with host root. The image itself does not change in this change; only the compose setup, the examples, and the README do.

The host socket is `root:docker 660`. When hooked runs as `HOST_UID`, it reaches the socket only through the docker group.

When the build script runs `docker compose` inside the container, compose resolves relative bind and secret paths to absolute paths on the client side and passes them to the host daemon, which mounts host paths. A repository mounted at a different path inside the container therefore produces mounts that point at nonexistent host paths; the current `# - /path/to/repo/:/repo/` comment leads there.

## Goals / Non-Goals

**Goals:**
- Docker access through the host socket, with buildx using the daemon's own builder.
- Documentation that states the trust boundary plainly and shows a deployment pattern that keeps repository content away from host root.
- Examples whose cleanup commands affect only the deployed project.

**Non-Goals:**
- Restricting what hooked can do through the socket. Any filter in front of the socket that allows container creation is bypassable, so the boundary has to be what the build script runs, not the API.
- Changing `GIT_HOST` handling or the entrypoint.
- Supporting both the proxy and the socket in `compose.yaml`.

## Decisions

- **Mount the socket instead of tightening the proxy.** Tighter proxy ACLs cannot inspect request bodies, so they cannot stop a privileged `containers/create` while still allowing `docker compose up`. Mounting the socket admits the same trust level openly and lets buildx use the docker driver.
- **`group_add: ["${DOCKER_GID:?set DOCKER_GID in .env to the host docker group ID}"]`, required for every user.** A default of `0` would let root deployments skip the variable, but a deployment that sets `HOST_UID` and forgets `DOCKER_GID` would then start and fail only at the first build with a permission error on the socket. Requiring it fails at `docker compose up`, and the release is breaking anyway.
- **`.env.example` at the repository root.** It lists `HOST_UID`, `HOST_GID`, and `DOCKER_GID` with the command for each (`id -u`, `id -g`, `getent group docker | cut -d: -f3`). `HOST_UID` and `HOST_GID` stay commented out so copying the file keeps the root default; `DOCKER_GID` is a placeholder the user must fill in. `.env` stays git-ignored and Docker-ignored.
- **Same-path repository mount.** The example comment becomes `# - /path/to/repo/:/path/to/repo/`, and the README explains why.
- **`example/build.sh` commands.**
  - `docker compose --progress plain up -d --build`: builds and recreates only services whose configuration or image changed; `--force-recreate` also restarts unchanged services, such as a tunnel, and drops their connections.
  - `docker builder prune -f --max-used-space <size>`: the builder's default GC policy scales with disk size (93.13GiB on a 1.8T disk), and an unbounded prune also drops base image layers, so every build pulls them again.
  - `docker image prune -f --filter label=com.docker.compose.project=<name>`: images built by compose carry this label; an unfiltered prune removes dangling images of every stack on the host.
  - A secret read shown between `set +x` and `set -x`, since the script's stdout and stderr go to Discord and the log.
- **`example/static-site/` as a self-contained deployment.** `build.sh`, `compose.yaml`, and `nginx.conf` sit together so the compose file can be checked with `docker compose config` and copied as a unit; a root-level `compose.*.yaml` would also be git-ignored. The script is the version verified in production, made generic:
  - The repository is fetched into a bare repository in hooked's working directory and exported with `git archive`, so `.git` and untracked files in any checkout never reach the build.
  - The commit ID is checked to be lowercase hex before it names directories that the script removes.
  - Hugo runs with `--network none`, `--read-only`, a tmpfs `/tmp`, the caller's UID and GID, the source mounted read-only, and only the release directory writable. `--noBuildLock` and `HUGO_CACHEDIR` on the tmpfs are what Hugo needs with a read-only source.
  - The `current` symlink is replaced by creating `current.tmp` and renaming it with `mv -fT` (verified with the image's busybox), because `ln -sfn` unlinks before linking and nginx could see no `current` in between. nginx reads `root /srv/site/current` per request, so the swap needs no reload.
  - Older releases beyond `keep_releases` are removed.
  - The build and site directories are mounted into hooked at the same absolute path as on the host, because the script passes them to `docker run -v`. The example uses one placeholder base path in both `compose.yaml` and `build.sh`.
  - The repository URL, Hugo image, and paths are variables at the top of the script with placeholder values. The fetch relies on the entrypoint's credentials for `GIT_HOST` rather than the production `http.extraHeader` form, because `Authorization: token` is accepted by Gitea but is not a documented scheme for GitHub's git endpoint.
  - The `web` service mounts `./.srv/site/` read-only and is never touched by the build script.
- **Version 0.5.0.** Pre-1.0 semver puts a breaking change in the minor version. The image content is the same as 0.4.0, but the tag marks the point the README's Migration entry refers to.

## Risks / Trade-offs

- [hooked, and anyone who can push to the watched branch, has host root through the socket] → This was already true with the proxy's permissions; the README now says so and points to the static site example.
- [Deployments that copy the new `compose.yaml` without `DOCKER_GID` fail to start] → The `:?` message names the variable, and `.env.example` and the Migration entry show how to get it.
- [The proxy container keeps running after the upgrade as an orphan with the socket mounted] → Migration uses `docker compose up -d --remove-orphans`.
- [buildx resources the proxy caused remain on hosts that built through it] → Migration lists `docker buildx rm` for the builder state and `docker image rm moby/buildkit:buildx-stable-1`, and notes that they exist only on hosts that built through the proxy.
- [`--max-used-space` needs a recent Docker CLI] → The image is `docker:latest`, which has it; the example does not branch on older versions.

## Migration Plan

Deploy by setting `DOCKER_GID` in `.env`, replacing `compose.yaml` with the new version (keeping local edits), and running `docker compose up -d --remove-orphans`. Rolling back means restoring the old `compose.yaml` and running the same command; the 0.5.0 image is identical to 0.4.0, so the image tag does not matter for rollback.
