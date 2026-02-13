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
  --continue               Continue after failures (run full matrix)

Notes:
  - PTY scenarios use `substrate -c ":pty <cmd>"` to force the PTY WS route.
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
    if [[ "${status}" -ne 0 ]]; then
        echo "!! exit_status=${status}" | tee -a "${logfile}" >&2
    fi
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

run_world_nonpty() {
    local ws_dir="$1"
    local logfile="$2"
    local program="$3"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \"${program}\"" >/dev/null
}

run_world_pty() {
    local ws_dir="$1"
    local logfile="$2"
    local program="$3"

    # Force PTY route; world-agent strips the leading ":pty ".
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \":pty ${program}\"" >/dev/null
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

    run_world_pty "${ws_dir}" "${logfile}" "printf hello > pty_new.md"

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
    local world_check
    world_check="$(
        run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \"if test -f host_only.md; then echo 'world_can_see_host_only=1'; else echo 'world_can_see_host_only=0'; fi; exit 0\""
    )"
    assert_contains "${world_check}" "world_can_see_host_only=1" "host create world visibility"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (combined)" "host create preview"
    assert_contains "${preview}" "total_paths: 0" "host create preview"
}

scenario_direction_from_host_keeps_non_conflict_world_only_shadow() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: direction=from_host => world-only shadow is kept (and never applied to host)"

    run_world_nonpty "${ws_dir}" "${logfile}" "printf world > world_only.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "from_host reconciliation plan" "from_host discard preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "from_host reconciliation" "from_host discard apply"

    local host_check
    host_check="$(
        run_capture "${logfile}" bash -lc "cd '${ws_dir}' && if test -e world_only.md; then echo 'host_did_not_receive_world_only=0'; else echo 'host_did_not_receive_world_only=1'; fi"
    )"
    assert_contains "${host_check}" "host_did_not_receive_world_only=1" "from_host discard host check"

    local world_check
    world_check="$(
        run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \"if test -f world_only.md; then echo 'world_kept_shadow=1'; else echo 'world_kept_shadow=0'; fi; exit 0\""
    )"
    assert_contains "${world_check}" "world_kept_shadow=1" "from_host keep world check"
}

scenario_from_host_conflict_prefer_host_discards_shadowed_path() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: direction=from_host + prefer_host => conflicting shadowed path is discarded (world sees host)"

    # Start the world session first so the host edit is definitely after session_started_at.
    run_world_nonpty "${ws_dir}" "${logfile}" "true"
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > shadow.md" >/dev/null
    run_world_pty "${ws_dir}" "${logfile}" "printf world > shadow.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "from_host reconciliation plan" "from_host prefer_host preview"
    assert_contains "${preview}" "conflicts_detected: true (1)" "from_host prefer_host preview"
    assert_contains "${preview}" "shadow.md => discard" "from_host prefer_host preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "from_host reconciliation" "from_host prefer_host apply"

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

    # Start the world session first so the host edit is definitely after session_started_at.
    run_world_nonpty "${ws_dir}" "${logfile}" "true"
    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > shadow.md" >/dev/null
    run_world_pty "${ws_dir}" "${logfile}" "printf world > shadow.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "from_host reconciliation plan" "from_host prefer_world preview"
    assert_contains "${preview}" "conflicts_detected: true (1)" "from_host prefer_world preview"
    assert_contains "${preview}" "shadow.md => keep" "from_host prefer_world preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "from_host reconciliation" "from_host prefer_world apply"

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
    run_world_pty "${ws_dir}" "${logfile}" "rm -f doomed.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (pty)" "prefer_world delete preview"
    assert_contains "${preview}" "deletes: 1" "prefer_world delete preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "deletes_applied: 1" "prefer_world delete apply"

    local host_ls
    host_ls="$(
        run_capture "${logfile}" bash -lc "cd '${ws_dir}' && if test -e doomed.md; then echo 'host_deleted=0'; else echo 'host_deleted=1'; fi"
    )"
    assert_contains "${host_ls}" "host_deleted=1" "prefer_world delete host check"
}

