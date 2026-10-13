#!/bin/sh
set -ex

commit_id=$1

# # Put your build and deployment commands here.
# # Example of the docker deployment:
# cd /path/to/repo
# git fetch
# git checkout $commit_id
#
# # The script's stdout and stderr go to Discord and the log, so turn tracing
# # off around anything that expands a secret.
# set +x
# api_key=$(cat /run/secrets/api_key)
# set -x
#
# # Only services whose configuration or image changed are recreated, so
# # unchanged ones, such as a tunnel, keep their connections.
# docker compose --progress plain up -d --build
#
# # The builder's default cache limit scales with the disk size, and an
# # unbounded prune also drops base image layers, so every build pulls them
# # again. Keep the cache under a fixed size instead.
# docker builder prune -f --max-used-space 10GB
#
# # Only dangling images that compose built for this project; an unfiltered
# # prune removes those of every stack on the host.
# docker image prune -f --filter label=com.docker.compose.project=<name>
