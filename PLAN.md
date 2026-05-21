<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/testing-autoplan-restore-20260521-120419.md -->
# PLAN: UAA Boundary and Naming Cleanup

Source SOW: [27-uaa-boundary-and-naming-cleanup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/27-uaa-boundary-and-naming-cleanup.md)  
Primary boundary anchors: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:53), [ADR-0042](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md:120), [ADR-0044](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md:177), [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:202), [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:235), [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md:184)  
Current workspace branch: `chore/uaa-boundary-and-naming-cleanup`  
Base branch: `main`  
Plan type: repo-wide naming and boundary cleanup, no runtime-model redesign  
Status: rewritten execution plan, unified to `/plan-eng-review` rigor on 2026-05-21

## Objective

Finish the naming cleanup so the repo tells one stable story and operators do not have to infer boundary meaning from context.

After this slice:

1. `Unified Agent API`, `UAA`, Rust `agent_api`, and adopted `agent_api.*` capability or schema ids mean the upstream CLI-agent runtime abstraction only.
2. `world-api` still means the abstract world backend contract only.
3. `transport-api-*` means the Substrate-local typed host↔world contract layer only.
4. `world-service*` means the Substrate-local in-world daemon or service boundary only.
5. `substrate.agent.session` is the only canonical local pure-agent protocol-family label on supported live surfaces.

This is a direct-cutover naming correction. It is not a runtime redesign, not a public contract redesign, and not a compatibility-migration project.

## Locked Decisions

These choices are already made. Implementation does not reopen them.

| Topic | Locked decision | Why |
| --- | --- | --- |
| Upstream UAA naming | Keep upstream `agent_api` and adopted `agent_api.*` ids unchanged | Those names are already correct and already mean the upstream runtime layer |
| Abstract world contract | Keep `world-api` unchanged | It already names the abstraction, not the in-world daemon |
| Local protocol-family label | Rename `uaa.agent.session` to `substrate.agent.session` | The current name reads like an upstream claim when it is actually a local protocol-family |
| Local daemon family | Rename `world-agent*` to `world-service*` | The service is a daemon boundary, not an AI-agent identity |
| Local transport family | Rename `agent-api-*` to `transport-api-*` | The current family collides conceptually with the real upstream UAA |
| Compatibility posture | No long-lived alias layer on supported live surfaces | Partial rename keeps the ambiguity alive and increases support burden |
| Failure posture | Old live names fail closed with explicit operator-readable errors | Silent fallback would hide broken routing and broken state interpretation |

## Acceptance Criteria

This plan is complete only when all of the following are true:

1. `Unified Agent API`, `UAA`, Rust `agent_api`, and adopted `agent_api.*` capability and schema ids have one stable repo meaning: the upstream CLI-agent runtime abstraction only.
2. `substrate.agent.session` is the only canonical local pure-agent protocol-family label on supported live surfaces.
3. `world-agent*` has been replaced by `world-service*` across code, package names, binaries, install aliases, systemd units, release payloads, CI, and live docs.
4. `agent-api-types`, `agent-api-core`, and `agent-api-client` have been replaced by `transport-api-types`, `transport-api-core`, and `transport-api-client`, including the Rust crate ids `transport_api_types`, `transport_api_core`, and `transport_api_client`.
5. `world-api` remains unchanged everywhere.
6. Mixed-boundary crates that import both upstream `agent_api` and local transport crates read clearly after the rename.
7. Old `uaa.agent.session` configs, fixtures, or persisted rows are not treated as supported canonical inputs after cutover. They fail closed with explicit operator-readable errors.
8. The live-surface grep wall passes with zero stale hits for `world-agent`, `substrate-world-agent`, `agent-api-*`, `agent_api_*`, and `uaa.agent.session` outside the explicit historical allowlist.
9. Linux, macOS Lima, and WSL install, provision, warm, smoke, doctor, uninstall, and release flows all use the renamed `world-service` family consistently.
10. World-required routing still hard-fails if renamed service discovery breaks. No rename drift path silently falls back to host execution.
11. Docs, ADRs, tests, fixtures, comments, and operator remediation strings all tell the same boundary story.
12. No public-handle, session-selector, `uaa_session_id`, or upstream capability semantics regress.

## Scope

In scope:

1. freezing one canonical vocabulary for upstream UAA versus local transport versus local service naming,
2. renaming the canonical local protocol-family label from `uaa.agent.session` to `substrate.agent.session`,
3. renaming `world-agent*` to `world-service*`,
4. renaming `agent-api*` to `transport-api*`,
5. freezing one exact rename matrix for crate paths, package names, Rust crate ids, binaries, install aliases, service units, socket names, bundle paths, and doctor or remediation strings,
6. updating code, tests, fixtures, docs, ADRs, scripts, CI, bundle assembly, and operator playbooks together,
7. defining the live-surface grep boundary and the explicit historical allowlist,
8. preserving fail-closed behavior anywhere the repo already requires world isolation or protocol validation.

## NOT in scope

1. Renaming true upstream `agent_api` imports or any upstream `agent_api.*` capability or schema ids.
2. Renaming `world-api`.
3. Changing public session selectors, widening public control surfaces, or revisiting public handle semantics.
4. Changing `uaa_session_id` or `internal.uaa_session_id`.
5. Adding a migration command, startup rewrite, compatibility alias, or persisted-state upgrader for old `uaa.agent.session` rows.
6. Reworking HTTP path schemas unless a rename is directly required by the chosen service family.
7. Bulk-rewriting archived evidence packs, old planning logs, or historical artifacts just to purge old tokens.
8. Renaming generic `SUBSTRATE_*AGENT*` environment variables solely because they contain the word `agent`.

## Step 0: Scope Challenge

### What already exists

| Sub-problem | Existing code or surface | Reuse decision |
| --- | --- | --- |
| Upstream UAA runtime boundary | [`crates/shell/Cargo.toml`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/Cargo.toml:61), [`crates/world-agent/Cargo.toml`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml:43), [docs/contracts/substrate-gateway-backend-adapter-schema.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-schema.md:21) | Reuse exactly. Upstream `agent_api` stays untouched. |
| Local pure-agent protocol label | [`mapping.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mapping.rs:3), [`validator.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:76), [`agent_events.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:17), [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2093) | Reuse the seam, rename the constant and every supported live consumer in one pass. |
| Local typed host↔world contract | [`crates/agent-api-types/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/), [`crates/agent-api-core/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-core/), [`crates/agent-api-client/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/) | Reuse the contract, rename the family to `transport-api-*`. |
| In-world daemon and control surface | [`crates/world-agent/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/), [`scripts/linux/world-provision.sh`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh:527), [`docs/WORLD.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:237) | Reuse the implementation, rename the family to `world-service*`. |
| Install, warm, release, and CI harnesses | [`scripts/substrate/install-substrate.sh`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:2041), [`scripts/mac/lima-warm.sh`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:686), [`scripts/windows/wsl-warm.ps1`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/wsl-warm.ps1:57), [`dist/scripts/assemble-release-bundles.sh`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/scripts/assemble-release-bundles.sh:261), [`.github/workflows/feature-smoke.yml`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/feature-smoke.yml:204) | First-class implementation scope, not follow-up polish. |
| Boundary wording and gap framing | [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:53), [`docs/WORLD.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:237), [`docs/TRACE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md:184) | Reuse as truth anchors, then rewrite all live wording to match. |

### Minimum viable change

The minimum honest change is still broad:

1. rename the local protocol-family label,
2. rename the local daemon and transport families,
3. update all service discovery and packaging surfaces that reference those names,
4. update docs and operator guidance in the same slice,
5. prove the rename with tests, grep gates, and platform validation.

Anything smaller creates a half-renamed repo. That is worse than the current state because operators would now have two partially valid vocabularies instead of one bad one.

### Complexity and distribution check

1. This slice touches far more than 8 files and more than 2 modules. That is a smell in most features. Here it is justified because the ambiguity already spans manifests, crates, scripts, CI, release packaging, and docs.
2. No new artifact type is introduced. But distribution still matters because install scripts, release bundles, and systemd units are user-facing delivery surfaces. If they are not renamed in the same slice, users cannot actually consume the cleanup.
3. There is no built-in framework feature that can do this cleanup for us. This is a repo-owned naming and packaging correction, not a library toggle.
4. The right scope posture is "broad but boring": direct rename, explicit validation, no extra abstraction, no migration subsystem, no alias layer.

### Target architecture after rename

