#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  bash scripts/e2e/workspace_sync_matrix.sh [options]

Purpose:
  Exercise a broad set of "workspace sync" scenarios (host/world create/modify/delete)
  across sync direction + conflict policy combinations, while capturing and checking
  verbose output.

Options:
  --substrate-bin <path>   Substrate binary (default: substrate)
  --log-dir <dir>          Where to write logs (default: target/world-sync-matrix/<utc>/)
  --keep                   Keep temp workspaces (default: cleanup)
  --no-assert              Do not assert log patterns (still writes logs)

Notes:
  - PTY scenarios require the `script` command (util-linux).
  - All work happens in temporary directories under /tmp.
USAGE
}

die() {
    echo "ERROR: $*" >&2
    exit 2
}

warn() {
    echo "WARN: $*" >&2
}

require_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        die "Missing dependency: $1"
    fi
}

utc_now_compact() {
    date -u +%Y%m%dT%H%M%SZ
}

log() {
    echo "== $*" >&2
}

run_capture() {
    local logfile="$1"
    shift

    echo "+ $*" | tee -a "${logfile}" >&2
    local out
    out="$(
        {
            "$@"
        } 2>&1 | tee -a "${logfile}"
    )"
    local status="${PIPESTATUS[0]}"
    echo "${out}"
    return "${status}"
}

assert_contains() {
    local haystack="$1"
    local needle="$2"
    local context="$3"

    if [[ "${ASSERT}" -ne 1 ]]; then
        return 0
    fi
    if ! grep -Fq -- "${needle}" <<<"${haystack}"; then
        echo "ASSERTION FAILED (${context}): missing: ${needle}" >&2
        return 1
    fi
}

assert_not_contains() {
    local haystack="$1"
    local needle="$2"
    local context="$3"

    if [[ "${ASSERT}" -ne 1 ]]; then
        return 0
    fi
    if grep -Fq -- "${needle}" <<<"${haystack}"; then
        echo "ASSERTION FAILED (${context}): unexpectedly contained: ${needle}" >&2
        return 1
    fi
}

init_workspace() {
    local ws_dir="$1"
    local logfile="$2"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' workspace init" >/dev/null
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' config current show" >/dev/null
}

set_sync_config() {
    local ws_dir="$1"
    local logfile="$2"
    local direction="$3"
    local conflict_policy="$4"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' config set sync.direction='${direction}'" >/dev/null
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' config set sync.conflict_policy='${conflict_policy}'" >/dev/null
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' config show" >/dev/null
}

run_repl_pty() {
    local ws_dir="$1"
    local logfile="$2"
    local repl_commands="$3"

    if ! command -v script >/dev/null 2>&1; then
        die "PTY scenario requires 'script' (util-linux). Install it or rerun with --no-assert and skip PTY scenarios."
    fi

    # `script` allocates a PTY for the child; feeding commands via stdin works well enough for our purposes.
    # We send an explicit `exit` so the REPL terminates.
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && script -q -e /dev/null -c '${SUBSTRATE_BIN}' <<'EOF'
${repl_commands}
exit
EOF" >/dev/null
}

ws_sync_dry_verbose() {
    local ws_dir="$1"
    local logfile="$2"
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' workspace sync --dry-run --verbose"
}

ws_sync_apply_verbose() {
    local ws_dir="$1"
    local logfile="$2"
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' workspace sync --verbose"
}

scenario_empty_sync_preview() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: empty workspace => dry-run shows no pending diffs"

    local out
    out="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${out}" "workspace sync --dry-run preview (WS1)" "empty preview"
    assert_contains "${out}" "pending diff summary (combined)" "empty preview"
    assert_contains "${out}" "total_paths: 0" "empty preview"
}

scenario_world_write_via_pty_applies_once() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: world creates file in PTY => sync applies write and clears in one run"

    run_repl_pty "${ws_dir}" "${logfile}" "printf 'hello\\n' > pty_new.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (pty)" "pty write preview"
    assert_contains "${preview}" "writes: 1" "pty write preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "workspace sync applied" "pty write apply"
    assert_contains "${applied}" "writes_applied: 1" "pty write apply"
    assert_not_contains "${applied}" "diff_id mismatch" "pty write apply"

    local after
    after="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${after}" "total_paths: 0" "pty write cleared"
}

scenario_world_write_via_nonpty_applies() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: world creates file in non-PTY (-c) => sync applies write"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \"printf 'hello\\n' > nonpty_new.md\"" >/dev/null

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (non_pty)" "non-pty write preview"
    assert_contains "${preview}" "total_paths: 1" "non-pty write preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "workspace sync applied" "non-pty write apply"
    assert_contains "${applied}" "writes_applied: 1" "non-pty write apply"
}

scenario_host_create_visible_in_world_no_sync_needed() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: host creates file => world sees it immediately (mount), sync has nothing to apply"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > host_only.md" >/dev/null
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \"test -f host_only.md && echo 'world_can_see_host_only=1'\"" >/dev/null

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (combined)" "host create preview"
    assert_contains "${preview}" "total_paths: 0" "host create preview"
}

