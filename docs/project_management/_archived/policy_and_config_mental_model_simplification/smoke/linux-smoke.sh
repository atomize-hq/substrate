#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: policy/config mental model linux smoke (not Linux)"
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

assert_file() {
  local path="$1"
  test -f "$path" || fail "missing file: $path"
}

assert_dir() {
  local path="$1"
  test -d "$path" || fail "missing dir: $path"
}

run_expect_exit() {
  local expected="$1"
  shift
  set +e
  "$@"
  local got=$?
  set -e
  [[ "$got" == "$expected" ]] || fail "expected exit $expected, got $got: $*"
}

run_expect_nonzero() {
  set +e
  "$@"
  local got=$?
  set -e
  [[ "$got" != "0" ]] || fail "expected nonzero exit, got 0: $*"
}

need_cmd substrate
need_cmd jq
need_cmd mktemp
need_cmd grep
need_cmd sha256sum
need_cmd awk
need_cmd find

TMP_HOME="$(mktemp -d)"
TMP_WS="$(mktemp -d)"
TMP_NOWS="$(mktemp -d)"
TMP_HOME2="$(mktemp -d)"
TMP_WS2="$(mktemp -d)"
TMP_PREFIX_IGN="$(mktemp -d)"
cleanup() { rm -rf "$TMP_HOME" "$TMP_WS" "$TMP_NOWS" "$TMP_HOME2" "$TMP_WS2" "$TMP_PREFIX_IGN"; }
trap cleanup EXIT

export SUBSTRATE_HOME="$TMP_HOME"
export HOME="$TMP_HOME"

echo "SMOKE: PCM0/PCM1 inventory (global + workspace)"
substrate config global init --force >/dev/null
assert_file "$SUBSTRATE_HOME/config.yaml"
assert_file "$SUBSTRATE_HOME/env.sh"

substrate policy global init --force >/dev/null
assert_file "$SUBSTRATE_HOME/policy.yaml"

substrate workspace init "$TMP_WS" >/dev/null
assert_file "$TMP_WS/.substrate/workspace.yaml"
assert_file "$TMP_WS/.substrate/policy.yaml"
assert_dir "$TMP_WS/.substrate-git/repo.git"

grep -qxF '.substrate-git/' "$TMP_WS/.gitignore"
grep -qxF '.substrate/*' "$TMP_WS/.gitignore"
grep -qxF '!.substrate/workspace.yaml' "$TMP_WS/.gitignore"
grep -qxF '!.substrate/policy.yaml' "$TMP_WS/.gitignore"

echo "SMOKE: PCM0 workspace walk-up discovery"
mkdir -p "$TMP_WS/a/b"
cd "$TMP_WS/a/b"
substrate config show --json | jq -e '.world.anchor_mode=="workspace"' >/dev/null
substrate policy show --json | jq -e '.world_fs.isolation=="project"' >/dev/null
substrate config set policy.mode=enforce >/dev/null
substrate config show --json | jq -e '.policy.mode=="enforce"' >/dev/null

echo "SMOKE: PCM0 protected excludes always present"
substrate config show --json | jq -e '.sync.exclude | index(".git/**") != null' >/dev/null
substrate config show --json | jq -e '.sync.exclude | index(".substrate/**") != null' >/dev/null
substrate config show --json | jq -e '.sync.exclude | index(".substrate-git/**") != null' >/dev/null

echo "SMOKE: PCM0 precedence (workspace < env < CLI)"
cd "$TMP_WS"
substrate config global set world.caged=true >/dev/null
substrate config set world.caged=false >/dev/null
substrate config show --json | jq -e '.world.caged==false' >/dev/null
SUBSTRATE_CAGED=1 substrate config show --json | jq -e '.world.caged==true' >/dev/null
SUBSTRATE_CAGED=1 substrate --uncaged config show --json | jq -e '.world.caged==false' >/dev/null

