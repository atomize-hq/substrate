# PLAN: UAA Boundary and Naming Cleanup

Source SOW: [27-uaa-boundary-and-naming-cleanup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/27-uaa-boundary-and-naming-cleanup.md)  
Related validation plan: [spensermcconnell-testing-uaa-boundary-hardening-test-plan-20260521-111202.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-testing-uaa-boundary-hardening-test-plan-20260521-111202.md)  
Primary boundary anchors: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md), [ADR-0042](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md), [ADR-0044](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md), [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md), [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md), [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)  
Current workspace branch: `testing`  
Base branch: `main`  
Plan type: repo-wide naming and boundary correction, no UI scope, developer-facing runtime and operator scope  
Status: unified implementation plan, 2026-05-21  
Supersedes: previous root `PLAN.md`, which tracked the async persistent-session bootstrap readiness split instead of the UAA boundary cleanup slice

## Objective

Finish the boundary cleanup so the repo no longer blurs:

1. the external Unified Agent API runtime abstraction,
2. the Substrate-local host<->world typed transport,
3. the Substrate-local in-world daemon and service boundary,
4. and the Substrate-local pure-agent protocol-family label used in config, persistence, and trace identity.

This is a naming and supportability correction, not a runtime-model rewrite.

The exact choices in this plan are fixed:

1. upstream `agent_api` keeps its current meaning and does not rename,
2. `world-api` keeps its current meaning and does not rename,
3. canonical local pure-agent protocol-family becomes `substrate.agent.session`,
4. `world-agent*` renames to `world-service*`,
5. `agent-api*` renames to `transport-api*`,
6. the slice is a direct cutover on live supported surfaces, not a long-lived compatibility program,
7. and world-required flows must remain fail closed if rename drift breaks service discovery.

## Acceptance Criteria

This plan is complete only when all of the following are true:

1. `Unified Agent API`, `UAA`, Rust `agent_api`, and adopted `agent_api.*` capability and schema ids have one stable repo meaning: the upstream CLI-agent runtime abstraction only.
2. `substrate.agent.session` is the only canonical local pure-agent protocol-family label on supported live surfaces.
3. `world-agent*` has been replaced by `world-service*` across code, package names, binaries, systemd units, install helpers, bundle payloads, docs, and operator remediation text on live surfaces.
4. `agent-api-types`, `agent-api-core`, and `agent-api-client` have been replaced by `transport-api-types`, `transport-api-core`, and `transport-api-client`, including the Rust crate ids `transport_api_types`, `transport_api_core`, and `transport_api_client`.
5. `world-api` remains unchanged as the abstract world backend contract.
6. Mixed-boundary crates that import both upstream `agent_api` and local transport crates read clearly at the dependency and import layer after the rename.
7. Old `uaa.agent.session` live configs, fixtures, and persisted rows are no longer treated as supported canonical inputs after cutover. If encountered, they fail closed with explicit operator-readable errors.
8. The live-surface grep wall passes with zero stale hits for `world-agent`, `substrate-world-agent`, `agent-api-*`, `agent_api_*`, and `uaa.agent.session` outside the explicit historical allowlist.
9. Linux, macOS Lima, and WSL provision, warm, smoke, doctor, uninstall, and release-bundle flows all use the renamed `world-service` family consistently.
10. World-required routing still hard-fails if renamed service discovery breaks. No rename drift path silently falls back to host execution.

## Step 0: Scope Challenge

### What already exists

