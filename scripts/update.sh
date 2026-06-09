#!/usr/bin/env sh
set -eu

BIN_NAME="${OPENSTOCK_BIN_NAME:-openstock}"

if ! command -v "$BIN_NAME" >/dev/null 2>&1; then
  echo "$BIN_NAME is required for update. Install first with scripts/install.sh." >&2
  exit 1
fi

"$BIN_NAME" update "$@"
