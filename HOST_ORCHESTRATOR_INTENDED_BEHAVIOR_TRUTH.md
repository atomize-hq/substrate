# Host Orchestrator Intended Behavior Truth

## Purpose

This document records the intended behavior for the Substrate host orchestrator model as it exists after the durable host-session work and the newer host-to-world design stack.

It is a truth record, not a brainstorm. It should say:

1. what public host-session behavior is already frozen,
2. what internal host-to-world behavior the new design docs now freeze,
3. what is already landed in the repo,
4. what is still design-only or proposed.

Primary design references:

- [llm-last-mile/DESIGN-world-worker-lifecycle-model.md](./llm-last-mile/DESIGN-world-worker-lifecycle-model.md)
- [llm-last-mile/DESIGN-host-to-world-steering-policy-matrix.md](./llm-last-mile/DESIGN-host-to-world-steering-policy-matrix.md)
- [llm-last-mile/DESIGN-durable-orchestration-obligation-ledger.md](./llm-last-mile/DESIGN-durable-orchestration-obligation-ledger.md)
- [llm-last-mile/DESIGN-router-daemon-attach-trigger-integration.md](./llm-last-mile/DESIGN-router-daemon-attach-trigger-integration.md)
- [llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md](./llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md)
- [llm-last-mile/DESIGN-auto-attach-trigger-and-work-queue-contract.md](./llm-last-mile/DESIGN-auto-attach-trigger-and-work-queue-contract.md)
- [llm-last-mile/DESIGN-host-orchestrator-tool-invocation-surface.md](./llm-last-mile/DESIGN-host-orchestrator-tool-invocation-surface.md)
- [llm-last-mile/DESIGN-internal-toolbox-transport-and-session-binding.md](./llm-last-mile/DESIGN-internal-toolbox-transport-and-session-binding.md)
- [llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md](./llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md)
- [llm-last-mile/SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md](./llm-last-mile/SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md)

Public durable-session references that remain authoritative:

- [docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md](./docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](./llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](./llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)

## Scope

This document covers two layers that now have to stay coherent with each other:

1. the public durable host-session contract under `substrate agent start|turn|reattach|fork|stop|status`,
2. the internal host-orchestrator-to-world control-plane and deferred-obligation model that sits behind that public contract.

This document does not claim that every forward design detail is already shipped. It distinguishes landed truth from forward design truth explicitly.

## Current Repo Classification

### Landed and should be treated as live truth

1. the durable authority is the Substrate orchestration session, not one backend process,
2. public `agent start`, `turn`, `reattach`, `fork`, `stop`, and `status` semantics are frozen around that durable session,
3. the local obligation ledger exists and is authoritative for detached attention projection,
4. manual `reattach` and automatic attach share one launch authority path,
5. internal host-to-world dispatch exists for:
   - `run_world_task`
   - `spawn_world_worker`
   - `fork_world_worker`
   - `continue_world_worker`
   - `inspect_world_worker`
   - `cancel_world_work`
   - `stop_world_worker`
6. the internal toolbox transport is already live and session-scoped, with read-side operator truth exported through `substrate agent toolbox status|env`,
7. host-to-world steering is deny-by-default through an explicit policy layer before routing proceeds.

### Not yet landed and must not be described as shipped

1. the first runtime-family host tool surface above the landed toolbox transport is still a follow-on slice, not a shipped runtime truth,
2. worker-requested fork autonomy, fork recommendations, approval autonomy, and auto-fork remain deferred,
3. the broader router/daemon production watch loop described by the router integration design is still remaining implementation scope,
4. the broader host-global or cross-host obligation envelope is still design-only,
5. active-ephemeral exact identity for inspect/cancel remains narrower than the forward design stack,
6. Codex is the first intended real smoke/validation floor for the host-tool landing, but that should not be misread as “force codex” or “block all other selected runtimes” unless implementation truth actually requires that,
7. non-Codex host-tool validation/support posture is still less proven and should be described honestly as such.

## Core Model

The host orchestrator session is a Substrate-owned durable session.

The durable authority is:

1. the orchestration session record,
2. the authoritative participant linkage,
3. the host attach contract,
4. the canonical obligation ledger,
5. the authoritative routing and lifecycle state.

The durable authority is not:

1. one currently attached backend process,
2. one currently running `codex exec` process,
3. one helper PID,
4. one transient inbox row,
5. one router or daemon attempt.

