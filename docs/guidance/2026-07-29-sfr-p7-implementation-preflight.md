# SFR P7 implementation preflight

Date: 2026-07-29

## Authority

- User approval: explicit `/build auto` invocation against the approved P7 specification, plan,
  and task ledger.
- Review-clean planning commit:
  `e372ad81216fefd5b9ce4704398f0985fbbe9e30`.
- Required predecessor:
  `133b55249f88492e16f80f98a63368911d733c7e`.
- Branch: `feat/sfr-p4-path-semantics`.
- Ancestry check:
  `git merge-base --is-ancestor 133b55249f88492e16f80f98a63368911d733c7e e372ad81216fefd5b9ce4704398f0985fbbe9e30`
  returned success.
- Worktree and index were clean at the review-clean planning commit before this approval-state
  update.

## Default P7 path fence

P7 owns only:

- `crates/agent-drift-sentinel/tests/current_native_recall.rs`;
- `crates/agent-drift-sentinel/tests/fixtures/current_native_recall/**`;
- exact existing test-only seams named by the approved specification when required;
- `scripts/dev/drift-batch-scan/**`;
- P7 specification, plan, task, guidance, validation, and review documents; and
- root active-authority routers and task-status updates.

Production Rust, frozen historical fixtures/receipts, dependencies, CI configuration, pushes, and P8
work are not authorized.

## Protected historical walls

| Wall | Frozen inventory | Focused target inventory |
|---|---:|---:|
| semantic-goal-drift acceptance | 18 case directories | 2 tests |
| progress acceptance | 16 case directories | 4 tests |
| delegated acceptance | 10 matrix cases | 2 tests |

The delegated wall stores its ten cases in
`crates/agent-drift-analyzer/tests/fixtures/delegated_acceptance/matrix.json`, not as ten
directories.

## Focused owner inventory

| Test target | Tests |
|---|---:|
| `agent-session-compactor/current_native_adapter` | 7 |
| `agent-session-compactor/bounded_closure` | 10 |
| `agent-session-compactor/export_bundle` | 5 |
| `agent-drift-analyzer/semantic_goal_drift_acceptance` | 2 |
| `agent-drift-analyzer/progress_acceptance` | 4 |
| `agent-drift-analyzer/delegated_acceptance` | 2 |
| `agent-drift-analyzer/input_contract` | 17 |
| `agent-drift-sentinel/real_session_live` | 38 |
| `agent-drift-sentinel/live_event_shape` | 12 |
| `agent-drift-sentinel/replay_input` | 24 |

Inventory was captured with each target's `cargo test ... -- --list` command from the approved
specification. No production or frozen-corpus file changed during preflight.
