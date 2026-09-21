# B03 proposal: Lane, World association, and accepted-work binding

**Status:** Draft for owner-contract review, 2026-09-20. Not an accepted API, canonical-owner replacement, implementation dispatch, or runtime-readiness claim.
**Purpose:** Make the confirmed multi-World Lane model precise enough for Collider to display and control one exact accepted task without guessing its execution destination.
**Companion:** [B01–B05 handoff](collider-b01-b05-handoff.md), including evidence pins, shared boundaries, and remaining decisions.
**Intent source:** [Core architecture dossier](../../../SUBSTRATE_CORE_ARCHITECTURE_SESSION_DECISIONS_2026-09-20.md), especially SCA-02–05 and SCA-07–12. The dossier retains confirmed intent; this file proposes its contract expression.

## 1. Scope and authority

The narrow consumer needs authenticated Lane/World selection, a validated association, immutable execution provenance, and an authorized read of that provenance. It does not need a merge engine, automatic routing, remote backing, or an always-running lane orchestrator.

This proposal does not add fields to an existing V1 record. Operation labels and semantic field lists below are design vocabulary, not callable APIs or selected serialization. Existing strict decoding, reference validation, and owner boundaries remain controlling until explicitly amended.

The current [authority map](../../../llm-last-mile/runtime-refactor/architecture/authority-map.md) assigns session/world authority to HSA, physical realization to the runtime/backend, acceptance to the Receipt Registry, observation/terminal truth to the Supervisor, and worker lifecycle to RetainedWorkerRuntime. None becomes a second Lane scheduler through this proposal.

**Responsibility to establish:** Substrate needs a Workspace-scoped authority for Lane identity, current World association, association history, and validated changes to that association. Its module/persistence placement is unresolved. This is a proposed logical responsibility, not a new crate, registry implementation, or claim that HSA already owns multi-World Lane state.

## 2. Confirmed product constraints

| Constraint | Source |
|---|---|
| A Lane is a durable, explicitly created Workspace resource, independent of orchestration and worker sessions. | SCA-03 |
| One Lane can contain multiple Worlds; an associated World has one current Lane at a time. No one-World-per-worker requirement. | SCA-03/04 |
| Worlds contain meaningful state and diffs. Reassociation, replacement, and promotion must account for state and active work; moving is not merging. | SCA-03/05 |
| Host changes do not silently alter active World working state. Refresh and integration are explicit, conflict-aware gates. | SCA-05 |
| Selected Worlds/changes can advance independently. Sibling propagation is optional unless an applicable milestone requires it. | SCA-05 |
| Worlds inherit a Lane baseline; explicit per-World dependency/configuration overrides are V1. Policy overrides cannot silently widen parent authority. | SCA-02/10 |
| Workspace and Lane orchestrators are host-bound; workers/tasks are World-bound. Lane-orchestrator V1 delivery remains undecided. | SCA-01/07 |
| Alternative SSH/system/VM-image/container Lane backing is future scope, not V1; existing platform backends are not removed. | SCA-11 |

Durable identity does not by itself promise reboot recovery, a continuously running process, a particular checkout topology, or unrestricted execution concurrency.

## 3. Proposed identity and reference model

Keep these meanings distinct even if an implementation can store some together:

| Identity/reference | Meaning and owner |
|---|---|
| Workspace reference | Authorized Workspace identity; not a directory string supplied as authority. |
| Lane reference | Stable identity within its Workspace. A display name is not an authorization token. |
| World realization reference | Exact backend World identity and generation/realization discriminator validated by existing World owners. Do not repurpose generation to mean a sibling World or Lane. |
| Association reference/revision | Owner-issued evidence that this World is associated with this Lane at a specified revision. Changes create history, not retroactive relabeling. |
| Work/acceptance reference | Exact accepted task/turn identity from the existing acceptance owner; not a Lane, worker, provider session, or UI segment ID. |
| Orchestration/worker references | Coordinator and worker identities, separately authorized for their roles. Their lifetime does not define Lane lifetime. |
| Configuration provenance | References to baseline, explicit overrides, effective desired configuration, and applied environment evidence; not one interchangeable hash or permission grant. |

