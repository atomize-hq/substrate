# Session Decision Dossier

- Project: Substrate
- Repo / Workspace: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- Date: 2026-09-20
- Authoring Session: Interactive `interview-me` core-product-shape session
- Last Updated: 2026-09-20 — B01–B05 draft progress and required state/environment research follow-up reconciled in place.
- Branch / Commit: `feat/internal-host-orchestrator-world-dispatch-bootstrap` @ `a8b1a44ed2f9c33f6963eb552f9da05c198bce6a`
- Status: Confirmed product intent recorded; B01–B05 contract proposals drafted; explicit mechanism research/experiments required but not started. No implementation, runtime validation, archive, cutover, or canonical owner-contract rewrite is authorized by this dossier.
- Primary Topic: Durable Workspace lanes, hierarchical orchestration, and a bounded Collider B01–B05 handoff.

## Executive Summary

- The confirmed model separates normal human host work, restricted-by-default host orchestration, and World-bound workers/tasks. Workspace orchestration coordinates host-bound lane orchestrators; delivery of lane orchestrators in V1 remains open.
- A Lane is a durable, explicitly created Workspace resource, independent of orchestration/worker sessions. **One Lane can contain multiple Worlds; an associated World has one current Lane association at a time.** Worlds can serve different implementations or validation purposes within one Lane objective, not necessarily one World per worker.
- Worlds contain meaningful working state and diffs. Replacement, recovery, reassociation, and promotion must account for that state and active work. A move between Lanes is not itself a merge; historical execution retains its original association. This corrects the earlier oversimplified “replace a World without replacing the Lane” wording.
- Worlds inherit a common Lane dependency/configuration/policy baseline through Substrate configuration, with **explicit per-World dependency/configuration overrides included in V1**. Alternative-system backing (remote SSH, a separate local VM image, or a container image) is future scope, explicitly **not V1**. Policy cannot silently widen enclosing authority.
- Worlds or selected changes may advance independently through applicable gates. Other Worlds may optionally rebase/merge/sync those changes, but mandatory catch-up or a universal Lane-wide completion barrier is not intended. Host development and existing World working state must not change silently.
- A Lane has associated state; exact tracked/displayed fields remain open. Separate activity, attention, readiness, and available synchronization are design candidates, not an approved schema or single “running/done” flag.
- Core-intent interviewing is complete at approximately 95% confidence in intent, **not** implementation/readiness. [B03](docs/contracts/draft/lane-world-binding.md) and [B01–B05](docs/contracts/draft/collider-b01-b05-handoff.md) proposals now exist; schemas and runtime proof remain unresolved. The user requires [explicit research/experiments](docs/ideas/world-state-environment-research.md) before selecting new state/environment mechanisms. Older guest-rootfs/sync/history plans are inputs, not inherited selections or automatically deferred with alternative-Lane backing. Semantic contract work continues separately.

## Objective / Problem Statement

### Objective

- Settle Substrate's core product shape far enough to unlock a coherent Collider B01–B05 contract handoff, while preserving the broader Substrate architecture as the objective.
- Preserve the initial core-model and hierarchy confirmations together with subsequent user clarifications of multi-world lanes, stateful world transitions, V1 overrides, and independent integration/synchronization.

### In Scope

- Human host work, Workspace orchestration, Lanes, World realizations, workers/tasks, parking/reactivation, permission narrowing, coordination, direct intervention, and explicit host↔Lane change gates.
- The confirmed product decisions, meaningful alternatives, explicit supersessions, and boundaries relevant to a later B01–B05 owner-contract pass.

### Out of Scope

- Implementation, runtime/source readiness claims, tests, a documentation reset, archiving/replacing owner documents, or a Collider implementation.
- Selecting a worktree/Git-alternative mechanism, a merge engine, exact policy/config/profile schema, message/approval taxonomy, or exact B01–B05 APIs.
- Treating Collider's Control Plane presentation/projection as a new Substrate agent, Lane, or execution authority.

### Constraints

- Do not distort complete Substrate product behavior to satisfy the first Collider proof. Conversely, do not make all eventual Substrate capabilities prerequisites for that bounded proof.
- Preserve host developer work and world execution as separate concerns. Do not infer that the current E3 service-exclusion safety boundary is removed, replaced, or approved for removal.
- The interview is primary evidence for intended behavior. Historical Linux findings and older planning documents are supporting context only; this macOS-local binding is not fresh authoritative Linux runtime proof.

## Current State

