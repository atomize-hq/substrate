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
  - and shared-world reuse for retained member sessions keyed by exact `backend_id`.

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
  - Host orchestration sessions now persist explicit durable posture and attachment truth: `posture`, `attached_participant_id`, `pending_inbox_count`, and per-session durable inbox artifacts under `sessions/<orchestration_session_id>/inbox/<item_id>.json`.
  - `parked_resumable` and `awaiting_attention` are retained durable host postures, not gone-session aliases; `terminal` remains the only non-routable posture family.
  - Persistent world sessions now support explicit shared-world owner binding via `SharedWorldOwnerSpec` / `SharedWorldBindingSnapshot`, and the shell invalidates stale world members when `world_generation` rolls forward.
  - Canonical `AgentEvent` production is live for the REPL-owned orchestrator/member runtime, and `substrate agent status` now prefers live runtime state with trace fallback while surfacing parked and attention-needed durable host sessions on the read path.
  - The narrow public control plane is now live under `substrate agent start|turn|reattach|fork|stop`, with exact `--backend <backend_id>` / `--session <orchestration_session_id>` selectors, one-of prompt-source validation for `start` and `turn`, helper-owned streaming NDJSON for prompt-taking calls, authoritative completion-time `session_posture`, hidden-owner launch handoff, exact-session `reattach` recovery to durable attached truth, owner-transport stop routing as the canonical closeout path for attached and parked durable sessions, and explicit `Completed`/`Failed` terminal delivery after `Accepted`.
- Recent validation tightened several earlier suspected gaps:
  - Linux shared-world replacement ordering and `session.json` durability are already landed in the current backend.
  - Linux world-scoped member runtime placement over the existing host↔world execute-stream transport is now landed, including authoritative world binding validation, replacement-member relaunch, fail-closed startup semantics, and retained live-member coexistence for distinct backend ids such as `cli:codex` and `cli:claude_code` within one orchestration session / world generation.
  - The selected-follow-up contract is now explicit and regression-proven for the Linux-first / REPL-first surface: exact `::<backend_id> <prompt>` targeting stays fail-closed, host follow-up turns resume only the active orchestrator backend, same-generation world follow-up turns reuse the exact retained member, and stale world state relaunches the exact backend slot before typed `/v1/member_turn/stream` submit.
  - The broad PID-based orchestration-stamping concern is mostly retired from production event emission; the remaining authority cleanup is narrower.
- The gateway runtime is also real for nested LLM/gateway lifecycle work, but it is still a separate runtime path from pure-agent orchestration.
- The main remaining gaps are:
  - `substrate -c` is still shell wrap mode rather than an agent-prompt caller surface,
  - the shipped public control plane is intentionally narrow: root `start` is host-only, existing-session prompt-taking accepts only exact `orchestration_session_id` plus exact `backend_id`, there is still no default-agent routing, and there is still no public world-root start,
  - exact targeted turns now exist on both narrow caller surfaces, but they stay intentionally strict: the REPL keeps exact `::<backend_id> <prompt>`, public follow-up keeps exact `(orchestration_session_id, backend_id)`, detached world follow-up still requires `reattach`, and there is still no broader member-selector surface,
  - broader operator-facing routing policy outside those exact caller paths is still unsettled; there is still no default-agent surface and no broader fuzzy non-REPL targeting contract,
  - `substrate agent status` now degrades torn parent/session linkage into warnings on the read surface, but strict control-plane surfaces such as `toolbox status` / `toolbox env` still fail closed and the trace-only fallback remains coarse when the rows omit `participant_id`,
  - macOS/Lima now uses the same shared-owner/member-runtime backend seam as Linux for the supported forwarded path, with regression coverage for shared-owner proof, member dispatch, targeted follow-up reuse, and guest-owned cancel.
  - Linux remains the source-of-truth ownership implementation and still has the broadest platform maturity; Windows/WSL remains fail-closed outside the supported contract.

## Gap Matrix