References are opaque and owner-validated. Reuse existing exact ref and store conventions where applicable; do not expose private leases, credentials, resume handles, or internal capabilities to Collider. Whether a World can be unassociated before admission remains outside this proposal's selected path, which requires an existing associated World.

### 3.1 Review candidate: separate membership, association, and acceptance revisions

Use an owner-qualified Workspace/Lane reference and an exact backend World reference. A bare `world_id` is insufficient outside its owning scope; preserve the existing store/backend/world/generation qualification wherever the owner requires it. Restoring or copying storage must not silently mint a second authority for the same references.

Recommended semantic shapes (not serialized records or new public type names):

```text
DestinationExpectation = {
  workspace_ref, lane_ref, exact_world_ref, expected_world_association_revision
}
AcceptedBinding = {
  exact_acceptance_ref, destination_at_acceptance
}
```

`destination_at_acceptance` contains the same four identity/revision meanings as `DestinationExpectation`, validated by owners rather than copied from the client. `exact_acceptance_ref` retains the existing authority-store/acceptance-record/revision qualification; the authenticated join resolves the exact task/run, request, session, and caller. Names here do not replace the existing ref schemas.

- **Lane membership revision** identifies the association-list cut; adding an unrelated sibling World may change it.
- **World association revision** identifies this World's current Lane association. A move A→B→A must not make the old A selector valid again. Reject stale revisions even when names/IDs appear unchanged. Numeric encoding and revision allocation remain open.
- **Acceptance/observation revisions** belong to the existing work owners, not either association revision. A new observation or renamed Lane does not rewrite `AcceptedBinding`.

Bind admission to the selected World's association revision, not a Lane-wide display revision. This avoids rejecting unrelated work merely because another World joined the Lane. Separately validate current policy/configuration requirements; an unchanged association revision is not proof that they remain valid. List-cursor invalidation and admission validity are different decisions.

## 4. Proposed operation semantics

### 4.1 Read a Lane and its associated Worlds

**Input:** authenticated caller, Workspace/Lane reference, optional continuation cursor.
**Success:** Lane-owned facts at an identified association revision; authorized associated World references; separately sourced observations and their freshness/completeness. Lists use owner-scoped cursors and an explicit end/continuation indicator.
**Failure:** denied, unknown target, stale cursor, unavailable owner, unsupported operation/version, or invalid reference. Unauthorized callers must not receive hidden World identities through errors or lists.

This read is not a global Workspace snapshot. A coherent association listing does not imply all worker, diff, policy, or readiness projections share one atomic cut. Partial observations remain explicitly partial.

### 4.2 Validate a concrete execution destination

**Input:** authenticated caller/delegation, Workspace, Lane, exact World realization, expected association revision, requested operation, and relevant configuration/context expectations.
**Checks:** same Workspace; current exact association; authorized caller and operation; matching realization; current applicable policy; required capability/readiness; configuration/context preconditions.
**Success:** owner-validated binding evidence for the submission boundary. It is not acceptance, a permanent authorization grant, or permission to skip launch-time validation.

For the first proof, require an explicit existing World selection. A Lane-only request with several Worlds is not executable work until an authorized coordinator selects a concrete destination. Reject absent/ambiguous selection rather than choose an arbitrary World or create one silently. An orchestration message may target a Lane to request planning; it is not yet B01 World-work acceptance.

Proposed public failure meanings include missing destination, association changed, wrong Workspace/Lane, realization mismatch, policy denied, unsupported capability, readiness unavailable, and owner unavailable. Exact serialized variants and privacy-preserving mappings remain owner design work.

### 4.3 Bind accepted work without a race