```text
Upstream CLI-agent runtime
    Unified Agent API / agent_api / agent_api.*
                    |
                    v
            substrate shell runtime
                    |
      +-------------+-------------+
      |                           |
      v                           v
transport-api-*              world-api
(typed host<->world          (abstract backend
 contract layer)              contract)
      |
      v
world-service
(in-world daemon / REST + WS)
      |
      v
linux world / Lima guest / WSL guest

Local pure-agent protocol-family:
    substrate.agent.session
```

### Exact remaining gap

The architecture already exists. The gap is naming drift across live surfaces:

1. the repo contains both real upstream `agent_api` integration and local `agent-api-*` transport crates,
2. the repo contains both abstract `world-api` and concrete `world-agent` daemon naming,
3. the local protocol-family is still `uaa.agent.session` in config, validators, traces, transport payloads, and durable state,
4. installers, bundle scripts, and doctor text still expose the old daemon family names,
5. docs and ADRs still risk implying the local transport layer is itself upstream UAA.

## Frozen Execution Contract

If implementation wants to violate any rule below, stop and revise the plan first.

1. Upstream `agent_api` and `agent_api.*` remain untouched.
2. `world-api` remains untouched.
3. `substrate.agent.session` is the only canonical supported local protocol-family label after cutover.
4. Supported live surfaces cut over directly. Do not add permanent compatibility aliases just to avoid touching code, docs, or scripts.
5. Unsupported stale rows and configs fail closed with explicit error text, not silent acceptance and not silent downgrade.
6. World-required flows must not fall back to host execution because a renamed binary, socket, or unit was missed.
7. Operator-facing unit names, install aliases, release payload names, and doctor text must match the code rename in the same slice.
8. Mixed-boundary files must read more clearly after the rename than before.
9. The plan lands in this order: freeze names first, rename live code second, sweep scripts and docs third, validate last.

## Canonical Rename Matrix

| Current canonical name | New canonical name | Decision |
| --- | --- | --- |
| `uaa.agent.session` | `substrate.agent.session` | rename |
| `crates/world-agent` | `crates/world-service` | rename |
| package `world-agent` | package `world-service` | rename |
| binary `world-agent` | binary `world-service` | rename |
| installed alias `substrate-world-agent` | `substrate-world-service` | rename |
| units `substrate-world-agent.service` / `.socket` | `substrate-world-service.service` / `.socket` | rename |
| socket or fd names containing `substrate-world-agent` | matching `substrate-world-service` names | rename |
| `scripts/mac/substrate-world-agent.service` | matching `world-service` filename and `ExecStart` | rename |
| release payload `bin/world-agent` and `bin/linux/world-agent` | `bin/world-service` and `bin/linux/world-service` | rename |
| `crates/agent-api-types` | `crates/transport-api-types` | rename |
| `crates/agent-api-core` | `crates/transport-api-core` | rename |
| `crates/agent-api-client` | `crates/transport-api-client` | rename |
| `agent_api_types` / `agent_api_core` / `agent_api_client` | `transport_api_types` / `transport_api_core` / `transport_api_client` | rename |
| workspace dependency keys `agent-api-*` | `transport-api-*` | rename |
| live embedded file or module identifiers such as `world_agent` | rename when they remain live support code | rename |
| `world-api` | `world-api` | keep |
| upstream `agent_api` | upstream `agent_api` | keep |
| upstream `agent_api.*` ids | upstream `agent_api.*` ids | keep |

## Live Rename Boundary And Historical Allowlist

### Live surfaces that must go green

1. `Cargo.toml`
2. `dist-workspace.toml`
3. `crates/**`
4. `scripts/**`
5. `.github/**`
6. `dist/**`
7. `README.md`
8. `AGENTS.md`
9. `config/**`
10. `docs/**`
11. `macos-hardening/**`

### Historical allowlist

Old tokens may remain only when the file is intentionally historical or archival, for example:

1. `docs/project_management/_archived/**`
2. historical planning packets and evidence packs that are not normative runtime or operator guidance
3. older `llm-last-mile/**` artifacts other than this root `PLAN.md`

### Grep policy

This slice does not allow vague language like "repo-wide rename." It requires both of these to be explicit:

1. the live-surface command wall that must be green before the slice is done,
2. the historical allowlist that explains why any remaining old-name hits still exist.

## Implementation Plan

### Phase summary

