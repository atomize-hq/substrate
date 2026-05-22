# ADR-0037 — Doctor/Health: attribute why world isolation is disabled (flag vs env vs config)

## Status

- Status: Draft
- Date (UTC): 2026-02-21
- Owner(s): TBD (ASSUMPTION: Substrate shell maintainers)

## Scope

- Feature directory: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/` (ASSUMPTION: created during planning)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

- Intake: `docs/project_management/intake/adrs/clarifying_owl_adr_intake.md`
- Plan: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/plan.md` (planned)
- Tasks: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/tasks.json` (planned)
- Spec manifest: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/spec_manifest.md` (planned)
- Decision Register: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md` (required; see “Decision Summary”)
- Impact Map: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/impact_map.md` (recommended; support/debug + CI implications)

## Executive Summary (Operator)

ADR_BODY_SHA256: 371c0fb08267e913c4537d53c2ad18f5dddfee23535d26760886fc06faa5552a

### Changes (operator-facing)

- Doctor/health output explains _why_ world isolation is disabled (flag vs env vs config), without changing behavior.
  - Existing: `substrate host doctor`, `substrate world doctor`, and sometimes `substrate health` can report “world isolation disabled by effective config (--no-world)” even when the actual disablement source is persisted config (`$SUBSTRATE_HOME/config.yaml` / workspace `.substrate/workspace.yaml`) or an override env var (`SUBSTRATE_OVERRIDE_WORLD=disabled`). This misleads operators and increases debug time.
  - New: When `world.enabled=false` is effective, doctor/health output includes a short attribution string that correctly identifies the _highest-precedence_ disable source:
    - CLI flag `--no-world`, or
    - env override `SUBSTRATE_OVERRIDE_WORLD=disabled` (when applicable), or
    - workspace config `.substrate/workspace.yaml` (`world.enabled: false`), or
    - global config `$SUBSTRATE_HOME/config.yaml` (`world.enabled: false`).
      JSON outputs gain additive fields so tooling can reliably detect the disable source.
  - Why: Make “enable later” and support/debug workflows deterministic: the operator can immediately see whether they opted out via flags, environment overrides, or persisted config (and where to change it).
  - Links:
    - `docs/project_management/intake/adrs/clarifying_owl_adr_intake.md`
    - `crates/shell/src/execution/config_model.rs` (effective config + explain provenance for `world.enabled`)
    - `crates/shell/src/execution/platform/mod.rs` (doctor entrypoints)
    - `crates/shell/src/execution/platform/linux.rs` / `crates/shell/src/execution/platform/macos.rs` / `crates/shell/src/execution/platform/windows.rs` (doctor text/JSON outputs)
    - `crates/shell/src/builtins/health.rs` (health summary surface)

## Problem / Context

- World isolation can be disabled by multiple layers (CLI flags, override env, workspace config patch, global config patch).
- Current doctor output collapses these causes into a misleading “(--no-world)” attribution, even when the effective cause is config/env.
- Misattribution causes operators to debug the wrong layer (e.g., searching shell history for a missing `--no-world` instead of updating `world.enabled`), increasing support time and making “disable now, enable later” flows harder to follow.

## Goals

- When world isolation is disabled (`world.enabled=false` effective), doctor/health output must include an accurate, short attribution of the disable source.
- Preserve the existing enable/disable behavior and precedence rules; this ADR is attribution + messaging only.
- Provide additive JSON fields so CI/tooling can reliably detect whether disablement came from CLI/env/workspace/global config.
- Avoid leaking sensitive information: do not print env values beyond key names; show config paths using stable “display paths” (e.g., `$SUBSTRATE_HOME/config.yaml`, `<workspace>/.substrate/workspace.yaml`) rather than absolute host paths.

## Non-Goals

- Changing when/why world isolation is enabled or disabled (no precedence changes; no new knobs).
- Reworking world-service readiness checks, provisioning, or failure modes.
- Applying the same attribution to replay warnings or other world-adjacent UX beyond doctor/health (explicit follow-up: `replaying_raccoon` intake).
- Changing exit code behavior for doctor/health commands (additive fields + messaging only).

## Out of Scope

- Adding a new “verbose doctor” mode or expanding doctor output beyond attribution.
- Redesigning config explain output; this ADR only consumes existing provenance for `world.enabled`.
- Rewriting “workspace enabled disables env overrides” legacy behavior (note: attribution must reflect the _actual_ effective config, even if the precedence is surprising).

## Options Considered

### Option A — Use config explain provenance (recommended)

