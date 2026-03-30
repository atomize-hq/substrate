---
codename: capturing_koala
created: "2026-03-30T23:04:33Z"
status: brainstorming
depends_on:
  - stashing_ferret
execution_order: 90
adr: ADR-0039
adr_path: docs/project_management/adrs/draft/ADR-0039-capturing-koala.md
workstream_id: WS-capturing_koala
lockdown_prompt: docs/project_management/system/prompts/discovery/adr_lockdown.md
---

# ADR Intake Sheet

## 1. Codename + date + status

- Codename: `capturing_koala`
- Created: 2026-03-30T23:04:33Z
- Status: brainstorming
- ADR draft: `docs/project_management/adrs/draft/ADR-0039-capturing-koala.md`

## 2. Working Title (tentative)

Persist macOS host OS details into `install_state.json` (schema_version=1) for later diagnostics, aligned with Windows.

## 3. Problem / Motivation

- We now persist Linux installer platform metadata into `install_state.json` (see `stashing_ferret`) so later tooling/support can answer “what did we detect at install time?” deterministically.
- macOS installs can still require repeated “what macOS version/build/arch are you on?” back-and-forth when diagnosing Lima + Virtualization.framework behavior, shim behavior, or world readiness.
- There is no canonical, installer-owned, later-readable record of macOS host OS details at the time the user installed Substrate.
- Windows already needs (and has) the same “persist host OS details” support; macOS should align so support/diagnostics can be uniform across non-Linux hosts.

## 4. Proposed Outcome

- macOS installs write (or update) `<effective_prefix>/install_state.json` (default `~/.substrate/install_state.json`) with additive macOS OS details.
- The write is best-effort + warning-only (never blocks a successful install).
- The file remains `schema_version = 1` and preserves existing fields (`host_state.group`, `host_state.linger`, unknown keys).

## 5. Non-Goals

- Changing Linux distro/pkg-manager persistence contract (this ADR is macOS-only host OS details).
- Introducing new provisioning behavior or changing Lima world setup steps.
- Persisting sensitive host metadata (no hostnames, no full `system_profiler` dumps).

## 6. Constraints / Invariants

- `schema_version` remains `1` (uninstall compatibility).
- Additive-only schema extension; older consumers must safely ignore new keys.
- Write path remains within `<effective_prefix>`; no writes outside `$SUBSTRATE_HOME`.
- Warning-only degradation on read/parse/write errors.

## 7. Interfaces / Contracts (concrete changes)

- File: `<effective_prefix>/install_state.json` (default `~/.substrate/install_state.json`)
- Additive fields (tentative; exact names locked by ADR):
  - `host_state.os.family` = `"macos"`
  - `host_state.os.product_version` (from `sw_vers -productVersion`)
  - `host_state.os.build_version` (from `sw_vers -buildVersion`)
  - `host_state.os.arch` (from `uname -m`)

## 8. Options

1) Extend `install_state.json` (recommended).
2) Store macOS OS details in `config.yaml` under `install.*` (avoid: conflates detection with operator intent).
3) Create a new `host_os.json` file (avoid: state fragmentation).

## 9. Recommendation

Choose Option 1: extend `install_state.json` with an additive, macOS-only `host_state.os` block.

## 10. Slice Decomposition (required)

- C0: Define the exact `host_state.os.*` contract and failure posture on macOS.
- C1: Implement macOS producer writes in `scripts/substrate/install-substrate.sh` with atomic replace + merge semantics.
- C2: Add/extend smoke assertions for macOS (parity harness) to prove the file is written and fields are present.

## 11. Acceptance Criteria Draft

- After a successful macOS install, `<effective_prefix>/install_state.json` exists.
- The JSON includes `host_state.os.family="macos"`, plus `product_version`, `build_version`, and `arch` when inputs are available.
- If OS detail collection fails, install still succeeds and either writes a partial `host_state.os` block or emits warning-only diagnostics (as specified by the ADR).
- Existing uninstall flows remain unchanged and continue to treat host-state cleanup as Linux-only.

## 12. Dependencies

- depends_on_adrs: [`stashing_ferret`]
- related_packs (reference-only):
  - `docs/project_management/packs/draft/best-effort-distro-package-manager-fse/`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager-fse/`

## 13. Work Lift (discovery estimate)

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "touch": {
    "create_files": 1,
    "edit_files": 2,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 0,
    "boundary_crossings": 1
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 1,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": 0, "new_test_cases": 1 },
  "docs": { "new_docs_files": 1 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": true,
    "security_sensitive": false,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": 1
  },
  "notes": "Estimate: extend install_state.json with a macOS-only host_state.os block (schema_version=1, additive) and add smoke/parity coverage."
}
```
<!-- PM_LIFT_VECTOR:END -->

## 14. Open Questions / Unknowns

- Exact macOS field set: do we want `product_name` and `kernel_release` in addition to version/build/arch?
- Exact “partial write” policy when `sw_vers` is missing/unavailable: write `family` + `arch` only, or skip the block entirely?