A Codex-backed host process is an attachable execution client. It may attach, run a prompt, exit cleanly, and later resume against the same durable orchestration session.

## Three Planes

There are three distinct planes and they must stay separate:

1. host orchestration plane:
   - reasons, plans, and decides,
   - owns the durable orchestration session.
2. Substrate control plane:
   - validates identity, policy, and lifecycle rules,
   - allocates or steers world work through explicit internal verbs.
3. world execution plane:
   - runs actual world-side agent work,
   - remains agent-native rather than becoming a generic toolbox execution path.

The host orchestrator does not directly become the world execution surface.

## Meaning Of `agent start`

`substrate agent start --backend <backend_id> --prompt ...` starts the host orchestration session.

It is intended to:

1. create or bind the durable Substrate-owned orchestration session,
2. run the user's initial prompt as the true initial backend prompt,
3. establish the orchestration session as active and durable,
4. leave that orchestration session available for future orchestration work until explicit stop.

It is not intended to:

1. send a hidden bootstrap prompt as the real backend user prompt,
2. wrap the user's prompt inside hidden control instructions and treat that wrapped text as the real initial backend prompt,
3. treat the backend process itself as the durable orchestration authority,
4. end the orchestration session merely because the initial prompt-bounded backend process exits.

## Prompt Semantics

For a Codex-style backend:

1. the initial user prompt for `agent start --prompt ...` must map to the real initial `codex exec` prompt semantics,
2. follow-up prompt-taking must map to resume semantics.

That means:

1. `agent start --prompt ...` uses the user prompt as the true initial backend prompt,
2. `agent turn --session ... --backend ... --prompt ...` is the prompt-taking follow-up path,
3. `agent reattach --session ...` is not a prompt-taking action.

The helper launched by `agent start` exists for Substrate-owned session setup, state bookkeeping, routing, and lifecycle management. It does not replace or rewrite the user's initial backend prompt semantics.

## Session Lifetime

When `agent start` succeeds, the orchestration session should remain open until:

1. `agent stop` is run,
2. or a future explicit stale/timeout lifecycle is added and deliberately transitions it.

The session should not stop being an open orchestration session simply because:

1. the initial backend process finished,
2. the foreground helper is no longer actively babysitting world agents,
3. the orchestrator is currently idle.

## Host Posture Meanings

### `parked_resumable`

`parked_resumable` does not mean the orchestration session ended.

It means:

1. the orchestration session is still active and durable,
2. no foreground attached host execution client is currently babysitting the session,
3. the session remains routable and resumable,
4. Substrate still owns the authoritative session state.

### `active_attached`

`active_attached` means:

1. the orchestration session is active,
2. an attached host execution client is currently present,
3. that attached client can immediately receive prompt traffic and foreground orchestration work.

### `awaiting_attention`

`awaiting_attention` is host-session truth, not world-worker lifecycle truth.

It means:

1. the orchestration session is still active and durable,
2. no host client is currently attached,
3. unresolved attention-driving obligations exist,
4. host-side review or reattachment is needed.

### `terminal`

`terminal` means:

1. the orchestration session is no longer routable,
2. the orchestration session is closed, failed, stopped, or invalidated and is no longer available for orchestration work.

## Worker Lifecycle Versus Host Posture

World-worker lifecycle state and host-session posture must remain separate.

Worker lifecycle answers:

1. is the worker allocated,
2. is it running,
3. is it `attention_pending`,
4. is it parked,
5. is it cancelled, stopped, completed, failed, or invalidated.

Host posture answers:

1. is a host execution client attached,
2. does the orchestration session have unresolved obligations,
3. is the host session routable.

Hard truth:

1. `attention_pending` belongs to retained world workers,
2. `awaiting_attention` belongs to the host orchestration session,
3. worker events may create obligations that drive host posture,
4. worker lifecycle state does not directly become host posture by itself.

## World Work Model

There are two lifecycle families for world work:

1. `ephemeral`
2. `retained`

### `ephemeral`

`ephemeral` means disposable one-shot work.

Its contract is:

1. no future continuation promise,
2. no stable future-routable participant identity promise,
3. if more work is needed, the system must escalate explicitly rather than pretending one-shot work can continue indefinitely.

### `retained`

`retained` means the world worker becomes part of authoritative orchestration state.

Its contract is:

1. stable worker identity exists,
2. later exact-target continuation is part of the contract,
3. worker events may create durable host obligations,
4. invalidation and world-binding drift fail closed.

## Host-To-World Steering Truth

The host orchestrator now has an internal control-plane contract for delegating or steering world work.

This is internal-only repo truth, not a widened public human CLI.

### Landed internal action surface

The current live internal action surface is:

1. `run_world_task`
2. `spawn_world_worker`
3. `fork_world_worker`
4. `continue_world_worker`
5. `inspect_world_worker`
6. `cancel_world_work`
7. `stop_world_worker`

### Not yet landed

1. worker-requested fork autonomy
2. approval autonomy
3. broader router-owned continuation behavior

## Host Tool-Surface Truth

The internal dispatch verbs and the internal toolbox transport are already live repo truth.

What is **not** fully landed yet is the runtime-family adapter that makes those verbs callable as agent-visible tools inside the selected host orchestrator session.

The current intended direction from the newer design/spec stack is:

1. dynamic orchestrator selection stays config/inventory/policy-driven rather than hard-coding `codex`,
2. the already-landed toolbox endpoint remains the runtime-owned discovery/binding seam,
3. selected host runtimes later receive `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` and `SUBSTRATE_AGENT_TOOLBOX_VERSION`,
4. because the first landing is not MCP-server registration, the selected runtime also needs startup/system-prompt disclosure of the seven-tool contract,
5. live calls still bridge through the frozen Slice `52` tool contract into the same internal toolbox transport,
6. the Codex-backed path is the first real smoke/validation floor for that landing,
7. that validation-first sequencing must not silently become a hard runtime gate or hard-coded orchestrator identity,
8. other selected runtimes should remain allowed unless the implementation proves a real incompatibility, and their posture should be reported as “not yet validated / not yet guaranteed” rather than being blocked by documentation alone.

Hard truth:

1. the missing seam is no longer “does the toolbox exist?”,
2. the missing seam is not “invent a generic tool catalog on the run request,”
3. the missing seam is the runtime-owned adapter above the landed transport,
4. Codex-first means “first proven smoke floor,” not “only legal runtime.”

### Exact identity rules

Internal host-to-world requests remain exact and fail closed.

The authoritative identity floor is:

1. `orchestration_session_id`
2. `caller_participant_id`
3. exact target `backend_id`
4. exact authoritative `world_id`
5. exact authoritative `world_generation`
6. exact retained `target_participant_id` where the action requires one

The runtime must not infer world targets heuristically.

## Steering Policy Truth

Host-to-world steering is not one boolean. It is deny-by-default and separate from runtime execution capability.

The policy layer must answer at least:

1. whether world dispatch is enabled,
2. which backends are allowed,
3. which modes are allowed,
4. which actions are allowed,
5. whether steering is same-session-only,
6. whether steering is same-world-binding-only,
7. whether capability narrowing is allowed,
8. whether worker concurrency or invalidation blocks routing.

Passing control-plane steering policy does not imply that runtime capability checks pass, and passing runtime capability checks does not imply that steering was authorized.

## Durable Obligation Truth

The canonical durable artifact for deferred host work is now the obligation ledger under the orchestration session.

That means:

1. obligations are canonical durable truth,
2. inbox or review surfaces are projections over obligations,
3. auto-attach processing is a projection over obligations,
4. host posture derives from unresolved attention-driving obligations,
5. no projection may bypass host-session authority or invent hidden prompt behavior.

The old split mental model of "one canonical inbox plus a separate canonical attach queue" is no longer the intended direction.

## Review and Inbox Truth

Inbox-like behavior still exists, but it is projection truth rather than canonical authority.

The intended contract is:

1. unresolved obligations may project to readable review or inbox state,
2. pending attention may normalize host posture from `parked_resumable` to `awaiting_attention`,
3. compatibility inbox artifacts may still exist,
4. those compatibility artifacts are not the canonical durable authority.

## Auto-Attach Truth

Auto-attach is a processing projection over obligations, not a second canonical ledger.

The hard intended rules are:

1. auto-attach evaluates unresolved obligations, not raw stream events,
2. attach-processing state belongs to the obligation record,
3. automatic attach restores a sanctioned host execution client but does not resolve obligations automatically,
4. manual `reattach` remains canonical and exact-session-scoped.

### Current landed scope

The repo already has:

