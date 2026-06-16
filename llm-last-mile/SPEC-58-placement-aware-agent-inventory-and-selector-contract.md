# Spec: Placement-Aware Agent Inventory And Selector Contract

Source dossier: [SESSION_DECISION_DOSSIER-agent-placement-shape-and-world-runtime-gap.md](./SESSION_DECISION_DOSSIER-agent-placement-shape-and-world-runtime-gap.md)  
Related authorities:
- [CODEX_WORLD_DISPATCH_GAP_WRITEUP.md](../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md)
- [SPEC-shell-owned-uaa-runtime-family-alias-support.md](./SPEC-shell-owned-uaa-runtime-family-alias-support.md)
- [docs/CONFIGURATION.md](../docs/CONFIGURATION.md)
- [`config/agents/codex.yaml`](../config/agents/codex.yaml)
- [`config/agents/codex_world.yaml`](../config/agents/codex_world.yaml)
- [`crates/shell/src/execution/config_model.rs`](../crates/shell/src/execution/config_model.rs)
- [`crates/shell/src/execution/agent_inventory.rs`](../crates/shell/src/execution/agent_inventory.rs)
- [`crates/transport-api-types/src/lib.rs`](../crates/transport-api-types/src/lib.rs)
Phase: `SPECIFY`  
Status: draft for review

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The current repo truth already settled that the active `cli:codex_world` gap is guest runtime realizability, not lost world binding.
2. The runtime-family alias slice is already landed baseline truth, so this slice must preserve `config.cli.runtime_family` rather than reopen it.
3. The next honest seam is the logical-agent inventory/selector shape, not guest runtime provisioning itself.
4. Exact backend ids must stay fail-closed and placement-explicit; host and world must not silently co-resolve.
5. The current backend-id grammar remains `<kind>:<name>` with exactly one colon, so placement must live in the name portion rather than a second colon segment.
6. A materially new inventory shape should use a new inventory version instead of overloading the current split-entry `version: 1` contract.

If any of these are wrong, correct them before implementation.

## Objective

Freeze the durable inventory and selector contract for one logical agent that can expose multiple concrete placements.

This slice must answer:

1. how one logical agent is represented in inventory,
2. how concrete host/world placements are represented inside that logical agent,
3. how exact backend ids derive for those placements,
4. how human-facing labels differ from exact selectors,
5. and how exact-selection semantics stay fail-closed once one logical agent can realize more than one placement.

This slice does **not** define the guest-runtime/bootstrap contract yet. It only defines where placement-local runtime truth belongs and how selectors/labels derive.

## Tech Stack

- Rust workspace (`cargo`)
- `crates/shell` inventory, selector, validator, and control-surface code
- `crates/transport-api-types` backend-id selector grammar
- YAML agent inventory under `config/agents/`
- `docs/CONFIGURATION.md` for public operator/config contract
- `llm-last-mile/` for spec/plan/tasks authority

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

Targeted inventory/selector tests once implemented:

```bash
cargo test -p shell agent_inventory -- --nocapture
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell agent_runtime::validator -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
```

Targeted repo-truth checks:

```bash
rg -n "codex_world|execution\\.scope|runtime_family|backend_id|allowed_backends" \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs
```

## Project Structure

```text
config/agents/
  Current split host/world logical-agent inventory files that this slice replaces.

crates/shell/src/execution/agent_inventory.rs
  Inventory schema, projection, and derived realized backend identity.

crates/shell/src/execution/config_model.rs
  Current execution-scope contract; important because `scope` is singular today.

crates/shell/src/execution/agent_runtime/dispatch_contract.rs
  Exact backend selection and resolved launch contract construction.

crates/shell/src/execution/agent_runtime/validator.rs
  Runtime realizability and exact backend validation.

crates/transport-api-types/src/lib.rs
  Exact backend-id grammar that constrains selector spelling.

docs/CONFIGURATION.md
  Public config and selector contract.

llm-last-mile/
  Spec/plan/tasks authority for the placement-aware redesign.
```

## Current Repo-Truth Gut Check

### 1. The repo still models placement as duplicate logical agents

Current inventory truth is still split across separate files such as:

1. `config/agents/codex.yaml`
2. `config/agents/codex_world.yaml`

Those files already share the same runtime family and most of the same capabilities, which is evidence that the split is mostly placement modeling rather than truly distinct logical-agent identity.

### 2. Exact selector identity is already non-negotiable

Current runtime and policy posture already depends on:

1. singular `execution.scope`,
2. exact derived `backend_id`,
3. fail-closed backend selection,
4. explicit host/world separation.

This slice must preserve those invariants while changing the inventory container shape.

### 3. The actual runtime blocker is adjacent but separate

`CODEX_WORLD_DISPATCH_GAP_WRITEUP.md` already proves the remaining `codex_world` failure is guest runtime bootstrap via a host-resolved NVM path. That means inventory cleanup alone is not sufficient, but the cleanup is still needed before the world-runtime contract can be expressed cleanly.

## Contract

### 1. Introduce a placement-aware logical-agent inventory shape

Add a new inventory schema version for logical agents with embedded placements:

```yaml
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: true
      cli:
        binary: codex
        mode: persistent
        runtime_family: codex
      capabilities:
        session_start: true
        session_resume: true
        session_fork: true
        session_stop: true
        status_snapshot: true
        event_stream: true
        llm: true
        mcp_client: false
    world:
      enabled: true
      cli:
        binary: codex
        mode: persistent
        runtime_family: codex
      capabilities:
        session_start: true
        session_resume: true
        session_fork: true
        session_stop: true
        status_snapshot: true
        event_stream: true
        llm: true
        mcp_client: false
```

Rules:

