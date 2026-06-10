# TASKS-52: Internal Runtime-Owned Host-Orchestrator Tool Adapter Contract Freeze

Source spec: [SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md](./SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md)  
Source plan: [PLAN-52.md](./PLAN-52.md)  
Execution model: four sequential implementation packets  
Phase: `TASKS`  
Status: Packet 4 closeout verified on `2026-06-09`

## Phase Gate

These tasks assume the `SPECIFY` and `PLAN` artifacts for Slice `52` have been reviewed and accepted before implementation begins.

Current tree note:

1. Packets `1` through `3` are already landed and verified on the live repo before this session.
2. This Packet `4` pass owns repo-local doc alignment plus the validation wall only.
3. The checklist below remains the slice ledger; this session marks only Packet `4` tasks directly after the validation wall reran green.
4. Packet `4` absorbed one bounded validation-driven refactor in [`crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`](../crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs) so clippy stayed green without widening Slice `52`.

This slice is a contract-freeze slice.

It must **not**:

1. register tools in a live runtime family,
2. land an MCP server,
3. widen the public human toolbox CLI,
4. redesign the internal toolbox transport.

## Tracker Update Rule

For this slice family, every packet must keep the canonical seam tracker up to date:

- [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)

When a packet surfaces:

1. new live-truth drift,
2. an intentional deferral,
3. a validation finding that needs a later return pass,
4. or a sequencing change,

record it in that tracker under the correct section:

1. `## Newly Surfaced During Execution`
2. `## Deferred / Circle-Back Items`
3. `## Resolved Since Last Update`

Do not leave those notes only in packet chat output or code-review commentary.

## Execution Packets

This slice should run as four sequential implementation packets:

1. Packet 1 freezes the shared adapter vocabulary and exact handle families.
2. Packet 2 freezes runtime-owned injection and fresh-allocation translation.
3. Packet 3 freezes receipt normalization and follow-up handle resolution.
4. Packet 4 aligns repo-local docs/comments and runs the validation wall.

Do not collapse multiple packets into one session unless a review checkpoint explicitly approves that compression after Packet 1.

## Packet 1: Shared Adapter Vocabulary And Handle Freeze

Session goal:

1. add the bounded adapter-visible contract surface,
2. freeze exact tool names,
3. freeze exact retained versus active-ephemeral handle families.

### Tasks

- [ ] Task 1.1: Add the bounded adapter-visible contract module
  - Acceptance: the repo has one bounded adapter-contract surface under `crates/shell/src/execution/agent_runtime/` that defines the seven host tool names and their primary model-visible argument families without pretending that tool registration has already landed, and that contract freezes against the live `WorldDispatchRequestV1` field set rather than widening the internal transport to match the broader conceptual design envelope.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Files:
    - one new bounded adapter-contract module under `crates/shell/src/execution/agent_runtime/` (for example `tool_invocation_contract.rs`)
    - [`crates/shell/src/execution/agent_runtime/mod.rs`](../crates/shell/src/execution/agent_runtime/mod.rs) only if export wiring is needed

- [ ] Task 1.2: Freeze exact follow-up handle families
  - Acceptance: the adapter contract exposes exactly two follow-up handle families:
    - active-ephemeral handle keyed by exact `task_run_id`
    - retained-worker handle keyed by exact `participant_id`
    and the contract does not allow fuzzy targeting or mixed handle kinds.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
  - Files:
    - the new adapter-contract module
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs) only if bounded helper types or test fixtures are required

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the tool vocabulary is frozen to the seven landed dispatch verbs,
2. exact `task_run_id` and `participant_id` handles are frozen,
3. no runtime-family registration logic has been introduced.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Runtime-Owned Injection And Fresh-Allocation Translation

Session goal:

1. freeze which arguments the runtime injects,
2. translate fresh allocation tools into the landed internal request shape,
3. satisfy live validation without making the model author authoritative state.

### Tasks

- [ ] Task 2.1: Translate `run_world_task` into the live internal request contract
  - Acceptance: a `run_world_task` adapter call maps into a valid `WorldDispatchRequestV1` with runtime-owned `request_id`, `idempotency_key`, `orchestration_session_id`, `caller_participant_id`, `world_id`, and `world_generation`; the model supplies only intent fields such as `target_backend_id` and task payload; `idempotency_key` is treated as a universal runtime-injected field because that is what live validation currently requires.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
  - Files:
    - the new adapter-contract module
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

- [ ] Task 2.2: Translate `spawn_world_worker` into the live internal request contract
  - Acceptance: a `spawn_world_worker` adapter call maps into a valid retained `WorldDispatchRequestV1` with the same runtime-owned injection rules as Task 2.1, and the mapping does not ask the model to invent retained-worker authority fields or supply `idempotency_key`.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
  - Files:
    - the new adapter-contract module
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

