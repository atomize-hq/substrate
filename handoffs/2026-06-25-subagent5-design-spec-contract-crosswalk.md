# Design/Spec Contract Crosswalk — retained world worker vs direct world task

## Short objective
Determine whether the still-similar live failures point to (a) code drift from valid docs, (b) remaining doc omission, (c) a design-stack contradiction between one-shot `run_world_task` and retained parked/resume semantics, and (d) whether active ownership now belongs somewhere broader than `SPEC-63`.

## Document set inspected
Required handoffs/design/spec/plan/tasks:
- `handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md`
- `handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md`
- `handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md`
- `handoffs/2026-06-23-design-lineage-audit-retained-worker-resume.md`
- `handoffs/2026-06-23-retained-world-worker-lifecycle-debug.md`
- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md`
- `llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md`
- `llm-last-mile/DESIGN-retained-world-worker-messaging-and-steering-contract.md`
- `llm-last-mile/DESIGN-auto-attach-trigger-and-work-queue-contract.md`
- `llm-last-mile/DESIGN-durable-orchestration-notification-inbox-contract.md`
- `llm-last-mile/DESIGN-router-daemon-attach-trigger-integration.md`
- `llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md`
- `llm-last-mile/PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md`
- `llm-last-mile/TASKS-63.md`

Spot-checked live runtime/proof touchpoints:
- `crates/world-service/src/member_runtime.rs`
- `crates/shell/tests/agent_public_control_surface_v1.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
- `crates/shell/src/execution/agent_runtime/session.rs`

## Symptom crosswalk
| Current symptom / question | Owning doc/spec today | Status | Notes |
| --- | --- | --- | --- |
| Direct `run_world_task` to `cli:codex-world` can still fail for bootstrap/auth/config reasons unrelated to retained continuity | `SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md` + 2026-06-21 handoffs | covered | This is a bounded direct-member compatibility bridge problem. It is adjacent to retained lifecycle, but not owned by `SPEC-63`. |
| Clean bootstrap exit used to delete retained continuity before later `continue_world_worker` | `DESIGN-world-worker-lifecycle-model.md` + `SPEC-63` / `PLAN-63` / `TASKS-63` | covered | Current docs now explicitly freeze `running -> parked` after surfaced retained identity + resumable session identity, and current `member_runtime.rs` reflects that via `finish_bootstrap(..., preserve_retained_member)`. |
| Pure public root `substrate agent start --backend cli:codex-world --scope world` -> public `agent turn` still being treated as if `start` had already allocated a world member | `SPEC-30-public-world-scoped-agent-start-and-capability-flags.md` (cited by Packet 4 docs) | covered / misleading when forgotten | Packet 4 explicitly says its mixed REPL/private + public proof is **not** proof of pure public root start -> turn. If live testing is still using that expectation, the expectation is off-contract, not a `SPEC-63` miss. |
| Shell/runtime can now preserve retained slot state, but live manual follow-up may still fail against a real Codex-backed worker | Split ownership: `DESIGN-host-orchestrator-world-dispatch-contract.md` + `SPEC-62` bridge family + only partial `SPEC-63` ownership | omitted / partial | Remaining gap is not the old “registry deleted on bootstrap exit” seam. The open gap is whether a surfaced backend session handle (`uaa_session_id`) from the **transitional direct member bridge** is actually strong enough to make a real retained worker receipt truthful. That ownership is not frozen cleanly anywhere. |
| Auto-attach / inbox / router should somehow heal or own this failure | `DESIGN-auto-attach-trigger-and-work-queue-contract.md`, `DESIGN-durable-orchestration-notification-inbox-contract.md`, `DESIGN-router-daemon-attach-trigger-integration.md` | covered but not owner | These docs explicitly operate on obligations / attach / review projections and explicitly do **not** authorize hidden prompt submission or direct worker continuation. They are downstream of the active bug, not the right owner. |

## Does `SPEC-63` still look like the right active owner?
Only as a **partial owner** now.

`SPEC-63` was the right owner for the original lifecycle bug: shell truth preserved a retained participant, while `world-service` deleted it on clean bootstrap exit. That bug is now explicitly covered in the lifecycle design and appears reflected in current runtime code/tests.

If live manual testing still shows the same/similar failure, the higher-probability remaining issue is broader: the repo is still treating a surfaced `uaa_session_id` from the **direct `cli:codex-world` compatibility bridge** as if that alone proves real retained resumability. That is not purely a parked/resume lifecycle question anymore.

## Most likely wrong architecture assumption today
The most likely wrong assumption is:

> the current direct member bridge (`PromptFulfillmentBridge::for_member_backend(...)` + isolated `CODEX_HOME` + surfaced `uaa_session_id`) is being treated as equivalent to a truth-owned retained world-worker runtime.

Why this looks wrong now:
- the 2026-06-21 handoffs already frame the direct path as a **bounded transitional compatibility bridge**, not the target architecture;
- `SPEC-63` then assumes that once retained identity plus surfaced resumable session identity exist, later parked resume is truthful;
- current proof walls are still largely synthetic at the backend seam (`ReadyAndExit`/stubbed follow-up proof), while live manual reports are about real Codex-backed behavior.

So the unresolved stack tension is not “ephemeral vs retained verbs are undefined” — those are defined clearly. The tension is that the same transitional direct-member bridge is being asked to satisfy both:
1. one-shot bootstrap compatibility for `run_world_task`, and
2. durable retained resumability for `spawn_world_worker` / `continue_world_worker`.

That is the assumption most likely to be wrong.

## Recommend the next doc move?
Only one narrow move looks justified:

- **Do not reopen or widen `SPEC-63` first.**
- If live repro still fails on current code, create or retarget a narrow follow-on spec adjacent to the `SPEC-62` / world-dispatch bridge family (rooted in `DESIGN-host-orchestrator-world-dispatch-contract.md`) that freezes what must be true before a direct-member retained receipt is allowed to claim real resumability.

Concretely, that owner should define whether `uaa_session_id` + preserved launcher state + isolated `CODEX_HOME` is actually sufficient backend truth for real parked resume, or whether retained world workers need a different gateway-owned runtime contract.

## Final take
- **(a) implementation drift from valid docs:** yes, but only partially and not in the original June 23 sense; if the live bug persists, the drift is now more likely in the direct-member resume realization than in the retained-slot lifecycle rule itself.
- **(b) doc omission still remains:** yes; the stack still does not cleanly freeze backend-resume sufficiency for the direct `cli:codex-world` bridge.
- **(c) design contradiction:** not a first-order verb contradiction; the contradiction is architectural/proof-level — a transitional one-shot compatibility bridge is being treated as sufficient evidence for durable retained-worker truth.
- **(d) active owner:** `SPEC-63` is now only a partial owner. The more likely active owner for the still-live bug is the world-dispatch/direct-member bridge contract family (`DESIGN-host-orchestrator-world-dispatch-contract.md` plus the `SPEC-62` adjacency or a new follow-on spec), not the router/inbox/auto-attach docs.