| Phase | Purpose | Modules or surfaces touched | Hard dependency |
| --- | --- | --- | --- |
| 1 | Freeze vocabulary, rename matrix, and grep boundary | `PLAN.md`, `AGENT_ORCHESTRATION_GAP_MATRIX.md`, top-level truth docs | — |
| 2 | Cut over the local protocol-family label | `crates/common/`, `crates/shell/src/execution/agent_runtime/`, `config/`, relevant fixtures and tests | 1 |
| 3 | Rename local transport and daemon families | `Cargo.toml`, `crates/world-agent/`, `crates/agent-api-*/`, mixed-boundary consumers | 1, 2 |
| 4 | Sweep installers, platform helpers, release bundles, and CI | `scripts/`, `dist/`, `.github/`, doctor or remediation text | 3 |
| 5 | Sweep docs, ADRs, and developer guidance | `docs/`, `README.md`, `AGENTS.md`, ADRs, truth docs | 2, 3 |
| 6 | Run grep wall, package tests, operator validation, and closeout | repo root, platform helpers, release scripts | 2, 3, 4, 5 |

### Phase 1: Freeze vocabulary, rename matrix, and grep boundary

Primary surfaces:

1. [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)
2. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
3. [README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/README.md)
4. [AGENTS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENTS.md)
5. [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
6. [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)

Required actions:

1. Freeze one authoritative vocabulary rule: upstream `agent_api`, local `transport-api-*`, local `world-service`, unchanged `world-api`, local `substrate.agent.session`.
2. Freeze one exact rename matrix for crate paths, package names, Rust crate ids, binaries, install aliases, service units, socket names, bundle paths, and doctor or remediation strings.
3. Freeze the live rename boundary and historical allowlist before grep work starts.
4. Record the exact grep wall commands in this plan so contributors are checking the same surfaces.

Done when:

1. every later phase can point back to one exact rename matrix,
2. no implementation phase has to guess which surfaces are live and which are historical,
3. no later phase invents new names or new exceptions.

### Phase 2: Cut over the local protocol-family label to `substrate.agent.session`

Primary surfaces:

1. [crates/common/src/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:17)
2. [crates/shell/src/execution/agent_runtime/mapping.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mapping.rs:3)
3. [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:76)
4. [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:57)
5. [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:80)
6. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2093)
7. [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:203)
8. [config/agents/codex.yaml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex.yaml:6)

Required actions:

1. Replace the canonical local protocol-family constant from `uaa.agent.session` to `substrate.agent.session`.
2. Update validator success paths, error text, config examples, emitted traces, transport payloads, and durable writes to the new canonical label.
3. Rewrite supported fixtures, checked-in examples, and test inventories in the same cutover.
4. Make stale old-label rows fail closed with explicit operator-readable errors after cutover.
5. Preserve all true upstream `agent_api.*` ids and `uaa_session_id` semantics unchanged.

Done when:

1. `protocol: substrate.agent.session` validates and shows up in emitted live surfaces,
2. `uaa.agent.session` is no longer documented or accepted as the preferred supported local label,
3. old rows or configs fail with explicit guidance instead of being silently interpreted,
4. no public selector or upstream capability semantics changed.

### Phase 3: Rename local transport and daemon families directly

Primary surfaces:

