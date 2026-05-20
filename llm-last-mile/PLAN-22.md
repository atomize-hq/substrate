# PLAN-22: Broaden Caller Surfaces From REPL-First To Public Session/Member Turns

Source SOW: [22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md)  
Gap matrix anchors: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:107), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:112)  
Adjacent landed slices: [PLAN-15.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-15.md), [PLAN-19.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-19.md), [PLAN-20.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-20.md), [PLAN-21.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-21.md)  
Branch: `feat/broaden-caller-surfaces-from-repl`  
Base branch: `main`  
Plan type: public caller-surface hardening and Linux-first world-member follow-up validation  
Review posture: unified execution plan, tightened to `/autoplan` and `/plan-eng-review` rigor  
Status: execution-ready planning pass on 2026-05-09

## Objective

Take the already-landed narrow public caller surface under `substrate agent` and make it an explicitly validated v1 contract for session-scoped and member-scoped prompt-taking.

This slice is complete only when all of the following are true:

1. `substrate agent start` remains the canonical public root prompt-taking surface for exact host backends.
2. `substrate agent turn` is proven end to end for exact `(orchestration_session_id, backend_id)` follow-up routing, not just lightly wired.
3. Linux world-sensitive public follow-up turns are explicitly proven from selector resolution through typed `MemberTurnSubmitRequestV1` submission into `world-agent`.
4. Detached host recovery and detached world fail-closed behavior are both explicit parts of the public contract.
5. The fail-closed taxonomy for public turns is covered with concrete tests instead of being partially implied by implementation.

This is productization and hardening of a real surface. It is not a new orchestration model.

## Plan Summary

The repo is much further along than the draft SOW implies.

Today the narrow public surface already exists:

