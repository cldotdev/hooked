# Static Site Example

This example deploys a Hugo site so that the repository supplies only the site source. `build.sh` fetches the branch into a bare repository, exports the pushed commit with `git archive` so `.git` and untracked files never reach the build, and builds it in a throwaway Hugo container with no network, a read-only root filesystem, and only the source (read-only) and the release directory mounted. A malicious push can change the site's pages but cannot reach the host. The script then swaps the `current` symlink atomically and keeps the newest `keep_releases` releases.

The pieces fit together as follows:

- `build.sh`: the build script; see [Deployment Steps](#deployment-steps) for where it goes. Set the repository path (appended to `GIT_URL`), Hugo image, and `base_dir` at the top; the branch comes from `WATCH_PUSH_BRANCH`. The fetch uses the credentials the image sets up for `GIT_URL`.
- `compose.yaml`: the `hooked` service mounts the build and site directories at the same absolute paths as on the host, because `build.sh` passes them to `docker run -v`. These paths must match `base_dir` in `build.sh`. The `web` service serves `.srv/site/` read-only and is never touched by the build script. The `tunnel` service connects the Cloudflare Tunnel; route the tunnel's public hostnames to `http://hooked:3000` for the webhook and `http://web:80` for the site.
- `nginx.conf`: serves `/srv/site/current`, and the site's `404.html`, if it has one, for missing pages. nginx resolves the symlink on each request, so a new release needs no reload. It sets `Cache-Control: no-cache` on HTML only, because a generic Hugo site does not fingerprint its assets, takes the client IP from `CF-Connecting-IP` when the request comes from a Docker network address, and enables gzip.
- `.env.example`: copy it to `.env` and set `DOCKER_GID`; see [Docker Socket](../README.md#docker-socket).

The build and site directories must be writable by the user hooked runs as.

## Deployment Steps

Run these steps in the directory you deploy from. The shared setup is explained in [the examples README](../README.md).

1. Copy `compose.yaml`, `nginx.conf`, and `.env.example` from this directory into the deployment directory. Copy `.env.example` to `.env`, set `DOCKER_GID`, and optionally set `HOST_UID` and `HOST_GID`; see [Docker Socket](../README.md#docker-socket) and [Running as Your Host User](../README.md#running-as-your-host-user).
2. Create `.srv/hooked/` as the user that will run hooked.
3. Copy `build.sh` to `.srv/hooked/build.sh` and run `chmod +x .srv/hooked/build.sh`. The `hooked` service mounts `./.srv/hooked/` at `/app/` and sets `BUILD_ENTRY_SCRIPT_PATH=./build.sh`, so this is the file hooked runs.
4. Edit the variables at the top of `.srv/hooked/build.sh`: the repository path in `repo_url` (`<owner>/<repo>`), `hugo_image`, `base_dir`, and `keep_releases`. Set `base_dir` to the absolute path of the directory that holds `compose.yaml`, so that `$base_dir/.srv/site` is the directory the `web` service serves from `./.srv/site/`.
5. In `compose.yaml`, replace `/path/to/site` in the two build and site volumes with the same `base_dir`, on both sides of the colon. Set `name:`, `WATCH_PUSH_BRANCH`, and `GIT_HOST` (or `GIT_URL` and `GIT_SSH_HOST`) as needed; see [Project Name](../README.md#project-name).
6. Create `.srv/build/` and `.srv/site/` under `base_dir` as the user hooked runs as. Otherwise Docker creates them as root and the build cannot write to them.
7. Create the secret files in `.secrets/`:
   - `webhook_secret`: the secret token you set on the repository webhook.
   - `git_token`: an access token for `GIT_URL`. Leave the file empty for a public repository; git then runs unauthenticated. Keep the file, because `compose.yaml` still references it and the container cannot start without it.
   - `discord_webhook_url`: the Discord webhook URL. Leave the file empty to disable Discord notifications; hooked treats an empty value as unset. Keep the file, because `compose.yaml` still references it and hooked exits if it cannot read `/run/secrets/discord_webhook_url`.
   - `cf_tunnel_token`: the Cloudflare Tunnel token.
8. Start the stack with `docker compose up -d`.
9. Configure the webhook on the repository as described in [Usage](../../README.md#usage), with the same secret as `.secrets/webhook_secret`.
10. In the Cloudflare Tunnel, route one public hostname to `http://hooked:3000` for the webhook and another to `http://web:80` for the site. Use the hooked hostname as the webhook URL's base, for example `https://<hooked-hostname>/webhooks/github`.
11. Push to the watched branch. The first build creates the bare repository and the first release, and `web` has nothing to serve until it finishes. To update the tunnel, see [Cloudflare Tunnel](../README.md#cloudflare-tunnel).
