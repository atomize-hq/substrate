# ORCH_PLAN: UAA Boundary And Naming Cleanup Execution Controller

Authoritative plan source: [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)  
Source SOW: [27-uaa-boundary-and-naming-cleanup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/27-uaa-boundary-and-naming-cleanup.md)  
Style reference: [ORCH_PLAN-25.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-25.md)  
Current workspace branch: `chore/uaa-boundary-and-naming-cleanup`  
Execution type: direct-cutover naming correction, parent-frozen contract, one serialized foundation code lane, two follow-on parallel lanes, one parent-only validation wall  
Live repo root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`  
Fresh worktree root: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-uaa-boundary-cleanup`  
Run id: `plan-uaa-boundary-and-naming-cleanup`  
Worker model: `GPT-5.4` with `reasoning_effort=high`  
Initial concurrent worker cap: `1`  
Peak concurrent worker cap: `2`  
Parent role: sole integrator, sole approval authority, sole writer of `.runs/**` artifacts

## Summary

This controller executes [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md). It is not a restatement of the plan.

The run shape is frozen:

1. Parent freezes the rename contract, the live-surface grep wall, the historical allowlist, and the worker ownership map.
2. One foundation code lane lands the whole shared-hotspot cutover in order:
   - `uaa.agent.session` -> `substrate.agent.session`
   - `world-agent*` -> `world-service*`
   - `agent-api*` -> `transport-api*`
3. Only after the foundation lane is accepted does the parent open the parallel window:
   - scripts, CI, release, and operator helper lane
   - docs, ADR, and repo-truth lane
4. Parent integrates the parallel lanes, runs the full validation wall from [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md), and decides final acceptance.

This run is honest only if the merged tree proves all of the following together:

1. upstream `agent_api`, adopted `agent_api.*`, `uaa_session_id`, and unchanged `world-api` remain intact,
2. `substrate.agent.session` is the only canonical supported local protocol-family label on live surfaces,
3. the local daemon family is `world-service*` everywhere live,
4. the local typed host↔world contract family is `transport-api*` everywhere live,
5. world-required routing still fails closed if renamed service discovery breaks,
6. the live-surface grep wall is green outside the explicit historical allowlist,
7. code, scripts, CI, release bundles, docs, ADRs, fixtures, tests, and operator guidance tell one coherent boundary story.

## Hard Guards

These are run-stopping invariants.

1. This slice is a naming correction and boundary cleanup only. It is not a runtime redesign, not a public contract redesign, and not a compatibility-migration project.
2. Upstream `agent_api` and adopted `agent_api.*` ids remain untouched.
3. `world-api` remains untouched.
4. `uaa_session_id` and `internal.uaa_session_id` remain untouched.
5. `substrate.agent.session` is the only canonical supported local protocol-family label after cutover.
6. Old supported live names do not remain as aliases on canonical live surfaces just to avoid repo work.
7. Stale `uaa.agent.session` configs, fixtures, or persisted runtime rows fail closed with explicit operator-readable errors after cutover.
8. World-required routing must still hard-fail if renamed binary, socket, or unit discovery breaks. No rename drift may silently fall back to host execution.
9. No parallel worker may touch shared foundation hotspots before the foundation lane is accepted:
   - [Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.toml)
   - `dist-workspace.toml`
   - `crates/shell/**`
   - mixed-boundary imports that mention upstream `agent_api` plus local transport or service crates
   - local protocol validation and persistence seams
10. Parent is the only integrator, the only gate authority, and the only writer of `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-uaa-boundary-and-naming-cleanup/**`.
11. Workers do not edit [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md), [ORCH_PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md), or `.runs/**`.
12. Every symbol edit requires prior GitNexus impact analysis. Any `HIGH` or `CRITICAL` result is a parent-only escalation point.
13. Every worker handoff must include `gitnexus_detect_changes()` status before the parent considers merge.
14. Parent runs a final `gitnexus_detect_changes()` on the merged tree before closeout.

Stop the run and write `blocked.json` if any of these become true:

1. The foundation cutover needs to be split across multiple concurrent code owners to make progress.
2. The rename cannot be landed without changing public selector semantics, `world-api`, upstream `agent_api.*`, or `uaa_session_id`.
3. The only path to green is a long-lived compatibility alias layer for old live names.
4. Fail-closed routing cannot be proven after the `world-service` rename.
5. Parallel lanes need to reopen shared manifests, shell runtime seams, or rename decisions that should have been frozen by the foundation lane.
6. Docs or ADRs can only be made truthful by contradicting the merged code and scripts tree.
7. The final validation wall cannot prove grep, cargo, operator, release, and fail-closed checks on the same merged tree.