Use `resolve_effective_config_with_explain(..., explain=true)` to obtain the per-key provenance for `world.enabled` and surface it in doctor/health outputs.

**Choose Option A when…**

- We want strict correctness aligned with the real precedence model (including workspace/global vs env nuances).
- We want the attribution logic to be future-proof as precedence rules evolve (source-of-truth is config resolution).

### Option B — Heuristic attribution (CLI/env/config) without provenance

Compute the reason with a local priority check (e.g., if `--no-world` passed → CLI, else if `SUBSTRATE_OVERRIDE_WORLD=disabled` present → env, else if config contains `world.enabled=false` → config).

**Choose Option B when…**

- We want the smallest implementation delta and accept that edge cases may be misattributed unless the heuristic replicates full precedence rules.

## Recommendation

- Choose Option A when correctness matters (support/debug UX) and we can reuse the existing config “explain” machinery cheaply.
- Choose Option B when we cannot depend on explain in the doctor/health code paths and are willing to accept the risk of misattribution.
- Recommended for this ADR: **Option A**, with a small, shared “world disabled attribution” helper used by `host doctor`, `world doctor`, and `health`.

## Slice Decomposition

- C0 — Doctor text output: correct attribution when world disabled.
  - Implement a shared classifier that derives `world_disable_reason` from config provenance for `world.enabled`, and replace “(--no-world)” messaging in `substrate host doctor` and `substrate world doctor` text output with an accurate attribution string.
- C1 — Doctor/health JSON: additive fields + health surface.
  - Add additive JSON fields to `substrate host doctor --json` and `substrate world doctor --json` describing the disable source. Update `substrate health` to display and emit the same attribution (text + JSON) when world is disabled.

## User Contract (Authoritative)

### CLI

- Commands:
  - `substrate host doctor` and `substrate world doctor` (text mode):
    - If `world.enabled=true` effective: no change in behavior for this ADR.
    - If `world.enabled=false` effective: the output must include an attribution line of the form:
      - `world isolation disabled by CLI flag --no-world`
      - `world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled`
      - `world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)`
      - `world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)`
    - The attribution must reflect the _effective_ (highest-precedence) disable source used to compute `world_enabled=false`.
  - `substrate host doctor --json` / `substrate world doctor --json`:
    - Additive fields are emitted when `world_enabled=false` (see “JSON schema additions”).
  - `substrate health`:
    - When world is disabled (`world_enabled=false`), the health summary must display the same attribution string (no change to world enable/disable behavior).
- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` (no overrides in this ADR).

### Config

- Files and locations (precedence is defined by current config resolution; this ADR must reflect the effective winner):
  1. CLI flags: `--world` / `--no-world` (highest precedence when provided)
  2. Workspace config patch: `<workspace>/.substrate/workspace.yaml` (when workspace exists and is enabled)
  3. Override env: `SUBSTRATE_OVERRIDE_WORLD` (applies when no workspace exists; legacy behavior)
  4. Global config patch: `$SUBSTRATE_HOME/config.yaml` (default: `~/.substrate/config.yaml`)
  5. Default config
- Schema:
  - `world.enabled`: boolean.
    - `false`: world isolation is disabled (doctor/health must attribute the source).
    - `true`: world isolation is enabled (no attribution emitted).

### JSON schema additions (additive; no breaking changes)

When `world_enabled=false`, the following fields are added:

- `world_disable_reason`: string enum (stable):
  - `cli_flag`
  - `override_env`
  - `workspace_patch`
  - `global_patch`
  - `default`
- `world_disable_source` (object; stable keys):
  - `key`: always `world.enabled`
  - `layer`: matches `world_disable_reason`
  - `flag` (optional): `--no-world` when `layer=cli_flag`
  - `env` (optional): `SUBSTRATE_OVERRIDE_WORLD` when `layer=override_env`
  - `path_display` (optional): one of:
    - `<workspace>/.substrate/workspace.yaml` when `layer=workspace_patch`
    - `$SUBSTRATE_HOME/config.yaml` when `layer=global_patch`
  - `value_display`: always `false` (string) when present

ASSUMPTION: `path_display` intentionally uses a stable tokenized display path rather than an absolute host path.

### Platform guarantees

- Linux/macOS/Windows: parity for attribution behavior and JSON fields (best-effort where a platform’s doctor command exists; Windows host doctor is currently limited but world doctor JSON must still carry the fields).

## Architecture Shape

- Components:
  - `crates/shell/src/execution/config_model.rs`: reused via `resolve_effective_config_with_explain(..., explain=true)` to obtain provenance for `world.enabled`.
  - `crates/shell/src/execution/platform/mod.rs`: switch doctor entrypoints to request explain and pass attribution into platform doctor implementations.
  - `crates/shell/src/execution/platform/*`: replace the “(--no-world)” string and add JSON fields.
  - `crates/shell/src/builtins/health.rs` and `crates/shell/src/builtins/shim_doctor/report.rs`: surface doctor-provided disable attribution in health output (text + JSON).
- End-to-end flow:
  - Inputs: CLI `--world/--no-world`, env `SUBSTRATE_OVERRIDE_WORLD`, workspace `.substrate/workspace.yaml`, global `$SUBSTRATE_HOME/config.yaml`
  - Derived state: effective `world.enabled` plus provenance (`cli_flag|override_env|workspace_patch|global_patch|default`)
  - Actions: compute `world_disable_reason` + human attribution string; render in doctor/health outputs
  - Outputs: updated doctor/health text; additive doctor/health JSON fields

## Sequencing / Dependencies

- Sequencing entry: `docs/project_management/packs/sequencing.json` → TBD (to be added during planning)
- Prerequisite integration task IDs: none for this ADR (pure shell UX + JSON contract additions).
- Follow-up (not part of this ADR): apply the same attribution to replay warnings and other world-adjacent messaging (see `docs/project_management/intake/adrs/replaying_raccoon_adr_intake.md`).

## Work Lift (discovery estimate)

<!-- PM_LIFT_VECTOR:BEGIN -->

```json
{
  "touch": {
    "create_files": null,
    "edit_files": 1,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 1,
    "boundary_crossings": 0
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
  "notes": "Discovery estimate; fill touch/test counts during lockdown."
}
```

<!-- PM_LIFT_VECTOR:END -->

## Security / Safety Posture

- Fail-closed vs degrade: unchanged (this ADR is messaging + attribution only).
- Sensitive data handling:
  - Do not print raw env values beyond the env var name (allowed to print `SUBSTRATE_OVERRIDE_WORLD=disabled` as a fixed, non-secret token).
  - Prefer tokenized config paths (`$SUBSTRATE_HOME/config.yaml`, `<workspace>/.substrate/workspace.yaml`) over absolute host paths.
- Invariants:
  - Attribution must always reflect the effective winner for `world.enabled`; if provenance is unavailable, fall back to a generic message without misattribution (e.g., “world isolation disabled by effective config (source unknown)”).

## Validation Plan (Authoritative)

### Tests

- Unit tests:
  - Add unit tests for the attribution helper that map config explain sources (`cli_flag|override_env|workspace_patch|global_patch|default`) to:
    - `world_disable_reason` enum values, and
    - the exact human attribution string.
- Integration tests:
  - Add/extend `crates/shell/tests/` coverage to assert that:
    - With `--no-world`, doctor output attributes disablement to the CLI flag.
    - With `SUBSTRATE_OVERRIDE_WORLD=disabled` (and no workspace), doctor output attributes disablement to the env override.
    - With a workspace `.substrate/workspace.yaml` setting `world.enabled: false`, doctor output attributes disablement to workspace config.
    - With global `$SUBSTRATE_HOME/config.yaml` setting `world.enabled: false` (and no workspace), doctor output attributes disablement to global config.
    - JSON output includes `world_disable_reason` and `world_disable_source` when disabled, and omits them when enabled.

### Manual validation

- None required beyond smoke-level CLI checks for this ADR (ASSUMPTION: no platform provisioning changes).

### Smoke scripts

- None required for this ADR (no platform provisioning or world-service changes).

## Rollout / Backwards Compatibility

- Policy: additive, non-breaking output changes.
- Compat work:
  - Existing JSON consumers must tolerate new fields.
  - Existing text output parsing (if any) may need adjustment; prefer JSON for automation.

## Decision Summary

- Decision Register entries (if applicable):
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md` (required):
    - DR-0001 — Attribution implementation strategy (Option A provenance vs Option B heuristic).
    - DR-0002 — JSON contract: field names + enum values + redaction strategy for paths.
- Options (required; at least two):
  - A) Use config “explain” provenance to compute the disable source (strict correctness).
  - B) Use heuristic attribution (CLI → env → workspace → global) without full provenance (recommended).
- Selection:
  - Chosen: B
  - Rationale: Likely sufficient for operator-facing attribution while keeping implementation and plumbing lighter; the contract remains “highest-precedence reason” and can later be upgraded to provenance without changing the surface.
  - Choose A when: we can reuse existing explain structures cheaply and want strict correctness in all edge cases.
  - Choose B when: we want a small vertical slice that is correct in the common cases and can be validated with focused tests.
