# hooked

A simple command-line CI/CD tool that triggers custom build and deployment commands when it receives push webhooks from GitHub or Gitea. It also supports Discord notification (including the stdout and stderr outputs).

## Screenshots

![GitHub webhook delivery accepted by hooked](doc/screenshot/github_webhook.png)
![Discord notification](doc/screenshot/discord_notification.png)

## Installation

### Build from the source code

Clone the repository:

```txt
git clone https://github.com/cldotdev/hooked.git
```

Run `cargo build`:

```txt
cd hooked
cargo build --release
```

You can find the executable file at `./target/release/hooked`.

## Usage

Find the usage:

```txt
hooked --help
```

The available arguments are described below:

- `-l, --log`: Path to the log file. Defaults to `log/app.log`.
- `-h, --host`: The domain or IP address where the server is hosted.
- `-p, --port`: The port number where the server is listening.
- `--build-entry-script-path`: Path to the build entry script.
- `--webhook-secret`: The secret token to verify incoming webhook messages. See [Creating webhooks - GitHub Docs](https://docs.github.com/en/webhooks/using-webhooks/creating-webhooks) or [Webhooks - Gitea Docs](https://docs.gitea.com/usage/webhooks) for creating a webhook and setting the secret token.
- `--watch-push-branch`: The Git branch name to watch.
- `--discord-webhook-url`: The Discord webhook URL to send the notification to a Discord channel. See [Intro to Webhooks – Discord](https://support.discord.com/hc/en-us/articles/228383668-Intro-to-Webhooks) for creating a Discord webhook URL. Optional; when it is unset or empty, hooked only writes each build's commit ID and exit status to the log.

Configure the webhook on the repository with the content type `application/json`, the same secret token, and the push event:

- GitHub: set the payload URL to `<your-server-address>/webhooks/github`.
- Gitea: add a webhook of the type "Gitea" and set the target URL to `<your-server-address>/webhooks/gitea`.

Copy the example `build.sh` to the current directory. Open it and add whatever commands you want to execute. hooked passes the pushed commit ID as the first argument.

```txt
cp example/build.sh .
```

Builds run one at a time. If several pushes arrive while a build is running, hooked builds only the newest of them after the current build finishes and skips the rest.

Finally, start the web server:

```txt
hooked -h 0.0.0.0 -p 3000 --build-entry-script-path=./build.sh \
  --webhook-secret=<string> --watch-push-branch=master \
  --discord-webhook-url=<string>
```

### Docker

The image runs the build script with git configured for one host over HTTPS:

- `GIT_HOST`: The git host the build script fetches from. Defaults to `github.com`. SSH remote URLs for this host are rewritten to HTTPS.
- `/run/secrets/git_token`: An access token for `GIT_HOST`, provided as a Docker secret. Without it, git runs unauthenticated.

This git setup applies only to processes started by hooked, so a shell opened with `docker exec` does not have it.

The container talks to the host Docker daemon through the mounted `/var/run/docker.sock`, and `compose.yaml` adds the host docker group to the container with `group_add`. Copy `.env.example` to `.env` next to `compose.yaml` and set `DOCKER_GID` to the host docker group ID (`getent group docker | cut -d: -f3`). `DOCKER_GID` is required even when the container runs as root, and `docker compose` refuses to start without it.

**Security**: through the socket, hooked, the build script, and anything the build script runs from the repository (such as a compose file or a Dockerfile) have root on the host. Whoever can push to the watched branch controls the host. To keep repository content away from the host, make the build script treat the repository as data only; see [Static Site Example](#static-site-example).

The container runs as root by default, so files that hooked and the build script write into bind mounts, such as the log file and the repository checkout, end up owned by root on the host. To run it as your host user instead, uncomment `HOST_UID` and `HOST_GID` in `.env` (the output of `id -u` and `id -g`); the `hooked` service passes them to `user:`. Bash's own `UID` variable is not exported, so compose cannot read it. Before the first start, create `.srv/hooked/` as that user. Otherwise Docker creates it as root, and hooked cannot open its log file and exits:

```txt
mkdir -p .srv/hooked
```

hooked runs the build script directly, so it needs the executable bit: `chmod +x .srv/hooked/build.sh`.

Mount a repository at the same absolute path inside the container as on the host (`/path/to/repo/:/path/to/repo/`). When the build script runs `docker compose`, compose resolves relative bind and secret paths on the client side and the host daemon mounts the result, so a repository mounted at a different path produces mount sources that do not exist on the host, and secret mounts fail.

See `compose.yaml` for a complete example. Released images are published as `ghcr.io/cldotdev/hooked`.

### Static Site Example

`example/static-site/` deploys a Hugo site so that the repository supplies only the site source. `build.sh` fetches the branch into a bare repository, exports the pushed commit with `git archive` so `.git` and untracked files never reach the build, and builds it in a throwaway Hugo container with no network, a read-only root filesystem, and only the source (read-only) and the release directory mounted. A malicious push can change the site's pages but cannot reach the host. The script then swaps the `current` symlink atomically and keeps the newest `keep_releases` releases.

The pieces fit together as follows:

- `build.sh`: copy it as the build script. Set the repository URL, Hugo image, and `base_dir` at the top; the branch comes from `WATCH_PUSH_BRANCH`. The fetch uses the credentials the image sets up for `GIT_HOST`.
- `compose.yaml`: the `hooked` service mounts the build and site directories at the same absolute paths as on the host, because `build.sh` passes them to `docker run -v`. These paths must match `base_dir` in `build.sh`. The `web` service serves `.srv/site/` read-only and is never touched by the build script.
- `nginx.conf`: serves `/srv/site/current`. nginx resolves the symlink on each request, so a new release needs no reload.

The build and site directories must be writable by the user hooked runs as.

## Migration

### Upgrading from 0.4.x

`compose.yaml` no longer uses `tecnativa/docker-socket-proxy`. Its filter matches only the request path and method, so an allowed `POST /containers/create` can still start a privileged container that mounts the host root, and `EXEC=1` reaches any container. It also broke buildx: the requests buildx makes (`POST /grpc`, `GET /v1.47/info`) fail through the proxy, so buildx switched to the `docker-container` driver and created a privileged `buildx_buildkit_default` container with its state volume and the `moby/buildkit:buildx-stable-1` image on the host. The hooked service now mounts the host socket directly.

1. Set `DOCKER_GID` in `.env`; see `.env.example`.
2. Update `compose.yaml`, keeping your local edits: remove the proxy service, the `docker-proxy` network, `depends_on`, and `DOCKER_HOST`, and add the socket mount and `group_add`. If you mount a repository, use the same path inside the container as on the host.
3. Run `docker compose up -d --remove-orphans` to remove the old proxy container.
4. If the host built through the proxy, remove the buildx leftovers with `docker rm -f buildx_buildkit_default`, `docker volume rm buildx_buildkit_default_state`, and `docker image rm moby/buildkit:buildx-stable-1`. The builder that created them was recorded only inside the old hooked container, so `docker buildx rm` on the host does not know about it. They do not exist on other hosts.
5. In your build script, consider the changes in `example/build.sh`: drop `--force-recreate`, bound `docker builder prune` with `--max-used-space`, and scope `docker image prune` with `--filter label=com.docker.compose.project=<name>`.

### Upgrading from 0.3.x

The settings no longer carry a `github` prefix, and the former names are not accepted:

| Former | Current |
|--------|---------|
| `--github-webhook-secret` | `--webhook-secret` |
| `GITHUB_WEBHOOK_SECRET` | `WEBHOOK_SECRET` |
| `--github-watch-push-branch` | `--watch-push-branch` |
| `GITHUB_WATCH_PUSH_BRANCH` | `WATCH_PUSH_BRANCH` |
| Docker secret `/run/secrets/github_pat` | Docker secret `/run/secrets/git_token` |

`compose.yaml` also renames the secret `github_webhook_secret` to `webhook_secret`, so rename the files in `.secrets/` to match.

To run the container as your host user with `HOST_UID` and `HOST_GID`, first give that user the files the root container already created, or hooked cannot open its log file and exits. With the stack stopped, run:

```txt
sudo chown -R "$(id -u):$(id -g)" .srv/hooked /path/to/repo
```

## Releasing

1. Set `version` in `Cargo.toml` to the new version and run `cargo build` so that `Cargo.lock` follows, then commit.
2. Push an annotated tag named after the version, without a `v` prefix (for example `0.4.0`). The annotation becomes the release notes.

The release workflow fails if the tag differs from the `Cargo.toml` version. Otherwise it pushes the image as `ghcr.io/cldotdev/hooked:<version>` and `latest` and creates the GitHub Release. GHCR creates the package as private on its first push, so set its visibility to public once in the package settings.

## License

Released under the [MIT License](https://opensource.org/license/mit/).
