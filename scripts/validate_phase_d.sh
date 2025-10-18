#!/usr/bin/env bash
set -euo pipefail

# Phase D validation script: shim status/deploy/resolution basics

BIN="${BIN:-./target/debug/substrate}"

echo "== Phase D: validate --shim-status, redeploy, PATH checks =="

if [[ ! -x "$BIN" ]]; then
  echo "error: substrate binary not found at $BIN" >&2
  exit 2
fi

echo "-- Baseline status --"
set +e
"$BIN" --shim-status
RC=$?
set -e
echo "RC=$RC"

echo
echo "-- Simulate drift (remove one shim) --"
SHIMS_DIR="$HOME/.substrate/shims"
if [[ -d "$SHIMS_DIR" ]]; then
  rm -f "$SHIMS_DIR/npm" || true
else
  echo "shims dir not found; attempting deploy first"
  "$BIN" --shim-deploy || true
fi

set +e
"$BIN" --shim-status
RC=$?
set -e
echo "RC=$RC (expect 1 and \"Needs redeploy\")"

echo
echo "-- Force redeploy --"
"$BIN" --shim-deploy || true

set +e
"$BIN" --shim-status
RC=$?
set -e
echo "RC=$RC (expect 0)"

echo
echo "-- PATH misorder (prepend /usr/bin) --"
export PATH="/usr/bin:$PATH"
set +e
"$BIN" --shim-status
RC=$?
set -e
echo "RC=$RC (expect 0 with PATH: WARN)"

echo
echo "Done."