- Core product intent and the later clarifications are recorded below. At the user's final confidence checkpoint, the assistant stated approximately 95% confidence and no further core-intent interview questions were needed; remaining questions belong to contract/implementation design unless a genuine product conflict emerges. This is not a quantitative readiness measure or individual approval of B01–B05 schemas.
- The user authorized continuing semantic contract design and recording the research requirement where later work can discover it. Current documentation scope: this dossier, the two proposal drafts, one research brief, and a backlog pointer. No research execution, mechanism experiment, runtime test, implementation audit, canonical owner-contract rewrite, or external comparison research has been performed.
- Local binding remains branch `feat/internal-host-orchestrator-world-dispatch-bootstrap`, HEAD `a8b1a44ed2f9c33f6963eb552f9da05c198bce6a`, tree `766709683f93fa5d151da1db74d297160324cba7`; no staged changes. Before this follow-up, the two dossiers and two contract drafts were untracked; this dossier's pre-edit SHA-256 was `3ccc7284baf04987136f0f3cdddd5091ae2843e28eec24a238c332e84ddd2fe5` (baseline only, not updated bytes).
- Prior read-only contract work rebound the clean authoritative Linux checkout at `088f62d81c182b92cac404983b8bf512becc829c`, tree `b40da914c1e5d73fc79f13c0946f6af9c97455d1`, and compared named owner documents; the later planning lookup also checked backlog/ADR/sync references against that checkout. This refreshed document evidence, not source/runtime readiness or the dirty E3 worktree. The historical runtime dossier and R1–R34 remain untouched. B03-first remains a proposed review order, not approved schemas or implementation dispatch.

## Key Decisions

Local labels below aid resumption only. They do not create a competing canonical decision system.

### SCA-01 — Separate host collaboration from World execution

- **Status:** Confirmed product intent.
- **Decision:** A human works in a Substrate Workspace bound to the host. The human collaborates with a host-bound Workspace orchestration capability, which uses Substrate tools to create, message, and fork World workers/tasks as well as dispatch work into Worlds. World workers are long-lived across turns; World tasks are short-lived. Both are World-bound. This describes intended product capability, not current implementation/proof or a settled fork API.
- **Why:** The product must permit background agent work without generally disrupting ordinary host development.
- **Authority and permission consequence:** The Workspace orchestrator is restricted on the host by default. It can perform authorized Substrate state/configuration/profile/delegation/sync operations, but is not implicitly a general host shell or unrestricted host file writer. Broader host execution/write access is configurable.
- **Not decided:** Whether the default configuration surface is scoped management tools, direct writes under `.substrate`, or a mixed model. The user rejected assuming that Workspace settings or Workspace policy alone own this behavior; an assigned orchestrator Agent Profile was raised as a possible owner. Exact precedence and self-escalation protections remain open. Detailed worker/task fork semantics—identity, state/context inheritance, authority, placement, and lifecycle—also remain open; they must not be inferred from the separate historical host-session fork behavior.
- **Boundary:** The assistant's statement that skills may guide behavior but not independently grant permissions was not separately confirmed as an owner contract; retain it only as a design consideration, not as a settled rule.

### SCA-02 — Policy narrows toward World work; worker identity survives authorized expansion

- **Status:** Confirmed product intent.
- **Decision:** World workers/tasks operate under a session-level policy that can only narrow from Workspace policy (or an equivalent parent constraint). Individual workers can have distinct visibility, read, and write scope inside a shared Lane World.
- **Decision:** When an authorized access request adds files, the worker remains the **same logical worker** with its task relationship and relevant context. The system may pause/re-establish an execution boundary, but it must not discard that logical identity merely because scope changed.
- **Why:** The orchestrator must partition work safely without losing continuity when legitimate work crosses the original file boundary.
- **Historical-source constraint / proposed reconciliation, not separately interview-confirmed:** The historical runtime dossier records immutable accepted permissions/evidence. Preserving that history while representing later authorization through a successor boundary is a design direction to reconcile with current owners, not an additional user-confirmed decision. Logical-worker continuity is the confirmed intent above.
- **Not decided:** The exact successor representation, revalidation behavior, context transfer rules, and how an immutable worker-cap model evolves to express the expansion without a turn-local patch shortcut.

### SCA-03 — Durable multi-World Lanes and explicit, stateful associations

- **Status:** Confirmed product intent, including later cardinality/transition clarifications.
- **Decision:** A Lane is explicitly created for a feature, long-term project, or component partition (frontend, backend, database, integration). It belongs to the Workspace, not to an orchestration or worker session. Ending/parking an orchestration session does not implicitly destroy the Lane or stop its workers.
- **Decision:** One Lane may contain multiple Worlds: separate working environments contributing to its objective, such as implementation alternatives and validation. Each associated World has **one current Lane association at a time**, providing an unambiguous baseline and coordination context. Other Lanes may inspect/consume authorized outputs without becoming co-owners of live writable state. This is not one World per worker.
- **Decision:** Lane and World identities are distinct. Worlds can transition between Lanes through explicit operations; historical execution must retain the Lane/World association under which it occurred rather than being relabeled with the World's new association.
- **Stateful boundary:** A World holds working state and diffs. Replacing or transitioning it must account for those changes and active work, including reconciliation when changes are integrated into another Lane or the host. Lane durability does not make a World disposable. Moving an association does not itself merge contents into a destination.
- **Design distinctions to specify:** runtime restart/recovery, stateful World replacement, merge/promotion of changes, and World reassociation are different operations even if a workflow combines them. Exact identities, atomicity, state custody, destination-policy validation, and active-work handling are not yet specified. No automatic copying, termination, deletion, or state loss is approved.
- **Lane state:** The user requires associated Lane state but left what it tracks/displays open. Associations, effective baseline/overrides, active work, attention obligations, integration readiness, and available synchronization are assistant-proposed categories. Separate Lane-owned facts from projections of World/worker/obligation owners; do not create duplicate durable truth or ratify an enum from this list.
- **Why / correction:** The earlier B03 phrase “replacement does not replace the Lane” omitted state/diff custody. The user corrected it, expressly allowed World transitions and multiple Worlds per Lane, and accepted one current Lane association per World. The old draft's session-owned-Lane direction is superseded; its isolation implementation is still unselected, not categorically rejected. Any singular “the Lane's World” wording is illustrative, not a cardinality constraint.
- **Do not infer:** Durable identity promises neither a permanently running process nor reboot survival. Worktree cardinality, simultaneous runtime limits, unassociated/pre-admission World states, and association mechanics remain open.

