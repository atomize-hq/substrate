#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: WAPS smoke (not macOS)"
  exit 0
fi

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

need_cmd() {
  local name="$1"
  command -v "$name" >/dev/null 2>&1 || fail "$name not found on PATH"
}

mktemp_dir() {
  local d
  d="$(mktemp -d 2>/dev/null || true)"
  if [[ -z "${d}" || ! -d "${d}" ]]; then
    d="$(mktemp -d -t substrate-waps 2>/dev/null || true)"
  fi
  [[ -n "${d}" && -d "${d}" ]] || fail "mktemp -d failed"
  printf '%s\n' "${d}"
}

run_capture() {
  local cmd="$1"
  local out_path="$2"
  local err_path="$3"
  local exit_path="$4"

  set +e
  substrate --world --ci --command "${cmd}" >"${out_path}" 2>"${err_path}"
  local rc=$?
  set -e
  printf '%s\n' "${rc}" >"${exit_path}"
}

trace_meta_for_marker() {
  local trace_log="$1"
  local marker="$2"
  jq -c --arg marker "${marker}" '
    select(.event_type=="command_complete")
    | select((.cmd? // "") | contains($marker))
    | {
        span_id,
        exit,
        policy_resolution_mode,
        policy_snapshot_hash
      }
    + (if (.policy_snapshot_schema? != null) then { policy_snapshot_schema: .policy_snapshot_schema } else {} end)
    + (if (.world_fs_strategy_primary? != null) then { world_fs_strategy_primary: .world_fs_strategy_primary } else {} end)
    + (if (.world_fs_strategy_final? != null) then { world_fs_strategy_final: .world_fs_strategy_final } else {} end)
    + (if (.world_fs_strategy_fallback_reason? != null) then { world_fs_strategy_fallback_reason: .world_fs_strategy_fallback_reason } else {} end)
  ' "${trace_log}" | tail -n 1
}

need_cmd substrate
need_cmd jq
need_cmd mktemp
need_cmd tail

if [[ "${EUID}" -eq 0 ]]; then
  fail "do not run as root"
fi

RUN_ID="waps-$(date +%s)-$$"
TMP_HOME="$(mktemp_dir)"
TMP_WS="$(mktemp_dir)"
TRACE_LOG="${TMP_HOME}/trace.jsonl"

cleanup() {
  rm -rf "${TMP_HOME}" "${TMP_WS}"
}
trap cleanup EXIT

export SUBSTRATE_HOME="${TMP_HOME}"
export HOME="${TMP_HOME}"
export SHIM_TRACE_LOG="${TRACE_LOG}"

substrate config global init --force >/dev/null
substrate policy global init --force >/dev/null
substrate config global set policy.mode=enforce >/dev/null
substrate config global set world.enabled=true >/dev/null
substrate config global set world.anchor_mode=follow-cwd >/dev/null

substrate workspace init "${TMP_WS}" >/dev/null
cd "${TMP_WS}"
mkdir -p writable

doctor_json="$(substrate world doctor --json 2>/dev/null || true)"
doctor_ok="$(printf '%s' "${doctor_json:-null}" | jq -r '.ok? // false' 2>/dev/null || printf 'false')"
doctor_snapshot_supported="$(printf '%s' "${doctor_json:-null}" | jq -r '(.policy_snapshot_v1_supported? // .world.policy_snapshot_v1_supported? // false)' 2>/dev/null || printf 'false')"

tests_ndjson="${TMP_HOME}/tests.ndjson"
: >"${tests_ndjson}"

logs_dir="${TMP_HOME}/logs"
mkdir -p "${logs_dir}"

policy_path="${TMP_WS}/.substrate/policy.yaml"

echo "[INFO] run_id=${RUN_ID}"
echo "[INFO] workspace=${TMP_WS}"
echo "[INFO] trace_log=${TRACE_LOG}"

## ---- Test 1: FS allowlist in full isolation ----
cat >"${policy_path}" <<'YAML'
id: "waps-smoke"
name: "waps-smoke fs"
world_fs:
  mode: writable
  isolation: full
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist:
    - "./writable/*"
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {}
YAML

FS_MARKER="__${RUN_ID}__fs__"
fs_cmd="echo '${FS_MARKER}' >/dev/null; set -eu; mkdir -p writable; echo ok > writable/ok.txt; test -s writable/ok.txt; if echo nope > not-allowlisted.txt 2>/dev/null; then echo 'unexpected: non-allowlisted write succeeded' >&2; exit 1; fi; test ! -e not-allowlisted.txt"
run_capture "${fs_cmd}" "${logs_dir}/fs.stdout" "${logs_dir}/fs.stderr" "${logs_dir}/fs.exit"
fs_exit="$(cat "${logs_dir}/fs.exit")"
fs_ok="false"
fs_err=""
if [[ "${fs_exit}" == "0" ]]; then
  fs_ok="true"
else
  fs_err="exit=${fs_exit}"
fi

fs_meta="$(trace_meta_for_marker "${TRACE_LOG}" "${FS_MARKER}" || true)"
if [[ -z "${fs_meta}" ]]; then
  fs_ok="false"
  fs_err="${fs_err} missing_trace_meta"
fi

printf '%s\n' "$(jq -nc \
  --arg name "fs_allowlist_full_isolation" \
  --argjson ok "${fs_ok}" \
  --arg exit_code "${fs_exit}" \
  --arg stderr_path "${logs_dir}/fs.stderr" \
  --arg stdout_path "${logs_dir}/fs.stdout" \
  --arg err "${fs_err}" \
  --argjson trace_meta "${fs_meta:-null}" \
  '{name:$name, ok:$ok, exit_code:($exit_code|tonumber), stderr_path:$stderr_path, stdout_path:$stdout_path, error:(if ($err|length)>0 then $err else null end), trace_meta:$trace_meta}' \
)" >>"${tests_ndjson}"

## ---- Test 2: net allowlist ----
cat >"${policy_path}" <<'YAML'
id: "waps-smoke"
name: "waps-smoke net"
world_fs:
  mode: writable
  isolation: full
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist:
    - "./writable/*"
net_allowed:
  - "example.com"
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {}
YAML

NET_MARKER="__${RUN_ID}__net__"
net_cmd="echo '${NET_MARKER}' >/dev/null; set -eu; if command -v curl >/dev/null 2>&1; then curl -fsS --max-time 10 https://example.com >/dev/null; if curl -fsS --max-time 10 https://example.net >/dev/null; then echo 'unexpected: disallowed host succeeded' >&2; exit 1; fi; elif command -v python3 >/dev/null 2>&1; then python3 -c 'import urllib.request; urllib.request.urlopen(\"https://example.com\", timeout=10).read(64)' >/dev/null; if python3 -c 'import urllib.request; urllib.request.urlopen(\"https://example.net\", timeout=10).read(64)' >/dev/null 2>&1; then echo 'unexpected: disallowed host succeeded' >&2; exit 1; fi; else echo 'missing curl/python3 for net test' >&2; exit 2; fi"
run_capture "${net_cmd}" "${logs_dir}/net.stdout" "${logs_dir}/net.stderr" "${logs_dir}/net.exit"
net_exit="$(cat "${logs_dir}/net.exit")"
net_ok="false"
net_err=""
if [[ "${net_exit}" == "0" ]]; then
  net_ok="true"
else
  net_err="exit=${net_exit}"
fi

net_meta="$(trace_meta_for_marker "${TRACE_LOG}" "${NET_MARKER}" || true)"
if [[ -z "${net_meta}" ]]; then
  net_ok="false"
  net_err="${net_err} missing_trace_meta"
fi

printf '%s\n' "$(jq -nc \
  --arg name "net_allowlist" \
  --argjson ok "${net_ok}" \
  --arg exit_code "${net_exit}" \
  --arg stderr_path "${logs_dir}/net.stderr" \
  --arg stdout_path "${logs_dir}/net.stdout" \
  --arg err "${net_err}" \
  --argjson trace_meta "${net_meta:-null}" \
  '{name:$name, ok:$ok, exit_code:($exit_code|tonumber), stderr_path:$stderr_path, stdout_path:$stdout_path, error:(if ($err|length)>0 then $err else null end), trace_meta:$trace_meta}' \
)" >>"${tests_ndjson}"

## ---- Test 3: limits (best-effort) ----
cat >"${policy_path}" <<'YAML'
id: "waps-smoke"
name: "waps-smoke limits"
world_fs:
  mode: writable
  isolation: full
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist:
    - "./writable/*"
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: 1000
  max_egress_bytes: null
metadata: {}
YAML

LIMITS_MARKER="__${RUN_ID}__limits__"
limits_cmd="echo '${LIMITS_MARKER}' >/dev/null; set -eu; sleep 2"
run_capture "${limits_cmd}" "${logs_dir}/limits.stdout" "${logs_dir}/limits.stderr" "${logs_dir}/limits.exit"
limits_exit="$(cat "${logs_dir}/limits.exit")"

limits_ok="true"
limits_skipped="false"
limits_err=""
if [[ "${limits_exit}" == "0" ]]; then
  limits_skipped="true"
fi

limits_meta="$(trace_meta_for_marker "${TRACE_LOG}" "${LIMITS_MARKER}" || true)"
if [[ -z "${limits_meta}" ]]; then
  limits_ok="false"
  limits_err="missing_trace_meta"
fi

printf '%s\n' "$(jq -nc \
  --arg name "limits_max_runtime_ms" \
  --argjson ok "${limits_ok}" \
  --argjson skipped "${limits_skipped}" \
  --arg exit_code "${limits_exit}" \
  --arg stderr_path "${logs_dir}/limits.stderr" \
  --arg stdout_path "${logs_dir}/limits.stdout" \
  --arg err "${limits_err}" \
  --argjson trace_meta "${limits_meta:-null}" \
  '{name:$name, ok:$ok, skipped:$skipped, exit_code:($exit_code|tonumber), stderr_path:$stderr_path, stdout_path:$stdout_path, error:(if ($err|length)>0 then $err else null end), trace_meta:$trace_meta}' \
)" >>"${tests_ndjson}"

tests_json="$(jq -s '.' "${tests_ndjson}")"

summary="$(jq -nc \
  --arg platform "macos" \
  --arg run_id "${RUN_ID}" \
  --arg substrate_home "${SUBSTRATE_HOME}" \
  --arg workspace "${TMP_WS}" \
  --arg trace_log "${TRACE_LOG}" \
  --argjson doctor "${doctor_json:-null}" \
  --argjson doctor_ok "${doctor_ok}" \
  --argjson snapshot_supported "${doctor_snapshot_supported}" \
  --argjson tests "${tests_json}" \
  '{
    platform: $platform,
    run_id: $run_id,
    substrate_home: $substrate_home,
    workspace: $workspace,
    trace_log: $trace_log,
    doctor_ok: $doctor_ok,
    policy_snapshot_v1_supported: $snapshot_supported,
    doctor: $doctor,
    tests: $tests
  }' \
)"

echo "${summary}"

overall_ok="$(printf '%s\n' "${tests_json}" | jq -r 'all(.[]; (.ok==true))')"
schema_ok="$(printf '%s\n' "${tests_json}" | jq -r 'all(.[]; (.trace_meta.policy_resolution_mode!="snapshot_v1") or (.trace_meta.policy_snapshot_schema==1))')"
if [[ "${doctor_snapshot_supported}" == "true" ]]; then
  snapshot_ok="$(printf '%s\n' "${tests_json}" | jq -r 'all(.[]; (.trace_meta.policy_resolution_mode=="snapshot_v1") and (.trace_meta.policy_snapshot_schema==1) and ((.trace_meta.policy_snapshot_hash? // "") | length > 0))')"
else
  snapshot_ok="true"
fi

dump_failures() {
  printf '[FAIL] dumping smoke logs (first 200 lines each)\n' >&2
  printf '%s\n' "${tests_json}" \
    | jq -r '.[] | select(.ok!=true) | [.name, .stdout_path, .stderr_path] | @tsv' \
    | while IFS=$'\t' read -r name stdout_path stderr_path; do
      printf '\n[FAIL] %s\n' "${name}" >&2
      printf '[FAIL] stderr: %s\n' "${stderr_path}" >&2
      sed -n '1,200p' "${stderr_path}" >&2 || true
      printf '[FAIL] stdout: %s\n' "${stdout_path}" >&2
      sed -n '1,200p' "${stdout_path}" >&2 || true
    done
}

if [[ "${overall_ok}" != "true" ]]; then
  dump_failures
  exit 1
fi
if [[ "${schema_ok}" != "true" ]]; then
  dump_failures
  exit 1
fi
if [[ "${snapshot_ok}" != "true" ]]; then
  dump_failures
  exit 1
fi