## Fresh Worktrees / Branches / Worker Model / Concurrency Cap

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-uaa-boundary-cleanup`

Authoritative integration checkout:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- branch: `chore/uaa-boundary-and-naming-cleanup`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-uaa-boundary-cleanup/foundation-code`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-uaa-boundary-cleanup/scripts-release`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-uaa-boundary-cleanup/docs-adrs`

Worker branches:

- `codex/chore-uaa-boundary-and-naming-cleanup-foundation`
- `codex/chore-uaa-boundary-and-naming-cleanup-scripts-release`
- `codex/chore-uaa-boundary-and-naming-cleanup-docs-adrs`

Exact setup order:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-uaa-boundary-cleanup

git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate fetch origin

git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate worktree add \
  /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-uaa-boundary-cleanup/foundation-code \
  -b codex/chore-uaa-boundary-and-naming-cleanup-foundation \
  chore/uaa-boundary-and-naming-cleanup
```

Do not create the parallel worktrees until `G1` is accepted.

```bash
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate worktree add \
  /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-uaa-boundary-cleanup/scripts-release \
  -b codex/chore-uaa-boundary-and-naming-cleanup-scripts-release \
  chore/uaa-boundary-and-naming-cleanup

git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate worktree add \
  /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-uaa-boundary-cleanup/docs-adrs \
  -b codex/chore-uaa-boundary-and-naming-cleanup-docs-adrs \
  chore/uaa-boundary-and-naming-cleanup
```

Concurrency contract:

1. `P0` and `A1` / `A2` are serialized.
2. `B1` and `C1` are the only honest parallel window.
3. Peak concurrency is `2`, not `3`, because there is one shared foundation hotspot lane and two isolated follow-on lanes.

## Parent-Owned Run-State Surface And Required Artifacts

Canonical run root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-uaa-boundary-and-naming-cleanup/`

Required parent-owned top-level artifacts:

- `run-state.json`
- `source-lock.json`
- `contract-freeze.json`
- `branch-map.json`
- `lane-ownership.json`
- `merge-order.json`
- `validation-wall.md`
- `session-log.md`
- `final-summary.md`
- `blocked.json` on blocked runs only
- `sentinels/`
- `tasks/`
- `gates/`

Required sentinels:

- `sentinels/RUN_OPEN`
- `sentinels/RUN_BLOCKED` on blocked runs only
- `sentinels/RUN_COMPLETE` on successful closeout only

Frozen gate directories:

- `gates/G0-parent-contract-freeze/`
- `gates/G1-foundation-accept-and-parallel-launch/`
- `gates/G2-parallel-window-integration/`
- `gates/G3-validation-launch/`
- `gates/G4-final-acceptance/`

Every gate directory must contain:

- `gate.json`
- `evidence.md`
- one sentinel exactly one of:
  - `OPEN`
  - `PASSED`
  - `FAILED`
  - `REOPENED`

Task map:

- `tasks/P0-parent-contract-freeze-and-run-init/`
- `tasks/A1-protocol-label-cutover/`
- `tasks/A2-family-rename-foundation-cutover/`
- `tasks/G1-foundation-accept-and-parallel-launch/`
- `tasks/B1-scripts-ci-release-cutover/`
- `tasks/C1-docs-adr-truth-convergence/`
- `tasks/G2-parallel-window-integration/`
- `tasks/P1-parent-parallel-integration/`
- `tasks/G3-validation-launch/`
- `tasks/P2-parent-validation-wall/`
- `tasks/P3-parent-closeout/`

Every task directory must contain:

- `task.json`
- `owner.txt`
- `status.txt`
- `scope.txt`
- `deliverable.txt`
- `dependencies.json`
- `changed-files.txt`
- `commands.txt`
- `exit-codes.json`
- `impact-analysis-summary.md`
- `gitnexus-detect-changes.txt`
- `handoff-notes.md`
- `summary.md`
- `HEAD_SHA.txt`
- `blocker-notes.md` if blocked
- one sentinel exactly one of:
  - `READY_FOR_REVIEW`
  - `ACCEPTED`
  - `REJECTED`
  - `BLOCKED`

Worker artifact rule:

1. Workers do not write `.runs/**`.
2. Workers return handoff content to the parent.
3. Parent transcribes the handoff into `.runs/**`, including changed files, commands, exit codes, impact notes, detect-changes output, acceptance notes, and blocker notes.
4. Parent creates or replaces the task sentinel after transcription. Workers never create task sentinels.
5. No task is complete until the parent writes the artifacts and marks the task `ACCEPTED`.

`contract-freeze.json` must record at minimum:

1. `authoritative_branch: "chore/uaa-boundary-and-naming-cleanup"`
2. locked rename families:
   - `uaa.agent.session -> substrate.agent.session`
   - `world-agent* -> world-service*`
   - `agent-api* -> transport-api*`