scenario_delete_host_file_in_world_prefer_host_discards_delete() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: host creates file; world deletes; prefer_host => sync discards world delete (host keeps)"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > kept.md" >/dev/null
    run_world_pty "${ws_dir}" "${logfile}" "rm -f kept.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (pty)" "prefer_host delete preview"
    assert_contains "${preview}" "deletes: 1" "prefer_host delete preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"

    # With prefer_host, the world-side delete is expected to be discarded during reconciliation.
    # We primarily assert host still has the file after sync.
    local host_ls
    host_ls="$(
        run_capture "${logfile}" bash -lc "cd '${ws_dir}' && if test -f kept.md; then echo 'host_kept=1'; else echo 'host_kept=0'; fi"
    )"
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
    run_world_pty "${ws_dir}" "${logfile}" "rm -f from_world_delete.md"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_not_contains "${applied}" "from_host reconciliation" "from_world delete apply"
    assert_contains "${applied}" "deletes_applied: 1" "from_world delete apply"

    local host_check
    host_check="$(
        run_capture "${logfile}" bash -lc "cd '${ws_dir}' && if test -e from_world_delete.md; then echo 'host_deleted=0'; else echo 'host_deleted=1'; fi"
    )"
    assert_contains "${host_check}" "host_deleted=1" "from_world delete host check"
}

scenario_world_create_then_delete_noop() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: world creates then deletes before sync => no pending diffs"

    run_world_pty "${ws_dir}" "${logfile}" "printf tmp > noop.md; rm -f noop.md"

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

    run_world_pty "${ws_dir}" "${logfile}" "printf a > rename_src.md"
    local first_apply
    first_apply="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${first_apply}" "workspace sync applied" "rename first apply"
    assert_any_contains "${first_apply}" "rename first apply count" "writes_applied: 1" "mods_applied: 1"

    run_world_pty "${ws_dir}" "${logfile}" "mv rename_src.md rename_dst.md"

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
    host_check="$(
        run_capture "${logfile}" bash -lc "cd '${ws_dir}' && \
if test ! -e rename_src.md && test -f rename_dst.md; then echo 'rename_ok=1'; else echo 'rename_ok=0'; fi"
    )"
    assert_contains "${host_check}" "rename_ok=1" "rename host check"
}

scenario_world_modifies_host_file_prefer_world_applies_mod() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: world edits host file; prefer_world => host updated on sync"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > edit.md" >/dev/null
    run_world_pty "${ws_dir}" "${logfile}" "printf world > edit.md"

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

    run_world_nonpty "${ws_dir}" "${logfile}" "printf n > nonpty.md"
    run_world_pty "${ws_dir}" "${logfile}" "printf p > pty.md"

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

    run_world_pty "${ws_dir}" "${logfile}" "mkdir -p .git; printf x > .git/excluded.md; printf y > .substrate/excluded.md"

    local preview
    preview="$(ws_sync_dry_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${preview}" "pending diff summary (combined)" "exclude preview"
    assert_contains "${preview}" "total_paths: 0" "exclude preview"
    assert_contains "${preview}" "excluded_by_patterns: true" "exclude preview"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "workspace sync applied" "exclude apply"

    local host_check
    host_check="$(
        run_capture "${logfile}" bash -lc "cd '${ws_dir}' && \
if test ! -e .git/excluded.md && test ! -e .substrate/excluded.md; then echo 'excluded_not_on_host=1'; else echo 'excluded_not_on_host=0'; fi"
    )"
    assert_contains "${host_check}" "excluded_not_on_host=1" "exclude host check"

    local world_check
    world_check="$(
        run_capture "${logfile}" bash -lc "cd '${ws_dir}' && '${SUBSTRATE_BIN}' -c \"if test -f .git/excluded.md && test -f .substrate/excluded.md; then echo 'excluded_in_world=1'; else echo 'excluded_in_world=0'; fi; exit 0\""
    )"
    assert_contains "${world_check}" "excluded_in_world=1" "exclude world check"
}

scenario_delete_directory_tree_applies() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: world deletes directory tree => sync deletes on host"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && mkdir -p tree/a && printf 'host\\n' > tree/a/file.md" >/dev/null
    run_world_pty "${ws_dir}" "${logfile}" "rm -rf tree"

    local applied
    applied="$(ws_sync_apply_verbose "${ws_dir}" "${logfile}")"
    assert_contains "${applied}" "workspace sync applied" "tree delete apply"
    assert_any_contains "${applied}" "tree delete apply count" "deletes_applied: 1" "deletes_applied: 2"

    local host_check
    host_check="$(
        run_capture "${logfile}" bash -lc "cd '${ws_dir}' && if test -e tree; then echo 'tree_deleted=0'; else echo 'tree_deleted=1'; fi"
    )"
    assert_contains "${host_check}" "tree_deleted=1" "tree delete host check"
}

scenario_conflict_policy_abort_refuses_on_from_host_conflict() {
    local ws_dir="$1"
    local logfile="$2"
    log "Scenario: conflict_policy=abort + direction=both => sync refuses with exit 5 on conflict"

    # Ensure a baseline exists (and is earlier than our host change) by starting a world session first.
    run_world_nonpty "${ws_dir}" "${logfile}" "true"

    run_capture "${logfile}" bash -lc "cd '${ws_dir}' && printf 'host\\n' > abort.md" >/dev/null
    run_world_pty "${ws_dir}" "${logfile}" "printf world > abort.md"

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
    run_world_pty "${ws_dir}" "${logfile}" "printf world > conflict.md"
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
    run_world_pty "${ws_dir}" "${logfile}" "printf world > conflict.md"
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
CONTINUE_ON_ERROR=0
FAILED_CASES=()

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
        --continue)
            CONTINUE_ON_ERROR=1
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

