# world-disabled-reason-attribution — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-disabled-reason-attribution/`
- ADRs:
  - `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`
  - `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
- Spec manifest:
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created or edited. Use repo-relative paths.

### Create
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/alignment_report.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/decision_register.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/manual_testing_playbook.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/platform-parity-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/quality_gate_report.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/session_log.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA0/WDRA0-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA1/WDRA1-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA2/WDRA2-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/windows-smoke.ps1`

### Edit
- `crates/shell/src/execution/config_model.rs`
- `crates/shell/src/execution/routing/replay.rs`
- `crates/replay/src/replay/executor.rs`
- `crates/shell/tests/replay_world.rs`
- `docs/COMMANDS.md`
- `docs/REPLAY.md`
- `docs/TRACE.md`
- `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json`
- `docs/project_management/packs/sequencing.json`

### Deprecate
- None

### Delete
- None

## Cascading implications (behavior and UX)

### Replay origin and warning copy
- Change: replay origin summaries and host warnings gain effective disable attribution when host replay is explained by `world.enabled=false`.
  - Direct impact:
    - replay no longer implies `--no-world` when the active cause is persisted config or `SUBSTRATE_OVERRIDE_WORLD=disabled`.
    - operators can move straight to the winning layer that disabled world isolation.
  - Cascading impact:
    - `contract.md` must lock the exact reason fragments and tokenized path displays.
    - replay tests must cover override env, workspace config, global config, and unknown-source fallback.
  - Contradiction risk:
    - replay-local opt-outs (`--no-world`, `SUBSTRATE_REPLAY_USE_WORLD=disabled`, `--flip-world`) must stay stable and distinct from effective-config attribution.

### Replay telemetry
- Change: `replay_strategy` gains additive disable-attribution fields when replay stays on host due to effective world disablement.
  - Direct impact:
    - trace consumers gain stable non-secret provenance fields instead of parsing a free-form stderr fragment.
  - Cascading impact:
    - `telemetry-spec.md` must define field names, emission rules, and redaction rules.
    - `docs/TRACE.md` must stay aligned with the final field names and enum values.
  - Contradiction risk:
    - `origin_reason_code` must remain valid for existing replay-local causes; new config-attribution values must extend the enum set rather than replace existing values.

### Provenance and redaction
- Change: replay reuses ADR-0037 provenance semantics for `world.enabled=false`.
  - Direct impact:
    - `SUBSTRATE_OVERRIDE_WORLD=disabled` is allowed as a fixed non-secret token.
    - config origins use tokenized displays: `<workspace>/.substrate/workspace.yaml` and `$SUBSTRATE_HOME/config.yaml`.
  - Cascading impact:
    - `crates/shell/src/execution/config_model.rs` or a helper adjacent to it must expose the same winning-layer explanation that doctor and health use.
    - replay tests must assert that no absolute host paths and no raw env values leak.
  - Contradiction risk:
    - the workspace rule from `docs/reference/env/contract.md` remains authoritative: when a workspace exists, `SUBSTRATE_OVERRIDE_WORLD` does not win.

## Cross-queue scan (ADRs and Planning Packs)

### ADR-0037 and `docs/project_management/packs/draft/make-doctor-health-output-explain-why/`
- Overlap surfaces:
  - shared effective-config attribution semantics for `world.enabled`
  - tokenized path display contract
- Conflict: yes
- Resolution:
  - ADR-0037 owns the shared classifier semantics and doctor/health wording.
  - ADR-0038 reuses that classifier and extends only replay surfaces and replay telemetry.

### `docs/project_management/packs/draft/json-mode/`
- Overlap surfaces:
  - trace and machine-readable output docs
- Conflict: low
- Resolution:
  - keep replay telemetry additive and top-level inside the existing `replay_strategy` object.

## Full-planning follow-ups

- Lock a single structured telemetry contract before task wiring lands.
- Keep the touch set aligned with the final helper placement in `crates/shell/src/execution/config_model.rs` or an adjacent shared helper.
- Keep `docs/REPLAY.md`, `docs/TRACE.md`, and `docs/COMMANDS.md` in the same slice that finalizes regression coverage so operator docs and telemetry docs land together.