3. preserved names:
   - upstream `agent_api`
   - upstream `agent_api.*`
   - `uaa_session_id`
   - unchanged `world-api`
4. the fail-closed rule for renamed service discovery and stale protocol labels
5. the live rename boundary
6. the historical allowlist
7. the exact validation wall commands
8. the initial worker cap of `1` and peak worker cap of `2`

## GitNexus Workflow And Required Impact Targets

GitNexus is a required run control, not a best-effort check.

### Source-Lock Stage

1. Parent checks GitNexus availability and index freshness before any worker edits.
2. If the index is stale, parent runs `npx gitnexus analyze` from the authoritative checkout before launching `A1`.
3. Parent records the freshness result in `tasks/P0-parent-contract-freeze-and-run-init/impact-analysis-summary.md`.

### Minimum Required Targets By Task

`A1` protocol-label cutover

1. Concrete symbol targets if present:
   - `LOCAL_AGENT_PROTOCOL_FAMILY`
   - the validator entrypoint that accepts or rejects local protocol labels in `crates/shell/src/execution/agent_runtime/validator.rs`
   - the trace-emission seam in `crates/common/src/agent_events.rs`
2. File-level seam targets that must be analyzed if symbol names drift:
   - `crates/shell/src/execution/agent_runtime/mapping.rs`
   - `crates/shell/src/execution/agent_runtime/validator.rs`
   - `crates/shell/src/execution/agent_runtime/session.rs`
   - `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
   - `crates/shell/src/execution/agent_runtime/state_store.rs`
   - `crates/shell/src/execution/routing/dispatch/world_ops.rs`

`A2` family-rename foundation cutover

1. Concrete symbol targets if present:
   - the Linux world-routing or socket-activation resolution path that locates the in-world daemon
   - the `world-agent` binary entrypoint crate target being renamed to `world-service`
2. File-level seam targets that must be analyzed if symbol names drift:
   - [Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.toml)
   - `crates/shell/src/repl/async_repl.rs`
   - `crates/shell/src/execution/socket_activation.rs`
   - `crates/shell/src/execution/platform/linux.rs`
   - `crates/shell/src/execution/routing/world.rs`
   - `crates/world-agent/**` or renamed equivalents
   - `crates/agent-api-types/**`, `crates/agent-api-core/**`, and `crates/agent-api-client/**` or renamed equivalents

`B1` scripts / CI / release cutover

1. File-level seam targets:
   - `scripts/linux/world-provision.sh`
   - `scripts/substrate/install-substrate.sh`
   - `scripts/substrate/dev-install-substrate.sh`
   - `scripts/substrate/uninstall-substrate.sh`
   - `scripts/mac/lima-warm.sh`
   - `scripts/mac/smoke.sh`
   - `scripts/windows/wsl-warm.ps1`
   - `scripts/windows/wsl-smoke.ps1`
   - `scripts/windows/wsl-doctor.ps1`
   - `dist/scripts/assemble-release-bundles.sh`
   - `.github/workflows/feature-smoke.yml`
   - `.github/workflows/nightly.yml`
2. If GitNexus has script or workflow indexing for callable entrypoints in these files, analyze those entrypoints before editing; otherwise the file-level seam analysis is sufficient and must be stated explicitly in the handoff.

`C1` docs / ADR / truth convergence

1. File-level seam targets:
   - `AGENT_ORCHESTRATION_GAP_MATRIX.md`
   - `docs/WORLD.md`
   - `docs/TRACE.md`
   - `docs/CONFIGURATION.md`
   - `docs/INSTALLATION.md`
   - `docs/UNINSTALL.md`
   - `docs/USAGE.md`
   - `README.md`
   - `AGENTS.md`
   - relevant ADRs under `docs/project_management/adrs/**`
2. GitNexus impact is docs-truth only here. If no symbol-level targets are relevant, the worker must state that the lane is file-level and docs-only.

### Escalation Rule

1. Any `HIGH` or `CRITICAL` impact result stops that worker before edits.
2. The worker returns a blocker handoff instead of proceeding.
3. Parent records the blocker, decides whether the run still fits the frozen contract, and either relaunches with a narrower brief or blocks the run.

## Workstream Plan With Parent-Owned Gates And Worker-Owned Lanes

### Workstream Map

| PLAN.md workstream | Orchestration tasks | Ownership |
| --- | --- | --- |
| Freeze vocabulary, rename matrix, grep wall, historical allowlist | `P0`, `G0` | Parent only |
| Cut over `substrate.agent.session` on runtime, validation, trace, config, and persistence surfaces | `A1` | Foundation lane |
| Rename `world-agent*` and `agent-api*` families on code and manifests; keep mixed-boundary imports clear | `A2` | Foundation lane |
| Sweep scripts, CI, release bundles, install/warm/smoke/uninstall, and operator helper text | `B1` | Scripts/release lane |
| Sweep docs, ADRs, repo truth docs, examples, operator guidance, and matrix language | `C1` | Docs/ADR lane |
| Integrate, validate, and close out | `G2`, `P1`, `G3`, `P2`, `G4`, `P3` | Parent only |

### Parent-Owned Gates

`G0`: Contract freeze

1. Parent locks the rename matrix, preserved-name list, live boundary, historical allowlist, validation wall, and branch map.
2. Parent confirms that no worker will need to guess names, scope, or final grep commands.

`G1`: Foundation acceptance and parallel launch

1. `A1` and `A2` are both merged or otherwise accepted into the authoritative branch.
2. Parent confirms that shared hotspots are stable:
   - manifests
   - `crates/shell/**`
   - local protocol validation and persistence seams
   - mixed-boundary imports
3. Parent confirms the new canonical names are real enough for downstream scripts and docs work.
4. Only after `G1` is accepted may `B1` and `C1` start.

`G2`: Parallel-window integration

1. Parent receives both worker handoffs.
2. Parent verifies each lane stayed inside its ownership boundary.
3. Parent merges accepted outputs into `chore/uaa-boundary-and-naming-cleanup`.
4. Parent quarantines any lane that reopens foundation hotspots without approval.

`G3`: Validation launch

1. Parent confirms the merged tree is the validation candidate.
2. Parent confirms no pending naming disputes remain.
3. Parent freezes the validation command list from the plan and starts `P2`.

`G4`: Final acceptance

1. Grep wall is green outside the historical allowlist.
2. Cargo wall is green.
3. Operator surface gates are green.
4. Release gates are green.
5. Fail-closed checks are green.
6. Parent runs final GitNexus scope verification and writes closeout.

### Worker-Owned Lanes

`A` Foundation code lane: serialized under one owner

1. `A1` protocol-label cutover
2. `A2` family rename cutover

`B` Scripts / CI / release lane: launch only after `G1`

1. install and provision scripts
2. dev-install and uninstall scripts
3. Lima and WSL warm/smoke/doctor helpers
4. release bundle assembly
5. CI workflow package invocations and smoke checks
6. operator helper, remediation, and script output text in those surfaces

`C` Docs / ADR / truth lane: launch only after `G1`

1. live docs
2. repo truth docs
3. ADRs
4. README and AGENTS guidance
5. examples, diagrams, and operator remediation wording in docs

## Task Execution Contracts

### `P0-parent-contract-freeze-and-run-init`

Primary owned surfaces:

- [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)
- [ORCH_PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md)
- `.runs/plan-uaa-boundary-and-naming-cleanup/**`

Required actions:

1. Freeze the rename matrix, preserved-name list, live rename boundary, historical allowlist, and validation wall exactly from the current plan.
2. Create `branch-map.json`, `lane-ownership.json`, `merge-order.json`, and `contract-freeze.json`.
3. Record GitNexus freshness status and the concurrency contract.
4. Write the initial gate and task directories with `OPEN` and `READY_FOR_REVIEW` only where appropriate for parent-owned setup completion.

Verification commands:

```bash
test -f /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md
test -f /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md
```

Acceptance conditions:

1. No worker would need to guess names, ownership, or final validation commands.
2. `.runs/**` skeleton exists on paper with required artifacts and sentinel rules.
3. Parent has recorded whether GitNexus freshness work is needed before `A1`.

### `A1-protocol-label-cutover`

Primary owned surfaces or file families:

- `crates/common/src/agent_events.rs`
- `crates/shell/src/execution/agent_runtime/**`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `config/**`
- code-adjacent tests and fixtures for local protocol validation, trace emission, and persisted runtime state

Required actions:

1. Replace the canonical local protocol-family label from `uaa.agent.session` to `substrate.agent.session`.
2. Update validator success paths, rejection text, config examples, emitted traces, transport payloads, durable writes, and durable reload paths to the new label.
3. Rewrite supported fixtures and checked-in examples that still treat `uaa.agent.session` as supported canonical input.
4. Make stale old-label configs or persisted rows fail closed with explicit operator-readable errors.
5. Preserve upstream `agent_api.*` ids and `uaa_session_id` semantics unchanged.

Verification commands:

```bash
cargo test -p shell agents_validate -- --nocapture
cargo test -p shell agent_successor_contract_ahcsitc0 -- --nocapture
rg -n "substrate\.agent\.session|uaa\.agent\.session" crates/common crates/shell config
```

Acceptance conditions:

1. `substrate.agent.session` is the only supported canonical local protocol-family label on live code and config surfaces owned by `A1`.
2. `uaa.agent.session` no longer succeeds silently on supported live paths.
3. Trace, status, validation, and persistence seams owned by `A1` all reflect the new label.
4. No upstream capability or session-correlation semantics changed.

### `A2-family-rename-foundation-cutover`

Primary owned surfaces or file families:

- [Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.toml)
- `dist-workspace.toml`
- `crates/world-agent/**` to be renamed
- `crates/agent-api-types/**`, `crates/agent-api-core/**`, `crates/agent-api-client/**` to be renamed
- mixed-boundary code under `crates/shell/**`
- code-level service discovery and doctor logic coupled to renamed package, binary, socket, or unit names

Required actions:

1. Rename `world-agent*` to `world-service*` across workspace members, package names, Rust crate ids, and binary names.
2. Rename `agent-api*` to `transport-api*` across workspace members, package names, Rust crate ids, and imports.
3. Update mixed-boundary consumers so upstream `agent_api` remains visually distinct from local `transport_api_*`.
4. Preserve `world-api` unchanged.
5. Keep service discovery fail closed when renamed binary, unit, or socket lookup breaks.

Verification commands:

```bash
cargo build --workspace
cargo test -p world-service -- --nocapture
cargo test -p transport-api-types -- --nocapture
cargo test -p transport-api-core -- --nocapture
cargo test -p transport-api-client -- --nocapture
rg -n "world-agent|world_agent|substrate-world-agent|agent-api-types|agent-api-core|agent-api-client|agent_api_types|agent_api_core|agent_api_client" Cargo.toml dist-workspace.toml crates
```

Acceptance conditions:

1. Workspace manifests and crate imports compile with `world-service` and `transport-api*`.
2. Mixed-boundary consumers remain readable: upstream `agent_api` untouched, local transport crates clearly renamed.
3. `world-api` is unchanged.
4. Code-level service discovery and doctor logic no longer depend on stale canonical `world-agent` naming.

### `B1-scripts-ci-release-cutover`

Primary owned surfaces or file families:

- `scripts/**`
- `.github/**`
- `dist/**`
- live packaging or launch helpers under `macos-hardening/**` if applicable

Required actions:

1. Rename package, binary, alias, socket, and systemd unit references to `world-service` and `substrate-world-service`.
2. Update install, dev-install, uninstall, provision, warm, smoke, and doctor flows to the new daemon and transport family names.
3. Update release bundle assembly and release template payload naming.
4. Update CI workflow package invocations and smoke steps.
5. Ensure upgrade-capable paths remove or stop legacy `substrate-world-agent` assets where the plan requires cleanup.

Verification commands:

```bash
rg -n "world-agent|substrate-world-agent|agent-api-types|agent-api-core|agent-api-client" scripts .github dist macos-hardening
rg -n "world-service|substrate-world-service|transport-api-types|transport-api-core|transport-api-client" scripts .github dist macos-hardening
./dist/scripts/assemble-release-bundles.sh
```

Acceptance conditions:

1. Live scripts, workflows, and release surfaces use the renamed service and transport families consistently.
2. No operator helper script or release script points canonical usage back to `substrate-world-agent`.
3. Release bundle assembly and CI naming surfaces prove the cutover beyond code compilation.

### `C1-docs-adr-truth-convergence`

Primary owned surfaces or file families:

- `docs/**`
- `README.md`
- `AGENTS.md`
- `AGENT_ORCHESTRATION_GAP_MATRIX.md`
- relevant ADRs under `docs/project_management/adrs/**`

Required actions:

1. Rewrite docs, ADRs, and truth docs so upstream UAA, local `transport-api*`, local `world-service*`, unchanged `world-api`, and local `substrate.agent.session` are clearly separated.
2. Update operator examples and remediation guidance to the new canonical names.
3. Remove live doc guidance that still teaches `uaa.agent.session` or `substrate-world-agent` as supported canonical names.
4. Keep historical references only where explicitly allowable as historical.

Verification commands:

```bash
rg -n "world-agent|substrate-world-agent|agent-api-types|agent-api-core|agent-api-client|uaa\.agent\.session" docs README.md AGENTS.md AGENT_ORCHESTRATION_GAP_MATRIX.md
rg -n "world-service|substrate-world-service|transport-api-types|transport-api-core|transport-api-client|substrate\.agent\.session|world-api|agent_api\." docs README.md AGENTS.md AGENT_ORCHESTRATION_GAP_MATRIX.md
```

Acceptance conditions:

1. Live docs and ADRs tell the same boundary story as the merged code and scripts tree.
2. Canonical examples use `substrate.agent.session`, `world-service*`, and `transport-api*`.
3. No live operator doc instructs readers to inspect `substrate-world-agent` or configure `uaa.agent.session`.

### `P1-parent-parallel-integration`

Primary owned surfaces:

- authoritative branch `chore/uaa-boundary-and-naming-cleanup`
- `.runs/**`

Required actions:

1. Review `B1` and `C1` handoffs against ownership boundaries.
2. Merge accepted lane outputs in the frozen order.
3. Replay or quarantine lanes when overlap breaks the ownership contract.
4. Record integration outcomes, replays, and quarantines in `.runs/**`.

Verification commands:

```bash
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate status --short
```

Acceptance conditions:

1. The authoritative branch contains exactly the accepted outputs from `B1` and `C1`.
2. Any overlap is resolved by explicit replay or quarantine, not silent manual blending.
3. `.runs/**` reflects the actual merge and replay history.

### `P2-parent-validation-wall`

Primary owned surfaces:

- merged authoritative tree
- `.runs/**`

Required actions:

1. Run the grep gates.
2. Run the cargo gates.
3. Run the operator surface gates.
4. Run the release gates.
5. Record fail-closed proof points and environment blockers, if any.

Verification commands:

1. The exact grep, cargo, operator, release, and fail-closed commands already frozen in `Validation Wall`.

Acceptance conditions:

1. The validation wall is green or explicitly blocked by environment availability with no ambiguity about follow-up.
2. The merged tree proves the rename across code, scripts, docs, release surfaces, and operator flows.
3. Parent records exact commands, exit codes, and evidence in `.runs/**`.

### `P3-parent-closeout`

Primary owned surfaces:

- `.runs/**`
- final authoritative branch state

Required actions:

1. Run final `gitnexus_detect_changes()` on the merged tree.
2. Confirm the changed scope matches the frozen plan.
3. Write `final-summary.md`, task and gate outcomes, and `RUN_COMPLETE`.
4. If the run cannot close honestly, write `blocked.json` instead.

Verification commands:

```bash
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate status --short
```

Acceptance conditions:

1. Final run-state artifacts match the actual merged result.
2. Parent has recorded final scope verification and residual risks.
3. The run ends with exactly one terminal sentinel: `RUN_COMPLETE` or `RUN_BLOCKED`.

## Exact Lane Ownership Boundaries By Directories / Modules

### Lane A: Foundation Code Ownership

Owns these surfaces end to end:

- [Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.toml)
- `dist-workspace.toml`
- `crates/common/**` where protocol-family trace identity is emitted
- `crates/shell/**`
- `crates/world-agent/**` to be renamed to `crates/world-service/**`
- `crates/agent-api-types/**` to be renamed to `crates/transport-api-types/**`
- `crates/agent-api-core/**` to be renamed to `crates/transport-api-core/**`
- `crates/agent-api-client/**` to be renamed to `crates/transport-api-client/**`
- code-adjacent tests and fixtures under those crates
- `config/**` for supported live protocol examples

Lane A owns both of these subproblems:

1. protocol-label cutover:
   - `uaa.agent.session -> substrate.agent.session`
   - validators
   - trace emission
   - transport payloads
   - durable writes and reload paths
   - fail-closed handling for stale rows and configs
2. family renames:
   - `world-agent* -> world-service*`
   - `agent-api* -> transport-api*`
   - workspace members
   - package names
   - Rust crate ids
   - mixed-boundary imports
   - in-code service discovery and doctor logic that depends on the renamed family

Lane A may also touch:

- code-level doctor or remediation strings that are directly coupled to renamed runtime or service discovery logic

Lane A may not defer any shared-hotspot name choice to later lanes.

### Lane B: Scripts / CI / Release Ownership

Owns these surfaces after `G1`:

- `scripts/**`
- `.github/**`
- `dist/**`
- `macos-hardening/**` if the file is part of live packaging, launch, or operator setup

Lane B specifically owns:

1. package invocation updates
2. binary, alias, socket, and systemd unit references in scripts
3. install, warm, smoke, doctor, provision, and uninstall flows
4. release payload staging and release-template references
5. upgrade cleanup of legacy `substrate-world-agent` units or binaries where required by the plan

Lane B may not edit:

- crate code
- workspace manifests
- local protocol validation logic
- docs or ADR narrative except tiny inline comments inside its own scripts if needed

### Lane C: Docs / ADR / Truth Ownership

Owns these surfaces after `G1`:

- `docs/**`
- `README.md`
- `AGENTS.md`
- `AGENT_ORCHESTRATION_GAP_MATRIX.md`
- `docs/project_management/adrs/**`

Lane C specifically owns:

1. upstream-vs-local boundary wording
2. examples using `substrate.agent.session`
3. docs and ADR references to `world-service*`
4. docs and ADR references to `transport-api*`
5. truth-table cleanup so no live doc implies local transport equals upstream UAA

Lane C may not edit:

- crate code
- scripts
- CI files
- release bundle assembly
- manifests

### Hotspot No-Split List

These surfaces must not be split across concurrent workers:

1. [Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.toml)
2. `dist-workspace.toml`
3. `crates/shell/**`
4. any file importing upstream `agent_api` and local transport crates in the same module
5. any validator, mapping, trace, routing, or persistence file that interprets the local protocol-family label
6. any code path that resolves the renamed `world-service` binary, unit, or socket

## Merge / Integration Order

Frozen integration order:

1. `P0-parent-contract-freeze-and-run-init`
2. `A1-protocol-label-cutover`
3. `A2-family-rename-foundation-cutover`
4. `G1-foundation-accept-and-parallel-launch`
5. parallel launch:
   - `B1-scripts-ci-release-cutover`
   - `C1-docs-adr-truth-convergence`
6. `G2-parallel-window-integration`
7. `P1-parent-parallel-integration`
8. `G3-validation-launch`
9. `P2-parent-validation-wall`
10. `G4-final-acceptance`
11. `P3-parent-closeout`

Integration rule inside the parallel window:

1. `B1` merges before `C1` unless the parent explicitly records that there is zero overlap.
2. Reason: script, release, and operator helper names are execution surfaces; docs and ADRs should match the final merged operator vocabulary.
3. Parent must diff the `B1` and `C1` file sets before merge. This is mandatory, not optional.
4. If `B1` and `C1` overlap unexpectedly on any file outside an explicitly parent-approved shared file list, parent immediately rejects the later lane handoff and records `REJECTED` pending replay.
5. If `C1` was prepared before the final `B1` operator-visible naming settled, parent must replay or rebase `C1` on the accepted `B1` tree before `C1` can become `ACCEPTED`.
6. Parent may not manually cherry-pick prose fragments from a rejected `C1` handoff into the authoritative tree. The lane must be replayed as a coherent docs pass.
7. If `B1` reopens any foundation hotspot, parent quarantines the lane and either narrows the brief or blocks the run.

## Validation Wall

Parent owns and runs the full validation wall on the merged tree only.

### Grep Gates

Zero-hit wall on live surfaces:

```bash
rg -n "world-agent|world_agent|substrate-world-agent|agent-api-types|agent-api-core|agent-api-client|agent_api_types|agent_api_core|agent_api_client|uaa\.agent\.session" \
  Cargo.toml dist-workspace.toml crates scripts docs .github dist README.md AGENTS.md config macos-hardening
```

Positive guardrails that must still succeed:

```bash
rg -n "agent_api\.run|agent_api\.session\.resume\.v1|agent_api\.session\.handle\.v1" crates docs config
rg -n "world-api" Cargo.toml crates docs
rg -n "world-service|substrate-world-service|transport-api-types|transport-api-core|transport-api-client|transport_api_types|transport_api_core|transport_api_client|substrate\.agent\.session" \
  Cargo.toml dist-workspace.toml crates scripts docs .github dist README.md AGENTS.md config macos-hardening
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

### Release Gates

```bash
./dist/scripts/assemble-release-bundles.sh
rg -n "world-service|substrate-world-service" dist/release-template.md dist/scripts/assemble-release-bundles.sh
```

### Fail-Closed Checks

1. `protocol: substrate.agent.session` validates.
2. Old `uaa.agent.session` live configs or persisted rows are rejected clearly after cutover.
3. World-required routing still hard-fails if renamed service discovery breaks.
4. No doctor, help, or remediation string points operators back to `substrate-world-agent`.

### Manual Validation Proof Points

Parent closeout must explicitly record proof of:

1. canonical config and docs use `protocol: substrate.agent.session`,
2. live trace identity emits `substrate.agent.session`,
3. local host↔world contract crates are `transport-api-*`,
4. local in-world daemon surfaces are `world-service*`,
5. `world-api` remains unchanged,
6. stale old-name live surfaces are gone outside the historical allowlist,
7. world-required rename drift still fails closed,
8. upstream `agent_api.*` wording remains unchanged.

## Blocked-Run Contract

If the run blocks, parent writes:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-uaa-boundary-and-naming-cleanup/blocked.json`

Required fields:

- `run_id`
- `authoritative_branch`
- `timestamp`
- `current_task_id`
- `gate_state`
- `summary`
- `stop_condition_id`
- `worker_lane`
- `blocking_files`
- `accepted_outputs`
- `quarantined_outputs`
- `next_required_parent_action`

Blocked-run rules:

1. `blocked.json` is parent-written only.
2. It is written exactly once at the stop point.
3. No further worker launches occur after it is written.
4. Existing worker outputs are either accepted and recorded or quarantined and named explicitly.

## Context-Control Rules For Parent And Workers

### Parent Rules

1. Parent keeps the canonical plan state in `.runs/**` only.
2. Parent is the only actor allowed to reopen a gate or reinterpret the rename contract.
3. Parent must keep the frozen rename matrix and live boundary visible in every worker brief.
4. Parent owns all cross-lane rebases, merges, and acceptance decisions.
5. Parent records GitNexus findings, handoffs, validation commands, and final acceptance artifacts.

### Worker Rules

1. Read [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md), this controller, and only the lane-relevant files before editing.
2. Do not widen scope beyond the lane boundary.
3. Run GitNexus impact analysis before editing symbols in the owned lane.
4. Stop and escalate on `HIGH` or `CRITICAL` impact.
5. Do not write `.runs/**`.
6. Do not edit [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) or [ORCH_PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md).
7. Handoffs must include:
   - changed files
   - commands run
   - exit codes
   - GitNexus impact summary
   - `gitnexus_detect_changes()` output
   - blockers or residual risks

### Context Hygiene

1. No worker loads broad unrelated surfaces just because the rename is repo-wide.
2. Lane A reads only the code, manifests, config, and code-adjacent tests it owns.
3. Lane B reads only scripts, CI, release, and packaging surfaces after `G1`.
4. Lane C reads only docs, ADRs, truth docs, and guidance surfaces after `G1`.
5. Archived or historical files are read only when needed to prove allowlist rationale.

## Acceptance / Completion Criteria

The run is complete only when all of the following are true on the merged tree:

1. `P0`, `A1`, `A2`, `B1`, `C1`, `P1`, `P2`, and `P3` are accepted by the parent.
2. The rename contract in `.runs/**` matches [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) exactly.
3. The foundation lane completed before the parallel lanes started.
4. The live-surface grep wall is green outside the historical allowlist.
5. Positive guardrails still show upstream `agent_api.*`, unchanged `world-api`, and the new canonical local names.
6. Cargo fmt, clippy, build, workspace tests, and targeted renamed-package tests are green.
7. Linux, Lima, and WSL operator surface gates are green or explicitly recorded as environment-blocked with no ambiguity about required follow-up.
8. Release bundle assembly is green and stages the renamed payloads.
9. Fail-closed checks prove stale `uaa.agent.session` and broken renamed service discovery do not silently succeed.
10. Code, scripts, release bundles, CI, docs, ADRs, fixtures, and operator guidance all tell one coherent boundary story.
11. Parent runs final `gitnexus_detect_changes()` and confirms the changed scope matches the plan.
12. Parent writes `final-summary.md` and marks `sentinels/RUN_COMPLETE`.

## Task Acceptance Checklist

| Task | Done means |
| --- | --- |
| `P0` | Rename contract, live boundary, historical allowlist, validation wall, branch map, lane ownership, and `.runs` artifact contract are frozen in parent-owned artifacts. |
| `A1` | `substrate.agent.session` is canonical on owned live code and config surfaces, stale `uaa.agent.session` fails closed, and owned tests or greps show the cutover. |
| `A2` | `world-service*` and `transport-api*` compile and test on owned code surfaces, `world-api` stays unchanged, and mixed-boundary imports remain clear. |
| `B1` | Scripts, CI, release, and operator helpers use the renamed service and transport families with no stale canonical `substrate-world-agent` guidance. |
| `C1` | Docs, ADRs, and truth docs teach the same boundary story as the merged code and script surfaces and use the new canonical names. |
| `P1` | Parent has integrated accepted parallel outputs, replayed or quarantined overlaps, and recorded the real integration history in `.runs/**`. |
| `P2` | Grep, cargo, operator, release, and fail-closed checks are executed and recorded against the merged validation candidate. |
| `P3` | Final GitNexus scope verification, final summary, terminal sentinel, and any residual-risk notes are written by the parent. |

## Assumptions

1. The authoritative execution branch remains `chore/uaa-boundary-and-naming-cleanup`.
2. Fresh worker worktrees can be created under `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/`.
3. GitNexus is available or can be refreshed before worker edits begin.
4. The repo can tolerate a direct live-name cutover with fail-closed handling and no supported migration layer for stale `uaa.agent.session` runtime state.
5. The final validation wall may require Linux, Lima, and WSL prerequisites already described in repo docs; if a platform environment is unavailable, the parent records that explicitly instead of silently skipping it.
6. Historical files outside the live-surface wall may retain old tokens only when the parent records the allowlist rationale.
7. This controller supersedes the stale async persistent-session orchestration topic previously present in this file.
