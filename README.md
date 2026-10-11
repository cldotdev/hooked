# hooked

A simple command-line CI/CD tool that triggers custom build and deployment commands when it receives push webhooks from GitHub or Gitea. It also supports Discord notification (including the stdout and stderr outputs).

## Screenshots

![](doc/screenshot/github_webhook.png)
![](doc/screenshot/discord_notification.png)

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

The container runs as root by default, so files that hooked and the build script write into bind mounts, such as the log file and the repository checkout, end up owned by root on the host. To run it as your host user instead, set the IDs in a `.env` file next to `compose.yaml`; the `hooked` service passes them to `user:`:

```txt
HOST_UID=1000
HOST_GID=1000
```

Use the output of `id -u` and `id -g`. Bash's own `UID` variable is not exported, so compose cannot read it.

See `compose.yaml` for a complete example. Released images are published as `ghcr.io/cldotdev/hooked`.

## Migration

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

The image moved from `ghcr.io/jlhg/hooked` to `ghcr.io/cldotdev/hooked`, and the former location no longer receives updates.

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
