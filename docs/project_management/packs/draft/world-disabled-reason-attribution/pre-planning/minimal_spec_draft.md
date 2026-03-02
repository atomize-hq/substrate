**PRE‑PLANNING ONLY — This document is a draft alignment backbone and MUST be deleted or retired during full planning.**

## Scope + authority

This draft is allowed to define **only** cross-cutting defaults, precedence, and invariants needed to keep feature-local specs aligned while they are drafted.

This draft must **not** define:
- Exact replay stderr line templates (owned by `contract.md`).
- Exact telemetry field placement and schema details (owned by `telemetry-spec.md`).
- Slice-level behavior details and acceptance criteria (owned by `slices/WDRA0/WDRA0-spec.md`).
- Any changes to world/replay routing behavior (explicitly out of scope per ADR-0038).

Authoritative inputs (this feature MUST NOT redefine):
- World-disable attribution contract (tokens/enums/strings + precedence): `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
- Env var parsing + precedence: `docs/reference/env/contract.md`
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Feature intent + constraints: `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`
- Required-doc ownership map: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/spec_manifest.md`
- Touch set + conflicts: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md`

## Defaults + precedence

### `world.enabled` disable-attribution precedence (source of truth: ADR-0037)

Disable attribution MUST match the effective winner for `world.enabled` with the following precedence order:
1. CLI flags: `--world` / `--no-world` (when provided)
2. Workspace config patch: `<workspace>/.substrate/workspace.yaml` (when workspace exists and is enabled)
3. Override env: `SUBSTRATE_OVERRIDE_WORLD` (applies only when no workspace is enabled)
4. Global config patch: `$SUBSTRATE_HOME/config.yaml` (default: `~/.substrate/config.yaml`)
5. Default config

Tokenized display paths used by this feature are fixed:
- `$SUBSTRATE_HOME/config.yaml`
- `<workspace>/.substrate/workspace.yaml`

### Replay toggles (out of scope to change; source of truth: env contract)

Replay world-toggle precedence is owned by `docs/reference/env/contract.md` and MUST NOT be changed by this feature.

## Failure posture + invariants

- Behavior posture: this feature is **attribution + messaging only**. Replay routing/selection semantics and world enable/disable behavior MUST remain unchanged.
- Misattribution posture: replay messaging MUST NOT claim “world disabled by flag/env/config” unless host-mode replay is attributable to `world.enabled=false` being effective.
- Unknown-provenance posture: if disable provenance cannot be determined, the feature MUST use a single deterministic fallback that does not misattribute to `--no-world` (fallback wording is owned by `contract.md`, sourced from ADR-0037 constraints).
- Redaction posture:
  - Absolute host paths MUST NOT appear in replay stderr or telemetry fields introduced/modified by this feature.
  - Env values MUST NOT be printed beyond key names; the only allowed fixed token is `SUBSTRATE_OVERRIDE_WORLD=disabled`.
- Telemetry posture: trace changes MUST be additive-only and must define explicit absence semantics (fields omitted when not applicable).

## Exit-code posture

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This feature introduces **no new exit codes** and makes **no exit-code behavior changes**.

## Cross-cutting seams / constraints

### Disable-attribution tokens and field names (source of truth: ADR-0037)

If this feature emits disable attribution in any structured output, it MUST use ADR-0037 field names and values verbatim:
- `world_disable_reason` (string enum): `cli_flag`, `override_env`, `workspace_patch`, `global_patch`, `default`
- `world_disable_source` (object) with stable keys and semantics, including:
  - `key`: `world.enabled`
  - `layer`: matches `world_disable_reason`
  - `flag` / `env` / `path_display` (optional; conditional on layer)
  - `value_display`: the string `false` when present

### Replay messaging boundaries (source of truth: ADR-0038 + impact map)

Replay can be host-mode for reasons that are not “world disabled by effective config” (replay opt-out toggles, recorded-origin constraints, platform limitations). This feature’s attribution must not collapse those distinct reasons into a single “world disabled” narrative.

### Telemetry naming alignment

If replay telemetry records world-disable attribution, it MUST align naming and meaning with ADR-0037 and MUST NOT introduce replay-specific synonyms for those fields.

## Follow-ups for full planning

1) Reconcile ADR-0038 precedence text with ADR-0037 + `docs/reference/env/contract.md` and update ADR-0038 so it does not contradict “workspace overrides env override”.
2) Define the exact condition under which replay is “host due to world-disablement” vs:
   - host due to replay opt-out toggles, or
   - host due to recorded-origin constraints, or
   - host due to platform limitations (no world isolation available).
3) Decision Register DR-0002: choose the replay trace contract strategy (explicit `world_disable_reason` / `world_disable_source` on `replay_strategy` vs overloading existing fields) and lock one option in `telemetry-spec.md` with explicit absence semantics.
4) Inventory the current replay stderr surfaces and exact string templates in `crates/shell/src/execution/routing/replay.rs` to pin the minimal substitution points and ensure redaction invariants.
5) Confirm replay verbose gating contract (flag/env precedence and absence semantics) in `docs/reference/env/contract.md` and reflect it consistently in `contract.md` and the slice spec.

## Draft slice skeleton (pre-planning only)

Disclaimer: draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): `WDRA`

- `slice_id`: `WDRA0`
  - `name`: Attribute world-disabled reason in replay output
  - `intent`: When replay runs on host because `world.enabled=false` is effective, emit the highest-precedence disable attribution (ADR-0037) in replay verbose stderr surfaces and record aligned attribution in replay strategy telemetry (additive).
  - `likely touch surfaces`:
    - `crates/shell/src/execution/routing/replay.rs`
    - `crates/shell/src/execution/config_model.rs`
    - `crates/replay/src/replay/executor.rs`
    - `crates/replay/src/replay/mod.rs`
    - `crates/shell/tests/replay_world.rs`
    - `crates/shell/tests/world_disable_attribution.rs`
    - `docs/REPLAY.md`
    - `docs/TRACE.md`

Downstream note:
- CI-checkpoint MUST prefer this slice list when populating the machine-readable slice list in `pre-planning/ci_checkpoint_plan.md` (do not validate mechanically until slice tasks exist in `tasks.json`).
- Workstream triage may propose edits to this slice skeleton as recommendations in `pre-planning/workstream_triage.md` (must not edit this file).
