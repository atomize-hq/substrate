# Agent Orchestration Gap Matrix

## Canonical Product Intent

This section is the canonical description of the intended product shape for the current Substrate agent-orchestration effort.

### v1 intent

Substrate v1 orchestration is intended to use real external CLI agents through the Unified Agent API (`unified-agent-api` on crates.io; imported in Rust code as `agent_api`), not a bespoke Substrate-native agent runtime.

In v1:

- Substrate selects, policy-gates, and launches supported CLI agents such as Codex, Claude Code, and future additions that are onboarded into Unified Agent API.
- Substrate assigns those agents roles:
  - a host-scoped orchestrator, and
  - one or more member agents that execute inside a Substrate-managed world boundary.
- Substrate provides the execution boundary, policy, inventory, world lifecycle, trace correlation, and orchestration control plane.
- Unified Agent API provides the backend abstraction for the CLI agents themselves:
  - capability discovery,
  - run semantics,
  - explicit cancellation,
  - session resume/fork extensions,
  - and session-handle surfacing.
- The first meaningful shipped path is:
  - host orchestrator,
  - in-world member execution,
  - policy-gated orchestration,
  - and shared-world reuse where needed for member sessions.

### later intent

A later Substrate-native harness may still sit on top of Codex, Claude Code, or other Unified Agent API backends, but that is a follow-on architecture layer, not a prerequisite for v1.

That later layer may own:

- Substrate-specific session/context models,
- custom orchestration UX,
- and deeper runtime integration.

For the current phase, the priority is to get the existing CLI agents working cleanly through Unified Agent API inside Substrate’s orchestration and world model.

## Canonical Decisions

These items should be treated as decided unless explicitly revisited.

1. `unified-agent-api` is the canonical source of truth for CLI-agent runtime semantics.
2. Substrate intends to consume `unified-agent-api` as a normal crates.io dependency, not as a path dependency and not as an indefinitely separate local protocol.
3. The crates.io package is `unified-agent-api`; the default Rust import name in code is `agent_api`.
4. Substrate v1 should use external CLI agents through Unified Agent API rather than waiting for a custom Substrate-native harness.
5. Substrate owns orchestration semantics, world placement, policy, and trace/audit semantics.
6. Unified Agent API owns backend registration, capability discovery, run control, and session extension semantics for the CLI agents.

## Important Boundary Clarification

There are two different API layers in this repository and they should not be conflated.

- External CLI-agent abstraction:
  - `unified-agent-api` from crates.io
  - Default Rust import name in code: `agent_api`
  - Purpose: run and control Codex / Claude Code / other CLI agents through one capability-gated contract
- Substrate-local host/world transport:
  - `crates/agent-api-types`
  - `crates/agent-api-core`
  - `crates/agent-api-client`
  - Purpose: Substrate host components talking to `world-agent`

The local `agent-api-*` crates are not the same thing as Unified Agent API. They currently represent Substrate’s transport layer for host-to-world execution, not the canonical CLI-agent wrapper contract.

Terminology rule for this repository:

- Reserve `Unified Agent API`, `UAA`, and the external Rust import name `agent_api` for the CLI-agent runtime abstraction.
- Refer to `crates/agent-api-types`, `crates/agent-api-core`, and `crates/agent-api-client` as the Substrate-local host/world transport layer or world-agent transport layer.

## Current State Summary

- This is no longer modeling-only.
  - `crates/shell` now depends on `unified-agent-api` (`agent_api`) and the REPL boots a shell-owned UAA runtime in `crates/shell/src/repl/async_repl.rs`.
  - Live orchestration and participant state is persisted by `AgentRuntimeStateStore` under `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/...`, with compatibility snapshots under `~/.substrate/run/agent-hub/{sessions,participants}`.
  - Persistent world sessions now support explicit shared-world owner binding via `SharedWorldOwnerSpec` / `SharedWorldBindingSnapshot`, and the shell invalidates stale world members when `world_generation` rolls forward.
  - Canonical `AgentEvent` production is live for the REPL-owned orchestrator/member runtime, and `substrate agent status` now prefers live runtime state with trace fallback.
