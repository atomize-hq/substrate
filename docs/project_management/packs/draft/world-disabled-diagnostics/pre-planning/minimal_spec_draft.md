**PRE‑PLANNING ONLY — This document is a temporary alignment backbone and MUST be deleted or retired during full planning.**

# world-disabled-diagnostics — minimal spec draft (pre-planning)

## Scope + authority

This draft defines **cross-cutting defaults, precedence, and invariants only** for the planning pack at:
- `docs/project_management/packs/draft/world-disabled-diagnostics/`

Authority inputs (derive-only; do not extend scope):
- ADR: `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
- Spec manifest: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md`
- Impact map: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md`

This draft MUST NOT define:
- Slice-specific behavior, acceptance criteria, or implementation tasks.
- Detailed JSON schemas, field lists, or enum spellings (owned by `decision_register.md` + `diagnostics-json-schema-spec.md`).
- Any new surfaces not already implied by ADR-0036 (no new CLI flags, config keys, env vars, protocols, telemetry fields).

## Defaults + precedence

### Effective `world.enabled` resolution (authoritative; unchanged by this feature)

Diagnostics in scope (`substrate health`, `substrate shim doctor`) MUST resolve the effective `world.enabled` value using the **existing Substrate effective-config resolver** and its precedence rules (no alternate precedence stack).

Authoritative references for precedence + semantics:
- `docs/CONFIGURATION.md`
- `docs/reference/env/contract.md`

Precedence (explicit; aligns with ADR-0036 and env contract):
- When an enabled workspace exists:
  1. CLI flags: `--world` / `--no-world`
  2. Workspace config patch: `<workspace_root>/.substrate/workspace.yaml`
  3. Global config patch: `$SUBSTRATE_HOME/config.yaml` (default: `~/.substrate/config.yaml`)
  4. Built-in defaults
- When no enabled workspace exists:
  1. CLI flags: `--world` / `--no-world`
  2. Override env inputs: `SUBSTRATE_OVERRIDE_WORLD`
  3. Global config patch: `$SUBSTRATE_HOME/config.yaml` (default: `~/.substrate/config.yaml`)
  4. Built-in defaults

Environment/path invariants:
- `SUBSTRATE_HOME` path resolution and defaults are unchanged; it remains the source for global config lookup.
- `SUBSTRATE_OVERRIDE_WORLD` parsing and invalid-value behavior are unchanged; this feature consumes the resolved effective value.

## Failure posture + invariants

### Posture

- When effective `world.enabled=false`:
  - Diagnostics MUST degrade by **skipping world-backend probes performed solely for diagnostics**.
  - Diagnostics MUST report an explicit, machine-detectable **disabled** status for the world backend and an explicit **skipped because disabled** status for world-deps probing (non-error).
  - Diagnostics MUST NOT imply world health or world-deps “applied” state when probes are skipped.

- When effective `world.enabled=true`:
  - Diagnostics MUST remain fail-closed: real world backend failures remain visible as “needs attention” (or equivalent) with actionable error details.
  - Diagnostics MUST NOT mask backend unavailability/errors.

### Security / redaction invariants (high level)

- This feature MUST NOT introduce new sensitive data surfaces.
- This feature MUST NOT add or modify structured log schema fields, trace span fields, or redaction rules.
- New world/world-deps status fields (text + JSON) are treated as non-sensitive operational state.

## Exit-code posture

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This work requires **no new exit codes**.
- Default posture (from ADR-0036; must be made explicit in `contract.md` during full planning):
  - Successful report generation exits `0` regardless of disabled/skipped vs “needs attention”.
  - Non-zero exits are reserved for command execution failures (usage/config/serialization/dependency when required by the command).

## Cross-cutting seams / constraints

### Disabled/skipped states (text + JSON)

- Both diagnostics surfaces (`substrate health`, `substrate shim doctor`) MUST encode the disabled/skipped states consistently:
  - World backend: explicit **disabled** state when effective `world.enabled=false`.
  - World deps: explicit **skipped because disabled** state when effective `world.enabled=false`.
- Exact JSON field paths + enum spellings are owned by DR-0001 and MUST be identical across both commands once decided.
- Legacy error-field behavior when disabled/skipped is owned by DR-0002; the chosen behavior MUST avoid ambiguous “failure-looking” encodings for intentionally disabled operation.

### Deterministic operator-facing copy (stable phrases)

- Deterministic copy constraints (stable phrases/substrings) for the disabled/skipped states are owned by DR-0003 and MUST be enforced across both commands.
- Disabled remediation guidance MUST be minimal and must not imply provisioning failures:
  - The remediation path for disabled-by-choice is “enable world isolation” (e.g., `substrate world enable`) and not “fix the backend”.

### Compatibility + cross-queue boundaries

- JSON changes are additive-only; existing payload fields remain present and unchanged (schema spec must inventory current outputs before adding fields).
- `SUBSTRATE_OVERRIDE_WORLD` remains the override input; `SUBSTRATE_WORLD` / `SUBSTRATE_WORLD_ENABLED` remain exported state variables and MUST NOT be treated as operator override inputs.
- `json-mode` pack overlap boundary: WDD status fields remain stable and MUST NOT be renamed/removed by any cross-command JSON envelope work without an explicit compat posture.

### No-new-surfaces constraints

- No new CLI flags or subcommands.
- No new config keys.
- No new environment variables.
- No new host↔agent protocol surfaces; only conditional probe invocation changes are allowed.
- No new telemetry/log schema fields.

## Follow-ups for full planning

1) Decision Register (required; blocks implementation):
   - DR-0001: Decide JSON field paths + enum spellings for world/world-deps status fields (avoid collisions with existing world-doctor vocabulary).
   - DR-0002: Decide legacy `error`/`ok`-style field behavior when disabled/skipped (strictly additive; unambiguous non-error posture).
   - DR-0003: Decide deterministic copy constraints (stable phrases/substrings) for disabled/skipped messaging across `substrate health` and `substrate shim doctor`.

2) Additive-schema grounding (required):
   - Inventory the full existing `--json` shapes for `substrate shim doctor --json` and `substrate health --json` before specifying additive fields in `diagnostics-json-schema-spec.md`.

3) Exit semantics confirmation (required):
   - Confirm current exit behavior for `substrate health` and `substrate shim doctor` across:
     - disabled/skipped state,
     - enabled + backend broken (“needs attention”),
     - config/env parse failures (e.g., invalid `SUBSTRATE_OVERRIDE_WORLD`),
     and encode the final contract in `contract.md`.

4) Pre-planning doc integrity (required):
   - Remove any “…chars truncated…” placeholders present in `pre-planning/spec_manifest.md` and `pre-planning/impact_map.md` before planning quality gate.

## Draft slice skeleton (pre-planning only)

Disclaimer: **draft; may split/merge; do not wire `tasks.json` yet.**

Slice prefix (draft): `WDD`

Per `pre-planning/spec_manifest.md`, baseline slice set:

### Slice entry: `WDD0`
- slice_id: `WDD0`
- name: Resolve effective `world.enabled` for diagnostics
- intent: Stabilize how diagnostics consumes the canonical effective-config resolver so later classification can branch deterministically on the resolved `world.enabled` value.
- likely touch surfaces:
  - `crates/shell/src/execution/config_model.rs`
  - `crates/shell/src/builtins/shim_doctor/report.rs`
  - `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/WDD0-spec.md`

### Slice entry: `WDD1`
- slice_id: `WDD1`
- name: Classify disabled/skipped diagnostics; skip probes when disabled
- intent: Enforce the disabled posture: no world probes and explicit disabled/skipped statuses in both text and JSON; preserve fail-closed behavior when enabled.
- likely touch surfaces:
  - `crates/shell/src/builtins/shim_doctor/report.rs`
  - `crates/shell/src/builtins/shim_doctor/output.rs`
  - `crates/shell/src/builtins/world_deps/mod.rs`
  - `crates/shell/src/builtins/health.rs`
  - `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD1/WDD1-spec.md`

### Slice entry: `WDD2`
- slice_id: `WDD2`
- name: Lock contract + schema + validation coverage
- intent: Make the user contract and validation plan deterministic: decision register selections, additive JSON schema spec, and runnable test/manual/smoke coverage.
- likely touch surfaces:
  - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
  - `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md`
  - `docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md`
  - `crates/shell/tests/shim_doctor.rs`
  - `crates/shell/tests/shim_health.rs`
  - `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD2/WDD2-spec.md`

Downstream note:
- CI-checkpoint planning MUST prefer this slice list when populating the slice list in `pre-planning/ci_checkpoint_plan.md` (no mechanical validation until `tasks.json` exists).
- Workstream triage may propose edits to this slice skeleton as recommendations in `pre-planning/workstream_triage.md` (this file remains unchanged during pre-planning).