| Area | Status | What exists now | What is missing for the intended v1 path |
|---|---|---|---|
| Product intent | `Now explicit` | This file now records the intended v1 and later product shape | Keep downstream docs aligned to this statement |
| Unified Agent API adoption | `Landed for host orchestrator and supported Linux/macOS world member runtime` | `crates/shell/Cargo.toml` now pulls `unified-agent-api`, `agent_runtime/registry.rs` registers Codex / Claude Code backends, the REPL keeps the host orchestrator under attached control, and supported world-scoped members now launch through `world-agent` over the existing host↔world transport on Linux and the macOS/Lima forwarded path | Remaining v1 work is no longer basic UAA runtime adoption; it is on operator-facing controls, status resilience, Windows/WSL posture, and broader caller-surface productization |
| UAA vs local `agent-api-*` boundary | `Partially clarified` | Code now distinguishes shell-owned UAA runtime from local host/world transport; `PURE_AGENT_PROTOCOL` comments explicitly say `uaa.agent.session` is a Substrate-local normalized label, not an upstream protocol claim | Local crates are still named `agent-api-*`, and the `uaa.agent.session` label still exists, so naming confusion has been reduced but not removed |
| Agent config and inventory | `Landed` | Inventory schema, capability gates, derived `backend_id`, scope resolution, and backend selection are live in `agent_inventory.rs` and `agent_runtime/validator.rs` | No major v1 inventory gap remains; future work is mostly on runtime breadth rather than selection modeling |
| Agent CLI inspection surface | `Landed for inspection plus narrow public control` | `substrate agent list|status|doctor|toolbox status|env` is live, `agent status` now merges live runtime-state snapshots with trace fallback and surfaces durable `parked_resumable` / `awaiting_attention` host sessions, and the namespace now exposes first-class `start|turn|reattach|fork|stop` actions with exact selectors plus one-of prompt-source validation for `start` and `turn` | Remaining gaps are broader productization only: there is still no default-agent routing, no member-level public selector surface beyond exact `(orchestration_session_id, backend_id)`, and no public world-root start |
| REPL agent invocation grammar | `Landed, Linux-first and REPL-first` | The interactive shell now accepts exact single-line targeted follow-up turns as `::<backend_id> <prompt>`, routes them before shell fallback, preserves `:host` / `:pty`, and continues to treat plain REPL input as shell execution | Remaining work is broader product-surface breadth only: no default-agent routing exists, and non-REPL surfaces still need their own explicit caller contract |
| Non-interactive agent invocation surface | `Landed narrowly` | `substrate agent start --backend <backend_id> (--prompt <text> | --prompt-file <path> | --prompt-file -) [--json]` and `substrate agent turn --session <orchestration_session_id> --backend <backend_id> ...` now provide a real public prompt-taking caller surface with helper-owned streaming, completion-time `session_posture`, explicit fail-closed exact selector coverage, Linux world-member follow-up proof through typed `/v1/member_turn/stream`, and an explicit non-regression guard that keeps `substrate -c` / `--command` in shell-wrap mode | Remaining gaps are broader caller-surface breadth and parity only: there is still no default-agent routing, no public world-root start, and Windows/WSL world-sensitive follow-up still fails closed |
| Terminology guardrails from the proposed v1 slice | `Partially landed` | Backend-kind mapping for supported UAA agents exists in `agent_runtime/mapping.rs`; runtime selection fails closed for unsupported kinds; `agent_events.rs` documents the local meaning of `uaa.agent.session` | Local `agent-api-*` crate names are still in place, and the local trace/session label has not been renamed |
| Shell-owned UAA runtime | `Landed` | The REPL prepares orchestrator startup, validates allowlisting, builds a shell-owned UAA gateway, starts the orchestrator, retains cancel/event/completion handles, and persists lifecycle transitions in `repl/async_repl.rs` | This is still REPL-owned, not a general hub service or reusable daemonized control plane |
| Live agent session registry | `Landed` | `AgentRuntimeStateStore` persists authoritative parent sessions and participant records under `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json` plus per-session `participants/`, `leases/`, and durable `inbox/` artifacts; host sessions now persist explicit posture/attachment truth instead of inferring validity from attached-control diagnostics alone | Only a future product-surface rename to something like `agent-sessions` remains undecided; there is no current runtime gap here |
| Agent session control plane | `Landed for narrow public control, Linux-source with supported macOS/Lima world-sensitive parity` | Internal runtime paths now support attached-control `start`, public `reattach` / `fork`, prompt-taking `start` / `turn`, authoritative persistence, clean host detach parking into `parked_resumable`, pending-inbox escalation into `awaiting_attention`, exact-session `reattach` recovery back to durable attached truth, `stop`/cancel as the canonical closeout path for attached and parked durable sessions, replacement-member creation across world-generation rollover, the hidden owner-helper launch seam, the same private stop-owner plane for helper-owned and REPL-owned sessions, and explicit post-`Accepted` terminal envelopes | Remaining gaps are broader surface breadth and parity: no public world-root start, no member-level public selectors beyond exact session/backend targeting, and Windows/WSL world-sensitive control posture still fails closed |
| User-turn submission into active UAA sessions | `Landed for REPL and narrow public caller surfaces on Linux plus supported macOS/Lima` | The REPL can now submit arbitrary targeted follow-up turns into retained active sessions, and the public `substrate agent start|turn` surface can now submit exact prompts with helper-owned streaming: parked host follow-up turns resume the exact authoritative orchestrator session, supported world follow-up turns go through the typed `/v1/member_turn/stream` route using the persisted retained-member tuple (`participant_id`, `orchestrator_participant_id`, `backend_id`, `world_id`, `world_generation`), reusing the exact retained backend slot when current and relaunching that exact slot when stale, and detached-world follow-up remains fail-closed with `reattach` guidance | Remaining work is broader operator-surface breadth and platform parity. Public root `start` remains host-only, default-agent routing still does not exist, and Windows/WSL world follow-up still fails closed |
| Session handles | `Partial` | The runtime extracts and persists surfaced UAA session ids (`internal.uaa_session_id`) and correlates them with orchestration/participant metadata; parent sessions also track `active_session_handle_id` | The Substrate registry still mixes local participant ids, orchestration ids, and optional surfaced UAA ids rather than exposing a clean public session-handle contract |
| Host orchestrator process management | `Landed` | A real host-scoped orchestrator backend is launched via UAA, tracked through persisted lifecycle states, and stopped authoritatively on teardown | No major thin-slice blocker remains here |
| Explicit backend/member targeting | `Landed for the exact REPL and narrow public caller surfaces` | Effective inventory entries derive canonical `backend_id` values, targeted REPL follow-up turns route by exact `backend_id`, public `turn` requires exact `(orchestration_session_id, backend_id)` routing, host follow-up targeting fails closed unless it names the active orchestrator backend, Linux world reuse/relaunch is keyed by exact `backend_id` rather than `agent_id`, distinct retained members such as `cli:codex` and `cli:claude_code` can coexist in one orchestration session / world generation, and duplicate retained members for the same `backend_id` still fail closed | Remaining work is broader productization rather than runtime correctness: there is still no default-agent routing, and policy/default-selection decisions outside these exact caller paths are still open |
| Explicit shared-world ownership | `Landed on Linux and supported macOS/Lima; Linux remains source-of-truth` | Persistent world startup now sends `SharedWorldOwnerSpec`, world-agent echoes `SharedWorldBindingSnapshot`, session metadata persists `orchestration_session_id` / `world_generation`, shared-world replacement rolls back correctly on failure, and the shell invalidates stale world members after rollover on Linux plus the supported macOS/Lima forwarded path | Remaining work is broader platform scope and operator maturity, not missing shared-owner proof on the supported macOS/Lima path. Windows/WSL still does not implement this contract |
| Shared-world replacement ordering and world metadata durability | `Landed` | Linux replacement already uses a two-phase `Active -> Replacing -> Replaced` flow with rollback on creation failure, and world `session.json` persistence is atomic-by-rename with failure-preserving behavior | No current correctness gap was confirmed here; only future hardening beyond the current contract would remain |
| In-world member dispatch over existing host↔world transport | `Landed on Linux and supported macOS/Lima` | The shell now emits typed `member_dispatch` requests over `/v1/execute/stream`, `world-agent` validates the authoritative shared-world binding and launches the member UAA runtime inside the active world, `/v1/execute/cancel` reaches retained in-world control ownership, and regression tests cover lazy launch, reuse, replacement, and fail-closed startup behavior on Linux plus the supported macOS/Lima path | Remaining work is broader platform scope and follow-on hardening, not missing member-dispatch transport on the supported macOS/Lima path |
| macOS/Lima parity for agent orchestration | `Landed in the supported Lima-backed posture` | `world-mac-lima` now forwards the same shared-owner/member-runtime contract as Linux for the supported backend-detected path: `SharedWorldOwnerSpec` reaches the guest, `SharedWorldBindingSnapshot` returns through `WorldHandle.shared_binding`, `member_dispatch` and typed `/v1/member_turn/stream` follow-up are preserved, and cancel remains guest-owned through `/v1/execute/cancel` | Remaining work is outside this slice: Windows/WSL parity, broader public caller-surface breadth, and any future live-guest coverage beyond the documented macOS/Lima smoke plus regression suite |
| Replacement-member semantics across world-generation rollover | `Landed` | Restart/drift handling invalidates stale members, advances `world_generation`, creates a distinct replacement participant with preserved lineage, and relaunches the replacement member through the in-world transport path on Linux | Remaining work is mainly parity and operator-surface breadth rather than replacement correctness |
| Agent event schema and trace flattening | `Landed` | Tuple-aware `AgentEvent` schema is live, runtime events are emitted from orchestrator/member lifecycle code, and status surfaces validate world identity + nested parent correlation | Remaining work is narrower read-side cleanup and producer breadth, not missing support for the now-landed in-world member launch path |
| Event-emission authority plumbing | `Mostly landed` | Production REPL/host/world emitters now require explicit runtime-owned orchestration context before publishing orchestration-scoped `agent_event` rows | Remaining cleanup is narrower: command rows still use synthetic `run_id = cmd_id`, and read-side utilities still keep heuristic recovery helpers such as PID lookup / synthetic parent reconstruction |
| Bootstrap ordering | `Mostly landed` | Parent orchestration session persistence now happens before persisted child/runtime state, and the parent is not marked live until the runtime surfaces a session handle | The remaining nuance is in-memory construction order: the child manifest is still built with the new `orchestration_session_id` before the authoritative parent record is persisted |
| Nested gateway identity split | `Landed` | `substrate agent status` now separates pure-agent sessions from nested gateway-backed rows using live runtime state plus trace correlation | Remaining work here is secondary and mostly depends on broader runtime rollout, not a schema gap |
| Gateway lifecycle | `Landed for nested LLM runtime` | `status|sync|restart` are live end-to-end for the nested gateway lifecycle | This should still not be conflated with pure-agent session orchestration |
| Status ambiguity handling | `Partially landed` | `substrate agent status` now keeps valid rows visible while surfacing degraded warnings for torn parent/session state such as missing `active_session_handle_id`, missing parent-session metadata, and incomplete selected participants; durable parked and attention-needed host sessions remain visible on the read surface instead of collapsing into terminal absence | Strict selectors still fail closed by design on the control-plane surfaces (`toolbox status`, `toolbox env`, doctor/member selection), and status still needs a clearer operator-facing ambiguity policy for participant-less trace-only rows |
| Trace-only participant-aware fallback | `Partially landed` | Trace rows that carry `participant_id` now stay sibling-distinct, sibling-specific suppression is keyed by `participant_id`, and nested correlation honors `parent_participant_id` when same-agent siblings coexist in one `orchestration_session_id` | Participant-less trace rows still warn and fall back to coarse matching, so the fallback is not yet fully participant-aware unless the trace carries `participant_id` / `parent_participant_id` |
| Secret handoff into the world gateway | `Landed` | Host-side policy/auth selection still owns auth-source precedence, `world-agent` now delivers a read-once `GatewayAuthBundleV1` over inherited FD via `SUBSTRATE_LLM_AUTH_BUNDLE_FD`, and `gateway` integrated startup overlays that bundle in memory before provider construction | Remaining work is follow-on hardening and broader parity, not the default carrier: integrated delivery no longer depends on secret-bearing child env vars |
| Toolbox surface | `Partial` | Config, `toolbox status`, `toolbox env`, live-session endpoint derivation, and world-binding projection are present | The internal MCP server, mutation tools, and auth/audit plumbing are still unimplemented |
| Toolbox role in orchestration | `Constrained by design` | ADR-0045 is introspection-only in [ADR-0045](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md:77) | Nothing here should be treated as a substitute for member launch or control-plane execution |
| Custom Substrate harness | `Deferred by intent` | Product intent allows for a future Substrate-native harness layer | Do not block v1 on this; revisit only after CLI-agent orchestration through UAA is working cleanly |

