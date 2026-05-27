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
- the ADR destination question is now resolved:
  - the stable registry is `docs/adr/**`
  - repeated backlink cleanup inside `docs/project_management/adrs/**` now has diminishing returns
    unless a live stable reader still points there
- the curation-policy question is now resolved:
  - use `restate + supersede`, not a blind directory move
  - promote current stable keepers into `docs/adr/{implemented,draft,historical}/`
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
  - the orchestration/workflow batch is now curated:
    - implemented ADRs: ADR-0017, ADR-0028, ADR-0047
    - queued draft ADRs: ADR-0021, ADR-0022, ADR-0026, ADR-0029, ADR-0044, ADR-0045
  - the remaining current ADR tail is now curated:
    - implemented ADR: ADR-0016
    - queued draft ADRs: ADR-0019, ADR-0020

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
- the provisioning ADR pair `ADR-0030` and `ADR-0033` is now curated into
  `docs/adr/implemented/`, and current planning readers have been repointed toward the curated
  namespace
- the world/runtime foundation ADR batch is now curated into `docs/adr/implemented/`:
  - ADR-0004
  - ADR-0007
  - ADR-0014
  - ADR-0015
  - ADR-0018
- the world-deps predecessor pair is now split correctly:
  - ADR-0002 is curated into `docs/adr/historical/` as stale historical framing
  - ADR-0011 is curated into `docs/adr/implemented/` as the current inventory + enabled-set
    contract
- the remaining installer/diagnostics/backend ADR tail is now classified into `docs/adr/**`:
  - implemented ADRs: ADR-0031, ADR-0032, ADR-0034, ADR-0035, ADR-0036, ADR-0037, ADR-0038
  - curated drafts: ADR-0009, ADR-0010, ADR-0039, ADR-2026-02-13 macOS Virtualization.framework
- the remaining stable-doc ADR path hits outside `docs/project_management/**` have been repointed:
  - `docs/internals/config/world_root_and_caging.md` now points at curated ADR-0018
  - `docs/TRACE.md` and `docs/internals/trace/schema.md` now point at curated ADR-0028
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
- curated implemented ADR files now also exist for:
  - ADR-0004
  - ADR-0007
  - ADR-0011
  - ADR-0014
  - ADR-0015
  - ADR-0018
  - ADR-0016
  - ADR-0017
  - ADR-0028
  - ADR-0031
  - ADR-0032
  - ADR-0034
  - ADR-0035
  - ADR-0036
  - ADR-0037
  - ADR-0038
  - ADR-0047
- curated historical ADR files now exist for:
  - ADR-0002
  - ADR-0023
  - ADR-0024
  - ADR-0025
- curated draft ADR files now exist for:
  - ADR-0009
  - ADR-0010
  - ADR-0019
  - ADR-0020
  - ADR-0026
  - ADR-0021
  - ADR-0022
  - ADR-0029
  - ADR-0039
  - ADR-0044
  - ADR-0045
  - ADR-2026-02-13 macOS Virtualization.framework

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

### E. Remaining Pack Contract and Schema Triage

The remaining `contract.md` / `SCHEMA.md` / `*schema-spec.md` files under
`docs/project_management/packs/**` are not all equal. Before the atomic `packs/**` cut, treat them
in three buckets:

#### Already absorbed into stable docs; pack files can retire after backlink cleanup

- trace parity pack surfaces:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/contract.md`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md`
  - stable home:
    - `docs/internals/trace/schema.md`
    - `docs/internals/trace/protocol.md`
    - `docs/adr/implemented/ADR-0028-in-world-process-execution-tracing-parity.md`
- world-deps provisioning and package-contract pack surfaces:
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - `docs/project_management/packs/implemented/world-deps-apt-provisioning/contract.md`
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
  - `docs/project_management/packs/implemented/add-non-apt-system-package-provisioning-support/contract.md`
  - `docs/project_management/packs/implemented/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md`
  - stable home:
    - `docs/reference/world/deps/README.md`
    - `docs/reference/world/deps/provisioning.md`
    - `docs/internals/world/deps.md`
    - `docs/adr/implemented/ADR-0011-world-deps-packages-bundles-contract.md`
    - `docs/adr/implemented/ADR-0030-provisioning-time-system-package-mutation-for-world-deps.md`
    - `docs/adr/implemented/ADR-0033-manager-aware-system-package-provisioning-for-world-deps.md`
- policy / tuple / gateway contract pack surfaces:
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/{contract.md,SCHEMA.md}`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/{contract.md,tuple-policy-schema-spec.md}`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/{contract.md,identity-tuple-schema-spec.md}`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/{contract.md,gateway-status-schema-spec.md}`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/{contract.md,gateway-backend-adapter-schema-spec.md}`
  - stable home:
    - `docs/reference/policy/{contract.md,schema.md,tuple_constraints.md}`
    - `docs/contracts/gateway/{operator-contract.md,status-schema.md,policy-evaluation.md,backend-adapter-selection.md,backend-adapter-protocol.md,backend-adapter-schema.md,runtime-parity.md}`
    - `docs/adr/implemented/ADR-0027-llm-and-agent-config-policy-surface.md`
    - `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
    - `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`
    - `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
    - `docs/adr/implemented/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
    - `docs/adr/implemented/ADR-0046-gateway-backend-selection-runtime-integration.md`
- workspace sync pack contract:
  - `docs/project_management/packs/implemented/world-sync/contract.md`
  - stable home:
    - `docs/reference/cli/workspace_sync.md`
    - `docs/internals/world/workspace_sync_filesystem_model.md`
