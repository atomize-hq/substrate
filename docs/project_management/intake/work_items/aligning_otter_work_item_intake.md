---
codename: aligning_otter
created: "2026-04-02T15:49:04Z"
status: draft
depends_on: []
---

# Work Item Intake Sheet

## 1. Codename + date + status

- Codename: `aligning_otter`
- Created: 2026-04-02T15:49:04Z
- Status: draft

## 2. Title (imperative)

Align replay network routing with the current world netfilter posture.

## 3. Why not ADR

- This is parity and maintenance work, not a new architectural fork.
- The current shell and world-agent contract already defines when network isolation is requested: effective `world.net.filter` plus canonical restrictive `policy_snapshot.net_allowed`.
- The gap is that replay currently bypasses that contract in two places instead of consuming it consistently.
- If the team wants replay to intentionally diverge from normal world execution semantics, that would warrant an ADR. This WI assumes the desired direction is parity with the shipped contract.

## 4. Task definition (bounded)

- Make replay derive and honor world network routing using the same effective-policy and config semantics as the main shell/world-agent path.
- Eliminate the current replay drift where:
  - local world-backend replay hardcodes `isolate_network: true`
  - local world-backend replay uses `substrate_broker::allowed_domains()` directly
  - agent-backed replay sends `world_network: None`
  - replay-generated `PolicySnapshotV3` omits the effective `net_allowed` contract
- Ensure replay treats the four canonical routing states the same way as normal execution:
  - gate off + restrictive `net_allowed` => no requested isolation
  - gate on + allow-all `["*"]` => no requested isolation
  - gate on + deny-all `[]` => requested isolation with empty allowlist
  - gate on + restrictive allowlist => requested isolation with canonical domains
- Prefer a minimal shared helper extraction over duplicating routing logic inside replay.
- Update replay-facing docs so the documented behavior matches the shipped routing semantics after the implementation lands.

## 5. Done means (<= 8 outcomes)

- Replay no longer hardcodes `isolate_network: true` on the local world-backend path.
- Replay no longer derives network allowlists from raw broker `allowed_domains()` when constructing world routing.
- Agent-backed replay sends a canonical `policy_snapshot` with effective `net_allowed` plus a matching `world_network` payload when isolation is requested.
- Replay and normal shell execution agree on the same four-case routing matrix for `world.net.filter` and `net_allowed`.
- Replay tests pin the parity contract so future routing drift is caught automatically.
- Replay docs describe the current routing behavior accurately and no longer imply stale or backend-divergent semantics.
- No new public CLI/config surface is introduced.

## 6. Likely touch paths

- `crates/replay/src/replay/executor.rs`
  - Replace hardcoded replay routing/request construction with contract-aligned policy snapshot + `world_network` derivation for both local and agent-backed paths.
- `crates/shell/src/execution/policy_snapshot.rs`
  - Likely source of a small shared extraction unless replay duplicates logic, which it should avoid.
- `crates/replay/tests/`
  - Add parity coverage for the four routing cases, or extend an existing replay routing suite if one already exists.
- `docs/REPLAY.md`
  - Update replay behavior notes to match the post-fix contract.
- `docs/TRACE.md`
  - Update only if replay strategy/logging fields change materially as part of the implementation.

## 7. Dependencies (ADR/WI)

- depends_on_adrs: []
- depends_on_work_items: []
- blocks: []

## 8. Lift Summary

### Lift Vector v1

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 1,
    "edit_files": 4,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 3,
    "boundary_crossings": 3
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": {
    "new_test_files": 1,
    "new_test_cases": 4
  },
  "docs": {
    "new_docs_files": 0
  },
  "ops": {
    "new_smoke_steps": 0,
    "ci_changes": 0
  },
  "risk": {
    "cross_platform": false,
    "security_sensitive": true,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": 1
  },
  "notes": "Estimate assumes a bounded replay parity fix touching replay request construction, shared routing helper extraction, replay tests, and replay documentation."
}
```
<!-- PM_LIFT_VECTOR:END -->

### Computed outputs

```text
Lift Score (v1): 48
Estimated slices: 4
Confidence: high
Triggers:
- likely_split:crates_touched>2
- likely_split:lift_score>24
- split_required:estimated_slices>3
```

## 9. Open questions

- Should replay import the existing shell helper as-is via a small visibility/shared-module refactor, or should the routing logic move to a more neutral shared home?
  - Recommended default: extract the minimal shared helper and keep replay thin.
- Should replay preserve any intentional distinction between local fallback netns behavior and agent/world-agent routing, or should the request-construction contract be fully unified even if the underlying fallback mechanisms still differ?
  - Recommended default: unify request construction and policy derivation; runtime fallback mechanics can remain implementation-specific.
- Do we want replay-specific doctor or trace assertions as part of this work, or is routing parity coverage enough for the first pass?
  - Recommended default: routing parity tests first, observability expansion only if the implementation naturally touches those surfaces.
