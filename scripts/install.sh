#!/usr/bin/env sh
set -eu

INSTALL_DIR="${OPENSTOCK_INSTALL_DIR:-$HOME/.local/bin}"
BIN_NAME="${OPENSTOCK_BIN_NAME:-openstock}"
ARCHIVE_URL="${OPENSTOCK_ARCHIVE_URL:-https://git.hananakick.cc/Autotrade/openstock/archive/main.tar.gz}"
SOURCE_DIR=""
TMP_DIR=""
TMP_TARGET=""

cleanup() {
  if [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ]; then
    rm -rf "$TMP_DIR"
  fi
  if [ -n "$TMP_TARGET" ] && [ -e "$TMP_TARGET" ]; then
    rm -f "$TMP_TARGET"
  fi
}
trap cleanup EXIT

is_openstock_source() {
  [ -f "$1/Cargo.toml" ] && grep -q 'name = "openstock"' "$1/Cargo.toml"
}

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" 2>/dev/null && pwd || pwd)"

if is_openstock_source "$PWD"; then
  SOURCE_DIR="$PWD"
elif is_openstock_source "$SCRIPT_DIR/.."; then
  SOURCE_DIR="$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)"
else
  if ! command -v curl >/dev/null 2>&1; then
    echo "curl is required when installing outside an openstock source tree" >&2
    exit 1
  fi
  if ! command -v tar >/dev/null 2>&1; then
    echo "tar is required when installing outside an openstock source tree" >&2
    exit 1
  fi

  TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/openstock-install.XXXXXX")"
  curl -fsSL "$ARCHIVE_URL" | tar -xz -C "$TMP_DIR"
  SOURCE_DIR="$(find "$TMP_DIR" -mindepth 1 -maxdepth 2 -name Cargo.toml -print | head -n 1 | xargs dirname)"

  if ! is_openstock_source "$SOURCE_DIR"; then
    echo "failed to locate openstock source in downloaded archive" >&2
    exit 1
  fi
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to build openstock" >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"

cd "$SOURCE_DIR"
cargo build --release
TMP_TARGET="$INSTALL_DIR/.$BIN_NAME.tmp.$$"
cp "$SOURCE_DIR/target/release/openstock" "$TMP_TARGET"
chmod 755 "$TMP_TARGET"
mv -f "$TMP_TARGET" "$INSTALL_DIR/$BIN_NAME"
TMP_TARGET=""

echo "installed: $INSTALL_DIR/$BIN_NAME"
echo "config: ${OPENSTOCK_CONFIG_DIR:-$HOME/.config/openstock}"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo "warning: $INSTALL_DIR is not in PATH" >&2
    echo "add this to your shell profile: export PATH=\"$INSTALL_DIR:\$PATH\"" >&2
    ;;
esac
