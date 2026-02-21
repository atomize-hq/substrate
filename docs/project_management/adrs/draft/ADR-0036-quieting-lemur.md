# ADR-0036 — Treat `world.enabled: false` as a first-class “disabled” status in `substrate health` / `substrate shim doctor`

## Status
- Status: Draft
- Date (UTC): 2026-02-21
- Owner(s): TBD (ASSUMPTION: Substrate shell maintainers)

## Scope
- Feature directory: `docs/project_management/packs/active/world-disabled-diagnostics/` (ASSUMPTION: new pack)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md` (automation/worktree execution)

## Related Docs
- Intake: `docs/project_management/intake/adrs/quieting_lemur_adr_intake.md`
- World config precedence + exported state: `docs/CONFIGURATION.md`
- Env/config contract (authoritative precedence + `SUBSTRATE_OVERRIDE_WORLD`): `docs/reference/env/contract.md`
- Shim doctor implementation: `crates/shell/src/builtins/shim_doctor/report.rs`
- Health summary implementation: `crates/shell/src/builtins/health.rs`
- Adjacent ADR (follow-up referenced): `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: <run `make adr-fix ADR=docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md` after drafting>

### Changes (operator-facing)
- Quiet diagnostics when world isolation is intentionally disabled
  - Existing: With effective `world.enabled: false`, `substrate health` / `substrate shim doctor` can still probe the world backend and/or compute world-deps “applied” state, producing “needs attention” and “unavailable” errors that look like a broken setup.
  - New: When the effective config resolves to `world.enabled: false`, diagnostics report **World backend: disabled** and **World deps: skipped (world disabled)**, and MUST NOT execute world-backend probes to compute world-deps “applied” state.
  - Why: Reduce false-negative health signals; make “host-only by choice” explicit without masking real failures when the world is enabled.
  - Links:
    - `crates/shell/src/builtins/shim_doctor/report.rs` (doctor report generation)
    - `crates/shell/src/builtins/health.rs` (summary classification + operator text)
    - `docs/CONFIGURATION.md` (effective world-enabled resolution)

## Problem / Context
- Substrate supports intentional host-only operation when the effective config resolves to `world.enabled: false` (e.g., after installs performed with `--no-world` or when `SUBSTRATE_OVERRIDE_WORLD=disabled` applies).
- Today, `substrate health` / `substrate shim doctor` can still report “needs attention” for the world backend and “world deps unavailable …” because diagnostics attempt world-backend probing even though the world is disabled.
- These diagnostics are misleading: they suggest breakage, not an intentional configuration choice, and they make it harder to separate real provisioning failures (when world is enabled) from “disabled by design”.

## Goals
- When the effective config resolves to `world.enabled: false`, `substrate health` reports the world backend as **disabled**, not “needs attention”.
- When the effective config resolves to `world.enabled: false`, world-deps “applied” probing is **skipped** and reported as an explicit skipped/disabled status (non-error).
- JSON output for `substrate health --json` and `substrate shim doctor --json` includes an explicit, stable status enum for world and world-deps (additive fields; no schema break).
- No regression: when the effective config resolves to `world.enabled: true` and the world backend is broken/unreachable, diagnostics still report “needs attention” with actionable error details.

## Non-Goals
- Changing `substrate world enable` provisioning behavior.
- Changing world-deps inventory semantics or adding new inventory items.
- Reworking world-agent readiness probe logic (beyond skipping when disabled).
- Changing normal command execution routing/enforcement behavior.

## Out of Scope
- “Attribution UX” improvements that explain *which* layer disabled the world (CLI flag vs workspace config vs env override vs global config). (This is adjacent work; see `docs/project_management/intake/adrs/clarifying_owl_adr_intake.md`.)
- Extending “disabled short-circuit” semantics to `substrate world deps ...` subcommands (early guidance vs `--force`), beyond the diagnostics surfaces in this ADR.

## Slice Decomposition

### C0 — Effective `world.enabled` resolution inside diagnostics
Teach `substrate shim doctor` report generation to resolve the effective `world.enabled` bit (from the same precedence stack used for normal invocations) and to carry that into the doctor report as an explicit `disabled` status when false.

