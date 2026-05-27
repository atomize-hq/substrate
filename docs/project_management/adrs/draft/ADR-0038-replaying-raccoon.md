# ADR-0038 — Replay: attribute why world isolation is disabled in replay warnings (flag vs env vs config)

## Status

- Status: Draft
- Date (UTC): 2026-02-21
- Owner(s): TBD (ASSUMPTION: Substrate shell maintainers)

## Stable Curated ADR

- Current stable ADR: `docs/adr/implemented/ADR-0038-replay-attribute-why-world-is-disabled-in-warnings.md`
- This project-management file remains the planning-rich historical source retained for
  compatibility while `docs/project_management/**` is being retired.

## Scope

- Feature directory: `docs/project_management/packs/draft/world-disabled-reason-attribution/` (ASSUMPTION: created during planning)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

- Intake: `docs/project_management/intake/adrs/replaying_raccoon_adr_intake.md`
- Prerequisite ADR (required): `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
- Plan: `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md` (planned)
- Tasks: `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json` (planned)
- Spec manifest: `docs/project_management/packs/draft/world-disabled-reason-attribution/spec_manifest.md` (planned)
- Decision Register: `docs/project_management/packs/draft/world-disabled-reason-attribution/decision_register.md` (required; see “Decision Summary”)
- Impact Map: `docs/project_management/packs/draft/world-disabled-reason-attribution/impact_map.md` (recommended; cross-surface UX consistency + trace/CI implications)

## Executive Summary (Operator)

ADR_BODY_SHA256: 6c765ad8b6ec6afd7202ad9d7c0eb9b488db2160dbfa8331e338655822f3d9ce

### Changes (operator-facing)

- Replay output reuses the same “world disabled reason attribution” as doctor/health, so replay does not imply `--no-world` when the actual cause is config/env.
  - Existing: `substrate --replay <span_id>` can emit replay origin/warning text that either (a) attributes host-only replay to `--no-world` when the true cause is persisted config/env, or (b) leaves operators guessing which layer disabled world (flag vs env override vs workspace/global config).
  - New: When replay runs on host due to world isolation being disabled, replay’s warning/origin text (and any associated structured replay strategy fields) include an accurate attribution of the _highest-precedence_ disable source, aligned with doctor/health:
    - CLI `--no-world`, or
    - env override `SUBSTRATE_OVERRIDE_WORLD=disabled` (when applicable), or
    - workspace config `<workspace>/.substrate/workspace.yaml` (`world.enabled: false`), or
    - global config `$SUBSTRATE_HOME/config.yaml` (`world.enabled: false`).
  - Why: Reduce “disable now, enable later” confusion and support/debug time by making replay’s world-adjacent messaging consistent with doctor/health.
  - Links:
    - `docs/project_management/intake/adrs/replaying_raccoon_adr_intake.md`
    - `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md` (shared disable attribution contract)
    - `crates/shell/src/execution/routing/replay.rs` (replay origin + warnings)
    - `crates/replay/src/replay/executor.rs` (replay_strategy trace event)

## Problem / Context

- After ADR-0037 (`clarifying_owl`) lands, doctor/health output will correctly attribute why world is disabled (flag vs env vs config provenance).
- Replay (and other world-adjacent flows) still emit warnings/origin summaries that can be generic or misleading, especially when users follow “install with `--no-world`, enable later” workflows.
- Misattribution is worse than a generic message: it directs operators to the wrong knob (flag vs env vs config file), increasing confusion and support time.

## Goals

- Replay warnings/origin summaries must attribute the _highest-precedence_ world-disable reason using the same semantics and phrasing as doctor/health (ADR-0037).
- Preserve replay’s execution selection semantics (world vs host) as they exist today; this ADR is attribution + messaging only.
- Avoid leaking sensitive data: do not print env values beyond key names; use stable tokenized config display paths (not absolute host paths).
- Avoid new noise in healthy/successful paths: only modify existing replay warning/origin surfaces.

## Non-Goals

- Changing replay routing behavior (host vs world selection), timeouts, or backend selection.
- Reworking config precedence or provenance semantics (reuse the existing model from ADR-0037).
- Applying disable-attribution to non-replay UX surfaces (doctor/health are handled by ADR-0037; other world-adjacent commands are follow-ups).

## Out of Scope

- Introducing a new replay JSON schema or a new user-facing replay subcommand solely for attribution.
- Rewriting or reformatting existing replay verbose output beyond the minimal attribution substitution.
- Changing the replay toggle precedence `--world > --no-world > SUBSTRATE_REPLAY_USE_WORLD` (replay-specific knob remains).

## Slice Decomposition

- C0 — Enumerate replay attribution surfaces
  - Scope: Identify the replay stderr surfaces that mention host-only/world-disabled semantics (at minimum: replay origin summary and the “running on host” warning) and define the exact attribution string contract they must share with doctor/health.
- C1 — Implement shared attribution in replay output + trace fields
  - Scope: Reuse the ADR-0037 disable-attribution classifier to render the correct reason (flag/env/workspace/global) in replay messages and (if present) structured replay strategy fields, without changing replay selection behavior.
- C2 — Regression tests for precedence + redaction
  - Scope: Add/extend tests to cover CLI flag, override env, workspace config, and global config cases; assert highest-precedence selection and that no sensitive env values or absolute paths leak.

## Options

### Option A — Reuse the shared “world disabled attribution” helper (recommended)

Refactor/locate the attribution logic (introduced by ADR-0037) in a shared helper and call it from replay routing, so replay and doctor/health share the same precedence mapping, enum-like reason codes, and human-friendly attribution string.

### Option B — Replay-local attribution duplication

Implement the same precedence mapping and formatting in replay routing without sharing code with doctor/health.

## Recommendation

- Choose Option A when we want stable, shared semantics and a single precedence implementation to test.
- Choose Option B when refactor constraints block Option A and a short-lived bridge is required (with an explicit follow-up to remove duplication).

## User Contract (Authoritative)

### CLI

- Surface(s) in scope:
  - `substrate --replay <span_id> [--replay-verbose]` (ASSUMPTION: primary replay UX surface today).
- When replay emits an “origin” summary and/or “running on host” warning due to world isolation being disabled, the attribution **must** reflect the highest-precedence disable source using the same contract as doctor/health (ADR-0037).
  - Allowed attribution tokens (stable, non-secret):
    - `--no-world` (CLI flag)
    - `SUBSTRATE_OVERRIDE_WORLD=disabled` (override env)
    - `<workspace>/.substrate/workspace.yaml` (workspace config provenance)
    - `$SUBSTRATE_HOME/config.yaml` (global config provenance)
- No change to replay exit codes or stderr/stdout routing; only attribution text changes within existing replay warning/origin surfaces.

### Config

- Precedence and provenance semantics are inherited from ADR-0037 (doctor/health):
  1. CLI `--no-world`
  2. env override `SUBSTRATE_OVERRIDE_WORLD=disabled`
  3. workspace config `<workspace>/.substrate/workspace.yaml` with `world.enabled: false`
  4. global config `$SUBSTRATE_HOME/config.yaml` with `world.enabled: false`

### Platform guarantees

- Linux/macOS/Windows: replay warning/origin attribution uses the same precedence mapping and does not leak sensitive values; platform-specific replay backends may differ, but attribution must be consistent.

## Architecture Shape

- Components:
  - `crates/shell/src/execution/routing/replay.rs`: render replay origin/warn surfaces using shared disable attribution when world is disabled by effective config/env/flag.
  - `crates/shell/src/execution/config_model.rs`: source of effective config + provenance (reused via the ADR-0037 helper).
  - `crates/replay/src/replay/executor.rs`: `replay_strategy` trace event; include (additively) stable disable-attribution fields when replay runs host due to world being disabled (ASSUMPTION: via `extra` fields or new optional fields on `ExecutionState`).
- End-to-end flow:
  - Inputs: CLI replay flags, env `SUBSTRATE_OVERRIDE_WORLD`, workspace/global config for `world.enabled`, replay-specific toggle `SUBSTRATE_REPLAY_USE_WORLD`
  - Derived state: effective `world.enabled` + provenance; replay-selected origin (world/host) + reason
  - Actions: compute disable attribution string + stable reason code; render in replay warnings/origin and (optionally) trace fields
  - Outputs: corrected replay stderr messaging; additive structured replay strategy metadata (where applicable)

## Sequencing / Dependencies

- Sequencing entry: `docs/project_management/packs/sequencing.json` → TBD (to be added during planning)
- Prerequisites:
  - ADR-0037 (`clarifying_owl`) must land first (shared attribution classifier + enum/phrasing contract).
- Integration task IDs:
  - TBD (ASSUMPTION: this work can be scheduled after the `clarifying_owl` integration task lands).

## Work Lift (discovery estimate)

<!-- PM_LIFT_VECTOR:BEGIN -->

```json
{
  "touch": {
    "create_files": null,
    "edit_files": 2,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 2,
    "boundary_crossings": null
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": null, "new_test_cases": null },
  "docs": { "new_docs_files": 0 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": true,
    "security_sensitive": false,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": null
  },
  "notes": "Discovery estimate; replay/warnings messaging alignment with doctor/health disable attribution."
}
```

<!-- PM_LIFT_VECTOR:END -->

## Security / Safety Posture

- Fail-closed vs degrade: unchanged (this ADR is messaging + attribution only).
- Sensitive data handling:
  - Do not print raw env values beyond the env var key name (allowed fixed token: `SUBSTRATE_OVERRIDE_WORLD=disabled`).
  - Use tokenized config display paths (`$SUBSTRATE_HOME/config.yaml`, `<workspace>/.substrate/workspace.yaml`) rather than absolute host paths.
- Invariants:
  - Attribution must match the effective highest-precedence disable reason; if provenance is unavailable, fall back to a generic non-misattributing message (e.g., “world isolation disabled by effective config (source unknown)”).

## Validation Plan (Authoritative)

### Tests

- Unit tests:
  - Reuse or extend the ADR-0037 attribution helper unit tests to cover replay callers (formatting + code mapping).
- Integration tests:
  - Extend `crates/shell/tests/replay_world.rs` (or adjacent replay integration coverage) to assert that replay’s host-mode warning/origin text attributes disablement correctly for:
    - CLI `--no-world`
    - env override `SUBSTRATE_OVERRIDE_WORLD=disabled`
    - workspace config `<workspace>/.substrate/workspace.yaml` with `world.enabled: false`
    - global config `$SUBSTRATE_HOME/config.yaml` with `world.enabled: false`
  - Assert redaction rules: no absolute host paths; no env values beyond key names.

### Manual validation

- None required beyond smoke-level CLI checks for this ADR (ASSUMPTION: no backend/provisioning changes).

### Smoke scripts

- None required for this ADR (no world backend changes).

## Rollout / Backwards Compatibility

- Policy: additive/non-breaking output changes; replay behavior unchanged.
- Compat work:
  - Text output parsing (if any) may need adjustment; prefer structured fields for automation when available.

## Decision Summary

- Decision Register entries (if applicable):
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/decision_register.md` (required):
    - DR-0001 — Implementation strategy: shared helper reuse vs replay-local duplication.
    - DR-0002 — Trace/JSON surface: add explicit `world_disable_reason` fields for replay vs reuse existing `origin_reason(_code)` only.
- Options (required; at least two):
  - A) Central “disable attribution” helper reused by doctor/health/replay (recommended).
  - B) Replay-specific heuristic duplication (temporary bridge; drift risk).
- Selection:
  - Chosen: A
  - Rationale: Keeps attribution semantics and precedence consistent across UX surfaces and reduces drift risk; tests can be shared.
  - Choose A when: we want stable, shared semantics and can afford small refactors/plumbing.
  - Choose B when: refactor scope blocks shipping and we accept a short-lived bridge with an explicit follow-up to remove duplication.
