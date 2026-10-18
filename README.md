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

Write a build script with the commands to run on each push, and make it executable. hooked passes the pushed commit ID as the first argument and sends the script's stdout and stderr to Discord. If the script traces commands with `set -x`, turn tracing off with `set +x` around any command that expands a secret. See [the static site example](example/static-site/build.sh) or [the Docker app example](example/docker-app/build.sh) for a complete script.

Builds run one at a time. If several pushes arrive while a build is running, hooked builds only the newest of them after the current build finishes and skips the rest.

Finally, start the web server:

```txt
hooked -h 0.0.0.0 -p 3000 --build-entry-script-path=./build.sh \
  --webhook-secret=<string> --watch-push-branch=master \
  --discord-webhook-url=<string>
```

### Docker

The image runs the build script with git configured for one base URL over HTTP or HTTPS:

- `GIT_URL`: The base URL the build script fetches from: `http` or `https`, a host, an optional port, and an optional path (for example `https://git.example.com` or `http://server:3000`). Defaults to `https://<GIT_HOST>`. Any other scheme makes the container print an error and exit. The resolved value, without a trailing slash, is exported to the build script, so the script can build repository URLs as `"$GIT_URL/<owner>/<repo>.git"`. Use `http` only on an internal network you trust, because the token is then sent in cleartext.
- `GIT_HOST`: A shorthand for `GIT_URL=https://<GIT_HOST>`. Defaults to `github.com`. `GIT_URL` takes precedence when both are set.
- `GIT_SSH_HOST`: The host, with an optional `:port`, whose SSH remotes (`ssh://git@<host>/`, `git+ssh://git@<host>/`, and `git@<host>:`) are rewritten to `GIT_URL`. Defaults to the host of `GIT_URL` without its port. Set it when the SSH domain differs from the web domain, for example `git-ssh.example.com` for `GIT_URL=https://git.example.com`. The SCP-style form `git@<host>:` cannot carry a port, so it matches the host alone. Only the `git` SSH user and non-IPv6 hosts are supported.
- `/run/secrets/git_token`: An access token for `GIT_URL`, provided as a Docker secret and stored for its scheme, host, and port. Without it, or when the file is empty, git runs unauthenticated.

This git setup applies only to processes started by hooked, so a shell opened with `docker exec` does not have it.

The container talks to the host Docker daemon through the mounted `/var/run/docker.sock`. Released images are published as `ghcr.io/cldotdev/hooked`. See [Examples](#examples) for complete compose setups.

**Security**: through the socket, hooked, the build script, and anything the build script runs from the repository (such as a compose file) have root on the host. Whoever can push to the watched branch controls the host unless the build script treats the repository as data only. Building a repository Dockerfile does not give host root under the conditions listed in [Dockerfile Build Isolation](#dockerfile-build-isolation). See the [static site example](example/static-site/README.md) and the [Docker app example](example/docker-app/README.md) for two ways to keep repository content away from the host.

#### Dockerfile Build Isolation

On Docker 29.7.2 with the default builder and no `daemon.json`, tests with host-controlled build flags showed that a repository Dockerfile could not reach host root:

- BuildKit rejected `RUN --security=insecure` and `RUN --network=host` as not allowed.
- `RUN` steps had default capabilities (mounting was denied) and no `docker.sock`.
- Symlinks and `../` paths could not reach host files, and a bind mount of `/` saw only the build context.
- Secret and SSH mounts failed unless the build passed them.

This has limits, and only the default Dockerfile frontend was tested:

- It holds only while the daemon does not enable the `security.insecure` or `network.host` entitlements and the build command does not pass `--allow`.
- The build has network access, including to services the host exposes on the bridge network. `build.network: none` closes this at the cost of dependency downloads.
- `RUN --mount=type=cache` mounts are shared across builds on the same daemon by default, so an untrusted build can poison a cache that another project uses.
- The running app gets whatever secrets, volumes, and networks its compose file grants it.
- A build can exhaust disk or CPU. The prune commands in the [Docker app example](example/docker-app/build.sh) only partly bound this.

### Examples

`example/` has two complete compose setups, a static site and a Docker app. See [example/README.md](example/README.md) for the setup they share, including running the container as your host user, and for links to each example's deployment steps.

## Migration

### Upgrading from 0.4.x

`compose.yaml` no longer uses `tecnativa/docker-socket-proxy`. Its filter matches only the request path and method, so an allowed `POST /containers/create` can still start a privileged container that mounts the host root, and `EXEC=1` reaches any container. It also broke buildx: the requests buildx makes (`POST /grpc`, `GET /v1.47/info`) fail through the proxy, so buildx switched to the `docker-container` driver and created a privileged `buildx_buildkit_default` container with its state volume and the `moby/buildkit:buildx-stable-1` image on the host. The hooked service now mounts the host socket directly.

1. Set `DOCKER_GID` in `.env`; see `.env.example` in either example directory.
2. Update `compose.yaml`, keeping your local edits: remove the proxy service, the `docker-proxy` network, `depends_on`, and `DOCKER_HOST`, and add the socket mount and `group_add`. If you mount a repository, use the same path inside the container as on the host.
3. Run `docker compose up -d --remove-orphans` to remove the old proxy container.
4. If the host built through the proxy, remove the buildx leftovers with `docker rm -f buildx_buildkit_default`, `docker volume rm buildx_buildkit_default_state`, and `docker image rm moby/buildkit:buildx-stable-1`. The builder that created them was recorded only inside the old hooked container, so `docker buildx rm` on the host does not know about it. They do not exist on other hosts.
5. In your build script, drop `--force-recreate`, bound `docker builder prune` with `--max-used-space`, and scope `docker image prune` with `--filter label=com.docker.compose.project=<name>`.

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
