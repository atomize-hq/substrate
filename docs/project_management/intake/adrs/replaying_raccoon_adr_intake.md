---
codename: replaying_raccoon
created: "2026-02-20T01:32:05Z"
status: brainstorming
depends_on:
  - clarifying_owl
execution_order: 90
adr: ADR-0038
adr_path: docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md
workstream_id: WS-replaying_raccoon
lockdown_prompt: docs/project_management/system/prompts/discovery/adr_lockdown.md
---

# ADR Intake Sheet

## 1. Codename + date + status

- Codename: `replaying_raccoon`
- Created: 2026-02-20T01:32:05Z
- Status: brainstorming
- ADR draft: `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`

## 2. Working Title (tentative)

Apply “world disabled reason attribution” to replay warnings and other world-adjacent UX.

## 3. Problem / Motivation (3–6 bullets)

- After `clarifying_owl` lands, doctor/health output will accurately attribute *why* world is disabled (CLI flag vs env override vs config).
- Replay and other world-adjacent flows can still emit generic or misleading messaging that implies `--no-world` (or implies “broken backend”) when the actual cause is config/env.
- In practice, users hit these messages during “install with `--no-world`, enable later” workflows, and the mismatch increases confusion/support time.

## 4. Proposed Outcome (1–3 bullets)

- Replay warnings (and other selected world-adjacent messaging) reuse the same disable-attribution logic as doctor/health and report the *highest-precedence* disable reason.

## 5. Non-Goals (explicit)

- Changing replay behavior (host vs world execution selection), only messaging.
- Reworking the config precedence model (reuse existing semantics).
- Expanding doctor/health attribution scope (this ADR is downstream UX).

## 6. Constraints / Invariants (security, UX, compatibility, performance)

- Must not leak sensitive env values (only key names + config file paths).
- Must remain stable for tooling (prefer enum-like values, avoid fragile strings).
- Must not add noisy output in successful/healthy paths.

## 7. Interfaces / Contracts (CLI/config/API/files/events)

- Replay user-facing warnings (stderr) include:
  - `world_disable_reason` summary (text), and
  - consistent phrasing with doctor/health.
- If replay has JSON output surfaces (now or future), reuse the same enum values as `clarifying_owl`.

## 8. Options (at least 2)

### Option 1 — Central “disable attribution” helper reused by doctor/health/replay

**Description**
Refactor the attribution logic into a shared helper so replay calls the same classifier as doctor/health.

**Pros/Cons**
- Pros: consistent behavior and strings; single place to test precedence.
- Cons: might require plumbing additional context into replay paths.

**Risk notes**
- Ensure no accidental dependency cycles between modules.

### Option 2 — Replay-specific heuristic duplication

**Description**
Implement attribution locally in replay code paths using the same heuristics, without sharing code.

**Pros/Cons**
- Pros: faster to ship; minimal refactor risk.
- Cons: drift risk (doctor/health and replay diverge).

**Risk notes**
- Misattribution is worse than generic messaging; requires strong test coverage.

## 9. Recommendation (tentative) + “Choose Option X when…”

Tentative: **Option 1**.

Choose Option 1 when we want stable, shared semantics and can afford small refactors.
Choose Option 2 only as a temporary bridge if refactor scope blocks shipping.

## 10. Slice Decomposition (required)

- ADR Candidate A (this one): replay warnings attribute world-disabled reason.
  - Slice 1: Identify replay warning surfaces that mention world disabled/unavailable.
  - Slice 2: Add attribution (highest-precedence reason) with consistent wording.
  - Slice 3: Add/adjust tests to prevent regressions (env/config/flag cases).

## 11. Acceptance Criteria Draft (<= 8 items)

- With `world.enabled: false` in config, replay warnings attribute disablement to config (not `--no-world`).
- With CLI `--no-world`, replay warnings attribute disablement to the flag.
- With `SUBSTRATE_OVERRIDE_WORLD=disabled`, replay warnings attribute disablement to the env override.
- No change in replay routing behavior; only messaging changes.

## 12. Dependencies

- depends_on_adrs: [`clarifying_owl`]
- depends_on_work_items: []
- blocks: []

## 13. Lift Summary

### Lift Vector v1

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "touch": {
    "create_files": null,
    "edit_files": 2,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 2,
    "boundary_crossings": null
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": null, "new_test_cases": null },
  "docs": { "new_docs_files": 0 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": true,
    "security_sensitive": false,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": null
  },
  "notes": "Discovery estimate; replay/warnings messaging alignment with doctor/health disable attribution."
}
```
<!-- PM_LIFT_VECTOR:END -->

### Computed outputs (from `make pm-lift-intake`)

```text
Lift Score (v1): 14
Estimated slices: 2
Confidence: low
Triggers:
- missing_inputs:qa.new_test_cases
- missing_inputs:qa.new_test_files
- missing_inputs:risk.unknowns_high
- missing_inputs:touch.boundary_crossings
- missing_inputs:touch.create_files
Missing inputs:
- qa.new_test_cases
- qa.new_test_files
- risk.unknowns_high
- touch.boundary_crossings
- touch.create_files
```

## 14. Open Questions / Unknowns (with priority)

- P0: Which replay messages are in scope for attribution (only “world disabled” warnings vs broader world-backend-unavailable messages)?
- P1: Should replay emit the same JSON fields as doctor/health when `--json` is used elsewhere (future-proofing)?

## 15. Ready-to-lockdown checklist (yes/no with reasons)

- [ ] Scope of replay messaging surfaces is enumerated.
- [ ] Shared helper vs local duplication is chosen.
- [ ] Test coverage plan is explicit.
