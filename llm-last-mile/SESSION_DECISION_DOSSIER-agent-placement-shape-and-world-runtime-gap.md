# Session Decision Dossier

- Project: Substrate
- Repo / Workspace: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- Date: 2026-06-16
- Authoring Session: Codex session on 2026-06-16 covering agent placement shape, world-scoped Codex runtime realizability, and installer/provisioning sequencing
- Branch / Commit: `feat/internal-host-orchestrator-world-dispatch-bootstrap` @ `4aa0cb8d1`
- Status: in progress
- Primary Topic: sequential landing plan for placement-aware agent inventory plus world-scoped Codex guest-runtime delivery

## Executive Summary

- Confirmed: the current `cli:codex_world` blocker is **not** world-binding persistence. World-backed start already persists authoritative `world_id` / `world_generation`; the active gap is guest runtime/bootstrap realizability for world-scoped Codex.
- Confirmed: current runtime realizability still treats host `which codex` success as sufficient, then passes the host-resolved absolute path into member bootstrap. That is why world member launch can fail late with exit `127`.
- Confirmed: current world-deps contract already supports **runtime-applied script packages** via `substrate world deps current sync`; there is no live repo authority saying a world refresh/restart is required after sync. Only system-package mutation is gated behind `substrate world enable --provision-deps`.
- Confirmed: current "what world agents exist" truth is split — inventory files define concrete host/world agent entries, policy allowlists govern whether exact backends may be used, and merged `substrate config show` output does not enumerate inventory-backed world agents by itself.
- Decision: the preferred runtime-delivery approach is **Option A** — use the existing world-deps `install.method: script` contract to install a guest-visible Codex runtime under `/var/lib/substrate/world-deps/...` and expose `codex` via `/var/lib/substrate/world-deps/bin/codex`.
- Decision: the implementation sequence should be **runtime truth before config-shape cutover**: first land fail-closed world-runtime validation/remediation, then land the Codex guest runtime package/bundle, then land the placement-aware inventory/selector migration.
- Decision: installer surfaces must gain an explicit flag to provision this runtime at install time, and that flag must be wired into both the **production installer** and the **dev installer**.
- Confirmed: Slice `58` spec/plan/tasks now exist for the placement-aware config/selector redesign, but they intentionally defer the actual world-runtime fix.

## Objective / Problem Statement

### Objective
- Freeze the durable landing plan for the world-scoped Codex runtime gap and package it so a fresh agent can implement the work in the correct order without re-litigating the architecture.

### In Scope
- The real current blocker for `cli:codex_world`.
- The current world-deps operator/runtime contract and whether sync versus refresh/restart is required.
- The preferred guest-runtime delivery model for Codex in worlds.
- The sequencing relationship between the runtime slice and the placement-aware inventory slice.
- The requirement to add installer-time provisioning flags to both dev and prod installer flows.

### Out of Scope
- Implementing the new runtime validation/remediation code.
- Implementing the Codex world-deps package/bundle.
- Implementing the placement-aware schema.
- Reopening the `SPEC-30` host-vs-world root-start contract.
- Designing a new first-class binary artifact transport/install system unless the preferred world-deps script approach proves insufficient.

### Constraints
- Exact backend identity remains a core selector contract.
- Host and world must not silently cross-match.
- World-scoped runtimes need a **guest-visible, guest-executable** binary/runtime contract, not just host-side `which` success.
- Current backend-id validation accepts `<kind>:<name>` with a single colon; names cannot contain `:`.
- Current world-deps runtime contract expects runnable tools to resolve from `/var/lib/substrate/world-deps/bin`.
- Current world-deps runtime contract is probe-only for APT/pacman items; `substrate world enable --provision-deps` is the only operator-facing system-package mutation path on supported guest backends.

## Current State

