# Handoff: World Agent `run_world_task` Auth Seeding

## Session Metadata
- Created: 2026-06-21 13:52:49
- Project: /home/spenser/__Active_code/substrate
- Branch: feat/internal-host-orchestrator-world-dispatch-bootstrap
- Session duration: about 2 hours across diagnosis, implementation, and focused verification

### Recent Commits (for context)
  - 87129df3 chore: format slice 61 packet 4 control test
  - e974d6c7 fix: keep structured stderr fallback visible
  - e4ead231 fix: bound structured prompt fallback output
  - 57aa7ecc fix: harden public prompt structured fallback
  - 0bb33fef chore: refresh gitnexus stats and format tests

## Handoff Chain

- **Continues from**: [2026-06-20-163552-world-deps-acl-bridge-and-agent-doctor.md](./2026-06-20-163552-world-deps-acl-bridge-and-agent-doctor.md)
  - Previous title: World-Deps ACL Bridge And Agent Doctor Cleanup
- **Supersedes**: None

> Review the previous handoff for broader world-deps / gateway-smoke context. This handoff is narrowly about the host-orchestrated world member `run_world_task` failure on `cli:codex-world`.

## Current State Summary

The original `missing_world_binding` blocker was resolved earlier by using the public world bootstrap posture (`substrate agent start --backend cli:codex-world --scope world`). After that, direct toolbox `run_world_task` calls began reaching `cli:codex-world`, but every ephemeral run still failed with exit status `1`, including a trivial `Reply with exactly OK` probe. The landed fix now seeds a per-member isolated `CODEX_HOME` for direct world member Codex launches by copying host Codex login artifacts from `~/.codex` when policy explicitly allows `cli:codex-world` in `agents.host_credentials.read.allowed_backends`. Focused unit and routing regression tests pass, but there has not yet been a live rebuild/redeploy plus manual smoke of the installed `~/.substrate/bin/substrate` / deployed `world-service`.

## Codebase Understanding

### Architecture Overview

The relevant flow is: host orchestrator session -> toolbox socket request -> shell-side typed member dispatch request builder -> `world-service` member runtime launcher -> direct Codex binary execution inside the world. The important distinction is that `cli:codex-world` member dispatch is not launched through the gateway-style integrated auth handoff path. Instead, the member runtime uses `PromptFulfillmentBridge::for_member_backend(...)`, which goes through `substrate-gateway`'s adapter runtime and then the `unified-agent-api-codex` crate. That direct Codex path expects a usable `CODEX_HOME` with local auth artifacts, not gateway `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*` env vars. Before this fix, the world member inherited a minimal world env, defaulted to guest `HOME=/root`, and therefore started Codex with no usable auth state. The new path now passes only an internal seed-home hint from shell to `world-service`, then `world-service` materializes an isolated `<launcher_dir>/codex-home`, copies `auth.json` and optional `.credentials.json` from the host `~/.codex`, removes the internal hint, and launches the world Codex process with `CODEX_HOME` pointed at that isolated home.

### Critical Files