| Sub-problem | Existing code or surface | Plan decision |
| --- | --- | --- |
| Upstream UAA runtime boundary | [`crates/shell/Cargo.toml`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/Cargo.toml), [`crates/world-agent/Cargo.toml`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml), [`docs/contracts/substrate-gateway-backend-adapter-schema.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-schema.md) | Reuse. Upstream `agent_api` stays untouched. |
| Local pure-agent protocol label | [`mapping.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mapping.rs), [`validator.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs), [`agent_events.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs), [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse central seams. Cut over to `substrate.agent.session` in one supported pass. |
| Local typed host<->world contract | [`crates/agent-api-types/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/), [`crates/agent-api-core/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-core/), [`crates/agent-api-client/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/) | Reuse the actual contract. Rename the local family to `transport-api-*` without touching upstream `agent_api`. |
| In-world daemon and control surface | [`crates/world-agent/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/), [`scripts/linux/world-provision.sh`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh), [`docs/WORLD.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md) | Reuse the implementation. Rename the service family to `world-service*`. |
| Install, warm, release, and CI harnesses | [`scripts/substrate/install-substrate.sh`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh), [`scripts/mac/lima-warm.sh`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh), [`scripts/windows/wsl-warm.ps1`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/wsl-warm.ps1), [`dist/scripts/assemble-release-bundles.sh`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/scripts/assemble-release-bundles.sh), [`.github/workflows/feature-smoke.yml`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/feature-smoke.yml) | Reuse as the real validation wall. These are first-class implementation scope, not polish. |
| Boundary wording | [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md), [`docs/WORLD.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md), [`docs/TRACE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md) | Reuse as source-of-truth anchors, then rewrite so the same story appears everywhere. |

### Minimum honest diff

The minimum honest implementation touches these module and surface groups:

1. workspace manifests and crate directories under `Cargo.toml` and `crates/`
2. runtime label and validator seams under `crates/common/` and `crates/shell/src/execution/agent_runtime/`
3. host<->world routing and payload seams under `crates/shell/src/execution/routing/` and the renamed transport crates
4. the in-world daemon crate and its tests under the renamed `crates/world-service/`
5. installers, provisioners, warm scripts, uninstallers, bundle builders, and CI under `scripts/`, `dist/`, and `.github/`
6. operator and developer docs under `docs/`, `README.md`, and `AGENTS.md`
7. targeted tests in `crates/shell/tests/`, the renamed transport crates, `world-mac-lima`, `world-windows-wsl`, and `substrate-replay`

Anything smaller is a shortcut. It would leave drift in install surfaces, service discovery, validation, or mixed-boundary imports.

### Repo-truth findings

1. This is not greenfield. `world-agent`, `substrate-world-agent`, `agent-api-*`, and `uaa.agent.session` already appear in code, package names, binaries, units, docs, CI, and persisted state.
2. The biggest product risk is not runtime speed. It is supportability drift. If rename work misses systemd units, bundle paths, or doctor text, users get broken worlds and misleading remediation.
3. The highest-severity technical risk is fail-open behavior. If renamed service discovery breaks and world-required flows quietly fall back to host execution, the cleanup weakens the isolation model.
4. The highest-severity migration seam is `uaa.agent.session`. It is exact-matched in validator paths, config examples, traces, transport payloads, and durable runtime/session state.
5. Mixed-boundary consumers already import both upstream `agent_api` and local `agent-api-*` names. The rename must make those files clearer, not just different.
6. `world-agent` is not just transport. It owns execute, cancel, gateway, member-runtime, and world-fs behavior. `world-service` is the truthful replacement. `world-transport` is not.

### Complexity, completeness, and distribution verdict

1. This is a large slice. It touches more than eight files and crosses crates, scripts, CI, release, docs, and operator guidance. That is justified because the ambiguity is already cross-cutting.
2. The complete fix is still the right fix. A code-only rename without install, bundle, and grep walls is fake completeness and will produce support debt immediately.
3. No new binary class or distribution pipeline is introduced, but existing distribution surfaces are part of the implementation scope because the renamed daemon already ships through them.
4. Direct cutover is approved for supported live surfaces. That means supported examples, fixtures, and docs get rewritten now, and unsupported stale rows fail closed with clear errors after the cutover.

### Scope ruling

The scope is accepted as-is.

This plan is intentionally broader than a few crate renames and intentionally narrower than a public runtime or protocol redesign. That is the right middle.

## Architecture Review

### Problem statement

The repo already has three real boundaries:

1. upstream `agent_api` for external Unified Agent API semantics,
2. local typed host<->world contracts,
3. local in-world daemon and control surfaces.

The bug is not missing architecture. The bug is that the names still blur those boundaries.

### Boundary thesis

Responsibility decides the noun.

- upstream runtime semantics keep `agent_api`
- local contract crates use `transport`
- the in-world daemon uses `service`
- the abstract world backend layer keeps `world-api`
- the local pure-agent protocol-family uses `substrate.agent.session`

### Target boundary map

```text
TARGET BOUNDARY MAP
===================
upstream Unified Agent API
    -> Rust crate/import: agent_api
    -> capability/schema ids: agent_api.*
    -> meaning: external CLI-agent runtime abstraction only

Substrate local world contract
    -> crates: transport-api-types / transport-api-core / transport-api-client
    -> Rust crate ids: transport_api_types / transport_api_core / transport_api_client
    -> meaning: typed host<->world transport contract only

Substrate local in-world daemon
    -> crate/package/binary/service: world-service
    -> systemd/socket family: substrate-world-service.*
    -> meaning: in-world execution and control daemon

Abstract world backend layer
    -> crate/package: world-api
    -> meaning: backend contract only

Local pure-agent protocol-family
    -> canonical label: substrate.agent.session
    -> meaning: Substrate-local pure-agent identity only
```

### Canonical rename contract

| Current canonical name | New canonical name | Decision |
| --- | --- | --- |
| `uaa.agent.session` | `substrate.agent.session` | rename |
| `crates/world-agent` | `crates/world-service` | rename |
| package `world-agent` | package `world-service` | rename |
| binary `world-agent` | binary `world-service` | rename |
| installed alias `substrate-world-agent` | `substrate-world-service` | rename |
| `substrate-world-agent.service` / `.socket` | `substrate-world-service.service` / `.socket` | rename |
| `crates/agent-api-types` | `crates/transport-api-types` | rename |
| `crates/agent-api-core` | `crates/transport-api-core` | rename |
| `crates/agent-api-client` | `crates/transport-api-client` | rename |
| `agent_api_types` / `agent_api_core` / `agent_api_client` | `transport_api_types` / `transport_api_core` / `transport_api_client` | rename |
| `world-api` | `world-api` | keep |
| upstream `agent_api` | upstream `agent_api` | keep |
| upstream `agent_api.*` ids | upstream `agent_api.*` ids | keep |

### Live rename boundary and historical allowlist

Live rename scope must go green on:

1. `Cargo.toml`
2. `crates/**`
3. `scripts/**`
4. `.github/**`
5. `dist/**`
6. `README.md`
7. `AGENTS.md`
8. `config/**`
9. operator and developer docs under `docs/**`

Historical or evidence-bearing files may retain old tokens only when intentionally preserved, for example:

1. `docs/project_management/_archived/**`
2. older planning packs or captured artifacts that are not normative runtime or operator guidance

This plan does not allow a vague "repo-wide rename." It requires the live-surface command wall and the historical allowlist to be named explicitly.

### Frozen contract

If implementation wants to violate any rule below, stop and revise the plan first.

1. Upstream `agent_api` and `agent_api.*` remain untouched.
2. `world-api` remains untouched.
3. `substrate.agent.session` is the only canonical supported local protocol-family label after cutover.
4. Direct cutover applies to supported live surfaces. Do not add permanent compatibility aliases just to avoid touching docs or scripts.
5. Unsupported stale rows and configs must fail closed with explicit error text, not silent acceptance and not silent downgrade.
6. World-required flows must not fall back to host execution because a renamed binary, socket, or unit was missed.
7. Operator-facing unit names, install aliases, release payload names, and doctor text must match the code rename in the same slice.
8. Mixed-boundary files must read more clearly after the rename than before.

## Code Quality Review

### Explicit design choices

1. Rename only the local transport and daemon families. Do not widen this slice into a public runtime or protocol redesign.
2. Treat scripts, CI, bundles, and docs as product surfaces. Do not dump them into "truth sync later."
3. Centralize the protocol-family cutover through the current validator, mapping, trace, and persistence seams instead of scattering custom translation code.
4. Keep the direct-cutover rule honest: all supported examples and checked-in live fixtures update now; unsupported stale rows fail clearly after cutover.
5. Use one exact rename matrix and one exact grep wall. No ad hoc exceptions.

### Naming and ownership expectations

1. `world-service` means "the in-world daemon and control surface." It is not just a transport pipe.
2. `transport-api-*` means "the local typed host<->world contract layer." It does not mean upstream UAA.
3. `world-api` keeps the abstract backend meaning and should be explicitly called out as unchanged anywhere the rename is explained.
4. Mixed-boundary consumers should preserve explicit wording in comments and tests: upstream runtime on one side, local transport on the other.

### DRY guardrails

1. There must be one authoritative rename matrix, not slightly different rename lists in code comments, docs, and scripts.
2. There must be one grep wall for old-name detection across live surfaces, not one command per contributor with different scopes.
3. There must be one canonical protocol-family constant after cutover. Do not leave stale helper constants lying around under alternate names.
4. Error and remediation copy should be sourced from renamed truth, not duplicated with half-updated old examples.

## Implementation Plan

### Phase summary

| Phase | Purpose | Modules or surfaces touched | Hard dependency |
| --- | --- | --- | --- |
| 1 | Freeze vocabulary, rename matrix, and live-surface boundary | `Cargo.toml`, `AGENT_ORCHESTRATION_GAP_MATRIX.md`, `docs/`, `README.md`, `AGENTS.md` | — |
| 2 | Cut over local protocol-family label | `crates/common/`, `crates/shell/src/execution/agent_runtime/`, `config/`, relevant fixtures and tests | 1 |
| 3 | Rename local transport and daemon families | `crates/world-agent/`, `crates/agent-api-*/`, `Cargo.toml`, mixed-boundary consumers | 1 |
| 4 | Sweep installers, release bundles, platform helpers, and CI | `scripts/`, `dist/`, `.github/` | 1, 3 |
| 5 | Sweep operator docs, ADRs, and developer guidance | `docs/`, `README.md`, `AGENTS.md`, ADRs | 1, 2, 3, 4 |
| 6 | Run grep wall, package tests, operator validation, and closeout | repo root, platform scripts, release scripts | 2, 3, 4, 5 |

### Phase 1: Freeze vocabulary, rename matrix, and live-surface boundary

Files and surfaces:

1. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
2. [README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/README.md)
3. [AGENTS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENTS.md)
4. [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
5. [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)

Actions:

1. Freeze the vocabulary contract in one place: upstream `agent_api`, local `transport-api-*`, local `world-service`, unchanged `world-api`, local `substrate.agent.session`.
2. Freeze the exact rename matrix for crate directories, package names, Rust crate ids, binaries, install aliases, unit names, and bundle paths before file moves start.
3. Freeze the live rename boundary and historical allowlist before grep work starts.
4. Record the exact grep wall commands in the plan so contributors are checking the same thing.

Exit criteria:

1. Every later phase can point back to one exact rename matrix.
2. No implementation phase has to guess which surfaces are live and which are historical.

### Phase 2: Cut over the local protocol-family label to `substrate.agent.session`

Files and surfaces:

1. [crates/common/src/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs)
2. [crates/shell/src/execution/agent_runtime/mapping.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mapping.rs)
3. [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
4. [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
5. [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
6. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
7. [config/agents/codex.yaml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex.yaml)

Actions:

1. Replace the canonical local protocol-family constant from `uaa.agent.session` to `substrate.agent.session`.
2. Update validator success paths, error text, config examples, emitted traces, transport payloads, and durable writes to the new canonical label.
3. Rewrite supported fixtures, checked-in examples, and test inventories in the same cutover.
4. Make stale old-label rows fail closed with explicit operator-readable errors after cutover. Do not silently accept them as canonical.
5. Preserve all true upstream `agent_api.*` ids and `uaa_session_id` semantics unchanged.

Exit criteria:

1. `protocol: substrate.agent.session` validates and shows up in emitted live surfaces.
2. `uaa.agent.session` is no longer documented or accepted as the preferred supported local label.
3. No public selector or upstream capability id semantics changed.

### Phase 3: Rename local transport and daemon families directly

Files and surfaces:

1. [Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.toml)
2. [crates/world-agent/](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/)
3. [crates/agent-api-types/](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/)
4. [crates/agent-api-core/](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-core/)
5. [crates/agent-api-client/](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/)
6. mixed-boundary consumers such as [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) and [`member_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)

Actions:

1. Rename `crates/world-agent` to `crates/world-service`, package `world-agent` to `world-service`, and binary `world-agent` to `world-service`.
2. Rename installed alias and service family from `substrate-world-agent` to `substrate-world-service`.
3. Rename the local contract crates to `transport-api-types`, `transport-api-core`, and `transport-api-client`, including their Rust crate ids.
4. Update all imports, dependency keys, workspace member entries, crate references, and tests to the new names.
5. Preserve `world-api` unchanged.
6. Tighten comments and tests in mixed-boundary files so the upstream-vs-local distinction is explicit.

Exit criteria:

1. The workspace builds with the renamed local families.
2. Mixed-boundary files read cleanly with upstream `agent_api` alongside local `transport_api_*`.
3. No supported live surface still treats `world-agent` or `agent-api-*` as canonical names.

### Phase 4: Sweep installers, platform helpers, release bundles, and CI

Files and surfaces:

1. [scripts/linux/world-provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh)
2. [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh)
3. [scripts/substrate/dev-install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh)
4. [scripts/substrate/uninstall-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/uninstall-substrate.sh)
5. [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh)
6. [scripts/windows/wsl-warm.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/wsl-warm.ps1)
7. [scripts/windows/uninstall-substrate.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/uninstall-substrate.ps1)
8. [dist/scripts/assemble-release-bundles.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/scripts/assemble-release-bundles.sh)
9. [dist/release-template.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/release-template.md)
10. [`.github/workflows/feature-smoke.yml`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/feature-smoke.yml) and [`.github/workflows/nightly.yml`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/nightly.yml)

Actions:

1. Update install, warm, provision, uninstall, and smoke flows to the renamed binary, socket, and unit names.
2. Update release bundle assembly so host and guest payloads stage `world-service` in the right locations.
3. Update CI package invocations, smoke checks, and release checks to the renamed package and service family names.
4. Update doctor, remediation, and helper output so operators are never pointed back at `substrate-world-agent`.

Exit criteria:

1. Linux, macOS Lima, and WSL helpers all target `world-service`.
2. Release bundle and CI surfaces prove the rename beyond code compilation.
3. Operator guidance and helper text are consistent with the new service family.

### Phase 5: Sweep boundary docs, ADRs, and developer guidance

Files and surfaces:

1. [docs/CONFIGURATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md)
2. [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
3. [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
4. [docs/INSTALLATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/INSTALLATION.md)
5. [docs/UNINSTALL.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/UNINSTALL.md)
6. [docs/COMMANDS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/COMMANDS.md)
7. [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
8. [docs/REPLAY.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/REPLAY.md)
9. [docs/reference/env/contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/env/contract.md)
10. [docs/manual_verification/linux_world_socket.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/manual_verification/linux_world_socket.md)
11. [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md)
12. [docs/cross-platform/wsl_world_troubleshooting.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/wsl_world_troubleshooting.md)
13. [README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/README.md)
14. [AGENTS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENTS.md)
15. relevant ADRs including ADR-0042, ADR-0044, ADR-0045, and ADR-0021

Actions:

1. Rewrite operator and developer docs so upstream UAA and local transport or service layers are never described as the same thing.
2. Rewrite doc examples to `substrate.agent.session` and the renamed `world-service` and `transport-api-*` families.
3. Keep historical references only where explicitly marked as historical or archived.
4. Update the gap matrix so the boundary row is no longer "partially clarified."

Exit criteria:

1. The same exact boundary story appears in docs, ADRs, code comments, and install guides.
2. No live operator doc still tells someone to inspect `substrate-world-agent.service` or use `uaa.agent.session`.

### Phase 6: Validation and closeout

Files and surfaces:

1. repo root grep wall
2. target package tests
3. platform helper and smoke commands
4. release bundle assembly

Actions:

1. Run the grep wall on live surfaces and confirm zero stale hits outside the historical allowlist.
2. Run workspace tests and the targeted renamed-package tests.
3. Run Linux, Lima, and WSL validation commands against the renamed surfaces.
4. Run release bundle assembly and verify renamed payloads.
5. Confirm explicit fail-closed behavior for rename drift and old local protocol label usage.

Exit criteria:

1. The rename is proven across code, scripts, docs, bundles, and operator flows.
2. Old-name drift is either gone or explicitly historical.

## Test Review

Rust and shell or operator validation are the authoritative test layers for this slice.

### Code path coverage

```text
CODE PATH COVERAGE
==================
[+] Local protocol-family cutover
    mapping.rs
        -> canonical label constant
    validator.rs
        -> config validation
    agent_events.rs / agents_cmd.rs / state_store.rs
        -> emitted trace + persisted state + status grouping
    Required tests:
        - substrate.agent.session validates
        - emitted trace rows use substrate.agent.session
        - stale uaa.agent.session input fails clearly after cutover

[+] Local transport-family rename
    Cargo.toml
        -> workspace members + dependency keys
    renamed transport-api-* crates
        -> package names + Rust crate ids
    mixed-boundary consumers
        -> upstream agent_api stays unchanged
    Required tests:
        - mixed-boundary crates compile cleanly
        - imports remain unambiguous

[+] world-service rename
    renamed world-service crate
        -> package + binary + tests
    socket activation / world routing / doctor paths
        -> renamed binary + socket + unit discovery
    Required tests:
        - service discovery uses new names
        - world-required routing still fails closed if service is missing

[+] Operator and release surfaces
    install/dev-install/uninstall scripts
    Lima/WSL warm and smoke flows
    release bundle assembly
    feature-smoke and nightly workflows
    Required tests:
        - renamed payloads are staged
        - renamed unit names are enabled and queried
        - CI invokes renamed packages and binaries
```

### User-flow and error-state coverage

```text
USER FLOW COVERAGE
==================
[+] New config or inventory author
    - writes protocol: substrate.agent.session
    - validator accepts it
    - docs show the same label

[+] Operator provisions or warms a world
    - install or provision script stages world-service
    - systemd unit names are substrate-world-service.*
    - doctor and remediation text point at world-service, not world-agent

[+] World-required execution path
    - renamed service discovery succeeds
    - if it fails, command errors explicitly
    - no host fallback when require_world=true

[+] Contributor runs CI or bundle commands
    - cargo package names are renamed
    - feature-smoke and nightly still pass
    - release bundles contain world-service payloads

[+] Legacy unsupported old-name input
    - uaa.agent.session row or config is encountered post-cutover
    - validator or load path rejects it clearly
    - user sees explicit remediation, not silent drift
```

### Concrete test additions

| Target | Test requirement | Type |
| --- | --- | --- |
| [`crates/shell/tests/agents_validate.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agents_validate.rs) and [`validator.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) | Validate `protocol: substrate.agent.session`; reject `uaa.agent.session` with explicit post-cutover error text. | focused unit and integration |
| [`crates/common/src/agent_events.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs), [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs), [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs) | Prove trace, status, and grouping emit and consume `substrate.agent.session` without regression. | focused unit and integration |
| renamed `transport-api-*` crates plus mixed-boundary consumers such as [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Compile and runtime tests that prove upstream `agent_api` remains distinct from renamed local `transport_api_*` crates. | compile and focused unit |
| renamed `world-service` crate tests, including current `world-agent` test families | Keep daemon execute, stream, cancel, and member-runtime behavior intact under the new package and binary names. | crate unit and integration |
| [`socket_activation.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/socket_activation.rs), [`platform/linux.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/linux.rs), [`routing/world.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs) | Prove rename drift still fails closed when service or socket discovery is broken. | focused integration |
| install, warm, smoke, and bundle helpers under `scripts/`, `dist/`, and `.github/` | Prove renamed package invocations, renamed payload staging, and renamed service or socket names across Linux, Lima, and WSL. | shell smoke and CI validation |
| repo-root grep wall | Prove no stale old-name hits remain on live surfaces, while positive guardrails still find upstream `agent_api.*` and unchanged `world-api`. | static validation |

### Regression rule for this slice

Two regressions are mandatory blockers:

1. any path that causes world-required routing to stop failing closed because a renamed binary, socket, or unit was missed
2. any path that leaves supported live surfaces still advertising or emitting `uaa.agent.session`

No defer. No "docs follow-up." These are slice-completion gates.

## Failure Modes Registry

| Failure mode | Where it happens | Required handling | Test requirement | Critical gap if missing |
| --- | --- | --- | --- | --- |
| World-required routing falls back to host execution because renamed service discovery broke | socket activation, world routing, platform helpers | fail closed with explicit error | fail-closed routing tests | Yes |
| Renamed unit or binary is missed in install or warm flows | Linux, Lima, WSL scripts | explicit install or warm failure with renamed guidance | platform smoke and doctor checks | Yes |
| Release bundles still stage old payload names | `dist/` scripts and release template | bundle assembly fails or is corrected before release | release gate checks | Yes |
| Mixed-boundary code becomes more confusing after rename | shell and world-service consumers | explicit import and comment cleanup | compile and focused tests | No, but fix in-slice |
| Old `uaa.agent.session` rows or configs drift through silently | validator and persistence load paths | explicit rejection after cutover | validator and state tests | Yes |
| Docs or remediation text still point operators to `substrate-world-agent` | docs, doctor output, helper strings | rewrite all live guidance | grep wall plus manual validation | No, but unacceptable for ship |

Any failure mode with no test, no explicit handling, and a silent or misleading user experience is a release blocker for this slice.

## Performance Review

This slice is not driven by runtime throughput, but there are still real performance and operational guardrails:

1. No repeated service-discovery loops should be added just to paper over rename drift.
2. No compatibility shim should sit on the hot path for supported live flows after cutover.
3. No extra validation pass should be added to the steady-state execution path beyond the existing semantics required to enforce the renamed label.
4. CI and bundle validation may get broader, but runtime execution should remain unchanged in steady state.
5. The dominant cost center is support churn, not CPU time. The plan should optimize for zero operator ambiguity.

## NOT in scope

1. Renaming true upstream `agent_api` imports or any upstream `agent_api.*` capability or schema ids.
2. Renaming `world-api`.
3. Changing public session selectors or widening public control surfaces.
4. Reworking HTTP path schemas unless a rename is explicitly required by the chosen service family.
5. Bulk-rewriting archived evidence packs, old planning logs, or historical artifacts just to purge old tokens.
6. Renaming generic `SUBSTRATE_*AGENT*` environment variables solely because they contain the word `agent`.

## Deferred Follow-Ups

1. Optional archival cleanup for historical documents after the live-surface wall is green.
2. Any future naming cleanup for non-blocking environment variable families, only if it earns its own scope and rollout story.
3. Any later public contract cleanup beyond the local transport and service boundary, only if separately justified.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Freeze rename contract and grep boundary | `Cargo.toml`, `docs/`, `README.md`, `AGENTS.md` | — |
| 2. Cut over `substrate.agent.session` | `crates/common/`, `crates/shell/src/execution/agent_runtime/`, `config/` | 1 |
| 3. Rename `world-agent*` and `agent-api*` families | `crates/world-service/`, `crates/transport-api-*/`, `Cargo.toml`, mixed-boundary consumers | 1 |
| 4. Sweep scripts, CI, and release bundles | `scripts/`, `.github/`, `dist/` | 1, 3 |
| 5. Sweep docs and ADRs | `docs/`, `README.md`, `AGENTS.md` | 1, 2, 3, 4 |
| 6. Run validation wall and closeout | repo root, platform helpers, release scripts | 2, 3, 4, 5 |

### Parallel lanes

Lane A: Step 2 -> Step 3  
Reason: shared crate and Cargo naming work, one owner, sequential

Lane B: Step 4  
Reason: scripts, CI, and bundle surfaces are mostly isolated once Step 3 names are frozen

Lane C: Step 5  
Reason: docs and ADR rewrites can proceed in parallel once the final names are frozen, but they should trail the code rename enough to avoid doc churn

Lane D: Step 6  
Reason: validation must run after A, B, and C converge

### Execution order

1. Land Step 1 first. No parallel work before the rename matrix is frozen.
2. Launch Lane A first. The protocol cutover and code-family rename define the real names everyone else must reference.
3. Launch Lane B once the renamed package, binary, and unit names are stable enough to wire into scripts and CI.
4. Launch Lane C once the same names are stable enough for docs. It can run in parallel with Lane B.
5. Run Lane D last. Validation is the merge gate for the whole slice.

### Conflict flags

1. Lane A and Lane B both indirectly depend on exact package, binary, and unit names. Freeze those before parallelizing.
2. Lane B and Lane C both touch operator command strings and remediation copy. Keep one canonical rename matrix open while both lanes run.
3. Do not split Step 2 and Step 3 across different owners if they both touch the same mixed-boundary consumers in `crates/shell/`.

## Validation Commands

### Grep Gates

Run after the rename and expect zero live-surface hits unless the hit is in the explicit historical allowlist.

```bash
rg -n "world-agent|substrate-world-agent|agent-api-types|agent-api-core|agent-api-client|agent_api_types|agent_api_core|agent_api_client|uaa\.agent\.session" \
  Cargo.toml crates scripts docs .github dist README.md AGENTS.md config
```

Positive guardrails that must still return expected upstream and unchanged-boundary hits:

```bash
rg -n "agent_api\.run|agent_api\.session\.resume\.v1|agent_api\.session\.handle\.v1" crates docs config
rg -n "world-api" Cargo.toml crates docs
```

### Cargo Gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace -- --nocapture
cargo test -p shell -- --nocapture
cargo test -p world-service -- --nocapture
cargo test -p transport-api-types -- --nocapture
cargo test -p transport-api-core -- --nocapture
cargo test -p transport-api-client -- --nocapture
cargo test -p world-mac-lima -- --nocapture
cargo test -p world-windows-wsl -- --nocapture
cargo test -p substrate-replay -- --nocapture
```

### Operator Surface Gates

Linux:

```bash
systemctl status substrate-world-service.socket --no-pager
systemctl status substrate-world-service.service --no-pager
substrate host doctor --json | jq .
substrate world doctor --json | jq .
```

macOS Lima:

```bash
scripts/mac/lima-warm.sh --check-only
scripts/mac/smoke.sh
limactl shell substrate systemctl status substrate-world-service.socket
limactl shell substrate systemctl status substrate-world-service.service
```

WSL:

```powershell
pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .)
pwsh -File scripts/windows/wsl-smoke.ps1
pwsh -File scripts/windows/wsl-doctor.ps1
```

### Release Gates

```bash
./dist/scripts/assemble-release-bundles.sh
rg -n "world-service|substrate-world-service" dist/release-template.md dist/scripts/assemble-release-bundles.sh
```

### Fail-Closed Checks

1. `protocol: substrate.agent.session` validates.
2. Old `uaa.agent.session` live configs or fixtures are rejected clearly after cutover.
3. World-required routing still hard-fails if renamed service discovery breaks.
4. No doctor, help, or remediation string points operators back to `substrate-world-agent`.

### Manual validation proof points

Manual validation for this slice should explicitly prove:

1. a canonical new config entry using `protocol: substrate.agent.session`
2. the local in-world daemon is renamed to the `world-service` family through package, binary, installed alias, bundle path, and systemd unit naming
3. the local host<->world contract crates are renamed to the `transport-api-*` family
4. `world-api` remains unchanged
5. pure-agent trace output now emits `substrate.agent.session`
6. stale old-name live surfaces are gone outside the historical allowlist
7. world-required rename drift still fails closed
8. true upstream `agent_api.*` capability and session-handle wording is unchanged

## Completion Summary

This plan is ready to implement when the work lands with this end state:

1. Step 0: scope accepted as-is, direct cutover on supported live surfaces, no upstream `agent_api` rename, no `world-api` rename.
2. Architecture: one stable boundary story, one rename matrix, one live-surface wall.
3. Code quality: local transport and service families are renamed truthfully, mixed-boundary code is clearer, and no duplicated rename rules remain.
4. Tests: protocol cutover, mixed-boundary compile paths, service discovery, fail-closed routing, install or warm flows, bundle staging, and grep gates are all covered.
5. Performance: no new hot-path compatibility shims, no extra fallback loops, no runtime ambiguity.
6. `NOT in scope`: written.
7. `What already exists`: written.
8. Failure modes: six named production failures accounted for, four of them critical if left untested.
9. Parallelization: four lanes, with one code lane and two safe side lanes after the contract freeze.
10. Lake score: the complete rename and validation wall wins over the fake-cheap partial rename.
