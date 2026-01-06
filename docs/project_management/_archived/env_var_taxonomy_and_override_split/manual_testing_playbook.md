# Manual Testing Playbook — Env Var Taxonomy + Override Split (ADR-0006)

This playbook validates the EV0 slice contract:
- `SUBSTRATE_*` exported state values do not act as effective-config override inputs.
- `SUBSTRATE_OVERRIDE_*` values act as effective-config override inputs.
 - Smoke/manual validation covers policy mode plus multiple non-policy keys so partial implementations can’t accidentally pass.

Standards:
- Exit codes: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- Platform integration: `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`

## Fast path (preferred): run smoke scripts

Smoke scripts are the auditable, repeatable version of this playbook. Success is exit code `0` and an `OK:` line.

- Linux: `bash docs/project_management/_archived/env_var_taxonomy_and_override_split/smoke/linux-smoke.sh`
- macOS: `bash docs/project_management/_archived/env_var_taxonomy_and_override_split/smoke/macos-smoke.sh`
- Windows: `pwsh -File docs/project_management/_archived/env_var_taxonomy_and_override_split/smoke/windows-smoke.ps1`

Cross-platform CI dispatch (preferred when validating parity):
- `make feature-smoke FEATURE_DIR="docs/project_management/_archived/env_var_taxonomy_and_override_split" PLATFORM=all RUNNER_KIND=self-hosted WORKFLOW_REF="feat/env_var_taxonomy_and_override_split" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## Manual validation (debugging)

All steps run in temp directories and must not modify a real `$SUBSTRATE_HOME`.

### Preconditions
- `substrate` is on `PATH`.
- Linux/macOS: `/bin/bash` exists.

### Linux/macOS steps

1) Create a temp `$SUBSTRATE_HOME` and write a known global config for multiple keys.

```bash
TMP_HOME="$(mktemp -d)"
export HOME="$TMP_HOME"
export SUBSTRATE_HOME="$TMP_HOME"

substrate config global init --force
substrate config global set policy.mode=observe
substrate config global set world.caged=true
substrate config global set world.anchor_mode=follow-cwd
```

Expected:
- exit code `0` for all commands.

2) Verify baseline values are visible in a host-only invocation.

```bash
substrate --no-world --shell /bin/bash -c 'printf "%s|%s|%s" "${SUBSTRATE_POLICY_MODE:-}" "${SUBSTRATE_CAGED:-}" "${SUBSTRATE_ANCHOR_MODE:-}"'
```

Expected:
- stdout: `observe|1|follow-cwd`
- exit code `0`

3) Verify legacy exported-state env vars do not override effective config.

```bash
SUBSTRATE_POLICY_MODE=disabled SUBSTRATE_CAGED=0 SUBSTRATE_ANCHOR_MODE=workspace \
  substrate --no-world --shell /bin/bash -c 'printf "%s|%s|%s" "${SUBSTRATE_POLICY_MODE:-}" "${SUBSTRATE_CAGED:-}" "${SUBSTRATE_ANCHOR_MODE:-}"'
```

Expected:
- stdout: `observe|1|follow-cwd`
- exit code `0`

4) Verify `SUBSTRATE_OVERRIDE_*` overrides are applied.

```bash
SUBSTRATE_OVERRIDE_POLICY_MODE=enforce SUBSTRATE_OVERRIDE_CAGED=0 SUBSTRATE_OVERRIDE_ANCHOR_MODE=workspace \
  substrate --no-world --shell /bin/bash -c 'printf "%s|%s|%s" "${SUBSTRATE_POLICY_MODE:-}" "${SUBSTRATE_CAGED:-}" "${SUBSTRATE_ANCHOR_MODE:-}"'
```

Expected:
- stdout: `enforce|0|workspace`
- exit code `0`

5) Verify workspace config overrides override env vars.

```bash
TMP_WS="$(mktemp -d)"
substrate workspace init "$TMP_WS"
cd "$TMP_WS"
substrate config set policy.mode=observe >/dev/null
substrate config set world.caged=true >/dev/null
substrate config set world.anchor_mode=follow-cwd >/dev/null

SUBSTRATE_OVERRIDE_POLICY_MODE=enforce SUBSTRATE_OVERRIDE_CAGED=0 SUBSTRATE_OVERRIDE_ANCHOR_MODE=workspace \
  substrate --no-world --shell /bin/bash -c 'printf "%s|%s|%s" "${SUBSTRATE_POLICY_MODE:-}" "${SUBSTRATE_CAGED:-}" "${SUBSTRATE_ANCHOR_MODE:-}"'
```

Expected:
- stdout: `observe|1|follow-cwd`
- exit code `0`

6) Verify invalid override values fail as user errors for config commands.

```bash
set +e
SUBSTRATE_OVERRIDE_POLICY_MODE=bogus substrate config show --json >/dev/null 2>&1
code1=$?
SUBSTRATE_OVERRIDE_CAGED=bogus substrate config show --json >/dev/null 2>&1
code2=$?
set -e
echo "$code1 $code2"
```

Expected:
- stdout: `2 2`

Cleanup:
```bash
rm -rf "$TMP_HOME" "$TMP_WS"
```

### Windows steps (PowerShell)

1) Create a temp `$env:SUBSTRATE_HOME` and write a known global config for multiple keys.

```powershell
$tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("substrate-ev0-" + [System.Guid]::NewGuid().ToString("N"))
$tmpHome = Join-Path $tmpRoot "home"
New-Item -ItemType Directory -Force -Path $tmpHome | Out-Null
$env:SUBSTRATE_HOME = $tmpHome
$env:HOME = $tmpHome
$env:USERPROFILE = $tmpHome

