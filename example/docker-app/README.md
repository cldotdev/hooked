# Docker App Example

This example deploys a long-running app so that the repository supplies only a Dockerfile and source, while the host owns all runtime configuration. `build.sh` fetches the branch into a bare repository, exports the pushed commit with `git archive` into a fixed source directory, and runs `docker compose up -d --build --wait` on a compose file that lives on the host. The build fails if the app does not pass the healthcheck in `app/compose.yaml` within the `--wait-timeout` set in `build.sh`; without a healthcheck, it only fails if the container never reaches the running state. A malicious push can change what the build runs but cannot grant it host access. The script then prunes the daemon-wide build cache and the project's untagged images.

The pieces fit together as follows:

- `build.sh`: the build script; see [Deployment Steps](#deployment-steps) for where it goes. Set the repository path (appended to `GIT_URL`) and `deploy_dir` at the top; the branch comes from `WATCH_PUSH_BRANCH`. The script reads the compose project name from `app/compose.yaml`, so the name is set in one place.
- `compose.yaml`: the deploy project with the `hooked` and `tunnel` services. The `hooked` service mounts the deploy directory at the same absolute path as on the host, because `docker compose` runs in the hooked container, where it resolves any bind mounts and secret files in `app/compose.yaml`. This path must match `deploy_dir` in `build.sh`. Route the tunnel's public hostnames to `http://hooked:3000` for the webhook and `http://app:<port>` for the app.
- `app/compose.yaml`: the app as a separate compose project, so that `up` can never recreate the hooked container that is running the build. Its `build.context` is the exported source, relative to this file. Hardening options and `build.network: none` are commented out because they can break an app.
- `.env.example`: copy it to `.env` and set `DOCKER_GID`; see [Docker Socket](../README.md#docker-socket).

The deploy directory must be writable by the user hooked runs as, because `build.sh` replaces `src/` in it on each build.

The app and the `tunnel` service are in different projects, so they share a network named `my-app-tunnel`; give it a name unique on the host. The deploy project defines it and attaches only the `tunnel` service to it, and `app/compose.yaml` joins it as external. hooked stays off it, so the app cannot reach hooked directly. Start the deploy project first, because it creates the network. Rollback is a `git revert` and a push; the image is rebuilt under the same name and not tagged per commit.

The build context does not need the same path on the host and in the container: `docker compose build` streams it to the daemon, and a relative `build.context` resolves against the directory of the compose file. Only bind mounts and secret files in `app/compose.yaml` need matching paths. Build secrets must also be under `deploy_dir`; see [Build Secrets and the Bake Check](#build-secrets-and-the-bake-check).

For what a repository Dockerfile can and cannot reach during the build, see [Dockerfile Build Isolation](../../README.md#dockerfile-build-isolation).

## Deployment Steps

Run these steps in the directory you deploy from. The shared setup is explained in [the examples README](../README.md).

1. Copy `compose.yaml` and `.env.example` from this directory into the deployment directory. Copy `.env.example` to `.env`, set `DOCKER_GID`, and optionally set `HOST_UID` and `HOST_GID`; see [Docker Socket](../README.md#docker-socket) and [Running as Your Host User](../README.md#running-as-your-host-user).
2. Create `.srv/hooked/` as the user that will run hooked.
3. Copy `build.sh` to `.srv/hooked/build.sh` and run `chmod +x .srv/hooked/build.sh`. The `hooked` service mounts `./.srv/hooked/` at `/app/` and sets `BUILD_ENTRY_SCRIPT_PATH=./build.sh`, so this is the file hooked runs.
4. Edit the variables at the top of `.srv/hooked/build.sh`: the repository path in `repo_url` (`<owner>/<repo>`) and `deploy_dir`.
5. In `compose.yaml`, replace `/path/to/deploy` in the `hooked` volumes with the same `deploy_dir`, on both sides of the colon. Set `name:`, the `tunnel` network `name:`, `WATCH_PUSH_BRANCH`, and `GIT_HOST` (or `GIT_URL` and `GIT_SSH_HOST`) as needed; see [Project Name](../README.md#project-name).
6. Create the deploy directory at the `deploy_dir` path, owned by the user hooked runs as, and copy `app/compose.yaml` to `<deploy_dir>/app/compose.yaml`. Edit it: set `name:`, set the network `name:` to the same network name as in the deploy `compose.yaml`, enable the `healthcheck` with the app's port, and add the secrets, volumes, and hardening options the app needs. Bind mounts, secret files, and any other files this file reads must use paths under `deploy_dir`, which exist both on the host and in the hooked container.
7. Create the secret files in `.secrets/`:
   - `webhook_secret`: the secret token you set on the repository webhook.
   - `git_token`: an access token for `GIT_URL`. Leave the file empty for a public repository; git then runs unauthenticated. Keep the file, because `compose.yaml` still references it and the container cannot start without it.
   - `discord_webhook_url`: the Discord webhook URL. Leave the file empty to disable Discord notifications; hooked treats an empty value as unset. Keep the file, because `compose.yaml` still references it and hooked exits if it cannot read `/run/secrets/discord_webhook_url`.
   - `cf_tunnel_token`: the Cloudflare Tunnel token.
8. Start the deploy project with `docker compose up -d`. Do this before the first push, because it creates the `tunnel` network that `app/compose.yaml` joins as external.
9. Configure the webhook on the repository as described in [Usage](../../README.md#usage), with the same secret as `.secrets/webhook_secret`.
10. In the Cloudflare Tunnel, route one public hostname to `http://hooked:3000` for the webhook and another to `http://app:<port>` for the app. Use the hooked hostname as the webhook URL's base, for example `https://<hooked-hostname>/webhooks/github`.
11. Push to the watched branch. The first build creates the bare repository, exports the source into `<deploy_dir>/src`, and starts the app project. To update the tunnel, see [Cloudflare Tunnel](../README.md#cloudflare-tunnel).

## Build Secrets and the Bake Check

Docker Compose v5 builds through `buildx bake`, which refuses to read a file outside the process's working directory unless the build passes `--allow=fs.read=<path>`. This covers files such as build secrets declared with `file:` in `app/compose.yaml`; the build context is not checked. A build that reads a file outside the working directory fails with:

```txt
additional privileges requested: pass "--allow=fs.read=<path>" to grant requested privileges
```

hooked starts the build script in `/app`, so `build.sh` runs `cd "$deploy_dir"` before `docker compose`, and every file under `deploy_dir` passes the check. Keep build secrets and any other files `app/compose.yaml` reads under `deploy_dir`, for example in `<deploy_dir>/.secrets/`. Running compose from the compose file's directory (`<deploy_dir>/app`) does not work for a secret in `<deploy_dir>/.secrets/`, because the path leaves that directory. Neither `--project-directory` nor `COMPOSE_BAKE=false` changes the check. This was observed with Compose v5.6.0 and buildx v0.38.0 in `ghcr.io/cldotdev/hooked:0.6.3`; a later buildx release may resolve paths differently.

If the files must stay outside `deploy_dir`, set `BUILDX_BAKE_ENTITLEMENTS_FS=0` under `environment:` of the `hooked` service in `compose.yaml`. This turns the check off for every build in the container. The bake definition comes from `app/compose.yaml` on the host, not from the repository, so a push cannot use the setting to read host files, but a compose file that names a wrong path then reads it without any error.
