<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/testing-autoplan-restore-20260521-120419.md -->
# SOW: UAA Boundary And Naming Cleanup

Status: remaining-work draft. This SOW closes the remaining naming/governance gap called out in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:106) and [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:149). It is anchored to the current boundary framing in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:53), the normalized identity rules in [ADR-0042 — LLM and Agent Identity Tuple and Deployment Posture](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md:120), the pure-agent identity model in [ADR-0044 — Agent Hub Core Successor Identity Tuple Compatible](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md:177), the durable public-handle contract in [ADR-0047 — Host Orchestrator Durable Session and Parked-Resumable Ownership](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:202), and the live docs split in [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:235) and [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md:184).

This is not a runtime-model slice. The external Unified Agent API integration is already real. The shell-owned pure-agent runtime is already real. The host↔world typed transport is already real. The remaining gap is that the repo still names those seams in ways that are easy to misread.

The two primary local rename families in this slice are:

- `world-agent*` -> `world-service*`
- `agent-api*` -> `transport-api*`

Upstream `agent_api` does not change.
`world-api` does not change.

## Objective

Finish the boundary cleanup so the repo no longer blurs:

- the external Unified Agent API runtime abstraction,
- the Substrate-local host↔world typed transport,
- the Substrate-local in-world `world-service` daemon,
- and the Substrate-local pure-agent protocol-family label used in config, persistence, and trace identity.

This slice is done only when all of the following are true:

1. `Unified Agent API`, `UAA`, Rust `agent_api`, and adopted `agent_api.*` capability/schema ids have one stable meaning in the repo: the upstream CLI-agent runtime abstraction only.
2. The current local pure-agent protocol-family label is renamed away from `uaa.agent.session` to a clearly Substrate-local canonical label.
3. The pre-UAA local transport families are renamed directly:
   - `world-agent*` becomes `world-service*`
   - `agent-api*` becomes `transport-api*`
4. `world-api` keeps its current meaning as the abstract world backend contract.
5. Docs, ADRs, tests, fixtures, and comments stop implying that the local host↔world transport is itself “the UAA” or “an agent.”
6. The repo no longer carries local transport naming that collides conceptually with upstream UAA naming.

## This SOW Chooses The Cleanup Shape

This SOW intentionally makes the product/naming decision instead of leaving it open.

### 1. External UAA terms stay reserved for the real upstream runtime

The following keep their current upstream meaning and are not renamed here:

- crates.io package `unified-agent-api`
- Rust import name `agent_api`
- adopted capability/schema ids such as `agent_api.run`, `agent_api.session.resume.v1`, and `agent_api.session.handle.v1` documented in [docs/contracts/substrate-gateway-backend-adapter-schema.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-schema.md:21)
- surfaced internal correlation fields such as `internal.uaa_session_id` / `uaa_session_id`, which remain internal and correctly describe upstream session-handle identity in [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:120) and [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:210)

This slice does not rename upstream `agent_api` under any spelling. The rename target is only the pre-UAA local transport naming.

### 2. The canonical local pure-agent protocol-family label becomes `substrate.agent.session`

`uaa.agent.session` is no longer the canonical local protocol-family id.

The new canonical local label is:

- `substrate.agent.session`

Reason:

- it stays a normalized dotted id consistent with [ADR-0042](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md:127),
- it is clearly local to Substrate,
- and it stops looking like an upstream Unified Agent API wire/API claim.

### 3. Local transport naming is renamed directly, not aliased

Because this seam is still greenfield, this slice does not preserve the pre-UAA transport names as compatibility names.

This slice intentionally chooses direct rename:

- `world-agent` naming becomes `world-service`,
- `agent-api-types`, `agent-api-core`, and `agent-api-client` become `transport-api-types`, `transport-api-core`, and `transport-api-client`,
- and corresponding Rust crate/import names follow the same replacement:
  - `agent_api_types` -> `transport_api_types`
  - `agent_api_core` -> `transport_api_core`
  - `agent_api_client` -> `transport_api_client`

This slice also keeps `world-api` unchanged because that crate already names the abstract backend contract rather than the in-world daemon or host↔world transport layer.

That is the cleanest low-churn rename because it preserves the surrounding noun structure while separating:

- `world-api` as the abstract world backend contract,
- `transport-api-*` as the typed local host↔world contract layer,
- and `world-service` as the in-world daemon/service implementation.

## Already Landed And Assumed

This SOW assumes the following are already true and are not being redesigned here:

- `crates/shell` already depends on the real external Unified Agent API as `agent_api` in [crates/shell/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/Cargo.toml:61).
- `crates/world-agent` also already depends on the real external Unified Agent API for member runtime launch/control.
- `world-api` already owns the abstract world backend contract and is not part of this rename.
- The local typed host↔world transport remains a separate boundary documented today in [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:237) and [docs/contracts/substrate-gateway-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-runtime-parity.md:21), even though those docs still use the old pre-cleanup names.
- The public session/control contract is already frozen around `orchestration_session_id` plus exact `backend_id`; public callers do not target `participant_id`, `active_session_handle_id`, or `internal.uaa_session_id` as documented in [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:116).
- `backend_id` remains the adapter/allowlist identity only; this slice does not reopen provider/auth/protocol overloading questions already settled in [ADR-0044](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md:147).

## Why This Follow-On Exists

The repo already documents the semantic split:

- external Unified Agent API / Rust `agent_api` is the upstream CLI-agent runtime abstraction in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:55),
- local `crates/agent-api-*` and `world-agent` are the Substrate-local host↔world transport layer in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:61),
- `world-api` is the abstract world backend contract layer,
- and `uaa.agent.session` is already documented as a local normalized label, not an upstream claim, in [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md:184).

But the repo also still leaves the cleanup open:

- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:149) still asks whether `uaa.agent.session` should remain,
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:153) still asks how aggressively to deconflict local `agent-api-*`,
- and several live docs/ADRs still present `uaa.agent.session` as if it were a settled canonical label in [docs/CONFIGURATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md:42), [ADR-0042](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md:127), and [ADR-0044](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md:183).

That drift now costs more than it did earlier, because the codebase contains both:

- real external `agent_api` runtime integration,
- and local transport crates/services still branded as `agent-api-*` and `world-agent`.

## Current Repo Truth

### The upstream UAA boundary is real and already adopted

This repo already uses real upstream UAA semantics:

- external crate import `agent_api` in [crates/shell/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/Cargo.toml:61),
- adopted `agent_api.*` capability/schema ids in [docs/contracts/substrate-gateway-backend-adapter-schema.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-schema.md:21),
- and surfaced internal `uaa_session_id` semantics in [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:214).

Those are not the things being renamed here.

### The local typed host↔world transport is also real and already distinct

The local transport boundary is already explicit in implementation:

- transport/client/schema crates under [Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.toml:55),
- typed transport ownership in [docs/contracts/substrate-gateway-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-runtime-parity.md:21),
- and the `world-agent` socket API boundary in [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:237).

The ambiguity is naming, not architectural absence. The misleading names are specifically:

- `world-agent`, which is the in-world service boundary rather than an AI agent identity,
- and `agent-api-*`, which is a local transport contract rather than the upstream Unified Agent API.

### The local protocol-family label is used in live config, trace, transport, and persistence

`uaa.agent.session` is currently enforced and persisted through several live seams:

- local protocol constant in [mapping.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mapping.rs:3),
- trace identity stamping in [agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:17),
- runtime validator gates in [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:76),
- default inventory examples in [config/agents/codex.yaml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex.yaml:6),
- transport payload propagation in [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:203),
- and persisted participant/session records in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:57), [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:80), and [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2093).

This means the rename must be treated as a real code-and-doc cutover, not just a doc edit.

## In Scope

