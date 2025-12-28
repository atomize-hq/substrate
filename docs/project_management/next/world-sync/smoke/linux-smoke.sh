#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: world-sync linux smoke (not Linux)"
  exit 0
fi

if ! command -v substrate >/dev/null 2>&1; then
  echo "FAIL: substrate not found on PATH" >&2
  exit 1
fi

if ! substrate --help 2>/dev/null | grep -q "sync"; then
  echo "SKIP: world-sync smoke (substrate sync not available in this build)"
  exit 0
fi

WS_TEST_WS="$(mktemp -d)"
WS_TEST_NO_INIT="$(mktemp -d)"
cleanup() { rm -rf "$WS_TEST_WS" "$WS_TEST_NO_INIT"; }
trap cleanup EXIT

cd "$WS_TEST_WS"
git init -q
substrate init >/dev/null
test -d .substrate
test -d .substrate-git
test -d .substrate-git/repo.git

substrate sync --dry-run >/dev/null

cd "$WS_TEST_NO_INIT"
set +e
substrate sync >/dev/null 2>&1; test $? -eq 2
substrate checkpoint >/dev/null 2>&1; test $? -eq 2
substrate rollback last >/dev/null 2>&1; test $? -eq 2
set -e

echo "OK: world-sync linux smoke (core gating)"