### SCA-04 — Workers may share a World; writable-state mechanics remain open

- **Status:** Confirmed topology; execution mechanism intentionally open.
- **Decision:** Multiple workers may share a World associated with a Lane. A Lane may also contain multiple independent Worlds (SCA-03). The orchestrator/lane orchestrator can give workers disjoint see/read/write scope; workers may request additional access.
- **Why:** Workers can collaborate in a shared World without receiving unnecessary visibility or write authority; independent Worlds support separate working environments within the same Lane objective.
- **Alternatives retained:** permission-partitioned common checkout/shared World state; worktrees within a Lane/World arrangement; another Git-like/versioning/isolation mechanism selected through bounded research. The host-main-worktree/Lane-worktree analogy remains a candidate and must now account for multiple Worlds rather than imply one Lane equals one checkout.
- **Not decided:** Same-World writable-checkout topology, worktree placement, conflict detection/resolution, advertised concurrency guarantees, and exact cross-World/cross-Lane isolation realization. No external research or implementation selection has occurred. Access partitioning alone is not proof of conflict-free editing.

### SCA-05 — Explicit gates, independent advancement, and optional synchronization

- **Status:** Confirmed product intent, extended by the later promotion/synchronization discussion.
- **Decision:** World working state within a Lane stays predictable while the human edits on the host. Host changes do not silently appear beneath active workers; refresh is explicit and conflict-aware. Returning changes to the host or integrating them into an integration Lane passes through an explicit gate.
- **Decision:** Individual Worlds or explicitly selected changes may advance independently under applicable gates while sibling Worlds keep experimenting or await clarification. A Lane-wide milestone may require a broader gate, but a Lane-wide completion barrier is not mandatory for every integration.
- **Decision:** Worlds remaining in the source Lane may optionally receive a rebase/merge/sync from the promoted or transitioned World. Tools/workflows/skills should support this, but propagation and catch-up are **not mandatory by default**. Independent continuation against existing state is allowed. A particular milestone may still require reconciliation.
- **Distinction:** Promotion of changes, transition of a World's association, and propagation into other Worlds are separate effects. Combining them in a workflow does not authorize silent overwrites, disposal, or bypass of conflict handling and gates. Optional content synchronization does not waive destination authorization or policy checks.
- **State implication (proposal):** “Changes available to incorporate” should not automatically mean “blocked/invalid.” Separate activity, attention, and integration readiness rather than infer Lane completion from one World finishing. Exact state fields and presentation remain open.
- **Not decided:** Gate authority (human/delegated), source/destination/revision selectors, selected-change representation, conflict ownership/UI, active-work coordination, rebase/merge/sync mechanisms, and early delivery scope for a merge engine. No Git/worktree implementation or automatic merge engine is selected.

### SCA-06 — Parking retains orchestration relationship; event handling is classified and authorized

- **Status:** Confirmed product intent; detailed trigger policy open.
- **Decision:** An orchestration session may be parked rather than torn down, later reattached/reactivated, and able to receive messages from workers/tasks. Continued orchestration is intended.
- **Decision:** A received message can be an acknowledgment only, an authorized trigger for a bounded reasoning turn, or an escalation/notification for human action. An authorized bounded reasoning turn may handle ordinary coordination within previously delegated authority and park again.
- **Why:** Completion acknowledgments should not consume reasoning turns, while ordinary delegation should not require the human to manually wake an orchestrator for every routine event.
- **Boundary:** Restoring/reattaching ownership is not itself permission to prompt, approve, answer, or continue work. The historical auto-attach discussion remains a separate, limited ownership-restoration concept.
- **Not decided:** Message classification, trigger enablement, approval matrix, notification behavior, budgets, and the governing source for those choices (the user specifically left Workspace policy/settings versus orchestrator Agent Profile placement unresolved).

### SCA-07 — Hierarchical orchestration is target architecture

