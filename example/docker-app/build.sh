#!/bin/sh
# The repository supplies only a Dockerfile and source. Every runtime setting
# (secrets, volumes, networks, ports) comes from app/compose.yaml on the host,
# so a malicious push can change what the build runs but not what the host
# grants it.
set -ex

commit_id=$1

# GIT_URL is the base URL the entrypoint sets up credentials for.
repo_url="$GIT_URL/<owner>/<repo>.git"
branch=$WATCH_PUSH_BRANCH
# Mounted at the same path in this container; see compose.yaml.
deploy_dir=/path/to/deploy
compose_file=$deploy_dir/app/compose.yaml
src=$deploy_dir/src

# Accept only a hex ID, so git archive cannot read it as an option.
case "$commit_id" in
  '' | *[!0-9a-f]*) echo "invalid commit ID: $commit_id" >&2; exit 1 ;;
esac

[ -d repo.git ] || git init -q --bare repo.git
# Fetching into a ref lets the next fetch send only new objects.
git -C repo.git fetch "$repo_url" "+refs/heads/$branch:refs/heads/$branch"

# Builds run one at a time, so a fixed directory is safe.
rm -rf "$src"
mkdir -p "$src"
git -C repo.git archive "$commit_id" | tar -x -C "$src"

# Builds run one at a time, so without a timeout, a start that never settles
# would hold up every later push.
docker compose -f "$compose_file" up -d --build --remove-orphans \
  --wait --wait-timeout 300

# Bound disk use: the daemon-wide build cache, then this project's untagged
# images.
project=$(docker compose -f "$compose_file" config | sed -n 's/^name: //p')
docker builder prune -f --max-used-space 10GB
docker image prune -f --filter "label=com.docker.compose.project=$project"
