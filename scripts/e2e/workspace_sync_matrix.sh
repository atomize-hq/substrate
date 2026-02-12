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

touch_future() {
    local path="$1"

    # GNU coreutils touch supports -d; BSD touch (macOS) supports -t.
    if touch -d '2099-01-01' "${path}" >/dev/null 2>&1; then
        return 0
    fi
    touch -t 209901010000 "${path}"
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

assert_any_contains() {
    local haystack="$1"
    shift
    local context="$1"
    shift

    if [[ "${ASSERT}" -ne 1 ]]; then
        return 0
    fi

    local needle
    for needle in "$@"; do
        if grep -Fq -- "${needle}" <<<"${haystack}"; then
            return 0
        fi
    done

    echo "ASSERTION FAILED (${context}): none of the expected strings were found:" >&2
    for needle in "$@"; do
        echo "  - ${needle}" >&2
    done
    return 1
}

assert_status_eq() {
    local got="$1"
    local want="$2"
    local context="$3"

    if [[ "${ASSERT}" -ne 1 ]]; then
        return 0
    fi
    if [[ "${got}" -ne "${want}" ]]; then
        echo "ASSERTION FAILED (${context}): exit status ${got} != ${want}" >&2
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

    local precheck
    precheck="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && (test ! -e pty_new.md && echo 'host_has_pty_new=0') && '${SUBSTRATE_BIN}' -c \"test -f pty_new.md && echo 'world_has_pty_new=1'\"")"
    assert_contains "${precheck}" "host_has_pty_new=0" "pty write precheck"
    assert_contains "${precheck}" "world_has_pty_new=1" "pty write precheck"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (pty)" "pty write preview"
    assert_contains "${preview}" "total_paths: 1" "pty write preview"
    assert_any_contains "${preview}" "pty write preview kind" "writes: 1" "mods: 1"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "workspace sync applied" "pty write apply"
    assert_any_contains "${applied}" "pty write apply count" "writes_applied: 1" "mods_applied: 1"
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

scenario_direction_from_host_keeps_non_conflict_world_only_shadow() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: direction=from_host => world-only shadow is kept (and never applied to host)"

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
    world_check="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \"test -f world_only.md && echo 'world_kept_shadow=1'\"")"
    assert_contains "${world_check}" "world_kept_shadow=1" "from_host keep world check"
}

scenario_from_host_conflict_prefer_host_discards_shadowed_path() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: direction=from_host + prefer_host => conflicting shadowed path is discarded (world sees host)"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > shadow.md" >/dev/null
    run_capture "${logfile}" touch_future "${ws_dir}/shadow.md" >/dev/null
    run_repl_pty "${ws_dir}" "${logfile}" "printf 'world\\n' > shadow.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "from_host reconciliation plan" "from_host prefer_host preview"
    assert_contains "${preview}" "conflicts_detected: true (1)" "from_host prefer_host preview"
    assert_contains "${preview}" "shadow.md => discard" "from_host prefer_host preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "from_host reconciliation" "from_host prefer_host apply"
    assert_contains "${applied}" "workspace sync applied" "from_host prefer_host apply"

    local world_cat
    world_cat="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \"cat shadow.md\"")"
    assert_contains "${world_cat}" "host" "from_host prefer_host world sees host"

    local host_cat
    host_cat="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && cat shadow.md")"
    assert_contains "${host_cat}" "host" "from_host prefer_host host unchanged"

    # Confirm the shadow was actually cleared (no pending diffs once we switch to from_world preview).
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' config set sync.direction='from_world'" >/dev/null
    local after
    after="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${after}" "total_paths: 0" "from_host prefer_host cleared shadow"
}

scenario_from_host_conflict_prefer_world_keeps_shadowed_path() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: direction=from_host + prefer_world => conflicting shadowed path is kept (world continues to shadow host)"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > shadow.md" >/dev/null
    run_capture "${logfile}" touch_future "${ws_dir}/shadow.md" >/dev/null
    run_repl_pty "${ws_dir}" "${logfile}" "printf 'world\\n' > shadow.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "from_host reconciliation plan" "from_host prefer_world preview"
    assert_contains "${preview}" "conflicts_detected: true (1)" "from_host prefer_world preview"
    assert_contains "${preview}" "shadow.md => keep" "from_host prefer_world preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "from_host reconciliation" "from_host prefer_world apply"
    assert_contains "${applied}" "workspace sync applied" "from_host prefer_world apply"

    local world_cat
    world_cat="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \"cat shadow.md\"")"
    assert_contains "${world_cat}" "world" "from_host prefer_world world still shadows"

    local host_cat
    host_cat="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && cat shadow.md")"
    assert_contains "${host_cat}" "host" "from_host prefer_world host unchanged"
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