scenario_direction_from_host_discards_world_only_changes() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: direction=from_host => world-only changes are discarded (not applied to host)"

    run_repl_pty "${ws_dir}" "${logfile}" "printf 'world\\n' > world_only.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "from_host reconciliation plan" "from_host discard preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "from_host reconciliation" "from_host discard apply"
    assert_contains "${applied}" "workspace sync applied" "from_host discard apply"

    local host_check
    host_check="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && (test ! -e world_only.md && echo 'host_did_not_receive_world_only=1')")"
    assert_contains "${host_check}" "host_did_not_receive_world_only=1" "from_host discard host check"

    local world_check
    world_check="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \"test ! -e world_only.md && echo 'world_discarded_upper=1'\"")"
    assert_contains "${world_check}" "world_discarded_upper=1" "from_host discard world check"
}

scenario_delete_host_file_in_world_prefer_world_applies_delete() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: host creates file; world deletes; prefer_world => sync applies delete to host"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > doomed.md" >/dev/null
    run_repl_pty "${ws_dir}" "${logfile}" "rm -f doomed.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (pty)" "prefer_world delete preview"
    assert_contains "${preview}" "deletes: 1" "prefer_world delete preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "deletes_applied: 1" "prefer_world delete apply"

    local host_ls
    host_ls="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && (test ! -e doomed.md && echo 'host_deleted=1')")"
    assert_contains "${host_ls}" "host_deleted=1" "prefer_world delete host check"
}

scenario_delete_host_file_in_world_prefer_host_discards_delete() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: host creates file; world deletes; prefer_host => sync discards world delete (host keeps)"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > kept.md" >/dev/null
    run_repl_pty "${ws_dir}" "${logfile}" "rm -f kept.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (pty)" "prefer_host delete preview"
    assert_contains "${preview}" "deletes: 1" "prefer_host delete preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"

    # With prefer_host, the world-side delete is expected to be discarded during reconciliation.
    # We primarily assert host still has the file after sync.
    local host_ls
    host_ls="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && (test -f kept.md && echo 'host_kept=1')")"
    assert_contains "${host_ls}" "host_kept=1" "prefer_host delete host check"

    # Deleting in world is still a pending diff before reconciliation; after sync it should clear.
    local after
    after="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${after}" "total_paths: 0" "prefer_host delete cleared"
}

scenario_delete_host_file_in_world_direction_from_world_deletes_host_even_prefer_host() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: direction=from_world + prefer_host => world delete is applied to host (no from_host reconciliation)"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > from_world_delete.md" >/dev/null
    run_repl_pty "${ws_dir}" "${logfile}" "rm -f from_world_delete.md"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_not_contains "${applied}" "from_host reconciliation" "from_world delete apply"
    assert_contains "${applied}" "deletes_applied: 1" "from_world delete apply"

    local host_check
    host_check="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && (test ! -e from_world_delete.md && echo 'host_deleted=1')")"
    assert_contains "${host_check}" "host_deleted=1" "from_world delete host check"
}

scenario_concurrent_mod_conflict_prefer_host_skips_apply() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: concurrent mods; prefer_host => sync skips applying world change"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'base\\n' > conflict.md" >/dev/null

    # World edits first (creates an upper shadow).
    run_repl_pty "${ws_dir}" "${logfile}" "printf 'world\\n' > conflict.md"
    # Host edits after the world session to ensure host mtime > session_started_at.
    sleep 1
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > conflict.md" >/dev/null

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "from_host reconciliation plan" "prefer_host concurrent preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "workspace sync applied" "prefer_host concurrent apply"

    # Expect host content to remain "host".
    local host_cat
    host_cat="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && (cat conflict.md && echo '')")"
    assert_contains "${host_cat}" "host" "prefer_host concurrent host content"
}

scenario_concurrent_mod_conflict_prefer_world_applies() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: concurrent mods; prefer_world => sync applies world change"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'base\\n' > conflict.md" >/dev/null

    # World edits first (upper shadow).
    run_repl_pty "${ws_dir}" "${logfile}" "printf 'world\\n' > conflict.md"
    # Host edits after.
    sleep 1
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > conflict.md" >/dev/null

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "workspace sync applied" "prefer_world concurrent apply"

    # Expect host to end up with "world".
    local host_cat
    host_cat="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && (cat conflict.md && echo '')")"
    assert_contains "${host_cat}" "world" "prefer_world concurrent host content"
}

SUBSTRATE_BIN="substrate"
LOG_DIR="target/world-sync-matrix/$(utc_now_compact)"
KEEP=0
ASSERT=1

while [[ $# -gt 0 ]]; do
    case "$1" in
        --substrate-bin)
            SUBSTRATE_BIN="${2:-}"
            shift 2
            ;;
        --log-dir)
            LOG_DIR="${2:-}"
            shift 2
            ;;
        --keep)
            KEEP=1
            shift
            ;;
        --no-assert)
            ASSERT=0
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "Unknown argument: $1 (use --help)"
            ;;
    esac
done

