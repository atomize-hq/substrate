# PLAN-25: Durable Host Session Closeout, Inbox Contract Honesty, And QA Hardening

Source SOW: [25-host-durable-session-closeout-and-qa-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)  
Gap matrix anchors: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:79), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:113)  
Truth anchors: [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md:184), [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:105), [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md:61)  
Adjacent landed slices: [PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-22.md), [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md), [24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)  
Branch: `feat/host-orchestrator-durable-session`  
Base branch: `main`  
Plan type: closeout pass for the durable host-session contract, focused on truth convergence, QA hardening, and inbox-contract honesty  
Review posture: unified execution plan, tightened to `/autoplan` and `/plan-eng-review` rigor  
Status: execution-ready planning pass on 2026-05-14

## Objective

Close the durable host-session slice without reopening the model.

This plan is complete only when all of the following are true:

1. The public recovery contract is frozen and stated unambiguously: `turn` is prompt-taking resume, `reattach` is attached-owner recovery, `stop` is durable session closeout, and `status` reflects durable session truth while parked.
2. Parked and attention-needed host sessions remain visible on `substrate agent status --json` from canonical session-root truth, not just from attached-live process truth.
3. `reattach` and `stop` remain regression-proof for the same durable session after detached parking, with command-level evidence instead of only store-level evidence.
4. The repo stops overstating inbox reality. Docs must say exactly what is shipped today and exactly what is not.
5. Manual and automated validation cover the whole operator lifecycle: parked `start`, parked `status`, parked `turn`, parked `reattach`, parked `stop`, and detached-world fail-closed behavior.

This is a closeout and honesty pass. It is not a new orchestration model and it is not an inbox product expansion.

## Plan Summary

The source SOW is directionally right, but the branch has moved a lot since it was written.

What is already landed on this branch:

