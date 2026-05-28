# Spec: Shell-Owned UAA Runtime Family Alias Support

## Assumptions

ASSUMPTIONS I'M MAKING:
1. The real user-facing goal is to support exact backend aliases such as `cli:codex_world` without weakening the existing exact-`backend_id` routing and policy model.
2. The correct fix should preserve the current host-rooted world-start architecture from slices 28.5, 29, 29.75, 30, and the draft 31 direction.
3. The current bug is not just a bad error message; it is an architectural mismatch where runtime family is being inferred from literal inventory `agent_id`.
4. This is greenfield for the intended contract, so no compatibility fallback or migration posture should shape the design.
5. We should not infer runtime family from binary path, backend-id naming conventions, provider tuple, or other heuristics.
6. The preferred solution is to make shell-owned UAA runtime family an explicit required property of CLI inventory entries used by the shell-owned runtime-realizability path.

If any of these are wrong, correct them before planning or implementation.

## Objective

Fix shell-owned UAA runtime realization so Substrate can support aliased exact backends such as `cli:codex_world` and `cli:claude_code_world` honestly, without conflating:

1. selected backend identity,
2. inventory agent identity,
3. runtime family / adapter family,
4. host versus world execution role.

The correct architecture is:

1. Exact selection remains keyed by derived `backend_id` in `<kind>:<name>` form.
2. Runtime family is a separate resolved property that tells the shell-owned runtime which supported family to instantiate after exact selection succeeds.
3. The shell-owned runtime family for CLI-backed persistent agent-session entries is declared explicitly in inventory, not guessed from literal `agent_id`.
4. Persisted and wire-visible runtime-family state remains canonical as `codex` or `claude_code`; aliases remain exact backend identity only.
5. All shell-owned UAA inventory entries must declare runtime family explicitly; there is no legacy agent-id fallback path.

Primary operator story:

1. A host-scoped orchestrator entry may remain `id: codex`, `backend_id: cli:codex`.
2. A separate world-scoped entry may be `id: codex_world`, `backend_id: cli:codex_world`.
3. Both entries may point at the same underlying binary.
4. Both entries may declare the same runtime family, for example `runtime_family: codex`.
5. Exact routing, policy allowlisting, status, and persisted session truth continue to preserve `cli:codex` versus `cli:codex_world` as distinct backends.

## Architectural Decision

### New source of truth

Introduce an explicit required CLI inventory field for shell-owned runtime-family resolution:

```yaml
version: 1
id: codex_world
config:
  kind: cli
  protocol: substrate.agent.session
  execution:
    scope: world
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

`runtime_family` is the explicit shell-owned UAA adapter/runtime family. Initial supported values:

1. `codex`
2. `claude_code`

### Config and policy alignment

`runtime_family` must live in agent inventory at `config.cli.runtime_family`.

It must not be added to:

1. top-level Substrate config in `config_model.rs`,
2. policy surfaces in `policy_model.rs`,
3. backend allowlist or tuple-constraint policy families.

Rationale:

1. The repo's typed architecture already treats agent inventory as the home for backend-specific runtime realization details.
2. Policy is keyed on exact `backend_id` selectors such as `cli:codex_world`, and that should remain unchanged.
3. `runtime_family` is not a routing or authorization axis; it is a post-selection realization detail for the shell-owned UAA runtime.
4. Keeping it in `config.cli` matches current ergonomics because `binary` and `mode` already live there as CLI runtime-realization fields.

### Requirement scope

`config.cli.runtime_family` is required only when a selected inventory entry is attempting to enter the shell-owned UAA runtime-realizability path.

Concretely, that means the selected entry has:

1. `config.kind = cli`
2. `protocol = substrate.agent.session`
3. effective `cli.mode = persistent`

It is not required for unrelated CLI entries outside that runtime path.

### Why this is the correct fix

This design preserves the repo's current architecture:

1. `backend_id` remains the exact selector and policy token.
2. `agent_id` remains the inventory identity and operator-facing name component.
3. `runtime_family` becomes the shell-owned runtime realization input.
4. `binary` remains a runtime prerequisite, not the selector or identity source.
5. The field lands in the same typed inventory object that already owns CLI runtime details, preserving existing shape and ergonomics.

This matches the stable contract surfaces:

1. `backend_id` is an adapter selector only, not an overloaded identity label.
2. exact backend selection remains fail-closed,
3. host/world role split remains separate from runtime family,
4. persisted attach truth and world binding truth remain authoritative.

### Rejected alternatives

Do not:

1. infer runtime family from literal `agent_id`,
2. infer runtime family from `backend_id` naming patterns such as `_world`,
3. infer runtime family from `config.cli.binary`,
4. infer runtime family from provider/router/protocol tuple fields,
5. widen support to all `kind=cli` entries without an explicit runtime-family contract.
6. add a temporary fallback that maps literal `agent_id` values like `codex` or `claude_code` to runtime family.
7. add `runtime_family` to policy or top-level config instead of inventory.

Those options are either brittle, violate existing backend-selection architecture, or hide new product semantics inside heuristics.

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
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell agent_runtime::validator -- --nocapture
```

Focused local smoke once implemented:

```bash
substrate agent doctor --json
substrate agent start --backend cli:codex --scope host --prompt "host ok" --json
substrate agent start --backend cli:codex_world --scope world --prompt "world ok" --json
substrate agent turn --session <session_id> --backend cli:codex_world --prompt "next" --json
```

## Project Structure