### C1 — Disabled-aware world + world-deps diagnostics classification
When world is disabled, diagnostics MUST NOT run world-backend probes (including world-deps “applied” computation) and MUST surface a non-error “skipped because world is disabled” status in both text and JSON outputs.

### C2 — Contract + validation coverage
Add focused tests/fixtures for disabled classification and JSON fields, and add/update operator-facing docs/playbooks for the disabled state messaging.

## Options

### Option A — Shim doctor consults effective config and emits `disabled` / `skipped_disabled` statuses (recommended)
`crates/shell/src/builtins/shim_doctor/report.rs` resolves effective `world.enabled` and uses it to short-circuit:
- world backend doctor snapshot collection (skip when disabled), and
- world-deps “applied” probing (skip when disabled),
while emitting explicit disabled/skip statuses (non-error).

### Option B — `substrate health` post-processes shim-doctor output and reclassifies disabled
Keep shim-doctor behavior largely intact, but in `crates/shell/src/builtins/health.rs` detect disabled state and suppress “world backend health check failed” / “world deps unavailable” failures when disabled.

### Recommendation (selection guidance)
- Choose Option A when you want consistent behavior across `substrate health` and `substrate shim doctor`, and you want to avoid triggering world-backend probes when the world is disabled.
- Choose Option B when you need the smallest immediate improvement and can accept `substrate shim doctor` remaining noisy/inconsistent until a follow-up.

## User Contract (Authoritative)

### CLI
- `substrate shim doctor`:
  - MUST resolve effective `world.enabled` using the same precedence rules as the Substrate CLI effective config (see `docs/CONFIGURATION.md` and `docs/reference/env/contract.md`).
  - When effective `world.enabled: false`:
    - MUST report world backend status as `disabled`.
    - MUST NOT execute world-backend probes (e.g., no `substrate world doctor --json`) for the purpose of diagnostics.
    - MUST report world-deps status as `skipped_disabled` (or equivalent) and MUST NOT compute “applied” status via world-backed probing.
  - When effective `world.enabled: true`:
    - MUST continue to report world backend status and world-deps availability as it does today (including actionable errors when the backend is broken).

- `substrate health`:
  - MUST treat effective `world.enabled: false` as a non-failure state:
    - “World backend: disabled (run `substrate world enable` to provision)” (exact phrasing may vary; must be explicit and actionable).
    - “World deps: skipped (world disabled)” (must be non-error).
  - MUST treat effective `world.enabled: true` and world-backend failures as “attention required”, with clear error details.

