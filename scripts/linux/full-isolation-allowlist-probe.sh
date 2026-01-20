#!/usr/bin/env bash
set -euo pipefail

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing dependency: $1" >&2
    exit 2
  }
}

need python3
need mktemp

resolve_substrate_bin() {
  if [[ -n "${SUBSTRATE_BIN:-}" ]]; then
    echo "$SUBSTRATE_BIN"
    return 0
  fi

  local repo_root
  repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
  if [[ -f "${repo_root}/Cargo.toml" ]] && command -v cargo >/dev/null 2>&1; then
    if [[ ! -x "${repo_root}/target/debug/substrate" ]]; then
      (cd "$repo_root" && cargo build --bin substrate >/dev/null)
    fi
  fi
  if [[ -x "${repo_root}/target/debug/substrate" ]]; then
    echo "${repo_root}/target/debug/substrate"
    return 0
  fi

  if command -v substrate >/dev/null 2>&1; then
    echo "substrate"
    return 0
  fi

  if [[ -x "${HOME}/.substrate/bin/substrate" ]]; then
    echo "${HOME}/.substrate/bin/substrate"
    return 0
  fi

  echo "substrate"
  return 0
}

SUBSTRATE_BIN="$(resolve_substrate_bin)"
if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1 && [[ ! -x "$SUBSTRATE_BIN" ]]; then
  echo "missing dependency: substrate (set SUBSTRATE_BIN=... to point at it)" >&2
  exit 2
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP_BASE="${REPO_ROOT}/target/full-isolation-allowlist-probe"
mkdir -p "$TMP_BASE"

TMP_HOME="$(mktemp -d -p "$TMP_BASE")"
TMP_WS="$(mktemp -d -p "$TMP_BASE")"
cleanup() { rm -rf "$TMP_HOME" "$TMP_WS"; }
trap cleanup EXIT

export SUBSTRATE_HOME="$TMP_HOME"

"$SUBSTRATE_BIN" config global init --force >/dev/null
"$SUBSTRATE_BIN" policy global init --force >/dev/null
"$SUBSTRATE_BIN" config global set policy.mode=enforce >/dev/null
"$SUBSTRATE_BIN" config global set world.enabled=true >/dev/null
"$SUBSTRATE_BIN" config global set world.anchor_mode=follow-cwd >/dev/null

"$SUBSTRATE_BIN" workspace init "$TMP_WS" >/dev/null
mkdir -p "$TMP_WS/writable"
cd "$TMP_WS"

"$SUBSTRATE_BIN" policy workspace set \
  world_fs.mode=writable \
  world_fs.isolation=full \
  world_fs.require_world=true \
  world_fs.write_allowlist='["./writable/*"]' \
  >/dev/null

echo "== host context =="
echo "SUBSTRATE_BIN=$SUBSTRATE_BIN"
echo "SUBSTRATE_HOME=$SUBSTRATE_HOME"
echo "TMP_WS=$TMP_WS"
"$SUBSTRATE_BIN" --shim-status 2>/dev/null | sed -n '1,120p' || true
echo

CMD="$(cat <<'WORLD'
set -eu

echo "== in-world env =="
echo "MOUNT_PROJECT_DIR=$SUBSTRATE_MOUNT_PROJECT_DIR"
echo "WORLD_FS_ISOLATION=$SUBSTRATE_WORLD_FS_ISOLATION"
echo "WORLD_FS_MODE=$SUBSTRATE_WORLD_FS_MODE"
echo "LANDLOCK_HELPER_PATH=${SUBSTRATE_LANDLOCK_HELPER_PATH:-}"
echo

echo "== landlock allowlists (raw) =="
echo "--- READ ---"
printf "%s\n" "${SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST:-}" | cat -n
echo "--- WRITE ---"
printf "%s\n" "${SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST:-}" | cat -n
echo

echo "== helper presence =="
ls -la /substrate-landlock-helper 2>&1 || true
echo