- **Status:** Confirmed expanded product intent.
- **Decision:** A host-bound Workspace orchestrator has top-down visibility across Lanes and normally dispatches to host-bound, lane-scoped orchestrators. A lane orchestrator allocates and coordinates workers/tasks across its Lane's associated Worlds; workers/tasks remain World-bound.
- **Decision:** Workers/tasks report into the Lane orchestrator. It may dispatch the next local work, or—when a result is final or needs approval, clarification, or escalation—roll that item up to Workspace orchestration. Not every worker completion or status update must roll upward. Initially, Lane orchestrators do not coordinate directly across Lanes; Workspace orchestration mediates cross-Lane coordination.
- **Decision:** The Workspace orchestrator may communicate directly with a worker when needed. An authorized Workspace-level redirection takes precedence and updates the Lane orchestrator's understanding, so direct access never creates a hidden competing instruction chain. Inspection/clarification are distinct from redirection and need not interrupt work.
- **Why:** The human should be able to coordinate at Workspace scope without manually managing every worker, while Lane agents retain local responsibility and tighter scope.
- **Not decided:** Whether lane orchestrators are delivered in the first version, whether every Lane has one continuously running, exact direct-message semantics, and detailed delegation/intervention authority. Reuse orchestration machinery where possible; do not invent a second lifecycle system by default.

### SCA-08 — Product orchestration is distinct from Collider's presentation/control-plane surface

- **Status:** Workspace-wide orchestration is interview-confirmed; the Collider consumer/presentation boundary below is supporting context from its canonical glossary and foundations (see Evidence / Source Map), not a separate interview confirmation.
- **Confirmed intent / sourced consumer context:** Workspace-wide orchestration is a product capability spanning all Lanes. The cited Collider canonical documents define its Control Plane as a consumer/presentation surface, not itself a Lane or an independent execution authority; this UI surface must not be conflated with a Substrate orchestration agent.
- **Why:** UI topology must not silently introduce a second execution, scheduler, or semantic authority.
- **Consequence:** Collider needs owner-provided identities and associations; it cannot infer a Lane from a directory, a session, a World ID, or a displayed control-plane location.

### SCA-09 — Scope of the Collider handoff

- **Status:** Confirmed objective; later contract formulations are proposal only.
- **Decision:** The near-term purpose is a coherent contract handoff for Collider B01–B05 without turning the selected proof into the definition of all Substrate.
- **Existing Collider planning scope, not separately confirmed in this interview:** The cited `substrate-accepted-work-boundary-plan.md` selects an ephemeral World task plus dependency read/add/remove as its bounded planning scenario. The planning/proposed handoff does not require lane orchestrators, retained-worker scope expansion, a merge engine, real approval continuation, queue promotion, or Qualified Session continuity first. This is inherited planning context, not new user ratification of the proof scenario or its exclusions.
- **Assistant proposal, not user-ratified contract:** Begin with B03 identity/binding, then B01/B02 acceptance/control, B04 context delivery, and B05 independently. The product identity/cardinality/transition rules are now clarified in SCA-03. Their exact B03 binding representation, acceptance freeze point, validation, and exposure remain owner-design proposals, not accepted schemas or APIs.
- **B01–B05 questions to carry forward:**
  - **B01:** authenticated caller/delegation binding, request identity versus accepted work, idempotency/conflicts, uncertain acknowledgement, reconciliation, and retention/expiry;
  - **B02:** exact-work observation/control/reconnect/cancellation/lifetime and visibility of direct redirection;
  - **B03:** stable Lane identity; multiple Worlds per Lane and one current Lane per associated World; explicit World selection and stateful transitions; immutable historical execution association; validation/mismatch behavior and Lane state ownership/projection;
  - **B04:** typed selected-reference/Handbook-resolution/authorized Substrate delivery and provenance, without converting transcript or implicit native continuity into authority;
  - **B05:** revision-checked dependency/configuration edits, coordinated supported writers, authoritative readback, and declaration-versus-application semantics. SCA-10 adds Lane inheritance and V1 per-World overrides to the full architecture; it does not silently expand the existing Workspace-only dependency proof into all configuration scopes.
- **Status caveat:** The existing Collider boundary plan's source classifications and proposed facade vocabulary are historical/advisory planning inputs. Recheck current authoritative owners before reusing them as implementation fact.

### SCA-10 — Shared Lane configuration baseline with V1 per-World overrides

- **Status:** Confirmed product intent and explicit V1 scope decision.
- **Decision:** Worlds in a Lane inherit the same baseline World dependencies/configuration/policies through Substrate dependency/configuration mechanisms. Individual Worlds can declare different dependencies/configuration via explicit overrides. The user explicitly confirmed that these per-World dependency/configuration overrides belong in **V1**, not just a future release.
- **Policy boundary:** “Override” does not authorize arbitrary widening of Workspace/parent limits. Preserve SCA-02's narrowing and separately authorized scope-expansion requirements. Exact configuration merge/precedence, restrictive policy composition, and revision semantics still require owner design.
- **Design consequence, not selected schema:** Distinguish inherited baseline, explicit World overrides, effective desired configuration, and actually applied environment state. Shared inheritance must not be interpreted as silent mutation of running environments or bypass of the confirmed refresh gates. Dependency/configuration reconciliation and security-policy revocation timing must be specified separately; stable files are not permission to retain revoked authority.
- **Open:** baseline source/ownership and precedence; add/remove/replace semantics; valid override dimensions; configuration drift/readiness representation; update/application gates; and how reassociation reconciles the destination Lane baseline with existing state and overrides. No override representation or atomic application protocol is selected.

