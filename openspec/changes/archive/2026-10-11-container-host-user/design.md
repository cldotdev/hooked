## Context

hooked reaches Docker through `DOCKER_HOST=tcp://docker-socket-proxy:2375`, not a mounted socket, so a non-root process needs no socket group membership. Git configuration already comes from `GIT_CONFIG_*` variables exported by the entrypoint, so nothing writes to `/root` or `/etc`. Two things still assume root: the entrypoint writes `~/.git-credentials`, and the Docker CLI writes `~/.docker/`. For a UID without a passwd entry, `HOME` falls back to `/`, which is not writable.

## Goals / Non-Goals

**Goals:**
- Files written into bind mounts are owned by the host user when the deployment asks for it.
- Root stays the default, so deployments that change nothing keep working.

**Non-Goals:**
- Changing the ownership of existing files automatically; the README documents a one-time `chown`.
- Ownership of files that deployed services write into their own volumes; those containers choose their own users.
- Adding a passwd entry for the runtime UID.

## Decisions

- **`user:` in compose rather than a root entrypoint that drops privileges.** `user: "${HOST_UID:-0}:${HOST_GID:-0}"` needs no extra package and no privilege handling in the entrypoint. The privilege-drop alternative (`PUID`/`PGID` plus `su-exec`) could `chown` the log directory automatically, but adds a package and root-time logic for a one-time migration step, which a documented `chown` covers instead.
- **`HOST_UID`/`HOST_GID` instead of `UID`/`GID`.** Bash's `UID` is a shell variable that is not exported, so compose would not see it; distinct names set in `.env` avoid that trap.
- **`ENV HOME=/home/hooked` with mode 1777.** The sticky bit keeps the directory shared but stops one UID from removing another's files, and the same path works for root. The entrypoint's `umask 077` and `chmod 600` keep `.git-credentials` private.

## Risks / Trade-offs

- [Switching to a non-root UID with a root-owned `log/` makes hooked exit at startup] → README Migration gives the `chown` command.
- [Tools that look up the current user in `/etc/passwd`, such as `ssh` or `whoami`, fail for a UID without an entry] → Git uses HTTPS through the `insteadOf` rules, so builds do not need `ssh`.
- [Secret files must be readable by the runtime UID] → Compose bind-mounts file-based secrets with their host permissions, and the host user owns `.secrets/`.

## Migration Plan

To adopt it, stop the stack, `chown` the bind-mounted directories to the host user, set `HOST_UID` and `HOST_GID` in `.env`, and start the stack. To roll back, remove the two variables; root can still write the re-owned files.
