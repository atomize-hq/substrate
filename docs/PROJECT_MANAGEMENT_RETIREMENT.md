# Project Management Retirement

## Scope

This plan retires the legacy top-level planning system under `docs/project_management/`.

In scope:
- `docs/project_management/packs/**`
- `docs/project_management/system/**`
- `docs/project_management/intake/**`
- `docs/project_management/future/**`
- obsolete `_archived/**` planning material

Out of scope for this cut:
- `crates/gateway/docs/project_management/**`
- the ADR registry under `docs/project_management/adrs/**`

Current recommendation for ADRs:
- keep the full ADR registry in place during the pack retirement
- decide later whether to move or curate ADRs into a new stable `docs/adr/` tree

## Constraints

- `docs/project_management/packs/**` should be retired in one atomic cut, not wave-by-wave.
- Any pack document that is still treated as current source-of-truth must be extracted before the pack removal lands.
- CI, Make targets, smoke scripts, and Rust tests must stop depending on pack paths before the cut.
- Gateway-local planning docs are not being retired in this effort, but they currently link back into top-level packs and will need rewrites to avoid broken references.

## Current Dependency Classes

### 1. Tooling and automation that assume `packs/**` exists

- `Makefile`
  - planning scaffolding, validation, triad execution, archive helpers, and feature smoke dispatch all assume `docs/project_management/packs/...`
- `.github/workflows/feature-smoke.yml`
  - workflow input model and smoke-script discovery depend on feature pack directories and `tasks.json`
- `scripts/e2e/triad_e2e_phase1.sh`
- `scripts/e2e/triad_e2e_phase2.sh`
- `scripts/e2e/triad_e2e_all.sh`
- `scripts/ci/dispatch_feature_smoke.sh`
- `scripts/ci-audit/ci_audit.sh`
- `scripts/ci-audit/ci_audit_record.sh`
- `scripts/mac/smoke.sh`

Disposition:
- delete if triad/planning-pack orchestration is dead
- otherwise replace with non-pack infrastructure before the pack cut

### 2. Rust tests and code that hard-read pack markdown

- `crates/broker/src/tests.rs`
  - contract tests for `adr-0027-identity-tuple-policy-surface`
- `crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`
  - asserts world-deps provisioning references and contract wording
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
  - asserts successor compatibility, parity, and manual validation playbooks
- `crates/shell/tests/playbook_alignment.rs`
  - recursively scans `docs/project_management/packs/**/manual_testing_playbook.md`
- `crates/transport-api-types/src/lib.rs`
  - test reads a manual testing playbook under a draft pack

Disposition:
- rewrite when the source-of-truth doc survives in a stable home
- delete when the test only protected planning-pack process mechanics

### 3. Stable docs that still point at pack files

These are the highest-priority extraction blockers because they treat pack docs as current truth:

- `docs/TRACE.md`
  - references `active/world_process_exec_tracing_parity/SCHEMA.md`
  - references `active/world_process_exec_tracing_parity/PROTOCOL.md`
  - references `packs/PHASE_8_CROSS_CUTTING_DECISION_REGISTRY.md`
- `docs/WORLD.md`
  - references world-deps provisioning pack contracts
- `docs/CONFIGURATION.md`
  - references world-deps provisioning pack contracts
- `docs/COMMANDS.md`
  - references world-deps provisioning pack contracts
- `docs/reference/world/deps/README.md`
  - references world-deps provisioning pack contracts
- `docs/internals/world/deps.md`
  - references world-deps provisioning pack contracts
- `docs/reference/config/world.md`
  - references `world-deps-host-visible-hardening/WDH0-spec.md`
- `docs/internals/world/workspace_sync_filesystem_model.md`
  - references `world-sync` spec documents

Disposition:
- extract the normative content into `docs/reference/**`, `docs/contracts/**`, or `docs/internals/**`
- then rewrite these references to the new stable locations

### 4. Gateway docs that link to top-level packs

Although `crates/gateway/docs/project_management/**` is out of scope for retirement, it currently depends on top-level pack closeouts, seam docs, and evidence paths.

Notable dependency surfaces:
- `crates/gateway/docs/foundation/*.md`
- `crates/gateway/tests/fixtures/azure_kimi/*.json`
- `crates/gateway/docs/project_management/packs/active/**`

Disposition:
- rewrite foundation docs and fixture provenance to stable gateway docs or stable top-level contracts before removing top-level packs
- do not leave `crates/gateway/**` pointing at deleted top-level pack paths

## Extraction Targets Before The Atomic `packs/**` Cut

