#!/bin/sh
set -e

GIT_HOST="${GIT_HOST:-github.com}"

# Append to any GIT_CONFIG_* entries the caller already exported.
n="${GIT_CONFIG_COUNT:-0}"
add_git_config() {
  export "GIT_CONFIG_KEY_${n}=$1" "GIT_CONFIG_VALUE_${n}=$2"
  n=$((n + 1))
}

add_git_config credential.helper store
for prefix in "ssh://git@${GIT_HOST}/" "git@${GIT_HOST}:" "git+ssh://git@${GIT_HOST}/"; do
  add_git_config "url.https://${GIT_HOST}/.insteadOf" "$prefix"
done
export GIT_CONFIG_COUNT="$n"

if [ -f /run/secrets/git_token ]; then
  GIT_TOKEN=$(cat /run/secrets/git_token)
  (umask 077 && echo "https://token:${GIT_TOKEN}@${GIT_HOST}" > ~/.git-credentials)
  chmod 600 ~/.git-credentials
else
  echo "Warning: No git token found at /run/secrets/git_token" >&2
fi

exec "$@"
