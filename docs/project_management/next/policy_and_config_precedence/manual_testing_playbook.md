# Manual Testing Playbook — Policy + Config Precedence (ADR-0005)

This playbook validates the precedence correction that makes workspace config override `SUBSTRATE_*` env vars when a workspace exists.

## Prerequisites
- `substrate` is on `PATH`.
- Linux/macOS: `jq` is on `PATH` for JSON assertions.

## Automation (preferred)
Run the feature-local smoke scripts:
- Linux: `bash docs/project_management/next/policy_and_config_precedence/smoke/linux-smoke.sh` → exit `0`
- macOS: `bash docs/project_management/next/policy_and_config_precedence/smoke/macos-smoke.sh` → exit `0`
- Windows: `pwsh -File docs/project_management/next/policy_and_config_precedence/smoke/windows-smoke.ps1` → exit `0`

## Manual steps (debugging and deeper inspection)

### 1) Workspace config overrides env exports in a workspace
Commands:
```bash
TMP_HOME="$(mktemp -d)"
TMP_WS="$(mktemp -d)"

SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate workspace init "$TMP_WS"
cd "$TMP_WS"

SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate config set world.caged=false
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" SUBSTRATE_CAGED=1 substrate config show --json | jq -e '.world.caged==false' >/dev/null
```

Expected results:
- All commands exit `0`.
- The final `config show --json` output contains `world.caged=false`.

### 2) Workspace-scoped commands require a workspace (unchanged)
Commands:
```bash
TMP_HOME="$(mktemp -d)"
TMP_NOWS="$(mktemp -d)"

set +e
SUBSTRATE_HOME="$TMP_HOME" HOME="$TMP_HOME" substrate config show --json >/dev/null 2>&1
code=$?
set -e
test "$code" -eq 2
```

Expected results:
- `substrate config show --json` exits `2`.

## Cross-platform smoke via CI (preferred for parity)
Dispatch smoke on self-hosted runners:
```bash
FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"
make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=all WORKFLOW_REF="feat/policy_and_config_precedence"
```

