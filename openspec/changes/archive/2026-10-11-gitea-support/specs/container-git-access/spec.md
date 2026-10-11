## Purpose

Defines how the hooked Docker image gives build scripts authenticated git access over HTTPS to the configured git host.

## ADDED Requirements

### Requirement: Configurable git host
The container SHALL configure git for the host named in `GIT_HOST`, defaulting to `github.com` when it is unset or empty.

#### Scenario: Default host
- **WHEN** the container starts without `GIT_HOST`
- **THEN** git in the build script uses the credential and URL rules for `github.com`

#### Scenario: Gitea host
- **WHEN** the container starts with `GIT_HOST=git.example.com`
- **THEN** git in the build script uses the credential and URL rules for `git.example.com`

### Requirement: SSH remotes are rewritten to HTTPS
The container SHALL rewrite `ssh://git@<host>/`, `git@<host>:`, and `git+ssh://git@<host>/` remote URLs to `https://<host>/` for the configured host.

#### Scenario: SCP-style remote
- **WHEN** a repository's remote is `git@git.example.com:owner/repo.git` and `GIT_HOST=git.example.com`
- **THEN** git fetches from `https://git.example.com/owner/repo.git`

### Requirement: Token from a Docker secret
When `/run/secrets/git_token` exists, the container SHALL store it as the HTTPS credential for the configured host. When it does not exist, the container SHALL print a warning and still start. The container SHALL NOT read `/run/secrets/github_pat`.

#### Scenario: Token present
- **WHEN** `/run/secrets/git_token` contains a token and `GIT_HOST=git.example.com`
- **THEN** git authenticates to `https://git.example.com` with that token

#### Scenario: Token missing
- **WHEN** `/run/secrets/git_token` does not exist
- **THEN** the container prints a warning and starts hooked
