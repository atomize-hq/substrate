# world-disabled-reason-attribution — minimal spec draft (pre-planning)

Standards:
- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- ADR: `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`
- Prerequisite ADR: `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
- Prerequisite planning pack: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/`
- Touch-set source: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md`

## Problem framing

Replay already reports explicit replay-local opt-outs (`--no-world`, `SUBSTRATE_REPLAY_USE_WORLD=disabled`, `--flip-world`).
That part stays intact.

The missing seam is effective disable attribution when replay stays on host because `world.enabled=false` after the same precedence chain that doctor and health use.
Without that seam, replay either looks generic or points operators at the wrong knob.

## Invariants (authoritative for full planning)

- Replay selection precedence stays unchanged:
  - `--world`
  - `--no-world`
  - `SUBSTRATE_REPLAY_USE_WORLD`
  - recorded origin plus `--flip-world`
- Effective disable attribution reuses ADR-0037 semantics for `world.enabled=false`:
  - `SUBSTRATE_OVERRIDE_WORLD=disabled` when no workspace exists
  - `<workspace>/.substrate/workspace.yaml` with `world.enabled: false`
  - `$SUBSTRATE_HOME/config.yaml` with `world.enabled: false`
  - fallback `world isolation disabled by effective config (source unknown)` when provenance cannot be trusted
- Replay-local opt-out fragments remain exact and unchanged:
  - `--no-world flag`
  - `SUBSTRATE_REPLAY_USE_WORLD=disabled`
  - `--flip-world`
- Redaction invariants:
  - no raw env value leaks beyond the fixed allowlisted tokens above
  - no absolute config paths in stderr or telemetry
- Telemetry invariants:
  - additive-only changes to `replay_strategy`
  - existing field names remain stable
  - new fields emit only for effective-config disable attribution

## Draft behavior contract

Human-visible reason fragments for effective disable attribution:
- `world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled`
- `world isolation disabled by config <workspace>/.substrate/workspace.yaml (world.enabled: false)`
- `world isolation disabled by config $SUBSTRATE_HOME/config.yaml (world.enabled: false)`
- `world isolation disabled by effective config (source unknown)`

Replay surfaces in scope:
- `[replay] origin: ...`
- `[replay] warn: running on host (...)`
- `replay_strategy` trace event

## Follow-ups for full planning

- DR-0001: lock helper placement and shared-classifier reuse strategy.
- DR-0002: lock the telemetry field set and `origin_reason_code` extension rules.
- DR-0003: lock the exact formatting for the recorded-host case so replay prints one deterministic origin line and one deterministic host warning.

## Draft slice skeleton (pre-planning only)

Slice prefix: `WDRA`

- `slice_id`: `WDRA0`
  - `name`: Shared replay disable-attribution classifier seam
  - `intent`: expose a shared winning-layer explanation for `world.enabled=false` that replay can call without redefining precedence or redaction.
  - `likely touch surfaces`:
    - `crates/shell/src/execution/config_model.rs`
    - `crates/shell/src/execution/routing/replay.rs`
    - `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md`
    - `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA0/WDRA0-spec.md`

- `slice_id`: `WDRA1`
  - `name`: Replay stderr copy and telemetry wiring
  - `intent`: render the correct reason fragment in replay stderr and add structured replay_strategy fields for effective disable attribution.
  - `likely touch surfaces`:
    - `crates/shell/src/execution/routing/replay.rs`
    - `crates/replay/src/replay/executor.rs`
    - `docs/TRACE.md`
    - `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md`
    - `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA1/WDRA1-spec.md`

- `slice_id`: `WDRA2`
  - `name`: Regression coverage and docs alignment
  - `intent`: extend replay tests for precedence and redaction, update replay docs, and lock cross-platform smoke/manual evidence.
  - `likely touch surfaces`:
    - `crates/shell/tests/replay_world.rs`
    - `docs/REPLAY.md`
    - `docs/COMMANDS.md`
    - `docs/project_management/packs/draft/world-disabled-reason-attribution/manual_testing_playbook.md`
    - `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA2/WDRA2-spec.md`