- Recent validation tightened several earlier suspected gaps:
  - Linux shared-world replacement ordering and `session.json` durability are already landed in the current backend.
  - Linux world-scoped member runtime placement over the existing host↔world execute-stream transport is now landed, including authoritative world binding validation, replacement-member relaunch, and fail-closed startup semantics.
  - The broad PID-based orchestration-stamping concern is mostly retired from production event emission; the remaining authority cleanup is narrower.
- The gateway runtime is also real for nested LLM/gateway lifecycle work, but it is still a separate runtime path from pure-agent orchestration.
- The main remaining gaps are:
  - there is still no explicit agent-invocation grammar for REPL turns such as `@cli:codex` or `::cli:codex`,
  - `substrate -c` is still shell wrap mode rather than an agent-prompt caller surface,
  - the shell-owned/runtime-owned UAA sessions are currently established through fixed bootstrap prompts, but there is not yet a real user-turn submission path for arbitrary agent prompts,
  - `resume` / `fork` are not surfaced under the `substrate agent` CLI namespace, and there is still no public `start|resume|fork|stop` family,
  - explicit backend/member targeting is still missing from the public caller surface, so world-member selection still depends on “the one eligible member” rather than “the backend the user named,”
  - `substrate agent status` still fails closed on ambiguity/stale parent linkage and its trace-only fallback is not fully participant-aware,
  - and Linux remains the strongest shared-world ownership implementation.

## Gap Matrix

