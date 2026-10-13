#!/bin/sh
set -e

# GIT_URL takes precedence over GIT_HOST, which stays as a shorthand for https://<host>.
GIT_URL="${GIT_URL:-https://${GIT_HOST:-github.com}}"
while [ "${GIT_URL%/}" != "$GIT_URL" ]; do GIT_URL="${GIT_URL%/}"; done
case "$GIT_URL" in
  http://* | https://*) ;;
  *)
    echo "Error: GIT_URL must start with http:// or https://, got: ${GIT_URL}" >&2
    exit 1
    ;;
esac
export GIT_URL

git_scheme="${GIT_URL%%://*}"
git_authority="${GIT_URL#*://}"
git_authority="${git_authority%%/*}"
git_host="${git_authority%%:*}"
# GIT_SSH_HOST may carry a port; the SCP-style syntax cannot, so it gets the host alone.
GIT_SSH_HOST="${GIT_SSH_HOST:-$git_host}"
ssh_hostname="${GIT_SSH_HOST%%:*}"

# Append to any GIT_CONFIG_* entries the caller already exported.
n="${GIT_CONFIG_COUNT:-0}"
add_git_config() {
  export "GIT_CONFIG_KEY_${n}=$1" "GIT_CONFIG_VALUE_${n}=$2"
  n=$((n + 1))
}

add_git_config credential.helper store
for prefix in "ssh://git@${GIT_SSH_HOST}/" "git+ssh://git@${GIT_SSH_HOST}/" "git@${ssh_hostname}:"; do
  add_git_config "url.${GIT_URL}/.insteadOf" "$prefix"
done
export GIT_CONFIG_COUNT="$n"

if [ -f /run/secrets/git_token ]; then
  GIT_TOKEN=$(cat /run/secrets/git_token)
  (umask 077 && echo "${git_scheme}://token:${GIT_TOKEN}@${git_authority}" > ~/.git-credentials)
  chmod 600 ~/.git-credentials
else
  echo "Warning: No git token found at /run/secrets/git_token" >&2
fi

exec "$@"
