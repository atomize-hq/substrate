#!/usr/bin/env bash
set -euo pipefail

feature_dir="docs/project_management/next/workspace-config-policy-unification"

slice_id="${SUBSTRATE_SMOKE_SLICE_ID:-}"

scratch_root="$(mktemp -d "${TMPDIR:-/tmp}/substrate-wcu-smoke.XXXXXX")"
workspace="${scratch_root}/ws"
home_dir="${scratch_root}/home"

trap 'rm -rf "${scratch_root}"' EXIT
mkdir -p "${workspace}" "${home_dir}"

export SUBSTRATE_HOME="${home_dir}"

cd "${workspace}"

echo "INFO: ${feature_dir} linux smoke (SUBSTRATE_SMOKE_SLICE_ID='${slice_id}')" >&2

init_minimal() {
  substrate config global init --force
  substrate policy global init --force
  substrate workspace init .
}

run_wcu1_smoke() {
  init_minimal

  test -f .substrate/workspace.yaml
  test -f .substrate/policy.yaml
  test -d .substrate/git/repo.git

  rg -n '^\\.substrate/$' .gitignore >/dev/null
  rg -n '^!\\.substrate/workspace\\.yaml$' .gitignore >/dev/null
  rg -n '^!\\.substrate/policy\\.yaml$' .gitignore >/dev/null

  # Workspace init flags: `--examples` creates non-active templates and Substrate does not read them for behavior.
  substrate workspace init . --examples
  test -f .substrate/workspace.example.yaml
  test -f .substrate/policy.example.yaml
  printf 'this: is: invalid: yaml: [\n' > .substrate/workspace.example.yaml
  printf 'this: is: invalid: yaml: [\n' > .substrate/policy.example.yaml

  # Workspace init flags: `--force` repairs missing entries and does not overwrite existing non-empty patch files.
  printf 'wcu1_sentinel: true\n' > .substrate/workspace.yaml
  printf 'wcu1_sentinel: true\n' > .substrate/policy.yaml
  ws_hash_before="$(sha256sum .substrate/workspace.yaml | awk '{print $1}')"
  pol_hash_before="$(sha256sum .substrate/policy.yaml | awk '{print $1}')"
  rm -rf .substrate/git/repo.git
  substrate workspace init . --force
  test -d .substrate/git/repo.git
  ws_hash_after="$(sha256sum .substrate/workspace.yaml | awk '{print $1}')"
  pol_hash_after="$(sha256sum .substrate/policy.yaml | awk '{print $1}')"
  if [ "${ws_hash_before}" != "${ws_hash_after}" ]; then
    echo "expected .substrate/workspace.yaml unchanged by workspace init --force" >&2
    exit 1
  fi
  if [ "${pol_hash_before}" != "${pol_hash_after}" ]; then
    echo "expected .substrate/policy.yaml unchanged by workspace init --force" >&2
    exit 1
  fi

  # Nested workspace refusal (when workspace is enabled).
  mkdir -p nested_refused
  set +e
  substrate workspace init nested_refused >/dev/null 2>&1
  ec=$?
  set -e
  if [ "${ec}" -ne 2 ]; then
    echo "expected exit 2 for nested workspace init refusal, got ${ec}" >&2
    exit 1
  fi

  # Disabled marker makes the parent workspace ignored for discovery.
  substrate workspace disable .
  test -f .substrate/workspace.disabled
  mkdir -p nested_ok
  substrate workspace init nested_ok
  test -f nested_ok/.substrate/workspace.yaml
  test -f nested_ok/.substrate/policy.yaml
  test -d nested_ok/.substrate/git/repo.git
  substrate workspace enable .
  if test -f .substrate/workspace.disabled; then
    echo "expected workspace.disabled to be removed by workspace enable" >&2
    exit 1
  fi

  echo "OK: ${feature_dir} linux smoke passed (slice=WCU1)"
}