| Area | Status | What exists now | What is missing for the intended v1 path |
|---|---|---|---|
| Product intent | `Now explicit` | This file now records the intended v1 and later product shape | Keep downstream docs aligned to this statement |
| Unified Agent API adoption | `Landed for host orchestrator and Linux world member runtime` | `crates/shell/Cargo.toml` now pulls `unified-agent-api`, `agent_runtime/registry.rs` registers Codex / Claude Code backends, the REPL keeps the host orchestrator under attached control, and Linux world-scoped members now launch through `world-agent` over the existing host↔world transport | Remaining v1 work is no longer basic UAA runtime adoption; it is on invocation semantics, operator-facing controls, status resilience, and broader platform parity |
| UAA vs local `agent-api-*` boundary | `Partially clarified` | Code now distinguishes shell-owned UAA runtime from local host/world transport; `PURE_AGENT_PROTOCOL` comments explicitly say `uaa.agent.session` is a Substrate-local normalized label, not an upstream protocol claim | Local crates are still named `agent-api-*`, and the `uaa.agent.session` label still exists, so naming confusion has been reduced but not removed |
| Agent config and inventory | `Landed` | Inventory schema, capability gates, derived `backend_id`, scope resolution, and backend selection are live in `agent_inventory.rs` and `agent_runtime/validator.rs` | No major v1 inventory gap remains; future work is mostly on runtime breadth rather than selection modeling |
| Agent CLI inspection surface | `Partially landed` | `substrate agent list|status|doctor|toolbox status|env` is live, and `agent status` now merges live runtime-state snapshots with trace fallback instead of relying only on projection from history | The namespace still does not expose first-class `start|resume|fork|stop` actions |
| REPL agent invocation grammar | `Open gap` | The interactive shell recognizes shell directives such as `:host` and `:pty`, and otherwise treats normal REPL input as shell execution | Add an explicit caller grammar for agent turns, including targeted invocation forms such as `@cli:codex` / `::cli:codex`, plus a clear rule for whether any default-agent routing exists |
| Non-interactive agent invocation surface | `Open gap` | `substrate -c` / `--command` is still defined as shell wrap mode and existing tests enforce that contract | Add a first-class non-interactive agent caller surface for one-shot prompts; if `substrate -c` is to participate, its contract must be explicitly redesigned rather than implied |
| Terminology guardrails from the proposed v1 slice | `Partially landed` | Backend-kind mapping for supported UAA agents exists in `agent_runtime/mapping.rs`; runtime selection fails closed for unsupported kinds; `agent_events.rs` documents the local meaning of `uaa.agent.session` | Local `agent-api-*` crate names are still in place, and the local trace/session label has not been renamed |
| Shell-owned UAA runtime | `Landed` | The REPL prepares orchestrator startup, validates allowlisting, builds a shell-owned UAA gateway, starts the orchestrator, retains cancel/event/completion handles, and persists lifecycle transitions in `repl/async_repl.rs` | This is still REPL-owned, not a general hub service or reusable daemonized control plane |
| Live agent session registry | `Landed` | `AgentRuntimeStateStore` persists authoritative parent sessions and participant records under `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json` plus per-session `participants/` and `leases/`; code, docs, and tests are aligned on that contract | Only a future product-surface rename to something like `agent-sessions` remains undecided; there is no current runtime gap here |
| Agent session control plane | `Partially landed` | Internal runtime paths now support attached-control `start`, authoritative persistence, stop/cancel, invalidation, and replacement-member creation across world-generation rollover | `resume` and `fork` are not yet surfaced as Substrate control-plane actions, and there is still no public `substrate agent start|resume|fork|stop` command family |
| User-turn submission into active UAA sessions | `Open gap` | The current shell/world runtimes can establish and retain long-lived attached-control ownership for orchestrator/member sessions | There is not yet a real path that takes an arbitrary user prompt/turn and submits it to the selected active UAA backend while streaming back the result as an operator-facing interaction |
| Session handles | `Partial` | The runtime extracts and persists surfaced UAA session ids (`internal.uaa_session_id`) and correlates them with orchestration/participant metadata; parent sessions also track `active_session_handle_id` | The Substrate registry still mixes local participant ids, orchestration ids, and optional surfaced UAA ids rather than exposing a clean public session-handle contract |
| Host orchestrator process management | `Landed` | A real host-scoped orchestrator backend is launched via UAA, tracked through persisted lifecycle states, and stopped authoritatively on teardown | No major thin-slice blocker remains here |
| Explicit backend/member targeting | `Open gap` | Effective inventory entries already derive canonical `backend_id` values, and runtime selection can reject unsupported kinds | Public invocation still lacks a stable way to target a named backend/member; world-member selection still fails closed on multiple eligible members instead of routing to the backend the user explicitly requested |
| Explicit shared-world ownership | `Mostly landed, Linux-first` | Persistent world startup now sends `SharedWorldOwnerSpec`, world-agent echoes `SharedWorldBindingSnapshot`, Linux session metadata persists `orchestration_session_id` / `world_generation`, shared-world replacement rolls back correctly on failure, and the shell invalidates stale world members after rollover | Linux is still the strongest ownership/reuse implementation; macOS and Windows depend on delegated Lima/WSL behavior rather than equivalent native backend ownership logic |
| Shared-world replacement ordering and world metadata durability | `Landed` | Linux replacement already uses a two-phase `Active -> Replacing -> Replaced` flow with rollback on creation failure, and world `session.json` persistence is atomic-by-rename with failure-preserving behavior | No current correctness gap was confirmed here; only future hardening beyond the current contract would remain |
| In-world member dispatch over existing host↔world transport | `Landed, Linux-first` | The shell now emits typed `member_dispatch` requests over `/v1/execute/stream`, `world-agent` validates the authoritative shared-world binding and launches the member UAA runtime inside the active world, `/v1/execute/cancel` reaches retained in-world control ownership, and Linux tests cover lazy launch, reuse, replacement, and fail-closed startup behavior | Remaining work is broader platform parity and follow-on hardening, not the core placement seam itself |
| Replacement-member semantics across world-generation rollover | `Landed` | Restart/drift handling invalidates stale members, advances `world_generation`, creates a distinct replacement participant with preserved lineage, and relaunches the replacement member through the in-world transport path on Linux | Remaining work is mainly parity and operator-surface breadth rather than replacement correctness |
| Agent event schema and trace flattening | `Landed` | Tuple-aware `AgentEvent` schema is live, runtime events are emitted from orchestrator/member lifecycle code, and status surfaces validate world identity + nested parent correlation | Remaining work is narrower read-side cleanup and producer breadth, not missing support for the now-landed in-world member launch path |
| Event-emission authority plumbing | `Mostly landed` | Production REPL/host/world emitters now require explicit runtime-owned orchestration context before publishing orchestration-scoped `agent_event` rows | Remaining cleanup is narrower: command rows still use synthetic `run_id = cmd_id`, and read-side utilities still keep heuristic recovery helpers such as PID lookup / synthetic parent reconstruction |
| Bootstrap ordering | `Mostly landed` | Parent orchestration session persistence now happens before persisted child/runtime state, and the parent is not marked live until the runtime surfaces a session handle | The remaining nuance is in-memory construction order: the child manifest is still built with the new `orchestration_session_id` before the authoritative parent record is persisted |
| Nested gateway identity split | `Landed` | `substrate agent status` now separates pure-agent sessions from nested gateway-backed rows using live runtime state plus trace correlation | Remaining work here is secondary and mostly depends on broader runtime rollout, not a schema gap |
| Gateway lifecycle | `Landed for nested LLM runtime` | `status|sync|restart` are live end-to-end for the nested gateway lifecycle | This should still not be conflated with pure-agent session orchestration |
| Status ambiguity handling | `Open gap` | The store can already enumerate live sessions and downgrade some bad parent/linkage states to warnings instead of panicking | `substrate agent status` still hard-preflights `resolve_single_live_session_for_agent()` and aborts wholesale on ambiguity, stale handles, or missing parent linkage instead of degrading to render what remains valid |
| Trace-only participant-aware fallback | `Open gap` | Producer-side trace lineage is already participant-aware, and live-runtime status rows preserve `participant_id` correctly | The trace-only fallback path still keys too coarsely by `(orchestration_session_id, agent_id)` and can collapse same-agent sibling participants inside one orchestration session |
| Secret handoff into the world gateway | `Landed` | Host-side policy/auth selection still owns auth-source precedence, `world-agent` now delivers a read-once `GatewayAuthBundleV1` over inherited FD via `SUBSTRATE_LLM_AUTH_BUNDLE_FD`, and `gateway` integrated startup overlays that bundle in memory before provider construction | Remaining work is follow-on hardening and broader parity, not the default carrier: integrated delivery no longer depends on secret-bearing child env vars |
| Toolbox surface | `Partial` | Config, `toolbox status`, `toolbox env`, live-session endpoint derivation, and world-binding projection are present | The internal MCP server, mutation tools, and auth/audit plumbing are still unimplemented |
| Toolbox role in orchestration | `Constrained by design` | ADR-0045 is introspection-only in [ADR-0045](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md:77) | Nothing here should be treated as a substitute for member launch or control-plane execution |
| Custom Substrate harness | `Deferred by intent` | Product intent allows for a future Substrate-native harness layer | Do not block v1 on this; revisit only after CLI-agent orchestration through UAA is working cleanly |

