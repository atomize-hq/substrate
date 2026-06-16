# Spec: World-Scoped CLI Runtime Realizability And Codex Guest Runtime Delivery

Source dossier: [SESSION_DECISION_DOSSIER-agent-placement-shape-and-world-runtime-gap.md](./SESSION_DECISION_DOSSIER-agent-placement-shape-and-world-runtime-gap.md)  
Related authorities:
- [CODEX_WORLD_DISPATCH_GAP_WRITEUP.md](../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md)
- [SPEC-58-placement-aware-agent-inventory-and-selector-contract.md](./SPEC-58-placement-aware-agent-inventory-and-selector-contract.md)
- [PLAN-58-placement-aware-agent-inventory-and-selector-contract.md](./PLAN-58-placement-aware-agent-inventory-and-selector-contract.md)
- [docs/reference/world/deps/README.md](../docs/reference/world/deps/README.md)
- [docs/reference/world/deps/provisioning.md](../docs/reference/world/deps/provisioning.md)
- [docs/reference/world/deps/authoring_packages.md](../docs/reference/world/deps/authoring_packages.md)
- [`config/agents/codex_world.yaml`](../config/agents/codex_world.yaml)
- [`crates/shell/src/execution/agent_runtime/validator.rs`](../crates/shell/src/execution/agent_runtime/validator.rs)
- [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
- [`scripts/substrate/world-enable.sh`](../scripts/substrate/world-enable.sh)
- [`scripts/substrate/install-substrate.sh`](../scripts/substrate/install-substrate.sh)
- [`scripts/substrate/dev-install-substrate.sh`](../scripts/substrate/dev-install-substrate.sh)
Phase: `SPECIFY`  
Status: draft for review

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The active `cli:codex_world` failure is a guest-runtime/bootstrap gap, not a world-binding gap.
2. This slice must be implemented **before** Slice 58 code lands, but its contract must survive the later placement-aware cutover where `cli:codex-world` is derived from logical agent id plus placement.
3. The preferred delivery path is a **Substrate-owned** world-deps `install.method: script` package that pulls official Codex Linux release artifacts rather than relying on host npm/NVM state.
4. The current repo truth does **not** require world refresh/restart after `substrate world deps current sync`; sync is the apply step unless new evidence proves otherwise.
5. The downloadable Linux Codex binary may or may not be fully self-contained in the target guest; this slice must require explicit guest verification instead of assuming either outcome.
6. Exact backend ids, policy allowlists, and host/world fail-closed separation remain unchanged in this slice; selector migration belongs to Slice 58.
7. The installer/runtime provisioning surface should be future-expandable, so the public flag shape should be generic to agent runtime family rather than Codex-specific.

If any of these are wrong, correct them before implementation.

## Objective

Freeze the truthful launchability and provisioning contract for world-scoped CLI runtimes, starting with Codex.

This slice must answer:

1. what must be true before a world-scoped CLI backend is considered launchable,
2. where that truth is validated,
3. what fail-closed error/remediation appears when only a host-local runtime exists,
4. how Substrate installs a guest-visible Codex runtime inside the world,
5. how prod/dev installers expose install-time provisioning of that runtime,
6. and how all of the above align with the later placement-aware inventory shape from Slice 58.

This slice does **not** redesign selector grammar, perform the placement-aware inventory cutover, or invent a first-class artifact transport system beyond the existing world-deps script-package contract.

## Tech Stack

- Rust workspace (`cargo`)
- `crates/shell` runtime selection, validator, world-deps, and installer-adjacent shell flows
- `crates/world-service` member bootstrap/runtime execution
- world-deps inventory + script packages under Substrate-managed prefixes
- shell installers under `scripts/substrate/`
- `llm-last-mile/` planning authority

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

Targeted validation/runtime test wall once implemented:

```bash
cargo test -p shell agent_runtime::validator -- --nocapture
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell world_deps -- --nocapture
cargo test -p world-service member_runtime -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
```

Repo-truth checks:

```bash
rg -n "codex_world|world deps current sync|provision-deps|sync-deps|config\.cli\.binary|which::which" \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts
```

## Project Structure

```text
config/agents/codex_world.yaml
  Current split world-scoped Codex inventory truth that Slice 59 must make semantically honest before Slice 58 replaces the split shape.

crates/shell/src/execution/agent_runtime/validator.rs
  Current host-side runtime realizability gate; this is where fake host-only launchability must stop.

crates/shell/src/execution/orchestrator_world_dispatch.rs
  Current world dispatch path that serializes resolved runtime truth into member bootstrap.

crates/world-service/src/member_runtime.rs
crates/world-service/src/gateway_runtime.rs
  Late bootstrap/runtime execution path; this slice should stop bad launches earlier, not rely on late failure.

crates/shell/src/builtins/world_deps/
  Runtime package inventory, sync/install logic, and wrapper behavior for guest-visible tools.

docs/reference/world/deps/
  Stable operator contract for enable/provision/sync flows and package authoring.

scripts/substrate/install-substrate.sh
scripts/substrate/install.sh
scripts/substrate/dev-install-substrate.sh
scripts/substrate/world-enable.sh
  Installer and provisioning surfaces that need explicit Codex runtime flag coverage.

llm-last-mile/
  Slice 58 + Slice 59 + Slice 60 planning authority.
```

## Code Style

Use explicit, fail-closed error shaping with actionable remediation and no silent host/world fallback.

```rust
return Err(RuntimeRealizabilityError {
    exit_code: 4,
    reason: format!(
        "selected runtime '{}' is not runtime-realizable in world scope because guest entrypoint '{}' is unavailable; install the world runtime and rerun 'substrate world deps current sync'",
        contract.agent_id,
        expected_entrypoint.display(),
    ),
});
```

Conventions:

1. keep host-truth and guest-truth variable names distinct,
2. prefer typed remediation reasons over ad-hoc stderr text,
3. keep world-deps package scripts idempotent and restricted to `/var/lib/substrate/world-deps` and `/tmp`,
4. do not encode selector migration or placement-aware inventory logic inside this slice.

## Testing Strategy

- **Unit tests**: validator/runtime selection helpers, world-deps inventory/install helpers, installer flag parsing.
- **Integration tests**: world-dispatch contract + world-service member bootstrap + world-deps runtime application.
- **CLI/contract tests**: fail-closed diagnostics, remediation text, installer help/docs, and operator flow examples.
- **Manual/smoke proof**: provision runtime, sync deps, launch world-scoped Codex, prove the resolved runtime comes from guest-visible world-deps paths instead of host NVM.

## Current Repo-Truth Gut Check

### 1. The current world-scoped runtime gate is host-biased

The shell validator still resolves `config.cli.binary` on the **host** with `which::which(...)`, then world dispatch serializes that host result into member bootstrap.

### 2. The late failure is semantically truthful but architecturally too late

The current failing `exit 127` proves the guest cannot actually execute the host-resolved NVM Codex path, but that truth appears after dispatch/bootstrap work has already started.

### 3. The repo already has an intended guest-tool delivery surface

world-deps already supports script-installed runnable packages under `/var/lib/substrate/world-deps/...` with stable entrypoints exposed via `/var/lib/substrate/world-deps/bin`.

## Contract

### 1. World-scoped CLI launchability must be guest-visible truth

For a world-scoped CLI backend, launchability must be determined by **guest-visible, guest-executable** runtime truth.

Rules:

1. host `which <binary>` success is insufficient for world-scoped launchability,
2. a world-scoped backend must name or derive a guest-visible executable contract,
3. if that contract is not satisfied, Substrate must fail closed before retained worker bootstrap,
4. this rule applies to the current split `cli:codex_world` topology and must remain true after Slice 58 derives `cli:codex-world` from placement-aware config.

### 2. Fail closed in validator/materialization, not only in late bootstrap

This slice must move the authoritative failure wall earlier.

Rules:

1. `validator.rs` or the immediately adjacent materialization path must reject world-scoped Codex when only host-local runtime truth exists,
2. world dispatch must not serialize a host-only binary path as if it were guest-truth for a world member,
3. late world-service bootstrap checks remain valuable defense-in-depth, but must no longer be the primary truth surface for this bug class.

### 3. Preferred delivery model is Substrate-owned world-deps script packaging

The chosen path is **Option A**.

Rules:

1. Substrate owns the package definition and install script,
2. installation happens through world-deps `install.method: script`,
3. package content lives under `/var/lib/substrate/world-deps/<package>`,
4. runnable entrypoint resolves via `/var/lib/substrate/world-deps/bin/codex`,
5. artifact retrieval targets official Codex release artifacts, with GitHub Releases as the expected source of the downloadable Linux binary.

### 4. Guest artifact verification is mandatory, not implied

The slice must freeze the verification seam, not hand-wave it.

Rules:

1. if the Linux release binary is truly self-contained in the guest, the package may install just that binary plus any thin wrapper/symlink needed,
2. if the binary still needs Node or other guest runtime pieces, the package must widen into a `codex-runtime` bundle (or equivalent) that provisions the required pieces explicitly,
3. the implementation must record which path was proven, rather than leaving both possibilities implied.

### 5. Operator flow remains inventory -> enable -> provision if needed -> sync

This slice must preserve current world-deps operator truth.

Rules:

1. package/bundle definitions live in inventory,
2. enablement lives in config patches,
3. `substrate world enable --provision-deps` remains the only operator-facing system-package mutation surface,
4. `substrate world deps current sync` remains the runtime apply step,
5. this slice must not assume world refresh/restart is required after sync without new evidence.

### 6. Installers must expose install-time runtime provisioning on both surfaces

Rules:

1. prod installer surfaces (`install-substrate.sh` / `install.sh`) must gain a generic runtime-family flag shaped as `--provision-agent-runtime <runtime_family>`,
2. dev installer must gain the same public flag shape,
3. this slice implements only the `codex` value for that flag; future families such as `claude_code` are intentionally deferred,
4. unsupported runtime-family values must fail closed with explicit unsupported-value diagnostics rather than silently no-oping or partially provisioning,
5. the flag means the runtime is made ready to use, so it must provision/install the selected runtime as needed and then run `substrate world deps current sync`,
6. docs/help must state that the flag includes sync rather than leaving a hidden post-install step,
7. the behavior must be the same conceptually across dev and prod even if implementation plumbing differs.

### 7. Slice 59 must align forward to Slice 58 without waiting for Slice 58

Rules:

1. Slice 59 implementation may target current `codex_world` inventory while Slice 58 is still unimplemented,
2. Slice 59 docs must describe the runtime contract in terms of the **world-scoped Codex backend**, not only the temporary split file name,
3. once Slice 58 lands, the runtime truth from Slice 59 must attach naturally to `placements.world` rather than forcing a redesign.

## Boundaries

- **Always do:**
  - fail closed before world member bootstrap when guest runtime truth is missing,
  - keep world-deps writes confined to `/var/lib/substrate/world-deps` and `/tmp`,
  - make remediation/operator steps explicit in user-facing diagnostics,
  - keep prod/dev installer flag behavior documented and consistent.

- **Ask first:**
  - changing public installer flag names after they are drafted,
  - widening into a new artifact transport/checksum system beyond world-deps script packaging,
  - changing world restart semantics if live evidence shows sync is insufficient.

- **Never do:**
  - rely on host NVM/npm paths as authoritative world runtime truth,
  - silently fall back from world-scoped runtime to host-scoped runtime,
  - bake placement-aware selector migration into this slice,
  - write world-deps package installers that mutate `$HOME`, `/usr`, `/etc`, or other non-Substrate-managed paths.

## Success Criteria

1. A world-scoped Codex backend is considered launchable only when guest runtime truth is satisfied.
2. Missing guest runtime truth fails closed before retained worker bootstrap with explicit remediation.
3. Substrate-owned world-deps packaging can install Codex into the guest from official release artifacts.
4. The implementation explicitly proves whether the Linux artifact is self-contained or requires a wider runtime bundle.
5. Prod and dev installers both expose a documented install-time provisioning flag for this runtime.
6. Slice 59 lands cleanly before Slice 58 implementation, and Slice 58 can later migrate shape/selectors without reopening the runtime contract.

## Open Questions

1. Should the package name be `codex`, `codex-runtime`, or another bundle-oriented name if guest prerequisites are needed?
2. If Slice 58 keeps temporary compatibility aliases, how long should Slice 59 diagnostics refer to old vs new exact ids during transition?