## Still-Open Decisions

These are the decisions that still need to be made to keep the path forward clean.

1. How far to widen the now-shipped `substrate agent start|turn|reattach|fork|stop` namespace before broader hub work.
- Current repo evidence points toward “keep the shipped namespace narrow until non-interactive caller semantics are frozen.”
- Session start is now explicit for host orchestrators, but plain REPL input is still treated as shell execution rather than “message the default backend agent.”
- The open question is how much additional caller breadth belongs beside this first-class control surface before a broader hub/service story exists.

2. What the explicit agent caller syntax should be in interactive and non-interactive surfaces.
- The REPL now has an explicit targeted-turn contract: `::<backend_id> <prompt>` on a single line, with no default-agent routing and with plain REPL input still reserved for shell execution.
- The remaining decision is how that explicit caller contract should extend beyond the REPL, because `substrate -c` still means shell wrap mode today and any non-interactive agent caller must be introduced deliberately rather than implied.

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
- The current explicit REPL caller path already requires named `backend_id` selection and supports multiple retained live world members when the backend ids are distinct.
- Productizing targeted invocation beyond the current REPL path still means deciding whether named `backend_id` selection becomes mandatory anywhere multiple eligible members may exist outside this exact caller path, or whether a default-selection layer is introduced elsewhere.