- freeze one canonical vocabulary for upstream UAA vs local world transport,
- rename the canonical local protocol-family label from `uaa.agent.session` to `substrate.agent.session`,
- rename `world-agent*` to `world-service*`,
- rename `agent-api*` to `transport-api*`,
- keep `world-api` unchanged,
- freeze one exact rename matrix for directory paths, Cargo package names, Rust crate ids, binary names, install aliases, systemd unit names, release-bundle paths, and doctor/help strings,
- define the live-surface grep boundary and the explicit historical allowlist so direct cutover does not devolve into a blind repo-wide rewrite,
- update install/provision/uninstall/release/CI/operator surfaces together with the crate rename,
- update comments/docs/ADRs/tests so boundary wording is consistent,
- and harden regression coverage around the renamed local protocol-family seam.

## Out Of Scope

This slice does not include:

- changing public session/control selectors or caller semantics,
- renaming real upstream UAA capability/schema ids like `agent_api.session.handle.v1`,
- renaming `internal.uaa_session_id` / `uaa_session_id`,
- changing the HTTP path/schema of the local transport service beyond what is required by rename,
- collapsing the local transport crates into upstream UAA,
- renaming true upstream `agent_api` identifiers,
- bulk-rewriting archived planning logs, historical ADR evidence, or generated run artifacts just to purge old tokens,
- or relaxing fail-closed world-required behavior just to make rename drift appear to work.

## Concrete Work Breakdown

### 1. Freeze The Canonical Vocabulary

Primary anchors:

- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:67)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:237)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md:184)
- [docs/contracts/substrate-gateway-backend-adapter-schema.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-schema.md:21)

Required outcome:

- one authoritative repo rule states:
  - `Unified Agent API` / `UAA` / `agent_api` / `agent_api.*` mean upstream CLI-agent runtime semantics only,
  - `world-api` means the abstract world backend contract,
  - `world service` means the Substrate-local in-world daemon/service boundary,
  - `transport-api-*` means the Substrate-local host↔world typed contract layer,
  - `substrate.agent.session` is the Substrate-local pure-agent protocol-family id.
- one explicit rename rule states:
  - local `world-agent*` becomes `world-service*`,
  - local `agent-api*` becomes `transport-api*`,
  - `world-api` remains unchanged,
  - upstream `agent_api` remains unchanged.

### 1A. Freeze The Live Rename Boundary And Historical Allowlist

Primary anchors:

- [Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.toml:49)
- [README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/README.md:157)
- [docs/INSTALLATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/INSTALLATION.md:13)
- [dist/release-template.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/release-template.md:33)
- [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:2041)
- [.github/workflows/feature-smoke.yml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/feature-smoke.yml:204)

Required outcome:

- the SOW explicitly distinguishes live rename scope from historical references that are allowed to keep old names,
- live surfaces include `Cargo.toml`, `dist-workspace.toml`, `crates/**`, `scripts/**`, `.github/**`, `dist/**`, `README.md`, `AGENTS.md`, `config/**`, and operator-facing docs under `docs/**`,
- `macos-hardening/**` and `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/**` are treated as truth-sync surfaces for this slice unless a subpath is explicitly allowlisted as historical with rationale,
- historical references are only exempt where the file is intentionally archival, historical, or evidence-bearing, for example `docs/project_management/_archived/**` and older planning/history files that are not normative runtime/operator guidance,
- repo-wide grep gates run against the live surface set first, then against the explicit historical allowlist to prove any surviving old-name hits are intentionally historical rather than accidental drift,
- and the SOW does not use vague wording like “repo-wide rename” without also naming the historical allowlist and the live-surface command wall that must go green.

### 2. Rename The Local Protocol-Family Label

Primary anchors:

- [agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:17)
- [mapping.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mapping.rs:3)
- [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:76)
- [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:699)
- [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:376)
- [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2466)
- [config/agents/codex.yaml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex.yaml:6)
- [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:203)
- [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:12)
- [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:30)

Required implementation outcome:

- all new config examples, runtime writes, emitted trace identity, and durable persistence use `substrate.agent.session`,
- public-facing validation and docs stop advertising `uaa.agent.session` as the preferred label,
- glossary guardrail: `uaa.agent.session` renames in this slice, while upstream/session-correlation fields such as `uaa_session_id` and `internal.uaa_session_id` remain unchanged,
- supported fixtures, golden traces, test inventories, and checked-in config examples are rewritten in the same cutover so no supported live artifact still emits or expects `uaa.agent.session`,
- this slice assumes a greenfield cutover with no supported continuity path for pre-cutover persisted runtime sessions,
- no migration command, startup rewrite, compatibility alias, or in-place persisted-state upgrade path is added for old `uaa.agent.session` runtime rows,
- `substrate agent doctor`, `substrate agent status`, and REPL/runtime selection paths must fail closed and state plainly that legacy `uaa.agent.session` runtime rows are unsupported after the cutover,
- any surviving inventory config entry using `protocol: uaa.agent.session` must be updated in place to `protocol: substrate.agent.session`,
- and this slice explicitly rejects one-time startup rewrite because the repo does not carry a general state-schema migration layer for safe in-place protocol migration,
- and the repo treats this as a direct-cutover naming cleanup rather than a long-lived compatibility story.

### 3. Rename The Local Transport Families Directly

Primary anchors:

- [crates/shell/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/Cargo.toml:61)
- [crates/world-agent/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml:43)
- [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:12)
- [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:1)

Required implementation outcome:

- `world-agent` crate/service/binary/doc naming becomes `world-service`,
- `agent-api-types`, `agent-api-core`, and `agent-api-client` become `transport-api-types`, `transport-api-core`, and `transport-api-client`,
- Rust imports and dependency keys follow the same rename,
- the exact rename matrix is frozen up front:
  - `crates/world-agent` -> `crates/world-service`
  - Cargo package `world-agent` -> `world-service`
  - binary `world-agent` -> `world-service`
  - installed binary `/usr/local/bin/substrate-world-agent` -> `/usr/local/bin/substrate-world-service`
  - systemd units `substrate-world-agent.service` / `.socket` -> `substrate-world-service.service` / `.socket`
  - helper template [scripts/mac/substrate-world-agent.service](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/substrate-world-agent.service:8) renames to the matching `world-service` filename and `ExecStart`
  - release bundle payloads `bin/world-agent` and `bin/linux/world-agent` rename to `bin/world-service` and `bin/linux/world-service`
  - crate directories `crates/agent-api-types`, `crates/agent-api-core`, and `crates/agent-api-client` rename to `crates/transport-api-types`, `crates/transport-api-core`, and `crates/transport-api-client`
  - Rust crate ids `agent_api_types`, `agent_api_core`, and `agent_api_client` rename to `transport_api_types`, `transport_api_core`, and `transport_api_client`
  - workspace dependency alias keys and consumer Cargo dependency keys `agent-api-types`, `agent-api-core`, and `agent-api-client` rename to the `transport-api-*` family in every `Cargo.toml`
  - embedded file/module path identifiers such as `world_agent` or `repl_world_agent` rename when they remain live support code rather than archived history
  - socket-activation naming renames atomically with the service family:
    - explicit socket/log filenames containing `substrate-world-agent`
    - `FileDescriptorName` / manual `--fdname=substrate-world-agent`
    - discovery probes and remediation strings that currently say `substrate-world-agent`
- `world-api` keeps its current package/crate name and contract meaning,
- comments in mixed-boundary files describe them as transport boundaries, not “Agent API” or “agent” generically,
- no local transport package keeps `agent` in its canonical name unless it is referring to actual upstream CLI-agent semantics,
- and no implementation work in this slice renames upstream `agent_api`.

Because this repo is still greenfield in this seam, this SOW treats direct rename as the cleaner option than alias-and-defer churn.

### 3A. Sweep Installers, Release Bundles, CI, And Platform Helpers

Primary anchors:

- [scripts/linux/world-provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh:527)
- [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:2041)
- [scripts/substrate/dev-install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh:1606)
- [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:686)
- [scripts/windows/wsl-warm.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/wsl-warm.ps1:57)
- [.github/workflows/feature-smoke.yml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/feature-smoke.yml:204)
- [.github/workflows/nightly.yml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/nightly.yml:164)
- [dist/scripts/assemble-release-bundles.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/scripts/assemble-release-bundles.sh:261)
- [dist/release-template.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/release-template.md:33)

