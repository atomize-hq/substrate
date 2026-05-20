<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-session-centric-state-store-autoplan-restore-20260501-155804.md -->

# PLAN-09: Live-State Authority and Compatibility Cutover

Source file: [09-live-state-authority-and-compatibility-cutover.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/09-live-state-authority-and-compatibility-cutover.md)  
Branch: `feat/session-centric-state-store`  
Plan type: shell/runtime authority-cutover contract, no UI scope, strong DX scope  
Review posture: `/autoplan`-style scope tightening with `/plan-eng-review` structure and rigor  
Status: execution-ready after [PLAN-08.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-08.md), with outside voice skipped on 2026-05-01 because `claude` CLI auth is missing

## Objective

This slice is not about inventing a new live registry.

The repo already has the right building blocks:

- canonical session-root records under
  [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- fail-closed operator surfaces in
  [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- one runtime writer choke point in
  [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- bounded contract tests in
  [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- operator wording in
  [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
  and
  [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

What is still too soft is the cutover contract. A future diff can still regress this area by:

1. treating flat compatibility files as current truth again,
2. letting trace history authorize current toolbox state,
3. loosening torn-root or ambiguity behavior into "pick the newest one",
4. removing the compatibility bridge before fixtures, docs, and downstream readers are ready,
5. spreading write ownership outside the store.

`PLAN-09` fixes that by freezing one boring rule set:

1. canonical session-root parent and participant records are live-state authority,
2. flat compatibility files are bridge input/output only during cutover,
3. legacy `handles/*.json` stays read-only compatibility input of last resort,
4. trace is historical fallback only for status gaps, never current-session authority,
5. `status` and `toolbox` fail closed on ambiguity, corruption, or broken parent/child linkage,
6. flat bridge retirement gets explicit gates instead of wishful thinking.

This matters because the user-facing experience is one of two things:

- clean: `substrate agent status` and `substrate agent toolbox env` tell the truth about the current live runtime, or
- a mess: the shell shows a stale endpoint because some old trace row or flat file looked newer.

This slice makes the first outcome durable.

## Step 0: Scope Challenge

### 0A. Repo truth and why this slice exists

The SOW is directionally right, and the repo already proves most of it.

What the code already does today:

1. `load_authoritative_session(...)` in
   [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   prefers canonical session-root parents and only falls back to flat parent files when the
   canonical root is absent.
2. `list_sessions()` and `list_live_sessions()` already distinguish discoverable torn roots from
   actually live sessions.
3. `resolve_single_live_session_for_agent(...)` already fails closed on multiple active parents,
   multiple live host orchestrators, missing selected participants, and inactive selected
   participants.
4. `build_status_report(...)` already checks live runtime state before projecting trace fallback.
5. `build_toolbox_status_report(...)` and `build_toolbox_env_report(...)` already resolve through
   store-owned live session discovery and reject trace-only liveness.
6. `persist_runtime_snapshots(...)` in
   [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   already centralizes parent + participant persistence through the store.
7. existing tests already pin the most dangerous edges:
   - canonical over flat precedence
   - torn-root degradation
   - trace not authorizing toolbox
   - tombstone suppression beating stale trace
   - multi-parent ambiguity failing closed

What is still missing is not a new mechanism. It is one locked contract around those mechanisms.

### 0B. Premise challenge

Premise check, one by one:

1. **Canonical session-root objects should be live-state authority.**
   - Accepted.
   - Verified by `load_authoritative_session(...)`, `list_live_sessions()`, and the canonical-over-flat tests.

2. **Flat compatibility files still need to exist during the cutover.**
   - Accepted.
   - Verified by the explicit dual-write tests in
     [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
     and by docs/tests that still mention flat compatibility paths as bridge input.

3. **Trace must stay historical, never current-session authority.**
   - Accepted.
   - Verified by
     `agent_toolbox_env_trace_history_does_not_authorize_active_session`
     and
     `agent_status_prefers_live_manifest_over_trace_fallback_for_selected_orchestrator`.

4. **`toolbox` should fail closed instead of picking the newest live candidate.**
   - Accepted.
   - That is the only sane 3am behavior. Choosing "latest" is a great way to aim an operator at the wrong socket.

5. **This slice should remove the bridge now.**
   - Rejected.
   - The repo is not ready. Tests, docs, and compatibility readers still rely on flat bridge posture. Doing freeze plus removal in one slice is how you ship a migration footgun.

Premise gate posture:

- accepted as-is for this plan,
- with one explicit constraint: this slice freezes the bridge contract and removal gates, it does not remove the bridge.

### 0C. Existing code to reuse

| Sub-problem | Existing code | Plan |
| --- | --- | --- |
| Canonical parent precedence | `load_authoritative_session(...)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse |
| Canonical + flat participant merge | `build_session_record(...)`, `list_sessions()`, canonical + flat participant readers in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse, tighten contract language |
| Live-session discovery | `list_live_sessions()`, `resolve_single_live_session_for_agent(...)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse, add any missing regression guards |
| Tombstone suppression | `list_invalidated_participants_across_sources()` plus fallback suppression helpers in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Reuse |
| Operator projections | `build_status_report(...)`, `build_toolbox_status_report(...)`, `build_toolbox_env_report(...)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Reuse |
| Centralized write ownership | `persist_runtime_snapshots(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse, keep as sole caller choke point |
| Contract-proof tests | [agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs) and store unit tests in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Extend |
| Operator wording | [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md), [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md) | Reuse, tighten if drift remains |

### 0D. Dream state and 12-month ideal

```text
CURRENT REPO
    │
    ├── canonical session-root parent + participant records exist
    ├── flat compatibility files still exist
    ├── legacy handles are still readable
    ├── status/toolbox mostly honor the right precedence
    └── the final cutover contract is spread across code, docs, and tests
            │
            ▼
THIS PLAN
    │
    ├── one explicit authority ladder
    ├── one explicit operator-surface rule set
    ├── one explicit torn-root posture
    ├── one explicit dual-write ownership rule
    └── one explicit bridge-removal gate list
            │
            ▼
12-MONTH IDEAL
    │
    ├── canonical session-root records only
    ├── no flat compatibility parent/participant/lease writes
    ├── no legacy handle reads
    ├── trace purely historical
    └── docs + tests + runtime all say the same thing
```

What changes for the builder:

- today, you still need to triangulate code, tests, and docs to know what is truth,
- after this slice, you read one contract and then implement against it,
- later, bridge retirement becomes a smaller, safer slice instead of a repo archeology expedition.

### 0E. Implementation alternatives

| Approach | Summary | Effort | Risk | Decision |
| --- | --- | --- | --- | --- |
| A. Freeze the contract now, keep the bridge, add explicit retirement gates | Smallest correct slice, matches current repo truth | Medium | Low | **Accepted** |
| B. Remove flat compatibility reads and writes immediately | Looks clean, breaks migration safety and fixtures | Medium | High | Rejected |
| C. Docs-only cleanup | Easy, but leaves runtime drift and caller misuse risk | Small | High | Rejected |
| D. Build a new transactional runtime registry | Fancy, unnecessary, spends an innovation token on the wrong thing | Large | Unacceptable | Rejected |

### 0F. Complexity, search, completeness, and distribution checks

`[Layer 1]` wins.

The runtime already has the right primitives. The correct move is to reuse them and make them
hard to misread. Not to invent a new registry, cache, or selection heuristic.

Minimal production seam:

1. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
2. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
3. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
4. [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
5. [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
6. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
7. store unit tests in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Secondary docs under
[docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/)
should only be touched if they contradict the active runtime contract.

Completeness check:

- the complete version is to freeze the bridge contract, test it, and state removal gates now,
- the shortcut is to say "we know what we mean" and leave it half-encoded across code and tests,
- with AI-assisted implementation, the complete version is the obvious choice.

Distribution check:

- no new binary, package, image, or artifact type is introduced here,
- distribution work is not applicable.

### 0G. What already exists

1. canonical session-root precedence already exists,
2. dual-write to canonical + flat compatibility layouts already exists,
3. torn-root degradation already exists,
4. live runtime beats trace fallback already exists,
5. tombstones already suppress stale trace fallback,
6. toolbox already refuses trace-only liveness,
7. docs already say flat and handle files are compatibility input only,
8. contract tests already grep for some of that wording.

This is good news. The plan is mostly about making that truth legible and harder to regress.

### 0H. NOT in scope

- removing flat compatibility reads in this slice
- removing flat dual-write in this slice
- removing legacy `handles/*.json` reads in this slice
- redesigning the parent session schema
- changing public JSON shape for `substrate agent status` or `substrate agent toolbox ...`
- changing trace schema or adding new trace families
- inventing a new live registry or transaction layer
- reopening [PLAN-08.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-08.md) event-emission authority cleanup
- reopening [PLAN-05.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-05.md) invalidation semantics
- member-runtime launch/lifecycle work from [10-member-runtime-launch-seam.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/10-member-runtime-launch-seam.md)
- UI work

## Architecture Contract

### Live-state authority order

For live operator surfaces, authority is ordered exactly like this:

1. canonical parent session record  
   `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json`
2. canonical participant record  
   `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/participants/<participant_id>.json`
3. flat compatibility parent/participant files only to fill incomplete canonical roots on read
4. legacy `handles/*.json` only as last-resort compatibility input
5. trace fallback only for historical status projection gaps, never current liveness

Hard rules:

1. canonical objects outrank conflicting flat compatibility objects,
2. flat compatibility data can fill a gap, never overrule a present canonical object,
3. legacy handles are never live-state authority,
4. trace never authorizes toolbox or current control-plane liveness,
5. torn roots are discoverable but non-live,
6. ambiguity and corruption fail closed.

### Authority flow diagram

```text
operator surface
    │
    ├── substrate agent status
    └── substrate agent toolbox status|env
            │
            ▼
    AgentRuntimeStateStore
            │
            ├── canonical parent root
            ├── canonical participant root
            ├── flat compatibility gap-fill only
            ├── legacy handle fallback only
            └── trace fallback only for status tuples still uncovered
                    │
                    ├── status: allowed after live-state + tombstone filtering
                    └── toolbox: never allowed
```

### Operator surface contract matrix

| Surface | Current truth source | Allowed fallback | Forbidden fallback | Failure posture |
| --- | --- | --- | --- | --- |
| `substrate agent status` | `list_live_sessions()` projections | trace for tuples not covered by live state or tombstones | trace to override live state | keep concurrent sessions distinct, never collapse to "latest" |
| `substrate agent toolbox status` | `resolve_single_live_session_for_agent(...)` | flat participant when canonical root is parent-only | trace-only liveness | `dependency_unavailable` or fail closed |
| `substrate agent toolbox env` | `resolve_single_live_session_for_agent(...)` | flat participant when canonical root is parent-only | trace-only liveness | exit `3` or fail closed |

### Torn-root contract

```text
canonical parent present, canonical child present
    -> complete, eligible for live discovery if parent active and owner alive

canonical parent present, canonical child absent
    -> incomplete torn root
    -> flat participant fallback allowed during cutover
    -> warnings preserved
    -> never promoted to live without a valid selected participant

canonical parent absent, canonical child present
    -> discoverable torn root
    -> warnings preserved
    -> never promoted to live

legacy handle only
    -> compatibility input only
    -> never treated as primary truth when canonical or flat data exists
```

### Dual-write contract

Dual-write remains intentional during cutover, but only here:

- `persist_orchestration_session(...)`
- `persist_participant(...)`
- `persist_parent_session_snapshot(...)`

inside
[state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs).

Rules:

1. callers write canonical truth through the store,
2. the store may also write flat compatibility copies while the bridge exists,
3. callers do not write flat compatibility copies directly,
4. callers do not reintroduce `handles/*.json` write ownership,
5. flat dual-write removal happens only after explicit bridge-removal gates are green.

## Concrete File Touch Plan

### 1. `crates/shell/src/execution/agent_runtime/state_store.rs`

Keep this file as the live-state owner. Do not split authority anywhere else.

Required work:

- preserve this read order in `load_authoritative_session(...)` and keep it obvious in code:
  canonical parent -> flat parent only if canonical parent is absent -> canonical participant
  -> flat participant only if canonical participant is absent -> legacy handle alias last,
- keep `list_live_sessions()` and `resolve_single_live_session_for_agent(...)` strict about
  parent state, owner PID liveness, selected-participant presence, selected-participant activity,
  and parent/child linkage,
- keep dual-write ownership local to `persist_orchestration_session(...)`,
  `persist_participant(...)`, and `persist_parent_session_snapshot(...)`,
- add or retain direct store-level regressions for:
  - canonical participant beating conflicting legacy-handle fallback,
  - flat participant beating conflicting legacy-handle fallback when the canonical child is absent,
  - selected participant present but inactive failing closed before any operator surface can project it,
- do not introduce a new cache, registry, or helper layer to "clarify" this logic.

### 2. `crates/shell/src/execution/agents_cmd.rs`

Keep operator surfaces strict and boring.

Required work:

- `build_status_report(...)` must keep live-session projections first and must only consult trace
  after live-state projection plus tombstone suppression have already decided which tuples are still
  unresolved,
- `build_toolbox_status_report(...)` must continue resolving through
  `resolve_single_live_session_for_agent(...)` rather than adding local precedence or recovery
  logic,
- `build_toolbox_env_report(...)` must continue to require a real resolved live endpoint and keep
  exit `3` for `dependency_unavailable`,
- parent-only canonical roots may degrade through flat participant fallback during cutover, but
  no path in this file may promote a torn root to live on its own,
- no heuristic "latest session", newest timestamp, or trace-only live-session recovery gets added
  anywhere in this file.

### 3. `crates/shell/src/repl/async_repl.rs`

Keep `persist_runtime_snapshots(...)` as the caller choke point.

Required work:

- runtime code continues writing through `persist_runtime_snapshots(...)`, and that helper
  continues delegating only to `store.persist_orchestration_session(...)` plus
  `store.persist_participant(...)`,
- no direct writes to `sessions/<orchestration_session_id>.json`, `participants/*.json`, flat
  lease files, or `handles/*.json` appear outside the store,
- if a helper extraction happens, it stays mechanical and local, with no new persistence owner and
  no new call graph branch around the store.

### 4. Active operator docs

Primary targets:

- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

Required work:

- keep one clear authority statement: canonical session-root parent + participant records are the
  live-state authority boundary,
- keep one clear toolbox fail-closed statement: `substrate agent toolbox env --json` emits
  variables only for a current live host-scoped orchestrator session and otherwise exits `3`,
- keep one clear trace statement: trace is historical fallback for `status` gaps only, never
  current-session toolbox authorization,
- keep one clear bridge-removal gate statement so a future cleanup diff cannot claim "the docs
  looked ready" while runtime/tests still depend on the bridge,
- make sure docs do not imply that `handles/*.json` or trace history are current truth.

### 5. Secondary docs audit

Audit only if active wording still contradicts runtime truth:

- [compatibility-spec.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md)
- [manual_testing_playbook.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md)

If they already agree, leave them alone. Minimal diff matters. If they disagree, update only the
contradictory sentence or bullet, not the whole pack.

### 6. Regression suites

Primary anchors:

- store unit tests in
  [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- contract tests in
  [agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

Required work:

- keep existing coverage green,
- add only the missing contract edges listed in this plan,
- prefer test names that describe the actual authority rule, not implementation trivia.

## Execution Plan

### 1. Freeze store-owned authority first

Make `state_store.rs` the irreversible source of truth before touching docs or operator wording.

- canonical parent first,
- canonical participant first,
- flat compatibility only for gap fill,
- legacy handle last,
- torn roots discoverable but non-live.

Required output:

- store-level regressions proving both canonical-over-legacy and flat-over-legacy participant
  precedence,
- a store-level regression proving a selected inactive participant fails closed,
- no production behavior change outside `state_store.rs` yet.

Do not start the next step until these targeted tests pass.

### 2. Keep operator surfaces strict

Update `agents_cmd.rs` only after the store contract is frozen. This step is about projection and
error posture, not new authority logic.

- no trace-based current liveness,
- no heuristic newest-session selection,
- no hidden live-state promotion for incomplete roots,
- no ambiguity collapse.

Required output:

- CLI-surface regression for an active parent whose selected participant exists but is inactive,
- `toolbox status|env` continuing to resolve through the store and fail closed rather than
  inventing local recovery,
- `status` continuing to treat trace as additive historical fill only.

### 3. Lock the write choke point

Verify runtime persistence still routes through `persist_runtime_snapshots(...)` and store helpers
only. This is a drift guard, not a new behavior branch.

Required output:

- a bounded ownership proof, ideally a contract/grep-style regression in the shell contract suite,
  that no production caller outside `state_store.rs` writes flat compatibility session,
  participant, lease, or handle artifacts directly,
- if `async_repl.rs` needs edits, they stay mechanical.

### 4. Finish docs while behavior is fresh

Update active docs only where wording can still mislead an operator or future maintainer.

This is not fluff. Docs are part of the contract because this slice is mostly about making truth
hard to misread.

Required output:

- `TRACE.md` and `USAGE.md` describing the same authority ladder as the runtime,
- bridge-removal gates written once in active docs or this plan, not contradicted elsewhere,
- secondary docs touched only if they disagree.

### 5. Retire the bridge later, not now

Write the bridge-removal gates into this plan, but do not execute them here. Bridge retirement is a
separate slice with its own green-light criteria.

1. explicit migration-only tests are the only remaining flat readers,
2. active docs no longer describe flat files as anything but compatibility input,
3. no required off-repo consumer still depends on flat parent/participant files,
4. no caller-owned write path exists outside the store,
5. legacy handle reads have dedicated replacement coverage.

### 6. Validate the slice end-to-end

Do the boring proof before calling this done.

Run:

```bash
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell -- --nocapture
```

Acceptance for this step:

- all new authority-order and fail-closed regressions are green,
- no doc wording contradicts runtime behavior,
- no new files or abstractions were added without a direct contract reason.

## Architecture Review

### Locked architecture decisions

1. **Keep the store as the only live-state owner.**
   - `status` and `toolbox` consume store truth. They do not invent their own precedence logic.

2. **Keep canonical session-root objects as authority.**
   - Flat files exist only because migration is real.

3. **Keep trace historical.**
   - Trace helps explain what happened. It does not decide what is live now.

4. **Keep fail-closed selection.**
   - If there is ambiguity, broken linkage, or corruption, error out. Do not "helpfully" guess.

5. **Keep dual-write localized.**
   - One write choke point is maintainable. Many write owners turn this into a split-brain bug farm.

6. **Keep bridge removal as a later slice.**
   - One migration variable at a time.

### Architecture acceptance gates

1. no live operator surface reads current truth from trace alone,
2. no live operator surface treats `handles/*.json` as primary truth,
3. no caller writes flat compatibility copies outside the store,
4. no incomplete root becomes live,
5. no ambiguous multi-candidate state gets silently collapsed.

### Architecture issues found

1. The authority contract still lives in too many places at once.
   - Fix: keep this plan anchored to store helpers, operator surfaces, and docs only. No new abstraction.

2. Bridge-removal pressure is the main strategic risk.
   - Fix: state explicit gates now, remove later.

3. Legacy handle fallback is still easy to mentally over-credit.
   - Fix: add one more direct regression and keep the docs blunt about last-resort posture.

## Code Quality Review

### Implementation guardrails

1. one live-state owner: `AgentRuntimeStateStore`,
2. one persistence choke point: `persist_runtime_snapshots(...)`,
3. one operator-resolution path for toolbox: `resolve_single_live_session_for_agent(...)`,
4. one live-first then trace-fallback status flow,
5. no duplicated precedence logic in callers,
6. no hidden bridge writes outside the store.

### Minimal-diff rules

- prefer clarifying tests and doc wording over new helper layers,
- keep runtime changes small and local,
- if current code already matches the SOW, do not churn it just to look busy,
- if a comment or tiny assertion can prevent future regression, take it.

### Code quality issues found

1. The bridge contract is easy to regress because precedence lives in behavior, not one explicit local contract.
   - Fix: add direct tests for the remaining edge cases instead of adding a new abstraction.

2. Secondary docs can silently drift even when runtime is correct.
   - Fix: audit only contradictory wording, keep active docs as the primary surface.

## Error & Rescue Registry

| Failure point | What goes wrong | Required rescue |
| --- | --- | --- |
| canonical parent exists, child missing | parent-only torn root is misread as live | keep warnings, allow flat child gap-fill only, never treat root as live without valid selected participant |
| multiple active parents for one orchestrator | operator gets pointed at the wrong session | fail closed with operator-readable error |
| active parent points at missing or inactive participant | toolbox endpoint looks plausible but is wrong | fail closed, no endpoint |
| trace has newer history than live store | stale trace resurrects dead session | keep live-first and tombstone suppression rules |
| caller writes flat compatibility file directly | bridge ownership splits across layers | route all writes through store only |
| docs imply handle files are live truth | human operator debugs the wrong artifact | keep active docs explicit and add grep-backed contract tests where helpful |

## Test Review

This slice is in much better shape than a typical migration seam. Good.

The remaining work is not "add tests everywhere." It is "close the last holes that could let
someone weaken the authority boundary later."

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/execution/agent_runtime/state_store.rs
    │
    ├── [★★★ TESTED] persist_writes_canonical_and_flat_compatibility_layouts
    ├── [★★★ TESTED] list_sessions_discovers_participant_only_roots_and_excludes_them_from_live_results
    ├── [★★★ TESTED] parent_only_torn_roots_degrade_with_warnings_instead_of_failing_discovery
    ├── [★★★ TESTED] list_invalidated_participants_across_sources_includes_legacy_fallback_tombstones
    ├── [GAP]         explicit regression that legacy handle data never outranks canonical or flat participant data
    └── [GAP]         bounded guard that store persistence remains the only dual-write owner

[+] crates/shell/src/execution/agents_cmd.rs
    │
    ├── [★★★ TESTED] agent_toolbox_env_trace_history_does_not_authorize_active_session
    ├── [★★★ TESTED] agent_toolbox_surfaces_prefer_canonical_session_roots_over_flat_compatibility_files
    ├── [★★★ TESTED] agent_toolbox_surfaces_fall_back_to_flat_participant_when_canonical_root_is_incomplete
    ├── [★★★ TESTED] operator_surfaces_fail_closed_when_multiple_active_parent_candidates_exist
    ├── [★★★ TESTED] agent_status_prefers_live_manifest_over_trace_fallback_for_selected_orchestrator
    ├── [★★★ TESTED] agent_status_tombstone_suppression_beats_stale_trace_fallback_for_world_member
    └── [GAP]         explicit operator-surface regression for active parent referencing an inactive selected participant

[+] crates/shell/src/repl/async_repl.rs
    │
    ├── [★★  TESTED] runtime startup/lifecycle paths already persist through store helpers
    └── [GAP]        narrow regression or grep-style proof that no caller-owned flat-file write path reappears outside the store

─────────────────────────────────
COVERAGE: 10/14 paths tested (71%)
QUALITY:  ★★★: 9  ★★: 1  ★: 0
GAPS: 4 paths need direct regression coverage
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] substrate agent toolbox env with trace-only history
    │
    └── [★★★ TESTED] fails closed, exit 3, no live endpoint

[+] substrate agent toolbox status/env with canonical parent + canonical participant
    │
    └── [★★★ TESTED] canonical live session wins over conflicting flat compatibility files

[+] substrate agent toolbox status/env with parent-only canonical root
    │
    └── [★★★ TESTED] flat compatibility participant fallback remains readable during cutover

[+] substrate agent status with live canonical orchestrator session
    │
    └── [★★★ TESTED] live manifest beats trace fallback

[+] substrate agent status --scope world with invalidated stale member
    │
    └── [★★★ TESTED] tombstone suppression beats stale trace fallback

[+] operator surfaces with broken parent linkage
    │
    ├── [★★★ TESTED] multiple active parents fail closed
    ├── [★★★ TESTED] missing active handle id fails closed
    ├── [★★★ TESTED] missing selected participant fails closed
    └── [GAP]         inactive selected participant fail-closed case should be pinned explicitly at the CLI surface

[+] bridge safety
    │
    ├── [★★★ TESTED] canonical + flat dual-write still occurs during cutover
    └── [GAP]         direct proof that legacy handles never retake live authority

─────────────────────────────────
COVERAGE: 8/10 flows tested (80%)
GAPS: 2 user-visible flows and 2 code-owner regressions need direct tests
─────────────────────────────────
```

### Required test additions by file

#### `crates/shell/src/execution/agent_runtime/state_store.rs`

Add direct coverage for exactly these authority rules:

- canonical participant beats conflicting legacy-handle fallback for the same
  `(orchestration_session_id, agent_id)` tuple,
- flat participant beats conflicting legacy-handle fallback when the canonical child is absent but
  the canonical parent exists,
- selected participant present but inactive causes `resolve_single_live_session_for_agent(...)` to
  fail closed with an operator-meaningful error instead of silently degrading to another candidate.

#### `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

Add direct coverage for exactly these operator-facing rules:

- `substrate agent toolbox status --json` fails closed when an active parent points at an existing
  but inactive selected participant,
- `substrate agent toolbox env --json` fails closed with exit `3` for the same scenario,
- contract/doc assertions only if the final wording still leaves room to misread bridge-removal
  posture or live-state authority order.

#### Write-ownership drift guard

Add a narrow proof, preferably in the shell contract suite rather than a new REPL abstraction,
that production code outside `state_store.rs` does not write flat compatibility session,
participant, lease, or handle artifacts directly. The guard may be grep-backed if it is precise and
stable.

### Test commands

Run at minimum:

```bash
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell agent_successor_contract_ahcsitc0 -- --nocapture
```

Then run:

```bash
cargo test -p shell -- --nocapture
```

If docs under `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/`
need edits, keep the existing pack-level contract assertions green as part of the same pass.

### QA artifact

The eng-review QA handoff artifact for this slice is:

[spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260501-155804.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260501-155804.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| canonical + flat + legacy read merge | legacy handle fallback silently outranks canonical or flat data | no | partial | no | yes until direct precedence test lands |
| store-owned bridge writes | future caller writes flat compatibility files directly and creates split write ownership | no | no | no | yes until bounded ownership guard lands |
| operator surface linkage validation | active parent references inactive participant and toolbox resolves a stale endpoint | no | yes in store | partial | yes until CLI-surface regression lands |
| trace fallback overlay | stale trace row resurfaces invalidated member | yes | yes | yes | no |
| parent-only torn root | incomplete root is promoted to live | yes | yes | yes | no |
| trace-only toolbox authorization | `toolbox env` succeeds with no current live session | yes | yes | yes | no |

Critical gap rule:

If a future diff can let legacy handles outrank canonical/flat runtime state, let callers write
flat bridge files directly, or let toolbox resolve an inactive selected participant, the cutover
contract is not actually frozen.

## Performance Review

This is a correctness slice, not a throughput slice.

Performance rules anyway:

1. keep live resolution store-owned and bounded,
2. do not add new scans or caches just to make the contract "feel explicit",
3. keep trace fallback additive and late, not on the hot live path,
4. do not build migration infrastructure to solve a documentation problem.

Performance issues found:

- 0 material performance issues.

The real footgun would be adding a new registry or cache to speed up a contract that the existing
store already resolves correctly. That would be classic software. Two hundred lines to avoid
reading the code you already have.

## DX Review

This slice has no UI scope. It absolutely has DX scope.

The primary user here is the developer or operator who is trying to answer one question:
"What is the live orchestrator session right now, and can I trust the answer?"

### Developer journey map

| Stage | What the developer is doing | Current friction | Target after this slice |
| --- | --- | --- | --- |
| 1 | Launch runtime and expect live state to persist | low | keep low |
| 2 | Run `substrate agent status --json` | medium, contract is spread across code/tests/docs | one clear live-first contract |
| 3 | Run `substrate agent toolbox status --json` | medium | obvious fail-closed semantics |
| 4 | Run `substrate agent toolbox env --json` | medium | clear exit `3` when dependency unavailable |
| 5 | See conflicting flat files or old trace history | high | know instantly they are compatibility/history only |
| 6 | Debug torn-root or ambiguous parent linkage | medium | one failure model, no heuristics |
| 7 | Audit docs for truth | medium-high | active docs say the same thing as tests |
| 8 | Prepare later bridge removal | high today | explicit gate list |
| 9 | Onboard a new maintainer | high today | they can read one plan and one test file instead of spelunking |

### Developer empathy narrative

I ran `substrate agent toolbox env` because I needed the live socket now.

If it fails, I should not have to wonder whether trace history is secretly good enough, or whether
some flat JSON file in `~/.substrate/run/agent-hub/` is more real than the canonical root. I
should get one clear answer: no live host-scoped orchestrator session, or ambiguity, or broken
parent linkage.

Then I run `substrate agent status --json` and I should see the same truth model. Same session.
Same precedence. Same fail-closed posture.

That is the DX win here. Less archaeology. More trust.

### DX Scorecard

| Dimension | Score | Notes |
| --- | --- | --- |
| Getting started | 6/10 | CLI names are good, authority contract still too spread out |
| Naming guessability | 8/10 | `status`, `toolbox status`, `toolbox env` are obvious |
| Error messages | 7/10 | fail-closed errors are mostly good, keep them operator-readable |
| Docs findability | 6/10 | active docs improved, still need one sharper contract story |
| Upgrade path safety | 8/10 | bridge stays until explicit retirement gates pass |
| Observability | 8/10 | trace is strong as history, once its boundary stays clear |
| Recovery guidance | 6/10 | torn-root and ambiguity rules need one clearer narrative |
| Escape hatches | 7/10 | compatibility bridge exists, but should stay obviously temporary |

Overall DX score: **7/10**

### DX Implementation Checklist

- keep one canonical authority statement in `TRACE.md`,
- keep one operator-facing statement in `USAGE.md`,
- preserve explicit error text for missing live session, ambiguity, and broken linkage,
- add missing regression coverage for inactive selected participant and legacy-handle precedence,
- state bridge-removal gates where implementers will actually read them.

### TTHW assessment

Current TTHW for "understand what is live-state truth here" is about **10-12 minutes** for a new
maintainer. They have to read the SOW, the docs, and the tests.

Target after this slice: **under 5 minutes**.

That means:

- read `PLAN-09.md`,
- read the active docs,
- run the two shell test suites.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A. Freeze store-owned authority and add store-level regressions | `crates/shell/src/execution/agent_runtime/` | — |
| B. Tighten operator surfaces and CLI-surface regressions | `crates/shell/src/execution/`, `crates/shell/tests/` | A |
| C. Tighten active docs and secondary doc drift only if needed | `docs/`, `docs/project_management/` | A |
| D. Final validation and cleanup | `crates/shell/`, `docs/` | B, C |

### Parallel lanes

- Lane A: Step A
- Lane B: Step B, after Lane A
- Lane C: Step C, after Lane A, in parallel with Lane B once the authority wording from Lane A is
  stable
- Lane D: Step D, after Lane B and Lane C merge

### Execution order

1. Launch Lane A first and let it own `state_store.rs`.
2. Once Lane A lands or is merged into the worktree base, launch Lane B and Lane C in parallel.
3. Lane B owns runtime/operator code plus `agent_successor_contract_ahcsitc0.rs`.
4. Lane C owns docs only.
5. Launch Lane D after B and C merge to run the full shell test pass and make any mechanical fixups.

### Conflict flags

- `state_store.rs` is Lane A only. Do not let Lane B touch it unless Lane A is already merged.
- `agent_successor_contract_ahcsitc0.rs` is Lane B only. Keep store-level tests in Lane A and
  CLI-surface tests in Lane B to avoid test-file merge fights.
- `TRACE.md` and `USAGE.md` are Lane C only.
- If a secondary doc requires assertion changes in a contract suite, that work moves back to Lane D.

### Parallelization verdict

This slice is sequential at the authority seam, then safely parallel for docs versus operator
surface tightening.

- **4 lanes total**
- **2 lanes can run in parallel after the authority freeze**
- **2 sequential checkpoints remain non-negotiable: store freeze first, full validation last**

## Deferred Work

There is no repo-root `TODOS.md`, so explicit deferrals stay here.

1. remove flat compatibility reads after explicit migration-only coverage exists
2. remove flat dual-write after no required compatibility readers remain
3. remove legacy `handles/*.json` reads last
4. collapse active and secondary compatibility docs further only if later slices still need them
5. any broader runtime-registry redesign, intentionally out of scope here

## Definition of Done

This slice is done only when all of the following are true:

1. canonical session-root parent and participant records are the documented and implemented
   live-state authority,
2. flat parent/participant/lease files are explicitly bridge input/output only during cutover,
3. legacy `handles/*.json` is documented and tested as last-resort compatibility input only,
4. `status` keeps concurrent sessions distinct and uses trace only as bounded fallback,
5. `toolbox status|env` never authorize current liveness from trace,
6. torn roots remain discoverable with warnings but never become live,
7. store-owned persistence remains the only bridge write owner,
8. bridge-removal gates are written down before anyone starts deleting compatibility code.
9. targeted authority tests and the full `cargo test -p shell -- --nocapture` pass are green.

## Completion Summary

- Step 0: scope accepted as contract-freeze slice, bridge removal deferred intentionally
- Architecture Review: 3 issues found, all about authority-boundary hardening rather than new infrastructure
- Code Quality Review: 2 issues found, both drift-prevention problems
- Test Review: diagram produced, 4 direct regression gaps identified
- Performance Review: 0 issues found
- DX Review: 7/10 overall, TTHW 10-12 min to target under 5 min
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0, repo has no root `TODOS.md`, deferrals captured here
- Failure modes: 3 critical gaps flagged
- Outside voice: skipped, `claude` CLI is installed but unauthenticated on 2026-05-01
- Parallelization: 4 lanes, with docs and operator-surface tightening parallel only after the authority freeze, plus a final validation lane
- Lake Score: complete option chosen for every in-slice decision

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope | Treat `PLAN-09` as contract freeze plus removal gates, not bridge removal | Mechanical | Pragmatic | The repo still depends on compatibility bridge behavior in tests and docs | Immediate bridge deletion |
| 2 | Authority | Keep canonical session-root parent + participant records as live-state truth | Mechanical | Explicit over clever | The runtime already encodes this, and it is the only maintainable source of truth | Split authority across trace or flat files |
| 3 | Operator safety | Keep `status` and `toolbox` fail closed on ambiguity or broken linkage | Mechanical | Completeness | Guessing "latest" is worse than erroring for control-plane selection | Heuristic newest-session fallback |
| 4 | Persistence | Keep dual-write ownership inside `AgentRuntimeStateStore` only | Mechanical | DRY | One owner prevents write drift and split-brain migration bugs | Caller-owned flat writes |
| 5 | Trace boundary | Keep trace as historical fallback only | Mechanical | Systems over heroes | Historical telemetry should not be allowed to authorize a live endpoint | Trace-as-live-registry behavior |
| 6 | Docs | Treat active docs as part of the runtime contract | Mechanical | Bias toward action | Future regressions here are more likely to come from misread docs than missing code | Leaving the contract buried only in tests |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/plan-ceo-review` | Scope and strategy | 1 | CLEAR | Accepted the narrow contract-freeze direction, rejected premature bridge removal, and defined the 12-month end state explicitly |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate outside-model review run |
| Eng Review | `/plan-eng-review` | Architecture and tests (required) | 1 | CLEAR | Locked the authority ladder to store-owned canonical records, identified 4 direct regression gaps, and preserved minimal diff around existing runtime seams |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**UNRESOLVED:** 0 plan-level decision points remain. The remaining work is implementation and the four bounded regression additions already listed.

**VERDICT:** CEO + ENG CLEARED. `PLAN-09` is ready to execute as the live-state authority
freeze and compatibility-cutover contract on top of the session-centric store work.
