## 1. Example

- [x] 1.1 In `example/docker-app/build.sh`, add `cd "$deploy_dir"` after the `git archive` step and before `docker compose ... up`, with a comment saying that buildx bake allows the compose file to read only files under the working directory
- [x] 1.2 Run `sh -n` and, if installed, `shellcheck` on `example/docker-app/build.sh`

## 2. Documentation

- [x] 2.1 In `example/docker-app/README.md`, state that build secrets and any other files `app/compose.yaml` reads must be under `deploy_dir` (next to the existing same-path rule and in Deployment Steps step 6), and add a short subsection on the check: the error message, that `build.sh` runs compose from `deploy_dir`, that `COMPOSE_BAKE=false` and `--project-directory` do not help, and `BUILDX_BAKE_ENTITLEMENTS_FS=0` on the `hooked` service as the escape hatch with its trade-off
- [x] 2.2 In `README.md`, add a Migration entry for images built with Compose v5: the symptom, the `cd "$deploy_dir"` fix for docker-app style scripts, and the escape hatch, linking to the example README for details
- [x] 2.3 In `example/docker-app/README.md`, keep the same-path sentence about bind mounts and secret files separate from the `deploy_dir` rule, which applies to build secrets

## 3. Verification

- [x] 3.1 Build a local image with the hooked image's runtime layers (`FROM docker:latest` plus git, without the hooked binary, which the script does not use) and, with the host socket mounted and a deploy directory mounted at the same path, run a copy of `example/docker-app/build.sh` whose `repo_url` points at a local bare repository containing a minimal Dockerfile that uses a build secret, and whose `app/compose.yaml` declares that secret from `<deploy_dir>/.secrets/`: confirm the build succeeds, and that the same script without the `cd` fails with the `--allow=fs.read` message (the test copy drops `--wait`, since the test app has no healthcheck, and the prune commands, so the host's build cache and images stay untouched)
- [x] 3.2 Remove the test image, containers, compose project, and temporary directories
