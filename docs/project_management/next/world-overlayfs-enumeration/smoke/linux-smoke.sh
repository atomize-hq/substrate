#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: overlayfs enumeration smoke (not Linux)"
  exit 0
fi

need_cmd() { command -v "$1" >/dev/null 2>&1 || { echo "FAIL: missing $1" >&2; exit 3; }; }
need_cmd substrate
need_cmd rg
need_cmd mktemp

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

substrate --world -c 'touch a.txt; ls -a' | rg -n -- '^a\.txt$' >/dev/null
echo "OK: overlayfs enumeration smoke"