Relevant implementation seams:

```text
crates/shell/src/execution/agent_inventory.rs
  Inventory schema and derived backend identity.

crates/shell/src/execution/agent_runtime/mapping.rs
  Runtime-family enum and family-resolution helpers.

crates/shell/src/execution/agent_runtime/dispatch_contract.rs
  Exact backend selection and resolved launch contract construction.

crates/shell/src/execution/agent_runtime/validator.rs
  Doctor/runtime-realizability checks for orchestrator and member selection.

crates/shell/src/execution/agent_runtime/control.rs
  Canonical resolved runtime descriptor and runtime-family serialization boundary.

crates/shell/src/execution/agent_runtime/orchestration_session.rs
  HostAttachContract persistence/reconstruction.

crates/shell/src/repl/async_repl.rs
  World-member bootstrap parity and retained-member reconstruction.

crates/shell/src/execution/prompt_fulfillment.rs
  Runtime-family to concrete runtime/client dispatch.

docs/CONFIGURATION.md
  Public configuration and runtime-realizability contract.

llm-last-mile/
  Slice/spec alignment docs for the fix.
```

## Code Style

Use explicit, typed resolution helpers instead of stringly typed identity checks.

Preferred style:

```rust
fn resolve_shell_owned_runtime_family(
    entry: &ProjectedInventoryEntryV1,
) -> Result<AgentRuntimeBackendKind> {
    entry.cli_runtime_family.ok_or_else(|| {
        anyhow::anyhow!(
            "selected runtime '{}' is not runtime-realizable because config.cli.runtime_family is required for shell-owned UAA runtimes",
            entry.agent_id
        )
    })
}
```

Conventions:

1. Prefer `runtime_family` for config/schema naming.
2. Reserve `backend_id` for exact selector identity.
3. Reserve `agent_id` for inventory identity.
4. Keep canonical persisted runtime-family strings as `codex` and `claude_code`.
5. Fail closed with explicit contract-language errors when runtime-family truth is missing or unsupported.

## Testing Strategy

Frameworks:

1. Rust unit tests inline with source modules.
2. Shell integration suites in `crates/shell/tests/`.

Coverage expectations:

1. Unit coverage for runtime-family resolution from inventory config, including fail-closed rejection when the field is missing.
2. Unit coverage for exact backend selection preserving `agent_id` / `backend_id` while resolving shared runtime family.
3. Integration coverage for host orchestrator `codex` plus world member `codex_world`.
4. Regression coverage for doctor, status, REPL retained-member reuse, and targeted world turns.
5. No change to canonical persisted/wire runtime-family spellings unless a compatibility migration is explicitly designed and tested.

Required regression cases:

1. `validate_member_selection(...)` returns `backend_kind = Codex` for `id: codex_world` with `cli.runtime_family: codex`.
2. `substrate agent doctor --json` succeeds with host orchestrator `codex` and world runtime alias `codex_world`.
3. `substrate agent start --backend cli:codex_world --scope world ...` selects the exact world backend and keeps canonical host/world truth.
4. `::cli:codex_world` follow-up reuses or relaunches the exact retained world backend rather than collapsing to `cli:codex`.
5. Persisted session manifests keep `resolved_agent_kind = "codex"` while preserving `backend_id = "cli:codex_world"` and `agent_id = "codex_world"`.

## Boundaries

- Always:
  - Keep exact backend selection keyed on `backend_id`.
  - Keep runtime-family serialization canonical as `codex` / `claude_code`.
  - Preserve fail-closed behavior when runtime-family truth is missing, invalid, or unsupported.
  - Preserve host-rooted authority and host-first inaugural prompt routing.
  - Preserve policy semantics keyed on exact `backend_id` only.

- Ask first:
  - Renaming persisted fields like `resolved_agent_kind`.
  - Changing wire enums in `world-api`.
  - Broadening shell-owned runtime support beyond `codex` and `claude_code`.
  - Introducing a different config field name than `config.cli.runtime_family`.
  - Promoting `runtime_family` into policy or top-level config.

- Never:
  - Infer runtime family from binary path or alias naming conventions.
  - Collapse distinct exact backends into a shared backend identity.
  - Change allowlist semantics away from exact `backend_id`.
  - Reopen slice-30 architecture to make world runtime the durable authority.

## Success Criteria

This work is successful only when all of the following are true:

1. An aliased CLI inventory entry such as `id: codex_world` can be runtime-realizable when it explicitly declares `config.cli.runtime_family: codex`.
2. Any shell-owned UAA inventory entry missing `config.cli.runtime_family` fails closed as not runtime-realizable.
3. Exact backend selection, policy allowlisting, status, and session truth continue to distinguish `cli:codex` from `cli:codex_world`.
4. The shell-owned runtime family is no longer inferred from literal inventory `agent_id` in production launch-contract resolution.
5. Canonical persisted/wire runtime-family values remain `codex` and `claude_code`; aliases do not leak into runtime-family serialization.
6. Doctor, start, retained-member reuse, and targeted world-turn flows all have regression coverage for the alias topology.
7. Policy schemas and allowlist semantics remain unchanged and continue to key only on exact `backend_id`.
8. Public config docs describe shell-owned runtime realizability in terms of explicit runtime family plus exact backend identity, not literal supported backend ids.

## Open Questions

1. Should the public config doc say “supported shell-owned runtime families are `codex` and `claude_code`,” or should it frame them as “supported `config.cli.runtime_family` values”?
