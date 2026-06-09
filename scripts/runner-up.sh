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

if grep -q 'replace-with-.*-runner-token' "$ENV_FILE"; then
  echo "missing GITEA_RUNNER_REGISTRATION_TOKEN in $ENV_FILE" >&2
  exit 1
fi

INSTANCE_URL="$(sed -n 's/^GITEA_INSTANCE_URL=//p' "$ENV_FILE" | tail -n 1)"
if [ -z "$INSTANCE_URL" ] || [ "$INSTANCE_URL" = "https://your-gitea.example.com" ]; then
  echo "missing GITEA_INSTANCE_URL in $ENV_FILE" >&2
  exit 1
fi

if command -v curl >/dev/null 2>&1; then
  version_url="${INSTANCE_URL%/}/api/v1/version"
  if ! curl -fsS "$version_url" >/dev/null 2>&1; then
    echo "GITEA_INSTANCE_URL does not look reachable as a Gitea API endpoint: $version_url" >&2
    echo "Use the exact Gitea web URL, not only the SSH host name." >&2
    exit 1
  fi
fi

cd "$RUNNER_DIR"
docker compose up -d
docker compose ps

echo
echo "If logs repeat 'permission_denied: 403 Forbidden', refresh the instance runner registration token in Gitea and update:"
echo "$ENV_FILE"
echo "Then restart with: docker compose down && docker compose up -d"
