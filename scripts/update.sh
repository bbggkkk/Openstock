#!/usr/bin/env sh
set -eu

SCRIPT_URL="${OPENSTOCK_INSTALL_SCRIPT_URL:-https://git.hananakick.cc/Autotrade/openstock/raw/branch/main/scripts/install.sh}"

curl -fsSL "$SCRIPT_URL" | sh
