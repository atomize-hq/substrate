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

## Current State

Completed extraction/rewrite slices:
- trace schema/protocol ownership moved into:
  - `docs/internals/trace/schema.md`
  - `docs/internals/trace/protocol.md`
  - with stable trace docs repointed from pack-backed sources
- world-deps provisioning contract moved into:
  - `docs/reference/world/deps/provisioning.md`
  - with `docs/WORLD.md`, `docs/CONFIGURATION.md`, `docs/COMMANDS.md`,
    `docs/reference/world/deps/README.md`, and `docs/internals/world/deps.md` repointed
- ADR-0027 / ADR-0043 stable policy references moved into:
  - `docs/reference/policy/contract.md`
  - `docs/reference/policy/schema.md`
  - `docs/reference/policy/tuple_constraints.md`
  - with stable gateway contracts repointed and `crates/broker/src/tests.rs` rewritten to lock
    stable docs instead of pack docs
- gateway backlink cleanup completed for:
  - `crates/gateway/docs/foundation/**`
  - `crates/gateway/tests/fixtures/azure_kimi/**`
  - `crates/gateway/docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md`
- gateway-local planning backlink cleanup completed for:
  - `crates/gateway/docs/project_management/packs/active/**`
- workspace-sync filesystem semantics completed in:
  - `docs/internals/world/workspace_sync_filesystem_model.md`
  - with stable path, diff, direction, conflict, safety-rail, and clear semantics absorbed out of
    the pack specs
- stale host-visible hardening and persistent-session planning anchors were already cleaned in
  earlier slices

Still remaining before the atomic top-level `packs/**` removal:
- planning automation and workflow machinery still assume `docs/project_management/packs/**`
- several tests outside `crates/broker/src/tests.rs` still read pack docs directly

Validation already completed for the finished slices:
- `cargo test -p substrate-broker --lib -- --nocapture`
- targeted world-deps test rewrites are present in
  `crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`
- scoped reference scans over stable docs and non-`project_management` gateway docs no longer show
  top-level pack backlinks for the completed ADR-0027 and gateway-foundation slices
- scoped reference scans over `crates/gateway/docs/project_management/**` no longer show
  top-level `docs/project_management/packs/**` or old `kimi-claude-adapter` pack backlinks
- scoped reference scans over `docs/reference/**` and `docs/internals/**` no longer show
  top-level world-sync or host-visible hardening pack backlinks

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

- `crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`
  - rewritten to the stable world-deps provisioning docs; keep as an example of the desired end
    state for other tests
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
  - asserts successor compatibility, parity, and manual validation playbooks
- `crates/shell/tests/playbook_alignment.rs`
  - recursively scans `docs/project_management/packs/**/manual_testing_playbook.md`
- `crates/transport-api-types/src/lib.rs`
  - test reads a manual testing playbook under a draft pack

Completed:
- `crates/broker/src/tests.rs`
  - ADR-0027 contract tests now lock stable policy docs under `docs/reference/policy/**`
  - planning-only slice/checkpoint/promote-pack assertions were deleted rather than migrated

Disposition:
- rewrite when the source-of-truth doc survives in a stable home
- delete when the test only protected planning-pack process mechanics

### 3. Stable docs that still point at pack files

Completed stable-doc rewrites:
- `docs/TRACE.md`
- `docs/WORLD.md`
- `docs/CONFIGURATION.md`
- `docs/COMMANDS.md`
- `docs/reference/world/deps/README.md`
- `docs/internals/world/deps.md`
- `docs/internals/world/workspace_sync_filesystem_model.md`

Remaining stable-doc blockers that still treat pack docs as current truth:
- none currently identified under `docs/reference/**` or `docs/internals/**`

Disposition:
- keep stable-doc scans in the validation loop so pack backlinks are not reintroduced while the
  remaining tests and automation are retired

### 4. Gateway docs that link to top-level packs

Although `crates/gateway/docs/project_management/**` is out of scope for retirement, it currently depends on top-level pack closeouts, seam docs, and evidence paths.

Completed rewrites:
- `crates/gateway/docs/foundation/*.md`
- `crates/gateway/tests/fixtures/azure_kimi/*.json`
- `crates/gateway/docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md`
- `crates/gateway/docs/project_management/packs/active/**`

Remaining dependency surface:
- none found in the current gateway-local markdown scan; this tree remains out of scope for
  deletion but is no longer a top-level pack backlink blocker

Disposition:
- keep gateway-local planning docs in place
- do not let future edits reintroduce `docs/project_management/packs/**` backlinks from
  `crates/gateway/**`

## Extraction Targets Before The Atomic `packs/**` Cut

