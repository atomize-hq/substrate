# S2-spec: System Packages Provisioning (ADR-0002)

## Scope
Provide a selection-driven, explicit provisioning route for `system_packages` tools that is:
- idempotent and safe to re-run (repair/upgrade story),
- compatible with Lima (macOS guest) and WSL (Windows distro),
- explicitly **unsupported** on Linux host backend (no host package mutation by Substrate).

This spec complements `S1` (install classes) by fulfilling the `system_packages` class via provisioning-time mechanisms.

---

## Command surface

Add:
- `substrate world deps provision [--all] [--dry-run] [--verbose]`

Semantics:
- Selection missing: no-op, exit `0` (DR-0004).
- Default scope: selected tools only.
- `--all`: ignore selection and use inventory scope (DR-0005).
- Only provisions packages required by tools with `install_class=system_packages` in the active scope.

Repair/upgrade:
- `provision` is idempotent. Re-running it is the supported “repair/upgrade” action.

---

## Platform strategy (required)

### macOS (Lima guest)
- Supported.
- Execution model:
  - Use the existing world backend connection to run apt inside the guest (world-agent runs as root in the VM).
  - Provisioning script must run outside the `world deps sync/install` runtime path; it is only invoked by `provision`.
- Package manager support:
  - apt-only (Ubuntu/Debian guest).
- Implementation detail requirements:
  - Run `apt-get update` before install (once per invocation).
  - Install union of packages via `apt-get install -y --no-install-recommends <pkgs...>`.

### Windows (WSL)
- Supported.
- Execution model:
  - Use the existing WSL world-agent via forwarder; run apt inside the distro (agent runs as root).
- Package manager support:
  - apt-only (Ubuntu WSL image from `scripts/windows/wsl-warm.ps1`).

### Linux host world-agent
- Explicitly unsupported (DR-0010).
- Behavior:
  - `provision` exits non-zero with exit code `4` and prints a clear message:
    - “Unsupported on Linux host backend because it would mutate host system packages.”
    - Print the exact required packages and a best-effort snippet for common package managers (not executed).

---

## Package selection algorithm (deterministic)

Inputs:
- Active tool scope (selection or inventory when `--all`).
- For each tool, `guest_install.class` and (if `system_packages`) `guest_install.system_packages`.

Algorithm:
1) Filter tools to those with `class=system_packages`.
2) Collect packages for the platform-supported package manager:
   - `apt` for Lima/WSL.
3) De-duplicate while preserving stable ordering:
   - First by tool priority order (manifest order), then lexical within each tool’s list.
4) If the resulting package list is empty:
   - Print: “No system packages required for the current selection.” and exit `0`.

Postcondition contract (required for consistency with S1):
- `provision` does not decide whether a `system_packages` tool is “present”.
- “Present” is determined solely by that tool’s `guest_detect.command` probe (see `S1` and `decision_register.md` DR-0014).

---

## Output requirements

Human output:
- Must print:
  - active selection path and scope
  - number of tools requiring system packages
  - the computed package list
  - whether `--dry-run` is active
  - on supported platforms: a summary line on success

JSON output:
- `provision` does not support `--json` in this track.

---

## Failure modes + exit codes

1) Selection YAML invalid / unknown tools
- Exit `2` (config error).

2) World backend unavailable (Lima/WSL)
- Exit `3` with actionable error and a pointer to:
  - `substrate world doctor --json`
  - the relevant warm script (`scripts/mac/lima-warm.sh` or `scripts/windows/wsl-warm.ps1`)

3) Unsupported platform (Linux host)
- Exit `4` with explicit “unsupported” + manual guidance + package list.

4) Package manager unsupported in guest
- Exit `4` with explicit message: “guest does not support apt; provisioning is not supported on this world image”.

5) Full-cage prevents apt execution or required mounts
- Exit `5` (hardening conflict) with message pointing to I2/I3.

---

## Sample outputs (required by ADR-0002)

### Unconfigured state
```
substrate: world deps not configured (selection file missing)
Next steps:
  - Create a selection file: substrate world deps init --workspace
  - Discover available tools: substrate world deps status --all
```

### Selection configured but a tool requires system packages (Linux host)
```
substrate: world deps provision: unsupported on Linux host backend (would mutate host system packages)
Required system packages for selected tools:
  - build-essential
  - libssl-dev
  - zlib1g-dev
Install them manually, then re-run:
  substrate world deps sync
```