main() {
    run_case() {
        local case_id="$1"
        local direction="$2"
        local conflict_policy="$3"
        local scenario_fn="$4"

        local ws_dir
        ws_dir="$(mktemp -d /tmp/substrate-ws-sync-matrix.XXXXXX)"
        TMP_DIRS+=("${ws_dir}")

        local logfile="${LOG_DIR}/${case_id}.log"

        local status=0
        (
            set -euo pipefail
            init_workspace "${ws_dir}" "${logfile}"
            set_sync_config "${ws_dir}" "${logfile}" "${direction}" "${conflict_policy}"
            "${scenario_fn}" "${ws_dir}" "${logfile}"
        )
        status=$?

        if [[ "${status}" -eq 0 ]]; then
            log "PASS: ${case_id}"
            return 0
        fi

        echo "CASE FAILED: ${case_id} exit_status=${status}" >&2
        echo "  log: ${logfile}" >&2
        echo "  ws:  ${ws_dir}" >&2
        FAILED_CASES+=("${case_id}")
        if [[ "${CONTINUE_ON_ERROR}" -ne 1 ]]; then
            exit "${status}"
        fi
    }

    run_case "scenario_00_empty" "both" "prefer_host" scenario_empty_sync_preview
    run_case "scenario_01_pty_write" "from_world" "prefer_host" scenario_world_write_via_pty_applies_once
    run_case "scenario_02_nonpty_write" "from_world" "prefer_host" scenario_world_write_via_nonpty_applies
    run_case "scenario_03_host_visible" "both" "prefer_host" scenario_host_create_visible_in_world_no_sync_needed
    run_case "scenario_03b_from_host_keeps_world_only_shadow" "from_host" "prefer_host" scenario_direction_from_host_keeps_non_conflict_world_only_shadow
    run_case "scenario_03c_from_host_conflict_prefer_host_discards" "from_host" "prefer_host" scenario_from_host_conflict_prefer_host_discards_shadowed_path
    run_case "scenario_03d_from_host_conflict_prefer_world_keeps" "from_host" "prefer_world" scenario_from_host_conflict_prefer_world_keeps_shadowed_path
    run_case "scenario_04_world_delete_prefer_world" "both" "prefer_world" scenario_delete_host_file_in_world_prefer_world_applies_delete
    run_case "scenario_05_world_delete_prefer_host" "both" "prefer_host" scenario_delete_host_file_in_world_prefer_host_discards_delete
    run_case "scenario_05b_from_world_delete_prefer_host" "from_world" "prefer_host" scenario_delete_host_file_in_world_direction_from_world_deletes_host_even_prefer_host
    run_case "scenario_05c_world_create_then_delete_noop" "from_world" "prefer_host" scenario_world_create_then_delete_noop
    run_case "scenario_05d_world_rename_mv" "from_world" "prefer_world" scenario_world_rename_mv_results_in_delete_plus_write
    run_case "scenario_05e_world_edits_host_file_prefer_world" "from_world" "prefer_world" scenario_world_modifies_host_file_prefer_world_applies_mod
    run_case "scenario_05f_combined_pty_nonpty_counts" "from_world" "prefer_host" scenario_combined_pty_and_nonpty_shows_combined_counts
    run_case "scenario_05g_excluded_paths" "from_world" "prefer_host" scenario_excluded_paths_not_synced
    run_case "scenario_05h_delete_directory_tree" "from_world" "prefer_world" scenario_delete_directory_tree_applies
    run_case "scenario_05i_abort_policy_refuses" "both" "abort" scenario_conflict_policy_abort_refuses_on_from_host_conflict
    run_case "scenario_06_concurrent_mod_prefer_host" "both" "prefer_host" scenario_concurrent_mod_conflict_prefer_host_skips_apply
    run_case "scenario_07_concurrent_mod_prefer_world" "both" "prefer_world" scenario_concurrent_mod_conflict_prefer_world_applies

    log "DONE"
    log "Logs written to: ${LOG_DIR}"
    if [[ "${#FAILED_CASES[@]}" -gt 0 ]]; then
        echo "FAILURES (${#FAILED_CASES[@]}):" >&2
        printf '  - %s\n' "${FAILED_CASES[@]}" >&2
        return 1
    fi
    return 0
}

main