### A. Trace schema and protocol

Current pack sources:
- `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md`
- `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md`

Stable destinations now in place:
- `docs/internals/trace/schema.md`
- `docs/internals/trace/protocol.md`

Completed follow-up:
- rewrote `docs/TRACE.md`
- rewrote `docs/internals/trace/README.md`
- removed stable-doc dependency on pack-backed schema/protocol ownership

Remaining follow-up:
- keep ADR-0028 in `docs/project_management/adrs/**` for historical record
- clean any downstream ADR/planning references separately from the stable trace surface itself

### B. World-deps provisioning contract

Current pack sources:
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
- `docs/project_management/packs/implemented/world-deps-apt-provisioning/contract.md`
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`

Stable destinations now in place:
- `docs/reference/world/deps/provisioning.md`
- `docs/reference/world/deps/README.md`
- `docs/internals/world/deps.md`

Completed follow-up:
- rewrote `docs/WORLD.md`, `docs/CONFIGURATION.md`, and `docs/COMMANDS.md`
- rewrote `crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`

Remaining follow-up:
- keep downstream ADR/planning references separate from the stable provisioning contract
- verify any still-dirty worktree state in `crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`
  before using it as a resume point in a new session

### C. ADR-0027 implemented policy contract/schema surfaces

Current pack sources:
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- draft pack docs under `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/`

Stable destinations now in place:
- `docs/reference/policy/contract.md`
- `docs/reference/policy/schema.md`
- `docs/reference/policy/tuple_constraints.md`

Completed follow-up:
- rewrote `crates/broker/src/tests.rs` to lock stable policy references instead of pack docs
- deleted planning-only broker assertions that were validating slice specs, checkpoint wiring, and promotion packaging rather than product contract

Remaining follow-up:
- keep the ADRs in `docs/project_management/adrs/**` for historical record until the broader ADR namespace move is addressed
- repoint downstream ADR and planning references as separate retirement slices rather than treating them as blockers for the stable policy reference itself

### D. Workspace sync filesystem semantics

Current pack sources:
- `docs/project_management/packs/implemented/world-sync/filesystem-semantics-spec.md`
- `docs/project_management/packs/implemented/world-sync/WS2-spec.md`
- `docs/project_management/packs/implemented/world-sync/WS5-spec.md`

Stable destination now in place:
- `docs/internals/world/workspace_sync_filesystem_model.md`

Completed follow-up:
- absorbed the stable Linux filesystem semantics into
  `docs/internals/world/workspace_sync_filesystem_model.md`
- removed the internal-doc dependency on the world-sync pack specs
- kept the operator-facing surface in `docs/reference/cli/workspace_sync.md` rather than creating
  another stable reference page

Remaining follow-up:
- none for the stable-doc dependency itself; the remaining blockers now sit in tests and
  automation rather than `docs/reference/**` or `docs/internals/**`

## Delete Or Rewrite Checklist Before Pack Removal

### Delete candidates

- planning-pack scaffolding and triad automation in `Makefile` if the planning system is fully dead
- feature-smoke workflow and helper scripts if they only exist for planning packs
- `crates/shell/tests/playbook_alignment.rs` if no stable replacement playbook corpus is needed
- Python tests under `docs/project_management/system/scripts/planning/tests/` if the planning scripts are retired rather than relocated

### Rewrite candidates

- any Rust test that validates product behavior by asserting current docs mention the right contract
- any stable operator or internal doc that cites a pack path as canonical
- any future gateway-local planning edits that reintroduce links to deleted top-level pack paths

## Recommended Resume Order

Use this order in the next session:

1. Triage the remaining pack-reading tests:
   - `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
   - `crates/shell/tests/playbook_alignment.rs`
   - `crates/transport-api-types/src/lib.rs`
   - Goal: rewrite to stable docs where appropriate; delete planning-process-only assertions.
2. Remove or replace planning automation and workflow dependencies:
   - `Makefile`
   - `.github/workflows/feature-smoke.yml`
   - triad / smoke / CI helper scripts
3. Re-run a repo-wide reference scan.
   - Goal: confirm what still points at `docs/project_management/packs/**` before attempting the
     atomic cut.

## Resume Notes

- Do not start by deleting any pack directories.
- The next correct move is still extraction/rewrite work.
- The top-level `packs/**` tree must be removed in one cut only after:
  - stable docs are repointed,
  - pack-reading tests are rewritten or deleted,
  - automation stops assuming pack directories exist,
  - and gateway-local backlinks have been cleaned far enough to avoid broken references.

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

## Historical Note

The original first safe implementation slice was the world-deps provisioning extraction. That
slice is now complete and should not be treated as the next step when resuming this work.
