# Spec: Human Public Agent Start Streaming Regression

Source authorities:
- [SPEC-55-broader-caller-surface-contract-freeze.md](./SPEC-55-broader-caller-surface-contract-freeze.md)
- [PLAN-55-broader-caller-surface-contract-freeze.md](./PLAN-55-broader-caller-surface-contract-freeze.md)
- [SPEC-60-post-placement-aware-compatibility-retirement.md](./SPEC-60-post-placement-aware-compatibility-retirement.md)
- [crates/shell/src/execution/agent_runtime/control.rs](../crates/shell/src/execution/agent_runtime/control.rs)
- [crates/common/src/agent_events.rs](../crates/common/src/agent_events.rs)
- [crates/common/src/identity.rs](../crates/common/src/identity.rs)
- [crates/shell/src/builtins/world_gateway.rs](../crates/shell/src/builtins/world_gateway.rs)
- [crates/shell/tests/agent_public_control_surface_v1.rs](../crates/shell/tests/agent_public_control_surface_v1.rs)
- [crates/common/tests/agent_hub_event_envelope_schema.rs](../crates/common/tests/agent_hub_event_envelope_schema.rs)
Phase: `SPECIFY`  
Status: draft for review

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The operator-visible bug is limited to the plain human output path for public prompt submission, especially `substrate agent start` and the shared renderer path used by `substrate agent turn`.
2. The upstream runtime and `--json` contract are already correct enough for this slice; the regression is in how the non-JSON path consumes or renders structured prompt events.
3. The identity-tuple validation tightened in recent repo truth is intentional and should remain strict; this slice should stop emitting invalid tuple values rather than relaxing validation.
4. The preferred fix point is shared identity normalization at event construction time, with renderer hardening as a defensive backstop rather than the primary correctness mechanism.
5. User-visible success means commentary, tool-activity, and status lines stream to a normal terminal before the final `action=... turn_outcome=...` summary line, not merely that the command eventually exits `0`.
6. `agent start --json` and persisted telemetry/trace semantics must remain wire-compatible with current landed behavior.

If any of these are wrong, correct them before implementation.

## Objective

Restore live streaming for the plain human `substrate agent start` surface, and preserve parity for the same shared renderer path used by `substrate agent turn`, after the identity-tuple validation tightening landed in the recent runtime-family commits.

Primary user story:

1. an operator runs plain `substrate agent start --backend cli:codex-host --prompt ...` in a terminal,
2. the runtime produces structured status/commentary/tool-activity events,
3. Substrate renders those events to the terminal as they arrive,
4. the terminal still ends with the existing completion summary line,
5. and `--json` output remains unchanged.

The bug this slice fixes:

1. `--json` currently streams accepted/event/completed envelopes correctly,
2. the plain human path currently prints only the terminal completion summary,
3. the silent middle section is caused by structured event payloads failing `AgentEvent` deserialization after identity validation rejects client ids like `codex-host`,
4. and the non-JSON fallback renderer then writes an empty string because the structured payload has no top-level `text` field.

## Tech Stack

- Rust workspace (`cargo`)
- Shared identity and event schema in:
  - [`crates/common/src/identity.rs`](../crates/common/src/identity.rs)
  - [`crates/common/src/agent_events.rs`](../crates/common/src/agent_events.rs)
- Public prompt streaming and rendering in:
  - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
- Existing normalization precedent in:
  - [`crates/shell/src/builtins/world_gateway.rs`](../crates/shell/src/builtins/world_gateway.rs)
- Regression coverage in:
  - [`crates/shell/tests/agent_public_control_surface_v1.rs`](../crates/shell/tests/agent_public_control_surface_v1.rs)
  - [`crates/common/tests/agent_hub_event_envelope_schema.rs`](../crates/common/tests/agent_hub_event_envelope_schema.rs)

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

Primary repro:

```bash
~/.substrate/bin/substrate agent start \
  --backend cli:codex-host \
  --prompt "Look at the native tools you have available to you within the substrate environment. Not your builtin codex tool_search and other tools, but the tools you have been informed about at runtime."
```

Control repro:

```bash
~/.substrate/bin/substrate agent start \
  --backend cli:codex-host \
  --prompt "Look at the native tools you have available to you within the substrate environment. Not your builtin codex tool_search and other tools, but the tools you have been informed about at runtime." \
  --json
```

Targeted validation:

```bash
cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
```

Focused source search:

