# PLAN-25: Durable Host Session Closeout, Inbox Contract Honesty, And QA Hardening

Source SOW: [25-host-durable-session-closeout-and-qa-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)  
Truth anchors: [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md:184), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:79), [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:105), [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md:61)  
Adjacent landed slices: [PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-22.md), [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md), [24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)  
Branch: `feat/host-orchestrator-durable-session`  
Base branch: `main`  
Plan type: closeout pass  
Status: execution-ready planning pass on 2026-05-14

## Objective

Close the durable host-session slice without reopening the product model.

This plan is done only when all of the following are true:

1. The repo states one unambiguous public recovery contract:
   - `turn` is prompt-taking follow-up on the same durable session,
   - `reattach` is attached-owner recovery only,
   - `stop` is durable closeout for attached and parked host sessions,
   - `status --json` reports durable session truth while the session is parked.
2. Parked and `awaiting_attention` host sessions stay visible on `substrate agent status --json` from authoritative session-root truth, not only from attached-live process truth.
3. One command-level lifecycle proof covers the same durable session through parked `status`, parked `turn`, `reattach`, and `stop`.
4. Docs stop overstating inbox reality. They must say what is shipped today and what is not shipped today.
5. Validation is complete enough that a future patch cannot silently regress the operator lifecycle or the contract wording.

This is a closeout slice. It is not a new orchestration design and it is not inbox product expansion.

## Why This Closeout Exists

The branch has already landed most of the runtime behavior that the older SOW still treated as open.

The remaining work is narrower and more important:

1. The code, tests, usage docs, truth doc, gap matrix, and packet README still do not tell exactly the same story.
2. The runtime has real inbox persistence and posture math, but the repo still risks implying a finished operator-facing inbox workflow that the code does not prove.
3. Lifecycle QA is strong but fragmented. The branch still lacks one crisp closeout wall that proves the full operator path on one durable session.
4. `status` is already functionally landed, but it is a high-risk operator read surface. A regression there lies to operators even if the underlying session store is correct.

The work here is boring on purpose. Freeze the contract, prove it end to end, and stop saying more than the code supports.

## Plan Summary

The runtime contract is already chosen on this branch:

1. `substrate agent start --backend <backend_id> --prompt ... --json` is the public root prompt-taking surface.
2. `substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ... --json` is the public parked-session follow-up surface.
3. `substrate agent reattach --session <orchestration_session_id> --json` is the explicit attached-owner recovery surface.
4. `substrate agent stop --session <orchestration_session_id> --json` is the public closeout surface.
5. Parked host sessions are durable and routable. Detached-world follow-up remains fail closed until `reattach` restores an active host owner.

What is not closed yet is the repo-wide explanation and the regression floor around that behavior.

This plan therefore does four things, in order:

1. freeze the already-chosen public contract in every truth surface,
2. add one command-level same-session lifecycle proof plus explicit parked-status assertions,
3. narrow inbox claims to the actual shipped surface,
4. publish one exact validation wall future patches must keep green.

## Scope

In scope:

1. truth-doc convergence across usage docs, truth docs, gap matrix, and packet docs,
2. command-level lifecycle regression coverage for parked host `status`, `turn`, `reattach`, and `stop`,
3. command-level `status --json` assertions for `parked_resumable` and `awaiting_attention`,
4. code-comment tightening where the shipped inbox scope is currently easy to misread,
5. one exact manual validation path plus one exact automated validation wall.

## NOT in scope

This slice does not include:

1. changing the public `turn` / `reattach` / `stop` split,
2. adding default-agent routing or fuzzy selectors,
3. adding a public inbox grammar such as `substrate agent inbox`,
4. adding a production runtime approval/completion/follow-up inbox workflow,
5. widening detached-world recovery beyond the current fail-closed contract,
6. redesigning the durable session model, daemon model, or state store,
7. Windows/WSL product-parity work.

Anything in that list is a separate slice. This plan explicitly does not smuggle it in.

## Locked Starting State

