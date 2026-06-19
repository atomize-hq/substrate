# Spec: Post-Placement-Aware Compatibility Retirement

Source authorities:
- [CODEX_WORLD_DISPATCH_GAP_WRITEUP.md](../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md)
- [SPEC-58-placement-aware-agent-inventory-and-selector-contract.md](./SPEC-58-placement-aware-agent-inventory-and-selector-contract.md)
- [PLAN-58-placement-aware-agent-inventory-and-selector-contract.md](./PLAN-58-placement-aware-agent-inventory-and-selector-contract.md)
- [TASKS-58.md](./TASKS-58.md)
- [SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)
- [TASKS-59.md](./TASKS-59.md)
- [docs/CONFIGURATION.md](../docs/CONFIGURATION.md)
- [config/agents/codex.yaml](../config/agents/codex.yaml)
- [config/agents/claude_code.yaml](../config/agents/claude_code.yaml)
- [crates/shell/src/execution/agent_inventory.rs](../crates/shell/src/execution/agent_inventory.rs)
- [crates/shell/src/execution/agent_runtime/validator.rs](../crates/shell/src/execution/agent_runtime/validator.rs)
- [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
- [crates/shell/src/execution/policy_model.rs](../crates/shell/src/execution/policy_model.rs)
- [crates/shell/src/repl/async_repl.rs](../crates/shell/src/repl/async_repl.rs)
Phase: `SPECIFY`  
Status: draft for review

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `59` is the landed runtime-truth floor and must not be reopened by this slice.
2. The placement-aware Slice `58` cutover is already reflected in forward repo truth even though [TASKS-58.md](./TASKS-58.md) still carries a stale `draft for review` status line.
3. Slice `60` is a bounded retirement slice for temporary compatibility posture left behind by Slice `58`, especially:
   - the effective-inventory compatibility bridge,
   - split-entry logical-agent assumptions,
   - and legacy exact backend ids such as `cli:codex_world`.
4. Historical planning docs under `llm-last-mile/` may retain old names as historical evidence, but forward operator/product truth must not present old and new identities as coequal.
5. Earlier state-store/session-root compatibility bridges are out of scope unless a surviving split-entry or placement-aware alias dependency forces this slice to touch them directly.
6. Exact backend-id grammar remains `<kind>:<name>` with one colon; this slice retires old names, not the grammar.
7. If implementation uncovers a still-supported persisted runtime/session path that genuinely depends on split-entry aliases, that is a reopen condition rather than a reason to silently keep the bridge forever.

If any of these are wrong, correct them before implementation.

## Objective

Retire the temporary compatibility posture that survived the placement-aware cutover so the repo has one clear forward truth:

1. multi-placement CLI agents are modeled as placement-aware `version: 2` logical-agent manifests,
2. exact selectors are placement-qualified exact backend ids such as `cli:codex-host` and `cli:codex-world`,
3. forward docs/examples/tests/policies no longer describe split-entry ids such as `codex_world` / `cli:codex_world` or unqualified pre-placement exact selectors such as `cli:codex` / `cli:claude_code` as live current truth,
4. and the shell no longer depends on the Slice `58` compatibility bridge that materializes placement-aware inventory back into split-entry-shaped effective rows.

Success means the repo stops carrying both the old pre-placement selector mental model and the new placement-aware model at the same time.

## Tech Stack

- Rust workspace (`cargo`)
- `crates/shell` inventory, selector, validator, REPL, and policy surfaces
- YAML agent inventory under `config/agents/`
- Operator/config documentation under `docs/`
- Shell smoke helpers under `scripts/substrate/`
- Contract and regression tests in `crates/shell/tests/`
- `llm-last-mile/` spec/plan/tasks authority

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

Targeted validation:

```bash
cargo test -p shell agents_validate -- --nocapture
cargo test -p shell agent_inventory -- --nocapture
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell agent_runtime::validator -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Forward-truth grep wall:

```bash
rg -n "\bcodex_world\b|\bclaude_code_world\b|cli:(codex|claude_code)_world\b" \
  docs config crates/shell scripts \
  -g '!target'
rg -nP "\bcli:(codex|claude_code)\b(?!-)" \
  docs config crates/shell scripts \
  -g '!target'
```

Packet `1` interpretation of that wall:

1. `docs/`, `config/`, and `scripts/` are forward-truth surfaces. Any split-entry legacy-name hit or unqualified pre-placement exact-selector hit there is in-scope current-truth debt, not acceptable history.
2. `crates/shell/` stays inside the wall so later packets cannot hide behind a narrower search root, but Packet `1` treats only the explicit allowlisted seams below as packet-owned retirement inventory; any other live `crates/shell/` hit from either grep is a reopen condition rather than proof that the old ids remain supported.
3. `llm-last-mile/` is intentionally outside the grep wall because planning provenance may retain old ids when explicitly historical.
4. The expanded wall is expected to expose active forward-surface Packet `4` debt in `docs/TRACE.md`, `docs/internals/world/gateway_auth_handoff.md`, `docs/reference/world/verification/gateway_auth_handoff.md`, `scripts/linux/world-provision.sh`, and `scripts/mac/smoke.sh`; those hits are not part of the historical allowlist, but they are the bounded Packet `1`-acknowledged forward-surface debt that may remain open until Packet `4`.

Packet `1` historical / retirement allowlist:

1. Packet `2` and Packet `3` retirement seams may keep legacy names until those packets land:
   - `crates/shell/src/execution/agent_inventory.rs`
   - `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
   - `crates/shell/src/execution/agent_runtime/validator.rs`
   - `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. compatibility-adjacent control/state seams may keep legacy names only while they are proving retained-worker continuity, persisted-state compatibility, or fail-closed retirement behavior:
   - `crates/shell/src/builtins/world_gateway.rs`
   - `crates/shell/src/execution/agents_cmd.rs`
   - `crates/shell/src/execution/cli.rs`
   - `crates/shell/src/execution/agent_runtime/control.rs`
   - `crates/shell/src/execution/agent_runtime/auto_attach.rs`
   - `crates/shell/src/execution/agent_runtime/host_inbox.rs`
   - `crates/shell/src/execution/host_inbox_materialization.rs`
   - `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
   - `crates/shell/src/execution/agent_runtime/session.rs`
   - `crates/shell/src/execution/agent_runtime/state_store.rs`
   - `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`
   - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
   - `crates/shell/src/repl/async_repl.rs`
3. Active forward docs/scripts may still hit the expanded Packet `1` wall only in `docs/TRACE.md`, `docs/internals/world/gateway_auth_handoff.md`, `docs/reference/world/verification/gateway_auth_handoff.md`, `scripts/linux/world-provision.sh`, and `scripts/mac/smoke.sh` until Packet `4` normalizes those operator/product surfaces to placement-qualified ids. Those five files are deferred forward-surface debt, not historical evidence and not a reason to mark Packet `1` red.
4. `crates/shell/tests/**` and inline `#[cfg(test)]` coverage may retain legacy names only for explicit bridge-removal coverage, persisted-state continuity coverage, or fail-closed retirement assertions. Packet `4` must shrink those remaining hits to intentional negative/historical coverage only.
5. Any split-entry legacy-name hit or unqualified pre-placement exact-selector hit outside this bounded allowlist is presumed to be forward-surface scope, not an automatic exception.

Packet `1` reopen conditions:

1. If any split-entry legacy-name hit or unqualified pre-placement exact-selector hit in `docs/`, `config/`, or `scripts/` is discovered outside the five Packet `4` forward-surface debt files above, or if one of those five files turns out to require either debt class beyond the planned Packet `4` cleanup, reopen Slice `60` instead of silently preserving dual truth.
2. If any split-entry legacy-name hit or unqualified pre-placement exact-selector hit outside the allowlist above is needed for supported persisted runtime/session continuity, reopen Slice `60` instead of broadening the allowlist ad hoc.
3. If removing an allowlisted hit would force a Slice `59` runtime-semantic change rather than an identity-only change, treat that as a reopen condition.

Placement-aware grep wall:

```bash
rg -n "cli:(codex|claude_code)-(host|world)|config\.placements|version: 2" \
  docs config crates/shell scripts \
  -g '!target'
```

## Project Structure

```text
config/agents/
  Forward placement-aware logical-agent manifests. This slice must keep these as the only live inventory model.

crates/shell/src/execution/agent_inventory.rs
  Owns the temporary Slice 58 compatibility bridge that materializes version-2 placement-aware inventory into split-entry-shaped effective inventory rows.

crates/shell/src/execution/agent_runtime/validator.rs
crates/shell/src/execution/agent_runtime/dispatch_contract.rs
  Exact backend selection and runtime-contract validation that still must reject retired split-entry selectors cleanly after the bridge is removed.

crates/shell/src/execution/policy_model.rs
crates/shell/src/repl/async_repl.rs
  Policy fixtures and REPL/runtime examples that must stop presenting old ids as forward truth.

crates/shell/tests/
  Contract and regression suites that currently pin both placement-aware exact ids and some legacy split-entry aliases.

docs/
  Authoritative operator truth. Historical references are allowed only where explicitly marked as historical.

scripts/substrate/
  Smoke helpers and install-time examples that must align with the final placement-aware identity model.

llm-last-mile/
  Planning authority. Historical slice docs may retain old names when clearly historical, but Slice 60 must replace the placeholder with executable retirement guidance.
```

## Code Style

Use fail-closed retirement logic with explicit guidance instead of silent compatibility fallback.

```rust
return Err(user_error(
    "legacy exact backend 'cli:codex_world' is retired; use 'cli:codex-world'",
));
```

Conventions:

1. prefer explicit names like `legacy_split_backend_id` or `placement_qualified_backend_id`,
2. keep forward-truth tests separate from historical allowlist checks,
3. remove compatibility branches rather than adding new aliasing,
4. keep Slice `59` runtime-truth assertions unchanged unless a contradiction is proven.

## Testing Strategy

- **Unit tests**: inventory materialization, selector derivation, and validator fail-closed behavior.
- **Integration tests**: public control surfaces, REPL routing, and policy/example surfaces that consume exact backend ids.
- **Negative grep validation**: prove forward docs/config/scripts/tests no longer present split-entry ids or unqualified pre-placement exact selectors as live current truth.
- **Regression coverage**: preserve Slice `59` world-runtime truth while retiring old split-entry naming.
- **Manual diff review**: ensure remaining legacy-name hits are historical records only, not forward product truth.

## Boundaries

- Always:
  - preserve Slice `59` runtime-realizability, world-deps, and installer truth,
  - keep exact backend selection fail-closed,
  - make any surviving historical references explicit and bounded,
  - run the grep wall plus targeted shell tests before calling the slice green.
- Ask first:
  - removing or rewriting historical `llm-last-mile/` records instead of leaving them as historical evidence,
  - changing persisted runtime/session compatibility for unsupported old rows,
  - widening the slice into session-store compatibility retirement unrelated to placement-aware identity.
- Never:
  - reintroduce split host/world manifests as forward truth,
  - leave `cli:codex_world` and `cli:codex-world` documented as equivalent current selectors,
  - weaken Slice `59` runtime truth to preserve a naming bridge,
  - silently map a retired split-entry selector onto a placement-qualified selector.

## Current Repo-Truth Gut Check

### 1. Forward product truth is only partially placement-aware

The forward config/docs surface already includes placement-aware `version: 2` manifests and placement-qualified exact backend ids such as `cli:codex-host` and `cli:codex-world`, but active docs/scripts still retain unqualified pre-placement selectors such as `cli:codex` and `cli:claude_code`. Those hits are current forward-surface debt, not historical evidence.

### 2. The compatibility bridge is still present in code

`agent_inventory.rs` still materializes placement-aware `version: 2` entries into split-entry-shaped effective rows for compatibility consumers. That is intentional bridge posture, not the desired end state.

### 3. Pre-placement selector debt still survives in repo truth

The tree still contains both split-entry names (`codex_world`, `claude_code_world`, `cli:codex_world`, `cli:claude_code_world`) and unqualified pre-placement exact selectors (`cli:codex`, `cli:claude_code`), including active forward-surface hits in docs/scripts plus compatibility/test seams under `crates/shell/`. Slice `60` must separate current forward-surface debt from the historical allowlist and retire the remaining live compatibility paths.

### 4. Slice 59 must remain frozen floor

The retirement work must not reopen the landed world-runtime contract. This slice is about identity and compatibility posture, not guest-runtime delivery.

## Contract

### 1. Placement-aware inventory is the only forward inventory model

Rules:

1. multi-placement CLI agents remain represented as one logical-agent manifest with `config.placements`,
2. split-entry files such as `codex_world.yaml` and `claude_code_world.yaml` are not reintroduced,
3. forward docs/examples/tests must describe placement-aware `version: 2` inventory as the current contract.

### 2. Placement-qualified exact backend ids are the only forward selectors

Rules:

1. forward exact selectors are placement-qualified ids such as `cli:codex-host`, `cli:codex-world`, `cli:claude_code-host`, and `cli:claude_code-world`,
2. retired split-entry selectors such as `cli:codex_world` and `cli:claude_code_world`, and unqualified pre-placement selectors such as `cli:codex` and `cli:claude_code`, must not remain documented as current truth,
3. if a runtime surface still encounters a retired split-entry selector, it must fail closed with explicit migration guidance rather than silently remapping.

### 3. The Slice 58 effective-inventory compatibility bridge must retire

Rules:

1. `version: 2` placement-aware manifests must no longer be materialized back into split-entry-shaped effective inventory rows as normal live behavior,
2. live control surfaces must consume placement-aware realized identities directly,
3. any implementation branch kept temporarily for unsupported historical input must be isolated, explicitly documented, and must not define forward truth.

### 4. Forward docs, fixtures, and policy examples must stop presenting old and new ids together

Rules:

1. authoritative docs under `docs/`, active scripts under `scripts/substrate/`, and forward-facing test fixtures must use the final placement-aware identity model,
2. a file may retain old ids only when it is clearly historical or explicitly tests retirement/fail-closed behavior,
3. old and new exact ids, including unqualified `cli:codex` / `cli:claude_code` examples, must not be shown as coequal valid operator choices.

### 5. Historical evidence is allowed, but only as history

Rules:

1. `llm-last-mile/` records, archived smoke notes, or historical findings may retain legacy split-entry names,
2. those surfaces must not be used as evidence that the old names are still supported current contract,
3. if historical docs need clarification, add explicit historical framing rather than rewriting provenance away.

### 5.1 Packet 1 historical allowlist and temporary retirement inventory

Rules:

1. Historical allowlist: `llm-last-mile/` planning records, Slice `59` closeout notes, and any explicitly labeled historical comment that preserves provenance may retain legacy ids.
2. Temporary retirement inventory is limited to `crates/shell/src/execution/agent_inventory.rs`, `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`, `crates/shell/src/execution/agent_runtime/validator.rs`, `crates/shell/src/execution/orchestrator_world_dispatch.rs`, `crates/shell/src/builtins/world_gateway.rs`, `crates/shell/src/execution/agents_cmd.rs`, `crates/shell/src/execution/cli.rs`, `crates/shell/src/execution/agent_runtime/control.rs`, `crates/shell/src/execution/agent_runtime/auto_attach.rs`, `crates/shell/src/execution/agent_runtime/host_inbox.rs`, `crates/shell/src/execution/host_inbox_materialization.rs`, `crates/shell/src/execution/agent_runtime/orchestration_session.rs`, `crates/shell/src/execution/agent_runtime/session.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`, `crates/shell/src/execution/routing/dispatch/world_ops.rs`, and `crates/shell/src/repl/async_repl.rs`.
3. Deferred forward-surface Packet `4` debt is limited to `docs/TRACE.md`, `docs/internals/world/gateway_auth_handoff.md`, `docs/reference/world/verification/gateway_auth_handoff.md`, `scripts/linux/world-provision.sh`, and `scripts/mac/smoke.sh`. These are current forward-surface debt, not historical allowlist entries, and their presence does not by itself block Packet `1` from being checkpoint-green.
4. `crates/shell/tests/**` and inline `#[cfg(test)]` coverage may retain legacy names only for explicit bridge-removal coverage, persisted-state continuity coverage, or fail-closed retirement assertions; Packet `4` must shrink those remaining hits to intentional negative/historical coverage only.
5. Those temporary code/test hits are not forward truth, are not historical evidence, and must not be cited as support for keeping the compatibility posture.
6. If a split-entry legacy-name hit or unqualified pre-placement exact-selector hit is found in `docs/`, `config/`, or `scripts/` outside the five deferred Packet `4` files above, or in a live `crates/shell/` surface outside the bounded inventory above, treat that as a reopen condition for Slice `60` planning rather than silently widening implementation.

### 6. Slice 59 runtime truth remains untouched

Rules:

1. world-scoped Codex/Claude runtime realizability remains placement-local truth,
2. the retirement of split-entry names must not change world binary/runtime-family semantics,
3. any discovered need to reopen runtime/bootstrap semantics is a spec reopen condition, not routine Slice 60 scope.

## Success Criteria

Slice `60` is complete when:

1. the repo has one clear forward identity model: placement-aware manifests plus placement-qualified exact backend ids,
2. the effective-inventory compatibility bridge for placement-aware manifests is removed or demoted out of forward live behavior,
3. retired split-entry selectors fail closed with explicit guidance instead of silently mapping,
4. authoritative docs, active scripts, forward fixtures, and policy examples no longer present legacy split-entry ids or unqualified pre-placement selectors as current truth,
5. targeted inventory/selector/runtime tests and the forward-truth grep wall are green,
6. Slice `59` runtime-realizability and world-deps proof remain green after the retirement.

## Open Questions

1. Is there any still-supported persisted runtime/session input that truly requires a temporary read-only split-entry alias bridge after the forward contract retirement? The default assumption in this spec is no.
2. Should Slice `60` update the stale status line in [TASKS-58.md](./TASKS-58.md) as part of documentation hygiene, or leave that file untouched as historical planning provenance?