| File | Purpose | Relevance |
|------|---------|-----------|
| [crates/shell/src/execution/routing/dispatch/world_ops.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | Shell-side world/member dispatch request construction | Now injects internal host Codex seed-home path only when effective policy allows host credential reads for `cli:codex-world` |
| [crates/world-service/src/member_runtime.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/member_runtime.rs) | In-world retained/ephemeral member runtime launch path | Now converts the internal seed-home hint into an isolated per-launch `CODEX_HOME` before direct Codex spawn |
| [config/agents/codex.yaml](/home/spenser/__Active_code/substrate/config/agents/codex.yaml) | Placement-aware Codex agent inventory | Confirms the world placement binary is `/var/lib/substrate/world-deps/bin/codex` |
| [scripts/substrate/dev-fresh-install-gateway-smoke.sh](/home/spenser/__Active_code/substrate/scripts/substrate/dev-fresh-install-gateway-smoke.sh) | Fresh-install Linux smoke setup | Previously allowed world dispatch to `cli:codex-world` but only host credential reads for `cli:codex-host`; now fixed |
| [scripts/mac/smoke.sh](/home/spenser/__Active_code/substrate/scripts/mac/smoke.sh) | macOS smoke policy scaffold | Had the same allowlist mismatch and is now fixed |
| [docs/USAGE.md](/home/spenser/__Active_code/substrate/docs/USAGE.md) | Public operator-facing usage notes | Now documents that direct `cli:codex-world` member runtime needs `agents.host_credentials.read.allowed_backends` to include `cli:codex-world` |
| [crates/shell/src/builtins/world_gateway.rs](/home/spenser/__Active_code/substrate/crates/shell/src/builtins/world_gateway.rs) | Existing gateway integrated auth resolution seam | Important contrast: gateway path can use `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*` or host `~/.codex`, but direct member path was not using this seam |
| [/home/spenser/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unified-agent-api-codex-0.3.7/src/home.rs](/home/spenser/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unified-agent-api-codex-0.3.7/src/home.rs) | Upstream Codex wrapper `CODEX_HOME` helper | Shows the supported way to isolate Codex state and seed `auth.json` / `.credentials.json` |

### Key Patterns Discovered

- Placement-aware exact backend IDs matter everywhere. `cli:codex-world` and `cli:codex-host` are distinct policy identities even though they share the same runtime family.
- The safe seam for this fix was the narrow member-dispatch path, not the shared generic world env builder. GitNexus impact on `build_world_env_map_for_cwd` was high; impact on the member-dispatch builder and world-service member launch path was low.
- Direct world member Codex launches should fail closed unless policy explicitly allows host credential reads for that exact backend.
- Internal runtime handoff state can cross the shell -> world-service boundary, but it should be stripped before the final child process spawn. The child should only see the final derived runtime env (`CODEX_HOME`), not internal routing hints.

## Work Completed

### Tasks Finished

- [x] Confirmed that host-only orchestrator sessions fail fresh world-dispatch with `missing_world_binding`, and documented the correct public bootstrap posture via `cli:codex-world --scope world`
- [x] Reproduced the second blocker where `run_world_task` reached `cli:codex-world` but all tasks exited `1`
- [x] Traced the direct member-dispatch launch path and proved it does not use the gateway-style `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*` auth handoff
- [x] Confirmed that the direct Codex wrapper path is centered on isolated `CODEX_HOME` plus optional seeding of `auth.json` / `.credentials.json`
- [x] Landed shell-side injection of an internal host Codex seed-home hint gated by effective policy for `cli:codex-world`
- [x] Landed world-service-side conversion of that hint into an isolated per-launch `CODEX_HOME`
- [x] Fixed fresh-install and mac smoke scaffolding so `cli:codex-world` is allowlisted for host credential reads where world-dispatch Codex flows are being exercised
- [x] Added focused unit coverage and reran an existing `run_world_task` routing regression

### Files Modified

| File | Changes | Rationale |
|------|---------|-----------|
| [crates/shell/src/execution/routing/dispatch/world_ops.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | Added `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME`, host `~/.codex` seed-home resolution, policy-gated injection for `cli:codex-world`, and unit tests | Allows shell-side member dispatch to pass only the minimal internal seed-home hint when exact backend policy allows it |
| [crates/world-service/src/member_runtime.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/member_runtime.rs) | Added pre-launch runtime env preparation for Codex members, isolated `CODEX_HOME` materialization under launcher dir, seeding from host `.codex`, and tests | Makes direct world member Codex launches start with usable auth state without exposing raw host auth paths or raw gateway env handoffs to the child |
| [crates/world-service/Cargo.toml](/home/spenser/__Active_code/substrate/crates/world-service/Cargo.toml) | Added direct dependency on `unified-agent-api-codex` as `codex` | Needed to use `CodexHomeLayout` and `AuthSeedOptions` in the world-service member runtime |
| [scripts/substrate/dev-fresh-install-gateway-smoke.sh](/home/spenser/__Active_code/substrate/scripts/substrate/dev-fresh-install-gateway-smoke.sh) | Changed `agents.host_credentials.read.allowed_backends` to include `cli:codex-world` alongside `cli:codex-host` | Fixes the fresh-install smoke mismatch that guaranteed world-member Codex auth fallback failure |
| [scripts/mac/smoke.sh](/home/spenser/__Active_code/substrate/scripts/mac/smoke.sh) | Added `cli:codex-world` under `agents.host_credentials.read.allowed_backends` | Keeps mac smoke policy consistent with the world-dispatch Codex posture |
| [docs/USAGE.md](/home/spenser/__Active_code/substrate/docs/USAGE.md) | Added operator-facing note about `cli:codex-world` needing host credential read allowlisting and isolated per-member `CODEX_HOME` seeding | Prevents future confusion about why direct world member Codex launches can still fail after world binding is correct |
| [crates/shell/src/execution/prompt_fulfillment.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/prompt_fulfillment.rs) | Earlier in session: clarified toolbox/world-binding requirement in runtime-owned prompt | Helps the host orchestrator understand when fresh world-dispatch verbs are actionable |
| [crates/shell/tests/agent_public_control_surface_v1.rs](/home/spenser/__Active_code/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | Earlier in session: added regression coverage for the prompt surface | Locks in the public control surface wording around world bootstrap posture |
| [Cargo.lock](/home/spenser/__Active_code/substrate/Cargo.lock) | Updated for the new direct `codex` dependency in `world-service` | Required by the dependency graph change |
| [AGENTS.md](/home/spenser/__Active_code/substrate/AGENTS.md) | Pre-existing dirty change, not modified by this session | Call out to avoid accidental revert by a future agent |
| [CLAUDE.md](/home/spenser/__Active_code/substrate/CLAUDE.md) | Pre-existing dirty change, not modified by this session | Call out to avoid accidental revert by a future agent |

### Decisions Made

| Decision | Options Considered | Rationale |
|----------|-------------------|-----------|
| Fix direct member auth in the member-dispatch path instead of the generic world env builder | Patch `build_world_env_map_for_cwd`; patch member-dispatch builder; patch Codex prompting | `build_world_env_map_for_cwd` had a high GitNexus blast radius, while the member-dispatch seam was narrow and directly on the failing path |
| Use host `~/.codex` as a seed source and launch Codex with isolated per-member `CODEX_HOME` | Forward raw `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*`; rely on `/root/.codex`; do nothing and require manual login in-world | The upstream direct Codex wrapper is designed around `CODEX_HOME` isolation and auth seeding, and the direct member path was not using gateway-style auth env handoff |
| Gate seed-home injection on `agents.host_credentials.read.allowed_backends` for exact backend `cli:codex-world` | Always inject host `.codex`; reuse `cli:codex-host` allowlist; infer allow from world-dispatch allowlist | Exact backend policy identity is the authoritative control surface; world-dispatch permission should not silently imply host credential read permission |
| Require `auth.json` but keep `.credentials.json` optional when seeding the isolated home | Require both files; require neither; silently skip missing auth | `auth.json` is necessary for a usable login state, but `.credentials.json` may or may not exist and does not need to be fail-closed for this first fix |

## Pending Work

### Immediate Next Steps

1. Rebuild and redeploy the installed runtime from this repo so `~/.substrate/bin/substrate` and the active `world-service` pick up the new direct member auth-seeding path.
2. Rerun the original manual smoke: `~/.substrate/bin/substrate agent start --backend cli:codex-world --scope world --prompt "...run_world_task..."` and confirm that `from_the_world_worker.md` is actually created.
3. If the live smoke still fails, inspect the resulting world member stderr / trace with the new auth-seeded posture to determine whether the remaining issue is Codex runtime behavior, working directory semantics, or something unrelated to auth.

### Blockers/Open Questions

- [ ] Live verification blocker: the code is fixed in the repo, but the manual smoke was not rerun against rebuilt/deployed binaries in this session.
- [ ] Open question: whether any deployment path outside the fresh-install/mac smoke scaffolds also seeds a policy that still omits `cli:codex-world` from `agents.host_credentials.read.allowed_backends`.
- [ ] Open question: whether an end-to-end integration test should be added for auth-seeded `cli:codex-world` member launch, rather than relying only on unit coverage plus existing routing regressions.

### Deferred Items

- Add a full integration regression that asserts direct `cli:codex-world` member launch receives a usable isolated `CODEX_HOME` and can complete a trivial prompt. Deferred because the immediate objective was to unblock the real runtime path first.
- Audit whether Claude Code world members need an analogous explicit seed-home or auth-materialization seam. Deferred because this failure was specific to direct Codex runtime behavior.

## Context for Resuming Agent

### Important Context

The most important thing to preserve is the distinction between three different auth models that look similar but are not interchangeable:

1. **Gateway integrated auth handoff**:
   - Used by `world gateway` flows.
   - Can rely on `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*` or host `~/.codex` resolution via `world_gateway.rs`.
   - Not the path that was failing here.

2. **Direct world member Codex launch**:
   - Used by `run_world_task` / `spawn_world_worker` when a `cli:codex-world` member is launched through `world-service` member runtime.
   - Goes through `PromptFulfillmentBridge::for_member_backend(...)` and `substrate-gateway` adapter runtime into `unified-agent-api-codex`.
   - That path expects a usable `CODEX_HOME`; it does not consume `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*` directly.

3. **The landed bridge between those worlds**:
   - Shell injects `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME=<host home>/.codex` only when effective policy allows `cli:codex-world` in `agents.host_credentials.read.allowed_backends`.
   - `world-service` removes that internal env var before spawning the child.
   - `world-service` materializes `<launcher_dir>/codex-home`, copies `auth.json` and optional `.credentials.json` from host `~/.codex`, and sets `CODEX_HOME=<launcher_dir>/codex-home` for the direct world member Codex process.

Also preserve this operational fact: the original live failure (`run_world_task failed on backend cli:codex-world with exit status 1`) happened even for a trivial `Reply with exactly OK` task, which was the strongest signal that the problem sat at world-member runtime bootstrap/auth posture, not at prompt content or file-writing behavior.

### Assumptions Made

- The direct world member Codex exit-1 posture was caused by missing usable auth state, not by the binary being absent or by world binding being missing. This was supported by the binary existing, world binding being present, and trivial prompts failing identically.
- Host `~/.codex/auth.json` is the authoritative seed source for the direct world member fix because that matches the upstream Codex wrapper’s documented isolation/seeding model.
- Requiring `auth.json` but not `.credentials.json` is sufficient for this first-pass unblock.
- The existing world member launch temp dir is an acceptable location for isolated per-member `CODEX_HOME`.

### Potential Gotchas

- The repo has unrelated dirty files: `AGENTS.md`, `CLAUDE.md`, and the earlier prompt/docs/tests changes from the same overall debugging arc. Do not revert anything blindly.
- `.codex` in this repo root is an empty file, not a directory. The session-handoff skill had to target the existing `handoffs/` directory explicitly.
- The installed `~/.substrate/bin/substrate` may still be running old code until rebuilt/redeployed. A failed live smoke after this point may simply mean the runtime was not refreshed.
- The direct Codex wrapper path uses `CODEX_HOME`, not raw `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*`. If a future agent forgets that distinction, they will likely chase the wrong auth seam again.
- The internal env var `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME` is intentionally runtime-internal. It should not be surfaced in user docs as a supported public contract.

## Environment State

### Tools/Services Used

- `GitNexus`:
  - Impact on `build_agent_client_and_member_dispatch_request_impl` in `world_ops.rs`: LOW
  - Impact on `build_world_env_map_for_cwd` in `shim_ops.rs`: HIGH
  - Impact on `MemberRuntimeManager::launch` in `member_runtime.rs`: LOW
- `cargo fmt --all`
- Focused tests:
  - `cargo test -p world-service prepare_codex_runtime_env -- --nocapture`
  - `cargo test -p shell codex_member_dispatch_injects_internal_seed_home_when_backend_is_allowlisted -- --nocapture`
  - `cargo test -p shell c3_internal_toolbox_run_world_task_fast_completion_still_streams_registered_task_run_id_before_terminal_result -- --nocapture`

### Active Processes

- No known intentional background dev servers or long-running helper processes were left running by this session.

### Environment Variables

- `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`
- `SUBSTRATE_AGENT_TOOLBOX_VERSION`
- `SUBSTRATE_WORLD_SOCKET`
- `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME` (runtime-internal, shell -> world-service only)
- `CODEX_HOME`
- `HOME`
- `USERPROFILE`

## Related Resources

- [Previous handoff: 2026-06-20-163552-world-deps-acl-bridge-and-agent-doctor.md](./2026-06-20-163552-world-deps-acl-bridge-and-agent-doctor.md)
- [SPEC-58 placement-aware agent inventory](/home/spenser/__Active_code/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md)
- [SPEC-59 world-scoped CLI runtime realizability](/home/spenser/__Active_code/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)
- [SPEC-60 compatibility retirement](/home/spenser/__Active_code/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md)
- [SPEC-61 human public agent start streaming regression](/home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md)
- [Shell member-dispatch builder](/home/spenser/__Active_code/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [World member runtime launcher](/home/spenser/__Active_code/substrate/crates/world-service/src/member_runtime.rs)
- [Codex placement inventory](/home/spenser/__Active_code/substrate/config/agents/codex.yaml)
- [Gateway integrated auth resolution seam](/home/spenser/__Active_code/substrate/crates/shell/src/builtins/world_gateway.rs)
- [Direct Codex wrapper home seeding helper](/home/spenser/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unified-agent-api-codex-0.3.7/src/home.rs)

---

**Security Reminder**: This handoff intentionally names only env var identifiers and auth file paths. It does not include tokens, file contents, or secret values.