run_wcu2_smoke() {
  init_minimal

  # Configure Phase A inputs using patch files directly (do not require config editor / set/reset surfaces).
  cat >"${SUBSTRATE_HOME}/config.yaml" <<'YAML'
world:
  deps:
    enabled:
      - bun
      - node-runtime
    inventory_mode: merged
    builtins: enabled
YAML

  cat >".substrate/workspace.yaml" <<'YAML'
world:
  deps:
    enabled:
      - node-runtime
      - deno
    inventory_mode: workspace_only
    builtins: disabled
YAML

  # Effective list must include all items and preserve ordered-set behavior (global then workspace; duplicate de-duped).
  substrate config current show --json | jq -e '
    .world.deps.enabled as $a |
    ($a|type) == "array" and
    ($a|index("bun")) as $i_bun |
    ($a|index("node-runtime")) as $i_node |
    ($a|index("deno")) as $i_deno |
    ($i_bun!=null and $i_node!=null and $i_deno!=null and $i_bun < $i_node and $i_node < $i_deno)
  ' >/dev/null

  # Effective enum keys must reflect workspace overrides when workspace is enabled.
  substrate config current show --json | jq -e '
    .world.deps.inventory_mode == "workspace_only" and
    .world.deps.builtins == "disabled"
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

  # Replace-key provenance: enum keys report merge_strategy=replace and exactly one contributing layer (workspace_patch).
  python - <<'PY' "${explain_1}"
import json, sys
p = sys.argv[1]
text = open(p, "r", encoding="utf-8").read()
i = text.find("{")
if i == -1:
    raise SystemExit("failed to locate JSON object in --explain stderr")
obj = json.loads(text[i:])
def chk(key, layer):
    entry = obj["keys"][key]
    if entry["merge_strategy"] != "replace":
        raise SystemExit(f"{key}: expected merge_strategy=replace")
    sources = entry["sources"]
    if len(sources) != 1 or sources[0]["layer"] != layer:
        raise SystemExit(f"{key}: expected exactly one source layer {layer}")
chk("world.deps.inventory_mode", "workspace_patch")
chk("world.deps.builtins", "workspace_patch")
print("OK: enum keys replace provenance (workspace_patch)")
PY

  # Workspace disabled marker must ignore workspace contribution for this merge key.
  substrate workspace disable .
  substrate config current show --json | jq -e '
    (.world.deps.enabled | index("bun") != null) and
    (.world.deps.enabled | index("deno") == null)
  ' >/dev/null

  # When workspace is disabled, enum keys fall back to global values.
  substrate config current show --json | jq -e '
    .world.deps.inventory_mode == "merged" and
    .world.deps.builtins == "enabled"
  ' >/dev/null

  explain_disabled="$(mktemp)"
  substrate config current show --json --explain >/dev/null 2>"${explain_disabled}"
  rg -n "global_patch" "${explain_disabled}" >/dev/null
  if rg -n "workspace_patch" "${explain_disabled}" >/dev/null; then
    echo "expected workspace_patch to be absent when workspace is disabled" >&2
    exit 1
  fi

  # When workspace is disabled, enum keys report a single contributing layer (global_patch).
  python - <<'PY' "${explain_disabled}"
import json, sys
p = sys.argv[1]
text = open(p, "r", encoding="utf-8").read()
i = text.find("{")
if i == -1:
    raise SystemExit("failed to locate JSON object in --explain stderr")
obj = json.loads(text[i:])
def chk(key, layer):
    entry = obj["keys"][key]
    if entry["merge_strategy"] != "replace":
        raise SystemExit(f"{key}: expected merge_strategy=replace")
    sources = entry["sources"]
    if len(sources) != 1 or sources[0]["layer"] != layer:
        raise SystemExit(f"{key}: expected exactly one source layer {layer}")
chk("world.deps.inventory_mode", "global_patch")
chk("world.deps.builtins", "global_patch")
print("OK: enum keys replace provenance (global_patch when workspace disabled)")
PY

  substrate workspace enable .

  rm -f "${effective_1}" "${effective_2}" "${explain_1}" "${explain_2}" "${explain_disabled}"
  echo "OK: ${feature_dir} linux smoke passed (slice=WCU2)"
}

