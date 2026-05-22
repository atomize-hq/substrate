---
codename: quieting_lemur
created: "2026-02-20T01:32:05Z"
status: brainstorming
depends_on: []
execution_order: 70
adr: ADR-0036
adr_path: docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md
workstream_id: WS-quieting_lemur
lockdown_prompt: docs/project_management/system/prompts/discovery/adr_lockdown.md
---

# ADR Intake Sheet

## 1. Codename + date + status

- Codename: `quieting_lemur`
- Created: 2026-02-20T01:32:05Z
- Status: brainstorming
- ADR draft: `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`

## 2. Working Title (tentative)

Treat “world disabled” as a first-class status in `substrate health` / `substrate shim doctor` (skip world-deps probing when disabled).

## 3. Problem / Motivation

- When `world.enabled: false` in `$SUBSTRATE_HOME/config.yaml` (e.g., after installs with `--no-world`), Substrate is intentionally host-only.
- Today, `substrate health` can still report “World backend: needs attention” and “World deps: unavailable …” because the shim-doctor report attempts to compute world deps “applied” state even though the world is disabled.
- This produces misleading errors (e.g., “world-service readiness probe failed”) that look like a broken setup rather than an intentionally disabled backend.
- The noisy/incorrect diagnostics make it harder to tell whether there’s a real provisioning issue vs. an intentional configuration choice.
- Note: world-deps install state is designed to persist across world sessions (stable prefix `/var/lib/substrate/world-deps` bind-mounted into worlds), so treating “disabled” as a non-error should not imply that deps were lost—just that Substrate isn’t probing them right now.

## 4. Proposed Outcome

- If the effective config resolves to `world_enabled=false`, then:
  - `substrate health` reports world as **disabled** (not “needs attention”), and
  - world-deps “applied” probing is **skipped** with a clear “skipped because world is disabled” status (non-error).
  - JSON output surfaces a stable status enum for world/world-deps (additive fields).

## 5. Non-Goals

- Changing `substrate world enable` provisioning behavior.
- Changing world-deps inventory semantics or adding new inventory items.
- Reworking the world-service readiness probe logic.
- Changing enforcement/routing behavior for normal command execution.

## 6. Constraints / Invariants

- UX: world-disabled should be quiet and explicit, but must not hide genuine failures when `world_enabled=true`.
- Compatibility: do not break existing JSON schemas without a version bump or additive fields.
- Observability: “skipped because disabled” should still be visible in JSON/text output.

## 7. Interfaces / Contracts (concrete changes)

- `substrate shim doctor` (report generation) should derive effective `world_enabled` and use it to:
  - avoid calling world-deps “applied” computation when disabled, and
  - avoid treating world-disabled as a failed health check.
- `substrate health` summary should treat `world_enabled=false` as a non-failure state and print actionable guidance:
  - “World backend: disabled (run `substrate world enable` to provision)”.
- JSON contract (additive):
  - report an explicit world status enum including `disabled`, and
  - report a world-deps status enum indicating “skipped because disabled” (not an error).
- Skip semantics:
  - When `world_enabled=false`, always skip world-deps “applied” probing (no escape hatch).

## 8. Options

### Option 1 — Shim doctor consults effective config and skips world-deps snapshot when disabled

**Description**
Teach `crates/shell/src/builtins/shim_doctor/report.rs` to resolve the effective config (or minimally the world-enabled bit) and pass that into `gather_world_deps_section`, returning a “skipped/disabled” section instead of running the snapshot.

**Pros**
- Fixes the root cause where the “applied” computation triggers world backend probes.
- Keeps the behavior centralized in shim-doctor, so `health` improves automatically.

**Cons**
- Requires a reliable “effective world-enabled” signal inside the shim-doctor code path.

**Risk notes**
- Must ensure we don’t introduce cycles/side effects from config resolution inside doctor paths.

### Option 2 — `health` post-processes shim-doctor output and reclassifies disabled as non-failure

**Description**
Keep shim-doctor behavior mostly intact, but in `crates/shell/src/builtins/health.rs` detect that `world_enabled=false` (by inspecting `world_doctor.details.world_enabled`) and change the printed summary + failures list accordingly. Optionally also suppress world-deps errors when disabled.

**Pros**
- Smaller code surface; minimal movement in doctor collection.

**Cons**
- Leaves shim-doctor JSON/text still misleading unless separately updated.
- Harder to keep consistent across `health`, `shim doctor`, and future tooling.

**Risk notes**
- Can mask real world-deps failures if we mis-detect disabled state.

## 9. Recommendation (tentative) + “Choose Option X when…”

Chosen: **Option 1**.

Choose **Option 1** when we want consistent behavior across `health` and `shim doctor` and want to avoid triggering backend probes when disabled.

Choose **Option 2** when we need the smallest immediate improvement and can accept shim-doctor remaining noisy/inconsistent until a follow-up.

## 10. Slice Decomposition (required)

- ADR Candidate A (this one): “World disabled” suppresses world-deps probing in `shim doctor`/`health`.
  - Slice 1: Add a reliable disabled signal to shim-doctor (effective config or parsed doctor JSON), and emit “skipped because disabled”.
  - Slice 2: Adjust `health` summary/failures so `world_enabled=false` is not “attention required”.
- Candidate B (follow-up if needed): Extend the same “disabled short-circuit” semantics to `substrate world deps …` subcommands (early error with guidance, or explicit `--force` behavior).

## 11. Acceptance Criteria Draft (observable outcomes)

- With `~/.substrate/config.yaml` containing `world.enabled: false`, `substrate health` does not report “world backend health check failed”.
- With `world.enabled: false`, `substrate health` prints a world summary of **disabled**.
- With `world.enabled: false`, `substrate health` does not report “World deps: unavailable (…)” due solely to world backend probes.
- `substrate shim doctor --json` includes an explicit status enum/field indicating that world-deps status was skipped/disabled (non-error).
- When `world.enabled: true` and the socket/agent is broken, `substrate health` still reports “needs attention” (no regression).

## 12. Dependencies

- depends_on_adrs: []
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
    "crates_touched": 1,
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
  "notes": "Discovery estimate; doctor/health status plumbing + skip world-deps probing when world is disabled."
}
```
<!-- PM_LIFT_VECTOR:END -->

### Computed outputs (from `make pm-lift-intake`)

```text
Lift Score (v1): 10
Estimated slices: 1
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

- P0: What are the exact enum values and field names for world + world-deps status (to keep JSON stable and additive)?
  - Proposed answer: reuse `world.status` from `substrate world doctor --json` (`ok|missing_prereqs|unreachable|…`) and add `disabled` when `world_enabled=false`. For world-deps, add an explicit status enum with `skipped_disabled` (and `ok|error`).
- P1: Should world-deps inventory inspection (non-probing views like `enabled`) still work when world is disabled?
  - Proposed answer: yes; only the “applied”/probe-backed surfaces are skipped.
- P2: Should the status enum be reused across `host doctor`, `world doctor`, `shim doctor`, and `health`, or introduced only in shim/health first?
  - Proposed answer: reuse across surfaces; implement shim-doctor + health first, then align host/world doctor JSON as a follow-up if needed.

## 15. Ready-to-lockdown checklist (yes/no with reasons)

- [ ] Behavior delta is locked (skip world-deps probing when disabled).
- [ ] Output contract decided (text + JSON semantics).
- [ ] Backward-compat plan for JSON (additive fields vs schema bump).