1. Resolve the concrete destination and capture its association revision before a possible launch effect.
2. The association authority and dispatch/admission owners establish a revision-bound reservation or equivalent serialized protocol. A check followed by an unguarded launch is insufficient.
3. Keep the selected association stable through runtime submission and uncertain acknowledgement. A transition cannot evade an in-flight admission merely because it has not produced a receipt yet.
4. Only existing runtime acknowledgement and acceptance owners can establish accepted work. Bind its immutable provenance to the exact Workspace, Lane, World realization, association revision, and applicable accepted policy/configuration references.
5. Publish a consumer acceptance result only after its required receipt, supervision, policy, and binding joins are durable and recoverable. If reporting fails after runtime acceptance, reconcile the same work; do not launch another task.

The candidate protocol deliberately avoids prescribing a new lock or record format. The responsible owners must specify the reservation, crash boundaries, and release conditions before implementation. An unknown submission remains unresolved; it is not safe evidence for releasing state custody or changing the selected destination.

**Preserve acceptance ownership:** B03 evidence neither manufactures acceptance nor changes the immutable acceptance owner's meaning. The receipt owner must choose an explicit versioned carrier or authenticated linked projection; never append unrecognized fields to a frozen V1 schema.

### 4.4 Read historical binding

Read accepted binding by exact work/receipt reference under current caller authorization. Return the association at execution acceptance, separately from any current World association. If World W moves from Lane A to Lane B, work accepted in A remains attributed to A. Authorized views may show the move separately; they must not relabel old work as executed in B.

Historical provenance does not grant present action authority. Cancellation/inspection continues through exact accepted-work owners; never substitute the World's current Lane or newest run for the original target. If a transition makes a live route unavailable, expose that condition rather than inventing terminality or redirecting cancellation to different work.

### 4.5 Proposed receipt composition and the HSA bridge

**Recommended carrier:** return a versioned consumer composition containing the existing authenticated receipt projection plus an authenticated `AcceptedBinding` projection. Keep the B03 relationship with its Workspace association owner and join it to the existing exact B1 acceptance reference; do not add Lane fields to frozen receipt V1, copy acceptance into a second registry, or expose the private E2-RM witness. Storage layout and the transaction/recovery protocol remain owner decisions.