scenario_world_create_then_delete_noop() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: world creates then deletes before sync => no pending diffs"

    run_repl_pty "${ws_dir}" "${logfile}" "printf 'tmp\\n' > noop.md\nrm -f noop.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (combined)" "create+delete noop preview"
    assert_contains "${preview}" "total_paths: 0" "create+delete noop preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "workspace sync applied" "create+delete noop apply"
    assert_contains "${applied}" "writes_applied: 0" "create+delete noop apply"
    assert_contains "${applied}" "mods_applied: 0" "create+delete noop apply"
    assert_contains "${applied}" "deletes_applied: 0" "create+delete noop apply"
}

scenario_world_rename_mv_results_in_delete_plus_write() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: world rename (mv) => sync applies delete+write"

    run_repl_pty "${ws_dir}" "${logfile}" "printf 'a\\n' > rename_src.md"
    local first_apply
    first_apply="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${first_apply}" "workspace sync applied" "rename first apply"
    assert_any_contains "${first_apply}" "rename first apply count" "writes_applied: 1" "mods_applied: 1"

    run_repl_pty "${ws_dir}" "${logfile}" "mv rename_src.md rename_dst.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (combined)" "rename preview"
    assert_contains "${preview}" "total_paths: 2" "rename preview"
    assert_contains "${preview}" "deletes: 1" "rename preview"
    assert_any_contains "${preview}" "rename preview kind" "writes: 1" "mods: 1"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "workspace sync applied" "rename apply"
    assert_contains "${applied}" "deletes_applied: 1" "rename apply"
    assert_any_contains "${applied}" "rename apply kind" "writes_applied: 1" "mods_applied: 1"

    local host_check
    host_check="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && (test ! -e rename_src.md && test -f rename_dst.md && echo 'rename_ok=1')")"
    assert_contains "${host_check}" "rename_ok=1" "rename host check"
}

scenario_world_modifies_host_file_prefer_world_applies_mod() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: world edits host file; prefer_world => host updated on sync"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > edit.md" >/dev/null
    run_repl_pty "${ws_dir}" "${logfile}" "printf 'world\\n' > edit.md"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "workspace sync applied" "world edit apply"
    assert_any_contains "${applied}" "world edit apply count" "mods_applied: 1" "writes_applied: 1"

    local host_cat
    host_cat="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && cat edit.md")"
    assert_contains "${host_cat}" "world" "world edit host content"
}

scenario_combined_pty_and_nonpty_shows_combined_counts() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: PTY + non-PTY diffs => combined summary includes both"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \"printf 'n\\n' > nonpty.md\"" >/dev/null
    run_repl_pty "${ws_dir}" "${logfile}" "printf 'p\\n' > pty.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (non_pty)" "combined preview"
    assert_contains "${preview}" "pending diff summary (pty)" "combined preview"
    assert_contains "${preview}" "substrate: pending diff summary (combined)" "combined preview"
    assert_contains "${preview}" "non_pty_total_paths: 1" "combined preview"
    assert_contains "${preview}" "pty_total_paths: 1" "combined preview"
    assert_contains "${preview}" "total_paths: 2" "combined preview"
}

