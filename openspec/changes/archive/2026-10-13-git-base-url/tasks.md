## 1. Entrypoint

- [x] 1.1 In `entrypoint.sh`, resolve `GIT_URL` (default `https://${GIT_HOST:-github.com}`), strip trailing slashes, reject schemes other than `http` and `https` with an error naming `GIT_URL`, and export the result
- [x] 1.2 Derive the authority and host from `GIT_URL`, resolve `GIT_SSH_HOST` (default the host), and emit the `insteadOf` rules to `$GIT_URL/` for `ssh://git@<ssh-host>/`, `git+ssh://git@<ssh-host>/`, and `git@<ssh-hostname>:`
- [x] 1.3 Write the credential store entry as `<scheme>://token:<token>@<authority>`

## 2. Configuration, Examples, and Docs

- [x] 2.1 In `compose.yaml` and `example/static-site/compose.yaml`, show `GIT_URL` and `GIT_SSH_HOST` next to `GIT_HOST`, commented out
- [x] 2.2 In `example/static-site/build.sh`, build the repository URL from `$GIT_URL`
- [x] 2.3 In the README Docker section, document `GIT_URL`, `GIT_SSH_HOST`, how they relate to `GIT_HOST`, and that `http` belongs on a trusted internal network only
- [x] 2.4 Change the Purpose of `openspec/specs/container-git-access/spec.md` from HTTPS-only to HTTP or HTTPS
- [x] 2.5 Set `version = "0.6.0"` in `Cargo.toml` and run `cargo build` so `Cargo.lock` follows

## 3. Verification

- [x] 3.1 Build the image locally and, for each scenario in the delta spec, run a container with the scenario's environment and a fake token at `/run/secrets/git_token`; check rewrites with `git ls-remote --get-url <remote>`, credentials with `git credential fill` (with `GIT_TERMINAL_PROMPT=0`), and the exported value with `echo "$GIT_URL"`
- [x] 3.2 Confirm `GIT_URL=ssh://git.example.com` exits non-zero with an error naming `GIT_URL`, and that a container without the token secret prints the warning and starts
- [x] 3.3 Run `sh -n` on `entrypoint.sh`, and `shellcheck` if it is installed
- [x] 3.4 Remove the test image, containers, and temporary files
