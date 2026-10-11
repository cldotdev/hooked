## Context

The sibling project `feedgen` already uses this release flow: a hand-pushed annotated tag triggers `release.yml`, which logs in to GHCR with `GITHUB_TOKEN`, derives image tags with `docker/metadata-action`, pushes with `docker/build-push-action`, and runs `gh release create --notes-from-tag`. hooked's Dockerfile compiles a Rust release build from scratch, including `aws-lc-sys`, which takes far longer than feedgen's Go build.

## Goals / Non-Goals

**Goals:**
- Every push to `master` and every pull request is checked for formatting, clippy warnings, and test failures.
- A pushed version tag produces the image and the GitHub Release without manual steps.

**Non-Goals:**
- Creating tags automatically; tags and their annotations stay manual.
- Multi-architecture images; only `linux/amd64` is built, since an emulated arm64 Rust build would multiply the release time.
- Running the test suite again in the release workflow; the tag points at a commit that already went through CI on `master`.

## Decisions

- **Follow feedgen's release workflow**, with the same action major versions (`actions/checkout@v7`, `docker/login-action@v4`, `docker/metadata-action@v6`, `docker/build-push-action@v7`), the same tag filter (`[0-9]+.[0-9]+.[0-9]+`, so a prerelease such as `1.0.0-rc1` does not ship), and the same `gh release create --verify-tag --notes-from-tag --title` step with the tag fetch it depends on.
- **Image `ghcr.io/${{ github.repository }}`**, which resolves to `ghcr.io/cldotdev/hooked`. `GITHUB_TOKEN` with `packages: write` can push there, whereas pushing to the personal `jlhg` namespace would need a separate token.
- **`latest` from metadata-action's default `flavor: latest=auto`**, which adds `latest` to every non-prerelease semver tag. `compose.yaml` keeps pulling `:latest`.
- **Fail the release when the tag differs from `Cargo.toml`'s `version`**, before anything is pushed, so `hooked --version` always matches the image tag.
- **Docker layer cache in GitHub Actions** (`cache-from`/`cache-to: type=gha,mode=max`, which needs `docker/setup-buildx-action@v4`), so a release whose dependencies did not change skips most of the Rust compilation.
- **CI toolchain taken from the `Dockerfile`'s `FROM rust:<x.y.z>-alpine` line** and installed with the runner's `rustup`, so a clippy lint that ships in a newer stable release cannot break CI before the Dockerfile moves to it. The `Dockerfile` stays the single place that names the Rust version. `Swatinem/rust-cache@v2` caches the cargo registry and `target/`.
- **Job timeouts of 20 minutes for CI and 60 for release**, above feedgen's 10 and 20, because a cold Rust build with `aws-lc-sys` takes far longer than a Go build.
- **CI permissions `contents: read` and a per-ref concurrency group that cancels superseded runs**, as in feedgen.

## Risks / Trade-offs

- [The workflows cannot be run locally; `actionlint` is not installed] → The YAML is checked for syntax locally, and the first push to `master` and the first tag exercise them.
- [The GHCR package is created private on its first push] → Its visibility has to be set to public once in the package settings; the README notes this for maintainers.
- [Existing deployments keep pulling the frozen `jlhg` image] → The README Migration section names the new image.
