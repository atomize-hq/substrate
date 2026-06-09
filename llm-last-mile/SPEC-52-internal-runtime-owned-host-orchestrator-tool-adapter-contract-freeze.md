# Spec: Internal Runtime-Owned Host-Orchestrator Tool Adapter Contract Freeze

Source tracker note: [REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md](./REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md)  
Companion design inputs:
- [DESIGN-internal-toolbox-transport-and-session-binding.md](./DESIGN-internal-toolbox-transport-and-session-binding.md)
- [DESIGN-host-orchestrator-tool-invocation-surface.md](./DESIGN-host-orchestrator-tool-invocation-surface.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
Related remaining-scope note: [REMAINING-family-1-and-2-scope-2026-06-08.md](./REMAINING-family-1-and-2-scope-2026-06-08.md)  
Prior numbered slice: [SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md](./SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md)  
Phase: `SPECIFY`  
Status: proposed on `2026-06-08`  
Session boundary: docs-only planning session; do not modify product code in this session.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `52` is the next honest numbered slice because live repo truth shows numbered `SPEC`/`PLAN`/`TASKS` artifacts through `51` and no existing `52` artifacts are present.
2. The target seam is not more Family-1 transport/runtime invention. The landed internal toolbox transport and typed world-dispatch runtime are already present; the missing seam is the runtime-owned adapter contract above them.
3. This slice should freeze the agent-visible tool contract only:
   - tool vocabulary
   - model-supplied versus runtime-injected arguments
   - receipt and follow-up handle semantics
   - Substrate-native v1 boundary
   It should not register tools inside a real runtime family yet.
4. The current internal transport remains the source of truth for what is already landed:
   - `WorldDispatchRequestV1` / `WorldDispatchOutcomeV1`
   - session-scoped UDS toolbox transport
   - newline-delimited `event` / `result` framing
   - exact session/world binding checks
5. `codex` first and `claude_code` second is still the default sequencing assumption for later runtime-family landing because current repo truth names both families but does not yet show a landed shared tool-registration seam.
6. This slice should stay Substrate-native in v1. It must leave room for a later MCP wrapper without making MCP the source of truth or a prerequisite for the first honest host tool surface.
7. This slice should not reopen Family-2 inbox, router, obligation, or federation semantics.
8. This slice should not rewrite ADR-0026 or ADR-0045 and should not depend on them as current implementation authority.

If any of these are wrong, correct them before implementation.

## Tracking Note Update Rule

For this slice family, the canonical running ledger is:

- [REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md](./REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md)

When implementation or review surfaces:

1. live-truth drift between design docs and code,
2. explicit deferrals that should be circled back around to later,
3. validation findings that change the next honest slice,
4. doc-truth alignment gaps for this seam,

record them there, not only here.

Add them under the tracker’s:

1. `## Newly Surfaced During Execution`
2. `## Deferred / Circle-Back Items`
3. `## Resolved Since Last Update`

Update this spec only when the Slice `52` contract authority itself changes.

## Objective

Write the next bounded implementation-shaping slice for the host orchestrator’s missing agent-visible tool invocation surface.

This slice is complete only when the repo has one frozen runtime-owned adapter contract that:

1. preserves the seven landed internal dispatch verbs as the canonical host tool vocabulary,
2. states exactly which fields are model intent versus runtime-owned injected truth,
3. freezes receipt-oriented semantics for `run_world_task` and `spawn_world_worker`,
4. freezes the exact identity later consumed by `inspect_world_worker`, `continue_world_worker`, `cancel_world_work`, and `stop_world_worker`,
5. keeps the current internal toolbox transport as the authority below the adapter contract,
6. makes the v1 boundary explicitly Substrate-native and explicitly not MCP-first,
7. does all of the above without widening into runtime-family registration, public CLI work, or transport redesign.

Primary product story after this slice:

1. Substrate has one adapter contract the runtime-family landing can consume later.
2. The host model will not be asked to invent request ids, idempotency keys, session ids, caller ids, or world binding from prose.
3. The host model will receive exact ephemeral-task or retained-worker receipts it can use later without guessing handle semantics.
4. The first runtime-family landing can consume a frozen contract instead of inventing family-specific tool shapes.

## Why This Slice Exists

The current repo already has the internal dispatch runtime but still does not have the live host-session tool contract the model can rely on.

Prompt-only is insufficient because the prompt cannot safely provide:

1. endpoint discovery,
2. authoritative session identity,
3. authoritative caller identity,
4. authoritative world binding,
5. event-frame handling,
6. deterministic receipt capture.

A subprocess `substrate ...` CLI hop is not the primary in-session model because:

1. the host session itself is the caller that needs the capability,
2. the runtime already owns the live session and toolbox endpoint,
3. a subprocess hop does not remove the need for runtime-owned identity injection,
4. a subprocess contract would still sit beside, not replace, the real adapter problem.

So the honest missing seam is an adapter contract, not better prompts and not a human CLI wrapper.

## Observed Repo Floor

### 1. The internal toolbox transport is already landed and session-scoped

Live repo truth already includes:

1. deterministic toolbox UDS path derivation in [`control.rs`](../crates/shell/src/execution/agent_runtime/control.rs),
2. operator-visible introspection-only toolbox surfaces in [`agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs),
3. runtime-side UDS registration plus newline-delimited response frames in [`async_repl.rs`](../crates/shell/src/repl/async_repl.rs),
4. typed dispatch request validation and typed action-specific outcomes in [`dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs),
5. routed execution and exact identity enforcement in [`orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs).

Repo-truth consequence:

1. Slice `52` must build above the landed transport.
2. Slice `52` must not invent a parallel control plane.

### 2. The current operator-visible toolbox surface is still introspection-only

The current operator-facing toolbox contract is still:

1. `substrate agent toolbox status`
2. `substrate agent toolbox env`

Those surfaces expose posture and endpoint truth, but they do not register callable tools inside a live host runtime family and they do not provide public human execution verbs for `run_world_task`, `spawn_world_worker`, or later follow-up actions.

Repo-truth consequence:

1. the missing seam is still the host agent’s callable tool contract,
2. public CLI work remains out of scope for this slice.

### 3. The landed internal verb family already matches the intended tool vocabulary

Live `WorldDispatchActionV1` already exposes:

1. `run_world_task`
2. `spawn_world_worker`
3. `fork_world_worker`
4. `continue_world_worker`
5. `inspect_world_worker`
6. `cancel_world_work`
7. `stop_world_worker`

Repo-truth consequence:

1. Slice `52` should freeze the tool vocabulary to those seven verbs.
2. Slice `52` should not invent a second naming family for the same control-plane actions.

### 4. `run_world_task` and `spawn_world_worker` already prove receipt-oriented runtime truth

Live runtime behavior already shows:

1. `run_world_task` can surface `task_run_id` early through a non-terminal internal toolbox `event` frame with `event_kind=task_run_id_registered`,
2. `run_world_task` also registers exact active-ephemeral task records under the authoritative orchestration session,
3. `spawn_world_worker` returns an authoritative retained-worker receipt built from the streamed `registered` event,
4. later retained follow-up routes through exact participant identity,
5. later ephemeral inspect/cancel routes through exact `task_run_id` plus authoritative session/backend/world-binding validation.

Repo-truth consequence:

1. Slice `52` must stay receipt-oriented.
2. Slice `52` must not flatten the model into one blocking “wait until everything finishes” call shape.

### 5. Later follow-up identity is exact, not fuzzy

Live routing already fails closed unless follow-up identity stays exact:

1. retained `continue` / `inspect` / `cancel` / `stop` use exact `target_participant_id`,
2. active-ephemeral `inspect` / `cancel` use exact `task_run_id`,
3. target backend and authoritative world binding must still match,
4. active-ephemeral task ids may repeat across sessions, so session binding remains part of exact truth.

Repo-truth consequence:

1. Slice `52` must freeze one exact follow-up handle model.
2. Slice `52` must not allow fuzzy targeting by role label or prompt wording.

### 6. Runtime-family truth exists, but tool registration does not

Current config and usage docs still name `codex` and `claude_code` as the shell-owned runtime families.

Current repo truth does **not** yet show a landed runtime-family-specific tool-registration or tool-injection surface for the internal dispatch verbs.

Repo-truth consequence:

1. Slice `52` should freeze a shared contract first.
2. Slice `53` or later can consume that contract for the first real runtime-family landing.

## Repo-Truth Deltas From The Design Inputs

The new design docs are directionally correct about the missing seam, but some design details are broader than the live code and should not be treated as current runtime truth.

### Delta 1: The live internal request envelope is narrower than the conceptual design envelope

`DESIGN-host-orchestrator-world-dispatch-contract.md` describes a conceptual envelope that includes fields such as:

1. `caller_backend_id`
2. `caller_role`
3. `capability_overrides`
4. `requested_policy_narrowing`
5. `created_at`

Live `WorldDispatchRequestV1` does **not** currently carry those fields. The landed request shape is narrower and should remain the transport source of truth for Slice `52`.

### Delta 2: Live validation currently requires `idempotency_key` on every internal request

The design doc frames `idempotency_key` mainly as a retry guard for `run_world_task` and `spawn_world_worker`.

Live `WorldDispatchRequestV1::validate()` currently requires a non-empty `idempotency_key` for every action.

Slice `52` should therefore freeze runtime-owned id generation against live validation, not against the narrower design-only phrasing.

### Delta 3: The conceptual inspect/outcome shapes are ahead of the live per-action outcome contract

The design doc describes more generic conceptual outcome fields such as `state`, `detail_ref`, and `escalation_hint`.

Live runtime truth is more specific:

1. each action has its own typed outcome struct,
2. `WorkerInspectPayloadV1` is currently empty rather than “optional scope”-bearing,
3. active-ephemeral inspect/cancel still accept `task_run_id` on input but currently echo that identity through `target_participant_id` inside the typed internal outcome structs.

Slice `52` should freeze an adapter-visible handle model that hides those internal asymmetries without rewriting the internal transport.

## Explicit Resolution Of These Deltas In Slice 52

Slice `52` resolves the three deltas above as follows.

### Resolution 1: Freeze against the live internal request envelope, not the broader conceptual one

For this slice:

1. the adapter contract should target the current live `WorldDispatchRequestV1` field set,
2. the broader conceptual design-envelope fields stay documented as possible future widening,
3. Slice `52` must not widen the internal transport just to make the design docs look more complete.

Repo-truth consequence:

1. the adapter contract is implementation-shaping against the real wire contract,
2. any future request-envelope widening should be its own transport-focused slice.

### Resolution 2: Treat `idempotency_key` as a universal runtime-injected field in v1

For this slice:

1. the model does not supply `idempotency_key`,
2. the runtime injects `idempotency_key` for every adapter call that becomes an internal dispatch request,
3. this follows current live validation rather than the narrower design-only create-action interpretation.

Repo-truth consequence:

1. Slice `52` stays aligned with the current validator,
2. if the repo later wants to narrow `idempotency_key` requirements to only some actions, that should be a separate validator/wire cleanup slice rather than a hidden change inside the adapter-contract freeze.

### Resolution 3: Normalize active-ephemeral follow-up identity at the adapter boundary

For this slice:

1. the canonical adapter-visible identity for active-ephemeral work is `task_run_id`,
2. the canonical adapter-visible identity for retained work is `participant_id`,
3. adapter-visible inspect/cancel outcomes for active-ephemeral work should surface `task_run_id` even though the current typed internal outcome structs still reuse `target_participant_id`,
4. Slice `52` should normalize that asymmetry at the adapter layer rather than rewriting the internal typed outcome plane.

Repo-truth consequence:

1. the model sees one clean exact-handle contract,
2. later internal outcome-field harmonization may still be worthwhile,
3. that harmonization belongs in a later receipt/resume hardening or typed-outcome cleanup slice, not inside this contract-freeze slice.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Current authority files:
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
  - [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)
  - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- Likely future implementation surfaces for this slice:
  - one new bounded adapter-contract module under `crates/shell/src/execution/agent_runtime/` (for example `tool_invocation_contract.rs`)
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs) only if exact follow-up handle resolution needs a bounded helper
- This session remains docs-only and should not modify those code files now.

## Commands

Build:

```bash
cargo build --workspace
```

Format:

```bash
cargo fmt --all -- --check
```

Lint:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Targeted future validation floor for Slice `52`:

```bash
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell orchestrator_world_dispatch -- --nocapture
cargo test -p shell async_repl -- --nocapture
cargo test -p shell state_store -- --nocapture
```

Full validation wall after implementation:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

Relevant repo structure for this slice:

- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - landed internal transport request/outcome contract
  - Slice `52` must consume this as transport truth, not replace it
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - exact routing, authoritative receipt building, active-ephemeral registry use, and fail-closed follow-up validation
  - Slice `52` should reuse this exact identity truth when freezing adapter-visible handles
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - authoritative retained participant records and active-ephemeral task records
  - Slice `52` may need bounded handle-resolution helpers here, but should not redesign the store
- `crates/shell/src/repl/async_repl.rs`
  - internal toolbox framing and `task_run_id_registered` event proof
  - Slice `52` should treat this as evidence that the runtime is already receipt/event-capable
- `crates/shell/src/execution/agents_cmd.rs`
  - truthful toolbox status/env and runtime-family/operator posture
  - Slice `52` should not widen this into a public execution CLI
- `docs/CONFIGURATION.md` and `docs/USAGE.md`
  - current public/operator truth for toolbox and runtime families
  - later slices may update these, but Slice `52` must avoid implying live runtime-family tool registration before it exists
- `llm-last-mile/`
  - repo-local planning authority for this new tool-adapter contract family

## Code Style

Follow the repo’s existing style: exact identity, fail-closed validation, runtime-owned authority, and explanation-ready semantics.

Preferred style for the future implementation of this slice:

```rust
let request = WorldDispatchRequestV1 {
    request_id: Some(runtime_generated_request_id()),
    idempotency_key: Some(runtime_generated_idempotency_key()),
    orchestration_session_id: Some(authority.orchestration_session_id.clone()),
    caller_participant_id: Some(authority.caller_participant_id.clone()),
    action: WorldDispatchActionV1::RunWorldTask,
    mode: WorldDispatchModeV1::Ephemeral,
    target_backend_id: Some(args.target_backend_id.clone()),
    task_run_id: None,
    target_participant_id: None,
    world_id: Some(authority.world_id.clone()),
    world_generation: Some(authority.world_generation),
    payload: WorldDispatchPayloadV1::Task(TaskPayloadV1 {
        prompt: args.prompt.clone(),
    }),
};
```

Conventions this slice must preserve:

1. the model supplies intent, not runtime authority,
2. the runtime injects authoritative session/caller/world binding fields,
3. the adapter contract stays one-to-one with the landed internal verb family,
4. exact `task_run_id` and exact `participant_id` remain visible as durable follow-up identities,
5. the adapter may normalize internal outcome quirks, but it must not invent fake or fuzzy handles,
6. Substrate-native adapter truth comes before any later MCP wrapper.

## Frozen Adapter Direction

### 1. Canonical tool vocabulary

The adapter contract should freeze exactly these tool names:

1. `run_world_task`
2. `spawn_world_worker`
3. `fork_world_worker`
4. `continue_world_worker`
5. `inspect_world_worker`
6. `cancel_world_work`
7. `stop_world_worker`

No second vocabulary should be introduced for the same actions.

### 2. Runtime-owned versus model-supplied fields

The adapter contract should freeze the following split.

#### Runtime-owned and injected

1. `request_id`
2. `idempotency_key`
3. `orchestration_session_id`
4. `caller_participant_id`
5. authoritative `world_id`
6. authoritative `world_generation`
7. follow-up `target_backend_id` when it can be derived from authoritative retained or active-ephemeral state

#### Model-supplied intent

1. the tool/action name,
2. `target_backend_id` for fresh `run_world_task` and `spawn_world_worker` allocation,
3. exact follow-up identity (`task_run_id` or `participant_id`) when steering existing work,
4. typed payload content such as task prompt, worker prompt, cancel reason, or retained follow-up payload.

### 3. Receipt and handle semantics

The adapter contract should freeze two exact follow-up handle families:

1. active-ephemeral handle
   - exact `task_run_id`
   - surfaced first from `run_world_task`
2. retained-worker handle
   - exact `participant_id`
   - surfaced first from `spawn_world_worker` or `fork_world_worker`

The adapter contract should then freeze later follow-up rules:

1. `continue_world_worker` requires a retained-worker handle,
2. `stop_world_worker` requires a retained-worker handle,
3. `inspect_world_worker` accepts either:
   - an active-ephemeral handle, or
   - a retained-worker handle,
   but never both,
4. `cancel_world_work` accepts either:
   - an active-ephemeral handle, or
   - a retained-worker handle,
   but never both.

### 4. Internal transport preservation

The adapter contract may normalize the model-visible surface, but it must preserve the live transport truths below it:

1. same-session enforcement,
2. same-world-binding enforcement,
3. exact backend matching,
4. typed payload families,
5. event-plus-terminal-result framing,
6. explanation-ready fail-closed errors.

### 5. V1 Substrate-native boundary

This slice should freeze that v1 is:

1. adapter-based,
2. Substrate-native,
3. layered over the internal toolbox transport,
4. compatible with a later MCP wrapper,
5. not blocked on MCP task semantics,
6. not defined by a subprocess CLI contract.

## Testing Strategy

Frameworks:

- Rust unit tests
- targeted shell runtime regression tests
- existing async REPL / orchestrator-world-dispatch regression suites

Test levels for the future Slice `52` implementation:

1. adapter request-mapping tests
   - each tool maps to exactly one landed internal dispatch action,
   - runtime-owned fields are injected rather than accepted from model input,
   - fresh allocation tools require model intent only for backend plus payload
2. receipt and handle tests
   - `run_world_task` returns an active-ephemeral handle carrying exact `task_run_id`,
   - `spawn_world_worker` returns a retained-worker handle carrying exact `participant_id`,
   - retained follow-up tools reject task handles,
   - active-ephemeral follow-up paths reject retained-only semantics
3. fail-closed follow-up tests
   - mixed or missing follow-up handle kinds fail closed,
   - stale session/backend/world-binding mismatches still fail closed through authoritative runtime resolution,
   - adapter normalization does not erase exact underlying identity
4. non-goal regression tests
   - no runtime-family registration is required yet,
   - no MCP server or task extension is required,
   - no new public toolbox execution CLI appears.

## Boundaries

- Always:
  - treat the landed internal toolbox transport as the source of truth below the adapter contract
  - freeze tool names and follow-up handle semantics to the landed dispatch verbs
  - keep runtime-owned authority injection explicit
  - keep the slice bounded to contract freeze rather than family landing
- Ask first:
  - widening into live runtime-family tool registration
  - changing the internal toolbox transport framing or request/outcome wire contract
  - adding a public human CLI for toolbox execution verbs
  - making MCP the required v1 internal adapter path
- Never:
  - ask the model to invent session/caller/world binding fields from prompt text
  - replace exact `task_run_id` / `participant_id` handles with fuzzy or heuristic targeting
  - reopen Family-2 inbox/router semantics in this slice
  - treat the conceptual design envelope as more authoritative than live runtime code

## Success Criteria

This slice is complete only when all of the following are true:

1. the repo has one frozen runtime-owned adapter contract for the seven host tool verbs,
2. the contract explicitly separates model-supplied intent from runtime-injected identity/binding fields,
3. `run_world_task` and `spawn_world_worker` have frozen receipt semantics tied to exact `task_run_id` or `participant_id`,
4. `inspect_world_worker`, `continue_world_worker`, `cancel_world_work`, and `stop_world_worker` have frozen exact follow-up handle semantics,
5. the adapter contract is grounded in the live internal transport rather than the broader conceptual design-only envelope,
6. the contract leaves room for a later MCP wrapper but does not require MCP in v1,
7. the slice lands without registering tools in a real runtime family, without widening the public CLI, and without reopening transport/runtime invention.

## Open Questions

1. Should the adapter-visible follow-up API take raw exact ids (`task_run_id` / `participant_id`) directly, or should it wrap them in structured receipt objects while still preserving the exact ids as first-class fields?
2. For retained and active-ephemeral follow-up tools, should the adapter require the model to repeat `target_backend_id`, or should the runtime always derive it from authoritative retained/active state once the exact handle is supplied?
3. Should the first runtime-family landing expose all seven frozen tools immediately, or may it register only a bounded implemented subset while still preserving the names and semantics for the deferred tools?
4. Should the contract-freeze implementation live in a new bounded adapter module, or should it extend `dispatch_contract.rs` directly without making that file mix transport-wire truth and model-surface truth too tightly?
