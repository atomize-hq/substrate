# ADR-0034 — Stabilize dev-install helper discovery under `SUBSTRATE_HOME`

## Status
- Status: Draft
- Date (UTC): 2026-02-21
- Owner(s): TBD (ASSUMPTION: Substrate shell maintainers)

## Scope
- Feature directory: `docs/project_management/packs/draft/dev-install-helper-discovery/` (ASSUMPTION)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)
- Intake: `docs/project_management/intake/adrs/staging_beaver_adr_intake.md`
- Plan: `docs/project_management/packs/draft/dev-install-helper-discovery/plan.md` (TBD)
- Tasks: `docs/project_management/packs/draft/dev-install-helper-discovery/tasks.json` (TBD)
- Spec manifest: `docs/project_management/packs/draft/dev-install-helper-discovery/spec_manifest.md` (TBD)
- Specs: (TBD)
- Contract (if present): `docs/project_management/packs/draft/dev-install-helper-discovery/contract.md` (TBD)
- Decision Register: `docs/project_management/packs/draft/dev-install-helper-discovery/decision_register.md` (TBD; required)
- Impact Map: `docs/project_management/packs/draft/dev-install-helper-discovery/impact_map.md` (TBD)
- Manual Playbook: `docs/project_management/packs/draft/dev-install-helper-discovery/manual_testing_playbook.md` (TBD)

## Executive Summary (Operator)

ADR_BODY_SHA256: eeac9675069d6bb9816517edb5a7b66846dd48204e4a1002a6341010ee562c8b

### Changes (operator-facing)
- Dev installs stage a stable runtime bundle for `substrate world enable` under `$SUBSTRATE_HOME`.
  - Existing: `dev-install-substrate.sh` links `~/.substrate/bin/substrate` directly to `<repo>/target/<profile>/substrate`, and helper/runtime assets are inferred from `<repo>/target/...`; that can break after `cargo clean` and can leave macOS `world enable` unable to find `scripts/mac/lima-warm.sh`.
  - New: `dev-install-substrate.sh` stages `world-enable.sh`, `install-substrate.sh`, `world-deps.yaml`, `scripts/mac/lima-warm.sh`, the `scripts/mac/lima/{substrate.yaml,substrate-dev.yaml}` profile subtree, and Linux guest binaries under `$SUBSTRATE_HOME/{scripts,bin/linux}/…`. On macOS, the normal world-enabled dev-install path now persists copied Linux `substrate` and `world-agent` ELFs from Lima into `$SUBSTRATE_HOME/bin/linux/`, so later `substrate world enable` can succeed from the prefix bundle without depending on repo sources.
  - Why: Decouple `world enable` runtime assets from `<repo>/target/*` build artifacts while keeping `$SUBSTRATE_HOME/bin/substrate` pointed at the live host build output.
  - Links:
    - `crates/shell/src/builtins/world_enable/runner/paths.rs#L33` (helper search order includes `$SUBSTRATE_HOME/scripts/…`)
    - `scripts/substrate/dev-install-substrate.sh#L787` (current helper staging into `<repo>/target/scripts/…`)
    - `scripts/substrate/dev-uninstall-substrate.sh#L391` (current dev uninstall does not manage `$SUBSTRATE_HOME/scripts/…`)

## Problem / Context
- Production installs stage a complete bundle under `$SUBSTRATE_HOME/versions/<version>/` and helper scripts live under that stable root.
- Dev installs currently link `$SUBSTRATE_HOME/bin/substrate` directly to `<repo>/target/<profile>/substrate`. As a result, `substrate world enable` infers its “version directory” as `<repo>/target/` unless the CLI can find a stable helper/runtime bundle under `$SUBSTRATE_HOME`.
- The previous dev-install layout depended on `<repo>/target/scripts/substrate/…` helper bridges, which disappear after `cargo clean` and never included `scripts/mac/lima-warm.sh`, leaving macOS `world enable` with an incomplete inferred release root.

## Goals
- After `dev-install-substrate.sh`, `substrate world enable` resolves its helper/runtime bundle from `$SUBSTRATE_HOME` even if `<repo>/target/scripts/…` does not exist.
- Keep the dev inner loop intact: `$SUBSTRATE_HOME/bin/substrate` continues to point at the live `<repo>/target/<profile>/substrate` output.
- Ensure dev-uninstall can remove only the staged runtime-bundle symlinks that were produced by dev-install (no accidental deletion of user-managed files).