- Confirmed: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/CODEX_WORLD_DISPATCH_GAP_WRITEUP.md` records that the active gap is guest runtime realizability for world-scoped Codex, not missing world binding.
- Confirmed: current world-scoped Codex inventory is still split into a duplicate file (`/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex_world.yaml`) with `binary: codex`, `runtime_family: codex`, and `execution.scope: world`.
- Confirmed: current merged config output can show default execution posture such as `agents.defaults.execution.scope: world`, but it does **not** by itself enumerate which concrete inventory-backed world agents exist because agent inventory files are a separate source of truth.
- Confirmed: current runtime realizability path still host-resolves `binary: codex` with `which::which(...)`, copies that absolute path into the member dispatch payload, and only checks “file exists” before bootstrap. There is no guest-side interpreter/runtime closure check today.
- Confirmed: the observed failing path still uses the host-local NVM Codex path and dies with exit `127` before authoritative member registration.
- Confirmed: current world PATH normalization is intentionally guest-stable and includes `/var/lib/substrate/world-deps/bin`, not the host user’s NVM PATH.
- Confirmed: current "world agent availability" truth is split across inventory (`config/agents/*.yaml`) and policy allowlists (`agents.allowed_backends`, `agents.world_dispatch.allowed_backends`), so "world agent exists" and "world agent can actually boot" are separate truths today.
- Confirmed: current world-deps contract is a three-step model: define inventory, enable it, then **apply** it with `substrate world deps current sync`.
- Confirmed: enabled deps are **not** applied until sync runs; there is no live repo authority saying world refresh/restart is required after sync.
- Confirmed: `substrate world enable --provision-deps` is a provisioning prerequisite only when required guest OS packages are missing, and it currently does **not** auto-run sync afterward.
- Confirmed: current repo already supports script-installed, world-global tools that land under `/var/lib/substrate/world-deps/...` and expose entrypoints under `/var/lib/substrate/world-deps/bin`.
- Confirmed: current repo does **not** yet define a first-class binary-artifact/checksum/archive contract for world-deps; `install.method: script` is the closest existing fit.
- Confirmed: the following placement-aware planning artifacts already exist and intentionally defer the runtime fix:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md`
- Confirmed: current installer surfaces relevant to the chosen follow-on are:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install.sh`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/world-enable.sh`

## Key Decisions

### Decision 1
- Decision: treat the current `cli:codex_world` blocker as a **world runtime/bootstrap contract** problem, not a world-binding or transport problem.
- Why: the write-up proves that world-backed start persists authoritative binding and that the failure moved to member bootstrap with exit `127`.
- Alternatives considered:
  - Reinterpret the issue as a `SPEC-32` / world-binding persistence failure.
  - Reinterpret the issue as a host `--scope host` semantics problem.
- Evidence:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/CODEX_WORLD_DISPATCH_GAP_WRITEUP.md:17-30`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/CODEX_WORLD_DISPATCH_GAP_WRITEUP.md:442-501`
- Consequences:
  - Future design and implementation work should focus on guest-visible runtime realization, validation, and remediation.
  - Do not spend follow-on slices reopening host/world binding semantics unless product goals change.

### Decision 2
- Decision: the existing `llm-last-mile/DESIGN*` stack should be treated as sufficient for binding/transport/tool-surface semantics but incomplete for world-scoped guest runtime realization.
- Why: the design docs freeze exact identity, control-plane verbs, and runtime-owned tool/transport behavior, but they do not spell out the guest-runtime/bootstrap contract for world-scoped CLI runtimes.
- Alternatives considered:
  - Treat the existing design docs as already covering the runtime bootstrap contract.
  - Rewrite the dispatch/binding designs instead of adding a narrower runtime-realizability seam.
- Evidence:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md:69-81`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-host-orchestrator-tool-invocation-surface.md:63-73`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-internal-toolbox-transport-and-session-binding.md:68-82`
- Consequences:
  - Add a new runtime-realizability slice instead of destabilizing the existing control-plane docs.

### Decision 3
- Decision: future inventory should move toward **one logical agent with placement-specific variants** rather than permanent duplicated `codex` + `codex_world` files.
- Why: that keeps one logical agent visible while still allowing placement-specific launch truth, validation, remediation, and exact backend identity. It also avoids forcing users to maintain duplicate host/world files purely to express placement.
- Alternatives considered:
  - Keep duplicated host/world inventory files forever.
  - Allow one entry with `execution.scope: [host, world]` only.
- Evidence:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex.yaml`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex_world.yaml`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/config_model.rs:396-402`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:127-139`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md`
- Consequences:
  - Placement-aware schema work remains real and should still land.
  - Inventory UX should converge on one logical file such as `codex`, while internal realized targets remain distinct host/world backends.
  - But schema cutover must not be treated as the actual runtime fix.

### Decision 4
- Decision: exact internal selectors should be placement-qualified (`cli:codex-host` / `cli:codex-world`) and human-facing UX should display `codex (host)` / `codex (world)`.
- Why: placement must remain part of exact backend identity, while UX should not force low-level selectors into every status surface.
- Alternatives considered:
  - Preserve long-term `cli:codex_world` / `cli:codex` topology.
  - Use `cli:codex:host` / `cli:codex:world`.
  - Hide placement outside the exact backend id.
- Evidence:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/transport-api-types/src/lib.rs:1245-1259`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-16.md:64-69`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md`
- Consequences:
  - Future migration will touch selectors, policies, fixtures, and docs.
  - But that migration should follow the runtime-truth slice, not precede it.

### Decision 5
- Decision: **do not** land the placement-aware config cutover before the world-runtime realizability slice.
- Why: config cleanup alone would leave the same broken host-resolved bootstrap path and would make the repo cleaner without making it truthful.
- Alternatives considered:
  - Land Slice `58` implementation first, then return later to runtime truth.
  - Attempt to land both slices simultaneously.
- Evidence:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs` (host-side `which` realizability path)
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs` (copies resolved host path into member dispatch)
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs` (late bootstrap checks)
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/CODEX_WORLD_DISPATCH_GAP_WRITEUP.md:496-501`
- Consequences:
  - Planning should still define Slice `58` now.
  - But implementation should land runtime truth first, then guest runtime delivery, then config-shape migration.

### Decision 6
- Decision: the preferred guest-runtime delivery model is **Option A** — use the existing world-deps `install.method: script` contract to install a guest-visible Codex runtime under `/var/lib/substrate/world-deps/...` and expose `codex` under `/var/lib/substrate/world-deps/bin`.
- Why: current repo truth already supports script-installed, runtime-applied world-global tools via `substrate world deps current sync`, while no first-class artifact transport contract exists yet.
- Alternatives considered:
  - Design a new first-class binary artifact/checksum/archive delivery system first.
  - Depend on the host-local NVM/npm install.
  - Assume the guest always has a system-installed Codex/Node runtime.
- Evidence:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/world/deps.md:52-57`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/authoring_packages.md:32-56`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/surfaces.rs:1570-1619`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/surfaces.rs:989-1003`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/inventory.rs:51-82`
- Consequences:
  - Follow-on runtime work should begin with a script-installed Codex package/bundle design.
  - New artifact-contract work should stay deferred unless the script-package approach proves insufficient.

### Decision 7
- Decision: treat “Codex standalone binary does not require Node” as **unverified** until explicitly proven in the guest.
- Why: official OpenAI sources confirm standalone installers and downloadable Linux binaries, but the Substrate repo does not yet prove that the downloaded Linux binary is fully self-contained in the exact guest environment.
- Alternatives considered:
  - Assume Node is always required because the current failing path is NVM-based.
  - Assume Node is never required because official binaries exist.
- Evidence:
  - OpenAI Codex CLI docs: `https://developers.openai.com/codex/cli`
  - OpenAI Codex GitHub README: `https://github.com/openai/codex`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/CODEX_WORLD_DISPATCH_GAP_WRITEUP.md:446-479`
- Consequences:
  - The follow-on slice should spec a verification seam for the standalone-binary assumption.
  - If the binary still needs Node or other guest runtime pieces, the chosen approach should widen from “Codex package” to “Codex bundle including its true guest runtime dependencies.”

### Decision 8
- Decision: add an installer-time flag to provision the chosen world-runtime path and wire it into both the **production installer** and the **dev installer**.
- Why: the user wants a one-command operator path to make the chosen Codex world-runtime available during install, and the current installer surfaces do not provide a dedicated Codex-runtime provisioning flag.
- Alternatives considered:
  - Rely only on post-install manual `substrate world deps ...` commands.
  - Add the behavior only to prod install.
  - Add the behavior only to dev install.
- Evidence:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:82-93`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:2238-2265`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh:21-36`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/world-enable.sh:24-38`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/INSTALLATION.md:253-260`
- Consequences:
  - The follow-on runtime slice must include installer-surface design, not just runtime/validator changes.
  - Docs must clarify the exact sequence between install-time provisioning, `--provision-deps`, and `world deps current sync`.

## Research Findings

### Research Topic 1
- Question: What is the real current blocker for `cli:codex_world`?
- Findings:
  - The blocker is not missing authoritative world binding.
  - Host `cli:codex` toolbox access now works after the sandbox-bypass fix.
  - World-backed `cli:codex_world` launch still fails because Substrate reuses a host-resolved NVM Codex path that is not guaranteed to be guest-visible or guest-executable.
- Most relevant sources:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/CODEX_WORLD_DISPATCH_GAP_WRITEUP.md`
- Implication:
  - The next seam is explicit world-runtime validation/remediation, not more binding/transport debugging.

### Research Topic 2
- Question: How does current runtime realizability decide that world-scoped Codex is launchable?
- Findings:
  - Shell-side validation still host-resolves `binary: codex` with `which::which(...)`.
  - World dispatch then serializes that absolute path into the member payload.
  - world-service only checks that the file exists before bootstrap; it does not validate guest-side interpreter/runtime closure.
  - `world_exec_guard` is not the primary protection for this member bootstrap path.
- Most relevant sources:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/gateway_runtime.rs`
- Implication:
  - The minimal truthful fix starts in runtime validation/materialization, not in config shape and not in late bootstrap-only checks.

### Research Topic 3
- Question: Does current world-deps contract require refresh/restart after enabling new deps?
- Findings:
  - Enabled deps are only desired state; `substrate world deps current sync` is the apply step.
  - No live repo authority says world refresh/restart is required after sync.
  - For APT/pacman-backed deps, sync is sufficient only after required guest OS packages already exist; otherwise `substrate world enable --provision-deps` is required first.
  - `substrate world enable --provision-deps` currently does not auto-sync afterward.
- Most relevant sources:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/README.md`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/provisioning.md`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/surfaces.rs`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/world-enable.sh`
- Implication:
  - The preferred Codex package approach should target the existing enable -> provision-deps if needed -> sync flow, not require an extra world refresh contract.

### Research Topic 4
- Question: Does current repo already have a suitable mechanism for shipping a standalone guest-visible tool into the world?
- Findings:
  - Yes, mostly: current world-deps `install.method: script` is a workable contract for installing tools under `/var/lib/substrate/world-deps/<pkg>` and surfacing entrypoints via `/var/lib/substrate/world-deps/bin`.
  - Existing examples (`hello`, `nvm`, built-in `bun`) show the relevant patterns.
  - The repo does not yet have a first-class binary artifact/checksum/archive transport contract for world-deps.
- Most relevant sources:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/authoring_packages.md`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/examples/README.md`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/examples/packages/nvm.yaml`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/inventory.rs`
- Implication:
  - Prefer the existing script-package contract first; defer first-class artifact-contract design unless needed.

### Research Topic 5
- Question: Which installer surfaces are relevant to the chosen runtime-provisioning path?
- Findings:
  - Production install currently exposes `--sync-deps` but no dedicated Codex runtime provisioning flag.
  - Dev install currently provisions world infrastructure but exposes no analogous runtime-provisioning flag.
  - `world-enable.sh` already owns world provisioning plus optional sync semantics and is a natural reuse point.
- Most relevant sources:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install.sh`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/world-enable.sh`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/INSTALLATION.md`
- Implication:
  - The runtime slice should include installer UX/flag design for both dev and prod, not treat it as incidental cleanup.

### Research Topic 6
- Question: Where does current repo truth say which agents are available in world?
- Findings:
  - Concrete host/world agent existence is defined by inventory files such as `config/agents/codex.yaml` and `config/agents/codex_world.yaml`, not by a single top-level merged config list.
  - Policy allowlists such as `agents.allowed_backends` and `agents.world_dispatch.allowed_backends` separately govern whether exact backends may actually be used.
  - Current `execution.scope` is a singular enum in the live config/inventory model, and the inventory projector realizes one execution scope per entry.
  - Therefore, `substrate config show` alone is insufficient to answer "which world agents exist?" and `execution.scope: [host, world]` would not match the live schema without a deliberate inventory redesign.
- Most relevant sources:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/config_model.rs`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex.yaml`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex_world.yaml`
- Implication:
  - The placement-aware redesign should be understood as an explicit inventory-shape upgrade, not a small tweak to the existing singular-scope field. It also needs to keep "world agent exists" separate from "world agent can boot".

## Evidence / Source Map

### Repository files
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/CODEX_WORLD_DISPATCH_GAP_WRITEUP.md` — primary durable write-up of the active Codex world-dispatch/runtime gap.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex.yaml` — current host-scoped Codex inventory entry.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex_world.yaml` — current world-scoped Codex inventory entry.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs` — current host-side runtime realizability logic.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs` — serializes resolved runtime truth into world member dispatch.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs` — guest PATH normalization and world dispatch helpers.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs` — current late bootstrap checks for member launch.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/gateway_runtime.rs` — wrapper/bootstrap execution path for member runtime launch.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/world_exec_guard.rs` — world guardrail against host-mounted toolchains.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/README.md` — operator-facing world-deps contract.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/provisioning.md` — stable provisioning contract for runtime vs provisioning-time mutation.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/authoring_packages.md` — authoring contract for script-installed runnable packages.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/world/deps.md` — implementation details confirming runtime-applied script-package behavior.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/surfaces.rs` — sync/install behavior, wrapper reconciliation, and runtime checks.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/inventory.rs` — current world-deps schema and built-in package methods.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md` — public policy/config contract showing allowlist surfaces separate from inventory files.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/config_model.rs` — live singular `execution.scope` config model.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs` — current inventory projector that realizes one execution scope per entry.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/world-enable.sh` — current world provisioning helper plus optional sync behavior.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh` — prod installer contract, including `--sync-deps`.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install.sh` — prod thin wrapper installer.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh` — dev installer surface that will also need the new runtime-provisioning flag.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/INSTALLATION.md` — published installer flag contract.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md` — drafted forward config/selector contract.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md` — drafted implementation order for placement-aware inventory.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md` — drafted packetized execution plan for placement-aware inventory.

### External sources
- `https://developers.openai.com/codex/cli` — official OpenAI Codex CLI install docs confirming standalone installer support.
- `https://github.com/openai/codex` — official OpenAI Codex repo README confirming downloadable Linux release binaries.
- `https://github.com/openai/codex/releases` — official OpenAI release distribution surface for downloadable CLI binaries.

### Logs / artifacts
- No new standalone raw logs were captured in this continuation; the durable evidence remains the repo-local write-up and the code/docs above.

## Options Considered

### Option A
- Description: use the existing world-deps `install.method: script` contract to install a guest-visible Codex runtime under `/var/lib/substrate/world-deps/...` and expose `codex` via `/var/lib/substrate/world-deps/bin`.
- Pros:
  - Fits the current repo contract cleanly.
  - Reuses existing enable/provision-deps/sync workflow.
  - Avoids inventing a new artifact transport system before it is proven necessary.
  - Can be paired with fail-closed validator/remediation work immediately.
- Cons:
  - Still needs guest verification for whether the standalone binary is truly self-contained.
  - Does not provide a first-class checksum/archive/offline artifact contract.
- Outcome: **selected preferred direction**.

### Option B
- Description: build a new first-class artifact/binary transport contract before delivering Codex into the world.
- Pros:
  - Could encode checksums, archive metadata, offline install, and host-staged asset transfer explicitly.
- Cons:
  - Larger scope than the currently observed runtime bug requires.
  - Delays the truthful fail-early/runtime fix.
  - Not required by current repo contract if script-installed package is sufficient.
- Outcome: deferred unless Option A proves insufficient.

### Option C
- Description: land placement-aware agent config/selector cutover before runtime-truth/remediation work.
- Pros:
  - Cleans up duplicated inventory sooner.
- Cons:
  - Leaves the actual `127` in place.
  - Makes the repo cleaner without making it truthful.
  - Risks conflating shape cleanup with runtime fix.
- Outcome: rejected as the implementation order.

## Open Questions / Unresolved Risks

### Open Questions
- Is the official downloadable Linux Codex binary truly self-contained in the target guest environment, or does it still require Node or other runtime pieces?
- Should the preferred world-deps package be a single `codex` package or a `codex-runtime` bundle that can include guest prerequisites if verification proves they are needed?
- What should the new installer/runtime-provisioning flag be called on prod and dev installer surfaces?
- Should install-time provisioning flag behavior imply sync as well, or expose provisioning and sync separately?
- After the runtime slice lands, should Slice `58` migration preserve temporary compatibility aliases for `cli:codex_world` or do a one-step exact-selector cutover?

### Risks
- If the runtime slice lands without explicit standalone-binary verification, the selected Option A could still hide a missing guest dependency.
- If the config-shape cutover lands before runtime truth, the repo will stay semantically wrong even if the inventory becomes cleaner.
- If installer UX is added only to one surface (prod or dev), operators will have divergent provisioning stories and reintroduce brittle manual steps.
- If the new provisioning flag is underspecified, users may assume install-time provisioning is enough even when a later `world deps current sync` step is still required.
- If migration from `cli:codex_world` to placement-qualified selectors is incomplete, old and new exact backend ids could coexist ambiguously.

## Recommended Next Steps

1. Draft the follow-on runtime-realizability slice before implementing Slice `58`.
   - Goal: freeze the truthful fail-closed contract for world-scoped CLI runtimes.
   - Output:
     - `SPEC-59-...` for world-scoped CLI runtime realizability/remediation
     - matching `PLAN-59-...`
     - matching `TASKS-59.md`
   - Success condition: a fresh implementer can answer “what must be true before world-scoped Codex is considered launchable?” without reading chat history.

2. Define the preferred Codex world-deps package/bundle around Option A.
   - Goal: make guest-visible Codex available through the existing world-deps script-package contract.
   - Output: a package/bundle design that installs under `/var/lib/substrate/world-deps/...`, exposes `/var/lib/substrate/world-deps/bin/codex`, and states whether extra guest runtime pieces are required.
   - Success condition: the runtime slice names one concrete guest-runtime delivery model instead of leaving it abstract.

3. Add installer-time provisioning flag requirements to the runtime slice.
   - Goal: make the chosen runtime-delivery model available from both install surfaces.
   - Output: explicit requirements for prod installer (`install-substrate.sh` / `install.sh`) and dev installer (`dev-install-substrate.sh`) plus docs updates.
   - Success condition: a user can discover one documented flag on both install surfaces that provisions the chosen Codex runtime path at install time.

4. After the runtime slice is approved, implement in this order.
   - Goal: avoid config-shape-first drift.
   - Output:
     1. fail-closed validator/remediation changes,
     2. Codex guest runtime package/bundle plus smoke proof,
     3. Slice `58` config/selector migration.
   - Success condition: runtime truth is fixed before inventory shape is migrated.

5. Only after runtime truth is green, execute the placement-aware inventory cutover.
   - Goal: complete the long-term logical-agent/placement model without masking the real bug.
   - Output: approved Slice `58` implementation plus migration updates for selectors, policies, fixtures, and docs.
   - Success condition: the repo is both semantically truthful and structurally cleaner.

## Resume Instructions

A fresh agent should:

1. Read:
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SESSION_DECISION_DOSSIER-agent-placement-shape-and-world-runtime-gap.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/CODEX_WORLD_DISPATCH_GAP_WRITEUP.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/README.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/provisioning.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/deps/authoring_packages.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/world/deps.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/world-enable.sh`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md`

2. Reconfirm:
   - whether any newer uncommitted work already changed runtime realizability in `validator.rs` or world member bootstrap,
   - whether any newer uncommitted work already added a Codex world-deps package/bundle,
   - whether any newer uncommitted work already added installer-time provisioning flags on prod/dev install surfaces,
   - whether the standalone Linux Codex binary is actually self-contained in the target guest environment.

3. Avoid redoing:
   - re-litigating whether the active gap is world binding versus guest runtime bootstrap,
   - re-proposing config-shape cutover as the first implementation step,
   - assuming world refresh/restart is required after `world deps current sync` without new evidence,
   - designing a new artifact transport system before testing whether the chosen script-package path is sufficient.

4. Start with:
   - drafting the follow-on runtime-realizability `SPEC/PLAN/TASKS` slice,
   - explicitly encoding the chosen Option A world-deps script-package approach,
   - explicitly encoding the new installer-time provisioning flag requirement for both prod and dev installer surfaces,
   - then only after that, begin implementation planning for validator/remediation, package delivery, and later Slice `58` cutover.