Required outcome:

- the direct cutover updates build/test/install/uninstall/provision/warm/smoke/release surfaces in the same slice rather than leaving them as “follow-up docs work,”
- no live installer, helper, or CI surface still builds `world-agent`, stages `bin/linux/world-agent`, enables `substrate-world-agent.service`, or tells operators to inspect `substrate-world-agent.socket`,
- cargo package invocations in CI, smoke, and docs use the renamed `world-service` and `transport-api-*` package names,
- socket-activation fd names, doctor output, helper aliases, and remediation text are renamed consistently with the new service family,
- every install, rewarm, and uninstall path that can run on an upgraded machine removes any legacy `substrate-world-agent.service`, `substrate-world-agent.socket`, and `substrate-world-agent` binary before enabling or probing the new `world-service` family,
- and this section is treated as first-class implementation scope, not as “truth sync” polish after code rename.

### 4. Sweep Boundary Docs, ADRs, And Developer Guidance

Primary anchors:

- [docs/CONFIGURATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md:42)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md:184)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:237)
- [AGENTS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENTS.md:13)
- [README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/README.md:157)
- [docs/INSTALLATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/INSTALLATION.md:13)
- [docs/UNINSTALL.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/UNINSTALL.md:43)
- [docs/COMMANDS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/COMMANDS.md:81)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:169)
- [docs/REPLAY.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/REPLAY.md:82)
- [docs/reference/env/contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/env/contract.md:63)
- [docs/manual_verification/linux_world_socket.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/manual_verification/linux_world_socket.md:12)
- [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md:108)
- [docs/cross-platform/wsl_world_troubleshooting.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/wsl_world_troubleshooting.md:163)
- [ADR-0042](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md:127)
- [ADR-0044](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md:183)
- [ADR-0045](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md:255)
- [ADR-0021](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md:174)

Required outcome:

- no operator-facing or developer-facing doc implies the local transport crates/services are the same thing as upstream UAA,
- no doc presents `uaa.agent.session` as the preferred ongoing local label,
- docs, install guides, uninstall guides, release notes, replay docs, env reference docs, manual verification playbooks, and cross-platform troubleshooting flows all use the same renamed `world-service` and `transport-api-*` vocabulary,
- operator remediation text mentions the new service/socket/binary names consistently,
- and docs stop using `world-agent` / `agent-api-*` as the conceptual names for the local transport boundary, except where historical references are explicitly marked historical.

### 5. Preserve Public Handle And Capability Boundary Rules

Primary anchors:

- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:120)
- [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:214)
- [docs/contracts/substrate-gateway-backend-adapter-schema.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-schema.md:21)

Required outcome:

- this slice must not accidentally widen public selectors,
- must not rename true upstream `agent_api.*` capability/schema ids,
- and must not blur the distinction between public `orchestration_session_id` and internal upstream `uaa_session_id`.

This is a guardrail seam, not a feature seam.

## Required Test Additions Or Tightening

### Config and validator coverage

Primary files:

- [crates/shell/tests/agents_validate.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agents_validate.rs:82)
- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:76)

Required scenarios:

- new canonical `protocol: substrate.agent.session` validates successfully,
- and failure messages / docs no longer tell users that `uaa.agent.session` is the preferred required label.

### Trace and status rename coverage

Primary files:

- [crates/common/src/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:299)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:12)
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:2429)

Required scenarios:

- newly emitted pure-agent trace rows stamp `protocol=substrate.agent.session`,
- and pure-agent status/session grouping does not regress during the rename.

### Transport payload coverage

Primary files:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:2512)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:2346)
- [crates/world-agent/tests/member_runtime_world_placement_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/member_runtime_world_placement_v1.rs:60)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs:93)

Required scenarios:

- typed host↔world transport payloads carry the canonical new local protocol label,
- and upstream schema ids such as `agent_api.session.handle.v1` remain unchanged.

### Renamed transport-family compile/runtime coverage

Primary files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:12)
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:1)

Required scenarios:

- renamed local transport crates compile cleanly across mixed-boundary consumers,
- crates that import both upstream `agent_api` and renamed local transport crates still read clearly at the dependency/import layer,
- and comments/tests in those files explicitly preserve the upstream-vs-local boundary.

### Fail-closed rename drift coverage

Primary files:

- [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:76)
- [socket_activation.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/socket_activation.rs:13)
- [linux.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/linux.rs:897)
- [world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs:200)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:299)

Required scenarios:

- post-cutover world-required flows still fail closed when the renamed service/socket/binary is missing or stale,
- doctor/status paths report the renamed service/socket identifiers consistently,
- old `uaa.agent.session` rows/configs are rejected with explicit error text after the cutover rather than being silently accepted or silently downgraded,
- the explicit error text contract names `uaa.agent.session`, makes clear that pre-cutover runtime rows are unsupported in this greenfield slice, and separately tells operators to update any surviving inventory config entry to `protocol: substrate.agent.session`,
- and no rename drift path silently falls back to host execution when `require_world=true`.

### Provisioning, bundle, and CI coverage

Primary files:

- [scripts/linux/world-provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh:527)
- [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:2073)
- [scripts/substrate/dev-install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh:1606)
- [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:328)
- [scripts/windows/wsl-warm.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/wsl-warm.ps1:79)
- [dist/scripts/assemble-release-bundles.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/scripts/assemble-release-bundles.sh:261)
- [.github/workflows/feature-smoke.yml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/feature-smoke.yml:204)

Required scenarios:

- release bundles stage the renamed `world-service` binaries in both host and guest layouts,
- Linux provision/install/uninstall flows use the renamed service/socket/unit/binary names end to end,
- macOS Lima warm/smoke/doctor flows use the renamed guest binary and unit names,
- WSL warm/smoke/doctor/uninstall flows use the renamed guest binary and unit names,
- validation proves the old unit family is absent after upgrade paths, not merely that the new unit family exists,
- and CI build/test/smoke jobs exercise the renamed package/binary/service surfaces instead of only proving Rust import renames.

### Repo-wide grep guard coverage

Required scenarios:

- live-surface grep checks prove there are no stale `world-agent`, `world_agent`, `substrate-world-agent`, `agent-api-*`, `agent_api_*`, or `uaa.agent.session` hits outside the explicit historical allowlist,
- positive grep checks prove upstream `agent_api` imports and `agent_api.*` schema ids still exist where expected,
- positive grep checks prove `world-api` remains unchanged,
- positive grep checks prove the new canonical names `world-service`, `transport-api-*` / `transport_api_*`, and `substrate.agent.session` exist in their expected writer/consumer surfaces,
- and the SOW records the exact grep commands so contributors can repeat the wall without guessing.

## Acceptance Criteria

- the repo has one explicit naming contract for upstream UAA vs local world transport.
- `substrate.agent.session` is the canonical local pure-agent protocol-family label.
- local canonical names use `service` for the in-world daemon and `transport` for the local contract crates, not `agent` unless the identifier really refers to upstream CLI-agent semantics.
- mixed-boundary crates no longer use ambiguous local `agent-api*` / `world-agent*` names beside upstream `agent_api`.
- the exact rename matrix for directory paths, Cargo package names, Rust crate ids, binaries, install aliases, service units, release bundle paths, and doctor/help strings is implemented without drift.
- the live-surface grep wall passes with zero stale old-name hits outside the explicit historical allowlist.
- the SOW explicitly states that this greenfield slice does not ship a cutover command or migration path for old persisted `uaa.agent.session` runtime rows.
- the SOW and the code both make the same exact distinction:
  - local `world-agent*` and `agent-api*` rename,
  - `world-api` does not,
  - upstream `agent_api` does not.
