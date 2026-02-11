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

SLICE_ID="${SUBSTRATE_SMOKE_SLICE_ID:-WS7}"

should_skip_for_output() {
  local out="$1"
  printf '%s' "$out" | grep -Fq "pending diff discovery is unsupported by this backend" && return 0
  printf '%s' "$out" | grep -Fq "unknown field `caged_required`" && return 0
  printf '%s' "$out" | grep -Fq "workspace sync requires world" && return 0
  return 1
}

run_or_skip() {
  local label="$1"
  shift

  set +e
  local out
  out="$("$@" 2>&1)"
  local rc=$?
  set -e

  if [[ $rc -ne 0 ]]; then
    if should_skip_for_output "$out"; then
      echo "SKIP: world-sync linux smoke ($SLICE_ID): missing/old world backend prerequisites ($label)"
      printf '%s\n' "$out"
      exit 0
    fi
    echo "FAIL: world-sync linux smoke ($SLICE_ID): $label (exit=$rc)" >&2
    printf '%s\n' "$out" >&2
    exit "$rc"
  fi

  printf '%s' "$out"
}

WS_DIR="$(mktemp -d)"
cleanup() { rm -rf "$WS_DIR"; }
trap cleanup EXIT

cd "$WS_DIR"
substrate workspace init . >/dev/null

case "$SLICE_ID" in
  WS2)
    run_or_skip "world exec" substrate --world -c "sh -lc 'echo hello > hello-from-world.txt'" >/dev/null
    set +e
    out="$(substrate workspace sync --direction from_world --verbose 2>&1)"
    rc=$?
    set -e
    if [[ $rc -ne 0 ]]; then
      if should_skip_for_output "$out"; then
        echo "SKIP: world-sync linux smoke ($SLICE_ID): missing/old world backend prerequisites (workspace sync --verbose)"
        printf '%s\n' "$out"
        exit 0
      fi
      echo "FAIL: world-sync linux smoke ($SLICE_ID): workspace sync --verbose (exit=$rc)" >&2
      printf '%s\n' "$out" >&2
      exit "$rc"
    fi
    printf '%s' "$out" | grep -Fq "hello-from-world.txt"
    test -f hello-from-world.txt
    run_or_skip "workspace sync apply" substrate workspace sync --direction from_world >/dev/null
    echo "OK: world-sync linux smoke ($SLICE_ID)"
    ;;
  WS5)
    echo "host" > host-only.txt
    substrate workspace sync --dry-run --direction from_host --verbose >/dev/null
    substrate workspace sync --direction from_host --verbose >/dev/null
    run_or_skip "world exec" substrate --world -c "sh -lc 'echo w > hello-both.txt'" >/dev/null
    set +e
    out="$(substrate workspace sync --direction both --verbose 2>&1)"
    rc=$?
    set -e
    if [[ $rc -ne 0 ]]; then
      if should_skip_for_output "$out"; then
        echo "SKIP: world-sync linux smoke ($SLICE_ID): missing/old world backend prerequisites (workspace sync both --verbose)"
        printf '%s\n' "$out"
        exit 0
      fi
      echo "FAIL: world-sync linux smoke ($SLICE_ID): workspace sync both --verbose (exit=$rc)" >&2
      printf '%s\n' "$out" >&2
      exit "$rc"
    fi
    printf '%s' "$out" | grep -Fq "hello-both.txt"
    test -f hello-both.txt
    echo "OK: world-sync linux smoke ($SLICE_ID)"
    ;;
  WS7)
    substrate workspace checkpoint --message "smoke" >/dev/null
    echo "mutated" > mutation.txt
    set +e
    substrate workspace rollback last >/dev/null 2>&1
    code=$?
    set -e
    test "$code" -eq 5
    substrate workspace rollback last --force >/dev/null
    test ! -f mutation.txt
    echo "OK: world-sync linux smoke ($SLICE_ID)"
    ;;
  *)
    echo "FAIL: unsupported SUBSTRATE_SMOKE_SLICE_ID=$SLICE_ID" >&2
    exit 1
    ;;
esac