### Selection configured and packages installed successfully (Lima/WSL)
```
Selection: .substrate/world-deps.selection.yaml (workspace)
Provisioning system packages for 1 tool (apt):
  build-essential libssl-dev zlib1g-dev
✓ system packages installed
Next: substrate world deps sync
```

---

## Automation hooks (required)

This slice is validated by the feature-local smoke scripts referenced below. These scripts are the authoritative automation surface for the acceptance matrix in this spec.

Smoke entrypoints (must exist and be used by CI smoke dispatch):
- Linux: `docs/project_management/next/world_deps_selection_layer/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/world_deps_selection_layer/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/world_deps_selection_layer/smoke/windows-smoke.ps1`

Requirements for the smoke scripts (no ambiguity):
1) **Environment isolation (required)**
   - Each smoke script must use a temporary `SUBSTRATE_HOME` and a temporary workspace directory so it does not mutate the operator’s real `~/.substrate` state or any real workspace config.

2) **Proving “no world-agent calls” for the no-op paths (required)**
   - For “selection missing” and “configured but empty selection” cases, each smoke script must set `SUBSTRATE_WORLD_SOCKET` to a non-existent path and assert the commands still exit `0`.
   - This is the mechanical proof that the code path did not attempt a backend connection.

3) **Backend prerequisites handling (required)**
   - Smoke scripts must not attempt to provision the world backend (no automatic calls to `scripts/mac/lima-warm.sh` or `scripts/windows/wsl-warm.ps1`).
   - If a backend-required command fails due to backend unavailability, the smoke script must fail with an explicit remediation block that points to:
     - `substrate world doctor --json`
     - the relevant warm/provision script for the platform.

4) **Capability-gated smoke design (required; DR-0017)**
   - The smoke scripts are allowed to gate WDL2-specific assertions on the presence of the `provision` subcommand in `substrate world deps --help`.
   - WDL2 integration tasks must include a separate explicit assertion that `provision` exists before dispatching CI smoke, so missing command surfaces cannot be hidden by gating.

5) **CI dispatch contract (required)**
   - Integration tasks dispatch feature smoke via:
     - `make feature-smoke FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" PLATFORM=behavior RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world_deps_selection_layer" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
   - Per-platform smoke dispatches use `PLATFORM=linux|macos|windows` with the same `FEATURE_DIR` and `WORKFLOW_REF`.

## Acceptance matrix (automatable; required)

Each row must be runnable by a script (no manual interpretation beyond viewing logs). “Pass checks” are concrete assertions.

| Journey | Linux (host agent) | macOS (Lima) | Windows (WSL) | Pass checks |
|---|---:|---:|---:|---|
| A: Unconfigured selection no-op | ✅ | ✅ | ✅ | `status/sync/install/provision` exit `0` and print “not configured”; prove no guest calls by setting invalid `SUBSTRATE_WORLD_SOCKET` and observing unchanged exit `0` |
| B: Provision on supported guests | ❌ | ✅ | ✅ | `provision` exit `0`; output includes apt package list and “installed”; rerun also exit `0` |
| C: Provision on Linux host unsupported | ✅ (as failure) | N/A | N/A | `provision` exit `4`; output includes “unsupported on Linux host backend” and prints package list |
| D: Sync blocked until provision | ✅ | ✅ | ✅ | When at least one selected `system_packages` tool’s probe is failing: `sync` exits `4` and references `provision` |
| E: Sync succeeds after provision | N/A (manual deps) | ✅ | ✅ | For tools whose probe becomes satisfied after `provision`: `sync` no longer exits `4` due to `system_packages` prerequisites |

Automation approach (required):
- Add and maintain feature-local smoke scripts (no repo-root `scripts/` changes required for this triad):
  - Linux: `bash docs/project_management/next/world_deps_selection_layer/smoke/linux-smoke.sh`
  - macOS: `bash docs/project_management/next/world_deps_selection_layer/smoke/macos-smoke.sh`
  - Windows: `pwsh -File docs/project_management/next/world_deps_selection_layer/smoke/windows-smoke.ps1`

Unit/integration tests (Rust):
- Add parsing/validation tests for system package metadata.
- Add deterministic package list computation tests (stable ordering, de-dup).
- Add exit-code mapping tests for common failure modes.

---

## Out of scope (S2)
- Supporting non-apt package managers in guests.
- Any privileged host package mutation flows on Linux.
- Implementing JSON output for `provision`.