## Non-Goals
- Changing how the Substrate binaries are built (still `cargo build` from the repo).
- Changing production install layout under `$SUBSTRATE_HOME/versions/…`.
- Changing `substrate world enable` version-directory inference logic (it may continue to infer `<repo>/target/` for dev installs).
- Solving other `<repo>/target/*` coupling beyond helper discovery (example: staging or discovering world-agent artifacts).

## Out of Scope
- Full “bundle parity” dev installs under `$SUBSTRATE_HOME/versions/<label>/{bin,scripts,config}` (follow-up ADR).
- Profile switching and multi-label semantics (`--profile`, `--version-label`) beyond current behavior (follow-up ADR if needed).
- Changing how world provisioning is enabled/disabled (`--no-world`) beyond existing semantics.

## Options

### Option A — Stable runtime bundle under `$SUBSTRATE_HOME` (this ADR)
Keep `$SUBSTRATE_HOME/bin/substrate -> <repo>/target/<profile>/substrate` for “always latest build output”, but ensure `dev-install-substrate.sh` stages the `world enable` runtime bundle under `$SUBSTRATE_HOME/{scripts,bin/linux}/…` so the CLI can prefer that stable bundle over the inferred `<repo>/target/...` layout.

### Option B — Full parity (follow-up): stage a prod-like version directory under `$SUBSTRATE_HOME/versions/<label>/`
Adopt a production-like bundle root (`bin/`, `scripts/`, `config/`) under `$SUBSTRATE_HOME/versions/<label>/` and link `$SUBSTRATE_HOME/bin/*` into that directory so helper discovery and artifact expectations no longer depend on `<repo>/target/*`.

### Recommendation
- Choose Option A when we want the smallest behavior delta that fixes the most brittle failure mode (helper discovery across `cargo clean`) while preserving the dev inner loop (binaries still point at `<repo>/target/*`).
- Choose Option B when we want dev/prod install parity and are willing to address broader “version dir” semantics (including how staged binaries stay aligned with ongoing `cargo build` outputs).

## Slice Decomposition
- C0 — Stage the `world enable` runtime bundle under `$SUBSTRATE_HOME`.
  - `dev-install-substrate.sh` ensures `$SUBSTRATE_HOME/scripts/substrate/{world-enable.sh,install-substrate.sh,world-deps.yaml}`, `$SUBSTRATE_HOME/scripts/mac/lima-warm.sh`, `$SUBSTRATE_HOME/scripts/mac/lima/{substrate.yaml,substrate-dev.yaml}`, and best-effort `$SUBSTRATE_HOME/bin/linux/{substrate,world-agent}` symlinks exist after dev install.
- C1 — Uninstall cleanup for staged runtime bundle artifacts.
  - `dev-uninstall-substrate.sh` removes only staged symlinks owned by the invoking repo, leaving user-managed scripts or binaries untouched.

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate world enable`:
    - Helper discovery order becomes:
      1) `SUBSTRATE_WORLD_ENABLE_SCRIPT` override
      2) `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh`
      3) `<inferred version dir>/scripts/substrate/world-enable.sh`
    - Dev installs guarantee that (2) exists post-install and that the corresponding `$SUBSTRATE_HOME/scripts/mac/{lima-warm.sh,lima/...}` and `$SUBSTRATE_HOME/bin/linux/...` bundle is present when available.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` (no overrides in this ADR).

### Config
- Files and locations (precedence): no changes in this ADR.
- Schema: no changes in this ADR.

### Platform guarantees
- Linux: helper staging and cleanup behavior is supported.
- macOS: helper staging and cleanup behavior is supported.
- Windows: `substrate world enable` remains unsupported; this ADR does not change Windows behavior.

## Architecture Shape
- Components:
  - `scripts/substrate/dev-install-substrate.sh`: stage the stable runtime bundle under `$SUBSTRATE_HOME/{scripts,bin/linux}/…`, including the Lima profile subtree, and remove legacy `target/scripts/substrate` helper bridges.
  - `scripts/substrate/dev-uninstall-substrate.sh`: remove staged runtime-bundle symlinks under `$SUBSTRATE_HOME/{scripts,bin/linux}/…`.
  - `crates/shell/src/builtins/world_enable/runner/paths.rs`: prefers `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh` over the inferred version-dir helper path.
