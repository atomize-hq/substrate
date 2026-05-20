# SOW A: Complete Explicit Backend Selection by Backend ID

Status: implementation-oriented follow-on draft. This SOW does not introduce backend-id targeting from scratch. The repo already has a partially landed exact-backend path for targeted REPL follow-up turns through `validate_exact_backend_selection(...)` in [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:244) and `resolve_targeted_turn_route(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2702). This document isolates the remaining work needed to harden and complete that model beyond the currently landed targeted-turn route: align retained-member reuse to `backend_id`, remove leftover singleton assumptions in both the shell and `world-agent`, keep fail-closed behavior precise, and prove that multiple eligible world members can coexist as simultaneously retained live runtimes without ambiguity.

## Objective

Complete the already-landed explicit backend-targeting model for Linux-first member runtime orchestration by:

- hardening inventory lookup and fail-closed exact selection around derived `backend_id`,
- moving retained world-member reuse from leftover singleton or `agent_id`-based heuristics to `backend_id`,
- ensuring launch and reuse paths consistently target the named backend beyond the current targeted-turn route,
- and adding regression coverage showing that multiple eligible world members in one orchestration session/world generation can coexist as simultaneously retained live runtimes and still be selected unambiguously by backend id.

This SOW is about finishing and hardening backend identity and retained-runtime semantics. It is not the initial caller-grammar landing, and it is not the broader prompt-submission product slice.

## Why This Is Needed

The repo already has the core selector for explicit backend targeting, but that selector is only fully exercised along the currently landed targeted-turn path. The remaining implementation still spans two different eras:

- inventory entries derive canonical backend ids such as `cli:codex` in [agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs)
- exact backend lookup already exists in [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs), alongside the older singleton-style `validate_member_selection(...)`
- REPL targeted-turn routing already uses exact backend selection in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- member dispatch and follow-up submit requests already carry `backend_id` over the shell<->world transport in [lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs) and [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- Linux world-member follow-up validation already checks that a submitted turn’s `backend_id` matches the retained runtime in [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)

What is still inconsistent:

- the gap matrix still records explicit backend targeting as only partially landed, with broader productization still open in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
- older selection logic still exists for non-exact/member-singleton flows and still fails on ambiguity in [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- the shell REPL still models world-member retention as a single slot via `let mut member_runtime: Option<AsyncReplAgentRuntime> = None` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:553), and readiness/reuse still flows through that singleton slot in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3577)
- live-member reuse in the shell path still keys off `agent_id` plus `world_generation` in `live_member_for_generation(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2837), which is narrower and less precise than "reuse the retained runtime for backend `cli:codex`"
- `world-agent` still enforces a single retained world member globally via `active_members_by_participant_id` and rejects coexistence outright in [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:391)

That leaves the repo with a real exact-backend targeted-turn route, but not yet a fully coherent backend-id ownership and reuse model across simultaneously retained world members. This slice closes that narrower gap.

## Relationship To Existing Slices

- [15-targeted-repl-agent-turns-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/15-targeted-repl-agent-turns-linux-first.md) is the parent combined SOW. This document carves out its backend-selection and retained-member identity work as SOW A.
- This SOW depends on the already-landed Linux member-runtime placement and world dispatch plumbing referenced in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md).
- A later SOW can focus on caller grammar, prompt submission UX, and non-interactive surfaces. Those are intentionally not reopened here.

## Current Relevant Code Surfaces

### Inventory identity and derived backend ids

- [crates/shell/src/execution/agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs)
  - `AgentFileV1::derived_backend_id()`
  - `AgentInventoryEntryV1::derived_backend_id()`
  - effective inventory loading and validation entry points

### Selection, scope checks, and currently landed exact targeting

- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
  - `validate_runtime_realizability(...)`
  - `validate_member_selection(...)`
  - `validate_exact_backend_selection(...)`
  - `RuntimeSelectionDescriptor { backend_id, execution_scope, binary_path, ... }`