## Still-Open Decisions

These are the decisions that still need to be made to keep the path forward clean.

1. Whether the next shipped slice should stop at the current REPL-owned control plane or promote it into a first-class `substrate agent start|resume|fork|stop` namespace before broader hub work.
- Current repo evidence points toward “the plumbing is mostly there; the missing layer is operator-facing UX and session controls.”
- Session start already exists, but it is implicit: the orchestrator session is bootstrapped automatically by the REPL rather than exposed as a first-class `substrate agent start`.
- Normal REPL input is still treated as shell execution, not “message the default backend agent,” so this decision is really about whether to productize the already-landed runtime seam.

2. What the explicit agent caller syntax should be in interactive and non-interactive surfaces.
- The repo still lacks an explicit syntax contract for targeted agent turns like `@cli:codex` / `::cli:codex`, and `substrate -c` still means shell wrap mode today.
- This decision needs to answer whether agent invocation is explicit-only, whether any default-agent routing exists, and whether `-c` is extended or a separate caller surface is introduced.

3. Whether `uaa.agent.session` should remain the Substrate-local normalized protocol-family label, or be renamed now that real UAA runtime integration is live.
- The recent runtime landing increases the cost of leaving this ambiguous, because the repo now contains both real external `agent_api` usage and local `uaa.*` labeling.
- The discussion above did not resolve this; it only reinforces that the label is still local and still easy to misread as an upstream protocol claim.