```bash
rg -n "set_pure_agent_telemetry_identity|PublicPromptRenderer|prompt_event_text|codex-host|identity_tuple" \
  crates/common/src crates/shell/src crates/shell/tests
```

## Project Structure

```text
crates/common/src/identity.rs
  Shared identity validation rules. This slice must keep validation strict while making emitted client ids conform.

crates/common/src/agent_events.rs
  Shared AgentEvent construction and telemetry identity helpers. This is the preferred source fix seam.

crates/shell/src/execution/agent_runtime/control.rs
  Public prompt event translation, startup/turn stream handling, and the human/JSON renderer split.

crates/shell/src/builtins/world_gateway.rs
  Existing local client-id normalization precedent that should inform or be replaced by the shared helper.

crates/shell/tests/agent_public_control_surface_v1.rs
  End-to-end contract tests for public `agent start|turn|stop`; this slice must add plain human streaming coverage here.

crates/common/tests/agent_hub_event_envelope_schema.rs
  Shared schema/round-trip coverage for AgentEvent and identity tuples.

llm-last-mile/
  Planning authority for this runtime-family regression slice.
```

## Code Style

Follow the existing fail-closed, explicit-contract style. Normalize at the shared source, keep the validator strict, and make the human renderer degrade loudly rather than silently.

Preferred shape:

```rust
let client = normalize_identity_client_id(raw_client)
    .unwrap_or_else(|| "human".to_string());
event.set_pure_agent_telemetry_identity(client);
```

Conventions:

1. keep identity normalization in one shared helper rather than copy-pasting hyphen-to-underscore logic,
2. preserve `anyhow::Context` on stream/transport boundaries,
3. do not weaken `IdentityTuple::validate()` just to accept bad emitted values,
4. if the human renderer cannot decode a structured event, print a useful message rather than an empty line,
5. keep the `--json` envelope contract stable.

## Testing Strategy

- **Unit/schema tests**:
  - prove shared normalization converts emitted runtime client ids like `codex-host` into validator-safe tuple ids such as `codex_host`,
  - prove direct deserialization of invalid kebab-case tuple ids still fails if raw invalid payloads are supplied intentionally,
  - prove existing identity/dotted-id validation rules remain intact.
- **Renderer/control tests**:
  - add plain human `agent start` streaming coverage,
  - add plain human `agent turn` streaming coverage if the same renderer path is exercised separately,
  - keep the existing `--json` acceptance/event/completed streaming contract green.
- **Regression expectations**:
  - the plain human path must surface at least one streamed event line before the completion summary,
  - the final completion summary line must still print,
  - `stderr` event behavior must remain unchanged,
  - `--json` output must remain envelope-based and unchanged in shape.
- **Manual verification**:
  - run the plain repro and confirm commentary/tool/status lines appear before the final summary,
  - run the `--json` control repro and confirm the existing envelope stream is unchanged.

## Boundaries

- Always:
  - preserve the strict identity-tuple validation contract,
  - preserve the `--json` public prompt envelope contract,
  - add regression coverage for the plain human path before or alongside the fix,
  - keep `agent start` and `agent turn` behavior aligned when they share the same renderer seam.
- Ask first:
  - changing the terminal summary line format,
  - changing persisted `agent_id` or `backend_id` semantics outside tuple normalization,
  - widening this slice into unrelated placement-aware compatibility retirement work,
  - changing trace schema or operator-facing docs beyond what the streaming fix requires.
- Never:
  - fix the bug only by weakening validation,
  - silently swallow structured events on the human path,
  - regress `--json` streaming to make the human path easier to patch,
  - replace live streaming with buffered-at-end output and call it equivalent.

## Success Criteria

1. Plain human `substrate agent start` prints streamed event lines before the final completion summary.
2. The same shared non-JSON renderer path used by public `agent turn` is covered and does not silently drop structured events.
3. `AgentEvent` payloads emitted by runtime-owned public prompt flows deserialize cleanly under current identity validation rules.
4. `agent start --json` remains wire-compatible with current accepted/event/completed envelopes.
5. Regression tests fail before the fix and pass after it for both the human path and the shared identity/event seam.
6. The renderer no longer has an empty-string failure mode for structured prompt events.

## Open Questions

1. Should the defensive renderer fallback land in the same slice even if shared identity normalization alone makes the human path green?
   - Default for this spec: yes, if the fallback can be added without altering the JSON contract or broadening scope.
