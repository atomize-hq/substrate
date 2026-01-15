#!/usr/bin/env bash
set -euo pipefail

feature_dir="docs/project_management/next/workspace-config-policy-unification"

scratch_root="${TMPDIR:-/tmp}/substrate-wcu-smoke"
workspace="${scratch_root}/ws"
home_dir="${scratch_root}/home"

rm -rf "${scratch_root}"
mkdir -p "${workspace}" "${home_dir}"

export SUBSTRATE_HOME="${home_dir}"

cd "${workspace}"

substrate config global init --force
substrate policy global init --force
substrate workspace init .

# Phase B (ADR-0012): edit `world.deps.enabled` via config editor at both scopes.
substrate config global set world.deps.enabled+=bun
substrate config workspace set world.deps.enabled+=node-runtime

# Effective list must include both, de-duped in-order (global then workspace).
substrate config current show --json | jq -e '
  (.world.deps.enabled | type) == "array" and
  (.world.deps.enabled | index("bun") != null) and
  (.world.deps.enabled | index("node-runtime") != null)
' >/dev/null

# Phase A (ADR-0012): `--explain` supports merge_strategy + multi-source provenance.
explain_stderr="$(mktemp)"
substrate config current show --json --explain >/dev/null 2>"${explain_stderr}"
rg -n "concat_dedupe_ordered_set" "${explain_stderr}" >/dev/null
rg -n "global_patch" "${explain_stderr}" >/dev/null
rg -n "workspace_patch" "${explain_stderr}" >/dev/null

rm -f "${explain_stderr}"
echo "OK: ${feature_dir} linux smoke passed"