- operator-facing install/provision/uninstall/release/troubleshooting flows use `world-service` naming consistently across Linux, macOS Lima, and WSL surfaces.
- post-cutover rename drift fails closed where the repo already requires fail-closed behavior, especially for world-required routing and protocol validation.
- no public-handle or upstream capability/schema semantics regress.
- docs/ADRs/tests/gap-matrix wording all align to the same boundary story.

## Validation Expectations

- run targeted tests for validator, status/trace rename behavior, and host↔world transport payloads,
- run targeted tests for socket activation, doctor/status, world-required fail-closed routing, install/provision helpers, and renamed package/binary surfaces,
- run full workspace tests:
  - `cargo test --workspace -- --nocapture`
- run at least one targeted compile/test path through each mixed-boundary consumer family:
  - `shell`
  - `world-service`
  - `world-mac-lima`
  - `world-windows-wsl`
  - `replay`
- run explicit live-surface grep gates after the rename, for example:
  - `rg -n "world-agent|world_agent|substrate-world-agent|agent-api-types|agent-api-core|agent-api-client|agent_api_types|agent_api_core|agent_api_client|uaa\\.agent\\.session" Cargo.toml dist-workspace.toml crates scripts docs .github dist README.md AGENTS.md config macos-hardening`
  - `rg -n "agent_api\\.run|agent_api\\.session\\.resume\\.v1|agent_api\\.session\\.handle\\.v1" crates docs config`
  - `rg -n "world-api" Cargo.toml crates docs`
  - `rg -n "world-service|transport-api-types|transport-api-core|transport-api-client|transport_api_types|transport_api_core|transport_api_client|substrate\\.agent\\.session" Cargo.toml dist-workspace.toml crates scripts docs .github dist README.md AGENTS.md config macos-hardening`
- run platform/operator validation commands against the renamed surfaces:
  - Linux: `systemctl status substrate-world-service.socket`, `systemctl status substrate-world-service.service`, `substrate world doctor --json`, `substrate host doctor --json`
  - Linux negative check: `systemctl list-unit-files | rg "substrate-world-agent"` returns no enabled or installed legacy unit entries after upgrade/install/rewarm
  - macOS Lima: `limactl shell substrate systemctl status substrate-world-service.socket`, `limactl shell substrate systemctl status substrate-world-service.service`, `scripts/mac/smoke.sh`
  - WSL: `pwsh -File scripts/windows/wsl-smoke.ps1`, `pwsh -File scripts/windows/wsl-doctor.ps1`, and uninstall/rewarm verification using the renamed unit and binary names
- run direct fail-closed validation for unsupported legacy runtime rows:
  - seed persisted runtime state with `protocol=uaa.agent.session`,
  - confirm `substrate agent doctor` fails closed with the exact unsupported-legacy-state message,
  - confirm the slice does not depend on any recovery or migration command for those rows,
- run release-bundle validation so `dist/scripts/assemble-release-bundles.sh` and `dist/release-template.md` both describe the renamed `world-service` payloads.

Manual validation for this slice should explicitly prove:

- a canonical new config entry using `protocol: substrate.agent.session`,
- the local in-world daemon is renamed to the `world-service` family all the way through Cargo package, binary, installed alias, bundle path, and systemd unit naming,
- the local host↔world contract crates are renamed to the `transport-api-*` family,
- `world-api` remains unchanged,
- pure-agent trace output now emits `substrate.agent.session`,
- stale old-name live surfaces are gone outside the historical allowlist,
- world-required rename drift still fails closed,
- and true upstream `agent_api.*` capability/session-handle wording is unchanged.

## Docs And Truth Sync

When this slice lands, the following must be updated together:

- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) so the boundary row is no longer “partially clarified,”
- [docs/CONFIGURATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md),
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md),
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md),
- [README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/README.md),
- [docs/INSTALLATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/INSTALLATION.md),
- [docs/UNINSTALL.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/UNINSTALL.md),
- [docs/COMMANDS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/COMMANDS.md),
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md),
- [docs/REPLAY.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/REPLAY.md),
- [docs/reference/env/contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/env/contract.md),
- [docs/manual_verification/linux_world_socket.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/manual_verification/linux_world_socket.md),
- [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md),
- [docs/cross-platform/wsl_world_troubleshooting.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/wsl_world_troubleshooting.md),
- [dist-workspace.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist-workspace.toml),
- [macos-hardening/macos-hardened-same-user-lima/phase-2-same-user-hardening/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-2-same-user-hardening/README.md),
- [AGENTS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENTS.md),
- [ADR-0042](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md),
- [ADR-0044](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md),
- [docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md),
- [dist/release-template.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/release-template.md),
- [scripts/linux/world-provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh),
- [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh),
- [scripts/substrate/dev-install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh),
- [scripts/substrate/uninstall-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/uninstall-substrate.sh),
- [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh),
- [scripts/windows/wsl-warm.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/wsl-warm.ps1),
- [scripts/windows/uninstall-substrate.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/uninstall-substrate.ps1),
- [.github/workflows/feature-smoke.yml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/feature-smoke.yml),
- [.github/workflows/nightly.yml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/nightly.yml),
- and any contract/docs surface still describing the local transport boundary as `world-agent` or `agent-api-*` after the rename.

## Done Shape

This slice is complete when the repo no longer hand-waves any of these points:

- whether `uaa.agent.session` is really the long-term local label,
- whether local `world-agent` / `agent-api-*` names are still allowed to survive as canonical transport names,
- whether upstream `agent_api.*` ids are being confused with Substrate-local transport identity,
- and whether mixed-boundary code is allowed to keep both sides under nearly identical names.

This slice is also not done if any of the following remain vague:

- which exact live surfaces must rename together versus which historical surfaces are intentionally preserved,
- which exact binary/unit/bundle names the direct cutover chooses for `world-service`,
- which exact grep wall proves the old names are gone from live surfaces,
- how fail-closed world-required behavior is preserved when renamed service discovery breaks,
- or which exact operator-readable command recovers persisted legacy `uaa.agent.session` runtime state after the direct cutover.

After this slice, the repo should read as if the boundary was intentional from the start:

- upstream UAA is upstream UAA,
- `world-api` is the abstract world backend contract,
- local `transport-api-*` is the local host↔world contract layer,
- local `world-service` is the in-world daemon/service boundary,
- and the pure-agent local protocol-family label is unmistakably Substrate-local.

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope and strategy | 1 | issues_open | 6 |
| Codex Review | subagent adversarial review | Independent outside challenge | 3 | issues_open | 17 |
| Eng Review | `/plan-eng-review` | Architecture and tests | 1 | issues_open | 6 |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | skipped | No UI scope detected |
| DX Review | manual fallback | Developer and operator rename risk | 1 | issues_open | 5 |

**VERDICT:** APPROVED WITH USER OVERRIDES.

The boundary-cleanup goal is valid.

The original SOW shape was not.

The repo reality is that `world-agent`, `substrate-world-agent`, `agent-api-*`, and `uaa.agent.session` already participate in installed artifacts, CI, systemd units, docs, state validation, trace/status grouping, and contributor workflows. That makes this a migration and supportability slice, not a cheap greenfield naming pass.

User-approved overrides applied on 2026-05-21:

- keep the direct-cutover / no-compat story
- rename `world-agent*` to `world-service*`
- rename `agent-api*` to `transport-api*`
- keep `world-api` unchanged

## Autoplan Review

### Intake Summary

- Plan reviewed: `llm-last-mile/27-uaa-boundary-and-naming-cleanup.md`
- Branch: `testing`
- Base branch: `main`
- UI scope: `no`
- DX scope: `yes`
- Outside voice: unavailable on this machine because `claude` is installed but not authenticated
- Host-subagent review: completed for strategy, engineering, and DX
- Test plan artifact: `/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-testing-eng-review-test-plan-20260521-103306.md`

### Step 0: Premise Challenge + Mode Selection

#### Premise Challenge

Premise 1, "this seam is still greenfield so migration is not a concern," did not survive review.