require_cmd bash
require_cmd "${SUBSTRATE_BIN}"

mkdir -p "${LOG_DIR}"
log "Logging to: ${LOG_DIR}"

TMP_DIRS=()
cleanup() {
    if [[ "${KEEP}" -eq 1 ]]; then
        warn "Keeping temp workspaces:"
        printf '%s\n' "${TMP_DIRS[@]}" >&2
        return 0
    fi
    for d in "${TMP_DIRS[@]}"; do
        rm -rf "${d}" || true
    done
}
trap cleanup EXIT

if ! command -v script >/dev/null 2>&1; then
    warn "Missing 'script' command; PTY scenarios will fail. Install util-linux or rerun with --no-assert and comment out PTY scenarios."
fi

main() {
    local logfile

    # Scenario 0: empty preview
    local ws0
    ws0="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws0}")
    logfile="${LOG_DIR}/scenario_00_empty.log"
    init_workspace "${ws0}" "${logfile}"
    set_sync_config "${ws0}" "${logfile}" "both" "prefer_host"
    scenario_empty_sync_preview "${ws0}" "${logfile}"

    # Scenario 1: PTY world write => apply once
    local ws1
    ws1="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws1}")
    logfile="${LOG_DIR}/scenario_01_pty_write.log"
    init_workspace "${ws1}" "${logfile}"
    set_sync_config "${ws1}" "${logfile}" "from_world" "prefer_host"
    scenario_world_write_via_pty_applies_once "${ws1}" "${logfile}"

    # Scenario 2: non-pty world write => apply
    local ws2
    ws2="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws2}")
    logfile="${LOG_DIR}/scenario_02_nonpty_write.log"
    init_workspace "${ws2}" "${logfile}"
    set_sync_config "${ws2}" "${logfile}" "from_world" "prefer_host"
    scenario_world_write_via_nonpty_applies "${ws2}" "${logfile}"

    # Scenario 3: host create visible in world (no sync)
    local ws3
    ws3="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws3}")
    logfile="${LOG_DIR}/scenario_03_host_visible.log"
    init_workspace "${ws3}" "${logfile}"
    set_sync_config "${ws3}" "${logfile}" "both" "prefer_host"
    scenario_host_create_visible_in_world_no_sync_needed "${ws3}" "${logfile}"

    # Scenario 3b: direction=from_host discards world-only writes
    local ws3b
    ws3b="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws3b}")
    logfile="${LOG_DIR}/scenario_03b_from_host_discards_world_only.log"
    init_workspace "${ws3b}" "${logfile}"
    set_sync_config "${ws3b}" "${logfile}" "from_host" "prefer_host"
    scenario_direction_from_host_discards_world_only_changes "${ws3b}" "${logfile}"

    # Scenario 4: host create, world delete, prefer_world => host delete applied
    local ws4
    ws4="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws4}")
    logfile="${LOG_DIR}/scenario_04_world_delete_prefer_world.log"
    init_workspace "${ws4}" "${logfile}"
    set_sync_config "${ws4}" "${logfile}" "both" "prefer_world"
    scenario_delete_host_file_in_world_prefer_world_applies_delete "${ws4}" "${logfile}"

    # Scenario 5: host create, world delete, prefer_host => delete discarded
    local ws5
    ws5="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws5}")
    logfile="${LOG_DIR}/scenario_05_world_delete_prefer_host.log"
    init_workspace "${ws5}" "${logfile}"
    set_sync_config "${ws5}" "${logfile}" "both" "prefer_host"
    scenario_delete_host_file_in_world_prefer_host_discards_delete "${ws5}" "${logfile}"

    # Scenario 5b: direction=from_world means deletes apply even with prefer_host
    local ws5b
    ws5b="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws5b}")
    logfile="${LOG_DIR}/scenario_05b_from_world_delete_prefer_host.log"
    init_workspace "${ws5b}" "${logfile}"
    set_sync_config "${ws5b}" "${logfile}" "from_world" "prefer_host"
    scenario_delete_host_file_in_world_direction_from_world_deletes_host_even_prefer_host "${ws5b}" "${logfile}"

    # Scenario 6: concurrent mods, prefer_host => host wins
    local ws6
    ws6="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws6}")
    logfile="${LOG_DIR}/scenario_06_concurrent_mod_prefer_host.log"
    init_workspace "${ws6}" "${logfile}"
    set_sync_config "${ws6}" "${logfile}" "both" "prefer_host"
    scenario_concurrent_mod_conflict_prefer_host_skips_apply "${ws6}" "${logfile}"

    # Scenario 7: concurrent mods, prefer_world => world wins
    local ws7
    ws7="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws7}")
    logfile="${LOG_DIR}/scenario_07_concurrent_mod_prefer_world.log"
    init_workspace "${ws7}" "${logfile}"
    set_sync_config "${ws7}" "${logfile}" "both" "prefer_world"
    scenario_concurrent_mod_conflict_prefer_world_applies "${ws7}" "${logfile}"

    log "DONE"
    log "Logs written to: ${LOG_DIR}"
}

main
