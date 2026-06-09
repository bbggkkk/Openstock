#!/usr/bin/env sh
set -eu

INSTALL_DIR="${OPENSTOCK_INSTALL_DIR:-$HOME/.local/bin}"
BIN_NAME="${OPENSTOCK_BIN_NAME:-openstock}"
TARGET="$INSTALL_DIR/$BIN_NAME"

if [ -e "$TARGET" ]; then
  rm -f "$TARGET"
  echo "removed: $TARGET"
else
  echo "not installed: $TARGET"
fi

echo "config and cache are preserved at: ${OPENSTOCK_CONFIG_DIR:-$HOME/.config/openstock}"
echo "remove them manually only if you no longer need credentials or cached data."
