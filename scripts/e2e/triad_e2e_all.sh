#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/e2e/triad_e2e_all.sh [phase1 options] -- [phase2 options]

Example:
  scripts/e2e/triad_e2e_all.sh --feature e2e-demo --push-orch --codex-jsonl -- \
    --platform-fixes linux,macos,windows --push-orch --cleanup

Notes:
  - Everything before `--` is passed to Phase 1.
  - Everything after `--` is passed to Phase 2.
  - Phase 2 always receives `--feature-dir docs/project_management/next/<feature>` based on Phase 1 output.
USAGE
}

die() {
    echo "ERROR: $*" >&2
    exit 2
}

require_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        die "Missing dependency: $1"
    fi
}

require_cmd awk
require_cmd bash

phase1_args=()
phase2_args=()
split=0

while [[ $# -gt 0 ]]; do
    if [[ "$1" == "--" ]]; then
        split=1
        shift 1
        continue
    fi
    if [[ "${split}" -eq 0 ]]; then
        phase1_args+=("$1")
    else
        phase2_args+=("$1")
    fi
    shift 1
done

out="$(scripts/e2e/triad_e2e_phase1.sh "${phase1_args[@]}")"
echo "${out}"

feature_dir="$(printf '%s\n' "${out}" | awk -F= '$1=="FEATURE_DIR"{print $2}')"
if [[ -z "${feature_dir}" ]]; then
    die "Could not parse FEATURE_DIR from phase1 output"
fi

scripts/e2e/triad_e2e_phase2.sh --feature-dir "${feature_dir}" "${phase2_args[@]}"