1. CLI verbs are already exposed in [`AgentAction`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:519).
2. Public `start` and `turn` already load prompt input and enter the shared prompt path in [`run_start(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:301) and [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:324).
3. Exact public turn routing already exists in [`resolve_public_turn_target(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711).
4. The shared public prompt submission and rendering path already exists in [`run_public_prompt_command(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1104).
5. The Linux world-member submit boundary is already typed in [`MemberTurnSubmitRequestV1`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:845), accepted in [`submit_member_turn_stream(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1485), and identity-validated in [`validate_submit_turn_request(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:869).
6. Host-scoped public `start`, `turn`, `stop`, `reattach`, selector rejection, world-root start rejection, and `substrate -c` non-regression already have meaningful coverage in [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:662).

What is still missing is narrower and sharper:

1. the public Linux world-member follow-up path is not yet explicitly proven in the public control suite,
2. several fail-closed public-turn cases exist in code but are not yet pinned as explicit acceptance gates,
3. detached-world follow-up rejection exists in code but still needs direct contract coverage,
4. retained-member identity drift is only lightly proven at the world-agent boundary and should become an explicit public-turn acceptance gate,
5. repo-truth docs still lag the shipped surface in a few places, especially [`llm-last-mile/README.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md:64).

The minimum honest implementation is one cohesive hardening slice with four ordered workstreams:

1. tighten and prove the shell-side exact public turn contract,
2. add explicit public Linux world-member follow-up proof,
3. widen retained-member drift and detached-world fail-closed proof,
4. update operator and planning docs so repo truth matches reality.

## Locked Starting State

### What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Public caller verbs and canonical naming | [`AgentAction`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:519) | Reuse. Do not invent a second caller family outside `substrate agent`. |
| Public root prompt-taking wiring | [`run_start(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:301) | Reuse. This stays the canonical root caller surface. |
| Public follow-up prompt-taking wiring | [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:324) | Reuse and harden. Do not split host and world follow-up into separate public verbs. |
| Exact `(session, backend)` selector resolution | [`resolve_public_turn_target(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711) | Reuse as the authoritative selector seam. Tighten coverage around it. |
| Shared prompt-source loading | [`load_public_prompt_source(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1067) | Reuse. This is already the one prompt-source contract. |
| Shared public prompt transport and rendering | [`run_public_prompt_command(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1104) | Reuse. Keep one public prompt path, not parallel host and world implementations. |
| Typed world-member submit request | [`MemberTurnSubmitRequestV1`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:845) | Reuse exactly. Do not invent a second world-turn request surface. |
| Linux world-agent public-turn submit entrypoint | [`submit_member_turn_stream(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1485) | Reuse exactly. This remains the typed public world-member seam. |
| Retained-member tuple validation | [`validate_submit_turn_request(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:869) | Reuse exactly. Tighten proof, do not loosen identity matching. |
| Host-scoped public control integration coverage | [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:662) | Reuse as the primary shell integration suite and extend it. |
| `substrate -c` shell-wrap contract | [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1129) | Reuse exactly. This remains a hard non-regression gate. |

### Exact remaining gap

1. [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:324) already branches on `PublicSessionPosture`, but detached-world rejection is still only implementation truth until there is a direct test for it.
2. [`resolve_public_turn_target(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711) already emits `missing_backend`, `missing_active_parent`, `stale_linkage`, `backend_not_in_session`, `ambiguous_backend_slot`, and `unsupported_platform_or_posture`, but the suite does not yet pin all of those as operator-facing acceptance cases.
3. [`run_public_prompt_command(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1104) already calls the exact turn resolver again before submission, but the suite still needs an explicit Linux world-member success path proving the public CLI reaches the retained world-member submit seam.
4. [`validate_submit_turn_request(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:869) already rejects tuple drift, but only a narrow backend-drift unit case is currently pinned in [`member_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:1112). That is not enough to make the hidden contract obvious.
5. [`llm-last-mile/README.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md:64) still describes the older `start|resume|fork|stop` public surface and needs to reflect the landed `turn`/`reattach` contract.

### Scope decision

Proceed as one hardening slice.

Do not split this into "shell fail-closed cleanup first" and "world-member proof later." That would keep the public surface half-proven and let docs drift another cycle. The public surface is either a deliberately validated v1 contract or it is not.

### Blast radius

GitNexus says the key caller-surface symbols are low individual blast-radius changes, but they sit on a cross-crate contract seam:

| Symbol | GitNexus risk | Why it matters |
| --- | --- | --- |
| [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:324) | `LOW` | One direct caller, but it is the entire public `agent turn` path and participates in the `handle_agent_command` and `run_shell_with_cli` flows. |
| [`resolve_public_turn_target(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711) | `LOW` | Two direct callers, [`run_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:324) and [`run_public_prompt_command(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1104). |
| [`run_public_prompt_command(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1104) | `LOW` | It is the one prompt-submission bridge for both public `start` and public `turn`. |
| [`submit_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:249) | `LOW` | One direct caller, [`submit_member_turn_stream(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1485), but it is the retained world-member trust boundary. |

Implication: keep changes boring and additive. The symbol graph is not huge, but the contract is sensitive because it crosses shell, state store, world-agent, and test fixtures.

## Frozen Execution Contract

If implementation wants to do something else, revise this plan first.

### Non-negotiable invariants

1. `substrate agent start` remains the canonical public root prompt-taking surface.
2. `substrate agent turn` remains the canonical public follow-up prompt-taking surface.
3. Public root start remains host-only in v1. World-only root start still fails closed.
4. Public follow-up turns require exact `--session <orchestration_session_id>` and exact `--backend <backend_id>`.
5. Exact `(session, backend)` resolution stays authoritative. No fuzzy routing, no latest-session fallback, no selector widening.
6. Linux world-sensitive follow-up turns keep using the typed `MemberTurnSubmitRequestV1` to `/v1/member_turn/stream`.
7. Detached host follow-up may recover through reattach semantics. Detached world follow-up must fail closed until the operator runs `substrate agent reattach --session ...`.
8. No public prompt-taking path may fall back to REPL state, `substrate -c`, or synthetic bootstrap prompt text.
9. `substrate -c` remains shell wrap mode, full stop.
10. Failures stay explicit and classifier-stable. If the command cannot prove the exact contract, it fails closed.

### Public start

`substrate agent start` stays exactly what it is now:

```text
substrate agent start --backend <backend_id> (--prompt <text> | --prompt-file <path> | --prompt-file -) [--json]
```

Rules:

1. exact host backend only,
2. exact prompt-source requirement stays intact,
3. output remains streamed through the existing public prompt plane,
4. final `session_posture` remains authoritative only at command completion time.

### Public turn

`substrate agent turn` stays exactly what it is now:

```text
substrate agent turn --session <orchestration_session_id> --backend <backend_id> (--prompt <text> | --prompt-file <path> | --prompt-file -) [--json]
```

Rules:

1. session selector must be canonical orchestration-session identity only,
2. backend selector must be exact `<kind>:<name>`,
3. exact selector resolution is necessary but not sufficient, the runtime path must also prove the target is currently valid,
4. host and world follow-up remain separate internal execution paths, hidden behind one public verb.

### Hidden engineering contract for world-member follow-up

For Linux world-sensitive public follow-up turns, the public selector pair `(orchestration_session_id, backend_id)` is only the front door. The retained identity tuple is the real trust boundary.

That tuple remains:

1. `participant_id`
2. `orchestrator_participant_id`
3. `backend_id`
4. `world_id`
5. `world_generation`

The shell must submit those fields through [`MemberTurnSubmitRequestV1`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:845), and [`validate_submit_turn_request(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:869) must keep rejecting drift before turn submission resumes the retained member.

### Fail-closed rules

Public follow-up turns must fail closed for:

1. `missing_backend`
2. `unknown_session`
3. all non-canonical session selectors
4. `missing_active_parent`
5. `backend_not_in_session`
6. `stale_linkage`
7. `ambiguous_backend_slot`
8. `unsupported_platform_or_posture`
9. `owner_unreachable`

No fallback is allowed to a guessed session, guessed backend, REPL handle, or shell-wrap execution.

## Step 0: Scope Challenge

### 0A. Minimum honest diff

The minimum honest implementation is:

1. extend shell tests so the public `turn` path explicitly proves Linux world-member follow-up success,
2. extend shell tests so each public fail-closed classifier named above is explicitly pinned,
3. extend world-agent tests so retained-member identity drift is proven as a first-class contract, not a lightly covered side case,
4. update docs and planning artifacts that still describe the older narrower public caller surface.

Anything smaller leaves the public surface partially implied instead of deliberately validated.

### 0B. Complexity check

This slice should stay below the "new infrastructure" threshold even if it touches more than eight files. The implementation should feel like hardening, not invention.

Expected primary modules:

1. `crates/shell/src/execution/agents_cmd.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. `crates/shell/src/execution/agent_runtime/control.rs`
4. `crates/shell/tests/agent_public_control_surface_v1.rs`
5. `crates/shell/tests/support/`
6. `crates/world-agent/src/member_runtime.rs`
7. `docs/USAGE.md`
8. `AGENT_ORCHESTRATION_GAP_MATRIX.md`
9. `llm-last-mile/README.md`

That is above the file-count smell, so the plan must stay boring:

1. no new command surface,
2. no new transport,
3. no new public selector type,
4. no platform-general parity push beyond the Linux-first contract already frozen.

### 0C. Search and reuse check

Repo search says the right reuse story is already sitting in the code:

- **[Layer 1]** reuse the exact public selector resolver in [`resolve_public_turn_target(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711),
- **[Layer 1]** reuse the exact public prompt bridge in [`run_public_prompt_command(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1104),
- **[Layer 1]** reuse the typed world-member submit contract in [`MemberTurnSubmitRequestV1`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:845),
- **[Layer 1]** reuse the existing public control integration harness in [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:662),
- **[EUREKA]** the missing work is not "build a public caller surface." The public caller surface is already live. The missing work is proving the world-member path and proving the failure edges so the public contract is honest.

### 0D. TODOS cross-reference

There is no `TODOS.md` in the repo root today. That means deferrals must be captured explicitly inside this plan's `NOT in scope` section and, if implementation discovers adjacent but non-blocking work, in a follow-up backlog artifact rather than assumed root TODOs.

### 0E. Completeness check

This slice should boil the lake inside its blast radius.

The complete version includes:

1. host-scoped public start/turn regressions staying green,
2. Linux world-member public-turn success proof,
3. explicit classifier coverage for each fail-closed case named in the SOW,
4. detached host and detached world posture proof,
5. retained-member identity drift proof at the world-agent seam,
6. doc truth updates.

Skipping any one of those would save little time and keep the public contract fuzzy.

### 0F. Distribution and docs check

No new artifact type is introduced.

Distribution still matters in the boring sense:

1. `docs/USAGE.md` must describe the public surface the code actually ships,
2. `AGENT_ORCHESTRATION_GAP_MATRIX.md` must stop talking about this surface as if it were still hypothetical,
3. `llm-last-mile/README.md` must stop lagging behind the landed `turn`/`reattach` surface,
4. `PLAN-22.md` itself becomes the implementable artifact for this slice.

## Architecture Review

### Architecture thesis

Keep the public surface exactly where it is, `substrate agent`, and finish proving it.

The shell remains the selector and owner-transport authority. The Linux world-agent remains the world-sensitive authority. The public `(session, backend)` pair remains the front door, not the full trust boundary.

### Data flow

```text
CURRENT
=======
substrate agent turn
    |
    +--> run_turn(...)
    |      |
    |      +--> load_public_prompt_source(...)
    |      +--> resolve_public_turn_target(...)
    |      +--> detached host can recover
    |      `--> detached world fails closed
    |
    `--> run_public_prompt_command(...)
           |
           +--> resolve_public_turn_target(...) again
           +--> host: private owner transport
           `--> world: typed retained-member submit on Linux

What is weak today:
    - host path is directly proven
    - world-member path is mostly code truth
    - several fail-closed branches are mostly code truth


TARGET
======
substrate agent turn
    |
    +--> exact prompt load
    +--> exact (session, backend) resolution
    +--> exact posture branch
    |      |
    |      +--> active host         -> private owner prompt stream
    |      +--> detached host       -> reattach then private owner prompt stream
    |      +--> active Linux world  -> MemberTurnSubmitRequestV1 -> /v1/member_turn/stream
    |      +--> detached world      -> fail closed with reattach guidance
    |      `--> terminal/invalid    -> fail closed with explicit classifier
    |
    `--> tests prove every branch above

world-agent boundary
    |
    +--> MemberTurnSubmitRequestV1.validate()
    +--> validate_submit_turn_request(...)
    `--> submit_turn(...)

Tuple drift or stale linkage never degrades into best effort.
```

### Workstream 1: shell-side public turn contract hardening

Goal: make the exact public turn selector and posture contract fully explicit in shell-level tests and only minimally adjusted in code where gaps appear.

Files:

- [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:324)
- [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711)
- [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1104)
- [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:662)

Required changes:

1. Add explicit coverage for `missing_backend`, `unknown_session`, `missing_active_parent`, `backend_not_in_session`, `stale_linkage`, `ambiguous_backend_slot`, and `owner_unreachable`.
2. Add explicit coverage for detached-world follow-up rejection with the required `reattach` guidance.
3. Keep the current host-scoped public `start` and `turn` assertions green.
4. Keep `substrate -c` shell-wrap behavior green.

Exit criteria:

1. every operator-facing public-turn classifier named in the frozen contract has a direct assertion,
2. detached host and detached world posture behavior are both pinned,
3. no host-scoped regression is introduced while tightening the contract.

### Workstream 2: public Linux world-member follow-up proof

Goal: explicitly prove that a public `substrate agent turn --session ... --backend <world-backend>` reaches the retained Linux world-member seam, not just a generic success path.

Files:

- [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:662)
- [`crates/shell/tests/support/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/)
- [`crates/shell/tests/repl_world_first_routing_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Required changes:

1. Add an integration scenario that prepares an authoritative world-scoped retained slot for a public session/backend pair.
2. Submit a public `agent turn` against that exact pair and assert the world-sensitive path is used.
3. Assert that the shaped request carries the retained member tuple required by [`MemberTurnSubmitRequestV1`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:845).
4. Assert that the path reaches the Linux `/v1/member_turn/stream` seam rather than a host-only shortcut.

Exit criteria:

1. the public suite proves Linux world-member follow-up success end to end,
2. the proof is explicit about retained-member identity, not just "turn succeeded",
3. REPL world-first routing tests still pass unchanged.

### Workstream 3: retained-member identity drift and fail-closed proof

Goal: make the hidden retained-member tuple contract impossible to miss during review or regression.

Files:

- [`crates/world-agent/src/member_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:869)
- [`crates/world-agent/src/service.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1485)
- relevant `world-agent` tests

Required changes:

1. Keep the existing backend-drift rejection test.
2. Add at least one additional tuple-drift case at the retained submit boundary. `world_generation` or `world_id` is preferred because those are the highest-risk stale-slot cases.
3. Add or extend a boundary-level test proving the typed submit entrypoint keeps rejecting identity drift before turn submission starts.
4. Keep the failure text explicit enough that the operator and reviewer can tell which tuple field drifted.

Exit criteria:

1. retained-member identity drift is directly proven beyond one backend-only mismatch,
2. the world-agent boundary remains fail-closed before submitted-turn execution begins,
3. the hidden tuple contract becomes obvious from tests, not just from reading production code.

### Workstream 4: operator and planning truth updates

Goal: make docs tell the truth after the hardening work lands.

Files:

- [`docs/USAGE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:62)
- [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:107)
- [`llm-last-mile/README.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md:64)
- the source SOW and this plan if wording needs tightening for posterity

Required changes:

1. update operator docs only where behavior or wording is still stale,
2. explicitly describe `turn`/`reattach` where older docs still imply `resume` or omit follow-up prompt-taking,
3. keep the Linux-first posture and host-only root start caveat explicit,
4. keep docs aligned with the actual acceptance gates, not aspirational future parity.

Exit criteria:

1. no repo-truth doc still describes the pre-`turn` public surface,
2. docs match the shipped failure posture,
3. no doc claims broader platform parity than the code proves.

## Code Quality Review

### Boring-by-default rules

1. One public caller family, `substrate agent`.
2. One exact public turn resolver, [`resolve_public_turn_target(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711).
3. One public prompt bridge, [`run_public_prompt_command(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1104).
4. One world-sensitive submit request, [`MemberTurnSubmitRequestV1`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:845).
5. One fail-closed posture for detached world follow-up, explicit rejection with reattach guidance.

### DRY and abstraction guardrails

1. Do not add a second public turn resolver for tests. Tests should exercise the real one.
2. Do not add a second world-turn request just to make assertions easier. Assert against the existing typed request.
3. Do not fork host and world prompt rendering behavior inside docs or tests. The test should pin where the internal path differs, not create a new contract.
4. If implementation needs helper fixture changes, keep them in `crates/shell/tests/support/` rather than hand-building parallel fake state in each test.

### Diagram maintenance

If touched files already contain nearby ASCII diagrams or contract comments, update them in the same change. Stale diagrams are worse than no diagrams.

Recommended inline diagram comment locations if code changes are needed:

1. [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:707), around the exact `(session, backend)` public-turn routing comment,
2. [`member_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:848), around retained-member identity and submit validation,
3. [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:324), if posture branching gets any new nuance.

## Test Review

### Test framework detection

This repo is Rust-first. The acceptance surface here is `cargo test`, with shell integration tests and world-agent unit or integration tests doing the real work.

Primary suites for this slice:

1. [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
2. [`crates/shell/tests/repl_world_first_routing_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
3. targeted tests around [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
4. targeted tests around [`crates/world-agent/src/member_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/tests/agent_public_control_surface_v1.rs
    |
    ├── public host start -> turn -> stop flow
    │   └── [★★★ TESTED] start/turn streaming NDJSON + authoritative state
    |
    ├── public reattach + fork lineage flow
    │   └── [★★★ TESTED] exact session + lineage preservation
    |
    ├── non-canonical selector rejection
    │   └── [★★  TESTED] active handle / participant / internal session rejection
    |
    ├── public world-root start rejection
    │   └── [★★★ TESTED] host-only v1 root start remains fail-closed
    |
    ├── shell-wrap `substrate -c`
    │   └── [★★★ TESTED] remains shell mode, not agent prompting
    |
    ├── public Linux world-member turn success
    │   └── [GAP -> INTEG] exact `(session, backend)` world follow-up reaches typed member-turn submit seam
    |
    ├── detached world follow-up rejection
    │   └── [GAP -> INTEG] explicit `reattach` guidance assertion
    |
    ├── missing_backend
    │   └── [GAP -> UNIT/INTEG] direct public-turn rejection
    |
    ├── unknown_session
    │   └── [GAP -> UNIT/INTEG] direct public-turn rejection
    |
    ├── missing_active_parent
    │   └── [GAP -> INTEG] missing parent metadata and inactive parent rejection
    |
    ├── backend_not_in_session
    │   └── [GAP -> INTEG] explicit exact-backend miss
    |
    ├── stale_linkage
    │   └── [GAP -> INTEG] backend mentioned but no authoritative retained target
    |
    ├── ambiguous_backend_slot
    │   └── [GAP -> INTEG] multiple authoritative retained targets for one backend
    |
    └── owner_unreachable / terminal posture
        └── [GAP -> INTEG] explicit terminal follow-up rejection

[+] crates/world-agent/src/member_runtime.rs
    |
    ├── validate_submit_turn_request(...)
    │   ├── [★★  TESTED] backend_id drift rejection
    │   └── [GAP -> UNIT] world_id or world_generation drift rejection as explicit boundary proof
    |
    └── submit_turn(...)
        └── [GAP -> UNIT/INTEG] boundary-level assertion that typed submit fails before execution on retained tuple drift

---------------------------------
COVERAGE TARGET
- keep existing host-scoped public caller coverage green
- add one explicit Linux world-member success path
- add explicit assertions for every named fail-closed classifier in this plan
- add at least one additional retained-tuple drift proof at the world-agent boundary
---------------------------------
```

### Operator flow coverage

```text
OPERATOR FLOW COVERAGE
===========================
[+] Operator runs `substrate agent start --backend cli:codex --prompt "hello" --json`
    |
    └── [★★★ TESTED] accepted -> event stream -> completed with session_posture

[+] Operator runs `substrate agent turn --session <host-session> --backend cli:codex --prompt "next" --json`
    |
    └── [★★★ TESTED] exact host follow-up succeeds

[+] Operator runs `substrate agent turn --session <world-session> --backend cli:claude_code --prompt "next" --json`
    |
    ├── [GAP -> INTEG] active Linux world-member follow-up succeeds through `/v1/member_turn/stream`
    └── [GAP -> INTEG] detached world follow-up rejects with reattach guidance

[+] Operator targets wrong or stale session/backend state
    |
    ├── [GAP] unknown_session
    ├── [GAP] missing_active_parent
    ├── [GAP] backend_not_in_session
    ├── [GAP] stale_linkage
    ├── [GAP] ambiguous_backend_slot
    └── [GAP] terminal owner_unreachable posture

[+] Operator uses a non-canonical session selector
    |
    └── [★★  TESTED] reject active handle / participant / internal selector

[+] Operator runs `substrate -c "printf shell-wrap"`
    |
    └── [★★★ TESTED] shell-wrap stays shell-wrap
```

### Required test files and assertions

| Area | File | Required assertions |
| --- | --- | --- |
| Public host caller non-regression | [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | current `start`/`turn`/`stop` and `reattach`/`fork` flows stay green |
| Public Linux world-member follow-up | [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) plus support fixtures | exact world-scoped public `turn` reaches typed world-member submit path |
| Exact fail-closed public-turn taxonomy | [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | direct classifier assertions for missing backend, unknown session, missing parent, stale linkage, ambiguous slot, terminal posture, detached-world guidance |
| World-first routing non-regression | [`crates/shell/tests/repl_world_first_routing_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) | REPL world routing stays unchanged while public proof is added |
| Retained identity drift | [`crates/world-agent/src/member_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) tests | at least one non-backend tuple drift case plus boundary-level submit rejection |

### Required commands

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
cargo test --workspace -- --nocapture
```

### Test plan artifact

Implementation should write the normal eng-review test artifact alongside code changes:

```text
~/.gstack/projects/<slug>/<user>-feat-broaden-caller-surfaces-from-repl-eng-review-test-plan-<timestamp>.md
```

Required contents:

1. host public start with streaming acceptance and completion,
2. host public follow-up turn,
3. Linux public world-member follow-up turn,
4. detached host follow-up recovery,
5. detached world follow-up rejection with `reattach` guidance,
6. explicit fail-closed selector and posture cases,
7. retained-member tuple drift rejection,
8. `substrate -c` shell-wrap non-regression.

## Performance Review

This slice is not algorithmically heavy. The real performance risk is accidental retry or lookup churn while tightening proof.

Guardrails:

1. keep exact resolution store-backed, no broad trace scans,
2. do not add duplicate world-member submission attempts just to gather more proof,
3. keep fixture setup deterministic so tests do not depend on long polling loops,
4. prefer direct state-store fixtures over heavyweight full-runtime setup when proving fail-closed cases that do not need streaming.

The performance principle here is simple: correctness first, but do not turn hardening into a slow flaky integration suite.

## Failure Modes Registry

| Codepath | Realistic production failure | Test required | Error handling required | User-visible outcome |
| --- | --- | --- | --- | --- |
| public host follow-up | session exists but backend slot does not | yes | explicit `backend_not_in_session` | clear follow-up rejection |
| public follow-up selector | session file exists but authoritative parent metadata is torn | yes | explicit `missing_active_parent` | clear follow-up rejection |
| public follow-up selector | backend mentioned in stale inventory but retained target is gone | yes | explicit `stale_linkage` | clear follow-up rejection |
| public follow-up selector | two authoritative slots claim the same backend | yes | explicit `ambiguous_backend_slot` | clear follow-up rejection |
| detached host follow-up | owner loop is gone but reattachable metadata remains | yes | reattach before submit | clear recovered success or explicit failure |
| detached world follow-up | world slot exists but no active host owner remains | yes | explicit detached-world fail-closed guidance | clear instruction to run `reattach` |
| active Linux world follow-up | selector resolves but request is mis-shaped before submit | yes | typed request validation plus exact tuple proof | explicit submission failure before execution |
| retained world-member follow-up | stale `world_generation` or wrong `world_id` reaches world-agent | yes | `validate_submit_turn_request(...)` rejection | clear tuple-drift rejection |
| public terminal follow-up | session is terminal but caller retries with a real backend | yes | explicit `owner_unreachable` | clear terminal rejection |
| shell-wrap mode | `-c` accidentally starts reading prompt semantics | yes | no prompt re-interpretation | normal shell output only |

Critical-gap rule for this plan:

Any path that can submit a public follow-up turn without proving the exact retained target identity is a release blocker. Any path that silently downgrades world-sensitive follow-up into host-local best effort is also a release blocker.

## Implementation Sequence

### Step 1. Freeze the exact public-turn acceptance matrix

Files:

1. [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
2. small production adjustments only if a classifier or wording mismatch is discovered while pinning tests

Deliver:

1. enumerate every fail-closed case already named by production code,
2. add explicit assertions for each,
3. keep the current host flows green.

Done means the operator-facing shell contract is no longer inferred from implementation.

### Step 2. Add public Linux world-member follow-up proof

Files:

1. [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
2. `crates/shell/tests/support/`
3. only the smallest production hooks needed to expose the right assertion points

Deliver:

1. prepare an authoritative world-scoped retained slot,
2. execute `substrate agent turn` against the exact pair,
3. prove the route reaches the typed retained-member seam,
4. prove detached-world posture rejects with reattach guidance.

Done means the public world-member path is no longer "probably right."

### Step 3. Tighten retained-identity drift proof at the world-agent boundary

Files:

1. [`crates/world-agent/src/member_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
2. any directly adjacent tests

Deliver:

1. preserve existing backend-drift proof,
2. add at least one more tuple-drift proof,
3. prove the boundary rejects drift before submitted-turn execution begins.

Done means the hidden tuple contract is obvious in tests and survives refactors.

### Step 4. Update operator and planning docs

Files:

1. [`docs/USAGE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
2. [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
3. [`llm-last-mile/README.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)
4. the source SOW only if language is still materially misleading after code lands

Deliver:

1. doc wording matches the shipped public surface,
2. Linux-first and host-only caveats remain explicit,
3. no stale mention of the older pre-`turn` public contract remains.

Done means repo truth matches code truth.

## Validation Matrix

| Promise | Proof |
| --- | --- |
| host-scoped public `start` and `turn` still work | existing `agent_public_control_surface_v1` assertions stay green |
| Linux public world-member follow-up now has direct proof | new shell integration scenario exercising exact `(session, backend)` world turn |
| detached-world follow-up remains fail-closed | explicit shell integration test with `reattach` guidance |
| each public-turn fail-closed classifier is stable | direct shell tests asserting classifier text |
| retained-member tuple drift remains rejected | world-agent tests around `validate_submit_turn_request(...)` and `submit_turn(...)` |
| `substrate -c` remains shell-wrap mode | existing shell integration assertion stays green |
| docs match shipped behavior | diff review in the same PR |

## NOT in scope

- redesigning `substrate -c`, because shell-wrap mode is already frozen and protected
- adding fuzzy or default backend routing, because exact `(session, backend)` is the whole public safety story
- public world-root start, because root start remains host-only in v1
- broad status-surface redesign, because this slice is prompt-taking hardening, not status UX
- `fork` or `stop` product expansion, because they are non-prompt-taking lifecycle verbs and only need non-regression coverage here
- Windows/WSL world-sensitive follow-up parity, because this plan is Linux-first validation of the existing retained-member seam
- any new daemon, background broker, or alternate public transport, because the current surface already exists

## Worktree Parallelization Strategy

This plan has real parallelization room if write boundaries stay disciplined.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Shell public-turn contract hardening | `crates/shell/src/execution/`, `crates/shell/tests/`, `crates/shell/tests/support/` | — |
| World-agent retained-identity drift proof | `crates/world-agent/`, `crates/agent-api-types/` | — |
| Public Linux world-member integration proof | `crates/shell/tests/`, `crates/shell/tests/support/`, possibly thin shell hooks | Shell public-turn contract hardening, World-agent retained-identity drift proof |
| Docs and planning truth updates | `docs/`, `AGENT_ORCHESTRATION_GAP_MATRIX.md`, `llm-last-mile/` | Public Linux world-member integration proof |

### Parallel lanes

Lane A: shell public-turn contract hardening  
Sequential within lane because the shell tests and any tiny wording or classifier fixes share the same public caller modules.

Lane B: world-agent retained-identity drift proof  
Sequential within lane because `member_runtime.rs` and its tests own the retained tuple contract.

Lane C: public Linux world-member integration proof  
Waits for A + B, then proceeds through shell fixtures and integration assertions.

Lane D: docs and planning truth updates  
Waits for C so wording matches shipped proof, not intermediate intent.

### Execution order

1. Launch Lane A and Lane B in parallel worktrees.
2. Merge A and B.
3. Launch Lane C on top of the merged result.
4. Finish with Lane D once the final tests pass.

### Conflict flags

1. Lane A and Lane C both touch `crates/shell/tests/agent_public_control_surface_v1.rs`. Do not run them in parallel.
2. Lane B must stay out of shell test fixtures. If it starts changing shell support code, the lane split stops being useful.
3. Lane D goes last. Updating docs before the public world-member proof lands will drift repo truth again.

### Parallelization verdict

Four workstreams, two immediately parallel implementation lanes, one integration lane, one final doc lane.

## Completion Summary

- Step 0: Scope Challenge, accepted as-is. This is the minimum honest hardening slice.
- Architecture Review: resolved in-plan. Reuse the landed public surface, prove the missing world-member and fail-closed edges.
- Code Quality Review: one caller family, one turn resolver, one prompt bridge, one typed world submit request.
- Test Review: coverage diagram produced, explicit gaps identified for world-member proof and remaining fail-closed cases.
- Performance Review: low algorithmic risk, moderate test-fixture churn risk if proof is implemented sloppily.
- NOT in scope: written.
- What already exists: written.
- Failure modes: critical-gap rule frozen.
- Parallelization: 4 workstreams, 2 parallel implementation lanes, 1 integration lane, 1 doc lane.
- Lake Score: the complete option wins because partial proof would still leave the public contract fuzzy.

## Completion Checklist

- [ ] public host `start` and `turn` remain green
- [ ] public Linux world-member `turn` is proven end to end
- [ ] detached host recovery stays explicit and tested
- [ ] detached world follow-up rejection is explicit and tested
- [ ] `missing_backend` is explicitly tested
- [ ] `unknown_session` is explicitly tested
- [ ] `missing_active_parent` is explicitly tested
- [ ] `backend_not_in_session` is explicitly tested
- [ ] `stale_linkage` is explicitly tested
- [ ] `ambiguous_backend_slot` is explicitly tested
- [ ] `owner_unreachable` terminal follow-up is explicitly tested
- [ ] retained-member tuple drift is explicitly tested beyond backend-only mismatch
- [ ] `substrate -c` remains shell-wrap mode
- [ ] docs and llm-last-mile truth artifacts match the shipped public surface

## Done Means

This slice is done when `substrate agent start` and `substrate agent turn` stop being "the public surface that mostly exists in code" and become "the public surface that is deliberately proven, fail-closed, and documented."

Not a new product surface. Not a transport rewrite. Just the last mile of honesty between the landed runtime and the contract we claim to support.
