# WDRA0-spec — shared replay disable-attribution classifier seam

## Behavior delta (single)
- Existing: replay can fall back to host execution because `world.enabled=false`, but the replay path does not expose one shared, replay-safe classifier that identifies the winning effective-disable layer with tokenized path/env displays aligned to ADR-0037 semantics.
- New: replay gains one shared helper or helper-adjacent seam that resolves effective-disable attribution for `world.enabled=false`, returning normalized source metadata that later slices can reuse for stderr copy and telemetry without re-implementing precedence logic.
- Why: keep effective-disable attribution deterministic, redacted, and reusable across replay operator output and trace emission.

## Scope
- Add one replay-safe helper or helper-adjacent seam that returns the winning effective-disable layer for `world.enabled=false`.
- Normalize attribution output to the fixed fragments and tokenized path displays required by `contract.md`.
- Add targeted tests for precedence, workspace override handling, and path/env redaction.

Likely touch surfaces (non-authoritative):
- `crates/shell/src/execution/config_model.rs`
- `crates/shell/src/execution/routing/replay.rs`
- `crates/replay/src/replay/executor.rs`
- `crates/shell/tests/replay_world.rs`

## Behavior (authoritative)

### Effective-disable attribution seam
- Replay resolves effective-disable attribution through one shared seam that returns exactly one of these outcomes:
  - override env
  - workspace patch
  - global patch
  - unknown source
- The seam never returns absolute host paths.
- The seam never returns raw env values.
- The seam does not redefine replay-local opt-out precedence or replay backend-selection rules.
- Workspace wins over `SUBSTRATE_OVERRIDE_WORLD` when a workspace exists, matching `docs/reference/env/contract.md` and ADR-0037 semantics.

## Acceptance criteria
- AC-WDRA0-01: replay can query one shared helper or helper-adjacent seam for effective-disable attribution without duplicating precedence logic.
- AC-WDRA0-02: the seam emits tokenized path displays only: `<workspace>/.substrate/workspace.yaml` or `$SUBSTRATE_HOME/config.yaml`.
- AC-WDRA0-03: the seam emits `SUBSTRATE_OVERRIDE_WORLD` as a variable name only and never emits a raw env value outside the fixed allowlisted fragment.
- AC-WDRA0-04: tests prove workspace config beats `SUBSTRATE_OVERRIDE_WORLD` when a workspace exists.

## Out of scope
- replay stderr copy changes
- `replay_strategy` field additions
- operator-doc edits