### REPL runtime selection and world-member launch

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:553)
  - single retained member slot: `member_runtime: Option<AsyncReplAgentRuntime>`
  - targeted route resolution by backend id in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2702)
  - `live_member_for_generation(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2837)
  - singleton readiness/reuse path in `ensure_member_runtime_ready_for_descriptor(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3577)
  - Linux follow-up submission via `submit_world_targeted_turn(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3891)

### Member dispatch transport contract

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
  - `MemberDispatchRequestV1`
  - `MemberTurnSubmitRequestV1`
  - `ResolvedMemberRuntimeDescriptorV1`

- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - `MemberDispatchTransportRequest`
  - request building for member dispatch payloads

### Linux retained member ownership and backend checks

- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
  - retained member registry
  - submit-turn request validation against retained identity
  - single-retained-member constraint that currently blocks coexistence

## In Scope

- hardening inventory lookup by derived `backend_id` for explicit backend-targeted runtime paths
- completing exact backend selection rollout beyond the currently landed targeted-turn route
- fail-closed behavior for:
  - missing backend
  - duplicate effective inventory matches for the same `backend_id`
  - wrong execution scope
  - disallowed backend under `agents.allowed_backends`
  - runtime-unrealizable backend selections
- retained-member reuse keyed by backend identity rather than singleton member heuristics
- launch-path changes needed so the named backend is what gets started or reused
- Linux-first shell and `world-agent` cardinality changes needed to allow multiple retained eligible members to coexist as simultaneously retained live runtimes
- focused regression tests for ambiguity removal and coexistence

## Out Of Scope

- changing REPL grammar
- redesigning `substrate -c`
- broader `substrate agent start|resume|fork|stop` productization
- macOS or Windows parity
- transcript UX or output formatting changes
- gateway auth-carrier work
- toolbox work
- replacing the host<->world transport schema with a new service family

## Required Invariants

### 1. `backend_id` remains the canonical selector

All explicit backend-targeting paths must continue to resolve by the effective inventory entry’s derived `backend_id`, not by `agent_id`, binary path, backend kind alone, or "the only eligible member."

### 2. Exact selection must fail closed

If a named backend is absent, duplicated, disabled, wrong-scope, not policy-allowed, missing required protocol/capabilities, or not runtime-realizable, the path must stop with a concrete error. There must be no fallback to another backend and no fallback to legacy singleton selection.

### 3. Scope is part of the contract

The same `backend_id` request must only match entries in the requested scope. A host-targeted path cannot silently select a world-scoped runtime, and a world-targeted path cannot silently select a host runtime.

### 4. Retained world-member reuse must key by backend id

Within one orchestration session and authoritative world generation, retained runtime lookup must be by the named backend. Reuse must mean "reuse the retained runtime for `cli:codex`" rather than "reuse whichever member exists."

### 5. Multiple eligible world members must be able to coexist

For this SOW, coexistence means more than inventory eligibility. If effective inventory contains more than one valid world-scoped backend, the system must be able to retain and reuse distinct live runtimes for those backends simultaneously within the same orchestration session and authoritative world generation without ambiguity.

### 6. Transport identity must stay end-to-end

`backend_id` must remain explicit in member dispatch, retained-runtime state, follow-up submit requests, and validation of those follow-up requests. World-member submission must never target a retained runtime whose backend identity does not exactly match the request.

## Recommended Implementation Shape

1. Keep `validate_exact_backend_selection(...)` in [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) as the canonical selector for explicit backend-targeted paths, and demote `validate_member_selection(...)` to legacy/non-targeted use only.
2. Normalize inventory lookup around derived `backend_id` so all explicit runtime callers consume one selector contract and one family of errors.
3. Replace shell-side singleton reuse and readiness in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:553) and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3577) with backend-aware retained-runtime management keyed by:
   - `orchestration_session_id`
   - `world_generation`
   - `backend_id`
4. Widen the shell REPL and Linux `world-agent` retained-member cardinality together:
   - the shell can no longer model retained world members as one `Option<AsyncReplAgentRuntime>`
   - `world-agent` can no longer model retained world members as a global singleton registry
   - both sides must support backend-aware lookup and validation for more than one simultaneously retained live member
5. Keep follow-up submit validation strict: the submitted turn must still match the retained participant, orchestrator participant, world binding, and `backend_id`.

## Concrete Work Breakdown

### A1. Harden the canonical exact-backend selector

- Audit explicit backend-targeted paths in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) and adjacent runtime helpers so the already-landed exact selector stays authoritative anywhere a named backend is supplied.
- Ensure error wording distinguishes:
  - backend missing from effective inventory
  - backend present but wrong-scope
  - backend present but disallowed by policy
  - backend present but unrealizable
  - backend duplicated across effective entries in the same scope

### A2. Retained-member lookup and reuse by backend id

- Replace `live_member_for_generation(...)` semantics in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2837) so lookup is keyed by `backend_id` plus authoritative world generation, not by `agent_id` alone.
- Preserve fail-closed behavior if more than one live participant claims the same backend key for the same orchestration session and world generation.

### A3. Replace shell-side singleton retained-member state

- Replace `member_runtime: Option<AsyncReplAgentRuntime>` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:553) with a backend-aware retained-member collection that can hold more than one live world member at once.
- Refactor `ensure_member_runtime_ready_for_descriptor(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3577) and `reconcile_member_runtime_generation(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3527) so readiness, invalidation, replacement, and reuse operate on the targeted backend entry instead of a single global member slot.
- Ensure member startup preparation consumes the exact selected `RuntimeSelectionDescriptor`.
- Keep policy allowlist enforcement against `descriptor.backend_id`.

### A4. Replace `world-agent` singleton retained-member state

- Refactor [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:391) so retained runtime ownership is not globally singleton.
- Add a backend-aware retained lookup that supports coexistence of `cli:codex` and `cli:claude-code` in the same live world.
- Retain strict duplicate detection for the same backend key.

### A5. Regression coverage

- Extend validator tests in [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) for exact-backend failure cases and scope-specific errors, without regressing the already-landed exact-match path.
- Add REPL/runtime tests in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) that prove:
  - multiple eligible world members can be retained live simultaneously within one orchestration session/world generation without ambiguity
  - explicit launch for `cli:codex` reuses only the retained `cli:codex` runtime
  - explicit launch for `cli:claude-code` reuses only the retained `cli:claude-code` runtime and does not collide with `cli:codex`
  - duplicate live retained entries for the same backend still fail closed
- Add Linux `world-agent` tests in [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) covering multi-member retention and backend-id submit validation.

## Validation Expectations

At minimum, this slice should ship with:

- `cargo test -p shell validate_exact_backend_selection`
- targeted REPL/runtime tests covering backend-id keyed reuse and coexistence
- `cargo test -p world-agent member_runtime`

Expected assertions:

- the currently landed exact targeted-turn route continues to resolve `cli:codex` and `cli:claude-code` by backend id
- inventory with both `cli:codex` and `cli:claude-code` world members no longer fails just because more than one eligible member exists in adjacent launch/reuse paths
- both backends can remain simultaneously retained live runtimes within one orchestration session/world generation, and each targeted launch/reuse path selects the correct retained runtime by `backend_id`
- explicit backend targeting still fails closed when the named backend is absent or wrong-scope
- explicit backend targeting still fails closed when policy disallows the selected backend
- retained member follow-up submission is rejected if the request `backend_id` does not match the retained runtime
- two simultaneously retained live members with distinct backend ids can coexist; two retained members with the same backend key cannot

## Explicit Non-Goals

- Do not introduce implicit default-agent routing.
- Do not add a backend aliasing layer beyond existing derived `backend_id` rules.
- Do not relax policy gating in order to make backend selection more permissive.
- Do not broaden this slice into session-resume UX, transcript modeling, or non-interactive agent callers.

## Open Risks

- The repo already has partially landed exact-backend behavior, so the main risk is leaving mixed identity models in place: exact selection in the targeted-turn route, but singleton or `agent_id`-based reuse deeper in the runtime.
- The largest behavioral shift in this slice is cardinality on both sides of the seam: the shell REPL no longer holding one member runtime slot, and `world-agent` no longer holding one retained member globally.
- If any persisted participant records or live-state helpers implicitly assume one world member at a time, those assumptions will need to be surfaced and tested before the slice is considered closed.