### What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Public parked-session follow-up | [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:505) | Reuse. `turn` is already the public prompt-taking follow-up verb. |
| Public attached-owner recovery | [`run_reattach(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:565) | Reuse. `reattach` already exists and stays public in this slice. |
| Public closeout | [`run_stop(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:667) | Reuse. `stop` remains the canonical closeout path for attached and parked host sessions. |
| Authoritative parked-session read path | [`build_status_report(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1333), [`list_status_sessions_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:630), [`status_visible_participants(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:66) | Reuse and harden. Do not add a second status model. |
| Detached host continuity classification | [`classify_public_session_posture(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2191), [`valid_detached_host_continuity_posture(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2216) | Reuse exactly. This is the public continuity contract. |
| Durable inbox persistence and posture math | [`persist_inbox_item(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1039), [`apply_pending_inbox_count(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1726) | Reuse. Narrow the claims, not the implementation. |
| Dev-support runtime alert ingress | [`persist_runtime_alert_for_dev_support(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs:11) | Reuse and document honestly. Do not present this as a public product surface. |
| Existing lifecycle proof points | [`public_start_turn_and_stop_emit_streaming_ndjson_and_authoritative_state()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1231), [`public_stop_cleanly_closes_same_durable_session_after_reattach()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1631), [`public_start_persists_detached_session_when_hidden_owner_helper_exits()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1880), [`detached_pending_inbox_normalizes_to_awaiting_attention()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1980), [`public_turn_fail_closed_taxonomy_is_explicit_for_missing_backend_unknown_session_and_parent_slot_errors(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:2363), [`public_turn_fail_closed_taxonomy_is_explicit_for_world_linkage_ambiguity_and_detached_rejection()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:2492), [`public_turn_routes_linux_world_member_follow_up_through_typed_submit_path()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:2689) | Reuse. Add a cohesive closeout wall rather than inventing new semantics. |

### Exact remaining gap

The remaining gap is not a missing runtime model. The remaining gap is contract convergence and closeout rigor:

1. Some truth docs still describe `reattach`, `stop`, and parked `status` as unfinished, even though the branch now ships them.
2. Inbox language is ahead of the proven runtime. Persistence exists. Posture math exists. A broad operator-facing inbox contract does not.
3. The branch has targeted lifecycle tests, but it still needs one same-session closeout proof that reads like the actual operator story.
4. The branch has store-level status visibility proof, but the command path still needs an explicit regression floor for parked and `awaiting_attention` rows.

### Blast radius

Even though the remaining work is narrow, the touched seams are high-consequence seams:

1. `run_turn(...)`, `run_reattach(...)`, and `run_stop(...)` are top-level CLI contract surfaces.
2. `build_status_report(...)` is the operator truth surface for durable session visibility.
3. `persist_inbox_item(...)` is where documentation can drift far ahead of product reality because the persistence primitive looks more complete than the operator surface actually is.

Implication: keep production code changes minimal, keep test additions explicit, and keep the docs exact.

## Frozen Execution Contract

If implementation wants a different contract, revise this plan first. Do not drift the behavior silently.

### Public operator contract

1. `substrate agent start --backend <backend_id> --prompt ... --json` remains the canonical public root prompt-taking surface and remains host-only in v1.
2. `substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ... --json` remains the canonical public follow-up surface.
3. `substrate agent reattach --session <orchestration_session_id> --json` remains attached-owner recovery only. It is not a prompt-taking alias.
4. `substrate agent stop --session <orchestration_session_id> --json` remains the durable closeout action for attached and parked host sessions.
5. `substrate agent status --json` remains the authoritative parked-session read surface for live-runtime rows, with `posture`, `attached_participant_id`, and `pending_inbox_count` sourced from durable session-root truth.
6. `parked_resumable` and `awaiting_attention` remain routable durable host postures. `terminal` remains the only non-routable posture family.
7. Detached-world follow-up remains fail closed until `reattach` restores an active host owner.
8. `substrate -c`, `--command`, and piped shell mode remain shell execution surfaces, not agent-prompt aliases.

### Durable inbox contract

This slice freezes the narrow, honest contract:

1. Durable inbox persistence primitives are shipped.
2. `pending_inbox_count` plus posture normalization into `awaiting_attention` are shipped.
3. Internal ack/dismiss support is shipped.
4. Dev-support and test ingress are shipped.
5. A public operator-facing inbox command surface is not shipped.
6. A proven production runtime path for world-originated approval/completion/follow-up inbox items is not shipped as part of the supported public contract.
7. An automatic parked-session resume mechanism driven by inbox items is not shipped.

### Contract diagram

```text
parked host session
    |
    +--> agent status --json
    |      `--> read durable session-root truth
    |
    +--> agent turn --session <sess> --backend <backend>
    |      `--> submit a prompt against the same durable session/backend pair
    |
    +--> agent reattach --session <sess>
    |      `--> restore active_attached host ownership without submitting a prompt
    |
    `--> agent stop --session <sess>
           `--> close the same durable session terminally

detached world member
    |
    `--> agent turn ... while detached
           `--> fail closed with reattach guidance
```

## Step 0: Scope Challenge

### 0A. Minimum honest diff

The minimum honest implementation is:

1. update the truth docs so they stop claiming shipped behavior is unshipped,
2. add one same-session lifecycle regression that covers parked `status`, parked `turn`, `reattach`, and `stop`,
3. add explicit command-level parked-status assertions for both `parked_resumable` and `awaiting_attention`,
4. narrow inbox wording everywhere to the actually shipped surface,
5. publish one exact validation wall and one exact manual smoke path.

Anything smaller leaves contradictions behind. Anything bigger expands scope into a new inbox feature.

### 0B. Existing-code leverage

This slice should build on the already-landed seams, not introduce parallel ones:

1. extend [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs), do not add a new integration harness,
2. keep the authoritative `status` path in `agents_cmd.rs` plus `state_store.rs`,
3. keep posture classification in `classify_public_session_posture(...)`,
4. keep inbox semantics where they are and document them correctly,
5. keep docs aligned to [`docs/USAGE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:105) as the operator contract source.

### 0C. Complexity check

Expected touched areas:

1. `crates/shell/tests/`
2. `crates/shell/src/execution/`
3. `docs/`
4. repo-root truth docs
5. `llm-last-mile/`

That is a multi-file slice, but it is still the right size because:

1. there are no new public verbs,
2. there is no new state model,
3. there is no new daemon or control plane,
4. almost all of the work is truth-sync and regression hardening,
5. any production code change should be tiny and test-driven.

### 0D. Search and reuse conclusion

This is a straight Layer 1 reuse exercise:

1. reuse the existing command-level control suite,
2. reuse the existing live-status authority path,
3. reuse the existing detached continuity classifier,
4. reuse the existing docs as the operator contract source,
5. fix the contradiction, not the architecture.

### 0E. TODO cross-reference

There is no `TODOS.md` in the repo root today.

That means all deferrals must stay explicit in the `NOT in scope` section of this plan. Do not assume a missing backlog file will preserve the reasoning.

### 0F. Completeness check

The complete version is still a lake:

1. all truth surfaces agree,
2. the lifecycle is proven on one durable session,
3. parked and `awaiting_attention` rows are command-level regression protected,
4. inbox wording is exact,
5. validation is published.

Trying to add a public inbox UX or broader runtime producer contract here would turn the lake into an ocean. This plan does not do that.

## Architecture Review

### Architecture thesis

Do not redesign the durable host-session model. Freeze it and prove it.

The architecture is already correct enough for v1:

1. the durable session is the authority,
2. the attached host owner is an attachable execution client, not the identity of the session,
3. parked host state remains real state,
4. detached-world follow-up remains intentionally fail closed,
5. the operator contract is narrow and exact.

### Current-to-target shape

```text
CURRENT BRANCH SHAPE
====================
runtime contract: mostly landed
docs contract: partially contradictory
qa contract: strong but fragmented
inbox wording: broader than the shipped operator surface

TARGET CLOSEOUT SHAPE
=====================
runtime contract: unchanged
docs contract: one story everywhere
qa contract: one same-session lifecycle wall + explicit status assertions
inbox wording: exactly matches shipped reality
```

### Dependency graph

```text
docs/USAGE.md
    |
    +--> repo-root truth docs
    |      +--> HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md
    |      `--> AGENT_ORCHESTRATION_GAP_MATRIX.md
    |
    `--> llm-last-mile docs
           +--> README.md
           +--> SOW 25
           `--> PLAN-25

crates/shell/src/execution/
    |
    +--> agents_cmd.rs
    |      +--> run_turn(...)
    |      +--> run_reattach(...)
    |      +--> run_stop(...)
    |      `--> build_status_report(...)
    |
    +--> agent_runtime/state_store.rs
    |      +--> list_status_sessions_for_agent(...)
    |      +--> status_visible_participants(...)
    |      +--> persist_inbox_item(...)
    |      `--> classify_public_session_posture(...)
    |
    `--> agent_dev_support.rs
           `--> persist_runtime_alert_for_dev_support(...)

crates/shell/tests/
    |
    `--> agent_public_control_surface_v1.rs
           `--> command-level lifecycle and status contract wall
```

### Realistic production failure scenarios

1. `status` regresses to trace-fallback null fields for a valid parked session. Operators think the session vanished or lost posture truth.
2. `reattach` returns success before durable attached truth is actually restored. Operators trust a fake success and the next command misbehaves.
3. `stop` still depends on the attached-live owner plane for a valid parked session. The session becomes impossible to close cleanly.
4. Docs advertise a richer inbox workflow than the code supports. Operators try a workflow that does not exist.

This plan must close all four.

## Code Quality Review

### Boring-by-default rules

1. One prompt-taking resume verb, `turn`.
2. One owner-recovery verb, `reattach`.
3. One closeout verb, `stop`.
4. One parked-session authority path, durable session-root truth.
5. One honest inbox story, no more and no less.

### DRY guardrails

1. Do not add a new status helper for tests. Test the real command path.
2. Do not add a second parked-session classifier for docs or assertions. Reuse the real posture logic.
3. Do not invent a fake production inbox producer just to make the docs sound nicer.
4. If comment tightening is needed, do it next to the real seams in `agent_dev_support.rs` or `state_store.rs`.

### Engineered-enough boundary

This slice should be explicit rather than clever:

1. one new lifecycle regression is better than three new micro-tests that still leave the operator story fragmented,
2. one command-level parked-status assertion is better than a new helper or abstraction layer,
3. a doc narrowing diff is better than speculative runtime code that tries to make the docs true.

### Diagram maintenance

If any touched production file already has nearby contract comments or ASCII diagrams, update them in the same change. Stale diagrams are worse than no diagrams.

Recommended comment targets if production code changes are needed:

1. [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:505), around the `turn` / `reattach` / `stop` split,
2. [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2191), around detached continuity and public posture classification,
3. [`agent_dev_support.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs:11), around dev-support-only inbox ingress.

## Test Review

### Test framework detection

This is a Rust workspace. The relevant proof wall for this slice is `cargo test`, with shell integration tests carrying the operator contract and state-store tests carrying the durable posture math.

Primary suites:

1. [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
2. targeted `state_store.rs` tests for detached continuity and inbox posture math

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] Public durable host lifecycle
    |
    ├── [TESTED] start + streamed turn + authoritative stop
    │       agent_public_control_surface_v1.rs:1231
    |
    ├── [TESTED] detached parked start persists durable session
    │       agent_public_control_surface_v1.rs:1880
    |
    ├── [TESTED] pending inbox normalizes to awaiting_attention
    │       agent_public_control_surface_v1.rs:1980
    |
    ├── [TESTED] same-session stop after reattach
    │       agent_public_control_surface_v1.rs:1631
    |
    ├── [TESTED] exact world-member follow-up routing
    │       agent_public_control_surface_v1.rs:2689
    |
    ├── [TESTED] fail-closed taxonomy for unknown session, bad backend, detached world
    │       agent_public_control_surface_v1.rs:2363, 2492
    |
    ├── [GAP -> INTEG] one same-session lifecycle:
    │       parked start -> status -> parked turn -> status -> reattach -> stop
    |
    └── [GAP -> INTEG] explicit command-level assertion that parked and
            awaiting_attention rows expose live-runtime posture fields

[+] Durable inbox persistence math
    |
    ├── [TESTED] pending count increments and posture becomes awaiting_attention
    │       state_store.rs existing unit coverage
    |
    ├── [TESTED] ack/dismiss resolution updates pending count and posture
    │       state_store.rs existing unit coverage
    |
    └── [GAP -> DOC HONESTY] no shipped public runtime producer beyond dev support
```

### Operator flow coverage

```text
OPERATOR FLOW COVERAGE
===========================
[+] start --backend <host_backend_id> --prompt ...
    |
    └── [TESTED] session can establish and park durably

[+] status --json on that session
    |
    └── [GAP -> INTEG] parked and awaiting_attention command-level field assertions

[+] turn --session <sess> --backend <host_backend_id> --prompt ...
    |
    └── [TESTED] exact follow-up resumes the durable session

[+] reattach --session <sess>
    |
    └── [TESTED] attached-owner recovery works and remains same-session

[+] stop --session <sess>
    |
    └── [TESTED] durable closeout works for valid parked or reattached host sessions

[+] turn against detached world member
    |
    └── [TESTED] fail closed with reattach guidance
```

### Required test additions

| Area | File | Required assertions |
| --- | --- | --- |
| Same-session lifecycle closeout | [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | parked `start` -> `status` -> parked `turn` -> `status` -> `reattach` -> `stop` on one orchestration-session id |
| Parked status authority | [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | parked rows show live-runtime `posture`, `attached_participant_id`, and `pending_inbox_count`, not trace-fallback nulls |
| Attention-needed status authority | [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | `awaiting_attention` rows show the same live-runtime fields |
| Detached-world non-regression | existing detached-world tests in `agent_public_control_surface_v1.rs` | detached-world follow-up still fails closed and still points operators to `reattach` |
| Inbox posture math | existing `state_store.rs` tests | keep pending-count and posture invariants green, add nothing that implies new product semantics |

### Automated commands

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test --workspace -- --nocapture
```

### Test plan artifact

Implementation should write the normal eng-review test artifact to:

```text
~/.gstack/projects/<slug>/<user>-feat-host-orchestrator-durable-session-eng-review-test-plan-<timestamp>.md
```

Required contents:

1. parked host `start` lifecycle,
2. parked `status` visibility,
3. parked host `turn` reuse of the same session id,
4. `reattach` recovery,
5. `stop` closeout from detached and reattached states,
6. detached-world fail-closed behavior,
7. explicit note that public inbox runtime producers are not part of the shipped operator contract yet.

## Performance Review

Runtime cost risk is low. Flake risk is real.

Guardrails:

1. extend existing fixture helpers, do not add new sleep-driven polling loops,
2. keep parked-status assertions store-backed and deterministic,
3. prefer authoritative runtime rows over broad trace scanning,
4. do not add a new smoke harness when the shell integration suite already models the lifecycle.

The performance rule is simple: harden correctness without turning the closeout suite into an integration swamp.

## Failure Modes Registry

| Codepath | Realistic production failure | Test required | Error handling required | User-visible outcome |
| --- | --- | --- | --- | --- |
| parked `status` read | valid parked session regresses to trace-fallback null posture fields | yes | keep live-runtime rows authoritative | parked session stays visible and understandable |
| `awaiting_attention` status read | detached session with pending inbox work loses posture and count fields | yes | keep pending-count posture truth authoritative | operator sees pending attention state, not a ghost row |
| `reattach` success path | command reports success before durable attached truth is restored | yes | succeed only after durable attached proof | no fake reattach success |
| detached durable `stop` | stop still depends on attached-live control plane | yes | detached closeout remains authoritative | session stops cleanly |
| lifecycle truth docs | repo says `reattach`, `stop`, or parked `status` are unfinished | yes, via doc review | update truth docs | operator expectations stay aligned |
| inbox docs | repo implies a broader public inbox workflow than the code ships | yes, via doc review | narrow docs and comments | no false operator expectation |
| detached-world follow-up | later edits silently widen detached-world recovery | yes | preserve fail-closed routing and guidance | clear rejection, no unsafe fallback |

Critical-gap rule for this plan:

1. any regression that hides a valid parked or `awaiting_attention` session from `agent status --json` is a release blocker,
2. any fake `reattach` success is a release blocker,
3. any doc claim that advertises a public inbox workflow not backed by code is a release blocker.

## Implementation Sequence

### Step 1. Freeze repo truth first

Files:

1. [`HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
2. [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
3. [`docs/USAGE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
4. [`llm-last-mile/README.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)
5. [`llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)

Deliver:

1. remove stale language that still calls parked `status`, `reattach`, or `stop` unfinished,
2. freeze the exact `turn` / `reattach` / `stop` contract everywhere,
3. narrow inbox claims to persisted scaffolding plus posture normalization plus dev-support/test ingress.

Done means the repo has one story again before any new test work lands.

### Step 2. Add one same-session lifecycle regression

Files:

1. [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

Deliver:

1. one lifecycle test covering parked `start` -> `status` -> parked `turn` -> `status` -> `reattach` -> `stop`,
2. stable orchestration-session id across every non-terminal step,
3. terminal closeout only after `stop`,
4. no change to the public command grammar.

Done means the operator story is proven in one place instead of scattered across several tests.

### Step 3. Add explicit parked-status assertions

Files:

1. [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
2. production files in `crates/shell/src/execution/` only if the new assertions expose a real mismatch

Deliver:

1. parked rows must expose live-runtime `posture`, `attached_participant_id`, and `pending_inbox_count`,
2. `awaiting_attention` rows must expose the same fields,
3. trace-fallback null behavior remains limited to true trace-only rows.

Done means `status` is hardened as a real operator seam.

### Step 4. Tighten inbox comments only if needed

Files:

1. [`crates/shell/src/execution/agent_dev_support.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs)
2. [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Deliver:

1. make dev-support-only ingress obvious,
2. make persistence-vs-product-surface scope obvious,
3. add zero new runtime behavior.

Done means code comments cannot be read as promising a public inbox feature that does not exist.

### Step 5. Publish the validation wall

Files:

1. [`llm-last-mile/PLAN-25.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-25.md)

Deliver:

1. exact automated command list,
2. exact manual host-lifecycle smoke path,
3. explicit note that `awaiting_attention` and detached-world fail-closed remain automated-only proofs because no public inbox producer surface exists and world-member reproduction is fixture-heavy.

Done means future reviewers know exactly what must still work and what cannot honestly be claimed as a manual public flow.

## Acceptance Criteria

1. No repo truth surface says parked `status`, `reattach`, or `stop` are unfinished.
2. No repo truth surface implies a public inbox workflow beyond persisted scaffolding, posture normalization, and dev-support/test ingress.
3. One command-level lifecycle test proves parked `status`, parked `turn`, `reattach`, and `stop` on the same durable session id.
4. Command-level `status --json` assertions prove live-runtime fields for `parked_resumable` and `awaiting_attention`.
5. Detached-world follow-up remains fail closed with `reattach` guidance.
6. Existing lifecycle tests remain green.
7. Workspace tests stay green.
8. This plan contains the final validation wall and manual smoke rules.

## Validation Matrix

| Promise | Proof |
| --- | --- |
| parked durable host sessions stay visible on `status` | new command-level parked-status assertions plus existing store-level visibility tests |
| `awaiting_attention` remains authoritative | new command-level attention-needed assertions plus existing state-store posture math tests |
| same-session parked lifecycle remains stable through follow-up and closeout | new same-session lifecycle regression |
| `reattach` remains real attached-owner recovery | existing same-session stop-after-reattach coverage plus new lifecycle regression |
| detached parked `stop` remains canonical closeout | existing stop coverage plus new lifecycle regression |
| detached-world follow-up remains fail closed | existing detached-world fail-closed tests stay green |
| inbox posture math is real but public inbox workflow claims stay narrow | existing state-store tests plus doc/comment narrowing |
| repo truth matches code truth | doc diff review in the same PR |

## Manual Validation

### Exact manual smoke path

The manual smoke path for this slice is host-only. It must be runnable without special internal inbox or world-member fixtures.

```bash
substrate agent start --backend <host_backend_id> --prompt "hello" --json
substrate agent status --json
substrate agent turn --session <orchestration_session_id> --backend <host_backend_id> --prompt "next" --json
substrate agent status --json
substrate agent reattach --session <orchestration_session_id> --json
substrate agent stop --session <orchestration_session_id> --json
substrate agent status --json
```

Manual expectations:

1. `start` creates one durable orchestration session and the prompt-driven host client can exit without invalidating it.
2. the first `status --json` call shows the session as `parked_resumable` if `pending_inbox_count == 0`.
3. `turn` reuses the same orchestration-session id and does not require `reattach` first.
4. the second `status --json` call still shows the same session with live-runtime posture fields populated.
5. `reattach` restores attached ownership without submitting a prompt.
6. `stop` closes the same durable session terminally.
7. the final `status --json` call no longer presents the session as a live non-terminal durable session.

### Automated-only checks

The following remain automated-only on purpose:

1. `awaiting_attention` command-level visibility, because there is no public inbox producer surface to create that state manually without dev-support/test ingress.
2. detached-world fail-closed reproduction, because the retained world-member setup is fixture-heavy and the branch already treats the shell integration suite as the authoritative proof surface.

Automated expectations:

1. detached-world failure output still points operators to `substrate agent reattach --session <orchestration_session_id>`,
2. `awaiting_attention` rows still expose live-runtime fields at the command layer,
3. no test adds fake public inbox behavior just to make a manual demo path prettier.

## Worktree Parallelization Strategy

This slice has one real code lane and one real docs lane. It does not justify splitting the shell control suite across multiple workers.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Truth-doc convergence | `docs/`, repo root docs, `llm-last-mile/` | frozen contract in this plan |
| Lifecycle QA hardening | `crates/shell/tests/`, maybe tiny `crates/shell/src/execution/` fixes | frozen contract in this plan |
| Inbox-scope comment tightening | `crates/shell/src/execution/`, maybe `docs/` | truth-doc convergence |
| Final validation wall | `llm-last-mile/` | truth-doc convergence, lifecycle QA hardening |

### Parallel lanes

Lane A: lifecycle QA hardening  
Owns `crates/shell/tests/agent_public_control_surface_v1.rs` and any tiny supporting runtime fix it exposes.

Lane B: truth-doc convergence  
Owns `docs/`, repo-root truth docs, and packet docs. Do not edit Rust files in this lane unless Lane A is finished.

Lane C: inbox-scope comments plus final validation wall  
Runs after Lane A and Lane B converge. Small cleanup lane.

### Execution order

1. Freeze the contract in this plan.
2. Launch Lane A and Lane B in parallel worktrees.
3. Merge Lane A first if any runtime fix is needed.
4. Rebase Lane B if necessary and merge it next.
5. Run Lane C last for comment tightening and final validation publication.

### Conflict flags

1. Do not split `agent_public_control_surface_v1.rs` across multiple workers. That file is one hotspot and will just create merge noise.
2. If Lane B wants to edit comments in `state_store.rs` or `agent_dev_support.rs`, wait until Lane A settles. Both lanes may need those files.
3. Keep Lane B doc-only unless a doc contradiction cannot be resolved without a tiny code comment fix.

### Parallelization verdict

Three lanes total:

1. one code lane,
2. one docs lane,
3. one short cleanup lane after both converge.

If only one engineer is working the slice, execute sequentially in the same order and do not over-optimize for parallelism.

## Completion Summary

- Step 0: Scope Challenge, accepted as-is. The runtime contract is already chosen; the remaining work is closeout and truth convergence.
- Architecture Review: reuse the landed durable-session model and harden the contract seams around `status`, `reattach`, `stop`, and inbox wording.
- Code Quality Review: one public recovery model, one status authority path, one honest inbox story.
- Test Review: coverage diagram produced, two real test gaps identified, same-session lifecycle proof and command-level parked-status proof.
- Performance Review: low runtime-cost risk, moderate flake risk if new tests add custom polling or sleep-driven behavior.
- NOT in scope: written.
- What already exists: written.
- Failure modes: release blockers frozen around status visibility, fake reattach success, detached stop regression, and overstated inbox claims.
- Parallelization: 3 lanes, 2 can start independently, 1 waits for convergence.
- Lake Score: the complete option is to finish truth sync and closeout proof now, not ship another cycle of contradictory docs.

## Completion Checklist

- [ ] truth docs no longer say parked `status`, `reattach`, or `stop` are unfinished
- [ ] inbox docs say exactly what is shipped today and nothing more
- [ ] one same-session lifecycle regression proves parked `status`, parked `turn`, `reattach`, and `stop`
- [ ] command-level parked `status` rows show live-runtime posture fields
- [ ] command-level `awaiting_attention` rows show live-runtime posture fields
- [ ] detached-world fail-closed guidance remains green
- [ ] existing lifecycle tests stay green
- [ ] workspace test wall passes
- [ ] manual host lifecycle validation sequence is published in this plan
- [ ] automated-only checks are explicitly called out where no honest manual public flow exists

## Done Means

This slice is done when the durable host-session story stops depending on tribal knowledge.

An operator should be able to read the docs, run `status`, `turn`, `reattach`, and `stop`, and get exactly the behavior the repo claims.

A reviewer should be able to inspect one lifecycle regression, one validation matrix, and one manual smoke path and know the contract is real.

And nobody should walk away thinking the inbox product is more complete than the code actually proves.
