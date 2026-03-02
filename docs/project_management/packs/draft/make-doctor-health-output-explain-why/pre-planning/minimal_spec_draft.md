**PRE‑PLANNING ONLY — this document is an alignment backbone draft and MUST be deleted/retired during full planning.**

# make-doctor-health-output-explain-why — minimal spec draft (pre-planning)

## Scope + authority

This document is allowed to define **cross-cutting** defaults, precedence, and invariants that every downstream spec must align on.

This document MUST NOT define:
- Slice-specific behavior and acceptance criteria
- Detailed schemas (JSON/object placement rules) beyond naming the shared seams
- Implementation tasks, worktrees, or execution sequencing details

Authoritative docs (once created) for this feature’s contract surfaces:
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` (per `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`)
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md`
- Slice specs:
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO0/DHO0-spec.md`
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO1/DHO1-spec.md`

## Defaults + precedence

Attribution correctness invariant:
- When effective `world.enabled=false`, doctor/health output attribution MUST reflect the **effective (highest-precedence) disable source** used to compute `world.enabled=false`.

Disable attribution precedence order (must match effective config resolution per ADR-0037):
1) CLI flags `--world` / `--no-world` (when provided)
2) Workspace config patch `<workspace>/.substrate/workspace.yaml` (`world.enabled: false`) (when workspace exists and is enabled)
3) Override env `SUBSTRATE_OVERRIDE_WORLD=disabled` (applies only when no workspace exists; legacy behavior)
4) Global config patch `$SUBSTRATE_HOME/config.yaml` (`world.enabled: false`)
5) Default config

Source-of-truth display paths in operator output:
- Workspace config display path: `<workspace>/.substrate/workspace.yaml`
- Global config display path: `$SUBSTRATE_HOME/config.yaml`

## Failure posture + invariants

Failure posture:
- This feature MUST NOT change doctor/health enable/disable behavior or exit codes.
- When disable-source provenance is unavailable, the feature MUST fail-open for attribution: emit a contract-defined “source unknown” attribution without guessing and without changing exit codes.

Security/redaction invariants:
- Text output MUST NOT print raw env values beyond the fixed token `SUBSTRATE_OVERRIDE_WORLD=disabled`.
- JSON output MUST NOT print env values; it MUST name the env var as `SUBSTRATE_OVERRIDE_WORLD` only.
- Output MUST use tokenized display paths (above) and MUST NOT print absolute host paths for config locations.

Platform invariants:
- `substrate world doctor --json` MUST emit the disable-attribution fields on Linux/macOS/Windows when effective `world.enabled=false` (including Windows even if `substrate host doctor` is limited).
- For any platform where `substrate host doctor` and `substrate health` exist, their attribution behavior and JSON fields MUST match the same cross-command contract (strings owned by `contract.md`, JSON owned by the schema spec).

## Exit-code posture

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This work introduces no new exit codes and defines no taxonomy overrides.

## Cross-cutting seams / constraints

Commands in scope (text + `--json`):
- `substrate host doctor`
- `substrate world doctor`
- `substrate health`

Text attribution copy:
- The same attribution string set MUST be used across doctor and health surfaces.
- Config-based attribution MUST use the tokenized display paths and include the key/value display context for `world.enabled: false` (owned by `contract.md`).

JSON seams (owned by the schema spec; listed here for cross-slice alignment only):
- Additive fields are emitted only when world is disabled (based on each output’s existing “world enabled” boolean).
- Fields:
  - `world_disable_reason` (string enum): `cli_flag`, `override_env`, `workspace_patch`, `global_patch`, `default`
  - `world_disable_source` (object with stable keys): `key`, `layer`, `flag`, `env`, `path_display`, `value_display`
- `world_disable_source.key` is `world.enabled`.
- `world_disable_source.layer` matches `world_disable_reason`.
- `world_disable_source.path_display` uses tokenized display paths (never absolute host paths).

Ordering rule:
- When multiple disable inputs are present, attribution selects the single effective winner that produced `world.enabled=false` (no multi-cause output).

## Follow-ups for full planning

Each follow-up is required to remove ambiguity before planning quality gate.

1) DR-0001 (decision_register): resolve ADR-0037’s Option A (provenance) vs Option B (heuristic) contradiction.
   - Acceptance: the selected strategy preserves the “attribution matches effective winner” invariant and matches the precedence model above (including workspace-gating for env override).

2) DR-0002 + schema spec: lock the JSON contract for doctor + health.
   - Confirm the existing “world enabled” JSON field name(s) per command (e.g., `world_enabled` vs other) and the gating rule for emitting new fields.
   - Pin the exact placement of `world_disable_reason` and `world_disable_source` inside each JSON output (strict additivity; no renames).
   - Decide whether `world_disable_reason=default` is emitted; if emitted, define when; if not emitted, remove it from the enum.
   - Confirm the `value_display` representation (type + exact value) and ensure parity across all emitting commands.

3) Integration boundary check (schema/copy): verify collision-free compatibility with queued packs listed in `pre-planning/impact_map.md` (world-disabled-diagnostics, json-mode, provisioning packs).
   - Acceptance: no field name collisions and no contradictory “world disabled” messaging when packs land in the sequenced order.

## Draft slice skeleton (pre-planning only)

Slice prefix (draft): DHO

Disclaimer: draft; may split/merge; do not wire `tasks.json` yet.

- `slice_id`: `DHO0`
  - `name`: Attribute doctor text output
  - `intent`: When effective `world.enabled=false`, doctor text output emits an accurate, actionable disable-source attribution line without changing behavior.
  - `likely touch surfaces`:
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO0/DHO0-spec.md`
    - `crates/shell/src/execution/platform/mod.rs`
    - `crates/shell/src/execution/platform/linux.rs`
    - `crates/shell/src/execution/platform/macos.rs`
    - `crates/shell/src/execution/platform/windows.rs`

- `slice_id`: `DHO1`
  - `name`: Add doctor JSON fields and surface via health
  - `intent`: When effective `world.enabled=false`, doctor/health JSON outputs emit additive disable-attribution fields, and health text output emits the same attribution string as doctor.
  - `likely touch surfaces`:
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO1/DHO1-spec.md`
    - `crates/shell/src/builtins/health.rs`
    - `crates/shell/src/builtins/shim_doctor/report.rs`
    - `crates/shell/src/execution/platform/mod.rs`

Note for downstream steps:
- CI-checkpoint planning MUST prefer this slice list when populating the machine-readable slices list in `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md` (do not validate mechanically until slice tasks exist in `tasks.json`).
- Workstream triage MAY propose edits to this slice skeleton as recommendations in `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/workstream_triage.md` (it MUST NOT edit this file).