echo "== mountinfo for project mountpoint =="
awk -v mp="$SUBSTRATE_MOUNT_PROJECT_DIR" '$5==mp {print; exit}' /proc/self/mountinfo || true
awk -v mp="$SUBSTRATE_MOUNT_PROJECT_DIR" '$2==mp {print; exit}' /proc/mounts || true
echo

echo "== mountinfo for allowlisted writable dir (best-effort) =="
awk -v mp="$SUBSTRATE_MOUNT_PROJECT_DIR/writable" '$5==mp {print; exit}' /proc/self/mountinfo || true
awk -v mp="$SUBSTRATE_MOUNT_PROJECT_DIR/writable" '$2==mp {print; exit}' /proc/mounts || true
awk -v mp="/project/writable" '$5==mp {print; exit}' /proc/self/mountinfo || true
awk -v mp="/project/writable" '$2==mp {print; exit}' /proc/mounts || true
echo

echo "== overlay backing dirs =="
opts="$(awk -v mp="$SUBSTRATE_MOUNT_PROJECT_DIR" '$2==mp {print $4; exit}' /proc/mounts || true)"
upper="$(printf "%s" "$opts" | tr "," "\n" | sed -n "s/^upperdir=//p" | head -n 1)"
work="$(printf "%s" "$opts" | tr "," "\n" | sed -n "s/^workdir=//p" | head -n 1)"
lower="$(printf "%s" "$opts" | tr "," "\n" | sed -n "s/^lowerdir=//p" | head -n 1)"
echo "lowerdir=$lower"
echo "upperdir=$upper"
echo "workdir=$work"
ls -ld "$upper" "$work" 2>&1 || true
echo

echo "== probes (errno) =="
UPPERDIR="$upper" WORKDIR="$work" python3 - <<'PY'
import errno
import os

def report(tag, fn):
    try:
        fn()
        print("OK ", tag)
    except OSError as e:
        print("BAD", tag, "errno=", e.errno, errno.errorcode.get(e.errno), str(e))

def write_file(path, data=b"hi"):
    fd = os.open(path, os.O_CREAT | os.O_WRONLY | os.O_TRUNC, 0o644)
    os.write(fd, data)
    os.close(fd)

report("write /tmp", lambda: write_file("/tmp/__tmp_probe__"))
try:
    os.unlink("/tmp/__tmp_probe__")
except OSError:
    pass

upper = os.environ.get("UPPERDIR", "")
work = os.environ.get("WORKDIR", "")
if upper:
    report("write upperdir", lambda: write_file(os.path.join(upper, "__upper_probe__")))
if work:
    report("write workdir", lambda: write_file(os.path.join(work, "__work_probe__")))

report("allowlisted mkdir", lambda: os.makedirs("writable/sub", exist_ok=True))
report("allowlisted write", lambda: write_file("writable/sub/ok.txt", b"ok"))

try:
    write_file("denied.txt", b"no")
    print("BAD denied write unexpectedly succeeded")
except OSError as e:
    print("OK  denied write blocked errno=", e.errno, errno.errorcode.get(e.errno))
PY
WORLD
)"

set +e
OUT="$("$SUBSTRATE_BIN" --world --ci --command "$CMD" 2>&1)"
RC=$?
set -e

echo "$OUT"
echo

if [[ $RC -ne 0 ]]; then
  echo "FAIL: world command exited rc=$RC" >&2
  exit 1
fi

if [[ -e "$TMP_WS/writable/sub/ok.txt" || -e "$TMP_WS/denied.txt" ]]; then
  echo "FAIL: host project directory mutated (unexpected)" >&2
  exit 1
fi

grep -q "^OK  allowlisted mkdir" <<<"$OUT" || { echo "FAIL: allowlisted mkdir did not succeed" >&2; exit 1; }
grep -q "^OK  allowlisted write" <<<"$OUT" || { echo "FAIL: allowlisted write did not succeed" >&2; exit 1; }
grep -q "^OK  denied write blocked" <<<"$OUT" || { echo "FAIL: denied write was not blocked" >&2; exit 1; }

echo "PASS"
