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

# Workspace init flags: `--examples` creates non-active templates and Substrate does not read them for behavior.
substrate workspace init . --examples
test -f .substrate/workspace.example.yaml
test -f .substrate/policy.example.yaml
printf ':\n' > .substrate/workspace.example.yaml
printf ':\n' > .substrate/policy.example.yaml
substrate config current show --json >/dev/null
substrate policy current show --json >/dev/null

# Workspace init flags: `--force` repairs missing entries and does not overwrite existing non-empty patch files.
substrate config workspace set world.caged=false >/dev/null
workspace_yaml_before="$(mktemp)"
cp -a .substrate/workspace.yaml "${workspace_yaml_before}"
rm -rf .substrate/git/repo.git
rm -f .substrate/policy.yaml
substrate workspace init . --force
test -d .substrate/git/repo.git
test -f .substrate/policy.yaml
diff -u "${workspace_yaml_before}" .substrate/workspace.yaml >/dev/null

# Phase B (ADR-0012): edit `world.deps.enabled` via config editor at both scopes (include a deliberate duplicate across scopes).
substrate config global set world.deps.enabled+=bun world.deps.enabled+=node-runtime
substrate config workspace set world.deps.enabled+=node-runtime world.deps.enabled+=deno

# Effective list must include all items and preserve ordered-set behavior (global then workspace; duplicate de-duped).
substrate config current show --json | jq -e '
  .world.deps.enabled as $a |
  ($a|type) == "array" and
  ($a|index("bun")) as $i_bun |
  ($a|index("node-runtime")) as $i_node |
  ($a|index("deno")) as $i_deno |
  ($i_bun!=null and $i_node!=null and $i_deno!=null and $i_bun < $i_node and $i_node < $i_deno)
' >/dev/null

# Determinism/idempotence: re-running show + explain without changes yields identical outputs.
effective_1="$(mktemp)"
effective_2="$(mktemp)"
substrate config current show --json >"${effective_1}"
substrate config current show --json >"${effective_2}"
diff -u "${effective_1}" "${effective_2}" >/dev/null

explain_1="$(mktemp)"
explain_2="$(mktemp)"
substrate config current show --json --explain >/dev/null 2>"${explain_1}"
substrate config current show --json --explain >/dev/null 2>"${explain_2}"
diff -u "${explain_1}" "${explain_2}" >/dev/null

# Phase A (ADR-0012): `--explain` supports merge_strategy + multi-source provenance.
rg -n "concat_dedupe_ordered_set" "${explain_1}" >/dev/null
python - <<'PY' "${explain_1}"
import sys
text = open(sys.argv[1], "r", encoding="utf-8").read()
g = text.find("global_patch")
w = text.find("workspace_patch")
if g == -1 or w == -1 or g >= w:
    raise SystemExit("expected global_patch to appear before workspace_patch in --explain output")
PY

# Workspace disabled marker must ignore workspace contribution for this merge key.
substrate workspace disable .
substrate config current show --json | jq -e '
  (.world.deps.enabled | index("bun") != null) and
  (.world.deps.enabled | index("deno") == null)
' >/dev/null
explain_disabled="$(mktemp)"
substrate config current show --json --explain >/dev/null 2>"${explain_disabled}"
rg -n "global_patch" "${explain_disabled}" >/dev/null
if rg -n "workspace_patch" "${explain_disabled}" >/dev/null; then
  echo "expected workspace_patch to be absent when workspace is disabled" >&2
  exit 1
fi
substrate workspace enable .

# List removal operator syntax (`-=`) removes the exact item from the patch list.
substrate config workspace set world.deps.enabled-=deno
substrate config current show --json | jq -e '
  (.world.deps.enabled | index("bun") != null) and
  (.world.deps.enabled | index("deno") == null)
' >/dev/null

# Workspace reset must remove the key from the workspace patch (inherit-only) while global still contributes.
substrate config workspace reset world.deps.enabled
substrate config current show --json | jq -e '
  (.world.deps.enabled | index("bun") != null) and
  (.world.deps.enabled | index("deno") == null)
' >/dev/null

rm -f "${effective_1}" "${effective_2}" "${explain_1}" "${explain_2}" "${explain_disabled}" "${workspace_yaml_before}"
echo "OK: ${feature_dir} linux smoke passed"