### A. Trace schema and protocol

Current pack sources:
- `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md`
- `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md`

Recommended destination:
- merge schema ownership into `docs/internals/trace/schema.md`
- merge protocol ownership into `docs/internals/trace/README.md` or a new sibling under `docs/internals/trace/`

Required follow-up:
- rewrite `docs/TRACE.md`
- rewrite `docs/internals/trace/README.md`
- remove any tests or comments that still name the pack path as authoritative

### B. World-deps provisioning contract

Current pack sources:
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
- `docs/project_management/packs/implemented/world-deps-apt-provisioning/contract.md`
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`

Recommended destination:
- move operator-facing contract material into `docs/reference/world/deps/README.md`
- move implementation/background material into `docs/internals/world/deps.md`
- if needed, add a dedicated stable doc under `docs/reference/world/deps/`

Required follow-up:
- rewrite `docs/WORLD.md`, `docs/CONFIGURATION.md`, and `docs/COMMANDS.md`
- rewrite `crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`

### C. ADR-0027 implemented policy contract/schema surfaces

Current pack sources:
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- draft pack docs under `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/`

Recommended destination:
- absorb stable policy contract wording into `docs/reference/policy/contract.md`
- absorb stable schema ownership into a new or expanded doc under `docs/reference/policy/`
- keep the ADR itself in `docs/project_management/adrs/**` for historical record

Required follow-up:
- rewrite `crates/broker/src/tests.rs`
- decide which draft-pack assertions remain valid as product contract tests versus planning-process checks to delete

### D. Workspace sync filesystem semantics

Current pack sources:
- `docs/project_management/packs/implemented/world-sync/filesystem-semantics-spec.md`
- `docs/project_management/packs/implemented/world-sync/WS2-spec.md`
- `docs/project_management/packs/implemented/world-sync/WS5-spec.md`

Recommended destination:
- absorb normative filesystem semantics into `docs/internals/world/workspace_sync_filesystem_model.md`
- create stable reference docs only if the content is operator-facing

Required follow-up:
- rewrite the internal world docs that currently cite those pack specs

### E. Host-visible hardening references

Current pack sources:
- `docs/project_management/packs/implemented/world-deps-host-visible-hardening/WDH0-spec.md`

Current code comments still reference old `next/` paths:
- `crates/world/src/guard.rs`
- `crates/common/src/world_exec_guard.rs`
- `crates/world-service/src/world_exec_guard.rs`
- `crates/shell/src/execution/routing/dispatch/tests/repl_persistent_session_client_fail_closed.rs`

Recommended destination:
- replace stale planning-path comments with stable doc anchors under `docs/reference/config/` or `docs/internals/world/`

Required follow-up:
- update comments and any tests relying on those comments as documentation anchors

## Delete Or Rewrite Checklist Before Pack Removal

### Delete candidates

- planning-pack scaffolding and triad automation in `Makefile` if the planning system is fully dead
- feature-smoke workflow and helper scripts if they only exist for planning packs
- `crates/shell/tests/playbook_alignment.rs` if no stable replacement playbook corpus is needed
- Python tests under `docs/project_management/system/scripts/planning/tests/` if the planning scripts are retired rather than relocated

### Rewrite candidates

- any Rust test that validates product behavior by asserting current docs mention the right contract
- any stable operator or internal doc that cites a pack path as canonical
- gateway docs and fixture provenance that would otherwise point at deleted top-level packs

## Atomic `packs/**` Retirement Procedure

1. Extract all surviving normative content out of `docs/project_management/packs/**`.
2. Rewrite all references in `docs/**`, `crates/**`, `scripts/**`, `.github/**`, and `Makefile`.
3. Remove or replace pack-based tests and automation.
4. Confirm `rg -n "docs/project_management/packs" .` returns only intentionally retained historical notes, if any.
5. Delete `docs/project_management/packs/**` in one cut.
6. Run formatting, clippy, tests, and a second reference scan.

## Post-Pack Cleanup

After `packs/**` is gone:
- remove `docs/project_management/system/**`
- remove `docs/project_management/intake/**`
- remove `docs/project_management/future/**`
- prune `_archived/**` to the minimum historical set worth keeping
- decide whether to keep the ADR registry where it is or move it into a non-project-management namespace

## First Safe Implementation Slice

The safest first execution slice is:

1. extract the world-deps provisioning contract into stable docs
2. rewrite the stable docs and WDAP1 Rust test to those new locations
3. re-run tests

Reason:
- it has a contained set of references
- it already has stable destination candidates
- it removes one of the largest live blockers before the atomic pack cut
