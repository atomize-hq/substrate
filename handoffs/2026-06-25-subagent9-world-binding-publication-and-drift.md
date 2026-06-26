# 2026-06-25 — subagent9 world-binding publication and drift

## Verdict
Short version: **partial yes**.

The two failures touch the **same authority seam**, but they do **not** look like one identical bug.

- **Host-only `missing_world_binding`** looks **intentional in the current slice**, not accidental. The contract explicitly says toolbox world-dispatch calls cannot bootstrap the first binding from a host-only session.
- **Stale injected `world_id` from a world-bound toolbox session** looks like a **real cross-layer drift bug**. The shell/toolbox path publishes and consumes world binding from the session record, while `world-service` enforces against its own live shared-world binding.

So the strongest statement is: **one shared architectural authority seam, with one deliberate fail-closed gate and one real stale-publication failure on that seam**.

## Why this thread has merit
The repo currently has **two authority surfaces for “current world binding”**:

1. **Shell/session publication surface**
   - `docs/USAGE.md:103-107` says session-root parent/participant records are the live-state authority boundary for `agent status` and `agent toolbox ...`.
   - `crates/shell/src/execution/agents_cmd.rs:2980-2993` publishes `active_world_binding` straight from `session.world_id/world_generation`.
   - `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs:1263-1281` injects toolbox dispatch `world_id/world_generation` straight from that same session snapshot.

2. **World-service execution truth**
   - `crates/world-service/src/service.rs:2640-2691` validates member dispatch against the live active shared binding and the live world id.
   - If shell-published binding and world-service live binding differ, dispatch fails before launch.

That split is exactly what the stale-world-id transcript shows.

## Evidence by failure
### 1) Host-only `missing_world_binding`
This looks contractually intentional, not like accidental drift.

- `docs/USAGE.md:92-97` freezes the rule: fresh toolbox world-dispatch is actionable **only when** the live host orchestrator session already carries `active_world_binding`; host-only start fails closed with `missing_world_binding`.
- `crates/shell/src/execution/prompt_fulfillment.rs:95-101` says the same thing in the injected prompt contract.
- The transcript in `.../6ae12191.../pasted-text.txt:75-108` matches that exact contract and explicitly concludes the session has no authoritative world binding and cannot bootstrap one through the toolbox path.

So this failure is not good evidence of stale publication. It is good evidence that the current design **allows host authority without world binding**, but **does not allow later world delegation through these tools unless some separate bootstrap has already attached binding**.

### 2) Stale `world_id` injected by a world-bound toolbox session
This looks like a real publication/consumption drift bug.

- `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs:1057-1067,1263-1281` show the toolbox adapter injects `world_id/world_generation` from the shell session snapshot.
- `crates/world-service/src/service.rs:2645-2691` independently validates against the active shared-world binding and current world id.
- The transcript in `.../725ba1c5.../pasted-text.txt:88-95,210-210` says exactly that: the raw host-tool call reached Substrate, but the session injected an older `world_id` while `world-service` expected a newer one.

That is a real split-brain symptom.

## Exact places shell/session truth can diverge from world-service truth
1. **Status publication vs execution enforcement**
   - `agents_cmd.rs:2980-2993` can report `active_world_binding` from session JSON.
   - `world-service/service.rs:2640-2691` can reject the same binding as stale.

2. **Toolbox request injection vs execution enforcement**
   - `tool_invocation_contract.rs:1263-1281` injects runtime-owned binding fields from the session snapshot.
   - `world-service/service.rs:2662-2689` fail-closes on mismatched `world_id/world_generation`.

3. **Binding publication happens only on narrow lifecycle hooks**
   - Initial public world start binding is published at `agents_cmd.rs:334-340` after `establish_public_world_start_binding()` (`agents_cmd.rs:1478-1491`).
   - Runtime startup persists initial binding at `async_repl.rs:3466-3471`.
   - Replacement/restart persistence happens at `async_repl.rs:8883-8908`.
   - Clear/tear-down paths happen at `async_repl.rs:8149-8153` and `8743-8749`.

   I did **not** find a per-dispatch revalidation/refresh step that re-derives authoritative binding from `world-service` right before toolbox injection.

4. **Healthy parked host state does not prove world-dispatch realizability**
   - `handoffs/2026-06-25-subagent4-host-orchestrator-parked-resume-routing.md:44-77` shows nearby mismatches: parked host/session truth can remain internally healthy while world-member routing or launch conditions have already gone stale.
   - This is adjacent to, but not identical with, the binding drift bug.

## Does the design intend to tolerate host authority without immediate world binding?
**Yes, but only in a narrow sense.**

Current intent appears to be:
- a host/orchestrator session may exist without world binding,
- toolbox discovery can still be exposed,
- but fresh world delegation through `run_world_task` / `spawn_world_worker` is not allowed until the session already has authoritative world binding.

That is consistent with:
- `docs/USAGE.md:90-97,125-139`
- `prompt_fulfillment.rs:99-101`
- the public world-start birth plan, which is still host-rooted but persists world binding before returning (`agents_cmd.rs:1349-1416`, `334-340`).

So the current design **does not** support the stronger idea of “host-only now, later delegate into the world on the same session through the toolbox path” unless a separate explicit binding bootstrap exists.

## Final judgment
- **As “one shared bug”**: **weak to medium**.
- **As “one shared authority/publication seam with one real bug on it”**: **strong**.

The stale-world-id failure is the real bug. The host-only `missing_world_binding` failure is mostly the repo telling the truth about a deliberate contract boundary.

## Recommended next step
Do one narrow design/implementation pass on **authoritative world-binding publication**:

1. Decide whether the session record is the **source of truth** or a **cache** for world binding.
2. If it is a cache, add an explicit **pre-dispatch refresh/revalidation** against `world-service` before toolbox injection and before surfacing `active_world_binding` as healthy.
3. Separately, decide whether host-only sessions should ever be able to acquire world binding later. If yes, add an **explicit bootstrap/bind verb**. Do not smuggle that through `spawn_world_worker`.

That split would cleanly separate the intentional fail-closed case from the real stale-binding drift bug.
