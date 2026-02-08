#!/usr/bin/env bash
set -euo pipefail

# Shared host-only smoke for:
# - world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment
#
# Caller responsibilities:
# - Ensure `substrate` is available via `$SUBSTRATE_BIN` (default: `substrate`).

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "wfgad-appendix-addon-v3-alignment: substrate binary not found (SUBSTRATE_BIN=$SUBSTRATE_BIN)" >&2
  exit 3
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "wfgad-appendix-addon-v3-alignment: python3 is required for smoke JSON assertions" >&2
  exit 4
fi

tmp_root="${SUBSTRATE_SMOKE_ROOT:-}"
if [[ -z "${tmp_root}" ]]; then
  tmp_root="$(mktemp -d)"
fi

cleanup() {
  if [[ "${SUBSTRATE_SMOKE_KEEP:-0}" == "1" ]]; then
    return 0
  fi
  rm -rf "$tmp_root"
}
trap cleanup EXIT

export SUBSTRATE_HOME="${SUBSTRATE_HOME:-$tmp_root/substrate-home}"

workspace="$tmp_root/workspace"
mkdir -p "$workspace"
cd "$workspace"

expect_exit() {
  local want="$1"
  shift
  local out
  set +e
  out="$("$@" 2>&1)"
  local got="$?"
  set -e
  if [[ "$got" -ne "$want" ]]; then
    echo "FAIL: expected exit $want, got $got: $*" >&2
    echo "$out" >&2
    exit 1
  fi
}

echo "== Setup: workspace + policy patch (isolated SUBSTRATE_HOME) =="
"$SUBSTRATE_BIN" workspace init --force >/dev/null
"$SUBSTRATE_BIN" policy init --force >/dev/null

echo "== Case 1: legacy V2 keys are rejected (no-backcompat; exit 2) =="
expect_exit 2 "$SUBSTRATE_BIN" policy set 'world_fs.mode=read_only'
expect_exit 2 "$SUBSTRATE_BIN" policy set 'world_fs.isolation=full'
expect_exit 2 "$SUBSTRATE_BIN" policy set 'world_fs.require_world=true'

echo "== Case 2: `substrate policy show` output is V3-shaped (Appendix A.6) =="
"$SUBSTRATE_BIN" policy set \
  'world_fs.host_visible=false' \
  'world_fs.fail_closed.routing=false' \
  'world_fs.write.enabled=true' \
  'world_fs.read.allow_list+=.' \
  >/dev/null

policy_json="$("$SUBSTRATE_BIN" policy show --json)"
python3 - <<'PY'
import json
import sys

data = json.loads(sys.stdin.read())
world_fs = data.get("world_fs")
if not isinstance(world_fs, dict):
    raise SystemExit("FAIL: policy JSON missing object: world_fs")

if world_fs.get("host_visible") is not False:
    raise SystemExit(f"FAIL: expected world_fs.host_visible=false, got: {world_fs.get('host_visible')!r}")

legacy_keys = ["mode", "isolation", "require_world", "enforcement", "read_allowlist", "write_allowlist"]
present_legacy = [k for k in legacy_keys if k in world_fs]
if present_legacy:
    raise SystemExit(f"FAIL: legacy V2 keys present under world_fs: {present_legacy}")

def assert_dimension(name: str, require_enabled: bool) -> None:
    dim = world_fs.get(name)
    if not isinstance(dim, dict):
        raise SystemExit(f"FAIL: missing world_fs.{name} object")
    if require_enabled:
        if dim.get("enabled") is not True:
            raise SystemExit(f"FAIL: expected world_fs.{name}.enabled=true, got: {dim.get('enabled')!r}")
    allow_list = dim.get("allow_list")
    deny_list = dim.get("deny_list")
    if not isinstance(allow_list, list) or not allow_list:
        raise SystemExit(f"FAIL: missing or empty world_fs.{name}.allow_list")
    if not isinstance(deny_list, list):
        raise SystemExit(f"FAIL: missing world_fs.{name}.deny_list (must be an array; empty must be explicit)")
    if deny_list != []:
        raise SystemExit(f"FAIL: expected world_fs.{name}.deny_list==[], got: {deny_list!r}")

assert_dimension("discover", require_enabled=False)
assert_dimension("read", require_enabled=False)
assert_dimension("write", require_enabled=True)
print("OK: policy show --json is V3-shaped and includes explicit empty deny_list arrays")
PY <<<"$policy_json"

policy_yaml="$("$SUBSTRATE_BIN" policy show)"
if ! grep -Eq '^[[:space:]]*world_fs:[[:space:]]*$' <<<"$policy_yaml"; then
  echo "FAIL: policy YAML missing top-level world_fs:" >&2
  echo "$policy_yaml" >&2
  exit 1
fi
if ! grep -Eq '^[[:space:]]{2}host_visible:[[:space:]]*false[[:space:]]*$' <<<"$policy_yaml"; then
  echo "FAIL: policy YAML missing expected world_fs.host_visible: false" >&2
  echo "$policy_yaml" >&2
  exit 1
fi

deny_count="$(grep -E '^[[:space:]]{4}deny_list:[[:space:]]*\\[\\][[:space:]]*$' <<<"$policy_yaml" | wc -l | tr -d ' ')"
if [[ "${deny_count}" -lt 3 ]]; then
  echo "FAIL: expected explicit empty deny_list: [] for discover/read/write (>=3), got: ${deny_count}" >&2
  echo "$policy_yaml" >&2
  exit 1
fi

if grep -Eq '^[[:space:]]{2}(mode|isolation|require_world|enforcement|read_allowlist|write_allowlist):' <<<"$policy_yaml"; then
  echo "FAIL: policy YAML contains legacy V2 world_fs keys (must not be operator-facing output)" >&2
  echo "$policy_yaml" >&2
  exit 1
fi

for dim in discover read write; do
  if ! grep -Eq "^[[:space:]]{2}${dim}:[[:space:]]*$" <<<"$policy_yaml"; then
    echo "FAIL: policy YAML missing world_fs.${dim}:" >&2
    echo "$policy_yaml" >&2
    exit 1
  fi
  if ! grep -Eq '^[[:space:]]{4}allow_list:' <<<"$policy_yaml"; then
    echo "FAIL: policy YAML missing allow_list (expected for ${dim})" >&2
    echo "$policy_yaml" >&2
    exit 1
  fi
done

echo "OK: wfgad-appendix-addon-v3-alignment smoke passed"

