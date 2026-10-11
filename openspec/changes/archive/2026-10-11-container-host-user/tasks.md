## 1. Image and Compose

- [x] 1.1 In the `Dockerfile` runtime stage, create `/home/hooked` with mode 1777 and set `ENV HOME=/home/hooked`
- [x] 1.2 Add `user: "${HOST_UID:-0}:${HOST_GID:-0}"` to the `hooked` service in `compose.yaml`

## 2. Verification

- [x] 2.1 Build the image and, with `--user 1000:1000` and a fake token mounted at `/run/secrets/git_token`, confirm that `$HOME/.git-credentials` is written and owned by 1000, that `docker buildx ls` succeeds and creates `$HOME/.docker`, and that `hooked --help` runs
- [x] 2.2 With `--user 1000:1000` and a bind-mounted temporary directory as `/app`, run `hooked` with a log path inside it long enough to create the log file, and confirm that the file is owned by 1000 on the host
- [x] 2.3 Confirm that the default user is still root (`id -u` prints 0) without `--user`, and run `docker compose config` with and without `HOST_UID`/`HOST_GID` to confirm that the rendered `user:` is `1000:1000` and `0:0` respectively
- [x] 2.4 Remove the test image and temporary directories