1. local obligation attach states and claim metadata,
2. deterministic session-scoped attach claim/coalescing,
3. manual `reattach` and automatic attach sharing one launch authority path,
4. fail-closed fresh-attach truth after the `31.25` remediation.

### Still remaining scope

The repo does not yet clearly ship the broader production router/daemon watch loop described by the router integration design.

So the truth here is:

1. the local attach claim and execution machinery exists,
2. the broader router/daemon production integration remains a remaining slice,
3. this document must not claim that the full router design is already shipped.

## Parked Session Responsibilities

While parked, the orchestration session is still supposed to represent a live durable authority that can continue owning orchestration responsibilities.

That means parked or awaiting-attention sessions are intended to remain capable of:

1. receiving world-originated messages or updates through sanctioned runtime paths,
2. recording follow-up, blocked, approval, failure, and related deferred work as obligations when policy allows,
3. retaining those items durably while no host client is attached,
4. allowing later `turn` or `reattach` against that same session.

The orchestrator should not need to stay foreground-attached just to keep the session valid.

## Meaning Of `agent turn`

`substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ...` is the exact host prompt-taking follow-up path.

It is intended to:

1. target one exact durable orchestration session,
2. resume prompt-taking work against that same session,
3. preserve the durable session identity,
4. allow the session to return to parked afterward if no attached client remains.

It is not intended to:

1. create a fuzzy new session,
2. bypass the durable session record,
3. require the original initial backend process to still be running.

## Meaning Of `agent reattach`

`substrate agent reattach --session <orchestration_session_id>` is attached-owner recovery only.

It is intended to:

1. restore attached host ownership for the already-existing durable orchestration session,
2. leave the same durable session active,
3. make the session truly attached again,
4. settle or supersede attach claims when appropriate without inventing prompt traffic.

It is not intended to:

1. submit a prompt,
2. implicitly resolve obligations merely by existing,
3. return success while the session immediately falls back to parked.

If `reattach` reports success, the intended truth is that attached ownership was actually restored.

## Meaning Of Public `agent fork`

`substrate agent fork --session <orchestration_session_id>` is public host-session successor allocation only.

It is intended to:

1. allocate a new durable orchestration session that points back to the exact source session lineage,
2. copy the source session's persisted host attach-contract shape,
3. clear any inherited backend-native continuity selector from the successor,
4. return the successor honestly as `parked_resumable` with no attached owner loop.

It is not intended to:

1. submit a prompt,
2. attach a live owner loop immediately,
3. borrow the parent's continuity token,
4. report false `active_attached` truth for the successor session.

Important distinction:

1. public `agent fork` is host-session successor allocation,
2. internal `fork_world_worker` is a separate already-landed world-worker lineage action and must not be conflated with public host-session successor allocation.

## Meaning Of `agent stop`

`substrate agent stop --session <orchestration_session_id>` is the explicit shutdown action for the durable orchestration session.

It is intended to:

1. stop the durable orchestration session cleanly,
2. be the canonical closeout path for an active orchestration session,
3. work for the durable session model, not only for a currently attached live owner process.

The session remains open until `stop` is run, subject only to future explicit stale or timeout lifecycle rules if those are later designed and implemented.

## Meaning Of `agent status`

`substrate agent status --json` is intended to surface the durable host orchestration session truth.

That includes:

1. parked sessions,
2. awaiting-attention sessions,
3. attached sessions,
4. terminal truth when the session is no longer routable.

A parked or awaiting-attention session is still supposed to be visible as a real durable orchestration session, not disappear merely because no attached live owner process is currently present.

## Non-Negotiable Truths

The following are the key truths the repo still needs to honor:

1. the durable authority is the Substrate orchestration session, not one backend process,
2. `agent start` does not end the orchestration session when the initial prompt-bounded backend run exits,
3. parked does not mean gone,
4. parked does not mean terminal,
5. `awaiting_attention` is host-session posture derived from obligations, not a world-worker lifecycle state,
6. world-worker lifecycle state and host posture must remain distinct,
7. world execution remains agent-native while host delegation happens through a policy-gated control plane,
8. internal host-to-world steering remains exact-target and fail-closed,
9. obligations are the canonical deferred-work authority, while inbox and auto-attach are projections,
10. manual `reattach` remains canonical even when automatic attach machinery exists,
11. the truth document must distinguish landed behavior from proposed behavior instead of collapsing them together.
