# S1c-windows-dry-run – Windows WSL warm WhatIf capture

## Background
- S1c integration validated Linux/macOS provisioning flows locally, but `pwsh -File scripts/windows/wsl-warm.ps1 -WhatIf` was skipped because PowerShell 7 is unavailable on the Linux dev host.
- We need a Windows operator to re-run the script under PowerShell 7 (pwsh) so we have an audited WhatIf transcript showing how `.service`/`.socket` deployment looks on Windows hosts before replay work begins.
- The captured output should live under `artifacts/windows/` and the session log must call out host context (Windows build, WSL state, pwsh version) and the command results/next steps.

## Prerequisites
- Windows 11 (or Windows 10 22H2+) workstation with:
  - PowerShell 7 (`pwsh`) installed and on PATH.
  - WSL optional components enabled (`Microsoft-Windows-Subsystem-Linux`, `VirtualMachinePlatform`).
  - The `substrate-wsl` distro (or whichever distro is referenced in `scripts/windows/wsl-warm.ps1`) created, or note if absent.
- Git checkout of `feat/p0-platform-stability` with the latest S1c integration commits.
- Ability to create `artifacts/windows/` under the repo root for log storage.

## Start Checklist
1. `git checkout feat/p0-platform-stability` (ensure working tree clean).
2. Verify PowerShell version: `pwsh --version` (should be 7.x). Capture output for the session log.
3. Confirm WSL availability: `wsl.exe --status` and `wsl.exe -l -v`.
4. Update `tasks.json` (`S1c-windows-dry-run` → `in_progress`) and append a START entry to `session_log.md` describing host context + intent.
5. Create `artifacts/windows/` if it does not exist.

## Required Commands
Run these from the repo root inside PowerShell 7 (`pwsh`):
```powershell
pwsh -Version            # confirm CLI version (log output)
wsl.exe --status         # capture WSL health
wsl.exe -l -v            # capture distro list/state
pwsh -File scripts/windows/wsl-warm.ps1 -WhatIf *>&1 | Tee-Object artifacts/windows/wsl-warm-whatif-$(Get-Date -Format 'yyyyMMdd-HHmmss').log
```
- The `Tee-Object` invocation must capture all output (stdout/stderr) into a timestamped log file while also emitting to the console.
- If the script references prerequisite steps (e.g., installing VS Code extensions, enabling optional features), follow them when possible or record blockers in the END log.

## Deliverables
- Log file under `artifacts/windows/` with the exact filename recorded in the session log.
- Session log END entry covering:
  - Host OS build, pwsh version, WSL distro state.
  - Command exit status and high-level summary (e.g., “WhatIf confirmed socket/service enablement; existing units already installed”).
  - Any follow-up actions taken or required.
- `tasks.json` updated with `S1c-windows-dry-run` status `completed`.
