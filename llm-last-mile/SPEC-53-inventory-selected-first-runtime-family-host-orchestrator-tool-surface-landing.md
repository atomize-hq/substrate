# Spec: Inventory-Selected First Runtime-Family Host-Orchestrator Tool Surface Landing

Source tracker note: [REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md](./REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md)  
Prior numbered slice: [SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md](./SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md)  
Companion inputs:
- [PLAN-52.md](./PLAN-52.md)
- [TASKS-52.md](./TASKS-52.md)
- [DESIGN-host-orchestrator-tool-invocation-surface.md](./DESIGN-host-orchestrator-tool-invocation-surface.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
Phase: `SPECIFY`  
Status: drafted on `2026-06-09` after repo-truth gut check of live orchestrator selection, runtime-family realization, and current host prompt-submission surfaces.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The next honest slice after Slice `52` is still the first live runtime-family adapter landing above the frozen adapter contract.
2. The live choice of host orchestrator is **not** hard-coded to `codex`; it is selected from effective config plus effective agent inventory.
3. Effective agent inventory is discovered dynamically from:
   - `$SUBSTRATE_HOME/agents/*.yaml`
   - `<workspace_root>/.substrate/agents/*.yaml`
   with later workspace entries overriding global entries by exact `agent_id`.
4. Policy remains exact-`backend_id` authority. Runtime realization still happens only after exact backend selection and policy allowlisting succeed.
5. The selected runtime family comes from inventory truth at `config.cli.runtime_family`, with currently supported values `codex` and `claude_code`.
6. This slice should use the Codex-backed path as the first real smoke/validation floor for live host tool exposure while keeping selection semantics dynamic. That should not by itself force `codex` selection or require non-Codex runtimes to be blocked unless implementation truth proves a real incompatibility.
7. Slice `53` should reuse the frozen seven-tool contract from Slice `52` and should not reopen tool names, injected-field ownership, or receipt semantics.
8. Slice `53` should not widen into public `substrate agent toolbox <verb>` execution, MCP-as-source-of-truth, or `claude_code` parity.
9. The current generic `unified-agent-api` run surface does not expose a first-class generic tool catalog, and the relevant design docs clarify that the toolbox already exists as a runtime-owned endpoint exported through `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` plus `SUBSTRATE_AGENT_TOOLBOX_VERSION`. Because the first landing is not an MCP server registration, Slice `53` should therefore use a two-part adapter path: runtime-owned env-var endpoint injection into the selected host runtime plus startup/system-prompt injection of the seven-tool contract details. Any Codex-specific config shaping stays secondary and bounded via UAA.

If any of these are wrong, correct them before implementation.

## Objective

Land the first **live** host-orchestrator tool surface on top of the Slice `52` contract freeze while preserving the repo's existing dynamic selection model.

## Tech Stack

- Rust workspace (`cargo`)
- `crates/shell` host-runtime selection, submission, reporting, and adapter wiring
- `crates/gateway` runtime-family adapter registration
- `unified-agent-api` run surface (`AgentWrapperRunRequest`)
- Existing session-scoped internal toolbox transport in `crates/shell/src/repl/async_repl.rs`

This slice is complete only when:

1. the host orchestrator is still selected by `agents.hub.orchestrator_agent_id` against the effective inventory,
2. the effective inventory is still sourced dynamically from user/workspace agent inventory roots,
3. the exact selected backend is still policy-gated by `agents.allowed_backends`,
4. the selected orchestrator's `config.cli.runtime_family` becomes the only runtime-family input for the new live tool surface,
5. the Codex-backed path is the first runtime path with a live, callable seven-tool surface wired to the frozen Slice `52` contract and covered by real smoke/validation,
6. the selected host runtime receives authoritative toolbox endpoint/version injection via env vars,
7. the selected host runtime also receives startup/system-prompt tool-contract details because the first landing is not MCP-server-based tool registration,
8. those tool calls route through the landed internal toolbox transport with runtime-owned request/session/caller/world injection,
9. tool results preserve Slice `52` receipt semantics for `task_run_id` and retained-worker handles,
10. non-Codex selected orchestrators keep existing prompt-turn behavior and their host-tool posture is reported truthfully as validated / not-yet-validated / not-guaranteed rather than being blocked by doc wording alone,
11. the slice lands without public toolbox verb expansion, without `claude_code` parity, and without reopening transport design.

Primary user story after this slice:

1. A user can select any orchestrator dynamically through config/inventory/policy as they do today.
2. The Codex-backed path proves one end-to-end validated landing where the live host session can actually call the frozen world-dispatch tools.
3. Other selected runtimes remain governed by the same selection truth, and the repo reports their host-tool validation/support posture honestly instead of silently swapping runtimes or hard-coding `codex`.

## Repo-Truth Gut Check

### 1. Host orchestrator selection is config-driven and inventory-backed

Live `validate_orchestrator_selection(...)` in [`validator.rs`](../crates/shell/src/execution/agent_runtime/validator.rs) selects the orchestrator from `effective_config.agents.hub.orchestrator_agent_id` and then resolves that id against the effective inventory.

Repo-truth consequence:

1. Slice `53` must not hard-code `codex` as the selected orchestrator id.
2. Any codex-first scope statement must mean **first validated runtime-family exposure mechanics**, not hard-coded orchestrator choice.

### 2. Available agent options are discovered dynamically from inventory roots

Live `discover_agent_inventory_roots(...)`, `discover_agent_files(...)`, and `load_effective_agent_inventory(...)` in [`agent_inventory.rs`](../crates/shell/src/execution/agent_inventory.rs) read agent files from `$SUBSTRATE_HOME/agents/` plus workspace `.substrate/agents/`.

Repo-truth consequence:

1. the available orchestrator candidates are already dynamic,
2. workspace inventory can override global inventory by exact `agent_id`,
3. Slice `53` should preserve this discovery model and consume it rather than introducing a new hard-coded runtime registry.

### 3. Policy remains exact-backend authority, not runtime-family authority

Live `resolve_inventory_projected_contract(...)` in [`dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs) still enforces `base_policy.agents_allowed_backends` against the exact derived `backend_id` before runtime realization.

Repo-truth consequence:

1. `runtime_family` must stay a post-selection realization detail,
2. Slice `53` must not repurpose `runtime_family` into a new allowlist axis,
3. exact backend ids like `cli:codex` and `cli:codex_world` must remain distinct even if both resolve to canonical runtime family `codex`.

### 4. Runtime-family realization is already inventory-truth-driven

Live `resolve_shell_owned_runtime_family(...)` in [`mapping.rs`](../crates/shell/src/execution/agent_runtime/mapping.rs) resolves canonical runtime family from `config.cli.runtime_family`, and [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md) documents supported values as `codex` and `claude_code`.

Repo-truth consequence:

1. Slice `53` should derive first validated behavior and truthful reporting from resolved runtime family,
2. the new slice should not infer family from literal `agent_id`, `backend_id`, or binary name,
3. family-specific landing order can stay codex-first without corrupting selection truth.

### 5. Current host prompt submission is family-aware, but not yet tool-surface-aware

Live `submit_host_prompt_turn(...)` in [`control.rs`](../crates/shell/src/execution/agent_runtime/control.rs) delegates through [`prompt_fulfillment.rs`](../crates/shell/src/execution/prompt_fulfillment.rs), which already switches by resolved backend kind (`Codex` vs `ClaudeCode`) via `PromptFulfillmentBridge::for_descriptor(...)`. The current `AgentWrapperRunRequest` is created there with an explicit `env` map and a single prompt string, while public-start / resumed-start flows stage startup prompt text through `HiddenOwnerHelperStartupPromptPlan.prompt_text` in [`agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs).

Repo-truth consequence:

1. Slice `53` has natural family-specific seams for both env injection and prompt augmentation above selection and below prompt submission,
2. the first live landing should happen at this boundary rather than by changing inventory resolution.

### 6. Dependency truth points toward runtime-owned endpoint injection, not direct tool-list injection

Current `unified-agent-api` `AgentWrapperRunRequest` carries:

1. `prompt`
2. `working_dir`
3. `timeout`
4. `env`
5. `extensions`

It does **not** currently carry a first-class generic tool catalog on the run request surface. The relevant design docs also make clear that the missing live tool surface is supposed to sit **above** the already-landed toolbox transport, whose endpoint/version are already published by runtime-owned env hints (`SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`, `SUBSTRATE_AGENT_TOOLBOX_VERSION`) for a live host-scoped orchestrator session. DeepWiki review of `atomize-hq/unified-agent-api` and `openai/codex` points in the same direction: Codex does not appear to want a direct per-request tool list; instead it can be shaped through runtime launch config while the actual tool capability already exists behind the endpoint.

Repo-truth consequence:

1. Slice `53` should prefer runtime-owned env-var endpoint injection as the first-family discovery/binding seam,
2. because the first landing is not MCP-server-based registration, Slice `53` should also inject the seven-tool contract details into the startup/system prompt so the host runtime knows the available tool names, input semantics, runtime-owned fields, and receipt/follow-up rules,
3. any Codex-specific config, override, or home shaping through UAA should be in service of consuming that injected endpoint rather than replacing it with a second synthetic tool-definition source,
4. if that still requires a bounded dependency widening, the spec/plan/tasks should name it explicitly.

## Architectural Decision

### Dynamic selection stays authoritative

The selected host orchestrator for the live tool surface remains:

1. chosen by `agents.hub.orchestrator_agent_id`,
2. resolved against the effective inventory,
3. constrained by effective policy,
4. realized through `config.cli.runtime_family` only after exact selection succeeds.

This slice must not introduce:

1. a hard-coded preferred orchestrator id,
2. a hidden fallback from `claude_code` to `codex`,
3. a second runtime-family selection config outside inventory.

### Codex is the first validated family path, not the only selectable family

For Slice `53`:

1. the Codex-backed path is the first runtime path that must be proven by real smoke/validation,
2. `claude_code` remains a supported runtime family for ordinary host sessions,
3. this slice does not, by itself, require non-Codex runtimes to be hard-blocked from future adapter reuse unless implementation truth proves that is necessary.

The truthful posture is:

1. ordinary host-session launch should keep working for non-codex families,
2. the Codex-backed path is the first guaranteed/validated floor for the new host tool surface,
3. non-Codex runtimes should report validation/support posture honestly rather than overclaiming parity,
4. any actual disablement should come from implementation evidence rather than from documentation drift alone,
5. no implicit family switching is permitted.

### The Slice 52 contract remains the semantic source of truth

The live runtime-family landing must consume the frozen adapter contract in [`tool_invocation_contract.rs`](../crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs).

It must not:

1. rename the seven tools,
2. ask the model to supply runtime-owned ids,
3. alter receipt shapes,
4. redesign internal dispatch transport.

### First validated landing should use a two-part adapter: runtime-owned env injection plus prompt-side tool disclosure

Because current prompt submission goes through a generic `AgentWrapperRunRequest` without first-class tool definitions, and because the design inputs freeze that the toolbox already exists as a session-scoped endpoint, Slice `53` should prefer one bounded two-part adapter path:

1. inject `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` and `SUBSTRATE_AGENT_TOOLBOX_VERSION` into the selected host runtime at launch / resume time,
2. augment the startup/system prompt with the seven-tool contract details needed for non-MCP discovery: tool names, purpose, agent-provided fields, runtime-owned injected fields, receipt semantics, and exact follow-up handles.

Any Codex-specific config, overrides, or isolated `CODEX_HOME` state should only be used as needed so the first validated Codex-backed path can consume that injected endpoint and prompt contract honestly, while keeping the adapter generic where that is already honest.

That path is acceptable so long as:

1. dynamic selection remains unchanged,
2. Slice `52` semantics remain unchanged,
3. the resulting user-visible tool contract stays runtime-family-independent at the semantic level,
4. any dependency widening stays explicitly bounded and documented,
5. the slice does not pretend Codex accepts a direct generic per-request tool list when the underlying runtime truth is runtime-owned endpoint injection over the already-landed toolbox transport,
6. the prompt augmentation is treated as complementary discovery/help text, not as a replacement for the real runtime-owned endpoint and binding injection.

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

Targeted tests:

```bash
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell agent_runtime::dispatch_contract -- --nocapture
cargo test -p shell agent_runtime::tool_invocation_contract -- --nocapture
```

Focused operator-truth checks once implemented:

```bash
substrate agent doctor --json
substrate agent toolbox status --json
substrate agent toolbox env --json
```

## Project Structure

Relevant implementation seams:

```text
crates/shell/src/execution/agent_inventory.rs
  Dynamic inventory discovery and projected inventory truth.

crates/shell/src/execution/agent_runtime/validator.rs
  Orchestrator selection and runtime-realizability checks.

crates/shell/src/execution/agent_runtime/dispatch_contract.rs
  Exact backend selection, policy gating, and resolved launch contracts.

crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs
  Frozen seven-tool adapter contract from Slice 52.

crates/shell/src/execution/agent_runtime/control.rs
  Host prompt submission boundary and runtime session ownership; `submit_host_prompt_turn(...)` is the likely `AgentWrapperRunRequest.env` injection seam.

crates/shell/src/execution/prompt_fulfillment.rs
  Family-specific prompt bridge; likely first-family runtime launch seam once toolbox env vars are injected.

crates/shell/src/repl/async_repl.rs
  Landed internal toolbox transport and request/response handling.

crates/shell/src/execution/agents_cmd.rs
  Doctor/toolbox status truth surfaces; public-start / resumed-start flows already stage startup prompt text through `HiddenOwnerHelperStartupPromptPlan.prompt_text`, which is the likely prompt-contract composition seam.

crates/gateway/src/adapter_runtime.rs
  Gateway adapter registration by runtime family.
```

## Code Style

Prefer explicit runtime-family posture handling at the submission boundary while keeping selection truth separate.

Example style:

```rust
match runtime.descriptor.backend_kind {
    AgentRuntimeBackendKind::Codex => build_validated_host_tool_surface(&runtime, tool_context)?,
    AgentRuntimeBackendKind::ClaudeCode => ToolSurfaceAvailability::not_yet_validated(
        "claude_code host-tool parity is not yet smoke-validated in Slice 53",
    ),
}
```

And compose prompt/env injection explicitly:

```rust
let toolbox_env = build_toolbox_env(orchestration_session_id)?;
let tool_prompt = build_tool_contract_prompt(&toolbox_contract)?;
let startup_prompt = compose_startup_prompt(base_prompt, tool_prompt);
// then inject `toolbox_env` into AgentWrapperRunRequest.env and route
// `startup_prompt` through HiddenOwnerHelperStartupPromptPlan.prompt_text
```

And keep selection truth inventory-driven:

```rust
let entry = validate_orchestrator_selection(&effective_config, &inventory)?;
let projected = project_inventory_entry(cwd, entry, &effective_config);
let resolved = resolve_inventory_projected_contract(&base_policy, &envelope, projected)?;
```

Do **not** replace that with string matching like:

```rust
if orchestrator_agent_id == "codex" {
    // wrong: this hard-codes agent identity instead of using inventory truth
}
```

## Testing Strategy

### Unit / focused contract coverage

1. preserve inventory and validator coverage that proves orchestrator selection remains config/inventory-driven,
2. add focused host-runtime tests that derive truthful validation/support posture by resolved runtime family,
3. add focused coverage for toolbox env injection plus startup/system-prompt tool-contract augmentation,
4. prove exact backend ids still survive even when multiple inventory entries realize canonical `codex`.

### Runtime-family landing coverage

1. the Codex-backed path receives toolbox endpoint/version env injection plus startup/system-prompt tool-contract disclosure,
2. a live tool call bridges through Slice `52` translation into internal toolbox transport,
3. returned receipts preserve `task_run_id` / retained-worker handles without family-specific drift,
4. non-Codex runtimes keep existing host prompt behavior and reporting does not overclaim parity or guaranteed tool support.

### Regression / truth-alignment coverage

1. `toolbox status` / `doctor` surfaces do not overclaim live tool-call support beyond what is actually validated,
2. policy denials still key on exact backend ids,
3. no test reintroduces hard-coded `codex` orchestrator identity assumptions,
4. no test turns “Codex is first smoke floor” into “Codex is the only allowed runtime.”

## Boundaries

- Always:
  - Preserve dynamic selection from config + effective inventory + exact backend policy.
  - Reuse Slice `52` tool names, injected-field ownership, and receipt semantics.
  - Treat startup/system-prompt tool details as a required companion to env injection for the non-MCP first landing.
  - Keep non-Codex validation/support posture explicit and truthful; do not force runtime blocks unless implementation truth requires it.
  - Run focused shell tests plus workspace fmt/clippy before closing the slice.

- Ask first:
  - Any widening of `unified-agent-api` or gateway adapter request surfaces.
  - Any decision to block ordinary host sessions or host-tool exposure for non-Codex runtimes.
  - Any expansion beyond `codex` into dual-family live landing.

- Never:
  - Hard-code `codex` as the selected orchestrator id.
  - Add a hidden fallback from `claude_code` to `codex`.
  - Reopen MCP as the required source of truth for v1.
  - Reopen public toolbox CLI execution in this slice.

## Success Criteria

1. The repo still resolves available orchestrator candidates dynamically from inventory roots under `$SUBSTRATE_HOME` and workspace `.substrate`.
2. The selected host orchestrator is still dictated by effective config/policy/inventory truth, not by hard-coded runtime-family preference.
3. The Codex-backed path proves one end-to-end validated landing where the live host session receives the seven frozen Slice `52` tools and can invoke them end-to-end through the landed internal toolbox transport.
4. Those live tool invocations preserve Slice `52` runtime-owned field injection and receipt semantics.
5. If the selected orchestrator resolves to a non-Codex runtime family, the session keeps existing behavior and surfaces explicit validation/support truth without doc-driven forced fallback or forced blocking.
6. Exact backend ids and allowlist semantics remain unchanged.
7. No public toolbox execution verbs, MCP-first redesign, or claude parity work lands implicitly.

## Open Questions

1. Can the codex-first landing be done by injecting `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` / `SUBSTRATE_AGENT_TOOLBOX_VERSION` into the host runtime through existing UAA env/config surfaces, with only bounded Codex-specific shaping on top, or does it still require a UAA widening?
2. Which operator-visible surface should carry the first validation/support-posture truth first:
   - `substrate agent doctor --json`,
   - `substrate agent toolbox status --json`,
   - or both?
3. Should Slice `53` include a minimal smoke harness for one real Codex-backed host tool call, or keep all live smokes deferred until the later docs/smoke alignment slice?
