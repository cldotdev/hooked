## Context

See proposal.md - Why. The entrypoint exports git configuration through `GIT_CONFIG_*` variables (`credential.helper store` and three `url.https://$GIT_HOST/.insteadOf` rules) and writes `https://token:<token>@$GIT_HOST` to `~/.git-credentials`. Git's credential store matches an entry on protocol, host, and port, and ignores the path unless `credential.useHttpPath` is set. The entrypoint runs under busybox `sh`.

## Goals / Non-Goals

**Goals:**
- A deployment can point git at any `http` or `https` base URL, and at an SSH host that differs from it, without handling the token in the build script.
- Deployments that set only `GIT_HOST`, or nothing, get exactly the configuration they get today.

**Non-Goals:**
- Several git hosts in one container.
- Extra request headers, such as Cloudflare Access service tokens; reaching the forge through an internal URL avoids them.
- SSH users other than `git`, and IPv6 literal hosts.

## Decisions

- **`GIT_URL` as a base URL, with `GIT_HOST` kept as a shorthand.** A full URL carries the scheme and port that an internal endpoint needs, and an optional path for a forge served under a subpath. Keeping `GIT_HOST` as the default source (`https://<GIT_HOST>`) makes the change additive. Deprecating `GIT_HOST` was considered and dropped: it would force every deployment to edit its configuration for no behavior gain.
- **Parse with shell parameter expansion.** The entrypoint strips trailing slashes, takes the scheme before `://`, the authority up to the first `/`, and the host before the first `:`. A scheme other than `http` or `https` exits with an error, because the URL would otherwise be silently written into git rules that never match. No URL library is added to the image.
- **Credential entry without the path.** The store entry is `<scheme>://token:<token>@<authority>`. Since the store ignores paths by default, a path in the entry would not narrow it, and dropping it keeps the entry simple. Scheme and port are part of the match, so the token for `http://server:3000` is not sent to `https://server`.
- **`GIT_SSH_HOST` defaults to the host of `GIT_URL` without the port.** The web port says nothing about the SSH port, so the default rewrites `ssh://git@<host>/` and `git@<host>:` exactly as today. When `GIT_SSH_HOST` carries a port, the `ssh://` and `git+ssh://` prefixes include it, and the SCP-style prefix uses the host alone, since that syntax cannot carry a port.
- **Rewrite target `<GIT_URL>/`.** The `insteadOf` rules point at the base URL including any path, so `git@host:owner/repo.git` becomes `<GIT_URL>/owner/repo.git`.
- **Export the resolved `GIT_URL`.** A build script that names its repository, such as the static site example, writes `"$GIT_URL/owner/repo.git"` and follows the deployment's configuration.
- **Version 0.6.0.** New configuration is a minor bump; patch releases in this project have carried fixes only.

## Risks / Trade-offs

- [An `http` URL sends the token in cleartext] → The README says to use `http` only for an internal network the deployment trusts.
- [A typo in `GIT_URL` stops the container] → The error names `GIT_URL` and the value; an unrecognised scheme would otherwise produce a silent misconfiguration.
- [`GIT_SSH_HOST` with a port leaves SCP-style remotes on the default port rewritten too] → SCP-style remotes on that host are rewritten as before; this matches what the user means by that host in practice.

## Migration Plan

No migration. Deployments adopt it by setting `GIT_URL` (and `GIT_SSH_HOST` when the SSH domain differs) and recreating the hooked container. Rolling back to 0.5.0 ignores both variables and falls back to `GIT_HOST`.