run_full_smoke() {
  init_minimal

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

# World-deps contract parity: enum keys (replace precedence).
substrate config global set world.deps.inventory_mode=merged world.deps.builtins=enabled >/dev/null
substrate config workspace set world.deps.inventory_mode=workspace_only world.deps.builtins=disabled >/dev/null

# Effective list must include all items and preserve ordered-set behavior (global then workspace; duplicate de-duped).
substrate config current show --json | jq -e '
  .world.deps.enabled as $a |
  ($a|type) == "array" and
  ($a|index("bun")) as $i_bun |
  ($a|index("node-runtime")) as $i_node |
  ($a|index("deno")) as $i_deno |
  ($i_bun!=null and $i_node!=null and $i_deno!=null and $i_bun < $i_node and $i_node < $i_deno)
' >/dev/null

# Effective enum keys must reflect workspace overrides when workspace is enabled.
substrate config current show --json | jq -e '
  .world.deps.inventory_mode == "workspace_only" and
  .world.deps.builtins == "disabled"
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

# Replace-key provenance: enum keys report merge_strategy=replace and exactly one contributing layer (workspace_patch).
python - <<'PY' "${explain_1}"
import json, sys
p = sys.argv[1]
text = open(p, "r", encoding="utf-8").read()
i = text.find("{")
if i == -1:
    raise SystemExit("failed to locate JSON object in --explain stderr")
obj = json.loads(text[i:])
def chk(key, layer):
    entry = obj["keys"][key]
    if entry["merge_strategy"] != "replace":
        raise SystemExit(f"{key}: expected merge_strategy=replace")
    sources = entry["sources"]
    if len(sources) != 1 or sources[0]["layer"] != layer:
        raise SystemExit(f"{key}: expected exactly one source layer {layer}")
chk("world.deps.inventory_mode", "workspace_patch")
chk("world.deps.builtins", "workspace_patch")
print("OK: enum keys replace provenance (workspace_patch)")
PY

# Workspace disabled marker must ignore workspace contribution for this merge key.
substrate workspace disable .
substrate config current show --json | jq -e '
  (.world.deps.enabled | index("bun") != null) and
  (.world.deps.enabled | index("deno") == null)
' >/dev/null

# When workspace is disabled, enum keys fall back to global values.
substrate config current show --json | jq -e '
  .world.deps.inventory_mode == "merged" and
  .world.deps.builtins == "enabled"
' >/dev/null

explain_disabled="$(mktemp)"
substrate config current show --json --explain >/dev/null 2>"${explain_disabled}"
rg -n "global_patch" "${explain_disabled}" >/dev/null
if rg -n "workspace_patch" "${explain_disabled}" >/dev/null; then
  echo "expected workspace_patch to be absent when workspace is disabled" >&2
  exit 1
fi

# When workspace is disabled, enum keys report a single contributing layer (global_patch).
python - <<'PY' "${explain_disabled}"
import json, sys
p = sys.argv[1]
text = open(p, "r", encoding="utf-8").read()
i = text.find("{")
if i == -1:
    raise SystemExit("failed to locate JSON object in --explain stderr")
obj = json.loads(text[i:])
def chk(key, layer):
    entry = obj["keys"][key]
    if entry["merge_strategy"] != "replace":
        raise SystemExit(f"{key}: expected merge_strategy=replace")
    sources = entry["sources"]
    if len(sources) != 1 or sources[0]["layer"] != layer:
        raise SystemExit(f"{key}: expected exactly one source layer {layer}")
chk("world.deps.inventory_mode", "global_patch")
chk("world.deps.builtins", "global_patch")
print("OK: enum keys replace provenance (global_patch when workspace disabled)")
PY

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

# Invalid enum values must be exit 2 and perform no writes (patch bytes unchanged).
hash_file() {
  python -c 'import hashlib,sys;print(hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest())' "$1"
}

cfg="${SUBSTRATE_HOME}/config.yaml"
ws=".substrate/workspace.yaml"

cfg_hash_before="$(hash_file "${cfg}")"
set +e
substrate config global set world.deps.builtins=bogus >/dev/null 2>&1
ec=$?
set -e
if [ "${ec}" -ne 2 ]; then
  echo "expected exit 2 for invalid enum value (world.deps.builtins=bogus), got ${ec}" >&2
  exit 1
fi
cfg_hash_after="$(hash_file "${cfg}")"
if [ "${cfg_hash_before}" != "${cfg_hash_after}" ]; then
  echo "expected global patch bytes unchanged after invalid enum value" >&2
  exit 1
fi

ws_hash_before="$(hash_file "${ws}")"
set +e
substrate config workspace set world.deps.inventory_mode=nope >/dev/null 2>&1
ec=$?
set -e
if [ "${ec}" -ne 2 ]; then
  echo "expected exit 2 for invalid enum value (world.deps.inventory_mode=nope), got ${ec}" >&2
  exit 1
fi
ws_hash_after="$(hash_file "${ws}")"
if [ "${ws_hash_before}" != "${ws_hash_after}" ]; then
  echo "expected workspace patch bytes unchanged after invalid enum value" >&2
  exit 1
fi

rm -f "${effective_1}" "${effective_2}" "${explain_1}" "${explain_2}" "${explain_disabled}" "${workspace_yaml_before}"
echo "OK: ${feature_dir} linux smoke passed (slice=${slice_id:-full})"
}

case "${slice_id}" in
  "") run_full_smoke ;;
  WCU1) run_wcu1_smoke ;;
  WCU2) run_wcu2_smoke ;;
  WCU3|WCU4|WCU5) run_full_smoke ;;
  *)
    echo "unknown SUBSTRATE_SMOKE_SLICE_ID='${slice_id}' (expected WCU1..WCU5 or empty)" >&2
    exit 2
    ;;
esac
