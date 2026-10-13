## MODIFIED Requirements

### Requirement: Configurable runtime user in compose
The compose configuration SHALL run hooked as the UID and GID given in `HOST_UID` and `HOST_GID`, and as root (`0:0`) when they are unset.

#### Scenario: Host user configured
- **WHEN** `.env` sets `HOST_UID=1000` and `HOST_GID=1000`
- **THEN** hooked and the build script run as UID 1000 and GID 1000, and files they create in bind mounts are owned by `1000:1000` on the host

#### Scenario: Not configured
- **WHEN** neither `HOST_UID` nor `HOST_GID` is set, and `DOCKER_GID` is set
- **THEN** hooked runs as root

## ADDED Requirements

### Requirement: Docker access through the host socket
The compose configuration SHALL give hooked Docker access by mounting the host's `/var/run/docker.sock` and adding the group ID in `DOCKER_GID` to the container's supplementary groups. It SHALL refuse to start when `DOCKER_GID` is unset, whichever user hooked runs as.

#### Scenario: Non-root user with the docker group
- **WHEN** `.env` sets `HOST_UID`, `HOST_GID`, and `DOCKER_GID` to the host docker group ID
- **THEN** a Docker CLI command in the build script reaches the host daemon

#### Scenario: Docker group ID missing
- **WHEN** `DOCKER_GID` is unset
- **THEN** `docker compose` fails before starting any container, with a message that names `DOCKER_GID`

#### Scenario: Builds use the daemon's builder
- **WHEN** the build script runs `docker compose build` or `docker buildx build`
- **THEN** buildx uses the host daemon's default builder, and no `buildx_buildkit_*` container is created
