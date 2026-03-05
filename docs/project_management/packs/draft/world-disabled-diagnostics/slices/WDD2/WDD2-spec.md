# WDD2-spec — `substrate health`: disabled/skipped summary + docs alignment

## Behavior delta (single)
- Existing: `substrate health` derives its summary and human output from shim-doctor probe results; when effective `world.enabled=false`, it can still report “needs attention” / “unavailable” and print enabled-world remediation guidance, producing false-negative “attention required” signals for host-only-by-choice installs.
- New: when effective `world.enabled=false`, `substrate health`:
  - prints deterministic disabled/skipped copy (DR-0003 / `contract.md`),
  - treats disabled/skipped as a **non-error** in the summary (world probe skipped; no world/world-deps failures),
  - emits JSON summary fields per `world-disabled-diagnostics-json-schema-spec.md` (including `summary.world_ok=null` and omission of legacy error fields),
  - suppresses enabled-world world-deps remediation hints.
  When effective `world.enabled=true`, health remains fail-visible (backend breakage stays “needs attention” with error detail; no masking).
- Why: make `substrate health` a trustworthy operator signal in host-only-by-choice workflows without hiding real backend failures when enabled.

## Scope
- Update health summary derivation to follow the status enums and omission rules introduced by WDD1:
  - `.shim.world.status`
  - `.shim.world_deps.status`
- Update `substrate health` human output to:
  - render the deterministic disabled/skipped copy, and
  - suppress enabled-world remediation guidance when disabled.
- Update operator docs (`docs/USAGE.md`) to describe the new status enums as the canonical machine-readable contract (DR-0001) and to align examples with shipped behavior.
- Update/extend integration tests in `crates/shell/tests/shim_health.rs` to cover disabled vs enabled behavior.

Likely touch surfaces (non-authoritative):
- `crates/shell/src/builtins/health.rs`
- `crates/shell/tests/shim_health.rs`
- `docs/USAGE.md`

## Inputs (authoritative)
- Operator copy + suppression rules: `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` (DR-0003)
- Health JSON emission/absence rules: `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`
- Status enum spellings + field paths: DR-0001 (`decision_register.md`) + WDD1

## Behavior (authoritative)

### Disabled mode (`effective_world_enabled=false`)
- Text output MUST include the exact required disabled/skipped lines for `substrate health` from `contract.md` (DR-0003).
- Text output MUST NOT print enabled-world world-deps remediation guidance when disabled (for example, it MUST NOT print lines containing ``substrate world deps current``).
- JSON summary MUST follow `world-disabled-diagnostics-json-schema-spec.md`:
  - `summary.world_ok` is `null` (no probe performed),
  - `summary.world_error` is omitted,
  - `summary.world_deps_error` is omitted,
  - `summary.world_deps_missing` is `[]`,
  - `summary.world_deps_blocked` is `[]`,
  - `summary.failures` MUST NOT include world-backend/world-deps probe failures solely due to the disabled short-circuit.

### Enabled mode (`effective_world_enabled=true`)
- Health MUST remain fail-visible: backend breakage and world-deps applied probing errors/missing/blocked deps remain visible and can contribute to an “attention required” summary.
- Disabled/skipped enum values MUST NOT appear when enabled.

## Acceptance criteria
- AC-WDD2-01: With `effective_world_enabled=false`, `substrate health` text output includes the required disabled/skipped copy from `contract.md` and does not include any enabled-world world-deps remediation hints (no ``substrate world deps current`` lines).
- AC-WDD2-02: In the same disabled case, `substrate health --json` emits:
  - `summary.world_ok=null`,
  - omits `summary.world_error` and `summary.world_deps_error`,
  - emits empty arrays for `summary.world_deps_missing` and `summary.world_deps_blocked`,
  and the embedded shim payload contains `.world.status="disabled"` and `.world_deps.status="skipped_disabled"`.
- AC-WDD2-03: With `--world` (forcing enabled) and fixtures indicating backend failure and/or missing applied deps, health remains fail-visible:
  - `summary.world_ok=false`,
  - `summary.ok=false` and/or “Overall status: attention required” in text output,
  - enabled-world remediation guidance remains present when missing/blocked deps exist.
- AC-WDD2-04: Docs alignment: `docs/USAGE.md` describes `.world.status` / `.world_deps.status` enums as the canonical machine-readable contract and includes an example or explanation of the disabled/skipped behavior consistent with `contract.md` + the schema spec.

## Out of scope
- Shim doctor report generation, text rendering, and JSON emission (handled in WDD1).
- Smoke scripts, manual playbook, and tasks/checkpoint wiring (owned by other PWS in this pack).

