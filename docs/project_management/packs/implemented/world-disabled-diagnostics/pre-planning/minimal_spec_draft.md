**PRE‑PLANNING ONLY — this draft is non-executable and MUST be deleted/retired during full planning.**

# world-disabled-diagnostics — minimal spec draft (pre-planning)

## Scope + authority

This document defines **pack-level alignment** only:
- cross-cutting defaults and precedence rules,
- cross-slice invariants (failure posture, output posture, compatibility posture),
- seam constraints that multiple specs MUST align on.

This document does **not** define:
- slice-level acceptance criteria,
- detailed JSON schemas / field lists,
- implementation tasks or test plans.

Authoritative inputs for this draft:
- ADR: `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
- Spec manifest: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md`
- Impact map: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md`
- External authoritative contracts (feature MUST NOT redefine):
  - Effective-config precedence + `SUBSTRATE_OVERRIDE_*` semantics: `docs/reference/env/contract.md`
  - Configuration reference (paths, related settings): `docs/CONFIGURATION.md`
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

Full-planning source-of-truth target (not created in pre-planning):
- `contract.md` per `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`

## Defaults + precedence

### Effective-config precedence (authoritative)

Effective config precedence is authoritative in `docs/reference/env/contract.md` and MUST NOT be redefined in feature-local docs.

When an enabled workspace exists:
1. CLI flags (when a flag exists): `--world` / `--no-world`
2. Workspace config patch: `<workspace_root>/.substrate/workspace.yaml`
3. Global config patch: `$SUBSTRATE_HOME/config.yaml` (default `~/.substrate/config.yaml`)
4. Defaults (built in)

In this case, `SUBSTRATE_OVERRIDE_*` MUST be ignored for effective config resolution.

When no enabled workspace exists:
1. CLI flags (when a flag exists): `--world` / `--no-world`
2. Override env inputs: `SUBSTRATE_OVERRIDE_*` (incl `SUBSTRATE_OVERRIDE_WORLD`)
3. Global config patch: `$SUBSTRATE_HOME/config.yaml` (default `~/.substrate/config.yaml`)
4. Defaults (built in)

### Definitions

- “World disabled” is defined by the effective `world.enabled=false` after precedence resolution (not by socket presence/absence).
- Diagnostics in scope consult the same effective-config resolver (see ADR-0036 references to `crates/shell/src/execution/config_model.rs`).
- This feature introduces **no new** config keys and **no new** environment variables (ADR-0036 + spec manifest).

## Failure posture + invariants

### Degrade posture (world disabled)

When effective `world.enabled=false`, diagnostics MUST:
- emit explicit **disabled** / **skipped (world disabled)** statuses in text and JSON output (final enum spellings per DR-0001),
- skip world-backend probes and world-deps “applied” probing,
- avoid implying world health or world-deps “applied” state.

### Fail-visible posture (world enabled)

When effective `world.enabled=true`, diagnostics MUST:
- continue to surface backend unavailability and probe failures as “needs attention” with actionable error details,
- avoid masking failures under “disabled/skipped” classifications.

### Config-resolution failure invariant

Diagnostics MUST NOT silently misclassify disabled vs enabled on effective-config resolution failure (invalid YAML, unreadable config). Full planning MUST define one deterministic behavior (error vs explicit unknown) and align tests to it.

### Security / redaction posture

- Telemetry/log schema changes are out of scope for this feature (spec manifest “MUST NOT change” category); no new fields are introduced.
- New disabled/skipped signals are status enums; they MUST NOT encode secrets or sensitive values.

## Exit-code posture

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This feature requires **no new exit codes**.
- ADR-0036 default posture: `substrate health` and `substrate shim doctor` remain informational surfaces where **successful report generation exits `0`**, independent of “needs attention” classification; non-zero exit is reserved for command execution failures (invalid flags, serialization failures, config-resolution errors).

## Cross-cutting seams / constraints

- Commands in scope (only): `substrate health`, `substrate shim doctor` (text + `--json`).
- JSON compatibility: additive-only; new disabled/skipped signals are machine-detectable via explicit status enums and MUST NOT be encoded only as error strings.
- Operator copy determinism: disabled/skipped copy and remediation hints MUST be deterministic and consistent across both commands; disabled-mode guidance points to enabling/provisioning (e.g., `substrate world enable`) and suppresses world-deps remediation text.
- Platform parity: Linux/macOS/Windows expose the same disabled/skipped semantics (text + JSON).
- Cross-queue boundary (impact map):
  - WDD owns **status** (`disabled` / `skipped_disabled`) for diagnostics.
  - ADR-0037 / attribution work owns **attribution** (“why disabled”) and must layer without changing WDD semantics.
  - `json-mode` work must preserve WDD’s additive fields inside any envelope and must not rename/remove them without an explicit compat posture.

## Follow-ups for full planning

1) Decide DR-0001: final JSON field paths + enum spellings for world/world-deps statuses for both `health --json` and `shim doctor --json`.
2) Decide DR-0002: deterministic legacy JSON error-field behavior when disabled/skipped applies (omit vs null vs still populated).
3) Decide DR-0003: deterministic operator-facing copy contract (exact templates, or exact required substrings + ordering rules) for disabled/skipped across both commands.
4) Define “skip probes” operational boundary (singular + testable): list forbidden operations when disabled and how tests assert “no probes”.
5) Pin behavior for effective-config resolution errors (invalid YAML, unreadable config) and the corresponding exit-code mapping for `health`/`shim doctor`.
6) Verify current exit-code behavior for “needs attention” vs hard failures and reconcile ADR-0036 assumptions in `contract.md`.
7) Update operator docs (`docs/USAGE.md`) to match shipped disabled/skipped semantics and remediation hints (impact map touch set).
8) Add sequencing entry and dependency edges for WDD in `docs/project_management/packs/sequencing.json` (impact map follow-up; at minimum WDD before ADR-0037 attribution work).

## Draft slice skeleton (pre-planning only)

Draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): `WDD`

Downstream notes:
- CI-checkpoint planning uses this slice list as the default when populating the machine-readable slice list in `pre-planning/ci_checkpoint_plan.md` (do not validate mechanically until slice tasks exist in `tasks.json`).
- Workstream triage may propose edits to this slice skeleton as recommendations in `pre-planning/workstream_triage.md` (without editing this file).

- slice_id: `WDD0`
  name: Make diagnostics world-disabled aware
  intent: Make `substrate health` and `substrate shim doctor` treat effective `world.enabled=false` as disabled/skipped (non-error) and skip probes; preserve fail-visible behavior when enabled-but-broken.
  likely touch surfaces:
    - `crates/shell/src/builtins/shim_doctor/report.rs`
    - `crates/shell/src/builtins/shim_doctor/output.rs`
    - `crates/shell/src/builtins/health.rs`
    - `crates/shell/tests/shim_doctor.rs`
    - `crates/shell/tests/shim_health.rs`
    - `docs/USAGE.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/WDD0-spec.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/manual_testing_playbook.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/linux-smoke.sh`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/macos-smoke.sh`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/windows-smoke.ps1`
