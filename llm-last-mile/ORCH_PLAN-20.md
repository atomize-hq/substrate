# ORCH_PLAN-20: Execute PLAN-20 Through Exact Prompt-Taking Verbs, Backend-Aware Turn Resolution, And A Helper-Owned Stream Bridge

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-20.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-20.md)  
Style reference: [ORCH_PLAN-19.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-19.md)  
Structure reference: M26 orchestration example provided in chat  
Packet index: [README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Execution type: fresh orchestration controller for prompt-taking public agent surface, Linux-first retained follow-up, parent-frozen contract, parent-only integration and approval  
Live root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`  
Worktree root: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-20`  
Worker model: `GPT-5.4` with `reasoning_effort=high`  
Max concurrent code workers before integration: `3`

## Summary

This document is the execution controller for `PLAN-20`, not a restatement of it.

The run lands one honest non-interactive public caller surface under `substrate agent` with:

- `start` as `new session + first real prompt`
- `turn` as exact follow-up prompt submission for an existing `(orchestration_session_id, backend_id)`
- `reattach` as the canonical lifecycle recovery verb
- helper-owned streaming for prompt-taking calls
- authoritative completion-time `session_posture`
- Linux-first world follow-up through the existing retained-member seam
- explicit fail-closed behavior for non-Linux world-sensitive follow-up
- unchanged `substrate -c`

True code concurrency is capped at `3` because there are exactly three disjoint implementation seams before closeout:

1. public CLI grammar and command-handler surface
2. backend-aware exact turn resolver and posture classification
3. shared prompt-submit extraction plus helper-owned bridge

A fourth code lane would immediately collide with these surfaces or reopen tests/docs too early.

Frozen run shape:

1. `task/m20-p1-parent-contract-freeze-run-init-and-api-scaffold`
2. `task/m20-g1-implementation-launch-gate`
3. parallel Window A
   - `task/m20-l1-public-cli-grammar-and-canonical-verb-surface`
   - `task/m20-l2-backend-aware-turn-resolver-and-posture`
   - `task/m20-l3-shared-prompt-submit-and-helper-stream-bridge`
4. `task/m20-g2-window-a-integration-gate`
5. `task/m20-p2-parent-window-a-integration`
6. `task/m20-g3-closeout-launch-gate`
7. `task/m20-l4-tests-docs-gap-matrix-and-qa-closeout`
8. `task/m20-g4-validation-wall-gate`
9. `task/m20-p3-parent-validation-wall-and-closeout`

## Hard Guards

1. The authoritative integration checkout remains `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/session-centric-state-store`.
2. The parent agent is the only integrator, the only approval authority, and the only writer of `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-20/**`.
3. Public prompt-taking grammar is frozen to exactly:
   - `substrate agent start --backend <backend_id> (--prompt <text> | --prompt-file <path> | --prompt-file -) [--json]`
   - `substrate agent turn --session <orchestration_session_id> --backend <backend_id> (--prompt <text> | --prompt-file <path> | --prompt-file -) [--json]`
   - `substrate agent reattach --session <orchestration_session_id> [--json]`
   - `substrate agent fork --session <orchestration_session_id> [--json]`
   - `substrate agent stop --session <orchestration_session_id> [--json]`
4. `resume` is never the canonical public term. If retained, it is a hidden deprecated alias only. All docs, help, human output, and JSON `action` values say `reattach`.
5. `start` and `turn` require exactly one prompt source. Missing, malformed, unreadable, invalid UTF-8, or effectively empty prompt input is a hard pre-launch failure.
6. No public prompt-taking path may inject or fall back to synthetic `runtime_bootstrap_prompt` text as public user input.
7. `substrate -c`, `--command`, pipe mode, and plain stdin remain shell-wrap semantics. This run must not reinterpret them as agent prompting.
8. Every public `turn` names both exact `orchestration_session_id` and exact `backend_id`. No fuzzy routing, no latest-session routing, no agent-id routing.
9. No public prompt-taking or recovery surface accepts `participant_id`, `active_session_handle_id`, `session_handle_id`, or `internal.uaa_session_id` as input.
10. Noncanonical handles remain explicit rejection paths. No worker may soften that contract.
11. Public `start` and `turn` stream real output while the submitted turn is running. No full-output buffering before completion.
12. `session_posture` is authoritative only at command completion time and is frozen to `active`, `detached_reattachable`, or `terminal`.
13. Host and world follow-up paths stay distinct. Linux world follow-up continues through the retained-member seam. Non-Linux world-sensitive follow-up fails closed with `unsupported_platform_or_posture`.
14. Root `start` remains exact-backend host-first only. World-only root start is an explicit rejection path in this slice.
15. `fork` and `stop` stay lifecycle surfaces, not prompt-taking surfaces.
16. No docs may claim non-Linux parity for world-sensitive follow-up or claim world-root public start shipped.
17. No worker may widen into daemon work, default-agent routing, toolbox mutation, macOS parity claims, or Windows parity claims.
18. No worker may edit `.runs/**`.
19. If the frozen NDJSON envelope, prompt-source contract, `session_posture`, or selector rules become disputed during implementation, the run stops and the parent writes `blocked.json`.

## Fresh Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-20`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-20/public-cli-grammar-and-canonical-verb-surface`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-20/backend-aware-turn-resolver-and-posture`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-20/shared-prompt-submit-and-helper-stream-bridge`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-20/tests-docs-gap-matrix-and-qa-closeout`

Worker branches:

- `codex/feat-session-centric-state-store-m20-public-cli-grammar-and-canonical-verb-surface`
- `codex/feat-session-centric-state-store-m20-backend-aware-turn-resolver-and-posture`
- `codex/feat-session-centric-state-store-m20-shared-prompt-submit-and-helper-stream-bridge`
- `codex/feat-session-centric-state-store-m20-tests-docs-gap-matrix-and-qa-closeout`

Worktree creation commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-20

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-20/public-cli-grammar-and-canonical-verb-surface \
  -b codex/feat-session-centric-state-store-m20-public-cli-grammar-and-canonical-verb-surface \
  feat/session-centric-state-store

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-20/backend-aware-turn-resolver-and-posture \
  -b codex/feat-session-centric-state-store-m20-backend-aware-turn-resolver-and-posture \
  feat/session-centric-state-store

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-20/shared-prompt-submit-and-helper-stream-bridge \
  -b codex/feat-session-centric-state-store-m20-shared-prompt-submit-and-helper-stream-bridge \
  feat/session-centric-state-store

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-20/tests-docs-gap-matrix-and-qa-closeout \
  -b codex/feat-session-centric-state-store-m20-tests-docs-gap-matrix-and-qa-closeout \
  feat/session-centric-state-store
```

Parent integration surface:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/session-centric-state-store`
- no separate parent integration worktree

## Parent-Owned Run-State Surface

Canonical parent-owned state under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-20/`:

- `run-state.json`
- `tasks.json`
- `session-log.md`
- `contract-freeze.json`
- `api-scaffold-freeze.json`
- `lane-ownership.json`
- `merge-order.json`
- `validation-wall.md`
- `blocked.json` on stop only
- `quarantine/`
- `sentinels/`

Canonical per-task roots:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m20-p1-parent-contract-freeze-run-init-and-api-scaffold/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m20-g1-implementation-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m20-l1-public-cli-grammar-and-canonical-verb-surface/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m20-l2-backend-aware-turn-resolver-and-posture/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m20-l3-shared-prompt-submit-and-helper-stream-bridge/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m20-g2-window-a-integration-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m20-p2-parent-window-a-integration/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m20-g3-closeout-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m20-l4-tests-docs-gap-matrix-and-qa-closeout/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m20-g4-validation-wall-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m20-p3-parent-validation-wall-and-closeout/`

Each task root must contain:

- `task.json`
- `commands.txt`
- `summary.md`

Each gate task also contains:

- `gate-checklist.md`
- `gate-result.json`

Each worker task also contains:

- `worker-report.md`
- `worker-output.patch`
- `evidence-manifest.json`

Sentinel convention:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-20/sentinels/<task-id>.ok`

`contract-freeze.json` is the single source of truth for:

- exact public verbs
- prompt-source rules
- canonical `reattach` naming
- NDJSON envelope
- human terminal summary fields
- `session_posture` meanings
- failure taxonomy
- Linux-first fail-closed rule
- `-c` non-regression rule
- doc truth boundaries
- stop conditions

`api-scaffold-freeze.json` is the single source of truth for:

- frozen shared types and function signatures the parent may scaffold before worker launch
- ownership of `resolve_public_turn_target(...)`
- ownership of prompt-source loader and stream-bridge entrypoints
- ownership of extracted shared submit helpers
- ownership of public handler entrypoints in `agents_cmd.rs`

## Task Ledger

| Task ID | Owner | Depends on | Worktree / branch | Deliverable |
| --- | --- | --- | --- | --- |
| `task/m20-p1-parent-contract-freeze-run-init-and-api-scaffold` | parent | — | authoritative checkout | frozen contract, run artifacts, narrow signature scaffold |
| `task/m20-g1-implementation-launch-gate` | parent | `p1` | authoritative checkout | launch approval for Window A |
| `task/m20-l1-public-cli-grammar-and-canonical-verb-surface` | worker | `g1` | `public-cli-grammar-and-canonical-verb-surface` / `codex/feat-session-centric-state-store-m20-public-cli-grammar-and-canonical-verb-surface` | public args, canonical verbs, JSON/human output wiring over frozen seams |
| `task/m20-l2-backend-aware-turn-resolver-and-posture` | worker | `g1` | `backend-aware-turn-resolver-and-posture` / `codex/feat-session-centric-state-store-m20-backend-aware-turn-resolver-and-posture` | exact `(session, backend)` resolver plus posture classification inputs |
| `task/m20-l3-shared-prompt-submit-and-helper-stream-bridge` | worker | `g1` | `shared-prompt-submit-and-helper-stream-bridge` / `codex/feat-session-centric-state-store-m20-shared-prompt-submit-and-helper-stream-bridge` | shared prompt-source loader, extracted submit seam, helper-owned stream bridge |
| `task/m20-g2-window-a-integration-gate` | parent | `l1`, `l2`, `l3` | authoritative checkout | acceptance or quarantine decision for Window A |
| `task/m20-p2-parent-window-a-integration` | parent | `g2` | authoritative checkout | merged code truth for CLI, resolver, and bridge |
| `task/m20-g3-closeout-launch-gate` | parent | `p2` | authoritative checkout | launch approval for tests/docs closeout |
| `task/m20-l4-tests-docs-gap-matrix-and-qa-closeout` | worker | `g3` | `tests-docs-gap-matrix-and-qa-closeout` / `codex/feat-session-centric-state-store-m20-tests-docs-gap-matrix-and-qa-closeout` | integration tests, REPL non-regression checks, gap matrix and packet index updates, QA artifact |
| `task/m20-g4-validation-wall-gate` | parent | `l4` | authoritative checkout | permission to run final wall |
| `task/m20-p3-parent-validation-wall-and-closeout` | parent | `g4` | authoritative checkout | final validation, operator checks, terminal run state |

## Merge Order

`merge-order.json` is frozen during `p1` and governs integration.

1. The parent may take one narrow compile-first scaffold before worker launch in:
   - `crates/shell/src/execution/agent_runtime/control.rs`
   - `crates/shell/src/execution/agent_runtime/mod.rs`
   - `crates/shell/src/execution/agent_runtime/state_store.rs`
2. The scaffold is signature-first only. It may freeze types and function names, not semantics.
3. `L2` integrates before `L3` acceptance is finalized if `L3` consumed target-resolution types from the scaffold.
4. `L3` integrates before `L1` acceptance is finalized if `L1` consumed bridge or prompt-source entrypoints from landed runtime truth.
5. `L1` is replayed on top of the accepted `L2 + L3` tree before final acceptance if handler assumptions drifted from landed runtime truth.
6. `L4` starts only after the merged `p2` tree is green.
7. The parent never hand-merges contradictory `session_posture`, failure-code, selector, or stream-envelope semantics out of worker-local assumptions.

Quarantine rules:

- Quarantine `L1` if it touches `async_repl.rs`, state-store files, or invents runtime semantics.
- Quarantine `L2` if it touches CLI, docs, or tests, or broadens handle acceptance.
- Quarantine `L3` if it touches CLI parse surfaces, docs, or tests, duplicates REPL submit logic, or buffers full output.
- Quarantine `L4` if it reopens any production Rust file.
- Quarantine any lane that invents non-Linux parity claims or world-root public start claims.

## Launch/Integration Gates

### `task/m20-p1-parent-contract-freeze-run-init-and-api-scaffold`

Parent only.

Scope:

1. Create `.runs/plan-20` and every per-task root.
2. Write `tasks.json`, `run-state.json`, `session-log.md`, `contract-freeze.json`, `api-scaffold-freeze.json`, `lane-ownership.json`, `merge-order.json`, and `validation-wall.md`.
3. Freeze the exact NDJSON envelope:
   - `accepted`
   - `event`
   - `warning`
   - `completed`
   - `failed`
4. Freeze operator-facing error codes from `PLAN-20`.
5. Freeze `session_posture` meanings and canonical `reattach` naming.
6. If needed, add compile-first placeholder signatures for:
   - prompt-source loader
   - public start/turn entrypoints
   - exact backend-aware turn resolver
   - shared stream sink / bridge entrypoints
   - shared host/world prompt-submit helpers

Acceptance:

- all freeze artifacts exist
- any scaffold is compile-clean and signature-only
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-20/sentinels/task-m20-p1-parent-contract-freeze-run-init-and-api-scaffold.ok` exists

### `task/m20-g1-implementation-launch-gate`

Parent only.

Checks:

1. `p1` is accepted.
2. All three worktrees are seeded from the exact same post-`p1` tree.
3. Worker prompts name only owned files, frozen contracts, command gates, retry budget, and sentinel paths.
4. `L1` prompt explicitly forbids touching `async_repl.rs`, `state_store.rs`, `session.rs`, and `orchestration_session.rs`.
5. `L2` prompt explicitly forbids touching `cli.rs`, `agents_cmd.rs`, docs, and tests.
6. `L3` prompt explicitly forbids touching `cli.rs`, docs, and tests and records that it owns the shared helper bridge and extraction seam.

Acceptance:

- no worker starts before this gate is green
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-20/sentinels/task-m20-g1-implementation-launch-gate.ok` exists

### `task/m20-g2-window-a-integration-gate`

Parent only.

Checks:

1. `L1`, `L2`, and `L3` each returned patch, report, command transcript, and evidence manifest.
2. Every touched file stays within lane ownership.
3. `L2` preserved exact selector rules and canonical internal/public handle split.
4. `L3` preserved the existing REPL host/world submit behavior while extracting shared helpers.
5. `L3` preserved Linux-first world follow-up and explicit non-Linux fail-closed behavior.
6. `L1` kept `-c` unchanged and emitted canonical `reattach`.
7. If `L1` assumed APIs that differ from accepted `L2 + L3`, the parent replays `L1` after those lanes integrate or quarantines `L1`.

Acceptance:

- accepted, rejected, or quarantined status is recorded for all three lanes
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-20/sentinels/task-m20-g2-window-a-integration-gate.ok` exists

### `task/m20-p2-parent-window-a-integration`

Parent only.

Integration sequence and commands:

1. Integrate accepted `L2` first.

```bash
git cherry-pick <accepted-l2-commit>
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
```

2. Integrate accepted `L3` second on top of `L2`.

```bash
git cherry-pick <accepted-l3-commit>
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

3. Compare frozen scaffold signatures versus `L1` assumptions.
   - If no drift, integrate `L1`.
   - If drift exists, replay `L1` worktree on top of the accepted `L2 + L3` tree first, regenerate its patch, then integrate or quarantine.

4. Integrate accepted or replayed `L1` last.

```bash
git cherry-pick <accepted-or-replayed-l1-commit>
cargo fmt --all -- --check
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
```

5. Run merged Window A proof wall.

```bash
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo clippy --workspace --all-targets -- -D warnings
```

Operational rules:

- The parent remains the only integrator.
- No hand-edited hybrid semantics for `session_posture`, selectors, or NDJSON fields.
- If `L1` needs replay due to signature drift, replay happens before any final `L1` acceptance marker is written.
- If replay cannot be completed cleanly, quarantine `L1` and stop the run.

Acceptance:

- the authoritative tree now contains exact public grammar, exact backend-aware turn resolution, shared prompt-submit truth, helper-owned bridge, canonical `reattach`, and unchanged `-c`
- no hybrid or hand-invented semantics were introduced
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-20/sentinels/task-m20-p2-parent-window-a-integration.ok` exists

### `task/m20-g3-closeout-launch-gate`

Parent only.

Checks:

1. `p2` is green.
2. The closeout worktree is seeded from exact post-`p2` truth.
3. The worker prompt names only allowed test and doc files.
4. The worker prompt explicitly forbids reopening production runtime files.

Acceptance:

- no closeout worker starts before this gate is green
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-20/sentinels/task-m20-g3-closeout-launch-gate.ok` exists

### `task/m20-g4-validation-wall-gate`

Parent only.

Checks:

1. `L4` returned and is classified.
2. `L4` is accepted.
3. No quarantined or blocked output remains unresolved.
4. `validation-wall.md` names exact final command order and manual spot checks.
5. The parent can map every `PLAN-20` done clause to a command, test, or artifact.

Acceptance:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-20/sentinels/task-m20-g4-validation-wall-gate.ok` exists

## Context-Control Rules

1. Workers receive only the contract fragment, owned-file list, forbidden surfaces, and current accepted-tree assumptions they need.
2. The parent does all cross-lane reasoning, merge-order decisions, and quarantine decisions.
3. No worker prompt may say “use best judgment” for selector semantics, `session_posture`, prompt-source rules, or NDJSON shape.
4. `L1` owns only:
   - `crates/shell/src/execution/cli.rs`
   - `crates/shell/src/execution/agents_cmd.rs`
5. `L1` must not touch:
   - `crates/shell/src/repl/async_repl.rs`
   - `crates/shell/src/execution/agent_runtime/state_store.rs`
   - `crates/shell/src/execution/agent_runtime/session.rs`
   - `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
   - docs
   - tests
6. `L2` owns only:
   - `crates/shell/src/execution/agent_runtime/state_store.rs`
   - `crates/shell/src/execution/agent_runtime/session.rs` if needed for posture helpers
   - `crates/shell/src/execution/agent_runtime/orchestration_session.rs` if needed for authoritative metadata comments or narrow support
7. `L2` must not touch:
   - `crates/shell/src/execution/cli.rs`
   - `crates/shell/src/execution/agents_cmd.rs`
   - `crates/shell/src/repl/async_repl.rs`
   - docs
   - tests
8. `L3` owns only:
   - `crates/shell/src/execution/agent_runtime/control.rs`
   - `crates/shell/src/execution/agent_runtime/mod.rs`
   - `crates/shell/src/repl/async_repl.rs`
9. `L3` owns:
   - shared prompt-source loader
   - extracted shared host/world prompt-submit seam
   - helper-owned stream bridge
   - bridge-friendly event sink surface for public `start` and `turn`
10. `L3` must not touch:
   - `crates/shell/src/execution/cli.rs`
   - `crates/shell/src/execution/agents_cmd.rs`
   - docs
   - tests
11. `L4` owns only:
   - `crates/shell/tests/agent_public_control_surface_v1.rs`
   - `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
   - `crates/shell/tests/repl_world_first_routing_v1.rs`
   - `AGENT_ORCHESTRATION_GAP_MATRIX.md`
   - `llm-last-mile/README.md`
12. `L4` must not reopen any production code.
13. `L4` also writes the QA artifact to:
   - `~/.gstack/projects/<slug>/<user>-feat-session-centric-state-store-eng-review-test-plan-<timestamp>.md`
14. Workers may not edit `.runs/**`, create new execution planes, or claim unsupported platform parity.
15. If a lane hits a blocked dependency on another lane’s unfrozen semantics, it stops and requests parent clarification instead of coding around the gap.

## Tests And Acceptance

### Lane command gates

`L1`:

```bash
cargo fmt --all -- --check
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
```

`L2`:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
```

`L3`:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

`L4`:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
rg -n "start|turn|reattach|session_posture|Linux-first|-c|world-root" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md
```

### Window A acceptance requirements

`L1` must prove:

- exact one-of prompt-source parse rules for `start` and `turn`
- canonical `reattach` public surface
- canonical `action` strings in JSON
- no public reinterpretation of `-c`

`L2` must prove:

- exact `(orchestration_session_id, backend_id)` resolution
- `backend_not_in_session` and `ambiguous_backend_slot` rejection
- live vs detached vs terminal posture inputs
- non-canonical handle rejection remains strict
- non-Linux world-sensitive follow-up remains fail-closed

`L3` must prove:

- one shared prompt-source loader
- no synthetic bootstrap prompt enters public input
- extracted host/world submit helpers keep REPL behavior green
- helper-owned stream bridge emits incremental output
- JSON mode emits only NDJSON after stream start
- post-stream bridge failures surface as terminal in-stream `failed`

### L4 closeout acceptance

`L4` must prove:

- `agent_public_control_surface_v1.rs` directly covers `start`, `turn`, `reattach`, `fork`, and `stop` for the shipped public surface
- at least one streaming assertion exists for incremental prompt-taking output
- `session_posture` outcomes are asserted explicitly
- noncanonical handle rejection remains covered
- `substrate -c` non-regression remains covered
- REPL world-first routing non-regression remains green
- `AGENT_ORCHESTRATION_GAP_MATRIX.md` says only what shipped:
  - public non-interactive caller surface landed
  - Linux-first world follow-up posture
  - non-Linux fail-closed posture
  - no claim of world-root public start
  - no claim of macOS or Windows parity
- `llm-last-mile/README.md` is updated so `PLAN-20` and `ORCH_PLAN-20` are discoverable
- the QA artifact exists and includes:
  - root `start` happy path with `--prompt`
  - root `start` stdin path with `--prompt-file -`
  - exact host `turn`
  - exact Linux world `turn`
  - detached recoverable `turn`
  - `reattach`
  - wrong-handle rejection
  - `-c` spot check

### Final validation wall

Run in this exact order on `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

If world-sensitive follow-up logic changed materially, then run:

```bash
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
```

Operator-surface checks after test green:

```bash
substrate agent doctor --json
substrate shim doctor --json
substrate world doctor --json
substrate health --json
```

Manual spot checks after operator-surface checks:

```bash
substrate agent start --backend <host_backend_id> --prompt "hello" --json
printf 'hello from stdin\n' | substrate agent start --backend <host_backend_id> --prompt-file - --json
substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt "next" --json
substrate agent reattach --session <orchestration_session_id> --json
substrate -c "echo hi"
```

Manual check expectations:

- `start` emits `accepted` before completion and ends with `completed` carrying `session_posture`
- stdin prompt is consumed once and only once
- `turn` targets the exact `(session, backend)` pair and does not guess
- `reattach` is canonical in output even if a hidden `resume` alias still exists
- `-c` remains shell-wrap behavior with no agent prompt interpretation

### Stop conditions and quarantine / retry rules

1. Stop the run if `-c` semantics change or become ambiguous.
2. Stop the run if canonical `reattach` naming is not representable without public `resume` drift.
3. Stop the run if exact `(session, backend)` targeting cannot be implemented without fuzzy fallback.
4. Stop the run if the extracted shared submit seam regresses REPL host/world routing.
5. Stop the run if non-Linux world follow-up stops failing closed.
6. Stop the run if any prompt-taking path injects `runtime_bootstrap_prompt`.
7. Quarantine any lane that touches files outside ownership, invents undocumented JSON fields, accepts noncanonical handles, or buffers full output before completion.
8. Retry budget is one targeted retry per quarantined worker lane after the parent writes explicit correction notes into that task root.
9. If the same lane fails twice on contract fidelity, the parent closes the run as blocked instead of launching a third attempt.

## Assumptions

1. The existing retained follow-up machinery in `async_repl.rs` and `world-agent` is the source of truth and is reused rather than rewritten.
2. A narrow signature-first scaffold is enough to let the three code lanes proceed in parallel without speculative API drift.
3. `agent_public_control_surface_v1.rs`, `agent_successor_contract_ahcsitc0.rs`, and `repl_world_first_routing_v1.rs` remain the right primary proof surfaces for this slice.
4. The public `start` implementation may remain host-first for root session creation while still allowing Linux world follow-up through retained-member routing.
5. Any temporary hidden `resume` alias is compatibility-only and does not change the documented surface or emitted `action` values.
6. The parent can truthfully classify `session_posture` at command completion without promising future reachability.
7. Repo-truth docs update only after merged code and tests are green. The gap matrix must not overclaim world-root start, macOS parity, or Windows parity.