1. top-level `id` is the **logical agent id**,
2. `config.kind` and `config.protocol` remain logical-agent-wide,
3. `placements.<placement>` is where concrete realized launch truth lives,
4. placement names are currently `host` and `world`,
5. placement-local `cli` and `capabilities` are the realized truth for that exact placement.

### 2. Do not model multiple placements as `execution.scope: [host, world]`

Rejected shape:

```yaml
execution:
  scope: [host, world]
```

Reason:

1. the repo already proves placement-specific runtime truth matters,
2. a list-valued `scope` only says “available in multiple places,”
3. it does not provide a placement-local container for binary/runtime/capability truth,
4. and it invites silent host/world blurring.

### 3. Derive realized placement identity explicitly

For each enabled placement row, derive:

1. `logical_agent_id = <top-level id>`
2. `placement = host | world`
3. `realized_agent_id = <logical_agent_id>-<placement>`
4. `backend_id = <kind>:<realized_agent_id>`

Examples:

1. logical agent `codex` + placement `host` -> realized agent id `codex-host` -> backend id `cli:codex-host`
2. logical agent `codex` + placement `world` -> realized agent id `codex-world` -> backend id `cli:codex-world`

This keeps the current exact-backend invariant while removing duplicate logical-agent files.

### 4. Separate human-facing labels from exact selectors

Human-facing labels should be derived display labels:

1. `codex (host)`
2. `codex (world)`

Exact control-plane selectors remain:

1. `cli:codex-host`
2. `cli:codex-world`

Rules:

1. status and suggestion surfaces may group by logical agent and display label,
2. backend-targeted control actions continue to use exact `backend_id`,
3. human-facing labels must not become implicit exact selectors.

### 5. Exact selection remains fail-closed

Once one logical agent can realize more than one placement:

1. exact backend-targeted paths still require an exact backend id,
2. omitted-placement selection must not silently choose host or world,
3. if a future UX wants logical-agent shorthand, it must still fail closed when more than one placement is enabled,
4. policy allowlisting remains keyed on exact realized `backend_id`, not logical-agent id.

### 6. Placement-local runtime truth belongs under the placement, not above it

This slice does **not** yet freeze the guest-runtime/bootstrap contract, but it does freeze the ownership boundary:

1. any world-only runtime/dependency/bootstrap truth must live inside the `world` placement subtree,
2. it must not be hidden in logical-agent-wide fields,
3. it must not be implied only by `placement = world`,
4. it must not be pushed into policy as a selector surrogate.

That is the architectural handoff to the next slice.

### 7. Migration posture

This spec assumes a bounded migration window:

1. existing split `version: 1` files remain supported until conversion lands,
2. the placement-aware schema is the forward contract,
3. exact backend ids will eventually move from `cli:codex` / `cli:codex_world` to `cli:codex-host` / `cli:codex-world`,
4. migration planning must cover inventory, docs, policies, fixtures, status surfaces, and follow-up syntax before implementation cutover.

This spec does **not** promise silent compatibility forever for the old exact ids.

## Code Style

Prefer explicit derived placement records instead of stringly typed host/world suffix heuristics.

```rust
ProjectedPlacementEntryV2 {
    logical_agent_id: "codex".to_string(),
    placement: AgentPlacement::World,
    realized_agent_id: "codex-world".to_string(),
    backend_id: "cli:codex-world".to_string(),
    display_label: "codex (world)".to_string(),
    cli_runtime_family: Some(AgentCliRuntimeFamily::Codex),
}
```

Conventions:

1. reserve `logical_agent_id` for the file/container identity,
2. reserve `realized_agent_id` for the placement-qualified identity,
3. reserve `backend_id` for exact control-plane selection,
4. derive display labels rather than storing duplicated host/world strings,
5. keep placement-local runtime truth typed and explicit.

## Testing Strategy

Framework:

1. Rust unit tests inline with source modules.
2. Shell integration suites in `crates/shell/tests/`.

Coverage expectations:

1. `version: 2` placement-aware inventory parses and projects correctly,
2. disabled placements do not realize backend rows,
3. realized `backend_id` derivation is `cli:<logical>-<placement>`,
4. exact backend validation continues to reject ambiguous or invalid selectors fail-closed,
5. policy allowlists continue to operate on exact realized backend ids only,
6. host/world display labels stay human-facing and do not replace exact selectors,
7. legacy split `version: 1` inventory remains readable during migration if the implementation chooses a transition window.

## Boundaries

- Always:
  - keep exact backend identity placement-explicit
  - keep host and world as separate realized backends
  - keep placement-local runtime truth inside the placement subtree
  - preserve `config.cli.runtime_family` as explicit runtime-realization truth
- Ask first:
  - changing backend-id grammar away from the current single-colon contract
  - making logical-agent shorthand a control-plane selector
  - dropping `version: 1` support in the same packet as schema introduction
- Never:
  - reintroduce `execution.scope: [host, world]` as the only multi-placement model
  - silently default a multi-placement logical agent to host or world
  - treat duplicated `*_world` files as the preferred long-term product shape
  - imply guest-runtime/bootstrap truth from placement name alone

## Success Criteria

This spec is successful only when a fresh implementer can answer all of these from the file alone:

1. what the forward inventory shape is for one logical agent with host/world placements,
2. how exact backend ids derive from that shape,
3. what human-facing labels should look like,
4. why `execution.scope: [host, world]` is rejected,
5. where future world-runtime/bootstrap truth must live,
6. and what migration boundary still remains before code changes begin.

## Open Questions

1. What exact placement-local key should hold world-runtime bootstrap/dependency requirements in the follow-on slice?
2. Should implementation keep a temporary compatibility alias path for `cli:codex` and `cli:codex_world`, or do a one-step exact-selector cutover once migration is ready?
3. Which read-only UX surfaces should group by logical agent versus emit one row per realized placement by default?