scenario_excluded_paths_not_synced() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: excluded paths (.substrate/**, .git/**) => never applied to host"

    run_repl_pty "${ws_dir}" "${logfile}" "mkdir -p .git\nprintf 'x\\n' > .git/excluded.md\nprintf 'y\\n' > .substrate/excluded.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (combined)" "exclude preview"
    assert_contains "${preview}" "total_paths: 0" "exclude preview"
    assert_contains "${preview}" "excluded_by_patterns: true" "exclude preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "workspace sync applied" "exclude apply"

    local host_check
    host_check="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && (test ! -e .git/excluded.md && test ! -e .substrate/excluded.md && echo 'excluded_not_on_host=1')")"
    assert_contains "${host_check}" "excluded_not_on_host=1" "exclude host check"

    local world_check
    world_check="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \"test -f .git/excluded.md && test -f .substrate/excluded.md && echo 'excluded_in_world=1'\"")"
    assert_contains "${world_check}" "excluded_in_world=1" "exclude world check"
}

scenario_delete_directory_tree_applies() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: world deletes directory tree => sync deletes on host"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && mkdir -p tree/a && printf 'host\\n' > tree/a/file.md" >/dev/null
    run_repl_pty "${ws_dir}" "${logfile}" "rm -rf tree"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "workspace sync applied" "tree delete apply"
    assert_any_contains "${applied}" "tree delete apply count" "deletes_applied: 1" "deletes_applied: 2"

    local host_check
    host_check="$(run_capture "${logfile}" bash -lc "cd '${ws_dir}' && (test ! -e tree && echo 'tree_deleted=1')")"
    assert_contains "${host_check}" "tree_deleted=1" "tree delete host check"
}

scenario_conflict_policy_abort_refuses_on_from_host_conflict() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: conflict_policy=abort + direction=both => sync refuses with exit 5 on conflict"

    # Ensure a baseline exists (and is earlier than our host change) by starting a world session first.
    run_repl_pty "${ws_dir}" "${logfile}" "true"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > abort.md" >/dev/null
    run_capture "${logfile}" touch_future "${ws_dir}/abort.md" >/dev/null
    run_repl_pty "${ws_dir}" "${logfile}" "printf 'world\\n' > abort.md"

    local out status
    set +e
    out="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    status=$?
    set -e
    assert_status_eq "${status}" 5 "abort policy status"
    assert_contains "${out}" "workspace sync refused: conflicts detected (policy=abort)" "abort policy message"
    assert_contains "${out}" "  - abort.md" "abort policy lists path"
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

    # Scenario 3b: direction=from_host keeps world-only shadow (never applied to host)
    local ws3b
    ws3b="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws3b}")
    logfile="${LOG_DIR}/scenario_03b_from_host_keeps_world_only_shadow.log"
    init_workspace "${ws3b}" "${logfile}"
    set_sync_config "${ws3b}" "${logfile}" "from_host" "prefer_host"
    scenario_direction_from_host_keeps_non_conflict_world_only_shadow "${ws3b}" "${logfile}"

    # Scenario 3c: direction=from_host, prefer_host discards conflicting shadowed host path
    local ws3c
    ws3c="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws3c}")
    logfile="${LOG_DIR}/scenario_03c_from_host_conflict_prefer_host_discards.log"
    init_workspace "${ws3c}" "${logfile}"
    set_sync_config "${ws3c}" "${logfile}" "from_host" "prefer_host"
    scenario_from_host_conflict_prefer_host_discards_shadowed_path "${ws3c}" "${logfile}"

    # Scenario 3d: direction=from_host, prefer_world keeps conflicting shadowed host path
    local ws3d
    ws3d="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws3d}")
    logfile="${LOG_DIR}/scenario_03d_from_host_conflict_prefer_world_keeps.log"
    init_workspace "${ws3d}" "${logfile}"
    set_sync_config "${ws3d}" "${logfile}" "from_host" "prefer_world"
    scenario_from_host_conflict_prefer_world_keeps_shadowed_path "${ws3d}" "${logfile}"

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

    # Scenario 5c: create+delete before sync => noop
    local ws5c
    ws5c="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws5c}")
    logfile="${LOG_DIR}/scenario_05c_world_create_then_delete_noop.log"
    init_workspace "${ws5c}" "${logfile}"
    set_sync_config "${ws5c}" "${logfile}" "from_world" "prefer_host"
    scenario_world_create_then_delete_noop "${ws5c}" "${logfile}"

    # Scenario 5d: rename in world => delete+write applied
    local ws5d
    ws5d="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws5d}")
    logfile="${LOG_DIR}/scenario_05d_world_rename_mv.log"
    init_workspace "${ws5d}" "${logfile}"
    set_sync_config "${ws5d}" "${logfile}" "from_world" "prefer_world"
    scenario_world_rename_mv_results_in_delete_plus_write "${ws5d}" "${logfile}"

    # Scenario 5e: world edits host file => mod applied (prefer_world)
    local ws5e
    ws5e="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws5e}")
    logfile="${LOG_DIR}/scenario_05e_world_edits_host_file_prefer_world.log"
    init_workspace "${ws5e}" "${logfile}"
    set_sync_config "${ws5e}" "${logfile}" "from_world" "prefer_world"
    scenario_world_modifies_host_file_prefer_world_applies_mod "${ws5e}" "${logfile}"

    # Scenario 5f: PTY + non-PTY combined preview counts
    local ws5f
    ws5f="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws5f}")
    logfile="${LOG_DIR}/scenario_05f_combined_pty_nonpty_counts.log"
    init_workspace "${ws5f}" "${logfile}"
    set_sync_config "${ws5f}" "${logfile}" "from_world" "prefer_host"
    scenario_combined_pty_and_nonpty_shows_combined_counts "${ws5f}" "${logfile}"

    # Scenario 5g: excluded paths never sync to host
    local ws5g
    ws5g="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws5g}")
    logfile="${LOG_DIR}/scenario_05g_excluded_paths.log"
    init_workspace "${ws5g}" "${logfile}"
    set_sync_config "${ws5g}" "${logfile}" "from_world" "prefer_host"
    scenario_excluded_paths_not_synced "${ws5g}" "${logfile}"

    # Scenario 5h: delete directory tree in world => deletes applied to host
    local ws5h
    ws5h="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws5h}")
    logfile="${LOG_DIR}/scenario_05h_delete_directory_tree.log"
    init_workspace "${ws5h}" "${logfile}"
    set_sync_config "${ws5h}" "${logfile}" "from_world" "prefer_world"
    scenario_delete_directory_tree_applies "${ws5h}" "${logfile}"

    # Scenario 5i: conflict_policy=abort refuses on from_host conflict
    local ws5i
    ws5i="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
    TMP_DIRS+=("${ws5i}")
    logfile="${LOG_DIR}/scenario_05i_abort_policy_refuses.log"
    init_workspace "${ws5i}" "${logfile}"
    set_sync_config "${ws5i}" "${logfile}" "both" "abort"
    scenario_conflict_policy_abort_refuses_on_from_host_conflict "${ws5i}" "${logfile}"

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
