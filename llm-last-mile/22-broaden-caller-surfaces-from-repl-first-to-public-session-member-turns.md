# SOW: Broaden Caller Surfaces From REPL-First To Public Session/Member Turns

Status: implementation-oriented draft. This slice is not about inventing a new prompt surface. The thin v1 path is already real in two places: exact REPL targeted turns via `::<backend_id> <prompt>` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:5124), and public prompt-taking via `substrate agent start` / `substrate agent turn` in [cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:520) and [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:301). The remaining work is to harden and explicitly validate the public runtime path for session/member turns, especially the Linux-first world-member follow-up path.

## Objective

Make `substrate agent start` and `substrate agent turn` the deliberately validated v1 public caller surfaces for prompt-taking orchestration by:

- validating the end-to-end public runtime path from CLI parsing to prompt streaming to retained target resolution to owner transport or world-member submit,
- locking exact `(orchestration_session_id, backend_id)` routing and fail-closed behavior on the public turn path,
- proving the retained world-member follow-up contract from public selector resolution to typed `MemberTurnSubmitRequestV1` submission into `world-agent`,
- and keeping recovery posture explicit: detached host follow-up may recover through owner reattach, detached world follow-up must fail closed until `substrate agent reattach --session ...` restores an active host owner.

## Why This Is Needed

The public surface exists, but the behavior boundary is spread across several layers and the missing work is mostly runtime hardening and coverage, not invention:

- `run_turn(...)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:324) loads prompt input, resolves the public target, branches on active versus detached posture, performs detached-host recovery, and explicitly rejects detached-world follow-up.
- `resolve_public_turn_target(...)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711) enforces exact `orchestration_session_id` plus exact `backend_id`, rejects noncanonical selectors, and resolves one authoritative retained target slot.
- `run_public_prompt_command(...)` in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1121) revalidates active posture before private prompt transport submission and is therefore part of the public “must be live now” contract.
- Linux world-member follow-up is not just “session plus backend.” The shell resolves that public selector into retained-member identity and submits a typed `MemberTurnSubmitRequestV1` carrying `participant_id`, `orchestrator_participant_id`, `backend_id`, `world_id`, and `world_generation`; `world-agent` validates and resumes the retained member through `submit_member_turn_stream(...)` in [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1485) and `submit_turn(...)` in [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:249).
- Coverage already exists for public `start`, public `turn`, selector rejection, and `-c` preservation in [agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:674), but the public world-member turn path and several fail-closed cases still need explicit end-to-end tests.

## Current Repo Truth

- `substrate agent start --backend <backend_id> ...` is already public and host-only in v1. `run_start(...)` launches the hidden owner helper, then submits the initial prompt through the same public prompt plane used by follow-up turns in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:301).
- `substrate agent turn --session <orchestration_session_id> --backend <backend_id> ...` is already public and exact in [cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:487).
- Public turn selectors already reject `active_session_handle_id`, `participant_id` / legacy `session_handle_id`, and `internal.uaa_session_id` as noncanonical in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1142).
- Linux world-member follow-up submission is already typed and retained. `MemberTurnSubmitRequestV1` in [lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:845) is the host-to-world submit contract, and `validate_submit_turn_request(...)` in [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:869) proves that tuple identity, not backend id alone, is authoritative.
- `substrate -c` remains shell wrap mode and is already protected by test in [agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1129).

## Scope

In scope:

- harden and validate the end-to-end public runtime path for `agent start` and `agent turn`,
- treat `reattach` as required recovery for detached world-follow-up posture, but not as the primary feature under test,
- explicitly test the world-member public turn path from public `(session, backend)` resolution to `/v1/member_turn/stream` submission,
- add missing fail-closed coverage for exact public turn error cases,
- update operator-facing docs only where needed to reflect the hardened runtime contract.

## Non-Goals

Out of scope:

- redesigning `substrate -c`,
- changing REPL grammar,
- default-agent or fuzzy routing,
- public world-root start,
- macOS or Windows parity,
- making `fork` or `stop` primary acceptance gates for this slice,
- broader status-surface redesign.

## Required Runtime Contract

### Public start

- `substrate agent start` remains the canonical public root prompt-taking surface.
- It requires exact `--backend <backend_id>`.
- Root start remains host-only in v1; world-only backends must fail closed.

### Public turn

- `substrate agent turn` remains the canonical public follow-up surface.
- It requires exact `--session <orchestration_session_id>` and exact `--backend <backend_id>`.
- Exact selector resolution is authoritative, but the full behavior contract spans:
  - `run_turn(...)` for posture branching and detached recovery or rejection,
  - `resolve_public_turn_target(...)` for exact retained-slot selection,
  - `run_public_prompt_command(...)` for active owner transport submission,
  - and Linux `MemberTurnSubmitRequestV1` submission plus `world-agent` validation for world-member turns.

### Hidden engineering contract for world follow-up

For world-sensitive public follow-up turns, the public selector pair `(session, backend)` must be translated into retained-member identity before submission. That translation must preserve:

- `participant_id`,
- `orchestrator_participant_id`,
- `backend_id`,
- `world_id`,
- `world_generation`.

The shell must submit those fields through `MemberTurnSubmitRequestV1`, and `world-agent` must reject identity drift before resuming the retained member session.

### Fail-closed rules

Public turn actions must fail closed for:

- `missing_backend`,
- `unknown_session`,
- all `noncanonical_session_selector` variants,
- `missing_active_parent`,
- `backend_not_in_session`,
- `stale_linkage`,
- `ambiguous_backend_slot`,
- `unsupported_platform_or_posture`,
- `owner_unreachable`.

