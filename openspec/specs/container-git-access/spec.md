# container-git-access Specification

## Purpose

Defines how the hooked Docker image gives build scripts authenticated git access over HTTP or HTTPS to the configured git base URL.

## Requirements

### Requirement: Configurable git host
The container SHALL configure git for the base URL in `GIT_URL`, which consists of an `http` or `https` scheme, a host, an optional port, and an optional path. When `GIT_URL` is unset or empty, the container SHALL use `https://<GIT_HOST>`, with `GIT_HOST` defaulting to `github.com` when it is unset or empty. The container SHALL export the resolved `GIT_URL`, without a trailing slash, to the build script. When `GIT_URL` has any other scheme, the container SHALL print an error and exit without starting hooked.

#### Scenario: Default host
- **WHEN** the container starts without `GIT_URL` and `GIT_HOST`
- **THEN** git in the build script uses the credential and URL rules for `https://github.com`, and `GIT_URL` is `https://github.com`

#### Scenario: Gitea host
- **WHEN** the container starts with `GIT_HOST=git.example.com` and without `GIT_URL`
- **THEN** git in the build script uses the credential and URL rules for `https://git.example.com`

#### Scenario: Internal URL
- **WHEN** the container starts with `GIT_URL=http://server:3000/`
- **THEN** git in the build script uses the credential and URL rules for `http://server:3000`, and `GIT_URL` is `http://server:3000`

#### Scenario: GIT_URL takes precedence
- **WHEN** the container starts with both `GIT_URL=http://server:3000` and `GIT_HOST=git.example.com`
- **THEN** git in the build script uses the rules for `http://server:3000`

#### Scenario: Unsupported scheme
- **WHEN** the container starts with `GIT_URL=ssh://git.example.com`
- **THEN** the container prints an error that names `GIT_URL` and exits with a non-zero status

### Requirement: SSH remotes are rewritten to the base URL
The container SHALL rewrite `ssh://git@<ssh-host>/`, `git+ssh://git@<ssh-host>/`, and `git@<ssh-hostname>:` remote URLs to `<GIT_URL>/`, where `<ssh-host>` is `GIT_SSH_HOST` (a host with an optional `:port`), defaulting to the host of `GIT_URL` without its port, and `<ssh-hostname>` is `<ssh-host>` without its port.

#### Scenario: SCP-style remote
- **WHEN** a repository's remote is `git@git.example.com:owner/repo.git` and `GIT_HOST=git.example.com`
- **THEN** git fetches from `https://git.example.com/owner/repo.git`

#### Scenario: Separate SSH host
- **WHEN** a repository's remote is `git@git-ssh.example.com:owner/repo.git`, `GIT_URL=https://git.example.com`, and `GIT_SSH_HOST=git-ssh.example.com`
- **THEN** git fetches from `https://git.example.com/owner/repo.git`

#### Scenario: SSH port
- **WHEN** a repository's remote is `ssh://git@git-ssh.example.com:2222/owner/repo.git`, `GIT_URL=http://server:3000`, and `GIT_SSH_HOST=git-ssh.example.com:2222`
- **THEN** git fetches from `http://server:3000/owner/repo.git`

### Requirement: Token from a Docker secret
When `/run/secrets/git_token` exists and is not empty, the container SHALL store it as the credential for the scheme, host, and port of `GIT_URL`. When it does not exist or is empty, the container SHALL store no credential, print a warning, and still start. The container SHALL NOT read `/run/secrets/github_pat`.

#### Scenario: Token present
- **WHEN** `/run/secrets/git_token` contains a token and `GIT_HOST=git.example.com`
- **THEN** git authenticates to `https://git.example.com` with that token

#### Scenario: Token for an internal URL
- **WHEN** `/run/secrets/git_token` contains a token and `GIT_URL=http://server:3000`
- **THEN** git authenticates to `http://server:3000` with that token, and does not send it to `https://server`

#### Scenario: Token missing
- **WHEN** `/run/secrets/git_token` does not exist
- **THEN** the container prints a warning and starts hooked

#### Scenario: Token empty
- **WHEN** `/run/secrets/git_token` exists but is empty
- **THEN** the container stores no credential, prints a warning, and starts hooked
