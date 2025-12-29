#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

case "$(uname -s)" in
  Linux)
    exec "$SCRIPT_DIR/linux-smoke.sh"
    ;;
  Darwin)
    exec "$SCRIPT_DIR/macos-smoke.sh"
    ;;
  *)
    echo "SKIP: p0 platform stability smoke (unsupported uname: $(uname -s))"
    exit 0
    ;;
esac

