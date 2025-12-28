# Manual Testing Playbook — Policy + Config Mental Model Simplification (ADR-0003)

This playbook validates the ADR-0003 CLI and on-disk contracts using disposable temp directories.

## Prerequisites
- `substrate` is on `PATH`.
- `jq` is on `PATH` for `--json` assertions.

## Automation (preferred)

Run the feature-local smoke scripts:
- Linux: `bash docs/project_management/next/policy_and_config_mental_model_simplification/smoke/linux-smoke.sh` → exit `0`
- macOS: `bash docs/project_management/next/policy_and_config_mental_model_simplification/smoke/macos-smoke.sh` → exit `0`
- Windows: `pwsh -File docs/project_management/next/policy_and_config_mental_model_simplification/smoke/windows-smoke.ps1` → exit `0`

## Manual steps (debugging and deeper inspection)

### 1) Workspace init creates the canonical inventory
Commands:
```bash
TMP_HOME="$(mktemp -d)"
TMP_WS="$(mktemp -d)"
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate workspace init "$TMP_WS"
test -f "$TMP_WS/.substrate/workspace.yaml"
test -f "$TMP_WS/.substrate/policy.yaml"
test -d "$TMP_WS/.substrate-git/repo.git"
rg -n '^\\.substrate-git/|^\\.substrate/\\*|^!\\.substrate/workspace\\.yaml|^!\\.substrate/policy\\.yaml' "$TMP_WS/.gitignore"
```

Expected results:
- `substrate workspace init` exits `0`.
- The four ignore rules exist in `$TMP_WS/.gitignore`.

### 2) Config global init/show/set operate on `$SUBSTRATE_HOME/config.yaml`
Commands:
```bash
TMP_HOME="$(mktemp -d)"
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate config global init --force
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate config global show --json | jq -e '.world.enabled|type=="boolean"' >/dev/null
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate config global set world.caged=false
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate config global show --json | jq -e '.world.caged==false' >/dev/null
test -f "$TMP_HOME/env.sh"
```

Expected results:
- All commands exit `0`.
- `$TMP_HOME/env.sh` exists after `config global init` or `config global set`.

### 3) Workspace config show/set operate on `.substrate/workspace.yaml`
Commands:
```bash
TMP_HOME="$(mktemp -d)"
TMP_WS="$(mktemp -d)"
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate workspace init "$TMP_WS"
cd "$TMP_WS"
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate config show --json | jq -e '.policy.mode=="observe"' >/dev/null
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate config set policy.mode=enforce
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate config show --json | jq -e '.policy.mode=="enforce"' >/dev/null
```

Expected results:
- All commands exit `0`.

### 4) Policy init/show/set operate on `.substrate/policy.yaml` (workspace) and `$SUBSTRATE_HOME/policy.yaml` (global)
Commands:
```bash
TMP_HOME="$(mktemp -d)"
TMP_WS="$(mktemp -d)"
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate policy global init --force
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate policy global show --json | jq -e '.world_fs.mode=="writable"' >/dev/null
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate workspace init "$TMP_WS"
cd "$TMP_WS"
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate policy init --force
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate policy show --json | jq -e '.world_fs.isolation=="project"' >/dev/null
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate policy set world_fs.require_world=true
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate policy show --json | jq -e '.world_fs.require_world==true' >/dev/null
```

Expected results:
- All commands exit `0`.

### 5) Legacy `.substrate/settings.yaml` produces a hard error
Commands:
```bash
TMP_HOME="$(mktemp -d)"
TMP_WS="$(mktemp -d)"
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate workspace init "$TMP_WS"
mkdir -p "$TMP_WS/.substrate"
printf '%s\n' 'world:' '  enabled: true' >"$TMP_WS/.substrate/settings.yaml"
cd "$TMP_WS"
set +e
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate config show >/dev/null 2>&1
code=$?
set -e
test "$code" -eq 2
```

Expected results:
- `substrate config show` exits `2`.

### 6) `substrate world enable` exposes `--home` and rejects `--prefix`
Commands:
```bash
substrate world enable --help | rg -n -- '--home' >/dev/null
set +e
substrate world enable --help | rg -n -- '--prefix' >/dev/null
code=$?
set -e
test "$code" -ne 0
```

Expected results:
- `--home` exists in help output.
- `--prefix` does not exist in help output.

