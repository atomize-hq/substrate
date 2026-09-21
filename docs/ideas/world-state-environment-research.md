# World state and environment: research and experiment brief

**Status:** Required follow-up recorded on 2026-09-20; research execution and experiments not started or dispatched. This is a proposed research scope and acceptance rubric, not a selected mechanism, implementation plan, or authority to alter runtime/security configuration.
**Intent:** [Core architecture dossier, SCA-12](../../SUBSTRATE_CORE_ARCHITECTURE_SESSION_DECISIONS_2026-09-20.md#sca-12--explicit-research-and-experiments-before-mechanism-selection).
**Discovery entry:** [Backlog](../BACKLOG.md#required-research--multi-world-state-and-environment).
**Consumers:** [B03 binding](../contracts/draft/lane-world-binding.md) and [B01–B05 handoff](../contracts/draft/collider-b01-b05-handoff.md).

## Objective and exit

Determine how to realize the confirmed multi-World Lane model without adopting old plans wholesale or treating existing mechanisms as already sufficient. Produce a comparison of alternatives, reproducible bounded experiments, and an evidence-backed recommendation, followed by a separately reviewed implementation plan.

Success means naming both what the recommended mechanism proves and what it does not. A snapshot demo is not policy enforcement, a package overlay is not a frozen project checkout, and a version-control branch is not a security boundary.

This brief is the single location for the detailed follow-up scope. The backlog is a pointer, the dossier records the decision, and contract drafts identify dependencies; none independently tracks experiment completion.

## Existing inputs and qualifications

The 2026-09-20 read-only investigation inspected current documents and selected Git-history sources. This is evidence discovery, not execution of the research/experiment program below.

| Input | Reuse / qualification |
|---|---|
| [Guest image backlog](../BACKLOG.md#deferred-product-follow-ups--guest-world-images--linux-guest_rootfs) | Existing distro expansion, provisioning-manager support, image import/pinning, lifecycle and possible VM work. Do not duplicate or silently reschedule these items. |
| [ADR-0009 curated draft](../adr/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md) | Queued guest-userspace and provisioning direction; not a landed runtime contract. Linux host-kernel guest-rootfs is distinct from a full VM or remote Lane backend. |
| [ADR-0010 curated draft](../adr/draft/ADR-0010-world-backend-contract-and-capability-divergence.md) | Proposed cross-backend capability/fail-closed contract. Preserve explicit capability differences instead of hiding them behind Lane names. |
| [Workspace filesystem model](../internals/world/workspace_sync_filesystem_model.md) and [sync surface](../reference/cli/workspace_sync.md) | Document live host-backed project files, conditional diff clearing, and best-effort apply. These are not proof of the newly required stable project baseline or transactional promotion. |
| [Internal Git model](../internals/world/workspace_internal_git_model.md) and backlog internal-Git-v2 item | Existing host checkpoint/history plus deferred richer history. History/rollback and isolation of live project state are separate properties. |
| [World deps](../reference/world/deps/README.md) | Separate inventory, enabled declarations, applied state, and provisioning. Manager support must be rechecked in its actual backend; historical non-APT backlog wording is not a current all-backend verdict. |
| [Multi-lane idea](multi-lane-worlds.md) | Useful isolation and identity options. Its session-owned Lane and singular allocation assumptions are superseded by confirmed intent; its mechanism recommendations are not inherited selections. |

The relevant current files were compared with the clean Linux checkout at `088f62d81c182b92cac404983b8bf512becc829c`; this did not audit source behavior or execute tests. Before research execution, rebind the authoritative source and affected owner contracts.

### Historical material that is no longer at its old filesystem path

Documentation hygiene commit `af88dc2562dd2abf5c579cff9a47e5079f973ba2` removed the planning-rich paths. They are recoverable at its parent, `38dc2838169f8b905554749693564cc52f58ac20`; do not restore them as current authority merely to read them:

- `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`
- `docs/project_management/adrs/draft/ADR-0010-world-backend-contract-and-capability-divergence.md`
- `docs/project_management/_archived/next/linux_guest_rootfs_backend/{spec.md,plan.md,decision_register.md,tasks.md}`
- `docs/project_management/future/INTERNAL_GIT.md`

Read-only recovery example:

```bash
git show 38dc2838169f8b905554749693564cc52f58ac20:docs/project_management/_archived/next/linux_guest_rootfs_backend/spec.md
```

The historical guest spec selects immutable guest images plus persistent writable overlays keyed by image and execution scope, generally one overlay per Workspace. That is useful precedent, but needs explicit reconciliation with multiple Worlds and per-World overrides. It does not establish an explicit-refresh-only snapshot of project mounts. The internal Git PRD leaves concurrent sessions and overlay integration open. Document keep/change/reject decisions for these assumptions rather than adopting their old acceptance labels as new approval.

## Required behavioral questions

1. **Project state:** how to maintain stable host-independent working baselines, track writes/deletes, coordinate same-World workers, refresh explicitly, promote selected changes, and optionally synchronize siblings.
2. **Environment state:** how immutable guest bases, mutable package/toolchain state, shared caches, Lane baselines, and per-World overrides compose without cross-World contamination. Configuration declarations and actually realized environments must remain distinguishable.
3. **Lifecycle and authority:** how replacement/reassociation preserves diffs, provenance and active/in-flight work, validates destination policy, and recovers interrupted operations. Stable content is not permission to retain revoked access.

Confirmed limits stay fixed: multiple Worlds per Lane, one current Lane per associated World, independent advancement, no mandatory sibling catch-up, restricted-by-default host orchestration, and V1 per-World dependency/configuration overrides. The precise mechanism, performance thresholds, reset/retention details, and active-transition protocol are not yet selected.

The existing guest-rootfs initiative retains a separate scope/delivery decision from the newly deferred alternative-Lane-backing feature. This brief neither promotes guest-rootfs into V1 nor automatically defers it with remote/VM/container Lane backing. It does not change SCA-11 or remove existing platform backends.

## Proposed bounded sequence

| Stage | Work and output | Exit / handoff |
|---|---|---|
| R0 — Reconcile evidence | Refresh relevant owners/source, recover only the named historical inputs, and map each old decision to keep/change/reject/open under current intent. | Source-bound requirement and gap map; no global reread or old-plan restoration. |
| R1 — Compare candidates | Research current primary documentation for a bounded shortlist: permission-partitioned shared state, Git worktree/snapshot variants, viable alternative versioning/snapshot approaches, and rootfs/package-state layering. Record versions and limitations. | Comparison matrix; explicit shortlist of at most three coherent designs and a falsifiable reason to experiment with each. Three is a proposed research budget, not a product constraint. |
| R2 — Execute selected experiments | Approve exact commands, fixtures, privileges, resource ceilings, allowed paths and cleanup before running each disposable Linux experiment. | Reproducible evidence for the matrix below; classify unsupported or failed claims honestly. No production installation or real Workspace migration. |
| R3 — Decide and plan | Recommend one design or reject all; describe surviving gaps, costs and capability limits. Produce an owner-reviewed contract delta and a bounded implementation plan/test plan. | Explicit review/selection before implementation. If no design passes, narrow the next experiment or raise the exact product conflict; do not silently reduce requirements. |

R0/R1 can inform semantic contract review. R2 requires its own execution authorization; the present request only records the follow-up. Research outputs should be linked here once they exist, without pre-creating a competing status ledger. A later implementation plan follows the repository's applicable planning workflow, not this brief's hypothetical implementation file list.

## Proposed experiment matrix

| Experiment | Acceptance observation |
|---|---|
| Host isolation | After World preparation, host edit/create/delete does not change its project view until explicit refresh. World writes do not mutate host or sibling state. Record baseline and before/after content evidence. |
| Same-World collaboration | Multiple workers obey separate see/read/write scopes; disjoint edits survive; overlapping edits and bypass attempts are detected/refused or follow an explicit coordination rule. Checkout separation alone does not prove enforcement. |
| Per-World environments | Two Worlds inherit one Lane baseline and use different declared dependency/configuration overrides without changing each other's realized state or the host OS. Readiness accurately reports unsupported combinations. |
| Refresh and selected promotion | Conflicting host/source/destination edits are surfaced; protected paths remain protected; only selected, authorized changes advance. No unsupported all-or-nothing/rollback claim. |
| Optional sibling propagation | One World promotes while another continues unchanged; an explicit later sync/rebase/merge incorporates the chosen revision or reports conflicts. Availability of updates alone does not invalidate the sibling. |
| Association/replacement race | A submission racing a move/replacement either has a valid exact binding or is refused/reconciled. No split ownership, lost diffs, hidden second launch, or relabeled accepted-work history. |
| Interruption and recovery | Kill/restart experiments at named publication/apply boundaries preserve state custody and distinguish committed, partial, and unknown effects; retries do not silently repeat effects or discard pending changes. |
| Resource and operational cost | Measure preparation/clone time, write amplification/storage, refresh/integration time and cleanup under small and representative fixtures. Agree workload/thresholds before accepting performance; do not invent results or thresholds now. |

Experiments may isolate dimensions rather than build an entire backend. A filesystem prototype cannot certify full agent mediation or credential-bearing concurrency. Failure-injection and privileged work belong only in an approved disposable environment; never weaken the existing E3 exclusion, credential custody, or host package-management safeguards to make a demonstration pass.

## Deliverables and acceptance into planning

- Evidence-indexed options matrix: guarantees, known failures, security boundary, state persistence, capabilities, portability, measured cost, and adoption/operational constraints.
- Exact scripts/commands, fixture revisions, host/backend identity, outputs, negative cases, cleanup evidence, and a clear untested/unsupported list for each experiment.
- Recommendation stating what is retained or replaced from older plans, how multi-World identity and per-World state are keyed, and which safety guarantees require further work.
- Explicit dependency map to B03 state/association operations and B05 environment realization; owner-contract amendments and later implementation slices only after selection.

## Relationship to B01–B05

Semantic contract work can continue: exact identities, authorization, acceptance/reconciliation, observation/cancel outcomes, typed context ownership, and revision-aware declaration edits do not require choosing Git, a rootfs mechanism, or a merge engine.

Mechanism-dependent promises cannot be marked implemented/proven until supported by the experiments and then production-path validation. This includes stable project baselines, actual per-World dependency isolation, safe stateful moves/replacement, and promotion/propagation behavior. The one-task/Workspace-declaration proof must not stand in for complete V1, but the entire research program is not automatically a predecessor to every narrow consumer semantic contract.