7. Whether and when to retire the remaining flat compatibility outputs after the `agent-hub` cutover is considered complete.
- Code and docs are already aligned that compatibility dual-writes are still intentional today.
- This is now a retirement-timing decision, not a current ambiguity about what the runtime is supposed to do.

8. How much cross-platform parity is required before calling the v1 slice complete, given that Linux currently has the clearest shared-world ownership semantics.
- The current implementation evidence still says Linux is the strongest and clearest slice for shared-world ownership and replacement behavior.
- macOS/Lima is now on the supported shared-owner/member-runtime path through the forwarded backend seam; the next parity question is Windows/WSL or any broader non-Linux backend model, not whether Lima needs a different orchestration design.
- The open question is the parity bar for non-Linux backends after that, not whether the host-orchestrator / world-member model itself is still the intended placement rule.

## Recommended v1 Runtime Slice

The thin-slice recommendation is still correct, but the remaining scope is now much narrower than when this file was first written:

1. Keep the existing shell-owned orchestrator runtime and live state registry.
2. Reuse the existing shared-world owner binding model and world-generation invalidation rules.
3. Keep the newly landed Linux world-owned member launch over the existing host↔world execute-stream seam as the v1 baseline.
4. Keep the exact REPL targeted-turn grammar `::<backend_id> <prompt>` as the explicit Linux-first / REPL-first caller surface rather than redesigning the grammar or adding implicit default-agent routing.
5. Extend the already-landed user-turn submission seam beyond the current REPL-first surface only through deliberate new caller contracts, not by changing `substrate -c` semantics implicitly.
6. Keep `start`, `turn`, `reattach`, `fork`, and `stop` under `substrate agent` as the clean public session-handle contract, with `reattach` reserved for exact-session attached-owner recovery and `stop` as the canonical closeout path for attached or parked durable sessions.
7. Keep `substrate agent status` on the current split contract: surface durable `parked_resumable` and `awaiting_attention` sessions on the read surface, degrade torn parent/session linkage into warnings, keep control-plane selectors fail-closed, and finish the remaining participant-less trace fallback hardening.
8. Keep the landed auth-bundle handoff as the default integrated carrier so nested/in-world gateway work does not regress back to secret-bearing child env vars.
9. Keep macOS/Lima on the same shared-owner/member-runtime path as Linux while deciding whether any future non-Linux backend should meet that same bar or stay fail-closed.

At this point, the missing work is broader caller-surface productization, control-plane/status hardening, and parity decisions, not basic UAA adoption, REPL targeted-turn grammar, REPL user-turn dispatch, session persistence, event modeling, gateway secret-carrier honesty, or Linux member-runtime placement.