4. How aggressively to deconflict the local `agent-api-*` crate names from external `agent_api` before more runtime code accumulates on both sides of that boundary.
- This is closely related to the `uaa.agent.session` decision above, but it is a crate/module/package naming question rather than a protocol/identity-label question.
- The recent runtime landing makes this more pressing from a governance/clarity standpoint, even though it is not a direct runtime-correctness blocker.

5. Whether to stop at the newly landed hidden shell↔world-agent helper over the existing execute/stream transport, or continue on to a broader reusable world-owned UAA service surface.
- The current repo state has now landed the thinner option first: host orchestrator on the shell, world member launched through the existing world transport seam, without building a broad new hub service up front.
- The remaining question is product breadth and reuse, not whether member placement can work through the existing transport seam.

6. Whether explicit backend targeting should be mandatory once multiple eligible world members exist.
- The current member-selection logic still assumes “pick the one eligible world member or fail closed.”
- Productizing targeted invocation means deciding whether named `backend_id` selection becomes mandatory whenever multiple members are configured, or whether a default-selection layer is introduced.

7. Whether and when to retire the remaining flat compatibility outputs after the `agent-hub` cutover is considered complete.
- Code and docs are already aligned that compatibility dual-writes are still intentional today.
- This is now a retirement-timing decision, not a current ambiguity about what the runtime is supposed to do.

8. How much cross-platform parity is required before calling the v1 slice complete, given that Linux currently has the clearest shared-world ownership semantics.
- The current implementation evidence still says Linux is the strongest and clearest slice for shared-world ownership and replacement behavior.
- The discussion above did not answer this, but it reinforces that the host-orchestrator / world-member model is already the enforced placement rule; the open question is the parity bar for non-Linux backends.

## Recommended v1 Runtime Slice

The thin-slice recommendation is still correct, but the remaining scope is now much narrower than when this file was first written:

1. Keep the existing shell-owned orchestrator runtime and live state registry.
2. Reuse the existing shared-world owner binding model and world-generation invalidation rules.
3. Keep the newly landed Linux world-owned member launch over the existing host↔world execute-stream seam as the v1 baseline.
4. Add an explicit agent-caller grammar for REPL and non-interactive usage, including targeted backend syntax rather than overloading shell execution implicitly.
5. Add a real user-turn submission seam that can send arbitrary prompts/turns into the selected active UAA backend and stream the result back to the operator.
6. Surface `start`, `resume`, `fork`, and `stop` under `substrate agent` with a clean public session-handle contract.
7. Make `substrate agent status` degrade cleanly on ambiguity/stale parent linkage and make its trace fallback participant-aware.
8. Keep the landed auth-bundle handoff as the default integrated carrier so nested/in-world gateway work does not regress back to secret-bearing child env vars.

At this point, the missing work is caller-surface design, user-turn dispatch, control-plane/status hardening, and parity decisions, not basic UAA adoption, session persistence, event modeling, gateway secret-carrier honesty, or Linux member-runtime placement.
