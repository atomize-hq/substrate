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

Current recommendation for ADRs:
- treat ADR curation into a stable `docs/adr/` tree as the next strategic milestone
- do not blindly move every ADR; curate them first
- expect some ADRs still labeled `draft` to be effectively implemented and needing promotion or
  restatement into the stable ADR tree
- the initial curation policy and first-cluster classification now live in:
  - `docs/adr/README.md`
  - `docs/adr/CURATION.md`

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
- repo-wide reference scan still finds non-pack-tree references to `docs/project_management/packs/**`
- the remaining refs are now concentrated in historical/root docs and in-progress
  `docs/project_management/**` material rather than live shell wrappers
- `llm-last-mile/**` and `FSE_PRE_PLANNING_*` are intentionally deferred for now and should not
  drive the next slice ordering
- the highest-value next namespace decision is ADR curation, because repeated backlink cleanup
  inside `docs/project_management/adrs/**` has diminishing returns while the long-term destination
  is now known to be `docs/adr/`
- the curation-policy question is now resolved for the first slice:
  - use `restate + supersede`, not a blind directory move
  - promote first-cluster keepers into `docs/adr/implemented/`
- the first-cluster promotion slice is now complete:
  - curated implemented ADRs exist for ADR-0027, ADR-0040, ADR-0041, ADR-0042, ADR-0043, and
    ADR-0046
  - stable gateway contract docs now point at curated implemented ADRs instead of the old draft
    project-management paths
  - live ADR-to-ADR prerequisite repointing for non-superseded current consumers is now complete
  - the remaining old first-cluster draft-path refs are confined to superseded predecessor ADRs
    ADR-0023, ADR-0024, and ADR-0025, plus queued draft ADR-0026
  - ADR-0023, ADR-0024, and ADR-0025 are now curated into `docs/adr/historical/`
  - ADR-0026 is now curated into `docs/adr/draft/` as queued work rather than historical-only

Validation already completed for the finished slices:
- `cargo test -p substrate-broker --lib -- --nocapture`
- targeted world-deps test rewrites are present in
  `crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`