No fallback is allowed to a fuzzy session, fuzzy backend, REPL state, or `-c` command mode.

## Concrete Work Breakdown

### 1. Runtime hardening and end-to-end validation

- Audit the public prompt-taking path across [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs), [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs), [lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs), [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs), and [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs).
- Tighten any mismatch where public turn routing, detached posture handling, private owner transport submission, and world-member submit identity do not form one coherent contract.

### 2. Public world-member turn coverage

- Add explicit end-to-end tests showing that a public `agent turn --session ... --backend <world-backend>` on Linux resolves the exact retained member slot, shapes `MemberTurnSubmitRequestV1`, and reaches the retained member submit path successfully.
- Prove that the resolved public selector is not merely “backend selected,” but is translated into the retained member tuple required by `world-agent`.

### 3. Missing fail-closed coverage for public turn

Add explicit tests for:

- `missing_backend`,
- `unknown_session`,
- `missing_active_parent`,
- `backend_not_in_session`,
- `stale_linkage`,
- `ambiguous_backend_slot`,
- noncanonical selector via `active_session_handle_id`,
- noncanonical selector via `participant_id` / legacy `session_handle_id`,
- noncanonical selector via `internal.uaa_session_id`,
- detached-world follow-up rejection with reattach guidance,
- `owner_unreachable`.

### 4. Retained-member identity-drift rejection

- Add at least one explicit negative test at the `MemberTurnSubmitRequestV1` / `submit_turn(...)` boundary proving that world-member follow-up is rejected when the retained identity tuple drifts.
- The minimum acceptable case is one mismatch in retained-member identity, such as `backend_id`, `world_generation`, `world_id`, `participant_id`, or `orchestrator_participant_id`, as validated by `validate_submit_turn_request(...)` in [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:869).
- This test should make the hidden contract explicit: successful public world follow-up depends on exact retained-member tuple continuity, not just exact public selector resolution.

### 5. Keep recovery posture exact

- Keep detached-host recovery in `run_turn(...)` as-is in principle, but validate it with tests as part of the public contract.
- Keep detached-world follow-up fail-closed until `reattach` restores an active host owner; do not broaden world follow-up recovery implicitly.

### 6. Focused docs and planning cleanup

If runtime hardening changes or clarifies operator-visible behavior, update only these files:

- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
- the new `llm-last-mile` SOW file created from this draft

This is secondary to runtime hardening and coverage.

## Acceptance Criteria

This slice is done when all of the following are true:

1. `substrate agent start` still succeeds only for exact host-scoped backends and still uses the public prompt plane end to end.
2. `substrate agent turn` is covered end to end for both:
   - host follow-up through the private owner prompt transport,
   - Linux world-member follow-up through `MemberTurnSubmitRequestV1` to `/v1/member_turn/stream` to `submit_turn(...)`.
3. The Linux world-member public turn path proves that `(orchestration_session_id, backend_id)` is translated into retained-member identity including `participant_id`, `orchestrator_participant_id`, `world_id`, and `world_generation`.
4. Public turn rejects `missing_backend` with an explicit test.
5. Public turn rejects `unknown_session` with an explicit test.
6. Public turn rejects `missing_active_parent` with an explicit test.
7. Public turn rejects `backend_not_in_session` with an explicit test.
8. Public turn rejects `stale_linkage` with an explicit test.
9. Public turn rejects `ambiguous_backend_slot` with an explicit test.
10. Public turn rejects all noncanonical selector variants with explicit tests:
   - `active_session_handle_id`,
   - `participant_id` / legacy `session_handle_id`,
   - `internal.uaa_session_id`.
11. Detached host follow-up recovery and detached world follow-up rejection are both covered as part of the public contract.
12. Public turn reaches `owner_unreachable` with an explicit test.
13. The world-member submit boundary has at least one explicit retained-member identity-drift rejection test.
14. `substrate -c` still remains shell wrap mode.
15. Any operator-facing doc changes are limited to `docs/USAGE.md`, `AGENT_ORCHESTRATION_GAP_MATRIX.md`, and the new `llm-last-mile` file for this slice.

## Testing Expectations

Primary test targets:

- [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)

Required assertions:

- public `start` streams acceptance and terminal completion and leaves authoritative active state behind it,
- public host `turn` succeeds end to end on the exact session/backend pair,
- public Linux world-member `turn` succeeds end to end on the exact session/backend pair,
- world-member public turn carries the retained identity tuple required by `MemberTurnSubmitRequestV1`,
- `missing_backend` fails closed,
- `unknown_session` fails closed,
- `missing_active_parent` fails closed,
- `backend_not_in_session` fails closed,
- `stale_linkage` fails closed,
- `ambiguous_backend_slot` fails closed,
- each noncanonical selector variant fails closed with the expected classifier,
- detached host recovery is exercised and verified,
- detached world follow-up fails closed and instructs the operator to reattach,
- `owner_unreachable` fails closed,
- one retained-member identity-drift case is rejected at the `MemberTurnSubmitRequestV1` / `submit_turn(...)` boundary,
- root start for world-only backends still fails closed,
- `substrate -c` is not reinterpreted as an agent prompt surface.

Recommended commands:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p agent-api-types member_turn_submit -- --nocapture
```

## Explicit Non-Goals For Review

- Do not widen scope into `fork` or `stop` product work.
- Do not redesign prompt grammar or add default routing.
- Do not repurpose `substrate -c`.
- Do not promise non-Linux world-follow-up parity before the backend path exists.
