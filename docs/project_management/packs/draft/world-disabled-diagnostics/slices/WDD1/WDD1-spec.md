# WDD1-spec — `substrate shim doctor`: disabled/skipped statuses (text + JSON)

## Behavior delta (single)
- Existing: `substrate shim doctor` gathers world backend diagnostics and world-deps “applied” state even when effective `world.enabled=false`, and `--no-world` only partially suppresses probing while still representing skipped world-deps as an error.
- New: when effective `world.enabled=false`, shim doctor:
  - reports **disabled** / **skipped (world disabled)** in text output (copy per DR-0003 / `contract.md`),
  - emits explicit status enums in JSON per DR-0001 / `world-disabled-diagnostics-json-schema-spec.md`, and
  - performs **no probes** for diagnostics purposes and omits legacy error/details fields per DR-0002 / schema spec.
  When effective `world.enabled=true`, shim doctor remains fail-visible (backend breakage is “needs attention” with actionable error detail; no masking).
- Why: eliminate misleading diagnostics for intentional host-only installs while preserving real failure visibility when the world is enabled.

## Scope
- Gate shim-doctor world/world-deps collection on `effective_world_enabled` (from WDD0’s shared resolver seam).
- Implement the disabled short-circuit (“no probes”) boundary inside shim-doctor report generation.
- Add additive JSON fields and omission rules per `world-disabled-diagnostics-json-schema-spec.md`:
  - `.world.status`
  - `.world_deps.status`
- Update shim-doctor text output to branch on status enums and to satisfy the deterministic disabled/skipped copy contract (DR-0003).
- Update/extend integration tests in `crates/shell/tests/shim_doctor.rs` to cover disabled vs enabled behavior, including omission rules.

Likely touch surfaces (non-authoritative):
- `crates/shell/src/builtins/shim_doctor/report.rs`
- `crates/shell/src/builtins/shim_doctor/output.rs`
- `crates/shell/tests/shim_doctor.rs`

## Inputs (authoritative)
- Operator copy + “no probes” invariant: `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` (Behavior — world disabled)
- JSON field paths/enums and omission rules: `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`
- DR selections: `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md` (DR-0001/2/3)
- Effective `world.enabled` resolution seam: `slices/WDD0/WDD0-spec.md`

## Behavior (authoritative)

### Disabled short-circuit boundary (“no probes”)
When `effective_world_enabled=false`, shim doctor MUST NOT:
- spawn `substrate world doctor --json` (or any world-doctor subprocess),
- probe the world backend via world-agent socket calls for diagnostics,
- compute world-deps “applied” state (probe-backed package presence checks).

### Text output (disabled mode)
- Text output MUST match the exact required disabled/skipped lines in `contract.md` for `substrate shim doctor` (DR-0003).
- Disabled/skipped MUST NOT be represented as an error:
  - no `Error:` line for disabled/skipped world backend state
  - no `Error:` line for skipped-disabled world deps state

### JSON output (status enums + omission rules)
- When the `.world` object is present, `.world.status` MUST be present and follow the schema spec (DR-0001).
- When the `.world_deps` object is present, `.world_deps.status` MUST be present and follow the schema spec (DR-0001).
- Disabled-mode emission/absence MUST match `world-disabled-diagnostics-json-schema-spec.md` / DR-0002 (disabled/skipped is non-error; omit legacy error/details/report fields).
- Enabled-mode MUST NOT emit the disabled/skipped enum values.

## Acceptance criteria
- AC-WDD1-01: With `effective_world_enabled=false` (e.g., `SUBSTRATE_OVERRIDE_WORLD=disabled` and no `--world`), `substrate shim doctor` text output includes the required disabled/skipped copy from `contract.md` and does not include any `Error:` line for world/world-deps disabled/skipped states.
- AC-WDD1-02: In the same disabled case, `substrate shim doctor --json` emits:
  - `.world.status="disabled"` and `.world_deps.status="skipped_disabled"`, and
  - omits all legacy world/world-deps error/details/report fields required by DR-0002 / the schema spec.
- AC-WDD1-03: Disabled-mode “no probes” is enforced: even if `$SUBSTRATE_HOME/health/world_doctor.json` and `world_deps.json` fixtures exist (and would otherwise report errors/missing deps), disabled mode output remains disabled/skipped and the JSON omits `.world.details` and `.world_deps.report`.
- AC-WDD1-04: With `--world` (forcing `effective_world_enabled=true`), shim doctor preserves existing probe-based behavior:
  - world backend renders healthy/needs-attention as before,
  - `.world.status` is `healthy` or `needs_attention` (not `disabled`),
  - `.world_deps.status` is `ok` or `error` (not `skipped_disabled`),
  - legacy fields (like `.world.details` / `.world_deps.report`) remain present when available.
- AC-WDD1-05: Enabled-but-broken remains fail-visible: with `--world` and a failing backend probe (fixture or real), `.world.status="needs_attention"` and actionable error context is preserved (no masking as disabled/skipped).

## Out of scope
- `substrate health` summary/text behavior and operator docs updates (handled in WDD2).
- Any “why disabled” attribution in outputs (reserved for ADR-0037 follow-up work).