1. Public prompt-taking and control verbs already exist in [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:505), [`run_reattach(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:565), and [`run_stop(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:667).
2. Hidden owner-helper readiness already accepts detached durable host continuity through [`wait_for_hidden_owner_helper_readiness(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:645) and [`valid_detached_host_continuity_posture(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2216).
3. Status already prefers authoritative runtime state through [`build_status_report(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1307) and [`list_status_sessions_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:630), and detached parked rows stay readable through [`status_visible_participants(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:66).
4. Command-level coverage already exists for the core lifecycle: [`public_start_turn_and_stop_emit_streaming_ndjson_and_authoritative_state()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1231), [`public_stop_cleanly_closes_same_durable_session_after_reattach()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1631), [`public_start_persists_detached_session_when_hidden_owner_helper_exits()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1880), [`detached_pending_inbox_normalizes_to_awaiting_attention()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1980), and [`public_turn_routes_linux_world_member_follow_up_through_typed_submit_path()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:2689).
5. The public turn fail-closed taxonomy is already substantially pinned in [`public_turn_fail_closed_taxonomy_is_explicit_for_missing_backend_unknown_session_and_parent_slot_errors(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:2363) and [`public_turn_fail_closed_taxonomy_is_explicit_for_world_linkage_ambiguity_and_detached_rejection()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:2492).

What is still actually open is narrower:

1. The truth docs disagree with the code. [`HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md:206) still says `reattach`, `stop`, and `status` are unfinished, while the branch now ships those paths and tests them.
2. The inbox contract is still overstated. The only production-code caller to durable inbox persistence is the dev-support seam in [`persist_runtime_alert_for_dev_support(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs:11). Repo docs currently talk like world-originated approvals, completions, and follow-up work are already a supported runtime contract. That is not what the code proves today.
3. The QA story is fragmented. The branch has good targeted tests, but it still needs one explicit closeout matrix that proves the whole operator lifecycle on the same durable session and publishes the exact manual validation sequence.
4. The `status` seam is functionally landed but still deserves regression-first treatment because GitNexus marks [`build_status_report(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1307) as `HIGH` blast radius: 1 direct caller, 2 affected execution flows, 3 affected modules.

The minimum honest implementation is one closeout slice with four ordered workstreams:

1. freeze the already-chosen public recovery contract and remove contradictory truth-doc language,
2. harden lifecycle QA around parked `status`, `turn`, `reattach`, and `stop`,
3. freeze the inbox contract honestly around what is truly shipped today,
4. publish a final validation matrix and manual smoke path that future edits must keep green.

## Locked Starting State

### What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Public parked-session prompt-taking | [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:505) | Reuse. `turn` is already the public follow-up verb. Do not collapse it back into `reattach` or invent a new prompt verb. |
| Public attached-owner recovery | [`run_reattach(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:565) | Reuse. The branch already chose `reattach` as a real public verb. This plan freezes that choice. |
| Public durable stop | [`run_stop(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:667) | Reuse. `stop` is already the canonical closeout surface for attached and parked durable host sessions. |
| Live-runtime status authority | [`build_status_report(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1307), [`list_status_sessions_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:630) | Reuse and regression-proof. Do not redesign status shape in this slice. |
| Detached parked-session posture classification | [`classify_public_session_posture(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2191), [`valid_detached_host_continuity_posture(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2216) | Reuse exactly. This is the durable host continuity contract. |
| Parked-session visibility on the read path | [`status_visible_participants(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:66) plus tests at [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:3695) | Reuse. Add command-level status assertions, not a second status model. |
| Pending inbox normalization | [`persist_inbox_item(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1039) and [`apply_pending_inbox_count(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1726) | Reuse the persistence and posture math. Do not pretend this is already a broad runtime product surface. |
| Detached start parking behavior | [`public_start_persists_detached_session_when_hidden_owner_helper_exits()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1880) | Reuse as the base regression. Extend validation around `status`, not startup semantics. |
| Awaiting-attention normalization | [`detached_pending_inbox_normalizes_to_awaiting_attention()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1980) | Reuse. This proves posture math, not runtime inbox product completeness. |
| Linux world follow-up fail-closed guidance | [`public_turn_fail_closed_taxonomy_is_explicit_for_world_linkage_ambiguity_and_detached_rejection()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:2492) | Reuse. This slice must not widen detached-world recovery. |

### Exact remaining gap

The remaining gap is now primarily contract honesty and QA completeness:

1. The code, tests, usage docs, README, truth doc, and gap matrix do not currently tell the same story.
2. The branch has real durable inbox persistence primitives, but it does not yet prove a production runtime path that creates or resolves world-originated inbox items. That means the current docs oversell the feature.
3. The branch has targeted lifecycle tests, but it still needs one closeout-level lifecycle proof that explicitly exercises the durable session across `status`, `turn`, `reattach`, and `stop` as one cohesive operator story.
4. The branch has store-level proofs for parked and attention-needed visibility, but the status command path still needs a clearer regression floor because `status` is a high-blast-radius read surface.

### Blast radius

GitNexus says the control/read seams are sensitive even though this slice should stay narrow:

| Symbol | GitNexus risk | Why it matters |
| --- | --- | --- |
| [`run_stop(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:667) | `HIGH` | 1 direct caller, 2 affected execution flows, 4 affected modules. This is a top-level CLI control surface. |
| [`run_reattach(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:565) | `HIGH` | Same blast profile as `run_stop(...)`. A fake success here poisons the whole durable-session story. |
| [`build_status_report(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1307) | `HIGH` | 1 direct caller, 2 affected execution flows, 3 affected modules. It is a read surface, but a wrong answer here lies to operators. |
| [`persist_inbox_item(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1039) | low production call-site count, high truth risk | Grep shows production callers are basically absent apart from dev support. The code seam is small. The product-claim risk is not. |

Implication: keep code changes boring, mostly additive, and test-first. The risky part is not complexity. The risky part is drifting the contract again.

## Frozen Execution Contract

If implementation wants to do something else, revise this plan first.

### Non-negotiable invariants

1. `substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ...` remains the exact follow-up prompt-taking surface.
2. `substrate agent reattach --session <orchestration_session_id>` remains attached-owner recovery only. It is not a prompt-taking alias.
3. `substrate agent stop --session <orchestration_session_id>` remains the durable session closeout action for attached and parked host sessions.
4. `substrate agent status --json` remains the authoritative parked-session read surface for live-runtime rows, with posture, `attached_participant_id`, and `pending_inbox_count` sourced from canonical session-root truth.
5. `parked_resumable` and `awaiting_attention` remain routable durable host postures. `terminal` remains the only non-routable posture family.
6. Detached-world follow-up remains fail closed until `reattach` restores an active host owner. This slice does not widen that posture.
7. No new public inbox grammar lands in this slice. No `agent inbox`, no public ack/dismiss surface, no new router.
8. Docs may only claim runtime inbox behavior that the code actually produces today.
9. `substrate -c`, `--command`, and piped shell mode remain shell execution surfaces, not agent-prompt aliases.

### Public recovery contract to freeze

```text
parked host session
    |
    +--> agent status
    |      `--> read durable parent-session truth
    |
    +--> agent turn
    |      `--> submit a prompt against the same orchestration session/backend pair
    |
    +--> agent reattach
    |      `--> restore active_attached host ownership without submitting a prompt
    |
    `--> agent stop
           `--> close the same durable orchestration session terminally
```

This is the chosen v1 shape. The plan does not revisit whether `reattach` should exist. The branch already answered that question.

### Durable inbox contract to freeze

The v1 inbox contract is narrower than the SOW originally implied:

1. The persistence primitives are real.
2. Posture normalization from parked to `awaiting_attention` based on `pending_inbox_count` is real.
3. Store-level ack/dismiss support is real.
4. A public operator-facing inbox UX is not shipped.
5. A production runtime path that creates world-originated approvals, completions, or follow-up inbox items is not yet proven on this branch.

That means this slice must choose honesty over aspiration:

1. either land one real production runtime producer and document it exactly, or
2. explicitly narrow the docs so inbox is described as persisted scaffolding plus posture normalization, not a finished runtime resume mechanism.

This plan chooses option 2. It is the smallest complete and honest closeout.

## Step 0: Scope Challenge

### 0A. Minimum honest diff

The minimum honest implementation is:

1. update the truth docs so they stop claiming `reattach`, `stop`, and parked `status` are unfinished,
2. add command-level lifecycle coverage that proves the same durable session survives parked `status`, parked `turn`, attached `reattach`, and `stop`,
3. add command-level status assertions for parked and attention-needed sessions so `build_status_report(...)` cannot silently regress,
4. narrow inbox wording everywhere so the repo only claims what the code actually ships today,
5. publish the exact manual validation sequence future patches must keep green.

Anything smaller leaves contradictions in the repo. Anything bigger widens scope into inbox productization.

### 0B. Complexity check

This slice should stay below the "new architecture" threshold even if it touches several files.

Expected primary modules:

1. `crates/shell/tests/agent_public_control_surface_v1.rs`
2. `crates/shell/src/execution/agents_cmd.rs`
3. `crates/shell/src/execution/agent_runtime/state_store.rs`
4. `docs/USAGE.md`
5. `HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`
6. `AGENT_ORCHESTRATION_GAP_MATRIX.md`
7. `llm-last-mile/README.md`
8. `llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md`

That is above the smell threshold on file count, but the shape is still boring:

1. no new public verbs,
2. no new selector types,
3. no new runtime daemon,
4. no new inbox UI,
5. mostly tests and truth-sync, with only tiny production changes if the new regressions expose a real hole.

### 0C. Search and reuse check

Search-before-building result, in practical terms:

- **[Layer 1]** reuse the existing command-level control suite in [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs).
- **[Layer 1]** reuse the status authority path in [`build_status_report(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1307) and [`list_status_sessions_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:630).
- **[Layer 1]** reuse the detached continuity classifier in [`valid_detached_host_continuity_posture(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2216).
- **[Layer 1]** reuse the existing docs surface in [`docs/USAGE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:105) as the operator contract source.
- **[EUREKA]** the real remaining gap is not runtime invention. It is that the repo still speaks in two voices: code says "landed narrowly," some docs still say "unfinished," and the inbox claims are ahead of the code.

### 0D. TODOS cross-reference

There is no `TODOS.md` in the repo root today.

That means deferrals must be captured explicitly in this plan's `NOT in scope` section and, if desired later, in a follow-on packet. Do not assume a root backlog file will preserve nuance.

### 0E. Completeness check

The complete version here is still a lake, not an ocean:

1. all truth docs agree on the same public contract,
2. status gets command-level parked and attention-needed regression coverage,
3. the same durable session lifecycle is proven end to end,
4. inbox wording is reduced to what the code actually proves,
5. manual validation steps are explicit.

Trying to add a broad inbox UX, approval flow, or automatic parked-world resume mechanism in this slice would turn a lake into an ocean. Do not do that here.

### 0F. Distribution and docs check

No new artifact type is introduced.

Distribution still matters in the boring sense:

1. `docs/USAGE.md` is operator contract,
2. `HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md` is architecture truth,
3. `AGENT_ORCHESTRATION_GAP_MATRIX.md` is rollout truth,
4. `llm-last-mile/README.md` is packet index truth,
5. this plan becomes the implementation truth for closeout.

If those documents disagree, the feature is not closed.

## Architecture Review

### Architecture thesis

Do not redesign the durable host-session model. Freeze it.

The runtime contract is already in the code. This slice just makes that contract explicit, regression-proof, and honest about where inbox behavior stops.

### Data flow

```text
CURRENT SHIPPED SHAPE
=====================
agent start
    |
    +--> hidden owner-helper launches
    +--> startup prompt completes
    `--> session can park as parked_resumable / awaiting_attention

agent status
    |
    +--> build_status_report(...)
    +--> live runtime state_store first
    `--> trace fallback only when live runtime truth is unavailable

agent turn
    |
    +--> exact (session, backend) resolution
    +--> parked host resumes same durable session
    `--> detached world fails closed with reattach guidance

agent reattach
    |
    `--> restore active_attached host ownership for same durable session

agent stop
    |
    +--> detached durable path closes via persisted closeout
    `--> attached path closes via private owner transport

WHAT IS STILL WEAK
==================
- truth docs still contradict runtime truth
- inbox persistence is real, runtime producers are not yet a supported product contract
- lifecycle QA exists in pieces rather than one closeout matrix

TARGET SHAPE
============
same runtime contract
    |
    +--> one repo-wide story
    +--> one command-level lifecycle validation story
    `--> one honest inbox story
```

### Workstream 1: contract convergence and truth-doc closeout

Goal: remove contradictory statements and freeze the already-chosen public recovery contract.

Files:

1. [`HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
2. [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
3. [`docs/USAGE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
4. [`llm-last-mile/README.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)
5. [`llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)

Required changes:

1. Rewrite the "Current Unfinished Gaps" section in the truth doc so it no longer says `reattach`, `stop`, and parked `status` are unlanded.
2. Keep the durable-session contract explicit: `turn`, `reattach`, `stop`, `status`, and detached-world fail-closed behavior.
3. Rewrite inbox language to distinguish shipped persistence/math from unshipped runtime producer and consumer paths.
4. Keep the gap matrix honest: remaining work here is closeout and parity breadth, not core durable-session existence.

Exit criteria:

1. no repo-truth doc claims that parked-session `status` is still missing,
2. no repo-truth doc claims a finished runtime inbox workflow that the code does not provide,
3. the same public recovery model appears everywhere.

### Workstream 2: lifecycle QA hardening on the same durable session

Goal: make the operator lifecycle provable as one cohesive story instead of several disconnected unit wins.

Files:

1. [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
2. only tiny production changes in [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) or [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) if the new regression floor exposes a real mismatch

Required changes:

1. Add one end-to-end command-level test that exercises the same durable session through:
   - parked `start`,
   - `status --json`,
   - parked `turn`,
   - `status --json` again,
   - `reattach`,
   - and `stop`.
2. Assert the orchestration-session id is stable across the parked `status`, parked `turn`, and `reattach` phases, then becomes terminal only after `stop`.
3. Assert that `status --json` shows live-runtime `posture`, `attached_participant_id`, and `pending_inbox_count` on parked rows, not trace-fallback nulls.
4. Preserve detached-world follow-up rejection in the same suite as a non-regression wall.

Exit criteria:

1. the entire host durable-session lifecycle is proven on one named session,
2. status visibility is proven at the command layer, not only at the store layer,
3. no new production code path is introduced unless a regression requires it.

### Workstream 3: inbox contract honesty

Goal: explicitly freeze what the inbox is today so future readers stop inferring a richer feature than exists.

Files:

1. [`crates/shell/src/execution/agent_dev_support.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs)
2. [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
3. the docs listed in Workstream 1

Required changes:

1. Make it explicit in code comments and docs that the shipped runtime surface today is:
   - durable inbox persistence primitives,
   - pending-count and posture normalization,
   - internal ack/dismiss support,
   - and dev-support/test ingress.
2. Make it explicit that the branch does **not** yet ship:
   - a public inbox operator surface,
   - a proven production runtime path for world-originated approval/completion/follow-up items,
   - or an automatic parked-session resume mechanism driven by inbox items.
3. If small comments are missing near [`persist_runtime_alert_for_dev_support(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs:11) or [`persist_inbox_item(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1039), add them so the code matches the docs.

Exit criteria:

1. no reader can confuse "durable inbox scaffolding exists" with "runtime inbox workflow is finished",
2. inbox claims are uniform across code comments and docs.

### Workstream 4: validation publication and manual smoke contract

Goal: leave behind one exact validation sequence that future patches must preserve.

Files:

1. [`llm-last-mile/PLAN-25.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-25.md)
2. optionally the closest durable-session validation doc if a better home exists

Required changes:

1. Publish the exact command sequence for parked `start`, parked `status`, parked `turn`, parked `reattach`, and parked `stop`.
2. Publish the exact detached-world fail-closed check and expected wording.
3. Tie the manual validation steps to the automated regression names in `agent_public_control_surface_v1.rs`.

Exit criteria:

1. the repo has one exact closeout validation path,
2. future contributors can tell what must still work without reverse-engineering several test files.

## Code Quality Review

### Boring-by-default rules

1. One prompt-taking resume verb, `turn`.
2. One owner-recovery verb, `reattach`.
3. One closeout verb, `stop`.
4. One parked-session authority path, canonical session-root state.
5. One honest inbox story, no more, no less.

### DRY and abstraction guardrails

1. Do not add a second status projection helper just for tests. Exercise the real command path.
2. Do not add a second parked-session classifier just for docs. Reuse [`classify_public_session_posture(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2191) and explain it accurately.
3. Do not add a faux production inbox writer just to make docs feel better. Either prove a real one or narrow the docs. This plan narrows the docs.
4. If code comments need tightening, update them near the real seams rather than adding a new explanatory file.

### Diagram maintenance

If touched files already contain nearby contract comments or ASCII diagrams, update them in the same change.

Recommended inline comment targets if production code changes are needed:

1. [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:505), around the `turn` / `reattach` / `stop` contract split,
2. [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2191), around public posture classification and detached continuity,
3. [`agent_dev_support.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs:11), around dev-support-only inbox ingress.

## Test Review

### Test framework detection

This repo is Rust-first. The relevant acceptance floor is `cargo test`, with shell integration tests carrying most of the contract proof.

Primary suites for this slice:

1. [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
2. targeted `state_store.rs` unit coverage where detached continuity and inbox posture are already modeled

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] Public durable start / turn / stop
    |
    ├── [★★★ TESTED] start + streamed turn + authoritative terminal stop
    │       agent_public_control_surface_v1.rs:1231
    |
    ├── [★★★ TESTED] detached parked start persists durable session
    │       agent_public_control_surface_v1.rs:1880
    |
    ├── [★★★ TESTED] detached pending inbox normalizes to awaiting_attention
    │       agent_public_control_surface_v1.rs:1980
    |
    ├── [★★★ TESTED] same-session stop after reattach
    │       agent_public_control_surface_v1.rs:1631
    |
    ├── [★★★ TESTED] world follow-up typed submit path
    │       agent_public_control_surface_v1.rs:2689
    |
    ├── [★★★ TESTED] fail-closed taxonomy for missing backend / unknown session / detached world
    │       agent_public_control_surface_v1.rs:2363, 2492
    |
    ├── [GAP -> INTEG] one cohesive same-session lifecycle:
    │       parked start -> status -> parked turn -> status -> reattach -> stop
    |
    └── [GAP -> INTEG] explicit command-level assertion that parked/attention-needed
            status rows are sourced from live runtime truth, not trace fallback

[+] Durable inbox persistence math
    |
    ├── [★★★ TESTED] pending count increments and posture becomes awaiting_attention
    │       state_store.rs:2749+
    |
    ├── [★★★ TESTED] ack/dismiss resolution updates pending count and posture
    │       state_store.rs:2802+
    |
    └── [GAP -> DOC HONESTY] no production runtime producer is proven beyond dev support

---------------------------------
COVERAGE TARGET
- keep all landed lifecycle tests green
- add one same-session closeout lifecycle proof
- add one command-level parked status proof
- add zero fake inbox tests for behavior the runtime does not ship
---------------------------------
```

### Operator flow coverage

```text
OPERATOR FLOW COVERAGE
===========================
[+] operator runs `substrate agent start --backend cli:codex --prompt "hello" --json`
    |
    └── [★★★ TESTED] session can complete startup and park durably

[+] operator runs `substrate agent status --json`
    |
    └── [GAP -> INTEG] explicit parked and awaiting_attention assertions at command layer

[+] operator runs `substrate agent turn --session <sess> --backend cli:codex --prompt "next" --json`
    |
    └── [★★★ TESTED] exact host follow-up resumes durable session

[+] operator runs `substrate agent reattach --session <sess> --json`
    |
    └── [★★★ TESTED] attached ownership recovery works and can be followed by stop

[+] operator runs `substrate agent stop --session <sess> --json`
    |
    └── [★★★ TESTED] detached or reattached durable session closes terminally

[+] operator targets detached world follow-up
    |
    └── [★★★ TESTED] fail closed with reattach guidance
```

### Required test files and assertions

| Area | File | Required assertions |
| --- | --- | --- |
| Same-session durable lifecycle | [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | parked `start` -> `status` -> `turn` -> `status` -> `reattach` -> `stop` on the same orchestration-session id |
| Parked status authority | [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | parked and `awaiting_attention` rows show live-runtime posture fields, not trace-fallback nulls |
| Detached-world non-regression | [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | detached world still fails closed and still points operators at `reattach` |
| Inbox posture math | existing `state_store.rs` tests | no new semantics, just keep existing pending-count and posture invariants green |

### Required commands

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test --workspace -- --nocapture
```

### Test plan artifact

Implementation should write the normal eng-review test artifact alongside code changes:

```text
~/.gstack/projects/<slug>/<user>-feat-host-orchestrator-durable-session-eng-review-test-plan-<timestamp>.md
```

Required contents:

1. parked host `start` lifecycle,
2. parked `status` visibility,
3. parked host `turn` reuse of the same session id,
4. `reattach` recovery,
5. `stop` closeout from both detached and reattached states,
6. detached-world fail-closed behavior,
7. explicit note that inbox runtime producers are not part of the shipped operator contract yet.

## Performance Review

This slice is low-risk on runtime cost. The risk is flakiness and regression noise.

Guardrails:

1. prefer extending existing fixture helpers over adding new polling loops,
2. keep status assertions store-backed and deterministic,
3. avoid broad trace scanning in tests when the live-runtime store already has the answer,
4. do not add expensive new smoke harnesses when the shell integration suite already models the lifecycle.

The performance rule here is simple: harden correctness without turning the closeout suite into a sleep-driven integration swamp.

## Failure Modes Registry

| Codepath | Realistic production failure | Test required | Error handling required | User-visible outcome |
| --- | --- | --- | --- | --- |
| parked `status` read | live parked session regresses to trace-fallback null posture fields | yes | prefer live state-store rows, warn only on true degradation | parked session remains visible and understandable |
| `reattach` success path | command returns success before durable attached truth is actually restored | yes | success only after active-attached proof | no fake "reattached" message |
| detached durable `stop` | command still depends on attached-live owner plane and fails on a valid parked session | yes | detached closeout path remains authoritative | session stops cleanly |
| lifecycle truth docs | repo claims `reattach` / `stop` / parked `status` are unfinished after code ships them | yes, via doc review | update docs | operator and maintainer expectations stay aligned |
| durable inbox docs | repo claims world-originated approvals/completions are already supported runtime behavior | yes, via doc review | narrow docs | no false operator expectation |
| detached world follow-up | later edits silently widen detached-world resume instead of requiring `reattach` | yes | keep fail-closed classifier and guidance | clear rejection, no unsafe fallback |

Critical-gap rule for this plan:

1. Any regression that hides a valid parked or attention-needed session from `agent status --json` is a release blocker.
2. Any doc claim that advertises a production runtime inbox path not backed by code is a release blocker.
3. Any `reattach` success that does not correspond to real durable attached truth is a release blocker.

## Implementation Sequence

### Step 1. Freeze the chosen public contract in the docs

Files:

1. [`HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
2. [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
3. [`docs/USAGE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
4. [`llm-last-mile/README.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)
5. [`llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)

Deliver:

1. remove stale "unfinished" wording for shipped `reattach`, `stop`, and parked `status`,
2. freeze the exact `turn` / `reattach` / `stop` split,
3. narrow inbox claims to persisted scaffolding plus posture normalization.

Done means the repo has one story again.

### Step 2. Add one same-session lifecycle regression

Files:

1. [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

Deliver:

1. create one lifecycle test that walks the same durable session through parked `status`, parked `turn`, `reattach`, and `stop`,
2. assert the session id stays stable until terminal closeout,
3. keep existing detached-world fail-closed assertions intact.

Done means the closeout contract is not scattered across five unrelated tests anymore.

### Step 3. Add command-level parked-status assertions

Files:

1. [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
2. only minimal production code if the new assertions expose a real bug

Deliver:

1. assert parked rows surface live-runtime `posture`, `attached_participant_id`, and `pending_inbox_count`,
2. assert `awaiting_attention` rows do the same,
3. preserve trace-fallback null behavior only for true trace-only rows.

Done means `status` is treated as the hardening seam it actually is.

### Step 4. Tighten inbox-contract comments if needed

Files:

1. [`crates/shell/src/execution/agent_dev_support.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs)
2. [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Deliver:

1. make dev-support-only ingress obvious,
2. make persistence vs product-surface scope obvious,
3. do not add new runtime behavior.

Done means the code comments stop fighting the docs.

### Step 5. Publish the exact validation wall

Files:

1. [`llm-last-mile/PLAN-25.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-25.md)

Deliver:

1. final automated command list,
2. final manual smoke steps,
3. mapping from manual checks to automated tests.

Done means future closeout reviews have one source of truth.

## Validation Matrix

| Promise | Proof |
| --- | --- |
| parked durable host sessions stay visible on `status` | new command-level parked-status assertions plus existing store-level visibility tests |
| same-session parked lifecycle remains stable through follow-up and closeout | new same-session lifecycle regression |
| `reattach` remains real attached-owner recovery | existing `public_stop_cleanly_closes_same_durable_session_after_reattach()` plus new lifecycle test |
| detached parked `stop` remains canonical closeout | existing stop-after-reattach and detached stop coverage stays green |
| detached world follow-up remains fail closed | existing detached-world fail-closed tests stay green |
| inbox posture math is real but runtime product claims are narrow | existing store tests plus truth-doc narrowing |
| repo truth matches code truth | doc diff review in the same PR |

## NOT in scope

- adding a public inbox grammar, because this slice is closeout and honesty, not new operator UX
- adding a production runtime approval/completion/follow-up inbox workflow, because that is a separate feature slice
- changing the `turn` / `reattach` / `stop` public split, because the branch already chose it
- widening detached-world recovery, because fail-closed guidance is already the intended v1 posture
- default-agent routing or fuzzy selectors, because exact session/backend targeting remains the safety story
- public world-root `start`, because root `start` remains host-only in v1
- Windows/WSL parity work, because this slice is about durable host-session closeout on the already-supported narrow surface

## Worktree Parallelization Strategy

This slice has limited but real parallelization room.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Lifecycle QA hardening | `crates/shell/tests/`, maybe tiny `crates/shell/src/execution/` adjustments | — |
| Truth-doc convergence | `docs/`, repo root truth docs, `llm-last-mile/` | frozen public contract only |
| Inbox contract honesty comments | `crates/shell/src/execution/agent_dev_support.rs`, `crates/shell/src/execution/agent_runtime/`, docs | Truth-doc convergence |
| Final validation wall | `llm-last-mile/` | Lifecycle QA hardening, Truth-doc convergence |

### Parallel lanes

Lane A: lifecycle QA hardening  
Sequential inside the shell control suite because the same integration file owns most of the contract.

Lane B: truth-doc convergence  
Can start once the parent freezes the public contract. Mostly doc-only, no shell test conflict.

Lane C: inbox-contract comments and final validation wall  
Waits for B so wording is not duplicated or contradictory.

### Execution order

1. Freeze the contract in the parent plan.
2. Launch Lane A and Lane B in parallel.
3. Merge Lane A first if it requires any tiny runtime adjustment.
4. Rebase Lane B if necessary, then finish Lane C.

### Conflict flags

1. If Lane B starts changing code comments in `state_store.rs` while Lane A is editing the same file for test fixes, merge conflicts are likely. Keep Lane B doc-heavy until Lane A settles.
2. Do not split the shell lifecycle assertions across multiple workers. `agent_public_control_surface_v1.rs` is one hotspot.

### Parallelization verdict

Three workstreams, one real code lane, one real docs lane, one short cleanup lane after both land.

## Completion Summary

- Step 0: Scope Challenge, accepted as-is. The branch already chose the runtime contract. This slice closes the honesty and QA gap.
- Architecture Review: reuse the landed durable-session model and harden the seams around `status`, `reattach`, and `stop`.
- Code Quality Review: one public recovery model, one status authority path, one honest inbox story.
- Test Review: coverage diagram produced, two real gaps identified, same-session lifecycle proof and command-level parked-status proof.
- Performance Review: low runtime risk, moderate flake risk if tests are implemented with new polling instead of existing helpers.
- NOT in scope: written.
- What already exists: written.
- Failure modes: release blockers frozen around status visibility, fake reattach success, and overstated inbox claims.
- Parallelization: 3 workstreams, 1 code lane, 1 docs lane, 1 cleanup lane.
- Lake Score: the complete option is to finish closeout and truth sync now, not ship another cycle of contradictory docs.

## Completion Checklist

- [ ] truth docs no longer say `reattach`, `stop`, or parked `status` are unfinished
- [ ] inbox docs say exactly what is shipped today and nothing more
- [ ] one same-session lifecycle regression proves parked `status`, parked `turn`, `reattach`, and `stop`
- [ ] command-level parked `status` rows show live-runtime posture fields
- [ ] command-level `awaiting_attention` rows show live-runtime posture fields
- [ ] detached-world fail-closed guidance remains green
- [ ] existing lifecycle tests stay green
- [ ] workspace test wall passes
- [ ] manual validation sequence is published in this plan

## Done Means

This slice is done when the durable host-session story stops depending on tribal knowledge.

Operators should be able to read the docs, run `status`, `turn`, `reattach`, and `stop`, and get exactly the behavior the repo claims. Reviewers should be able to inspect one lifecycle test and one validation matrix and know the contract is real. And nobody should walk away thinking the inbox product is more complete than the code actually proves.