### SCA-11 — Alternative Lane backing is future scope, not V1

- **Status:** Confirmed future direction and explicit exclusion from V1.
- **Decision:** The architecture should accommodate Lanes backed by different systems/machines, including an SSH-connected remote system, a separate local Substrate VM image, or a container image (Docker was an example). These alternative backing capabilities are **not part of V1**.
- **Boundary:** This deferral concerns new Lane backing/placement capability, not removal of existing Substrate platform backends. It does not defer the per-World overrides confirmed for V1, nor settle lane-orchestrator delivery timing.
- **Open:** backing identity, transport/authentication, capabilities, placement, persistence, portability, and per-World realization constraints. Naming SSH/VM/container possibilities does not select a provider, remote execution API, or implementation.

### SCA-12 — Explicit research and experiments before mechanism selection

- **Status:** Confirmed research requirement; brief recorded, execution not started.
- **Decision:** The user pointed to existing guest-rootfs/image backlog and planning as prior context, then explicitly required research/experiments for the revised model. Reuse and reconcile that material; neither automatically adopt it nor automatically defer the older guest-rootfs initiative with SCA-11's alternative-Lane backing. Its scope/delivery decision remains distinct.
- **Scope:** Compare project-state isolation/concurrent editing, environment/dependency isolation, and stateful lifecycle/transition safety against the confirmed multi-World model. Produce alternatives, bounded experiments, evidence-backed recommendations, and a reviewed later implementation plan. Detailed proposed stages and acceptance criteria live only in the [research brief](docs/ideas/world-state-environment-research.md), discoverable from the backlog.
- **Dependency rule:** B01–B05 semantic contract definition can continue now. Selection of filesystem/environment mechanisms and claims of their guarantees wait for research/experiment evidence and applicable production proof. Recording this requirement does not dispatch research, run privileged experiments, or authorize implementation. V1 per-World overrides remain required; SCA-11 remains unchanged.

## Research Findings

### Interview and supporting-record reconciliation

- **Question:** What existing material remains useful after the user clarified the product model?
- **Findings:** The interactive interview is the primary authority for current intent. The multi-lane document correctly separates Lane product identity from World runtime identity, but its session-owned-Lane choice is superseded. Later interview clarification establishes multiple Worlds per Lane, one current Lane per associated World, and stateful transitions; do not read its singular wording as a cardinality constraint. Its isolated-writable-lane V1 remains an unselected, nonauthoritative proposal—not a confirmed or rejected cross-Lane requirement. The 2026-09-18 dossier preserves historical safety, accepted-work, and auto-attach qualifications. The later Linux check refreshed named documents only; it did not rerun that historical runtime audit.
- **Most relevant sources:** the user-confirmed interview; `docs/ideas/multi-lane-worlds.md`; `RUNTIME_REFACTOR_SESSION_DECISIONS_2026-09-18.md`.
- **Implication:** Reuse the documents as linked evidence/context, but do not copy their older ownership model or claim their historical implementation inventory is current proof.

### Same-Lane concurrent editing/versioning

- **Question:** Which implementation mechanism best prevents workers in one Lane from overwriting each other?
- **Findings:** No mechanism was selected. Scoped see/read/write permissions, worktrees, and Git-like alternatives remain candidates. The later user direction makes explicit research/experiments required before selecting the revised state/environment mechanisms; this is no longer merely a possible follow-up. Execution remains unstarted.
- **Most relevant sources:** user interview; `docs/ideas/multi-lane-worlds.md` as an older design input only.
- **Implication:** Do not silently turn the old isolated-overlay recommendation, the worktree analogy, or an access-control concept into selected implementation. Use the [research brief](docs/ideas/world-state-environment-research.md) for comparison criteria and experiment proposals, without reopening confirmed product intent.

### Existing guest-image, sync, and history planning