- End-to-end flow:
  - Inputs: repo checkout path, build profile (`debug|release`), `--prefix` (`$SUBSTRATE_HOME`).
  - Derived state: `$SUBSTRATE_HOME` paths, inferred version dir (often `<repo>/target/` for dev installs).
  - Actions: dev-install stages a stable runtime bundle into `$SUBSTRATE_HOME`; on macOS, the world-enabled dev-install flow also caches Linux guest binaries from Lima into `$SUBSTRATE_HOME/bin/linux/`. Later, `substrate world enable` resolves the helper from `$SUBSTRATE_HOME` first and uses the staged macOS/Linux assets from that same root.
  - Outputs: stable helper/runtime resolution across `cargo clean`, deterministic macOS `world enable`, and predictable cleanup via dev-uninstall.

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → TBD
- Prerequisite integration task IDs: none

## Work Lift (discovery estimate)

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "touch": {
    "create_files": 0,
    "edit_files": 1,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 0,
    "boundary_crossings": 0
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": 0, "new_test_cases": 0 },
  "docs": { "new_docs_files": 0 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": true,
    "security_sensitive": false,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": 0
  },
  "notes": "Estimate: dev-install/dev-uninstall script staging for helper discovery under SUBSTRATE_HOME."
}
```
<!-- PM_LIFT_VECTOR:END -->

## Security / Safety Posture
- Fail-closed rules:
  - If the helper script cannot be found in either location, `substrate world enable` must error (no “best-effort” provisioning without an explicit helper path).
- Protected paths/invariants:
  - `dev-uninstall-substrate.sh` must not delete user-managed helper scripts under `$SUBSTRATE_HOME/scripts/substrate/…` unless they are confirmed to be owned/managed by dev-install (decision captured in the Decision Register).

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - Add/extend tests for helper discovery paths and fallbacks (likely in `crates/shell/src/builtins/world_enable/runner/paths.rs`).
- Integration tests:
  - Add an integration-style test that simulates dev install, removes `<repo>/target/scripts/…`, and asserts `substrate world enable --dry-run` locates the helper from `$SUBSTRATE_HOME/scripts/substrate/…` (exact harness TBD).

### Manual validation
- Manual playbook: `docs/project_management/packs/draft/dev-install-helper-discovery/manual_testing_playbook.md` (TBD), covering:
  - Run `scripts/substrate/dev-install-substrate.sh`.
  - Run `cargo clean` then rebuild `target/<profile>/substrate` without rerunning dev-install.
  - Run `substrate world enable --dry-run` and confirm it resolves the helper from `$SUBSTRATE_HOME/scripts/substrate/…`.
  - Run `scripts/substrate/dev-uninstall-substrate.sh` and confirm staged helpers are removed when they are managed by dev-install.

### Smoke scripts
- Linux: `docs/project_management/packs/draft/dev-install-helper-discovery/smoke/linux-smoke.sh` (TBD)
- macOS: `docs/project_management/packs/draft/dev-install-helper-discovery/smoke/macos-smoke.sh` (TBD)
- Windows: none (out of scope; `world enable` unsupported)

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none (dev-install behavior change only; production installs unchanged)

## Decision Summary
- Decision Register entries (if applicable):
  - `docs/project_management/packs/draft/dev-install-helper-discovery/decision_register.md`:
    - DR-0001 (helper staging mechanism: symlink vs copy)
    - DR-0002 (uninstall ownership guard: how to determine “managed by dev-install”)
    - DR-0003 (overwrite policy if `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh` already exists)
- Options (required; at least two):
  - A) Full parity: stage a production-like bundle under `$SUBSTRATE_HOME/versions/<label>/{bin,scripts,config}` and link `$SUBSTRATE_HOME/bin/*` into it.
  - B) Minimal parity: keep `$SUBSTRATE_HOME/bin/substrate -> <repo>/target/...` but stage helper scripts under `$SUBSTRATE_HOME/scripts/substrate/...` (recommended).
- Selection:
  - Chosen: B
  - Rationale: Fixes the brittle helper-discovery failure after `cargo clean` without changing the dev inner-loop “always use latest build output” symlink behavior.
  - Choose A when: we want dev+prod installs to share the same mental model and are willing to address version-dir inference and potential drift from `cargo build` outputs.
  - Choose B when: we want the smallest vertical slice that preserves the existing dev workflow but makes helper discovery stable under `$SUBSTRATE_HOME`.
