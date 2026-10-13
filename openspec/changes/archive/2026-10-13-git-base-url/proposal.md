## Why

The entrypoint ties git access to one `GIT_HOST`: the token is stored only for `https://GIT_HOST`, and SSH remotes on that same host are rewritten to `https://GIT_HOST/`. That fails for a self-hosted Gitea whose SSH domain differs from its web domain (`git-ssh.example.com` against `git.example.com`), and for a build that must reach the forge through an internal URL with a different scheme or port (`http://server:3000`), for example because the public URL sits behind Cloudflare Access. Deployments work around it today by reading the token in the build script and passing it through `http.extraHeader`.

## What Changes

- New `GIT_URL`: the base URL (`http` or `https`, host, optional port and path) that git credentials and SSH rewrites target. When unset, it is `https://<GIT_HOST>`, so existing `GIT_HOST` deployments behave as before.
- New `GIT_SSH_HOST`: the host, with an optional port, whose SSH remotes are rewritten to `GIT_URL`. It defaults to the host of `GIT_URL`.
- The entrypoint exports the resolved `GIT_URL`, so a build script can build repository URLs from it.
- The entrypoint exits with an error when `GIT_URL` has a scheme other than `http` or `https`.
- `compose.yaml`, the static site example, and the README show the new variables. Version 0.6.0.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `container-git-access`: The configured target becomes a base URL instead of a host, the SSH host is configurable separately, and credentials follow the URL's scheme and port.

## Impact

- `entrypoint.sh`, `compose.yaml`, `example/static-site/build.sh`, README, `Cargo.toml`/`Cargo.lock` (version only).
- No migration needed: deployments that set only `GIT_HOST` get the same configuration.