- **Findings:** `docs/BACKLOG.md`, curated draft ADR-0009/0010, and Git-history guest-rootfs spec/plan/decision-register inputs already address substantial adjacent design. The historical guest model uses an immutable image and persistent writable overlay keyed by image/execution scope, generally Workspace scope; it needs reconciliation with multi-World/per-World overrides. Guest OS isolation does not itself guarantee frozen project mounts. Current sync documentation describes a live host project baseline; internal Git history is a separate concern.
- **Evidence:** The old planning-rich paths were removed by documentation-hygiene commit `af88dc2562dd2abf5c579cff9a47e5079f973ba2` and are readable at parent `38dc2838169f8b905554749693564cc52f58ac20`. The [brief](docs/ideas/world-state-environment-research.md#historical-material-that-is-no-longer-at-its-old-filesystem-path) records exact paths and a read-only recovery command. No old plan was restored or promoted to current authority.
- **Implication:** Correct the framing from a wholly new isolation design to reuse/reconciliation plus experiments for revised behavior. Neither historical accepted decision labels nor a document describing current behavior supply fresh runtime proof.

### Collider B01–B05 planning boundary

- **Question:** What does Collider need next without prescribing the complete Substrate implementation?
- **Findings:** The existing Collider plan records B01–B05 as concrete owner-contract questions and identifies a narrow ephemeral-task/dependency scenario. Its operation vocabulary, current classifications, and proposed facade are explicitly planning material, not new user decisions or current source proof. Collider canonical documents reinforce that its Control Plane/Lane surfaces are projections over owner state, not execution authority.
- **Most relevant sources:** Collider canonical glossary/foundations and `substrate-accepted-work-boundary-plan.md` listed below.
- **Implication:** A focused owner-contract handoff can proceed before every supporting implementation lands, but it must distinguish agreed target behavior, owner contract, implementation/exposure, and proof.

## Evidence / Source Map

### Primary design evidence

- Interactive `interview-me` conversation in this authoring session — primary record of user-confirmed intent; not a repository-owned authority file. In addition to the initial core/hierarchy confirmations, the update draws on these later exchanges (quoted anchors identify the evidence, not new external research):
  - User clarification that a World “will contain diffs” and may transition between Lanes → SCA-03/SCA-05 state custody and move-versus-merge distinction.
  - “lanes should be able to have multiple worlds associated to them” and Lane state still needs definition; “Exactly” to separate working environments contributing to one Lane objective → SCA-03/SCA-04.
  - Shared dependency/config/policy inheritance; “yes, this is what we should do” to V1 per-World overrides and deferred alternative backing → SCA-10/SCA-11.
  - “I think this is the right structure” to one current Lane per World → SCA-03; not a ratification of detailed transfer schemas.
  - “yes, I think this is right” to independent advancement, followed by optional rebase/merge/sync for remaining Worlds → SCA-05.
  - Final “no more questions? confidence at 95%?” checkpoint and assistant answer: core-intent interview complete; remaining work is contract/design. User then explicitly requested this in-place dossier update.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/ideas/multi-lane-worlds.md` — draft supporting input; its Lane/World distinction is useful, while its session-owned-Lane model is superseded, and later multi-World/cardinality/V1 scope decisions in this dossier govern where its assumptions differ. Its specific filesystem/isolation mechanism remains unselected, not categorically rejected.

### Current design and research follow-up

- [B03 proposal](docs/contracts/draft/lane-world-binding.md) and [B01–B05 handoff](docs/contracts/draft/collider-b01-b05-handoff.md) — authored after initial dossier confirmation; owner-linked semantics, reconciliation needs and proof obligations, not approved APIs.
- [Research/experiment brief](docs/ideas/world-state-environment-research.md) and [backlog entry](docs/BACKLOG.md#required-research--multi-world-state-and-environment) — record SCA-12 with existing-input provenance, proposed staged work, experiments and exit criteria. No research execution or mechanism selection is implied.
- Latest user direction: “we need an explicit research/experiment for these new versions,” followed by authorization to continue semantics while documenting the follow-up. This distinguishes required research from the earlier optional-discovery suggestion.

### Historical Substrate context (do not treat as refreshed proof)

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/RUNTIME_REFACTOR_SESSION_DECISIONS_2026-09-18.md` — noncanonical historical decision/audit dossier, including its verbatim R1–R34 appendix. It was observed at SHA-256 `d3538ad11d215ac801ee3ba6c21605d98d4c376ef6033c174fd485765e1dcfca` before this record was created and must remain byte-for-byte unchanged. It supplies historical distinctions such as immutable acceptance and auto-attach ownership restoration; it does not ratify the new product shape or prove current Linux behavior.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.codex/review-evidence/collider-contract-consult-20260919/prompt.md` and `answer.txt` — advisory, pre-interview consultation. Useful for its B01–B05 question inventory; not current source proof or post-interview ratification.

### Collider consumer references (supporting, not rewritten here)

- `/Users/spensermcconnell/__Active_Code/atomize-hq/collider/docs/agent-native/canonical/00-collider-v1-glossary.md` — canonical consumer terminology and non-authority boundaries.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/collider/docs/agent-native/canonical/01-collider-v1-foundations.md` — Control Plane/Lane presentation, execution, and configuration boundaries.
- `/Users/spensermcconnell/__Active_Code/atomize-hq/collider/docs/agent-native/controller-ownership-plan/substrate-accepted-work-boundary-plan.md` — planning/partially-specified B01–B05 owner-boundary proposal; its classifications must be refreshed against current owners before reuse.

### Live-binding and validation limits

- This record uses the local binding in Current State. The preceding proposal/discovery work refreshed the authoritative Linux D document binding, not W or runtime behavior. No runtime test, source implementation audit, or canonical owner-document change occurred. Later implementation/readiness claims still require fresh applicable evidence.

## Options Considered

### Configuration and host authority

- **Scoped configuration/state tools:** limits changes to intended Substrate operations and can keep policy mutation separate from self-granting broader authority.
- **Direct `.substrate` access:** may be operationally simple for profiles/configuration but risks broad, self-modifying authority.
- **Agent-Profile-governed behavior:** named by the user as a plausible control point instead of assuming Workspace policy/settings own all automation behavior.
- **Outcome:** restricted host authority is the confirmed default; exact configuration owner, mutation interface, and broad-access setting are open.

### Same-Lane concurrency mechanism

- **Permission-partitioned shared World state:** matches the confirmed ability to give workers disjoint visibility/read/write access.
- **Worktrees within a Lane:** candidate that preserves a stable host/Lane synchronization gate.
- **Another versioning/isolation mechanism:** candidate for bounded later research.
- **Outcome:** workers may share a World, and a Lane may contain multiple Worlds. Writable-checkout topology and collision mechanisms remain open; multi-World support is no longer an unresolved cardinality choice.

### Event handling while parked

- **Acknowledge-only:** records completion/status without reasoning.
- **Authorized bounded reasoning:** lets parked orchestration continue ordinary delegated coordination.
- **Human escalation:** notifies/requests a human where authorization or judgment is missing.
- **Outcome:** all three are intended categories; their classification and authority rules are open.

### Lane-orchestrator delivery scope

- **Include the hierarchy in target architecture now:** confirmed.
- **Require it in the first shippable proof or run one per Lane continuously:** not decided.
- **Outcome:** preserve the target model in contracts without making every Lane orchestrator a near-term delivery prerequisite.

## Open Questions / Unresolved Risks

### Open Questions

1. What exact B03 identities, validation, and historical binding records implement the settled multi-World Lane model? How is a specific World explicitly selected when a Lane has several; what happens when no valid destination is selected?
2. Which configuration layer controls host-access mode, event-trigger behavior, approval authority, and agent profile assignment; how are precedence, audit, and anti-self-escalation enforced?
3. What message classes trigger acknowledgment, bounded reasoning, or human escalation; what authority, budget, retry, and notification rules apply?
4. Which project/environment mechanism satisfies the required visibility/read/write partitioning, stable baselines, per-World overrides, stateful transitions and integration gates? The required research brief now records criteria and proposed experiments; execute it only under a bounded research/experiment dispatch.
5. How does an authorized scope expansion retain logical-worker/task/context identity while immutably preserving earlier accepted authority and evidence?
6. Who authorizes host→Lane refresh and Lane→host integration, and what conflict/revision semantics apply? Independent promotion and optional propagation are settled in SCA-05; detailed operations are not.
7. What are the V1 inclusion, lifecycle, and direct-message rules for lane orchestrators? Is a Lane orchestrator required, optional, or on-demand for a particular Lane?
8. What worker/task fork semantics preserve or create identity, state/context, authority, placement, and lifecycle, without conflating them with historical host-session fork behavior?
9. What B01–B05 owner contracts, supported call paths, statuses, and proof are current on the authoritative platform? Historical classifications must not be repeated as current fact.
10. What does Lane state own versus derive/display: association, baseline, work, attention, integration readiness, and optional available synchronization? Do not collapse these into an unapproved status enum.
11. How do World replacement/reassociation and independent promotion preserve diffs, source baselines, active work, history, and destination authorization? What exact selectors and outcomes make optional sibling synchronization safe?
12. How do Lane baselines and V1 per-World overrides compose, version, and apply? What changes at reassociation; how are effective/realized state and conflicts exposed? Remote/VM-image/container Lane backing remains future work, not an open V1 inclusion question.

### Risks

- Collapsing Lane, World, worktree, worker, provider session, or UI projection identity would create stale targeting and potentially unsafe authorization.
- Treating auto-reattachment as automatic action authority would violate the distinction between restoring ownership and authorizing a reasoning/action turn.
- Broad host access or unrestricted `.substrate` mutation could let orchestration undermine the very policy that constrains it.
- A worktree or overlay choice made before behavior requirements are specified could freeze the wrong collaboration model.
- Treating Worlds as disposable, reassociation as a content merge, or current association as historical execution truth could lose diffs, misattribute work, or bypass destination authority.
- Assuming one World per Lane, mandatory sibling catch-up, or Lane-wide completion before every promotion would contradict the clarified product model.
- Treating baseline inheritance as automatic runtime mutation, or configuration overrides as permission widening, would conflate configuration state with applied state and authority.
- Calling the B01–B05 proposal an accepted interface, or treating Collider fixture/client behavior as Substrate proof, would overstate readiness.
- Removing or weakening the existing E3 safety boundary merely to allow parallel/ordinary work would be unsafe without an explicitly authorized, safety-equivalent replacement and proof.

## Recommended Next Steps

1. **Continue the existing B03 proposal into owner-contract review, not implementation.**
   - Goal: define explicit multi-World selection/binding, one-current-Lane association, historical execution identity, state/diff custody across transitions, and mismatch/absence semantics fitting SCA-03/SCA-05/SCA-08. Separate the narrow accepted-work association needed by Collider from the fuller promotion/transition design; do not require a merge engine to prove one task.
   - Output: refine the existing `docs/contracts/draft/lane-world-binding.md` with owner-reviewed identities, schemas, call boundaries and recovery rules; do not create a competing B03 note. Keep state/environment mechanisms gated on SCA-12 evidence.
   - Success condition: Collider can present one exact work item in its Control Plane and selected Lane view without inferring Lane from World/session/cwd, and no new scheduler/merge engine is required.

2. **Use the B03 result to bound B01/B02, then design B04 and B05 independently.**
   - Goal: retain exact caller/accepted-work/control semantics; keep context delivery and revision-aware dependency mutation with their proper owners. Account for SCA-10 baseline/override semantics without widening the bounded Workspace-only B05 proof by implication.
   - Output: owner-specific contracts that clearly separate target semantics, current exposure, implementation, and proof.
   - Success condition: no interface is invented from a UI projection or historical planning table, and the selected ephemeral-task/dependency proof is not misrepresented as whole-product completion.

3. **Dispatch the recorded state/environment research and experiments when separately authorized.**
   - Goal: follow `docs/ideas/world-state-environment-research.md`; reconcile existing plans, compare candidates, and run approved disposable experiments for project state, per-World environments and lifecycle safety. Keep older guest-rootfs scope separate from newly deferred alternative-Lane backing.
   - Output: evidence-indexed comparison, experiment results, recommendation, and a reviewed implementation plan. The backlog already links this brief; do not duplicate the work item.
   - Success condition: selection follows evidence and explicit review; negative/unsupported results remain visible. Semantic B01–B05 work continues without falsely claiming mechanism guarantees or requiring the whole research program before every narrow contract.

4. **Refresh evidence only when a contract or implementation decision needs it.**
   - Goal: rebind the authoritative Linux source/owner documents and validate the exact current operation path before claiming status.
   - Output: bounded evidence tied to a specific B contract or future implementation slice.
   - Success condition: historical D/W findings, the older draft, and this macOS-local branch are not presented as current runtime proof.

## Resume Instructions

A fresh agent should:

1. **Read, in order:**
   - this dossier;
   - `docs/contracts/draft/lane-world-binding.md`, `docs/contracts/draft/collider-b01-b05-handoff.md`, and the scope/dependency sections of `docs/ideas/world-state-environment-research.md`;
   - `docs/ideas/multi-lane-worlds.md`, treating its session-owned-Lane choice as superseded and its V1 filesystem/isolation choice as unselected rather than rejected;
   - the three Collider documents in the Evidence / Source Map;
   - `RUNTIME_REFACTOR_SESSION_DECISIONS_2026-09-18.md` only for historical safety/accepted-work context, preserving its full R1–R34 appendix byte-for-byte.

2. **Reconfirm before making a claim or edit:**
   - live repository/worktree, branch, commit, status, and the applicable authoritative platform/source binding;
   - the exact authorized artifact/path scope for the next contract pass (B03-first remains the recommended order, not an implemented or ratified API);
   - whether a new request authorizes contract documentation, implementation, research, or only analysis.

3. **Avoid redoing or misrepresenting:**
   - do not reopen the settled core-intent interview (completed at approximately 95% confidence) as though the older draft still controls; ask only on a genuine newly exposed product conflict;
   - do not promote assistant suggestions, the old consumer plan, or advisory consultation into user-approved contracts;
   - do not infer a Lane from World/session/worktree/cwd, treat a parked session as destroyed, or treat reattachment as permission to act;
   - do not assume one World per Lane, simultaneous co-ownership of one World by several Lanes, disposable World state, move-equals-merge, mandatory catch-up, or a required Lane-wide integration barrier;
   - preserve V1 per-World overrides and future-only alternative Lane backing; do not infer that lane orchestrators are either required or excluded from V1;
   - do not silently adopt old guest-rootfs decisions, equate guest OS isolation with a project snapshot, or automatically defer that older initiative under SCA-11; SCA-12 requires explicit research/experiments before mechanism selection;
   - do not alter runtime-refactor history, Collider canonical documents, or existing owner contracts as part of this decision record.

4. **Start with:**
   - continue the existing B03 identity/binding draft using SCA-03/SCA-05/SCA-08, with SCA-10–12 for configuration, scope and research boundaries. Resolve semantic owner gaps without selecting state/environment machinery prematurely. Do not restart a broad interview or begin implementation. Rebind authoritative owner documents before new contract/source claims; research execution, experiments, and implementation need their own bounded authorization.
