#!/usr/bin/env sh
set -eu

RUNNER_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/../ops/gitea-runner" && pwd)"
ENV_FILE="$RUNNER_DIR/.env"

if [ ! -f "$ENV_FILE" ]; then
  cp "$RUNNER_DIR/.env.example" "$ENV_FILE"
  echo "created: $ENV_FILE" >&2
  echo "edit GITEA_RUNNER_REGISTRATION_TOKEN before starting the runner" >&2
  exit 1
fi

if grep -q 'replace-with-repository-runner-token' "$ENV_FILE"; then
  echo "missing GITEA_RUNNER_REGISTRATION_TOKEN in $ENV_FILE" >&2
  exit 1
fi

cd "$RUNNER_DIR"
docker compose up -d
docker compose ps

echo
echo "If logs repeat 'permission_denied: 403 Forbidden', refresh the repository runner registration token in Gitea and update:"
echo "$ENV_FILE"
echo "Then restart with: docker compose down && docker compose up -d"