The repo already bakes `world-agent` and `substrate-world-agent` into live operator and contributor surfaces:

- workspace/package identity in `Cargo.toml`
- systemd units, install, uninstall, and warm scripts
- release bundle assembly and CI
- world routing discovery and fallback
- manual verification and troubleshooting docs

Premise 2, the original "`world-agent -> world-transport` and `agent-api-* -> transport-api-*` is the cleanest naming truth," did not survive review unchanged.

`world-agent` is not a dumb pipe. It owns execute, cancel, gateway, member-turn, pending-diff, and world-fs behavior, and the repo already treats it as an in-world execution daemon and control surface. The approved rename target therefore became `world-service`, while `transport-api-*` remained approved for the contract crates and `world-api` stayed unchanged.

Premise 3, "`uaa.agent.session` can be direct-cut over with no real compatibility story," failed.

That label is exact-matched in validator paths, config examples, trace/status grouping, transport payloads, and persisted runtime/session state. This is a live state migration problem, not a doc-only rename.

#### Existing Code Leverage

| Sub-problem | What already exists | Review outcome |
|---|---|---|
| Boundary wording | `AGENT_ORCHESTRATION_GAP_MATRIX.md`, `docs/WORLD.md`, `docs/TRACE.md` already explain upstream `agent_api` vs local transport | Reuse as vocabulary anchors |
| Pure-agent protocol label | `crates/common/src/agent_events.rs`, `crates/shell/src/execution/agent_runtime/mapping.rs`, `validator.rs`, `agents_cmd.rs` centralize the current label | Good migration seam, but requires dual-read planning |
| Local host/world contract | `crates/agent-api-types`, `crates/agent-api-core`, `crates/agent-api-client`, `world_ops.rs`, `member_runtime.rs` already define the typed contract | Reuse, but current "transport" rename target is semantically under-justified |
| Operator/runtime artifact inventory | install scripts, Lima/WSL warm scripts, service units, release bundle scripts, CI, top-level docs | Must become first-class rename inventory, not implied cleanup |

#### Dream State Mapping

```text
CURRENT STATE
  upstream `agent_api` is real,
  local host/world seams are real,
  but naming is mixed and operator artifacts still say `world-agent` / `agent-api-*`
        |
        v
THIS PLAN (AS WRITTEN)
  direct-cut protocol rename +
  direct rename of crate/package/binary/service/doc families +
  no compatibility story
        |
        v
12-MONTH IDEAL
  one explicit boundary vocabulary,
  no semantic confusion,
  no operator artifact churn surprises,
  and world execution still fails closed instead of degrading quietly
```

The current SOW moves toward vocabulary clarity, but away from safe rollout, supportability, and truthful ownership naming.

#### Implementation Alternatives

**Approach A: Full direct cutover**

- Summary: rename `uaa.agent.session`, `world-agent*`, and `agent-api*` everywhere in one slice with no compatibility names.
- Effort: XL
- Risk: High
- Pros:
  - clean end state on paper
  - no temporary aliases
- Cons:
  - collides with installed binary/unit names, CI, and bundle paths
  - turns protocol rename into a mixed-version state break
  - assumes `world-agent` and `transport` mean the same thing
- Reuses:
  - existing central constants and docs anchors only

**Approach B: Migration-safe boundary cleanup**

- Summary: freeze vocabulary now, add dual-read protocol migration, keep operator/runtime artifact names stable for a release window, and rename internal package/import surfaces only where the responsibility name is defensible.
- Effort: L
- Risk: Medium
- Pros:
  - preserves world-required safety and install ergonomics
  - handles persisted state honestly
  - gives docs and support copy a clean migration story
- Cons:
  - temporary mixed naming
  - requires explicit deprecation bookkeeping
- Reuses:
  - validator, state store, trace/status centralization, existing install/test harnesses

**Approach C: Responsibility-first cleanup**

- Summary: rename only the protocol label and docs now, keep `world-agent` until the repo explicitly decides whether that component is transport, runtime service, or control plane, and revisit `agent-api-*` only after that boundary is named truthfully.
- Effort: M
- Risk: Low
- Pros:
  - fixes the most misleading upstream-vs-local collision first
  - avoids baking in another possibly-wrong noun
  - smaller blast radius
- Cons:
  - local crate/service naming confusion remains for one more slice
  - less satisfying if the goal is one-shot cleanup
- Reuses:
  - current docs and code ownership split

**RECOMMENDATION AT REVIEW TIME:** Choose **Approach B** for the protocol and operator-surface problem, and do **not** auto-approve the original `world-agent -> world-transport` rename until the world-side service responsibility is named more truthfully.

### Mode Selection

Selected mode: `HOLD SCOPE`

Reason: the plan already covers a large cross-repo sweep. The review found missing truth and missing migration planning, not an opportunity to expand product scope.

### CEO Dual Voices

#### Outside Voice

Unavailable on this machine. `claude` is present, but auth is missing.

#### Host Subagent

The independent strategy pass raised six issues, led by:

- this is not actually greenfield
- naming cleanup is not obviously the highest-value remaining v1 work
- `uaa.agent.session` is a live migration seam, not a cheap rename
- the original `world-agent -> world-transport` target may be semantically wrong, not just risky

#### CEO Consensus Table

| Dimension | Subagent | Outside | Consensus |
|---|---|---|---|
| Premises valid? | No | N/A | N/A |
| Right problem to solve? | Partially | N/A | N/A |
| Scope calibration correct? | No | N/A | N/A |
| Alternatives sufficiently explored? | No | N/A | N/A |
| Competitive / product risk covered? | No | N/A | N/A |
| 6-month trajectory sound? | No | N/A | N/A |

### Review Findings

#### Section 1: Architecture Review

The current repo shape is:

```text
upstream `agent_api`
   |
   +--> crates/shell
   |      |
   |      +--> local `agent-api-client` / `agent-api-types`
   |      |        |
   |      |        +--> `world-agent` execute/stream/member-turn contract
   |      |
   |      +--> world backend selection / socket activation / fallback
   |
   +--> crates/world-agent
          |
          +--> execute + cancel + member runtime + gateway runtime
          +--> systemd units / installer / release artifacts
```

That is not a purely internal noun swap. The current SOW needs an explicit naming matrix for:

- workspace member directory
- Cargo package name
- Rust crate import name
- binary name
- installed alias
- systemd unit name
- release artifact path
- CI command surface
- user-facing docs and error strings

Without that matrix, implementation will drift.

#### Section 2: Error & Rescue Map

| Method / Codepath | What can go wrong | Exception / failure class | Rescued? | Rescue action | User sees |
|---|---|---|---|---|---|
| world daemon discovery | renamed unit/binary no longer found | startup / lookup failure | No | add deterministic alias or migration guidance | broken world routing or misleading fallback |
| world-required routing | renamed surface causes host fallback | isolation regression | No | fail closed when world is required | silent downgrade today is unacceptable |
| validator selection | legacy `uaa.agent.session` persisted/configured rows rejected | exact-match validation failure | No | dual-read migration window | existing sessions/configs stop validating |
| release / install staging | artifact path rename missed in bundles/scripts | missing binary during install/warm | No | bundle + install smoke coverage | operator remediation churn |

#### Section 3: Security & Threat Model

The highest-severity threat is not data exfiltration. It is isolation posture drift.

If `substrate-world-agent` discovery breaks and world-required flows fall back to host or local execution instead of failing closed, the rename would weaken the security model while claiming to be a cleanup. That is a real operations/security regression and must be blocked explicitly.

#### Section 4: Data Flow & Interaction Edge Cases

```text
NAME CHANGE
  -> Cargo/workspace identity
  -> binary + service discovery
  -> installer / warm / uninstall scripts
  -> config examples
  -> validator exact-match checks
  -> persisted state / trace / status grouping
  -> docs / support guidance
```

Edge cases currently missing from the SOW:

- mixed-version state where old persisted rows still say `uaa.agent.session`
- stale operator instructions pointing to `substrate-world-agent.service`
- CI steps still using `cargo test -p world-agent`
- staged artifact bundles still emitting `world-agent`
- WSL/Lima warm scripts installing or enabling old unit names

#### Section 5: Code Quality Review

The naming problem is real. The chosen target names are under-justified.

`world-agent` today is a runtime/control daemon, not a pure transport pipe. `agent-api-types` also carries service-contract, policy, and lifecycle schemas beyond transport. The approved `world-service` + `transport-api-*` split is materially better than the original `world-transport` target, because it leaves `world-api` as the abstract backend contract and reserves `transport` for the local contract crates.

#### Section 6: Test Review

See the generated test plan artifact:

- `/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-testing-eng-review-test-plan-20260521-103306.md`

Coverage summary:

- 0/16 critical rename paths are explicitly covered by the current SOW
- the biggest gaps are upgrade, bundle/install staging, service discovery, and operator messaging

Test diagram:

```text
NEW CODEPATHS / FLOWS
  1. protocol label read/write behavior
  2. package/import rename behavior
  3. binary/unit discovery behavior
  4. release bundle assembly behavior
  5. installer + warm + uninstall behavior
  6. docs/help/remediation behavior

CURRENT PLAN COVERAGE
  - targeted validator / trace / transport payload tests
  - workspace tests

MISSING
  - upgrade / mixed-version tests
  - install smoke + world provision smoke
  - bundle assembly checks
  - CI workflow checks
  - fail-closed rename drift checks
```

#### Section 7: Performance Review

No performance bottleneck is driving this slice.

The operational cost is contributor time and support churn. That is the dominant cost center here, not runtime latency.

#### Section 8: Observability & Debuggability Review

The SOW should add a first-class operator messaging and observability pass covering:

- doctor output
- fatal/warn remediation strings
- CI failure text
- temporary old-name to new-name mapping guidance

Otherwise debugging rename drift will be miserable.

#### Section 9: Deployment & Rollout Review

The current SOW has no honest rollout story.

It needs one of these:

- keep operator/runtime artifact names stable temporarily
- or require a single atomic cutover with upgrade, downgrade, uninstall, doctor, smoke, Lima, and WSL validation

Right now it promises neither.

#### Section 10: Long-Term Trajectory Review

The long-term win is a truthful boundary vocabulary.

The long-term naming risk is lower after the approved switch from `world-transport` to `world-service`, but the repo still needs to keep `world-service`, `transport-api-*`, and `world-api` semantically distinct in docs and mixed-boundary code.

### Design Review

Skipped. No UI scope detected.

### Manual DX Review

Overall DX score: `4/10`

| Dimension | Score | Notes |
|---|---|---|
| Getting started | 4 | top-level docs and install flows currently point at old names |
| Naming guessability | 5 | upstream/local split improves, but proposed local names are not fully explained |
| Error messages | 3 | SOW barely covers support copy and remediation text |
| Docs findability | 4 | sync list misses README, INSTALLATION, COMMANDS, UNINSTALL, release template |
| Upgrade path | 2 | "greenfield" assumption contradicts live repo surface |
| Dev environment friction | 4 | CI, bundle, and dev-install path changes are under-scoped |

### Cross-Phase Themes

- **Theme: not actually greenfield**. Flagged independently in strategy, engineering, and DX review.
- **Theme: `uaa.agent.session` needs migration rules**. Flagged independently in strategy and engineering review.
- **Theme: the original `world-agent -> world-transport` target was not obviously the right noun**. Flagged independently in strategy and engineering review. User approved `world-service` instead.
- **Theme: operator/runtime artifacts are first-class scope**. Flagged independently in engineering and DX review.

## NOT In Scope

- Renaming true upstream `agent_api` imports or `agent_api.*` capability/schema ids.
- Renaming generic `SUBSTRATE_*AGENT*` env vars just because they contain the word `agent`.
- Sweeping archived logs and historical evidence packs where churn adds noise without protecting live behavior.
- Reworking public HTTP path schemas unless a rename is explicitly approved for those surfaces.

## What Already Exists

- Boundary clarification prose already exists in `AGENT_ORCHESTRATION_GAP_MATRIX.md`, `docs/WORLD.md`, and `docs/TRACE.md`.
- The pure-agent protocol label is centralized enough to cut over safely if supported configs, fixtures, traces, and persisted examples are rewritten in-place during the rename.
- The local host/world typed contract is already well-defined in the `agent-api-*` crates and their consumers.
- Installer, warm, uninstall, doctor, and CI harnesses already exist and can be used as the real validation wall for any approved rename.

## Dream State Delta

The desired end state is still good: one clear upstream-vs-local boundary story.

The delta is that the repo needs to get there without:

- silent world-to-host fallback,
- broken bundle/install flows,
- rejected legacy persisted protocol rows,
- or a new misleading noun for the world-side execution daemon.

## Failure Modes Registry

| Codepath | Failure mode | Rescued? | Test? | User sees? | Logged? |
|---|---|---|---|---|---|
| systemd / binary discovery | renamed unit or binary not found | N | N | misleading doctor/install failures or fallback | partial |
| world-required execution | rename drift falls back to host/local | N | N | isolation regression | partial |
| validator / config selection | old protocol rows rejected | N | N | existing config/session stops validating | yes |
| bundle / installer staging | artifact path mismatch | N | N | missing binary during install/warm | yes |
| docs / support guidance | stale remediation names | N | N | operator confusion | inconsistent |

Any row above is a critical gap until the SOW adds explicit coverage.

## Deferred To TODOS.md

Not written yet.

The review found follow-up work, but this document now carries the rename matrix and validation wall directly, so emitting ad hoc TODOs before implementation starts would create noise.

## Completion Summary

```text
+====================================================================+
|            MEGA PLAN REVIEW — COMPLETION SUMMARY                   |
+====================================================================+
| Mode selected        | HOLD SCOPE                                  |
| System Audit         | Naming confusion is real; greenfield premise |
|                      | is false; operator/runtime blast radius huge |
| Step 0               | Current SOW challenged                      |
| Section 1  (Arch)    | 4 issues found                              |
| Section 2  (Errors)  | 4 error paths mapped, 4 gaps                |
| Section 3  (Security)| 1 high-severity isolation regression risk   |
| Section 4  (Data/UX) | 5 edge cases mapped, 5 unhandled            |
| Section 5  (Quality) | 2 issues found                              |
| Section 6  (Tests)   | Diagram produced, major gaps                |
| Section 7  (Perf)    | 0 perf blockers, high support cost          |
| Section 8  (Observ)  | messaging / doctor gaps found               |
| Section 9  (Deploy)  | rollout story missing                       |
| Section 10 (Future)  | Reversibility: 2/5, noun risk high          |
| Section 11 (Design)  | SKIPPED (no UI scope)                       |
+--------------------------------------------------------------------+
| NOT in scope         | written (4 items)                           |
| What already exists  | written                                     |
| Dream state delta    | written                                     |
| Error/rescue registry| 4 methods, 4 critical gaps                 |
| Failure modes        | 5 total, 5 critical gaps                   |
| TODOS.md updates     | deferred pending approval gate              |
| CEO plan             | skipped                                     |
| Outside voice        | attempted via user-specified Claude CLI, blocked by 401 auth failure |
| Diagrams produced    | architecture, flow, test coverage           |
| Unresolved decisions | 0, user responses applied below             |
+====================================================================+
```

## User Challenge Resolutions

User responses applied on 2026-05-21:

### Resolution 1: Direct cutover stays approved

- Approved direction: keep the direct-cutover / no-compat story.
- Consequence: the implementation must hard-gate the cutover with an exhaustive live-surface grep wall, installer/bundle/CI validation, and fail-closed checks instead of relying on compatibility aliases.

### Resolution 2: `world-agent*` renames to `world-service*`