echo "SMOKE: PCM0 precedence for anchor_mode/anchor_path"
substrate config global set world.anchor_mode=workspace >/dev/null
substrate config set world.anchor_mode=follow-cwd >/dev/null
substrate config show --json | jq -e '.world.anchor_mode=="follow-cwd"' >/dev/null
SUBSTRATE_ANCHOR_MODE=workspace substrate config show --json | jq -e '.world.anchor_mode=="workspace"' >/dev/null
substrate --anchor-mode custom --anchor-path "$TMP_WS" config show --json | jq -e '.world.anchor_mode=="custom"' >/dev/null
substrate --anchor-mode custom --anchor-path "$TMP_WS" config show --json | jq -e --arg p "$TMP_WS" '.world.anchor_path==$p' >/dev/null

echo "SMOKE: PCM0 list update + protected excludes non-removable"
substrate config set sync.exclude+=tmp/** >/dev/null
substrate config show --json | jq -e '.sync.exclude | index("tmp/**") != null' >/dev/null
substrate config set 'sync.exclude-=.git/**' >/dev/null
substrate config show --json | jq -e '.sync.exclude | index(".git/**") != null' >/dev/null
SUBSTRATE_SYNC_EXCLUDE='foo/**,bar/**' substrate config show --json | jq -e '.sync.exclude | index(".git/**") != null' >/dev/null
SUBSTRATE_SYNC_EXCLUDE='foo/**,bar/**' substrate config show --json | jq -e '.sync.exclude | index("foo/**") != null' >/dev/null

echo "SMOKE: PCM0 workspace-scope commands fail outside a workspace"
cd "$TMP_NOWS"
run_expect_exit 2 substrate config show --json >/dev/null 2>&1
run_expect_exit 2 substrate policy show --json >/dev/null 2>&1

echo "SMOKE: PCM0 nested workspace init refusal (exit 2; no writes)"
cd "$TMP_WS"
NESTED="$TMP_WS/nested"
mkdir -p "$NESTED"
run_expect_exit 2 substrate workspace init "$NESTED" >/dev/null 2>&1
test ! -e "$NESTED/.substrate/workspace.yaml" || fail "nested init wrote workspace marker"

echo "SMOKE: PCM0 legacy .substrate/settings.yaml hard error (exit 2)"
cd "$TMP_WS"
mkdir -p .substrate
printf '%s\n' 'world:' '  enabled: true' > .substrate/settings.yaml
run_expect_exit 2 substrate config show >/dev/null 2>&1
rm -f .substrate/settings.yaml

echo "SMOKE: PCM1 discovery precedence (workspace overrides global)"
cd "$TMP_WS"
substrate policy global set name=global >/dev/null
substrate policy set name=workspace >/dev/null
substrate policy show --json | jq -e '.name=="workspace"' >/dev/null

echo "SMOKE: PCM1 invariants fail closed"
cat > "$TMP_WS/.substrate/policy.yaml" <<'YAML'
id: "p"
name: "p"
world_fs:
  mode: read_only
  isolation: workspace
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []
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
run_expect_exit 2 substrate policy show --json >/dev/null 2>&1
substrate policy init --force >/dev/null

echo "SMOKE: PCM2 policy.mode disabled|observe|enforce"
cat > "$TMP_WS/.substrate/policy.yaml" <<'YAML'
id: "p"
name: "p"
world_fs:
  mode: writable
  isolation: workspace
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []
net_allowed: []
cmd_allowed: []
cmd_denied: ["echo*"]
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

substrate config set policy.mode=disabled >/dev/null
OUT_DISABLED="$TMP_WS/out_disabled.txt"
rm -f "$OUT_DISABLED"
run_expect_exit 0 substrate --no-world --command "echo disabled > \"$OUT_DISABLED\"" >/dev/null
assert_file "$OUT_DISABLED"

substrate config set policy.mode=observe >/dev/null
OUT_OBS="$TMP_WS/out_observe.txt"
rm -f "$OUT_OBS"
run_expect_exit 0 substrate --no-world --command "echo observe > \"$OUT_OBS\"" >/dev/null
assert_file "$OUT_OBS"

substrate config set policy.mode=enforce >/dev/null
OUT_ENF="$TMP_WS/out_enforce.txt"
rm -f "$OUT_ENF"
run_expect_nonzero substrate --no-world --command "echo enforce > \"$OUT_ENF\"" >/dev/null 2>&1
test ! -e "$OUT_ENF" || fail "enforce-mode deny executed command unexpectedly"

echo "SMOKE: PCM2 allowlist semantics (non-empty cmd_allowed blocks non-match in enforce)"
cat > "$TMP_WS/.substrate/policy.yaml" <<'YAML'
id: "p"
name: "p"
world_fs:
  mode: writable
  isolation: workspace
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []
net_allowed: []
cmd_allowed: ["echo*"]
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
OUT_ALLOW="$TMP_WS/out_allow.txt"
rm -f "$OUT_ALLOW"
run_expect_exit 0 substrate --no-world --command "echo ok > \"$OUT_ALLOW\"" >/dev/null
assert_file "$OUT_ALLOW"
OUT_BLOCK="$TMP_WS/out_block.txt"
rm -f "$OUT_BLOCK"
run_expect_nonzero substrate --no-world --command "date > \"$OUT_BLOCK\"" >/dev/null 2>&1
test ! -e "$OUT_BLOCK" || fail "enforce-mode allowlist executed non-allowed command unexpectedly"

echo "SMOKE: PCM2 requires-world constraint fails closed in enforce with --no-world"
cat > "$TMP_WS/.substrate/policy.yaml" <<'YAML'
id: "p"
name: "p"
world_fs:
  mode: writable
  isolation: workspace
  require_world: true
  read_allowlist: ["*"]
  write_allowlist: []
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
OUT_REQWORLD="$TMP_WS/out_reqworld.txt"
rm -f "$OUT_REQWORLD"
run_expect_nonzero substrate --no-world --command "echo reqworld > \"$OUT_REQWORLD\"" >/dev/null 2>&1
test ! -e "$OUT_REQWORLD" || fail "requires-world conflict executed command unexpectedly"

echo "SMOKE: PCM3 env.sh stable across runtime; manager_env.sh runtime-generated; SUBSTRATE_MANAGER_ENV ignored"
assert_file "$SUBSTRATE_HOME/env.sh"
ENV_HASH_BEFORE="$(sha256sum "$SUBSTRATE_HOME/env.sh" | awk '{print $1}')"
rm -f "$SUBSTRATE_HOME/manager_env.sh"
SUBSTRATE_MANAGER_ENV="$SUBSTRATE_HOME/override_manager_env.sh" run_expect_exit 0 substrate --no-world --command "true" >/dev/null
ENV_HASH_AFTER="$(sha256sum "$SUBSTRATE_HOME/env.sh" | awk '{print $1}')"
[[ "$ENV_HASH_BEFORE" == "$ENV_HASH_AFTER" ]] || fail "env.sh changed during runtime execution"
assert_file "$SUBSTRATE_HOME/manager_env.sh"
test ! -e "$SUBSTRATE_HOME/override_manager_env.sh" || fail "SUBSTRATE_MANAGER_ENV override path was used"
grep -q "env.sh" "$SUBSTRATE_HOME/manager_env.sh" || fail "manager_env.sh does not reference env.sh"
grep -q ".substrate_bashenv" "$SUBSTRATE_HOME/manager_env.sh" || fail "manager_env.sh does not reference .substrate_bashenv"
grep -q "export SUBSTRATE_HOME=" "$SUBSTRATE_HOME/env.sh" || fail "env.sh missing SUBSTRATE_HOME export"
grep -q "export SUBSTRATE_WORLD=" "$SUBSTRATE_HOME/env.sh" || fail "env.sh missing SUBSTRATE_WORLD export"
grep -q "export SUBSTRATE_CAGED=" "$SUBSTRATE_HOME/env.sh" || fail "env.sh missing SUBSTRATE_CAGED export"
grep -q "export SUBSTRATE_ANCHOR_MODE=" "$SUBSTRATE_HOME/env.sh" || fail "env.sh missing SUBSTRATE_ANCHOR_MODE export"
grep -q "export SUBSTRATE_ANCHOR_PATH=" "$SUBSTRATE_HOME/env.sh" || fail "env.sh missing SUBSTRATE_ANCHOR_PATH export"
grep -q "export SUBSTRATE_POLICY_MODE=" "$SUBSTRATE_HOME/env.sh" || fail "env.sh missing SUBSTRATE_POLICY_MODE export"

echo "SMOKE: PCM3 world enable help surface + dry-run no writes"
substrate world enable --help | grep -q -- '--home'
if substrate world enable --help | grep -q -- '--prefix'; then
  fail "--prefix present in help output"
fi
run_expect_exit 2 substrate world enable --prefix "$TMP_HOME2" --dry-run >/dev/null 2>&1

export SUBSTRATE_HOME="$TMP_HOME2"
export HOME="$TMP_HOME2"
run_expect_exit 0 substrate world enable --dry-run --home "$TMP_HOME2" >/dev/null
test -z "$(find "$TMP_HOME2" -mindepth 1 -print -quit)" || fail "world enable --dry-run performed filesystem writes"

echo "SMOKE: PCM3 SUBSTRATE_PREFIX ignored"
export SUBSTRATE_HOME="$TMP_HOME2"
export HOME="$TMP_HOME2"
export SUBSTRATE_PREFIX="$TMP_PREFIX_IGN"
run_expect_exit 0 substrate config global init --force >/dev/null
assert_file "$TMP_HOME2/config.yaml"
test ! -e "$TMP_PREFIX_IGN/config.yaml" || fail "SUBSTRATE_PREFIX affected config.yaml location"
unset SUBSTRATE_PREFIX

echo "SMOKE: PCM0/PCM1 strict parsing hard errors (global config/policy)"
run_expect_exit 0 substrate config global init --force >/dev/null
cat > "$TMP_HOME2/config.yaml" <<'YAML'
bogus: 1
YAML
run_expect_exit 2 substrate config global show --json >/dev/null 2>&1

cat > "$TMP_HOME2/config.yaml" <<'YAML'
world:
  enabled: "nope"
  anchor_mode: workspace
  anchor_path: ""
  caged: true
policy:
  mode: observe
sync:
  auto_sync: false
  direction: from_world
  conflict_policy: prefer_host
  exclude: []
YAML
run_expect_exit 2 substrate config global show --json >/dev/null 2>&1

run_expect_exit 0 substrate policy global init --force >/dev/null
cat > "$TMP_HOME2/policy.yaml" <<'YAML'
id: "p"
name: "p"
bogus: 1
world_fs:
  mode: writable
  isolation: workspace
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []
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
run_expect_exit 2 substrate policy global show --json >/dev/null 2>&1

cat > "$TMP_HOME2/policy.yaml" <<'YAML'
id: "p"
name: "p"
world_fs:
  mode: writable
  isolation: workspace
  require_world: "no"
  read_allowlist: ["*"]
  write_allowlist: []
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
run_expect_exit 2 substrate policy global show --json >/dev/null 2>&1

echo "SMOKE: PCM0 strict parsing hard errors (workspace config/policy)"
export SUBSTRATE_HOME="$TMP_HOME2"
export HOME="$TMP_HOME2"
substrate workspace init "$TMP_WS2" >/dev/null
cd "$TMP_WS2"
cat > "$TMP_WS2/.substrate/workspace.yaml" <<'YAML'
world:
  enabled: true
  root_mode: project
  root_path: ""
  caged: true
policy:
  mode: observe
sync:
  auto_sync: false
  direction: from_world
  conflict_policy: prefer_host
  exclude: []
YAML
run_expect_exit 2 substrate config show --json >/dev/null 2>&1

cat > "$TMP_WS2/.substrate/policy.yaml" <<'YAML'
id: "p"
name: "p"
world_fs:
  mode: writable
  isolation: workspace
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []
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
run_expect_exit 2 substrate policy show --json >/dev/null 2>&1

echo "SMOKE: PCM2 CLI flag existence and precedence for policy.mode"
substrate --help | grep -q -- '--policy-mode'
SUBSTRATE_POLICY_MODE=disabled substrate --policy-mode enforce config global show --json | jq -e '.policy.mode=="enforce"' >/dev/null
SUBSTRATE_POLICY_MODE=enforce substrate --policy-mode disabled config global show --json | jq -e '.policy.mode=="disabled"' >/dev/null
cd "$TMP_WS"
SUBSTRATE_POLICY_MODE=disabled substrate --policy-mode enforce config show --json | jq -e '.policy.mode=="enforce"' >/dev/null

echo "OK: policy/config mental model linux smoke (expanded)"
