# Plan: Human Public Agent Start Streaming Regression

Source spec: [SPEC-61-human-public-agent-start-streaming-regression.md](./SPEC-61-human-public-agent-start-streaming-regression.md)  
Related landed slices:
- [SPEC-55-broader-caller-surface-contract-freeze.md](./SPEC-55-broader-caller-surface-contract-freeze.md)
- [SPEC-60-post-placement-aware-compatibility-retirement.md](./SPEC-60-post-placement-aware-compatibility-retirement.md)  
Plan type: bounded runtime regression repair  
Status: draft for review  
Implementation posture: restore plain human streaming without weakening identity validation or changing the JSON wire contract

## Objective

Repair the public prompt human-rendering regression so plain `substrate agent start` and the same shared non-JSON renderer path used by `substrate agent turn` once again surface runtime commentary/status/tool-activity events before the terminal completion summary.

This plan must land:

1. a failing regression test for the non-JSON path,
2. one shared identity-normalization source of truth for emitted event tuple clients,
3. a non-silent human renderer behavior for structured prompt events,
4. proof that `--json` output and strict validation semantics remain unchanged.

## Plan Summary

The bug boundary is already narrow:

1. the runtime generates events,
2. `--json` streaming is already correct,
3. the plain human renderer is silent because `AgentEvent` deserialization rejects emitted tuple clients like `codex-host`,
4. and the current fallback writes an empty string for structured payloads with no top-level `text`.

So the correct plan is:

1. freeze the failing human-path contract in tests,
2. fix event construction at the shared identity seam,
3. harden the non-JSON renderer so future structured-event decode failures are visible instead of silent,
4. prove manual and automated parity between plain human and `--json` flows.

Do **not** widen this slice into:

1. placement-aware selector retirement beyond the already identified shared helper reuse,
2. public summary-line redesign,
3. trace schema changes,
4. or a general overhaul of all agent-event identity semantics.

## Locked Decisions

### What changes

1. Non-JSON public prompt streaming regains visible event output.
2. Shared `AgentEvent` telemetry identity construction normalizes emitted client ids before validation.
3. The human renderer stops degrading to empty output for structured events.
4. Public control-surface tests gain explicit non-JSON streaming coverage.

### What does not change

1. `IdentityTuple::validate()` remains strict.
2. `PublicPromptEnvelope` JSON shape remains unchanged.
3. The final `action=... orchestration_session_id=... turn_outcome=...` summary line remains the terminal human contract unless implementation evidence forces a narrowly scoped wording correction.
4. Runtime session/orchestration semantics remain unchanged; this is a rendering/event-shape repair, not a lifecycle redesign.

## Implementation Order

### Phase 1: Freeze The Human Streaming Contract In Tests

Goal:

1. make the silent non-JSON failure reproducible in automation,
2. lock the success condition to streamed lines appearing before the terminal summary,
3. keep the existing `--json` contract as the control case.

Primary touch surface:

1. `crates/shell/tests/agent_public_control_surface_v1.rs`
2. optionally `crates/common/tests/agent_hub_event_envelope_schema.rs` if identity-specific repro coverage is clearer there

Required changes:

1. add a plain human `agent start` streaming test,
2. extend coverage to the shared `agent turn` renderer path if separate coverage is needed,
3. assert that human output contains at least one streamed line and the final summary line,
4. keep the existing NDJSON stream contract test green.

Verification checkpoint:

1. the human regression reproduces under test before the fix,
2. the success condition is terminally visible output, not merely exit code `0`,
3. the JSON control path remains unchanged.

### Phase 2: Normalize Emitted Event Tuple Clients At The Shared Source

Goal:

1. stop generating invalid `identity_tuple.client` values for runtime-originated `AgentEvent` payloads,
2. centralize normalization rather than leaving multiple ad hoc implementations,
3. preserve strict validation.

Primary touch surface:

1. `crates/common/src/identity.rs`
2. `crates/common/src/agent_events.rs`
3. `crates/shell/src/builtins/world_gateway.rs`
4. any directly affected schema/unit tests

Required changes:

1. extract or add one shared helper that converts runtime client ids such as `codex-host` into validator-safe snake_case ids such as `codex_host`,
2. route `set_pure_agent_telemetry_identity` through that helper,
3. update `world_gateway` to reuse the shared helper or remove duplicate normalization,
4. preserve direct validation failures for intentionally malformed raw tuple payloads.

Verification checkpoint:

1. runtime-owned public prompt events deserialize as `AgentEvent`,
2. validation still rejects malformed raw tuple values,
3. no second normalization rule survives in a separate shell-only helper without explicit justification.

### Phase 3: Harden The Human Renderer Against Structured Decode Failure

Goal:

1. ensure the non-JSON renderer never silently emits an empty line for a structured prompt event,
2. keep stderr behavior unchanged,
3. make future decode failures visible even if they reappear through a different event-shape regression.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/control.rs`
2. related renderer/control tests

Required changes:

1. keep the preferred path as `AgentEvent` decode plus `format_event_line`,
2. if decode fails, render `data.message` or another compact human-meaningful fallback,
3. do not alter the JSON envelope path,
4. preserve the existing completion summary behavior.

Verification checkpoint:

1. structured message events no longer have an empty-string render path,
2. stderr passthrough still works,
3. the fallback is human-readable and bounded.

### Phase 4: Final Validation And Live Repro

Goal:

1. prove the bug is fixed end to end,
2. prove `--json` behavior is unchanged,
3. prove the shared validation contract remains strict.

Verification wall:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture`
4. `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
5. manual plain human repro with `~/.substrate/bin/substrate agent start ...`
6. manual `--json` control repro with the same prompt

Exit criteria:

1. plain human output streams commentary/status/tool-activity before the final summary,
2. `--json` still streams accepted/event/completed envelopes,
3. no validation weakening was required,
4. no structured prompt event is silently rendered as blank output.

## Risks And Mitigations

### Risk 1: The fix only patches `agent start` while leaving `agent turn` exposed

Mitigation:

1. test the shared non-JSON renderer path directly or through both caller surfaces,
2. treat `run_public_prompt_command` and startup-stream projection as the two authoritative callers of `PublicPromptRenderer`.

### Risk 2: Shared normalization changes behavior outside this bug

Mitigation:

1. keep the helper narrowly scoped to tuple-client normalization,
2. run common schema tests and targeted shell tests,
3. preserve direct validation failures for intentionally malformed raw payloads.

### Risk 3: Renderer fallback accidentally changes the JSON contract

Mitigation:

1. isolate all fallback logic to the non-JSON branch of `PublicPromptRenderer.render`,
2. leave `serde_json::to_string(envelope)` untouched in the JSON path.

## Sequencing And Parallelism

1. Phase 1 must land before any behavior fix so the regression stays pinned.
2. Phase 2 should land before Phase 3 so the primary correctness fix is at the source rather than hidden in renderer tolerance.
3. Phase 3 can be kept small and defensive once the source fix is in place.
4. Phase 4 runs only after both source correction and renderer hardening are complete.

## Review Checkpoints

After each phase:

1. confirm the scoped checkpoint is green,
2. confirm no validation rule was weakened,
3. confirm the bug remains bounded to public prompt event construction/rendering rather than expanding into broader runtime semantics.