- Approved direction: `world-agent* -> world-service*`
- Consequence: the plan no longer uses `world-transport` as the daemon target name.
- Guardrail: `world-api` stays unchanged and continues to mean the abstract world backend contract.

### Resolution 3: `agent-api-*` renames to `transport-api-*`

- Approved direction: `agent-api-types`, `agent-api-core`, and `agent-api-client` rename to the `transport-api-*` family.
- Consequence: mixed-boundary code should read as:
  - upstream `agent_api`
  - local `transport-api-*`
  - local `world-service`
  - unchanged `world-api`

### Resolution 4: No cutover command for old sessions

- Approved direction: this slice remains greenfield for pre-cutover session continuity and does not add a cutover or recovery command for old persisted runtime sessions.
- Consequence: legacy `uaa.agent.session` runtime rows may fail closed as unsupported after the cutover.
- Guardrail: inventory/config examples and live supported runtime surfaces still move to `protocol: substrate.agent.session` in the same slice.

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | Intake | Skip design review | Mechanical | Explicit over clever | No UI scope detected in the SOW | Running design review anyway |
| 2 | Intake | Run manual DX review fallback | Mechanical | Bias toward action | DX scope exists, but the dedicated DX skill file is unavailable in this session | Skipping DX entirely |
| 3 | Intake | Treat outside voice as unavailable | Mechanical | Bias toward action | `claude` auth missing on 2026-05-21, so only host subagents could run | Blocking the whole review |
| 4 | CEO | Use HOLD SCOPE mode | Mechanical | Pragmatic | The plan is already broad; the problem is truth and rollout, not missing expansion | Expansion mode |
| 5 | CEO | Challenge the greenfield premise | User Challenge | Choose completeness | Superseded by later user resolution: repo-wide rename blast radius is real, but this slice is still approved as greenfield for pre-cutover session continuity | Accepting "no migration concern" |
| 6 | CEO | Reject original `world-agent -> world-transport` target | User Challenge | Explicit over clever | Current service responsibility reads as runtime/control plane, not pure transport | Auto-accepting `world-transport` |
| 7 | Eng | Expand validation to installer/bundle/CI/service surfaces | Mechanical | Choose completeness | Compile-only validation misses the real blast radius | Crate-only validation |
| 8 | Eng | Require a protocol migration story | User Challenge | Choose completeness | Exact-match validators and persisted state make direct cutover unsafe | No-compat protocol cutover |
| 9 | User | Keep direct-cutover / no-compat story | User Override | User sovereignty | User explicitly approved hard cutover gated by exhaustive checks | Compatibility alias window |
| 10 | User | Rename `world-agent*` to `world-service*` | User Override | User sovereignty | User explicitly selected `world-service` instead of `world-transport` | Keeping `world-agent` or using `world-transport` |
| 11 | User | Rename `agent-api-*` to `transport-api-*` and keep `world-api` | User Override | User sovereignty | User explicitly approved the contract-layer rename and keeping `world-api` stable | Deferring the rename or repurposing `world-api` |
| 12 | User | Do not add a cutover command for old persisted runtime sessions | User Override | User sovereignty | User explicitly clarified this slice is greenfield and should not ship a recovery command for old sessions | Adding a migration or recovery command |

## Hardening Pass 2

Date: 2026-05-21

This pass treats the approved decisions as fixed baseline:

- direct cutover stays approved,
- `world-agent*` renames to `world-service*`,
- `agent-api*` renames to `transport-api*`,
- `world-api` stays unchanged,
- upstream `agent_api` stays unchanged,
- `uaa.agent.session` renames to `substrate.agent.session`.

### Outside Voice Attempt

- Attempted the exact user-specified binary: `/Users/spensermcconnell/.local/bin/claude`
- Result on 2026-05-21: `401 Invalid authentication credentials`
- Consequence: no fresh external Claude findings were available for this pass. The hardening delta below is grounded in repo evidence only.

### Hardening Findings

1. The SOW needed an explicit live-surface versus historical-surface rename boundary.

   The current repo has `world-agent` in at least `666` files, `substrate-world-agent` in `72`, and `uaa.agent.session` in `41`. A blind repo-wide replacement would trample archived plans, historical ADRs, and evidence logs that should remain historical. The execution-safe version is a live-surface grep wall plus an explicit historical allowlist.

2. The SOW needed an exact rename matrix, not just rename families.

   The repo hard-codes old names in workspace membership [Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.toml:49), socket-activation constants [socket_activation.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/socket_activation.rs:13), Linux provision/install scripts [world-provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh:527) and [install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:2073), macOS Lima warm paths [lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:686), WSL warm paths [wsl-warm.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/wsl-warm.ps1:79), release bundles [assemble-release-bundles.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/scripts/assemble-release-bundles.sh:261), and release notes [release-template.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist/release-template.md:33).

3. The direct-cutover validation wall was too vague for runtime and operator safety.

   `validate_member_selection(...)` feeds both world-member selection and doctor/status reporting according to GitNexus, via [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) and [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs). The protocol rename therefore affects runtime eligibility and operator diagnostics simultaneously. The SOW now requires explicit fail-closed tests and grep gates instead of assuming the crate rename proves runtime safety.

4. The docs/operator sync list was materially incomplete.

   Before this pass, the SOW called out core boundary docs but not README/install/uninstall/release/env/manual-verification/troubleshooting surfaces that currently direct operators to `substrate-world-agent` and `world-agent` names. Those are now first-class scope.

### Round-2 Verdict

No new concrete blocker was found against the approved direct-cutover shape.

The remaining risk was execution vagueness, not decision invalidity. With the rename matrix, live-versus-historical boundary, operator/packaging scope, grep wall, and fail-closed validation now made explicit in this SOW, the plan is materially harder to execute incorrectly while staying inside the approved naming decisions.

## Hardening Pass 3

Date: 2026-05-21

This pass kept the approved baseline fixed:

- direct cutover / no compatibility story stays approved,
- `world-agent*` still renames to `world-service*`,
- `agent-api*` still renames to `transport-api*`,
- `world-api` stays unchanged,
- upstream `agent_api` stays unchanged,
- `uaa.agent.session` still renames to `substrate.agent.session`.

### Outside Voice Attempt

- Attempted the exact user-specified binary: `/Users/spensermcconnell/.local/bin/claude`
- Completed successfully on `2026-05-21 11:46:21 EDT`
- Result: no new blocker against the approved rename shape
- Strongest outside-voice concern: the SOW still under-specifies upgrade cleanup, persisted-state recovery, socket-activation naming, and a few live rename surfaces

### Hardening Findings

1. The live-surface grep wall still misses a real release-packaging root file.

   [dist-workspace.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist-workspace.toml:1) still names `world-agent` in the root `packages` list, but the current negative grep wall only names `dist/`, not the root `dist-workspace.toml` file. That leaves a direct-cutover packaging hole where release automation can still stage the old package family after the implementation appears green elsewhere.

   The SOW should explicitly add `dist-workspace.toml` to:

   - the live-surface rename set,
   - the truth-sync set,
   - and the exact post-rename grep wall.

2. The rename matrix is still crate-producer heavy and consumer-light.

   The current matrix freezes the renamed producer crates, but it still leaves too much inference for consumer surfaces that must move in the same patch:

   - [crates/host-proxy/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/host-proxy/Cargo.toml:31) still depends on `agent-api-client`, `agent-api-core`, and `agent-api-types`.
   - [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:4) imports `agent_api_client` and `agent_api_types` while its module/docs still describe the guest daemon as `world-agent`.
   - [crates/world-windows-wsl/src/backend.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs:3) imports `agent_api_client` / `agent_api_types` and still documents delegation to `world-agent` inside WSL.
   - [Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.toml:53) still carries the workspace-member paths, so the SOW should call out workspace-member path rename and workspace dependency alias-key rename as separate matrix rows instead of assuming they are implied by package/crate-id rename.
   - [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs:1) embeds the old daemon name in an actual support-module file path, and [crates/shell/tests/support/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/mod.rs:17) imports that module by name.

   The SOW should add a first-class matrix row for:

   - consumer Cargo dependency keys,
   - workspace dependency alias keys,
   - and file/module paths that embed `world_agent` outside crate-root directories.

