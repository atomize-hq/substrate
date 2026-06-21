# TASKS: Human Public Agent Start Streaming Regression

Source spec: [SPEC-61-human-public-agent-start-streaming-regression.md](./SPEC-61-human-public-agent-start-streaming-regression.md)  
Source plan: [PLAN-61-human-public-agent-start-streaming-regression.md](./PLAN-61-human-public-agent-start-streaming-regression.md)  
Phase: `TASKS`  
Execution model: four sequential implementation packets  
Status: draft for review

## Phase Gate

These tasks assume:

1. the runtime and `--json` public prompt stream are already correct,
2. the regression is bounded to shared event construction and non-JSON rendering,
3. strict identity validation remains a non-negotiable constraint.

Do not begin implementation until the spec and plan are accepted.

## Execution Packets

This slice should be implemented as four sequential packets:

1. pin the failing human-path contract in tests,
2. normalize emitted tuple clients at the shared source,
3. harden the non-JSON renderer fallback,
4. run the final validation and live repro wall.

Do not begin a later packet until the prior packet checkpoint is green.

## Packet 1: Pin The Human Streaming Contract

Session goal:

1. reproduce the silent plain human path in automation,
2. lock success to visible streamed lines before terminal completion,
3. keep the existing `--json` contract as the control case.

### Tasks

- [ ] Task 1.1: Add plain human `agent start` streaming regression coverage
  - Acceptance: `agent_public_control_surface_v1` contains a non-JSON public start test that fails when only the terminal summary line is printed and passes only when streamed lines appear before completion.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](../crates/shell/tests/agent_public_control_surface_v1.rs)

- [ ] Task 1.2: Extend coverage to the shared non-JSON turn/renderer seam
  - Acceptance: the shared renderer path used by public `agent turn` is either covered by a dedicated plain-human turn assertion or by a focused renderer/control test that proves structured prompt events render visibly in non-JSON mode.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell control -- --nocapture` if a focused control-unit test is added
  - Files:
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](../crates/shell/tests/agent_public_control_surface_v1.rs)
    - optionally [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs) if a focused unit test lands there

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the silent human-path bug reproduces in automation,
2. the success condition requires visible streamed lines before completion,
3. the existing NDJSON control-path coverage remains green.

Do not start Packet 2 until Packet 1 is reviewed and green.

## Packet 2: Normalize Emitted Tuple Clients At The Shared Source

Session goal:

1. stop generating invalid `identity_tuple.client` values like `codex-host`,
2. centralize normalization in shared code,
3. keep validation strict for raw invalid payloads.

### Tasks

- [ ] Task 2.1: Add shared client-id normalization for telemetry identity emission
  - Acceptance: shared event construction emits validator-safe snake_case tuple clients such as `codex_host` when given runtime ids like `codex-host`; direct validation rules remain strict for malformed raw payloads.
  - Verify:
    - `cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture`
    - `cargo test -p substrate-common -- --nocapture`
  - Files:
    - [`crates/common/src/identity.rs`](../crates/common/src/identity.rs)
    - [`crates/common/src/agent_events.rs`](../crates/common/src/agent_events.rs)
    - [`crates/common/tests/agent_hub_event_envelope_schema.rs`](../crates/common/tests/agent_hub_event_envelope_schema.rs)

- [ ] Task 2.2: Align shell-side normalization call sites with the shared helper
  - Acceptance: `world_gateway` no longer carries a divergent local normalization rule for the same tuple-client concept, or it is explicitly routed through the shared helper.
  - Verify:
    - `cargo test -p shell --test world_gateway -- --nocapture`
    - `rg -n "resolve_originating_client|set_pure_agent_telemetry_identity|normalize.*client" crates/common/src crates/shell/src`
  - Files:
    - [`crates/shell/src/builtins/world_gateway.rs`](../crates/shell/src/builtins/world_gateway.rs)
    - any directly related shared helper files from Task 2.1

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. runtime-originated public prompt events deserialize cleanly as `AgentEvent`,
2. strict raw validation still rejects malformed tuple clients,
3. shared normalization logic is not duplicated across separate seams without justification.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Harden The Non-JSON Renderer

Session goal:

1. eliminate the empty-string structured-event failure mode,
2. preserve JSON output unchanged,
3. keep stderr behavior stable.

### Tasks

- [ ] Task 3.1: Add a human-readable structured-event fallback in `PublicPromptRenderer`
  - Acceptance: if non-JSON rendering cannot deserialize a structured prompt event into `AgentEvent`, the renderer still prints a useful line derived from the payload instead of silently writing an empty string.
  - Verify:
    - `cargo test -p shell control -- --nocapture` if focused control-unit coverage is added
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
    - related control test file(s) if added

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. the non-JSON renderer has no silent structured-event path,
2. JSON rendering remains untouched,
3. completion summary behavior still matches current human contract.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Final Validation And Live Repro

Session goal:

1. prove the regression is fixed end to end,
2. prove the JSON path is unchanged,
3. prove strict validation still holds.

### Tasks

- [ ] Task 4.1: Run the targeted validation wall
  - Acceptance: the shared schema tests and public control-surface tests are green after the fix.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test world_gateway -- --nocapture` if Packet 2 touched gateway normalization
  - Files:
    - no planned source edits; validation only

- [ ] Task 4.2: Re-run the live plain-human and JSON repro commands
  - Acceptance: the plain repro visibly streams commentary/status/tool-activity before the terminal summary, and the JSON repro still emits accepted/event/completed envelopes.
  - Verify:
    - `~/.substrate/bin/substrate agent start --backend cli:codex-host --prompt "Look at the native tools you have available to you within the substrate environment. Not your builtin codex tool_search and other tools, but the tools you have been informed about at runtime."`
    - `~/.substrate/bin/substrate agent start --backend cli:codex-host --prompt "Look at the native tools you have available to you within the substrate environment. Not your builtin codex tool_search and other tools, but the tools you have been informed about at runtime." --json`
  - Files:
    - no planned source edits; manual verification only

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. plain human `agent start` visibly streams before completion,
2. the shared non-JSON renderer path no longer drops structured events silently,
3. `--json` remains wire-compatible,
4. strict identity validation still stands.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

After each packet:

1. confirm its checkpoint is satisfied,
2. confirm no validation rule was weakened,
3. confirm the slice remains bounded to public prompt event construction and rendering.