substrate config global init --force | Out-Null
substrate config global set policy.mode=observe | Out-Null
substrate config global set world.caged=true | Out-Null
substrate config global set world.anchor_mode=follow-cwd | Out-Null
```

2) Verify override behavior via a host-only invocation.

```powershell
$out = & substrate --no-world --shell cmd.exe -c "echo %SUBSTRATE_POLICY_MODE%|%SUBSTRATE_CAGED%|%SUBSTRATE_ANCHOR_MODE%"
($out | Select-Object -Last 1).Trim()
```

Expected:
- stdout: `observe|1|follow-cwd`
- exit code `0`

3) Verify legacy exported-state env vars do not override effective config.

```powershell
$env:SUBSTRATE_POLICY_MODE = "disabled"
$env:SUBSTRATE_CAGED = "0"
$env:SUBSTRATE_ANCHOR_MODE = "workspace"
$out = & substrate --no-world --shell cmd.exe -c "echo %SUBSTRATE_POLICY_MODE%|%SUBSTRATE_CAGED%|%SUBSTRATE_ANCHOR_MODE%"
Remove-Item Env:SUBSTRATE_POLICY_MODE -ErrorAction SilentlyContinue
Remove-Item Env:SUBSTRATE_CAGED -ErrorAction SilentlyContinue
Remove-Item Env:SUBSTRATE_ANCHOR_MODE -ErrorAction SilentlyContinue
($out | Select-Object -Last 1).Trim()
```

Expected:
- stdout: `observe|1|follow-cwd`
- exit code `0`

4) Verify `SUBSTRATE_OVERRIDE_*` overrides are applied.

```powershell
$env:SUBSTRATE_OVERRIDE_POLICY_MODE = "enforce"
$env:SUBSTRATE_OVERRIDE_CAGED = "0"
$env:SUBSTRATE_OVERRIDE_ANCHOR_MODE = "workspace"
$out = & substrate --no-world --shell cmd.exe -c "echo %SUBSTRATE_POLICY_MODE%|%SUBSTRATE_CAGED%|%SUBSTRATE_ANCHOR_MODE%"
Remove-Item Env:SUBSTRATE_OVERRIDE_POLICY_MODE -ErrorAction SilentlyContinue
Remove-Item Env:SUBSTRATE_OVERRIDE_CAGED -ErrorAction SilentlyContinue
Remove-Item Env:SUBSTRATE_OVERRIDE_ANCHOR_MODE -ErrorAction SilentlyContinue
($out | Select-Object -Last 1).Trim()
```

Expected:
- stdout: `enforce|0|workspace`
- exit code `0`

5) Verify workspace config wins over overrides.

```powershell
$tmpWs = Join-Path $tmpRoot "ws2"
New-Item -ItemType Directory -Force -Path $tmpWs | Out-Null
& substrate workspace init $tmpWs | Out-Null

Push-Location $tmpWs
try {
  & substrate config set policy.mode=observe | Out-Null
  & substrate config set world.caged=true | Out-Null
  & substrate config set world.anchor_mode=follow-cwd | Out-Null

  $env:SUBSTRATE_OVERRIDE_POLICY_MODE = "enforce"
  $env:SUBSTRATE_OVERRIDE_CAGED = "0"
  $env:SUBSTRATE_OVERRIDE_ANCHOR_MODE = "workspace"
  $out = & substrate --no-world --shell cmd.exe -c "echo %SUBSTRATE_POLICY_MODE%|%SUBSTRATE_CAGED%|%SUBSTRATE_ANCHOR_MODE%"
  Remove-Item Env:SUBSTRATE_OVERRIDE_POLICY_MODE -ErrorAction SilentlyContinue
  Remove-Item Env:SUBSTRATE_OVERRIDE_CAGED -ErrorAction SilentlyContinue
  Remove-Item Env:SUBSTRATE_OVERRIDE_ANCHOR_MODE -ErrorAction SilentlyContinue
  ($out | Select-Object -Last 1).Trim()
} finally {
  Pop-Location
}
```

Expected:
- stdout: `observe|1|follow-cwd`
- exit code `0`

6) Verify invalid override values fail as user errors for config commands (multiple keys).

```powershell
$env:SUBSTRATE_OVERRIDE_POLICY_MODE = "bogus"
& substrate config show --json 2>$null | Out-Null
$code1 = $LASTEXITCODE
Remove-Item Env:SUBSTRATE_OVERRIDE_POLICY_MODE -ErrorAction SilentlyContinue

$env:SUBSTRATE_OVERRIDE_CAGED = "bogus"
& substrate config show --json 2>$null | Out-Null
$code2 = $LASTEXITCODE
Remove-Item Env:SUBSTRATE_OVERRIDE_CAGED -ErrorAction SilentlyContinue

"$code1 $code2"
```

Expected:
- stdout: `2 2`

Cleanup:
```powershell
Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
```

## Required repo audit (implementation review)

Before treating EV0 as complete, run a repo-wide grep/audit to confirm no non-test code bypasses effective config resolution by consuming config-shaped `SUBSTRATE_*` values directly as inputs.

Baseline commands (run from repo root):
```bash
rg -n "SUBSTRATE_(WORLD(_ENABLED)?|ANCHOR_MODE|ANCHOR_PATH|CAGED|POLICY_MODE|SYNC_AUTO_SYNC|SYNC_DIRECTION|SYNC_CONFLICT_POLICY|SYNC_EXCLUDE)" -S crates src scripts
rg -n "env::var(_os)?\\(\"SUBSTRATE_(WORLD(_ENABLED)?|ANCHOR_MODE|ANCHOR_PATH|CAGED|POLICY_MODE|SYNC_AUTO_SYNC|SYNC_DIRECTION|SYNC_CONFLICT_POLICY|SYNC_EXCLUDE)\"\\)" -S crates
```