3. The direct-cutover protocol story still lacks an operator-readable recovery path for persisted state.

   The repo evidence is not just validator-side. The old protocol token is baked into persisted-session fixtures and state-store helpers:

   - [mapping.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mapping.rs:3) still defines `PURE_AGENT_PROTOCOL` as `uaa.agent.session`.
   - [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:66) rejects members that do not advertise that exact protocol.
   - [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2466), [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2581), and [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:4341) still generate persisted participant/session rows with `protocol: "uaa.agent.session"`.

   GitNexus also confirms the blast radius is real, not hypothetical: `validate_member_selection` is `HIGH` risk, with direct upstream callers in REPL flow and doctor/reporting paths, including [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) and [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs).

   This review originally argued for a more explicit recovery surface. That recommendation is now superseded by the approved greenfield posture recorded below: no cutover command is added for old sessions, unsupported legacy rows fail closed with explicit messaging instead, and the earlier “not actually greenfield” concern should now be read only as rename-blast-radius caution rather than as a session-continuity requirement.

4. Upgrade cleanup still does not explicitly require removal of legacy unit names before enabling the new family.

   The current scope covers rename of the unit names, but it still does not say the installer/uninstaller must actively remove the legacy unit family first. That matters because this repo already manages old-name units directly in install and uninstall flows:

   - [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:2161) enables `substrate-world-agent.service` and `.socket`.
   - [scripts/substrate/uninstall-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/uninstall-substrate.sh:431) stops, disables, and removes the old unit family.
   - [scripts/substrate/dev-uninstall-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-uninstall-substrate.sh:613) does the same for dev/Lima flows.
   - [scripts/windows/uninstall-substrate.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/uninstall-substrate.ps1:140) disables the old WSL unit family explicitly.

   The SOW should add an exact upgrade rule:

   - before enabling `substrate-world-service.service` / `.socket`, every install or warm path that can run on an upgraded machine must disable and remove `substrate-world-agent.service` / `.socket` if present,
   - and validation must prove only the new unit family remains enabled after install, rewarm, and uninstall/rewarm cycles.

5. The socket-activation and discovery rename is still under-specified.

   The current SOW captures service and socket unit names, but the repo also hard-codes adjacent naming surfaces that must move atomically:

   - [socket_activation.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/socket_activation.rs:11) pins the socket path plus `SOCKET_UNIT` and `SERVICE_UNIT`.
   - [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:612) emits old-name remediation text and [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:639) still probes `which::which(\"substrate-world-agent\")`.
   - [plan.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/plan.rs:758) and [plan.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/plan.rs:815) still print `substrate-world-agent.socket` in shim-status/doctor summaries.
   - [.github/workflows/feature-smoke.yml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/feature-smoke.yml:242) still creates `substrate-world-agent.sock`, `substrate-world-agent.log`, and uses `--fdname=substrate-world-agent` for manual socket-activation fallback.
   - [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:105) still recognizes `substrate-world-agent` in service-name detection logic.

   The SOW should therefore freeze an exact socket-activation sub-matrix:

   - socket path,
   - systemd socket unit name,
   - systemd service unit name,
   - `FileDescriptorName` / manual `--fdname`,
   - fallback socket/log filenames used by CI,
   - discovery binary names and remediation strings.

6. The historical allowlist boundary is still too judgmental in two concrete path families.

   The current text names `_archived/**`, but the repo still contains non-archived path families with old names that are neither clearly live nor clearly exempt:

   - [macos-hardening/macos-hardened-same-user-lima/phase-2-same-user-hardening/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-2-same-user-hardening/README.md:43) still treats `world-agent` and `substrate-world-agent` as current contract terms.
   - [docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md:69) and adjacent draft-pack specs still describe `protocol=uaa.agent.session` as the canonical pure-agent label.

   Those are not under `docs/project_management/_archived/**`, and they are not currently called out in the SOW's truth-sync set or allowlist. That leaves exact room for implementation drift: one contributor can treat them as live normative surfaces, another can treat them as historical planning evidence, and both can claim the SOW supports that choice.

   The SOW should resolve that ambiguity explicitly by path:

   - either add `macos-hardening/**` and selected `docs/project_management/packs/draft/**` paths to the live rename wall,
   - or add them to an explicit historical allowlist with rationale.

### Round-3 Verdict

No new blocker was found against the approved direct-cutover / `world-service` / `transport-api-*` shape.

The remaining risk is still execution drift, not baseline decision failure.

The highest-severity remaining gap is the persisted-state/operator-recovery story for the protocol cutover. That does not require backing away from direct cutover, but it does require the SOW to pin one exact recovery path instead of relying on generic "fail closed with explicit error text" language.

## Hardening Pass 4

Date: 2026-05-21

This pass keeps the approved baseline fixed:

- direct cutover / no compatibility window stays approved,
- `world-agent*` still renames to `world-service*`,
- `agent-api*` still renames to `transport-api*`,
- `world-api` stays unchanged,
- upstream `agent_api` stays unchanged,
- `uaa.agent.session` still renames to `substrate.agent.session`.

### Outside Voice Attempt

- Attempted the exact user-specified binary: `/Users/spensermcconnell/.local/bin/claude`
- Completed successfully on `2026-05-21 12:01:44 EDT`
- Outcome: no new blocker against the approved rename shape
- Strongest outside-voice concern: the SOW still needed one exact persisted-state recovery path, plus tighter rename gates for `dist-workspace.toml`, `world_agent` module/file-path spellings, socket-activation fd names, and live-vs-historical path classification

### Hardening Findings

1. The protocol-cutover recovery story was still too vague to execute safely.

   Repo evidence shows the old token is not just a validator constant. It is stamped into persisted runtime state and test-backed on-disk shapes in [mapping.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mapping.rs:3), [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:699), [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:376), and [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2466). The validator remains exact-string based in [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:76), and GitNexus confirms `validate_member_selection` is `HIGH` risk with direct upstream callers in both REPL and doctor/reporting flow through [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:12) and [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:30).

   This pass hardens the missing operator story by freezing the supported behavior: fail closed on unsupported legacy runtime rows, fix the wording in doctor/status messaging, and do not add an implicit startup rewrite.

2. The rename wall still needed a couple of concrete drift catches.

   The live-surface negative grep wall previously missed [dist-workspace.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/dist-workspace.toml:15), and it did not search underscore spellings such as [repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs:1) or [mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/mod.rs:17). Those are real contributor/runtime drift surfaces, not cosmetic leftovers.

3. Socket activation and upgrade cleanup needed to be more explicit than “rename the units.”

   The old family is hard-coded not just in unit names but in socket-activation constants and fallback filenames: [socket_activation.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/socket_activation.rs:13), [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:612), [plan.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/plan.rs:758), and [.github/workflows/feature-smoke.yml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.github/workflows/feature-smoke.yml:242). This pass therefore locks the fd-name, fallback filename, and legacy-unit-removal requirements into the matrix and validation wall.

4. The live-vs-historical boundary needed explicit path decisions, not just category language.

   The earlier pass correctly found ambiguity around [macos-hardening/macos-hardened-same-user-lima/phase-2-same-user-hardening/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-2-same-user-hardening/README.md:43) and [docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md:69). This pass resolves that by treating those path families as truth-sync surfaces unless they are explicitly allowlisted as historical.

### Round-4 Verdict

No new blocker was found against the approved direct-cutover / `world-service` / `transport-api-*` shape.

The main remaining execution risk was ambiguity around persisted `uaa.agent.session` runtime state. This pass removes that ambiguity by freezing one direct-cutover-safe answer: unsupported legacy rows fail closed in this greenfield slice, with fixed doctor/remediation guidance and no compatibility shims, startup rewrite, or cutover command.
