# C1 consumer integration and differential review

Terminal subject fingerprint:
`sha256:e0a6dc627760a38ec77367bd25cf4b1dc7f431cbe972e7696445bea3fa1d34e8`

A fresh independent read-only gpt-5.4 Extra High reviewer using standard/default speed performed
the required delta-focused closure review and authored no subject byte. The terminal focus was the
real supervisor-to-dispatcher-to-ledger corridor, compatibility exclusions, allowlist integrity,
and the final broad-wall differential and evidence set.

## Subject binding

Exact fingerprint command:

```bash
git diff --cached --binary -- \
  crates/shell/src/execution/agent_runtime/obligation_ledger.rs \
  crates/shell/src/execution/agent_runtime/state_store.rs \
  crates/shell/src/execution/agent_runtime/world_work_execution_supervisor.rs \
  crates/shell/src/execution/orchestrator_world_dispatch.rs \
  llm-last-mile/runtime-refactor/00-README.md \
  llm-last-mile/runtime-refactor/03-phase-slice-map.md \
  llm-last-mile/runtime-refactor/05-debug-regression-ledger.md \
  llm-last-mile/runtime-refactor/review-control/c1-differential-evidence.json | sha256sum | cut -d' ' -f1
```

Ordered subject paths:

1. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. `crates/shell/src/execution/agent_runtime/world_work_execution_supervisor.rs`
4. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
5. `llm-last-mile/runtime-refactor/00-README.md`
6. `llm-last-mile/runtime-refactor/03-phase-slice-map.md`
7. `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
8. `llm-last-mile/runtime-refactor/review-control/c1-differential-evidence.json`

## Recorded causal lineage

The discovery subject was
`sha256:edbbd444cb2281a6378fcd503ef4b6f5b6a9e6ffaa46bb0dac59973e37ae55bb`.
Its accepted regression and closeout findings were:

- `C1-P4-001`: `RG-OBL-02` closeout wording overstated compatibility inbox durability authority;
  the packet still preserved compatibility pending-count/session-posture projections, but durable
  inbox item materialization remained legacy-only and could not be described as C1 truth.

The closure subject is
`sha256:e0a6dc627760a38ec77367bd25cf4b1dc7f431cbe972e7696445bea3fa1d34e8`.
The required delta-focused closure review returned `CLEAN` and confirmed:

- the session-wide cursor false-conflict fix is present via same-session plan rebasing, and the
  targeted rebase test passes;
- mismatched terminal-cut snapshot requests now fail closed instead of returning `Pending`, and the
  targeted snapshot test passes;
- immutable materialization drift is rejected on the exact-snapshot and compatibility fast paths;
- replay and repair remain no-op and single-writer on the accepted retained path;
- the closeout evidence set is internally consistent at `1330 discovered / 1282 passed / 48 failed
  / 0 ignored`, with exactly six new passing tests and exactly 23 location-only retained-failure
  moves.

## Terminal consumer and differential state

The terminal subject preserves the accepted supervisor/dispatcher integration corridor:

- `crates/shell/src/execution/orchestrator_world_dispatch.rs` reconciles C1 only after the exact
  durable retained typed `Event` or terminal `Exit` is committed, using the validated B3.1 worker
  envelope supplied by the existing supervisor journal truth.
- The old terminal-coupled production writer remains absent from the accepted retained C1 path and
  survives only on the non-C1 `WorkerContinueForkCommand` compatibility branch.
- `llm-last-mile/runtime-refactor/review-control/c1-differential-evidence.json` matches the final
  remediated wall artifacts: both `make shell-lib-wall` and `make shell-lib-wall-serial` record
  `1330 / 1282 / 48 / 0`, failure-name SHA-256
  `c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9`, normalized-signature
  SHA-256 `e53ffb35dbd4fe32ea60ad8da88efe449edc510a5bd0b3fe446e8a368d5beb40`, and zero semantic
  drift outside 23 FILE:LINE-only retained failures in
  `crates/shell/src/execution/orchestrator_world_dispatch.rs`.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
