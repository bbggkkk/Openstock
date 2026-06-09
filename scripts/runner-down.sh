#!/usr/bin/env sh
set -eu

RUNNER_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/../ops/gitea-runner" && pwd)"

cd "$RUNNER_DIR"
docker compose down