The binding may be prepared against a proposed acceptance ID before launch, but it is not an accepted binding until joined to actual B1 acceptance. Runtime acceptance can precede completion of this consumer composition. If any required join is missing or conflicting, return the recoverable B01 request disposition, not a fabricated complete receipt or a refusal implying nothing launched. [B01's outcome table](collider-b01-b05-handoff.md#31-request-admission-and-consumer-acceptance-are-separate) governs that distinction. A conflicting durable join needs owner recovery; it is never repaired by reading the World's current Lane.

**HSA integration remains a required owner decision, not a presumption of redesign:** the existing invariant permits one optional exact World binding per orchestration session; applicable physical adoption must equality-join that authority. Workspace-level permission to coordinate many Worlds does not make that tuple plural. The missing responsibility is Workspace Lane-association authority plus authenticated resolution from a selected Lane/World to an eligible execution authority. First determine whether existing exact-session authority and an approved delegation/resolution path can express that selection. Propose a new HSA carrier or successor contract only if a demonstrated gap requires it; singular binding alone does not prove that need.

Before an executable B01 path exists, HSA/dispatch must approve that exact resolution and its authorization checks. It may not impersonate a toolbox caller, silently rebind the Workspace orchestrator's session, create a session per task by inference, or make Lane lifetime depend on a session. Record the authenticating principal/delegation separately from execution session/caller refs; both must join admission evidence. The first proof may use a matching existing authority, but cannot claim general multi-World dispatch from that one case or bypass the companion B01 runtime-profile readiness gate.

## 5. Stateful operations: required distinctions, not implemented APIs

| Operation | Required accounting | Not implied |
|---|---|---|
| Runtime restart/recovery | Exact retained state/identity and pending-work recovery disposition. | State deletion, successful recovery, or a new Lane. |
| World replacement | Old and successor realization references; custody of base/diffs; explicit unfinished-work and cleanup disposition. | Discarding the predecessor or using generation as sibling identity. |
| World reassociation | Source/destination Lanes, expected association revision, destination authorization/baseline, override reconciliation, active and in-flight work disposition, durable history. | Copying or merging content, authority widening, or rewriting old receipts. |
| Change promotion/integration | Exact source changes/base and destination state, applicable gate, conflict check, result and failure/recovery evidence. | Moving the source World, terminating siblings, or atomic rollback not actually supported. |
| Optional propagation | Explicit source revision/change selection and recipient Worlds; each recipient's authorization and conflicts. | Mandatory catch-up or declaring all unsynchronized Worlds invalid. |

**Bounded proof recommendation:** keep World association fixed while the selected ephemeral work is in flight or nonterminal. Disable consumer-driven reassociation/replacement/promotion in that proof. This is not a permanent ban on active-work transitions in the complete architecture; it prevents the first proof from depending on an undesigned migration protocol.

For a later transition implementation, specify prepare/authorize, freeze or reconcile relevant work, commit association/state custody, and recover uncertain outcomes. These are required concerns, not a ratified state machine. A failed or interrupted operation must report which effects are known versus uncertain; it cannot erase pending diffs or claim that a move performed a merge.

## 6. Lane state: ownership proposal, display still open

| Lane-owned candidate facts | Referenced/derived observations; keep original owner |
|---|---|
| Lane identity, Workspace association, descriptive purpose reference. | Work-item meaning and semantic scope remain Handbook-owned. |
| Current World associations and association history. | World runtime availability, realization/generation, diffs and state custody. |
| Declared Lane baseline reference and authorized orchestration assignment references. | Effective configuration resolution, per-World overrides, applied dependency evidence, policy decisions. |
| Explicit Lane lifecycle intentions, if later defined. | Accepted work/terminal state, worker lifecycle, obligations, gate outcomes, and available change propagation. |

Do not create a competing durable inbox, receipt store, scheduler, or gate ledger. A Lane view can show active work, outstanding attention, integration readiness, and optional updates simultaneously. One completed World does not make the Lane complete. Exact lifecycle states, aggregate fields, timestamps, query guarantees, and UI layout remain unselected.

## 7. Existing-owner reconciliation

The linked owner documents were checked on the clean authoritative Linux checkout at `088f62d81c182b92cac404983b8bf512becc829c`; the corresponding local files were byte-identical. This is a document comparison, not a runtime/source audit.

| Existing boundary | Relation to target / required successor work |
|---|---|
| [HSA exact session/world binding](../../../llm-last-mile/runtime-refactor/architecture/invariants/01-durable-session-truth-is-process-independent.md) | Host placement already differs from World binding. Its singular session-bound authority and exact shared-owner adoption do not define durable Workspace multi-World Lane associations. Explicitly reconcile these owners; do not map Lane IDs onto session IDs or fabricate adoption proof. |
| [Receipt and supervision owners](../../../llm-last-mile/runtime-refactor/seams/runtime-event-receipt-supervision-and-retained-runtime.md) and [ephemeral receipt](../../../llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md) | Reuse exact acknowledgement, acceptance, supervision, and policy linkage. Choose how B03 association provenance is authenticated and exposed without mutating immutable history or introducing a second registry. |
| [Filesystem model](../../internals/world/workspace_sync_filesystem_model.md) | Describes a live host lower layer, host edits becoming visible, best-effort apply without rollback, and conditional diff clearing. This differs from SCA-05 stable working state. Reconcile existing sync/history and guest-image planning through the [required research brief](../../ideas/world-state-environment-research.md); this is not greenfield isolation design. Guest userspace isolation alone does not freeze project mounts. Preserve protected-path and conflict safeguards. |
| [E2 policy commitment](../../../llm-last-mile/runtime-refactor/contracts/dispatch-policy-commitment-v1.md) | Future turns intersect current parent with immutable launch cap. Confirmed same-logical-worker expansion needs a separately designed authorized successor boundary; a turn patch cannot silently widen that cap. Emergency revoke/invalidation remains distinct from changing immutable history. |
| [World dependency configuration](../../reference/world/deps/README.md) | Existing inventory/enabled/applied separation is useful; Lane baselines and V1 World overrides need explicit composition and scope ownership. Changing desired configuration is not automatic application or permission to ignore revocation. |

The historical E3 service exclusion is not removed by this draft. Multi-World concurrency requires safety-equivalent ownership, mediation, and cleanup proof, not merely finer identifiers or a deleted lock.

### Semantic decisions that can proceed independently of mechanism research

Proposed review disposition: Lane identity/association authority is Workspace-scoped; session authority, accepted work, worker lifecycle, and physical realization retain separate owners. Current association changes are revision-checked, while accepted-work attribution is immutable and separately readable under current authorization. A stale destination is refused or reconciled, never silently retargeted. Names, directory paths, Git branches, image IDs, and runtime liveness cannot substitute for these references.

These rules can be reviewed without selecting a persistence module or filesystem. The [research brief](../../ideas/world-state-environment-research.md#relationship-to-b01b05) instead gates selection and proof of project/environment isolation and stateful transition mechanisms. Older guest-rootfs plans remain inputs with a separate delivery decision; they are not automatically adopted or placed into the future alternative-Lane-backing category. No mechanism experiment has been dispatched.

## 8. B03 acceptance cases and remaining design decisions

Required tests for later implementation (not executed here):

| Case | Required observable result |
|---|---|
| Two Worlds in one Lane, multiple workers in a World | Distinct exact destinations and work identities; no forced one-to-one alias. |
| Missing/ambiguous World; forged or cross-Workspace association | No launch/acceptance; explicit authorized error without leaking forbidden identities. |
| Association changes after validation but before submission | Serialized resolution or conflict; no work accepted against an unvalidated destination. |
| Submission ACK lost while a transition is attempted | Same-work reconciliation; no second launch and no unaccounted association move. |
| World association moves A→B→A; a new request submits old A revision | Stale association refusal before any launch; equality of the Lane ID does not revive the selector. An already accepted request instead returns its original binding under current read authorization. |
| Unrelated sibling joins or Lane display name changes | Selected World binding remains usable if its own revision and all other admission checks still hold. |
| B1 accepted, but receipt/binding composition is incomplete | Recoverable request remains linked to original work; no complete consumer acceptance or false no-work refusal. |
| HSA bound World differs from selected destination | No dispatch through that authority; no silent HSA rebind or client-fabricated delegation. |
| Work accepted in A; World later moves to B | Historical binding remains A; current association is separately B; current read/action authorization still applies. |
| UI closes, host episode parks, or one worker finishes | No implicit Lane/world deletion or fabricated completion of unrelated work. |
| One World promotes while siblings continue | Gate applies to selected work; optional updates do not force a Lane-wide barrier. |
| World replacement or interrupted merge | Diffs and partial/uncertain effects remain accounted for; no unsupported rollback/success claim. |

Before declaring B03 wire-final or implementable, resolve: (a) association authority/persistence placement and authenticated execution-authority resolution, establishing whether any HSA extension is actually necessary; (b) wire encodings for the semantic references/revision scopes in §3.1; (c) admission/transition serialization and crash recovery; (d) owner acceptance of the linked receipt composition in §4.5 and its authenticated read exposure; (e) capability/error/pagination schema and supported call path. Broader Lane state, backing, and promotion mechanisms can remain separate, explicitly bounded successor work.

**Review exit:** every proposed rule is accepted or amended by its relevant owner, the consumer can identify an exact historical execution binding, and unsupported broader operations are explicit. Runtime readiness additionally requires actual authenticated access and the relevant installed-path tests. This draft satisfies neither exit by its existence alone.
