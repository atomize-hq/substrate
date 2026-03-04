# world-disabled-reason-attribution — plan (v1)

## Scope

- Feature directory: `docs/project_management/packs/draft/world-disabled-reason-attribution/`
- Planning standard:
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
- Authoritative contract inputs (source of truth):
  - `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md` (tokens + precedence; reused verbatim)
  - `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md` (feature context)
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md` (operator-facing stderr wording)
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md` (trace schema contract)
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA0/WDRA0-spec.md` (slice ACs + boundary)
  - `docs/reference/env/contract.md` (replay gating + toggles; MUST NOT be changed)

## Goal (operator-facing)

When replay executes on `host` specifically because effective config has `world.enabled=false`, attribute the highest-precedence disable source (ADR-0037) in:

- verbose replay stderr (origin summary + host-mode warning), and
- `event_type="replay_strategy"` telemetry (`world_disable_reason` / `world_disable_source`),

without changing replay routing/selection semantics or leaking sensitive data.

## Non-negotiable invariants

- Reuse ADR-0037 disable-attribution enum values, fields, and precedence **verbatim** (no replay-local taxonomy).
- No replay routing/selection semantic changes (host vs world selection rules stay the same).
- Redaction: no absolute host paths; no env value leaks beyond fixed allowlisted tokens.
- Absence semantics: omit fields when not applicable (do not emit `null`).

## Cross-platform posture

- Behavior smoke platforms: `linux`, `macos`, `windows`
- CI parity platforms: `linux`, `macos`, `windows`

## Slices + checkpoints

- `WDRA0` — Replay verbose stderr attribution + additive replay_strategy telemetry (`world_disable_*`) + tests/docs alignment.
- Checkpoint boundary: `CP1` after `WDRA0` (see `pre-planning/ci_checkpoint_plan.md`).

## Task graph status

- Target execution model: schema v4 cross-platform automation triads + CI checkpoint wiring (per `pre-planning/ci_checkpoint_plan.md`).
- Current limitation: this PWS output allowlist does not include `kickoff_prompts/`, which blocks wiring the schema v4 task graph in `tasks.json`.
- Current `tasks.json` is a mechanically-valid stub (schema v2 + automation disabled) that records the unblock task and the intended wiring task.