- [ ] Task 2.3: Freeze follow-up injection rules against authoritative runtime state
  - Acceptance: follow-up tools are specified so retained or active-ephemeral handles can drive authoritative backend/world-binding resolution from runtime state when possible; the adapter does not require the model to restate authoritative fields already present in retained participant records or active-task records.
  - Verify:
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - the new adapter-contract module
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs) only if a bounded helper is required

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. fresh allocation tools inject runtime-owned authority fields,
2. live `idempotency_key` validation is satisfied,
3. follow-up tools do not force the model to reconstruct backend/world-binding truth from prose.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Receipt Normalization And Exact Follow-Up Resolution

Session goal:

1. normalize internal typed outcomes into adapter-visible receipts,
2. hide internal outcome asymmetries without changing the transport,
3. freeze exact retained versus active-ephemeral follow-up rules.

### Tasks

- [ ] Task 3.1: Normalize `run_world_task` and `spawn_world_worker` into adapter-visible receipts
  - Acceptance: `run_world_task` returns an adapter-visible active-task receipt centered on exact `task_run_id`, and `spawn_world_worker` returns an adapter-visible retained-worker receipt centered on exact `participant_id`; both receipts may echo backend/world-binding metadata for traceability, but they do not require runtime-family registration, and they normalize away the current internal distinction where active-ephemeral internal outcome structs still reuse `target_participant_id`.
  - Verify:
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
  - Files:
    - the new adapter-contract module
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

- [ ] Task 3.2: Freeze `inspect_world_worker` and `cancel_world_work` exact dual-handle semantics
  - Acceptance: the adapter-visible contract for `inspect_world_worker` and `cancel_world_work` accepts either an exact active-task handle or an exact retained-worker handle, rejects mixed or missing handle families, and normalizes the model-visible active-ephemeral identity to canonical `task_run_id` even though the current internal typed outcomes still reuse `target_participant_id` for that identity.
  - Verify:
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - the new adapter-contract module
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs) only if bounded helpers are needed

- [ ] Task 3.3: Freeze retained-only follow-up semantics for `continue_world_worker` and `stop_world_worker`
  - Acceptance: the adapter-visible contract requires a retained-worker handle for `continue_world_worker` and `stop_world_worker`, preserves the existing typed `continue_world_worker` payload families, and does not widen into runtime-family exposure or broader autonomy semantics.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
  - Files:
    - the new adapter-contract module
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs) only if bounded helper types/tests are needed
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. fresh-allocation receipts are normalized for the adapter layer,
2. dual-handle follow-up tools are exact and fail closed,
3. retained-only follow-up tools remain retained-only,
4. the slice still has not landed runtime-family tool registration.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Repo-Local Docs, Comments, And Final Validation

Session goal:

1. align repo-local docs/comments with the frozen contract,
2. keep later runtime-family landing explicitly deferred,
3. run the validation wall.

### Tasks

- [x] Task 4.1: Align repo-local docs and bounded code comments with Slice `52`
  - Acceptance: repo-local planning docs and any needed bounded code comments describe Slice `52` as a Substrate-native adapter-contract freeze only, and do not imply that `codex`, `claude_code`, MCP, or a public human toolbox CLI have already landed.
  - Verify:
    - manual diff review
  - Files:
    - [`llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md`](./SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md)
    - [`llm-last-mile/PLAN-52.md`](./PLAN-52.md)
    - [`llm-last-mile/TASKS-52.md`](./TASKS-52.md)
    - bounded code comments only if needed

- [x] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, and full workspace tests are green against the bounded Slice `52` file set; if validation exposes a need for runtime-family registration or public toolbox execution work, stop and split that into the next slice instead of silently widening this one.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
    - `cargo test -p shell async_repl -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Files:
    - final validation may touch only the bounded Slice `52` contract surfaces if a narrowly scoped defect is found
  - Stop condition: if a failing command requires `codex` tool registration, `claude_code` parity work, MCP, public CLI work, or transport redesign, stop and create an explicit follow-on slice instead of fixing it here.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. docs/comments and runtime truth are aligned,
2. the validation wall is green,
3. Slice `52` still reads honestly as a contract-freeze slice.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

After each packet, treat the next step as a packet-checkpoint review, not a fresh spec restart.

Proceed directly to the next packet only when:

1. the packet’s verification commands are green,
2. the packet checkpoint is satisfied,
3. the Slice `52` boundaries still hold,
4. the work has not widened into runtime-family landing, public CLI, or MCP work,
5. any newly surfaced drift/deferral items have been recorded in [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md).

Reopen spec/plan/tasks only if one of these becomes true:

1. the shared contract cannot be frozen without redesigning the internal transport,
2. exact follow-up handle semantics cannot be preserved without a broader orchestration-state redesign,
3. runtime-family registration is unexpectedly required to prove the contract,
4. the slice cannot stay Substrate-native without immediately becoming MCP-first.

If none of those conditions are met, continue packet to packet.

## Packet Session Final Message Requirements

Every implementation packet session should end by saying:

1. which verification commands passed or failed,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether the slice stayed bounded to adapter-contract freeze instead of widening into runtime-family landing, public CLI, or MCP work.