- Exit codes:
  - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md` (authoritative unless explicitly overridden here)
  - ASSUMPTION: `substrate health` and `substrate shim doctor` remain informational surfaces where successful report generation exits `0`, regardless of “attention required” classification; non-zero exit is reserved for command execution failures (e.g., invalid flags, serialization errors).

### Config
- Effective config precedence for `world.enabled` is authoritative and unchanged by this ADR:
  - CLI flags: `--world` / `--no-world`
  - Workspace config: `<workspace_root>/.substrate/workspace.yaml` (when enabled)
  - Env override: `SUBSTRATE_OVERRIDE_WORLD` (when no enabled workspace exists)
  - Global config: `$SUBSTRATE_HOME/config.yaml` (default: `~/.substrate/config.yaml`)
- This ADR does not introduce new config keys.

### JSON output (additive fields; stable enums)
- `substrate shim doctor --json` MUST include additive fields that express:
  - `world.status`: enum including at least `healthy | needs_attention | disabled | unknown` (exact final naming to be decided; see Decision Register).
  - `world_deps.status`: enum including at least `ok | error | skipped_disabled` (exact final naming to be decided; see Decision Register).
- When world is disabled, JSON MUST carry the “skipped because disabled” signal as a status value (not as a generic error string).

### Platform guarantees
- Linux: in scope.
- macOS: in scope.
- Windows: in scope.

## Architecture Shape
- Components:
  - `crates/shell/src/execution/config_model.rs`: effective-config resolver (source of truth for `world.enabled` resolution).
  - `crates/shell/src/builtins/shim_doctor/report.rs`: report generation; adds disabled-aware short-circuit and status emission.
  - `crates/shell/src/builtins/health.rs`: summary classification and operator-facing text; consumes new status enums.
- End-to-end flow:
  - Inputs: CLI flags (`--world`/`--no-world`), workspace/global config files, `SUBSTRATE_OVERRIDE_WORLD` (when applicable).
  - Derived state: effective `world.enabled` boolean.
  - Actions:
    - If `world.enabled=false`: emit `disabled` / `skipped_disabled` statuses; skip world probes.
    - If `world.enabled=true`: run existing world and world-deps diagnostics paths.
  - Outputs: text and JSON diagnostics where “disabled” is explicit and non-error, and genuine failures remain visible when enabled.

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → TBD
- Prerequisite integration task IDs: none
- Adjacent work (not required for this ADR):
  - Attribution improvements for “why disabled” messaging (`clarifying_owl` intake).

## Security / Safety Posture
- Degrade posture:
  - When `world.enabled=false`, diagnostics degrade by skipping probes; they MUST NOT imply world health or world-deps “applied” state.
- Fail-closed rules:
  - When `world.enabled=true`, diagnostics MUST continue to surface backend unavailability and errors (no masking).
- Observability:
  - The disabled/skip states MUST be visible in both text and JSON output, and must be machine-detectable via explicit enum fields.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - Add/extend tests for `HealthSummary::from_report` classification when world is disabled (location: `crates/shell/src/builtins/health.rs` tests).
  - Add/extend tests for shim-doctor report generation to ensure it short-circuits probing and emits the disabled/skip status fields (location: `crates/shell/src/builtins/shim_doctor/report.rs` tests).
- Integration tests:
  - Add an integration test that sets up a temporary `$SUBSTRATE_HOME/config.yaml` with `world.enabled: false`, runs `substrate shim doctor --json` and `substrate health --json`, and asserts:
    - world status is `disabled`,
    - world-deps status is `skipped_disabled`,
    - no “world deps unavailable” failure is emitted solely due to probing.

### Manual validation
- Manual playbook: `docs/project_management/packs/active/world-disabled-diagnostics/manual_testing_playbook.md` (TBD), covering:
  - Set `world.enabled: false` in `$SUBSTRATE_HOME/config.yaml`.
  - Run `substrate shim doctor` and confirm world/world-deps show disabled/skip (non-error).
  - Run `substrate health` and confirm overall status is not “attention required” solely due to world-disabled.
  - Flip to `world.enabled: true` with an intentionally broken socket/agent and confirm “needs attention” still triggers.

### Smoke scripts
- Linux: `docs/project_management/packs/active/world-disabled-diagnostics/smoke/linux-smoke.sh` (TBD)
- macOS: `docs/project_management/packs/active/world-disabled-diagnostics/smoke/macos-smoke.sh` (TBD)
- Windows: `docs/project_management/packs/active/world-disabled-diagnostics/smoke/windows-smoke.ps1` (TBD)

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work:
  - JSON is additive: new status fields are added without removing existing fields.
  - Existing fields should remain present, but operators and internal tooling should migrate to the explicit status enums for disabled/skip detection.

## Decision Summary
- This ADR locks the behavior delta:
  - Disabled world is explicit and non-error in diagnostics.
  - World-deps “applied” probing is skipped when disabled (no escape hatch).
  - JSON carries stable, machine-detectable disabled/skip status values (additive fields).
- Decision Register entries (pack-local) MUST capture the remaining implementation choices that should *not* live in the ADR body:
  - Exact JSON field names and enum spellings for world + world-deps status.
  - Whether disabled status is represented as a nested field (e.g., `world.status`) vs top-level summary fields (e.g., `summary.world_status`).
  - Whether legacy `error` fields remain populated for disabled/skip (recommended: do not use `error` for disabled/skip).
  - Exact operator-facing copy for disabled/skip messaging (keep it consistent across `health` and `shim doctor`).