- installer detection / replay attribution contract surfaces that are already sufficiently covered:
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/contract.md`
  - `docs/project_management/packs/implemented/world-disabled-reason-attribution/contract.md`
  - stable home:
    - `docs/INSTALLATION.md`
    - `docs/reference/env/contract.md`
    - `docs/REPLAY.md`
    - `docs/adr/implemented/ADR-0031-best-effort-linux-distro-package-manager-discovery-during-install.md`
    - `docs/adr/implemented/ADR-0038-replay-attribute-why-world-is-disabled-in-warnings.md`

#### Preserve or move before deleting `packs/**`

These files still carry stable contract detail that is only partially summarized elsewhere:

- `docs/project_management/packs/implemented/agent-hub-concurrent-execution-output-routing/contract.md`
  - preserve before deletion
  - recommended destination: new stable contract doc under `docs/contracts/` for REPL structured
    output routing, including `repl.max_pty_buffered_lines` and suppression-summary behavior
- `docs/project_management/packs/implemented/agent-hub-concurrent-execution-output-routing/agent-hub-event-envelope-schema-spec.md`
  - preserve before deletion
  - recommended destination: new stable event-envelope schema doc under `docs/contracts/`
    because `TRACE.md` discusses `agent_event` rows but does not fully replace the envelope schema
- `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
  - preserve before deletion
  - recommended destination: new stable `install_state.json` schema doc under `docs/contracts/`
    because `docs/INSTALLATION.md` lists the fields but does not capture the full additive schema
    and compatibility rules
- `docs/project_management/packs/implemented/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`
  - preserve before deletion
  - recommended destination: new stable diagnostics JSON contract doc under `docs/contracts/`
    because `docs/USAGE.md` summarizes `.shim.world.status` and `.shim.world_deps.status` but does
    not replace the full machine-readable schema and omission rules

#### Draft-pack contracts that should be folded into draft ADRs rather than promoted to stable contracts

- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/contract.md`
  - queued work; if the pack tree is removed before implementation, fold the normative CLI copy
    into `docs/adr/draft/ADR-0019-warn-config-global-show-when-workspace-config-overrides.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
  - still referenced by `llm-last-mile/**`; if the pack tree is removed, fold any still-normative
    contract text into `docs/adr/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
    before deleting the pack path

No other current pack-local `contract.md` / schema file has yet shown a stronger stable-home need
than the curated ADR plus existing reference/contract docs above.

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
- classified the orchestration/workflow ADR batch
- promoted implemented ADRs `ADR-0017`, `ADR-0028`, and `ADR-0047` into `docs/adr/implemented/`
- promoted queued ADRs `ADR-0021`, `ADR-0022`, `ADR-0026`, `ADR-0029`, `ADR-0044`, and
  `ADR-0045` into `docs/adr/draft/`
- added relocation notes on the retained project-management copies for that batch
- classified the remaining current ADR tail
- promoted implemented ADR `ADR-0016` into `docs/adr/implemented/`
- promoted queued ADRs `ADR-0019` and `ADR-0020` into `docs/adr/draft/`
- repointed current ADR references from `ADR-0016` and `ADR-0020` toward the curated namespace
- classified the provisioning ADR pair
- promoted implemented ADRs `ADR-0030` and `ADR-0033` into `docs/adr/implemented/`
- repointed current provisioning ADR references toward the curated namespace
- classified the config/policy foundation ADR batch
- promoted implemented ADRs `ADR-0003`, `ADR-0005`, `ADR-0006`, `ADR-0008`, `ADR-0012`, and
  `ADR-0013` into `docs/adr/implemented/`
- repointed stable config/env readers and current draft ADR references toward the curated namespace

## Recommended Resume Order

Use this order in the next session:

1. Continue narrowing the remaining `docs/project_management/**` dependency surface now that the
   ADR registry stops being a destination question.
2. Keep current stable-reader rewrites ahead of broad historical cleanup.
3. Treat retained `docs/project_management/adrs/**` files as compatibility/history stubs unless a
   live stable consumer still points at them.
4. Do not reopen the completed ADR clusters unless a remaining stable consumer still points at the
   old copies.

## Resume Notes

- Do not start by deleting any pack directories.
- The first-cluster ADR promotion slice is complete.
- Live ADR-to-ADR prerequisite repointing is complete for the current consumer set, and the
  superseded predecessor cluster has been moved into `docs/adr/historical/` except ADR-0026,
  which is now treated as queued draft work under `docs/adr/draft/`.
- The orchestration/workflow batch is now classified and promoted.
- The remaining current ADR tail is now classified and promoted.
- The dedicated provisioning ADR slice (`ADR-0030` / `ADR-0033`) is now complete.
- The config/policy foundation ADR slice (`ADR-0003`, `ADR-0005`, `ADR-0006`, `ADR-0008`,
  `ADR-0012`, `ADR-0013`) is now complete.
- The world/runtime foundation ADR slice (`ADR-0004`, `ADR-0007`, `ADR-0014`, `ADR-0015`,
  `ADR-0018`) is now complete.
- The remaining installer/diagnostics/backend ADR tail is now classified:
  - implemented: `ADR-0031`, `ADR-0032`, `ADR-0034`, `ADR-0035`, `ADR-0036`, `ADR-0037`,
    `ADR-0038`
  - draft: `ADR-0009`, `ADR-0010`, `ADR-0039`, `ADR-2026-02-13 macOS World backend via
    Virtualization.framework`
- The ADR registry is now fully represented under `docs/adr/**`; the remaining work is dependency
  cleanup and eventual historical pruning, not open-ended ADR destination decisions.
- Treat the repo-wide `docs/project_management/**` cleanup as the follow-on to the completed ADR
  curation milestone; otherwise you risk mixing stable-reader rewrites with broad historical
  pruning before the dependency surface is clearly bounded.
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
