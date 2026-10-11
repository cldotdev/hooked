## Why

Nothing checks a push or pull request, and the Docker image and GitHub Releases are published by hand. The compose file also still pulls `ghcr.io/jlhg/hooked`, while the repository now lives at `cldotdev/hooked`.

## What Changes

- Add a CI workflow for pushes to `master` and pull requests: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.
- Add a release workflow triggered by pushing a tag `X.Y.Z`: check that the tag matches the `Cargo.toml` version, build the image for `linux/amd64`, push it to `ghcr.io/cldotdev/hooked` as `X.Y.Z` and `latest`, and create the GitHub Release from the tag annotation. Tags are still created by hand.
- Fix the existing clippy warnings in `src/crypto.rs` so the CI gate starts green.
- Point `compose.yaml` and the README at `cldotdev/hooked` (`ghcr.io/cldotdev/hooked:latest`, clone URL), and document the release steps in the README.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This change adds tooling and does not alter hooked's behavior, so it sets `skip_specs: true`.

## Impact

- New `.github/workflows/ci.yml` and `.github/workflows/release.yml`
- `src/crypto.rs` (test assertions only), `compose.yaml`, README
- Deployments that pull `ghcr.io/jlhg/hooked:latest` stop receiving updates and must switch to `ghcr.io/cldotdev/hooked:latest`