- `cargo test -p shell --test world_deps_apt_fail_early_wdap1 -- --nocapture`
- `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
- `cargo test -p transport-api-types --lib -- --nocapture`
- scoped reference scans over stable docs and non-`project_management` gateway docs no longer show
  top-level pack backlinks for the completed ADR-0027 and gateway-foundation slices
- scoped reference scans over `crates/gateway/docs/project_management/**` no longer show
  top-level `docs/project_management/packs/**` or old `kimi-claude-adapter` pack backlinks
- scoped reference scans over `docs/reference/**` and `docs/internals/**` no longer show
  top-level world-sync or host-visible hardening pack backlinks
- scoped reference scans over `Makefile`, `.github/workflows/feature-smoke.yml`, and the targeted
  triad / smoke / CI helper scripts no longer show `docs/project_management/packs/**` or
  `tasks.json` assumptions
- scoped reference scans over `docs/contracts/gateway/*.md` and `docs/BACKLOG.md` no longer show
  pack-path backlinks for the stable contract and backlog surfaces cleaned in this slice
- scoped rewrites under `docs/project_management/adrs/**` now point ADR-0027 foundation references
  at `docs/reference/policy/{contract,schema}.md`, and the provisioning reconciliation note now
  points at stable world-deps references instead of implemented pack contracts
- the remaining direct pack-contract citations under `docs/project_management/adrs/**` are now
  concentrated in draft provisioning ADRs that still cite their own feature-pack contract surfaces
  (`ADR-0030`, `ADR-0033`)
- initial gateway-local manifest normalization under
  `crates/gateway/docs/project_management/**` now uses monorepo-correct
  `crates/gateway/docs/project_management/packs/**` refs in evidence payloads that previously
  looked like top-level `docs/project_management/packs/**` backlinks
- stable ADR scaffolding now exists under:
  - `docs/adr/`
  - `docs/adr/implemented/`
  - `docs/adr/draft/`
  - `docs/adr/historical/`
- the first ADR curation ledger now exists at `docs/adr/CURATION.md`
- the first promoted cluster has been classified as stable keepers:
  - ADR-0027
  - ADR-0040
  - ADR-0041
  - ADR-0042
  - ADR-0043
  - ADR-0046
- those first-cluster ADRs were classified as `draft_but_implemented` before promotion so their
  stable curated ADRs could normalize status away from `Draft`
- curated implemented ADR files now exist for that cluster under `docs/adr/implemented/`
- stable gateway contract verification docs now reference curated implemented ADR paths for:
  - ADR-0040
  - ADR-0041
- scoped scans over the current ADR consumer set no longer show old first-cluster draft ADR paths
- curated historical ADR files now exist for:
  - ADR-0023
  - ADR-0024
  - ADR-0025
- curated draft ADR files now exist for:
  - ADR-0026

## Current Dependency Classes

### 1. Tooling and automation that assume `packs/**` exists

Completed retirements/replacements:
- `Makefile`
  - retired the pack-driven planning, feature-smoke, scaffolding, and triad entrypoints behind
    explicit failure stubs instead of pack-path validation
- `.github/workflows/feature-smoke.yml`
  - replaced with a retirement workflow that fails fast instead of discovering feature-pack smoke
    scripts
- `scripts/e2e/triad_e2e_phase1.sh`
- `scripts/e2e/triad_e2e_phase2.sh`
- `scripts/e2e/triad_e2e_all.sh`
- `scripts/ci/dispatch_feature_smoke.sh`
- `scripts/ci-audit/ci_audit.sh`
- `scripts/ci-audit/ci_audit_record.sh`
- `scripts/mac/smoke.sh`
  - BEDPM installer conformance mode is now retired instead of shelling through a pack-owned smoke
    wrapper
- `tests/installers/pkg_manager_detection_smoke.sh`
  - removed the obsolete assertion that a pack-owned BEDPM smoke wrapper still exists
- `docs/contracts/gateway/backend-adapter-protocol.md`
- `docs/contracts/gateway/backend-adapter-schema.md`
- `docs/contracts/gateway/runtime-parity.md`
  - removed pack-spec backlinks from stable gateway contract verification surfaces
- `docs/BACKLOG.md`
  - removed the remaining direct pack-path context note from the world-sync backlog entry

Remaining dependency surface after the repo-wide scan:
- root and historical docs outside `docs/project_management/packs/**`
  - examples now concentrate in `llm-last-mile/**`, `FSE_PRE_PLANNING_*`, and archived planning
    notes that still cite pack paths
- in-progress `docs/project_management/**` planning and ADR material
  - these references still need explicit triage as either acceptable retained planning history or
    blockers that must be repointed before the atomic cut
- `crates/gateway/docs/project_management/**`
  - remaining hits are mostly gateway-local self-references and planning-pack internals; continue
    normalizing any monorepo-incorrect `docs/project_management/packs/**` payload refs as they are
    found

Disposition:
- classify root/historical docs as either intentional history or blockers that must be rewritten
  before the atomic cut
- keep narrowing stable/root surfaces first so the remaining scan result is dominated by clearly
  historical or intentionally retained planning material
- for the current slice ordering, defer `llm-last-mile/**` and `FSE_PRE_PLANNING_*` and focus on
  `docs/project_management/**` plus gateway-local `project_management/**` cleanup

### 2. Rust tests and code that hard-read pack markdown

Completed:
- `crates/broker/src/tests.rs`
  - planning-only slice/checkpoint/promote-pack assertions were deleted
- `crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`
  - markdown-coupled doc-contract assertions were deleted
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
  - markdown-coupled successor doc assertions were deleted
- `crates/shell/tests/playbook_alignment.rs`
  - deleted
- `crates/transport-api-types/src/lib.rs`
  - manual playbook evidence assertion was deleted

Remaining dependency surface:
- none currently identified in Rust tests under `crates/**`

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

- Python tests under `docs/project_management/system/scripts/planning/tests/` if the planning scripts are retired rather than relocated

### Rewrite candidates

- any stable operator or internal doc that cites a pack path as canonical
- any future gateway-local planning edits that reintroduce links to deleted top-level pack paths

## ADR Curation Milestone

Before the broader `docs/project_management/**` retirement can finish cleanly, curate the ADR set
that still matters into a stable `docs/adr/` home.

Recommended order:

1. Define ADR keep criteria.
   - Keep ADRs that are still normative, implemented, referenced by stable docs/code, or still
     define a current operator/runtime contract.
2. Classify the ADR set.
   - Use buckets:
     - keep as stable ADR
     - keep as historical only
     - superseded
     - draft-but-implemented
     - draft-and-actually-still-draft
3. Create the `docs/adr/` structure.
   - Likely:
     - `docs/adr/implemented`
     - `docs/adr/draft`
     - optional `docs/adr/historical`
4. Promote the real keepers.
   - Move or restate the curated ADRs into `docs/adr/**`.
   - Normalize statuses so “draft but actually implemented” is no longer ambiguous.
5. Repoint stable references to `docs/adr/**`.
   - Once stable docs and current contracts point at `docs/adr/**`, the remaining
     `docs/project_management/adrs/**` content becomes much easier to archive or delete.

Completed in this slice:

- defined keep criteria and migration policy in `docs/adr/README.md`
- created the stable `docs/adr/{implemented,draft,historical}/` structure
- classified the first contract-heavy ADR cluster in `docs/adr/CURATION.md`
- promoted the first contract-heavy ADR cluster into `docs/adr/implemented/`
- repointed stable gateway contract docs to the curated implemented ADR paths
- added relocation notes on the legacy project-management ADRs retained for compatibility
- repointed live ADR-to-ADR prerequisite links for the non-superseded current consumer set
- classified the superseded predecessor cluster (`ADR-0023` through `ADR-0025`)
- promoted that predecessor cluster into `docs/adr/historical/`
- reclassified ADR-0026 as queued and moved it into `docs/adr/draft/`
- added updated relocation notes on the retained predecessor and queued draft ADRs

## Recommended Resume Order

Use this order in the next session:

1. Reclassify and promote the next current ADR cluster.
   - The likely next slice is the orchestration / workflow cluster, including queued ADR-0026,
     not the already-curated historical predecessor ADRs.
2. Keep the provisioning ADR decision around ADR-0030 and ADR-0033 as its own narrower slice.
3. Continue narrowing the remaining `docs/project_management/**` dependency surface after the
   current ADR cluster decisions stop pointing stable readers at the retiring namespace.

## Resume Notes

- Do not start by deleting any pack directories.
- The first-cluster ADR promotion slice is complete.
- Live ADR-to-ADR prerequisite repointing is complete for the current consumer set, and the
  superseded predecessor cluster has been moved into `docs/adr/historical/` except ADR-0026,
  which is now treated as queued draft work under `docs/adr/draft/`.
- The next correct move is classification of the next current ADR cluster, not another broad
  stable-doc extraction pass or a return to the predecessor cluster.
- Treat the repo-wide `docs/project_management/**` cleanup as subordinate to that ADR curation
  milestone; otherwise you risk repeatedly repointing docs toward a namespace that is still meant
  to be retired.
- The curation-policy and first-cluster-classification questions are no longer open; use the
  recorded policy and ledger under `docs/adr/**` rather than re-deciding them in a later session.
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
