#!/bin/sh
# The repository supplies only the Hugo source. The build runs in a throwaway
# container without network access or host mounts beyond its input and output,
# so a malicious push can change the site's pages but not reach the host.
set -ex

commit_id=$1

# GIT_URL is the base URL the entrypoint sets up credentials for.
repo_url="$GIT_URL/<owner>/<repo>.git"
branch=$WATCH_PUSH_BRANCH
hugo_image=ghcr.io/gohugoio/hugo:v0.167.0
# Mounted at the same paths in this container, because `docker run -v` takes
# host paths.
base_dir=/path/to/site
build_dir=$base_dir/.srv/build
site_dir=$base_dir/.srv/site
keep_releases=3

# commit_id names directories that get removed below.
case "$commit_id" in
  '' | *[!0-9a-f]*) echo "invalid commit ID: $commit_id" >&2; exit 1 ;;
esac

[ -d repo.git ] || git init -q --bare repo.git
# Fetching into a ref lets the next fetch send only new objects.
git -C repo.git fetch "$repo_url" "+refs/heads/$branch:refs/heads/$branch"

src=$build_dir/$commit_id
out=$site_dir/releases/$commit_id
trap 'rm -rf "$src"' EXIT
rm -rf "$src" "$out"
mkdir -p "$src" "$out"
git -C repo.git archive "$commit_id" | tar -x -C "$src"

# Add `-e TZ=<zone>` if the site's dates depend on the time zone.
docker run --rm --network none --read-only --tmpfs /tmp \
  --user "$(id -u):$(id -g)" \
  -e HUGO_CACHEDIR=/tmp/hugo_cache \
  -v "$src:/src:ro" -v "$out:/out" \
  "$hugo_image" --source /src --destination /out --minify --noBuildLock

# rename(2) swaps the link atomically; ln -sfn alone unlinks first.
ln -sfn "releases/$commit_id" "$site_dir/current.tmp"
mv -fT "$site_dir/current.tmp" "$site_dir/current"

ls -1t "$site_dir/releases" | tail -n +$((keep_releases + 1)) | while read -r old; do
  rm -rf "${site_dir:?}/releases/$old"
done
