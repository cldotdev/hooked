# container-runtime-user Specification

## Purpose

Defines which user the hooked container runs as and what the image provides so that hooked and build scripts work under an arbitrary UID and GID.

## Requirements

### Requirement: Configurable runtime user in compose
The compose configuration SHALL run hooked as the UID and GID given in `HOST_UID` and `HOST_GID`, and as root (`0:0`) when they are unset.

#### Scenario: Host user configured
- **WHEN** `.env` sets `HOST_UID=1000` and `HOST_GID=1000`
- **THEN** hooked and the build script run as UID 1000 and GID 1000, and files they create in bind mounts are owned by `1000:1000` on the host

#### Scenario: Not configured
- **WHEN** neither `HOST_UID` nor `HOST_GID` is set
- **THEN** hooked runs as root, as before

### Requirement: Writable home for any UID
The image SHALL set `HOME` to a directory that any UID can write to, so the entrypoint and the build script work for a UID that has no passwd entry in the image.

#### Scenario: Git credentials as a non-root UID
- **WHEN** the container runs as UID 1000 with `/run/secrets/git_token` present
- **THEN** the entrypoint writes `$HOME/.git-credentials` and starts hooked

#### Scenario: Docker CLI as a non-root UID
- **WHEN** the build script runs a Docker CLI command that writes client state, such as `docker buildx ls`
- **THEN** the command succeeds and its state is stored under `$HOME/.docker`