1. [Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.toml)
2. [crates/world-agent/](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/)
3. [crates/agent-api-types/](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/)
4. [crates/agent-api-core/](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-core/)
5. [crates/agent-api-client/](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/)
6. mixed-boundary consumers such as [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:12) and [`crates/world-agent/src/member_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:1)

Required actions:

1. Rename `crates/world-agent` to `crates/world-service`, package `world-agent` to `world-service`, and binary `world-agent` to `world-service`.
2. Rename the installed alias and service family from `substrate-world-agent` to `substrate-world-service`.
3. Rename the local contract crates to `transport-api-types`, `transport-api-core`, and `transport-api-client`, including their Rust crate ids.
4. Update all imports, dependency keys, workspace members, crate references, tests, helper filenames, and live module identifiers to the new names.
5. Preserve `world-api` unchanged.
6. Tighten comments and mixed-boundary docs in touched files so the upstream-versus-local distinction is explicit.

Done when:

1. the workspace builds with the renamed local families,
2. mixed-boundary files read cleanly with upstream `agent_api` alongside local `transport_api_*`,
3. no supported live surface still treats `world-agent` or `agent-api-*` as canonical names.

### Phase 4: Sweep installers, platform helpers, release bundles, and CI

Primary surfaces:

1. [scripts/linux/world-provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh:527)
2. [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:2041)
3. [scripts/substrate/dev-install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh)
4. [scripts/substrate/uninstall-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/uninstall-substrate.sh)
5. [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:686)
6. [scripts/windows/wsl-warm.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/wsl-warm.ps1:57)
7. [scripts/windows/uninstall-substrate.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/uninstall-substrate.ps1)
8. [dist/scripts/assemble-release-bundles.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/scripts/assemble-release-bundles.sh:261)
9. [dist/release-template.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/release-template.md:33)
10. [`.github/workflows/feature-smoke.yml`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/feature-smoke.yml:204)
11. [`.github/workflows/nightly.yml`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/nightly.yml:164)

Required actions:

1. Update install, warm, provision, uninstall, and smoke flows to the renamed binary, socket, and unit names.
2. Update release bundle assembly so host and guest payloads stage `world-service` in the right locations.
3. Update CI package invocations, smoke checks, and release checks to the renamed package and service family names.
4. Update doctor, remediation, and helper output so operators are never pointed back at `substrate-world-agent`.
5. Make upgrade-capable paths remove any legacy `substrate-world-agent.service`, `substrate-world-agent.socket`, and `substrate-world-agent` binary before enabling or probing the new family.

Done when:

1. Linux, macOS Lima, and WSL helpers all target `world-service`,
2. release bundle and CI surfaces prove the rename beyond code compilation,
3. operator guidance and helper text are consistent with the new service family.

### Phase 5: Sweep boundary docs, ADRs, and developer guidance

Primary surfaces:

1. [docs/CONFIGURATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md:42)
2. [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md:184)
3. [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:237)
4. [docs/INSTALLATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/INSTALLATION.md:13)
5. [docs/UNINSTALL.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/UNINSTALL.md:43)
6. [docs/COMMANDS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/COMMANDS.md:81)
7. [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:169)
8. [docs/REPLAY.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/REPLAY.md:82)
9. [docs/reference/env/contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/env/contract.md:63)
10. [docs/manual_verification/linux_world_socket.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/manual_verification/linux_world_socket.md:12)
11. [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md:108)
12. [docs/cross-platform/wsl_world_troubleshooting.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/wsl_world_troubleshooting.md:163)
13. [README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/README.md)
14. [AGENTS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENTS.md)
15. ADRs including ADR-0042, ADR-0044, ADR-0045, and ADR-0021

Required actions:

1. Rewrite operator and developer docs so upstream UAA and local transport or service layers are never described as the same thing.
2. Rewrite doc examples to `substrate.agent.session` and the renamed `world-service` and `transport-api-*` families.
3. Keep historical references only where explicitly marked as historical or archived.
4. Update the gap matrix so the boundary row is no longer partially clarified or open-ended.

Done when:

1. the same exact boundary story appears in docs, ADRs, code comments, and install guides,
2. no live operator doc still tells someone to inspect `substrate-world-agent.service` or use `uaa.agent.session`.

### Phase 6: Validation and closeout

Required actions:

1. Run the grep wall on live surfaces and confirm zero stale hits outside the historical allowlist.
2. Run workspace tests and the targeted renamed-package tests.
3. Run Linux, Lima, and WSL validation commands against the renamed surfaces.
4. Run release-bundle assembly and verify renamed payloads.
5. Confirm explicit fail-closed behavior for rename drift and old local protocol label usage.

Done when:

1. the rename is proven across code, scripts, docs, bundles, and operator flows,
2. old-name drift is either gone or explicitly historical,
3. the repo reads as if the boundary was intentional from the start.

## Test Review

Rust tests, shell or operator validation, and grep gates are the authoritative test layers for this slice.

### Code path coverage

```text
CODE PATH COVERAGE
==================
[+] Local protocol-family cutover
    mapping.rs
        -> canonical label constant
    validator.rs
        -> config validation + failure text
    agent_events.rs / agents_cmd.rs / state_store.rs
        -> emitted trace + persisted state + status grouping
    world_ops.rs and typed payloads
        -> transport propagation
    Required proof:
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
    Required proof:
        - mixed-boundary crates compile cleanly
        - imports remain unambiguous

[+] world-service rename
    renamed world-service crate
        -> package + binary + tests
    socket activation / world routing / doctor paths
        -> renamed binary + socket + unit discovery
    Required proof:
        - service discovery uses new names
        - world-required routing still fails closed if service is missing

[+] Operator and release surfaces
    install/dev-install/uninstall scripts
    Lima/WSL warm and smoke flows
    release bundle assembly
    feature-smoke and nightly workflows
    Required proof:
        - renamed payloads are staged
        - renamed unit names are enabled, queried, and cleaned up on upgrade paths
        - CI invokes renamed packages and binaries
```

### Concrete test additions

| Target | Test requirement | Type |
| --- | --- | --- |
| [`crates/shell/tests/agents_validate.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agents_validate.rs:82) and [`validator.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:76) | Validate `protocol: substrate.agent.session`; reject `uaa.agent.session` with explicit post-cutover error text that tells operators the old label is unsupported. | focused unit and integration |
| [`crates/common/src/agent_events.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:299), [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:12), [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:2429) | Prove trace, status, and grouping emit and consume `substrate.agent.session` without regression. | focused unit and integration |
| renamed `transport-api-*` crates plus mixed-boundary consumers such as [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:12) | Compile and runtime tests that prove upstream `agent_api` remains distinct from renamed local `transport_api_*` crates. | compile and focused unit |
| renamed `world-service` crate tests, including current `world-agent` test families | Keep daemon execute, stream, cancel, and member-runtime behavior intact under the new package and binary names. | crate unit and integration |
| [`socket_activation.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/socket_activation.rs:13), [`platform/linux.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/linux.rs:897), [`routing/world.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs:200) | Prove rename drift still fails closed when service or socket discovery is broken. | focused integration |
| install, warm, smoke, and bundle helpers under `scripts/`, `dist/`, and `.github/` | Prove renamed package invocations, renamed payload staging, and renamed service or socket names across Linux, Lima, and WSL. | shell smoke and CI validation |
| repo-root grep wall | Prove no stale old-name hits remain on live surfaces while positive guardrails still find upstream `agent_api.*` and unchanged `world-api`. | static validation |

### Regression rule for this slice

Two regressions are mandatory blockers:

1. any path that causes world-required routing to stop failing closed because a renamed binary, socket, or unit was missed,
2. any path that leaves supported live surfaces still advertising or emitting `uaa.agent.session`.

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

Any failure mode with no test, no explicit handling, and a silent or misleading operator experience is a release blocker for this slice.

## Performance And Complexity Review

1. This is a wide slice because the ambiguity is already cross-cutting. The breadth is justified.
2. No compatibility shim should sit on the hot path for supported live flows after cutover.
3. No repeated service-discovery retries should be added just to paper over rename drift.
4. The dominant cost center here is support churn and operator confusion, not CPU time. Optimize for zero ambiguity, explicit failure, and minimal moving parts.

## Deferred Follow-Ups

1. Optional archival cleanup for historical documents after the live-surface wall is green.
2. Any later naming cleanup for non-blocking environment variable families, only if it earns its own rollout story.
3. Any future public contract cleanup beyond the local transport and service boundary, only if separately justified.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A0. Freeze rename contract and grep boundary | `PLAN.md`, `docs/`, `README.md`, `AGENTS.md`, top-level truth docs | — |
| A1. Cut over `substrate.agent.session` | `crates/common/`, `crates/shell/src/execution/agent_runtime/`, `config/` | A0 |
| A2. Rename `world-agent*` and `agent-api*` families | `crates/world-service/`, `crates/transport-api-*/`, `Cargo.toml`, mixed-boundary consumers | A0, A1 |
| B. Sweep scripts, CI, and release bundles | `scripts/`, `.github/`, `dist/` | A2 |
| C. Sweep docs and ADRs | `docs/`, `README.md`, `AGENTS.md`, ADRs | A2 |
| D. Run validation wall and closeout | repo root, platform helpers, release scripts | A1, A2, B, C |

### Parallel lanes

Lane A: A0 -> A1 -> A2  
Reason: these steps share manifests, protocol validation, mixed-boundary imports, and `crates/shell/`. Splitting them creates merge-conflict bait and naming churn.

Lane B: B  
Reason: scripts, CI, and release surfaces are mostly isolated once the final package, binary, socket, and unit names are real.

Lane C: C  
Reason: docs and ADR rewrites can run in parallel with Lane B after implementation names are stable, but they should not start before code naming settles.

Lane D: D  
Reason: validation is the merge gate. It must run after A, B, and C converge.

### Execution order

1. Land A0 first. No parallel work before the rename matrix and grep boundary are frozen.
2. Run A1 next. The protocol label cutover touches the same runtime and persistence seams that later code work depends on.
3. Run A2 immediately after A1, ideally under the same owner or worktree lane, because both steps touch `Cargo.toml`, `crates/shell/`, and mixed-boundary imports.
4. Once A2 stabilizes the final code names, launch B and C in parallel.
5. Run D last. Validation is the only honest merge gate for this slice.

### Conflict flags

1. A1 and A2 both touch `crates/shell/`, runtime validation, and mixed-boundary imports. Do not split them across independent owners.
2. B depends on exact package, binary, socket, and unit names from A2. Starting B early creates rework.
3. C shares operator command strings and remediation copy with B. Keep the rename matrix open and visible while both lanes run.

### Parallelization verdict

This slice has one required foundation lane, two safe follow-on parallel lanes, and one final validation lane.

Peak honest parallelism is B + C. Everything before that is sequential because the same names and the same files are still moving.

## Validation Commands

### Grep gates

Run after the rename and expect zero live-surface hits unless the hit is in the explicit historical allowlist.

```bash
rg -n "world-agent|world_agent|substrate-world-agent|agent-api-types|agent-api-core|agent-api-client|agent_api_types|agent_api_core|agent_api_client|uaa\.agent\.session" \
  Cargo.toml dist-workspace.toml crates scripts docs .github dist README.md AGENTS.md config macos-hardening
```

Positive guardrails that must still return expected upstream and unchanged-boundary hits:

```bash
rg -n "agent_api\.run|agent_api\.session\.resume\.v1|agent_api\.session\.handle\.v1" crates docs config
rg -n "world-api" Cargo.toml crates docs
rg -n "world-service|substrate-world-service|transport-api-types|transport-api-core|transport-api-client|transport_api_types|transport_api_core|transport_api_client|substrate\.agent\.session" \
  Cargo.toml dist-workspace.toml crates scripts docs .github dist README.md AGENTS.md config macos-hardening
```

### Cargo gates

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

### Operator surface gates

Linux:

```bash
systemctl status substrate-world-service.socket --no-pager
systemctl status substrate-world-service.service --no-pager
systemctl list-unit-files | rg "substrate-world-agent"
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

### Release gates

```bash
./dist/scripts/assemble-release-bundles.sh
rg -n "world-service|substrate-world-service" dist/release-template.md dist/scripts/assemble-release-bundles.sh
```

### Fail-closed checks

1. `protocol: substrate.agent.session` validates.
2. Old `uaa.agent.session` live configs or persisted rows are rejected clearly after cutover.
3. World-required routing still hard-fails if renamed service discovery breaks.
4. No doctor, help, or remediation string points operators back to `substrate-world-agent`.

### Manual validation proof points

Manual validation for this slice should explicitly prove:

1. a canonical new config entry using `protocol: substrate.agent.session`,
2. the local in-world daemon is renamed to the `world-service` family through package, binary, installed alias, bundle path, and systemd unit naming,
3. the local host↔world contract crates are renamed to the `transport-api-*` family,
4. `world-api` remains unchanged,
5. pure-agent trace output now emits `substrate.agent.session`,
6. stale old-name live surfaces are gone outside the historical allowlist,
7. world-required rename drift still fails closed,
8. true upstream `agent_api.*` capability and session-handle wording is unchanged.

## Completion Summary

This plan is ready to implement when the work lands with this end state:

1. one stable boundary story,
2. one exact rename matrix,
3. one explicit live-surface grep wall,
4. one direct cutover for supported local protocol and local service or transport names,
5. one fail-closed story for rename drift and unsupported old protocol rows,
6. one operator truth set across Linux, Lima, WSL, CI, and release surfaces,
7. and one honest parallelization model: contract freeze first, code lane first, scripts and docs parallel second, validation last.

After this slice lands, the repo should read as if the boundary was intentional from the start:

1. upstream UAA is upstream UAA,
2. `world-api` is the abstract world backend contract,
3. local `transport-api-*` is the typed host↔world contract layer,
4. local `world-service` is the in-world daemon and service boundary,
5. and the local pure-agent protocol-family label is unmistakably Substrate-local.
