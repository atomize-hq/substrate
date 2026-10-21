# Substrate → Collider B01–B05 contract handoff draft

**Status:** Bounded contract-design handoff for consumer/owner review, updated 2026-10-21. Joint B01/B03 now has concrete proposed internal responsibilities and interfaces; new contracts are not claimed component-owner-approved, wire-final, implemented or runtime-qualified. The initial same-host Workspace opening correction and bounded no-send contract are recorded in [B01-B03-P1](../../../llm-last-mile/runtime-refactor/slices/tasks/b01-b03-authenticated-preflight.md); minimal proposed responsibility amendments are recorded in the three existing canonical owner documents; no runtime implementation, control-plane cutover or Collider production adoption is authorized.
**Start with:** [B03 Lane/World binding proposal](lane-world-binding.md).
**Objective:** Let Collider plan against coherent Substrate-owned operations without mistaking proposed semantics, existing source, supported exposure, and runtime proof for one status.

**Planning exit versus implementation exit:** the core model is sufficiently settled for this handoff. Collider can use the target inputs, outcomes, identity and race rules below to plan its adapters and UI; it must not treat them as present callable exports. Backend storage layout, fencing implementation, exact wire encoding and installed proof remain implementation obligations, not reasons to restart core-model discovery. Where an external owner mapping is outstanding, this handoff names that dependency and defines unsupported/failure behavior rather than inventing support. Consumer/owner review must still accept or amend the target; planning completeness is not implementation readiness.

## 1. Scope and source binding

The [core architecture dossier](../../../SUBSTRATE_CORE_ARCHITECTURE_SESSION_DECISIONS_2026-09-20.md) is the source of confirmed intent; it is not replaced here. The [historical runtime dossier](../../../RUNTIME_REFACTOR_SESSION_DECISIONS_2026-09-18.md) remains unchanged, including R1–R34.

Working assumption: retain the existing Collider planning scenario—one exact Linux ephemeral World task in Control Plane/Lane views plus Workspace dependency requirement read/add/remove—as the bounded handoff target. That selection comes from the consumer plan, not new user ratification of an implementation dispatch. It does not define the complete Substrate architecture or defer V1 per-World overrides.

This is a semantic proof target, not a selected executable runtime profile. B01's readiness gate below must be satisfied before treating that scenario as runnable or safe.

| Input | Bound identity / qualification |
|---|---|
| Core dossier | SHA-256 `6ff7f560a8d0c02178192051c207ef132a34aaa8b5e17d2c725a94a4ec43968f`; current 338-line revision includes SCA-12, E3-F chronology and the planning-handoff checkpoint. The initial 314-line dossier was read in full; subsequent research/design/handoff deltas were reviewed in place. |
| Historical dossier | SHA-256 `d3538ad11d215ac801ee3ba6c21605d98d4c376ef6033c174fd485765e1dcfca`; historical findings, not new runtime proof. |
| Historical initial authoring checkout | `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`, branch `feat/internal-host-orchestrator-world-dispatch-bootstrap`, HEAD `a8b1a44ed2f9c33f6963eb552f9da05c198bce6a`, tree `766709683f93fa5d151da1db74d297160324cba7`. Initially only the two dossiers were untracked; no tracked/staged changes. |
| Authoritative document inspection | SSH host `spenser-linux-codex`, `/home/spenser/__Active_code/substrate`, same branch, HEAD `088f62d81c182b92cac404983b8bf512becc829c`, tree `b40da914c1e5d73fc79f13c0946f6af9c97455d1`, clean when inspected. Eleven referenced owner/reference documents matched their local SHA-256 values. No source execution, tests, installed-runtime audit, W-worktree audit, or separate authority-branch refresh. |
| Historical consumer inspection | `/Users/spensermcconnell/__Active_Code/atomize-hq/collider`, branch `spike/agentkit-controller-correction`, HEAD `75631d432099fa18f1a320fc93863207ac7bcd2b`; only its pre-existing bundle ZIP was untracked. Consumer documents were read, not edited; fixture/native results were not rerun. |
| 2026-10-21 design correspondence | Mac authoring branch `docs/substrate-collider-contracts`, HEAD `aa5ef73ae087aa82d1f952ba6dd5c92361b8e24f`; accepted [reentry assessment §§4–6](../../guidance/2026-10-21-substrate-reentry-assessment.md#5-one-concrete-next-substrate-increment), SHA-256 `fe46f1050217240de01a67f20f637230c52140f0a66050114b30d68b43edce82`, and its preserved source/remote snapshots reused. Local source hashes/locators are distinguished from dated Linux `088f62d…` evidence; no new Linux/runtime proof. |

Consumer inputs: [boundary plan](../../../../collider/docs/agent-native/controller-ownership-plan/substrate-accepted-work-boundary-plan.md), [lifecycle mapping](../../../../collider/docs/agent-native/controller-ownership-plan/controller-owner-lifecycle-contract.md), [glossary](../../../../collider/docs/agent-native/canonical/00-collider-v1-glossary.md), and [foundations §11](../../../../collider/docs/agent-native/canonical/01-collider-v1-foundations.md#11-context-and-continuity-model). Cross-repository links assume sibling checkouts; pin the named Collider revision when handing this off elsewhere.

Initial authoring fence was this draft and the companion B03 draft. The 2026-09-20 user-authorized research follow-up additionally updates the core dossier, adds one research brief, and links it from the backlog. Existing canonical runtime owners, the historical dossier, source/tests, and Collider files remain unchanged. New operation names and result categories below describe meanings, not existing exports, Rust structs, HTTP routes, or a selected transport. The 2026-10-21 increment adds concrete proposed internal source interfaces, a local ingress transport and bounded responsibility amendments in §§3.2–3.4/B03; historical operation labels elsewhere remain semantic vocabulary. No generic facade subsystem is required merely to provide these operations.

**Current opening correction — 2026-10-21:** Collider initially opens a Substrate Workspace on the
same host. A selected folder/location is discovery input, distinct from Workspace identity and
from a Repository. Existing valid `.substrate` state loads through Substrate with current known
configuration, policy, Lanes and destinations; no separate Connect, enrollment, pairing or
setup-issuer approval is required. Genuinely absent state offers Substrate initialization, where
the user chooses/approves initial settings once. Invalid, incomplete, disabled or incompatible
state is diagnosed/refused, never silently repaired, re-enabled, regranted or treated as absent.
This human clarification supersedes the earlier external-client setup prerequisite; prior
acceptance records and their no-findings review remain history. Live Collider canonical
[Foundations §§3–4](../../../../collider/docs/agent-native/canonical/01-collider-v1-foundations.md#4-workspace-first-product-model)
and §§7–8/15 bind Workspace/presentation ownership; the human supplies this more specific open/init
flow. Those draft canonical documents do not claim the complete owner API already exists.

## 2. Shared boundary rules proposed for owner review

- Authenticate the real human/local caller or approved delegated caller. Workspace/Lane/World/profile fields supplied by a client are references, not authority. Read, reconciliation, and replay require current authorization too.
- Preserve the [existing authority map](../../../llm-last-mile/runtime-refactor/architecture/authority-map.md): HSA for session authority; dispatch for control orchestration; registry for acceptance; Supervisor for observation/terminal truth; policy owners for permissions; World/config owners for environment state. A UI, transport, or lane orchestrator is not a substitute durable writer.
- Expose capability support, current authorization, and availability/readiness separately. Unknown/unsupported is not false success or a frontend simulation of owner behavior.
- Maintain separate client request, accepted task/turn, logical worker, orchestration session, Lane, World realization, cancellation request, cursor, and UI projection identities.
- Retain strict schema/reference validation and version explicitly. Do not silently mutate existing V1 records or create compatibility-only execution paths to bypass missing contracts.
- No observer closure, timeout, missing tail, cancellation ACK, or process liveness manufactures acceptance or terminality. No retry after uncertainty creates new work automatically.
- Return structured outcomes and source references; separate human diagnostics from machine replies. Redact secrets and do not expose internal capabilities or provider credentials.
- New proposals below are contract-review candidates. Existing safety contracts remain controlling until reconciled; the draft does not promote any seam to `ContractCorrectAndProven`.

### 2.1 Bounded target surface and common reference rules

Before these execution operations, [B01-B03-P1](../../../llm-last-mile/runtime-refactor/slices/tasks/b01-b03-authenticated-preflight.md#substrate-workspace-open-load-and-initialize)
owns proposed native `WorkspaceOpenAdapter::{inspect_existing,load_existing,initialize_absent}`
and separate no-send `preflight_exact_destination`. Existing discovery/init/config/policy source
is reused through Substrate. Normal open/preflight is nonreconciling and creates no permission,
identity, B1 proposal, reservation, runtime or delivery effect; explicitly approved absent-only
initialization has separately declared writes and partial-failure/reopen behavior. Known state
can load with explicitly unsupported components; missing genuine required owner state cannot be
replaced by a path/session-derived ID or fixture. Loading is not execution readiness.

The following is the complete operation set for planning this experiment. Names are semantic labels; do not generate HTTP routes, Tauri commands or Rust imports from them. An implementation must publish one supported versioned binding and reject unknown versions before effects.

| Contract | Target operations | Planning commitment |
|---|---|---|
| B01 | Submit ephemeral work; Reconcile original request | Caller-scoped distinct intents, original immutable acceptance, explicit pending/refusal/unknown outcomes; no relaunch on uncertainty. |
| B02 | Read work; Observe work; Request cancellation; Reconcile cancellation | Exact accepted-work identity, coherent read/replay boundary, independent cancellation and terminal truth; no worker stop or pending-admission control. |
| B03 | List Lane Worlds; Validate selected destination; Read accepted binding | Existing owner-backed Lane/World selection with revision checks; immutable historical binding. Create/move/replace/promote are not consumer operations in this proof. |
| B04 | Resolve selected context for preparation; Read resolution/delivery provenance | Handbook-owned semantic resolution, Substrate-owned assembly/delivery, required/optional validation, explicit unsupported owner mappings. |
| B05 | Read Workspace requirements; Set one local requirement; Reconcile mutation | Revision-checked declaration edit with exact mutation result; no installation, provisioning or per-World override editor in this proof. |

All external references are typed by **owner, resource kind, owning scope and exact identity**, with revision/generation where their owner requires it. Their encoding is opaque to Collider; a path, display label, provider session, UI thread or equal-looking string is not an interchangeable reference. References carry no permission. Authenticate each operation and enforce current resource/read/action authorization; the principal comes from ingress, not a request field.

The supported binding must expose a capability descriptor for these operations: contract version, supported operations/reference kinds, authorization and availability as separate observations, and exact retry/history/replay limits. A limit is either an advertised finite bound, an explicit supported unbounded guarantee, or unsupported/unknown—never an invented default. Plan against expiry/gap handling, not fixed numeric retention. Capability discovery does not authorize effects; recheck at use. Missing ingress, delegation, Handbook mapping or runtime profile yields explicit unsupported/unavailable/denied behavior and cannot be filled by frontend execution or a fixture presented as owner truth.

**Planning default:** fresh ephemeral execution with an explicitly selected existing destination; no native-session continuation. The consumer can retain UI history, but cannot inject the entire transcript or continue a worker by implication. The full Substrate architecture remains broader than this target surface.

## 3. B01 — Authenticated submission and exact reconciliation

**Existing owners:** dispatch + HSA + Receipt Registry + E2 commitment + B2.2 receipt construction. [Receipt/supervision family](../../../llm-last-mile/runtime-refactor/seams/runtime-event-receipt-supervision-and-retained-runtime.md), [ephemeral receipt](../../../llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md), [B2.2 owner](../../../llm-last-mile/runtime-refactor/slices/b2-2-foreground-receipt-return.md).

**Runtime-profile readiness gate:** before running the consumer proof, name its supported ingress, runtime family/backend and version, action/tool capabilities, credential posture, and exact World execution route. Bind these to current source and applicable security-owner requirements, then prove the selected path on installed Linux. No such profile is selected or certified by this draft. Existing B1/B2.1/B2.2 receipt behavior is real bounded functionality; a receipt test or in-World process placement is not proof of credential isolation or complete side-effect mediation.

The [runtime dossier](../../../RUNTIME_REFACTOR_SESSION_DECISIONS_2026-09-18.md) records copied Codex configuration/auth debt (R16), incomplete E3 producer/custody/admission/retained execution and proof (R23–R28), and absent/partial D1 envelope, D2 brokerage and D3 joined proof (R29–R30). At the reviewed Linux `088f62d…` source binding, [world-service](../../../crates/world-service/src/service.rs) rejects V2 member launch as unimplemented. Credential-bearing E3/V2/retained routes therefore remain design-only here; the inherited copy bridge is not a substitute for their security contract. Any selected UAA shell/edit/write/tool/process/network/native side-effect channel must have applicable broker/enforcement evidence or be disabled. If that cannot be established for a proposed profile, keep it design-only rather than count it as the narrow proof.

This gate does not declare all ephemeral execution absent or require completion of every E3/D1/D2/D3 feature before any consumer experiment. A narrower supported profile may be proposed with its own explicit limits and evidence; it must not silently reduce required security or imply proof of excluded routes.

### Proposed operation meanings

| Operation | Request meaning | Result meaning |
|---|---|---|
| Submit ephemeral intent | Versioned caller request key; authenticated caller/delegation; Workspace; B03 concrete destination; typed action/prompt and B04 selected-reference inputs; permitted execution requirements. | Pending with a recoverable request reference and explicit admission/composition phase; conclusive pre-submission refusal; or exact accepted work/receipt and binding. Transport uncertainty is an unknown outcome, not an owner terminal result. |
| Reconcile request | Same authenticated request namespace/key, optionally its owner-issued correlation reference. | Exact known request disposition and original acceptance material if accepted; not-recorded/expired/unavailable distinctions where supported. Never resubmit from a read. |

Namespace a request key by authenticated principal, Workspace, and operation. Bind it to canonical semantic inputs before uncertain side effects. Include destination and selected-reference versions in equality; exclude incidental UI display/transport values. Canonicalization must be defined by the owner, not by hashing raw frontend messages or exposing secret-derived request material.

Same key and same immutable inputs joins the same request; reuse with changed inputs conflicts. Different keys with identical prompt text are distinct intents. A pending request may progress; retry does not mean freezing its first observed pending response forever. Once accepted, original immutable acceptance/policy/binding material is returned unchanged, alongside separately labeled current observations.

Do not promise exactly-once execution. An ACK-loss path must reconcile B1/runtime evidence and the original request, including a partially published receipt. Do not mark it refused merely because foreground reporting failed. Current read authorization and historical policy evidence are distinct; neither recomputes the other.

**Retention rule:** §3.3 requires explicit owner permission to retain original material/evidence until positive resolution, without a fixed cutoff that can expire while custody is unresolved; a finite-only incompatible grant makes the profile unsupported before possible send. Separately permitted finite retention can govern completed history after that resolution. Durable no-reuse tombstones remain; expired/unknown history never launches new work. Storage/recovery and service fencing are specified in B03 §§4.7–4.9; no current implementation is claimed.

**Hierarchy:** a direct Workspace-level redirection is a separate authorized control operation, not a changed-payload retry of the original request. Preserve causal links and inform the Lane coordinator. This draft does not define the redirection API or permit an AgentKit message to become that authority.

### 3.1 Request admission and consumer acceptance are separate

The following is a proposed result algebra, not exported enum names or a new runtime lifecycle. Existing owners remain authoritative; an admission correlation does not create a task, and a projection failure does not undo B1 acceptance.

| Result meaning | Required evidence / client behavior |
|---|---|
| `Pending(Preparing)` | Durable request correlation exists; submission has not crossed the guarded runtime boundary. Not accepted work. Only the admitted owner path may advance it after current authorization/preconditions pass. |
| `Pending(SubmissionUnresolved)` | Submission may have occurred; exact acknowledgement/acceptance is not yet resolved. Preserve the same request and proposed work identity. Never release it for a fresh launch because of timeout. |
| `Pending(AcceptedMaterialIncomplete)` | Exact B1 acceptance is known, but B2.1/E2/B2.2 or B03 consumer joins are not yet complete. Report that distinction without exposing private records; do not call it a pre-launch failure or return a fabricated receipt. |
| `Accepted` | Exact authenticated original receipt material and B03 binding are complete and durably recoverable. Return that immutable composition; current observations are separate. Work may already be terminal. |
| `RefusedBeforeSubmission` | Conclusive owner evidence that this recorded attempt was rejected before submission; no runtime acceptance is possible for it. Reason is privacy-filtered. Changed authorization or destination requires a new explicit intent, not mutation of this result. |
| `KeyConflict` | This key is already bound to different canonical inputs. The original request is unchanged and may still be running; conflict says nothing about its terminality. |
| `NotRecorded` / `ExpiredHistory` / `Unavailable` | Read coverage is distinguished from absence. None authorizes resubmission or implies no work ran. `NotRecorded` is only a coherent lookup observation, not a reservation against a concurrently arriving submit. |

Authentication/authorization failure, invalid envelope, and unsupported version are access/validation failures, not dispositions of an inaccessible original request. Transport loss is caller uncertainty. Neither may be collapsed into `RefusedBeforeSubmission`. After submission, only exact owner evidence may establish work state; do not force runtime launch failure into a pre-submission category. If recovery cannot establish an outcome, keep it unresolved.

`ReconcileRequest` is read-only: it may report newer owner recovery evidence but never resubmit, repair indexes, publish missing joins, or reconstruct historical policy/binding. Recovery/publication belongs to an explicitly authorized existing-owner path, outside the read. The E2-RM projection remains non-reconciling and consumes historical bytes, not current policy or supervisor revisions.

### 3.2 Immutable request identity and the admission boundary

**2026-10-21 target design:** the interfaces and placements below are expressly proposed internal extensions. They are sufficient to delimit source implementation; they are neither current exported APIs nor an installed consumer profile. The authorized contract design assigns responsibilities without claiming component-owner signoff or implementation admission. [B03 §§4.6–4.9](lane-world-binding.md#46-recommended-model-authorized-selection-not-session-rebinding) defines the matching transaction, effect fence and recovery composition.

```text
RequestKey = (authority_store_id, stable_principal_ref, workspace_ref,
              operation_version, client_key)
Submit = (schema_version, client_key, workspace_ref, destination_expectation,
          action_and_payload, selected_context_versions, execution_requirements,
          optional_delegation_selector)
Reconcile = (schema_version, workspace_ref, operation_version, client_key)
```

**Initial native owner binding:** proposed `execution/workspace_open.rs::WorkspaceOpenAdapter`
receives Substrate's actual same-host native caller and trusted root/store context. Collider is
presentation over that owner path; a request cannot choose its authoritative principal/root or
call generic World execute as B01. Existing configured opening needs no enrollment, client key,
challenge-signing ceremony or new permission approval. Current operation/resource/read checks
remain internal and are repeated at use. The adapter and full consumer Submit/Reconcile binding
are proposed, not present callable exports or installed proof.

Proposed `agent_runtime/workspace_admission_authority.rs::read_workspace_identity` validates
genuine Workspace/root/store state; `resolve_execution_delegation` still checks the exact
initiator, executing session/caller/backend/World and current operation/resource scope before
later execution. HSA supplies canonical actor facts, not a grant inferred from folder possession.
No approved execution mapping means denied/unsupported, never a fabricated orchestrator.
`resolve_current_request_read` remains separate current history authorization. Normal Workspace
loading does not issue a delegation or invoke the reconciling HSA resolver; the packet names the
required nonreconciling reader.

**Separate later ingress candidate:** the earlier proposed private Unix-socket
`ConsumerAdmissionIngress::{submit,reconcile}` / `WorkspacePrincipalDirectory::resolve_peer`,
peer-bound enrolled-client key and incarnation-bound nonce proof remain a possible later
external/third-party ingress design. That protocol is outside the initial native open/load/init
path and is not its prerequisite. Network/browser exposure still requires its own supported
binding; this correction creates no new enrollment or grant framework.

The first implementation route fixes one eligible existing exact session/caller/World and its existing backend owner. It preserves current session/World equality, singular HSA binding and original actor through retries. General selected-World dispatch needs its own composite resolver/custody extension; changing a steering flag cannot supply it. Credential rotation preserves a principal only through the authenticated directory mapping. A new current coordinator cannot silently replace the original execution actor.

**Complete semantic equality:** use a versioned tagged, length-prefixed canonical encoding with stable integer/enum forms, exact action/prompt UTF-8 bytes, explicit optional-versus-absent and ordered selections. Sort only fields declared sets by their owner; reject duplicate set entries. Include exact destination/association revision; action/payload; runtime/constraint requirements; delegation selector; all original B04 owner-qualified kinds/identities/revisions/requiredness, purpose/action, Scope, Resolution Posture, fidelity and budget semantics. Incidental display, connection and UI episode values are excluded. Namespace operation version and canonical-encoding version are distinct explicit fields.

Proposed dispatch-owned `agent_runtime/consumer_admission.rs::bind_or_read_request` binds that namespace once and allocates one correlation and fixed random B1 `request_id`. B1 alone allocates the proposed acceptance identity through its existing proposal allocator. Equality uses constant-time comparison of domain-separated HMAC-SHA-256 commitments over canonical semantic bytes, a random per-authority protected key, encoding version and key ID. Reuse a compatible existing keyed-commitment primitive where available. Authenticate the entire stored correlation separately, including its generation and all joins. A raw prompt hash, frontend hash or caller digest is not equality authority; no prompt/payload preimage or secret-derived raw digest is added to the correlation.

**Original B04 material:** proposed `execution/consumer_context.rs::prepare_original_context` consumes supported H1–H3 results and retains the actual ordered execution envelope in S1's protected material custody. An immutable prepared-material seal joins its owner-qualified source/dependency revisions, dispositions, representation/fidelity, authority/current-use basis, material/envelope identities and commitments to the original request. These are owner outputs, not new caller equality fields regenerated on retry. Caller-supplied material, if later supported, is itself canonical input requiring owner validation. Preserve original required and optional selections even after permitted omissions. Required failure prevents new submission; optional omission follows Handbook's exact result. A selected reference or resolution receipt is not material or runtime delivery.

Same-key Submit first authenticates namespace/current read access and compares the original semantic commitment. Equal contenders join the same proposal/seal; changed input conflicts without mutating the original. It never resolves latest Handbook content, chooses a new current Lane or substitutes an execution session to make a retry pass. New keys with identical prompt text remain distinct intents and validate the current destination. Derived policy changes cannot redefine accepted equality or historical material.

**Admission sequence:** authorize access → bind/read original key → reserve exact association and B1 proposal through their idempotent owner calls → retain/seal original material and policy inputs → exact readback of all joins → current owner-revision gate validation → durable possible-send marker → runtime consume → original B0/B1/B2.1/E2/B2.2/B03 composition. [B03 §4.9](lane-world-binding.md#49-durable-request-join-and-recovery-candidate) specifies interruption boundaries. No partial join can send; no persisted preparation freezes permission; no global persistence lock crosses runtime I/O.

### 3.3 Consumer boundary candidate and failure mapping — 2026-09-21

The heading preserves its existing link. The controlling 2026-10-21 refinement uses this one result family for Submit and Reconcile; names are semantic types, not selected serialization or exported enums:

```text
RequestReply = AccessOrValidationFailure(reason)
             | RequestObservation(request_ref, admission_revision, disposition)
             | LookupObservation(NotRecorded | ExpiredHistory)
             | ReadFailure(reason, authorized_request_ref_if_available)
```

`request_ref` is an opaque owner-qualified correlation, never a bearer capability, B1 work ID or cancellation target. Its revision orders this request's admission facts only; B02 work/observer revisions and B03 association revision remain separate. `KeyConflict` leaves original revision/work unchanged. No absent/expired lookup mints a request reference; unavailable lookup is a read failure. An unauthorized caller learns neither existence, equality, refs nor private material; privacy-filtered access diagnostics may collapse denial/unknown-target without conflating machine dispositions for authorized callers.

| Boundary outcome | Consumer mapping and required evidence |
| --- | --- |
| Authentication, current namespace/read access, strict envelope/version or enrollment validation fails | `AccessOrValidationFailure`; no inaccessible request disposition or equality disclosed. Strict decoding/authenticity alone does not authorize. |
| Existing key with changed canonical semantics | `KeyConflict` after authorized lookup; original proposal/input/destination remain unchanged. |
| Durable correlation, incomplete preparation | `Pending(Preparing)`; only the authorized owner advancement path may progress it after current checks. |
| Current permission, association, required context, capability or custody fails before any possible submission | `RefusedBeforeSubmission` only after closed correlation generation, durable runtime `NeverConsumed` fence and participant close/no-send ACKs exclude past/future send. Otherwise preserve pending uncertainty. |
| Durable `MaySend`, send error/EOF/timeout, ACK loss or absent durable Start | `Pending(SubmissionUnresolved)` until exact gate/producer/B1 evidence resolves it. Missing observation cannot prove no effects. |
| Exact B0/B1 acceptance known, one required composition join missing | `Pending(AcceptedMaterialIncomplete)`; retain original acceptance, never fabricate refusal or task terminality. |
| Exact authenticated B1/B2.1/E2/B2.2/B03 joins durably complete | `Accepted`, even if work is already terminal. Return immutable historical composition and separately labeled current observations. Runtime consume and task acceptance still do not prove B04 delivery. |
| Previously complete composition unexpectedly unreadable/corrupt | `ReadFailure` for unavailable/integrity; preserve independently verifiable known acceptance under current access. Never regress to preparation, reconstruct current binding or silently classify expiry. |
| Authorized completed-history material deletion after positive resolution under the published retention policy, with authenticated deletion/tombstone evidence | `ExpiredHistory` for original material no longer retained; retain immutable nonsecret disposition/key identity, never reuse or relaunch. This is distinct from unexpected read failure. |
| Coherent absent lookup | `NotRecorded` is an observation only; a racing Submit may bind later. It is not a reservation, no-work guarantee or permission to retry execution from the read. |

**Exact read/composition contract:** proposed `read_request` obtains a non-reconciling authenticated snapshot and verifies the original authority store/session/caller/backend, request/proposal/acceptance/task identity, B0 stream/acceptance sequence, B2.1 claim, immutable E2 material, B2.2 receipt and original B03 reservation/binding/material seal. Authentic unrelated records fail the join. Reconcile never publishes a missing binding, repairs an index, initializes/rotates a key, replays `execute_stream` or reruns context. Existing write-capable StateStore helpers are not read-only merely because they return an observation; provide a non-mutating adapter patterned on the E2-RM snapshot boundary. Current read denial can hide history without changing it. Full retained bytes require their own disclosure authorization.

**Retention selection:** no automatic key reuse, history expiry or garbage collection in the first implementation. Retain namespace/key tombstones for the authority-store lifetime. Before admission, require **explicit owner permission covering unresolved original material, correlation and gate/owner evidence custody until positive resolution**, with no fixed retention cutoff that can expire while those obligations remain unresolved. This permission must cover the actual original envelope/material and required recovery evidence, not only their references. An owner may decline that permission: a finite-only grant incompatible with unresolved custody makes the profile unsupported/refused **before possible send**, and never licenses emergency deletion, new execution, re-resolution, implicit renewal or unilateral permission extension. Positive resolution requires the original no-send close or exact terminal-plus-state/custody evidence in B03 §4.9; acceptance/composition alone does not close outstanding work/custody uncertainties.

Completed history may separately have a positively permitted finite horizon and authenticated deletion rules, applicable only after that positive resolution is recorded. Authorized completed-history deletion preserves a nonsecret authenticated disposition/key tombstone and yields `ExpiredHistory`; unexpected loss or corruption instead yields `ReadFailure`. No timer authorizes deletion of unresolved custody. **Retention permission does not authorize source access, disclosure, execution or later use**; current source/use revocation, service fencing and later delivery/current-use guards remain separate and fully effective. Old commitment keys remain verify-only while retained records need them; key loss/corruption is integrity failure, never a clean absent key. Publish the admitted profile’s explicit unresolved-custody permission and any distinct completed-history horizon, rather than inheriting fixture limits.

**Source correspondence:** current Mac `aa5ef73…` locators are `orchestrator_world_dispatch.rs::dispatch_orchestrator_world_request_for_principal:1907` (accepted retry precedes fresh authority/principal use), `prepare_task_acceptance_submission:4675`, `launch_task_supervision_and_handoff_receipt:9000` and `observe_run_world_task_stream:9085`/`execute_stream:9103`. `state_store.rs::resolve_world_work_registry_authority:5321–5428` enforces exact actor/World; `persist_world_work_acceptance:4871` requires an exact proposal. These are local source observations, not newly verified Linux bytes. The retained September source pass and October reentry snapshots bind Linux `088f62d…`; their qualified findings remain valid, including generic transport errors' inability to prove no-send and the absence of consumer ingress/delegation/fencing. No Linux mutation/probe was performed here.

### 3.4 Joint design acceptance cases and remaining closure gates

The target extension is concrete; current real-profile support remains **unsupported/unqualified**. No installed runtime family/backend/version, consumer ingress or end-to-end H1–H3/S1 route is certified by these drafts. Do not invent a fake agent, empty context or copied-credential route to convert that negative finding into readiness.

Before later execution, pin one `AdmissionProofProfile` manifest and complete its handshake:

| Manifest field | Required positive binding or refusal |
| --- | --- |
| Source and route | Exact Linux source/installed artifact/service build; eligible existing HSA session/caller, World/generation and backend custody; proposed native Substrate ingress/current execution-delegation/read binding (separate later external ingress only if selected); selected versioned execute carrier. |
| Real runtime | Actual family/backend ID, official executable/version/artifact digest and exact permitted action/tool capabilities. No default backend/version or fixture qualification. |
| Security | Credential source/posture and every shell/edit/write/MCP/tool/process/network/native channel's concrete broker/enforcement evidence or disabled status. Placement is insufficient. |
| Semantic material | Supported Handbook H1–H3 composition, conditional H4 population, source/disclosure/current-use scope/lifetime, original retained envelope/seal and later delivery guard. Require explicit original-material/evidence custody permission until positive resolution without an expiring unresolved cutoff; finite-only incompatible permission refuses before possible send. Record any separate completed-history horizon/deletion policy. Retention never grants use. |
| Fence handshake | Service incarnation, supported open/close/consume/fence/read versions, enrolled owner identities and current owner-revision vector; close-before-effective-commit participation of every applicable policy/delegation/source/disclosure/current-use/destination/custody invalidator. Old/mismatched service fails before `MaySend`. |
| Proof controls | Independently observed execute/acceptance/delivery counts and identities, crash barriers, operator recovery/status/retention limits and authorized installed commands chosen at implementation admission. |

A real UAA/native-agent profile depends on D1's exact execution envelope, D2 broker-or-disable coverage for every exercised channel and D3 joined proof. Credential-bearing E3/V2 retained execution additionally remains subject to current V2 rejection, F-work and I1/I2; it is not selected here. A no-secret narrow profile is possible only if its real action and all applicable owners support it with the manifest above; none is currently established. S1 acceptance retains original material, while actual delivery/current-use authorization and evidence remain a later guard. Fixture UI planning can proceed independently; no fixture closes B04 or Gate C.

Required later cases, none executed here:

| Case | Owner-visible proof |
| --- | --- |
| Native caller/Workspace/delegation spoof (or enrolled-client spoof for a separately selected external route), wrong root/custody or current read denied | No forbidden existence/material disclosure or new effects; exact typed access/unsupported result. |
| Concurrent equal key, changed-input key and distinct identical-content keys | One original correlation/proposal/attempt for equal requests, immutable conflict for changed semantics, distinct intents for distinct keys. |
| Original B04 material after latest source/selection change; missing/stale/denied/malformed required input; optional omission | Original seal/input custody on retry; owner-current-use rules enforced; required refusal before effects and permitted optional disposition preserved. |
| A→B→A association, unrelated sibling revision and duplicate after a permitted move | New intent checks monotonic selected-World revision; duplicate returns original authorized history; no stale revival or sibling-wide lock. |
| Crash after each reservation/proposal/material join; stale writer resumes | Exact reverse-link recovery/close ACKs, no new proposal or sender authority; no safe-abandon inference from absence. |
| Revocation close-first versus consume-first, close ACK lost, service incarnation changes, delayed old sender | Same gate order, close before effective commit, original close recovery, durable attempt tombstone; no bypass or clock/lease substitute. |
| Before/after `MaySend`; after consume/before producer; Start-before-B1; B1-before-claim/E2/receipt/binding; complete-return ACK lost | Honest Preparing/Unresolved/Incomplete/Accepted transitions; same original identity; no re-execute; exact original ACK evidence only. |
| Finite-only custody grant whose cutoff can arrive while submission/consume evidence is unavailable | Profile unsupported/refused before possible send, with zero execute/consume effects; no renewal assumption, emergency deletion, re-resolution or permission extension. Separately prove permitted finite completed-history deletion only after positive resolution. |
| Gate corruption, missing authenticated history, forged `NeverConsumed`, unexpected complete-material read failure and authorized retention deletion | Fail closed; distinguish integrity/unavailable from ExpiredHistory; no key recycling or custody release from uncertainty. |
| Reconcile concurrent with Submit/recovery and after host/service restart | Byte-identical read, coherent revision or explicit failure, no hidden owner repair and no execute count increase; producer replay availability reported honestly. |

**Later narrow source catalog:** new shell ingress/admission/Workspace/context modules and registration; existing dispatch/state-store/HSA trusted persistence adapters; B1 reverse proposal link and exact close/read primitives; existing Supervisor/E2/B2.2 joins only as needed; versioned transport types/client; service gate and exact execute integration/original-Start retention. No provider or retained lifecycle rewrite, scheduler, general multi-World custody, filesystem selection, Collider or Handbook source is authorized by this documentation. Runtime symbols need their own impact analysis and implementation/proof assignment. This task's exit is reviewed, source-bound working-tree contract design and docs checks; implementation, real owner signoff, installed proof and canonical runtime closure remain separate.

## 4. B02 — Exact-work read, observation, and cancellation

**Existing owners:** registry, Supervisor, runtime event transport, WorldDispatchControl; RetainedWorkerRuntime only for actual worker lifecycle. [Cancel categories](../../../llm-last-mile/runtime-refactor/contracts/cancel-outcome-categories.md), [B4 owner](../../../llm-last-mile/runtime-refactor/slices/b4-receipt-targeted-cancel-inspect-stop.md).

### Proposed operation meanings

| Operation | Input | Result |
|---|---|---|
| Read accepted work | Authenticated caller and exact owner receipt/work reference. | Coherent exact-work observation cut: acceptance reference, immutable B03 binding, owner state/revision, optional exact terminal result, observation completeness/availability, and source provenance. |
| Observe accepted work | Exact work plus owner cursor/observation epoch or an explicit initial-read request. | Authorized snapshot/replay/live continuation with a declared cut, duplicate handling and retention limits, or an explicit stale/gap/unavailable result. |
| Request cancellation | Exact accepted work; caller cancel-request key; current authorization. | Original owner cancellation identity when recorded, explicit refusal/invalid target/unreachable/unknown, pending closeout, or already-terminal evidence. |
| Reconcile cancellation | Exact work and cancellation correlation. | Current request disposition plus independently authoritative work outcome. No second cancellation intent merely because the ACK was lost. |

Bind receipt revision, stream/frame cursor, semantic-event identity, and observer epoch separately. Read-cut/replay-to-live handoff must avoid unreported gaps; a cursor is not a universal Workspace sequence or an AgentKit sequence. Exact retention counts, source epochs, durable-prefix boundaries, concurrent subscription limits, and transport binding remain owner-defined, not inherited from old planning constants.

Producer disappearance may leave an unresolved nonterminal claim. Preserve available history and expose an unavailable tail; do not replay by rerunning the prompt. UI unmount/disconnect only detaches observation. Owner recovery guarantees after shell/service restart must be stated and tested separately, not assumed from Lane durability.

Preserve the existing accepted-work cancellation distinctions: pending closeout, already terminal, no active cancelable work, unreachable owner, invalid target, World mismatch, ambiguous target, and policy denial. Only exact work-terminal evidence yields a cancelled task/turn. Completion can win before or after cancellation acknowledgement. Worker stop, turn/task cancel, and pending-admission cancel remain different operations.

The [current dispatch outcomes](../../../crates/shell/src/execution/agent_runtime/dispatch_contract.rs) also include `CancelledBeforeTransport` and `AlreadyRoutable` for pending-admission control. B4 further requires distinct cancelled-before-registration, registered-before-transport, transport-ambiguous/cancel-pending, already-routable and terminal dispositions. These describe admission progress, not interchangeable accepted-task terminal states. The bounded operations above target accepted work only: pending-admission cancellation is explicitly unsupported by this proposed surface until separately specified using exact admission identity. If exposed later, preserve these outcomes and their owner projections losslessly; never substitute an accepted-work receipt, `NoActiveCancelableWork`, or a fabricated task terminal result.

Expose `Completed`, `Failed`, `Cancelled`, `NeedsRetainedFollowup`, and `Invalidated` losslessly where the selected owner supports them. `NeedsRetainedFollowup` ends ephemeral work; it creates no worker or continuation route. Do not invent a terminal wall-clock timestamp: absent source time stays absent, even if the UI uses a labeled presentation timestamp.

**Hierarchy extension:** authorized redirection must produce coordinator-visible causation/state updates without a hidden competing instruction chain. Ordering, stale-plan rejection, notification recovery, and escalation acknowledgement need their own messaging/control contract. They are not implied by observing a receipt and need not be implemented for the one-task proof.

**Proof required:** acceptance without running; immediate terminal; two authorized views of one work item; disconnect/reconnect; duplicate/conflicting/out-of-order frames; missing tail and producer restart; cancel/completion races; late cancellation ACK; exact terminal readback; nonstandard terminal mapping; unauthorized read/cancel; explicit rejection of unsupported pending-admission targets without substituting task cancellation. Any later admission-control extension must prove lossless B4 outcome mapping. Unsupported observations remain visible as such.

### 4.1 Planning target: read/replay and restart behavior

```text
ReadWork(exact_work_ref)
  -> WorkSnapshot(acceptance_ref, accepted_binding, observation_revision,
                  work_state, terminal_or_absent, coverage, resume_cursor_or_absent)
ObserveWork(exact_work_ref, after_cursor_or_initial)
  -> SnapshotThenEvents | ReplayThenLive | TerminalSnapshotOrReplay
   | GapOrExpired | InvalidReferenceOrCursor | Unavailable | Denied
```

The snapshot is one exact-work owner cut, not a globally ordered Workspace view. `coverage` identifies the durable prefix, whether a live tail is available, and any unresolved gap. A cursor is opaque and bound to that work, stream/epoch and any server-side selection semantics; a cursor from another work item or incompatible epoch is refused. Authorize before exposing hidden work, events or gap details.

- **Initial observation:** return an authenticated snapshot and a continuation cursor whose meaning is strictly after that cut. Buffer or replay races between snapshot and live attachment; if the owner cannot bridge them, emit an explicit gap rather than claim seamless coverage.
- **Reconnect:** replay from the supplied cursor then join live delivery without unreported omissions. Duplicate owner event/frame identities may occur across reconnect; the same identity must not have conflicting content. Collider deduplicates by owner identity and keeps its own projection sequence separate.
- **Expiry/gap:** return a reason and an authorized fresh-snapshot path. A new snapshot can restore current state, not fabricate missed intermediate events. Never execute the prompt again to recover history.
- **UI/process lifetime:** observer detach does not cancel. A submitting command's return is not task lifetime. After host/service restart, expose authenticated persisted acceptance/history and exact recovered observation where supported; unavailable live routes or tails remain unavailable, not Failed/Cancelled or silently resumed execution. No provider-process survival or complete runtime-tail replay is promised by Lane durability.
- **Retention:** the supported binding must advertise retention/expiry and cursor validity. Reaching the advertised boundary yields explicit expired history; changing limits cannot silently make an old cursor mean a new stream. Current internal buffer sizes are not the target API's guarantees.
- **Terminal:** source-owned terminal evidence is monotonic. Return the actual result class, including `NeedsRetainedFollowup`; cancellation acknowledgement and stream closure cannot synthesize it. A terminal snapshot/replay may complete observation without opening a live subscription; completion of delivery still reports whether requested history was complete. Missing source timestamps stay absent.

### 4.2 Planning target: cancellation identity and results

`RequestCancellation(exact_work_ref, caller_cancel_key)` binds one authenticated caller/Workspace cancellation intent to that immutable work reference. Same key/same target returns the original recorded cancellation identity plus separately labeled current work outcome; same key/different target conflicts. A lost acknowledgement is reconciled by that original key, without requiring a token from the lost reply. Reconcile is read-only. A new explicit cancellation key does not authorize targeting a newer run or creating a second worker-stop operation.

Preserve B4's exact categories: `CancelledViaLiveTransport` only with terminal evidence; `CancelAcceptedPendingCloseout`; `AlreadyTerminal`; `NoActiveCancelableWork`; `OwnerUnreachable`; `InvalidTarget`; `WorldBindingMismatch`; `AmbiguousTarget`; and `PolicyDenied`. Redact forbidden candidate identities in errors. A transport failure before the caller learns whether cancellation was recorded is **unknown delivery**, not one of the terminal categories; retain the key and reconcile. Expired/unavailable cancellation history never implies the work was cancelled. Completion may win; return the winning terminal result without rewriting it as cancellation. Pending-admission targets and whole-worker stop return unsupported for this surface, not a fabricated task outcome.

**Existing versus required:** Supervisor/B4 supply internal work truth and cancellation distinctions. Authenticated consumer exposure, cursor/read-cut protocol, caller cancellation-key mapping and advertised lifetime/retention remain owner extensions or integration work. These are specified target behaviors, not a claim that raw runtime replay is already a supported GUI API.

## 5. B03 — Lane and World binding

Use the [companion B03 proposal](lane-world-binding.md). It recommends Workspace-scoped association ownership plus authorized selected-World dispatch, with source-backed existing-owner changes and alternatives. It supplies the identity split, concrete selection, admission/association crash boundaries, historical/current association distinction, and Lane facts versus derived state. The recommendation is not yet an approved canonical owner contract.

The selected task needs one validated existing Lane/World destination and exact accepted binding. It does not force one World per Lane, a new Lane per task, a continuously running Lane orchestrator, or a complete promotion API. Readiness for the narrow proof still requires a genuine owner-backed association—not a fixture Lane ID taped onto a receipt.

## 6. B04 — Typed selection, semantic resolution, and delivered context

**Owners:** Handbook owns semantic Scope/Resolution; Substrate owns authorized assembly and actual execution delivery; Collider owns selection/presentation, not semantic resolution. [Consumer foundations](../../../../collider/docs/agent-native/canonical/01-collider-v1-foundations.md#11-context-and-continuity-model) establish the consumer requirement. The bounded 2026-09-21 Handbook inspection below identifies real owner primitives and its consumer boundary; it does not establish an existing end-to-end per-invocation context API.

### Proposed operation meaning

Inputs identify purpose/action, permitted typed source/artifact/work-item references with their owner and revision, current Scope/Resolution posture, and the validated Workspace/Lane/World destination where applicable. A selector is not a grant to read its target. Define allowed reference kinds against the actual Handbook contract before writing wire types.

The handoff is: caller selects → authorized Handbook resolution → Substrate verifies execution visibility/policy and assembles → exact execution receives the admitted material. Resolution or assembly can fail without accepting a World task. Any preparation after acceptance must use explicit owner work-state semantics, not erase accepted work on error.

Recommended first-path handling: required missing, stale, malformed, or unauthorized references refuse preparation; do not silently substitute latest content. Optional omission is permissible only if explicitly declared and reported with provenance. Revalidate effective authority/binding before delivery; stale authority cannot be rescued by reusing a previous context pack.

Return/link distinct resolved and injected records bound to the actual work. Preserve acquired and produced lineage as execution supplies it; no full context inspector is required. A claim of delivery needs runtime-path evidence, not merely a successful resolver call. Transcript history can be selected material, never implicitly the complete injected context.

Fresh reconstruction is the default interaction posture; long-lived logical workers and durable orchestration identities remain allowed. Native/provider continuation is explicit and separately qualified, not inferred from a Lane view or adjacent message. The selected ephemeral proof does not require Qualified Session continuation.

**Proof required:** valid selected references reach the real execution; stale revisions, denied visibility and wrong Lane fail; required missing input is not silently dropped; a same-UI-thread invocation does not silently resume native context; resolved and injected provenance are distinguishable; permission/binding drift between selection and delivery is handled safely.

### 6.1 Handbook source binding and target adapter contract

Historical read-only Handbook inspection: `/Users/spensermcconnell/__Active_Code/atomize-hq/handbook`, then clean `8605d6539355f350c55b921d95fb450bca30ee18`. At that pin, `docs/specs/handbook-contract-membrane/00-README.md` and `slices/HCM-4.1/SPEC.md#non-negotiable-ownership-boundary` distinguished semantic owner crates, ordinary-consumer SDK composition and transports. Those historical paths are absent from the current sibling checkout; they are retained as commit-qualified provenance, not active link targets. The October reconciliation/reentry evidence controls the current H1–H3, conditional H4 and S1 gaps cited by §3.2. Prefer the supported SDK boundary for ordinary use; any direct advanced-owner integration must be explicitly supported, not inferred from Rust visibility.

The [SDK source](../../../../handbook/crates/sdk/src/lib.rs) provides `HandbookSdkV1::generate_packet` and `inspect_packet`; their [request types](../../../../handbook/crates/sdk/src/flow_api.rs) select a `packet_id`. The engine exports exact Context Resolution bindings/envelopes and a [grounding request](../../../../handbook/crates/engine/src/grounding.rs) over snapshot/delta/definition/disclosure references plus an envelope. These are real foundations, **not interchangeable with arbitrary Collider selected-reference lists**. Do not invent `resolve_context` as an existing export, relabel a packet ID as a Lane/Scope, bypass currentness through private state, or infer production Substrate delivery from a successful packet resolution. Documentation closure/publication histories were not re-audited.

Target adapter input is `ContextSelection(purpose, requested_action, exact_owner_scope_ref, resolution_posture_ref, selected_refs[], destination_expectation)`, where each selected entry carries an owner-qualified kind/identity, exact revision or owner-defined content fingerprint, and `required` or `optional`. Supported kinds come from the actual Handbook adapter capability descriptor—not a fixed list invented by Collider. Arbitrary filesystem paths and unversioned display names are not semantic references. An empty selection is explicit and still subject to mandatory owner context; it is not permission to bypass Handbook requirements.

Target outcomes are `Resolved(resolution_ref, exact_inputs, included_refs, omissions, owner_provenance)` or a typed refusal: unsupported kind/operation, malformed reference, missing required input, stale revision/currentness, authorization denial, incompatible Scope/Resolution, or owner unavailable. Required failures stop B01 preparation before submission; optional omission is allowed only when declared and permitted by the owner and must carry a reason. Security denial details are redacted. No silent substitution of latest content, escalation of Resolution, or downgrade of a required reference is permitted.

Substrate then validates execution visibility and assembles the admitted context against the same destination and current authority. It records the resolution-to-assembly link and, only on actual runtime delivery evidence, an `Injected` record bound to the exact work and delivered material identity. Resolution/assembly success alone yields **not-yet-delivered** status, not an injected record. Delivery failure after acceptance uses existing work outcome/recovery semantics; it cannot erase acceptance. `Acquired` and `Produced` are separate execution provenance when supported, not fabricated empty completion records. A lineage read authenticates access and distinguishes absent, pending, unavailable and recorded stages.

**Planning disposition:** this target lets Collider design selection, refusal and lineage handling now. The precise Handbook operation/ref-kind mapping and its released/supported consumer binding require joint owner agreement before implementation. Until that binding exists, return explicit context capability unsupported; do not substitute frontend resolution or a raw prompt/transcript workaround. This is a named cross-repository delivery dependency, not a reason to postpone the other four target contracts or claim Handbook has no usable foundations.

#### Outstanding positive-path binding for the selected B04 case

This is a concrete cross-owner mapping dependency, not merely missing serialization. The entries below are **open**; no callable operation, supported reference kind or runtime profile is selected by this clarification. Establish one representative case satisfying the actual experiment, not a new arbitrary-selection API.

| Binding item | Required owner answer / remaining verification |
|---|---|
| Selected reference | Handbook: supported kind, identity/revision, purpose, Scope, posture and required/optional representation. |
| Supported operation | Handbook: exact published SDK or explicitly supported advanced-owner operation and mapping of every input into its real request type; public Rust visibility alone is insufficient. |
| Validation/currentness | Handbook semantic checks plus Substrate access/execution visibility: who checks revision, disclosure, scope and authorization, and when admission/delivery revalidates them. |
| Success/refusal | Handbook: actual result/provenance/omission fields and typed failure sources. `FlowRouteError` diagnostic text alone is not the required machine contract; inspect structured results/owner APIs without inventing error-string parsing. |
| Delivery and identity | Substrate: exact preparation/assembly and runtime-delivery path; join selected inputs and resolved material to the original request/accepted work; actual evidence for `Injected`, including post-acceptance failure. |
| Evidence | Both owners: selected-path positive, missing/stale/denied/wrong-binding and lineage checks. Existing constructors or packet success do not prove delivery. |

Target mapping can proceed with this row open. Real B04 proof and the complete selected Gate C experiment require its closure plus the other applicable owner/runtime prerequisites. Separately authorized B05 or other partial proofs do not establish full Gate C; a positive task dispatch still must satisfy mandatory context on its real route. Empty selections or raw prompts are not bypasses. No new experiment is authorized here.

## 7. B05 — Revision-aware dependency requirement mutation

**Existing owner references:** [World deps](../../reference/world/deps/README.md) distinguishes inventory, enabled declarations, and applied status; [internals](../../internals/world/deps.md) describes owner implementation concerns. A bounded source inspection below covers the main declaration writers, not an exhaustive all-writer audit or executed mutation test.

### Proposed bounded operation meanings

| Operation | Input | Result |
|---|---|---|
| Read Workspace requirements | Authenticated caller and Workspace reference; optional page cursor. | Authorized known items, provenance, Workspace-local declaration versus inherited/effective enablement, editability, and opaque revision covering relevant preconditions. Applied status, if included, identifies the exact World and observation freshness. |
| Set Workspace requirement | Canonical item reference, desired Workspace-local enablement, expected revision, mutation key, current caller authorization. | Exact committed revision and authoritative result readback; no-op disposition; conflict; denied/invalid/unsupported; or unknown outcome requiring reconciliation. |
| Reconcile mutation | Caller-scoped mutation identity. | That operation's committed result/revision or explicit unresolved/refused/expired state—not an inference from the latest value alone. |

Removing a Workspace declaration does not necessarily disable an inherited dependency and does not uninstall anything. The boolean edit means the local declaration, not an arbitrary override of all effective enablement. The inherited contribution is read-only. An item with both inherited enablement and a Workspace-local declaration may edit only that local contribution when the owner explicitly marks it editable; removal can leave effective enablement true. Inherited-only rows default to read-only unless the owner explicitly permits adding a local declaration. Unknown/uneditable entries remain read-only. Never infer editability from the effective boolean.

The expected revision must cover the actual Workspace patch and relevant inventory/parent/config inputs used to validate the edit. Under the owner's serialization boundary: validate caller, compare relevant preconditions, normalize the operation, persist the declaration and recoverable mutation result, and return that committed revision/readback. Competing stale writers receive conflict without partial mutation. Identical retries join the same operation; changed-payload reuse conflicts.

All supported writers of the covered patch must participate in this protocol or be explicitly excluded by an accepted contract. A UI-only mutex or atomic rename does not provide compare-and-set across other supported CLI/config writers. Unsupported external edits are outside that guarantee: detect changed preimages where possible and refuse/reconcile; do not claim protection against arbitrary concurrent host writers without the required filesystem enforcement.

If another authorized mutation follows an ACK-lost success, the latest value may differ; preserve the earlier operation's committed result rather than reporting it never happened. A mutation that has not been durably correlated cannot be called safely retryable.

### Full-architecture compatibility, without expanding the proof

SCA-10 requires shared Lane baselines and explicit per-World overrides in V1. The eventual owner contract must distinguish inherited baseline, explicit overrides, effective desired configuration, and applied World state. Dependency/configuration merge rules must not masquerade as policy widening. Destination-baseline reconciliation on reassociation remains explicit; stable files do not excuse ignoring security revocation.

The current proof edits only Workspace declarations. It neither exposes Lane/World override editing nor certifies their V1 implementation. A declaration edit does not install/provision, refresh a World, merge code, or silently mutate active working state. Existing docs separately prohibit runtime system-package mutation and host-native provisioning of those packages; do not promise arbitrary OS dependency variants from an override alone. Unsupported environment realizations must be reported, not satisfied by granting host mutation or silently adding a deferred remote/container backend.

**Proof required:** authorized read/add/remove/readback; inherited versus local enablement; stale and concurrent mutations through every supported writer; inventory/parent drift; invalid item and denial; external-edit detection limits; result durability through crash/ACK loss and later writes; no install/provision or active-World mutation caused by the declaration operation.

### 7.1 Planning target: exact mutation and writer contract

`ReadRequirements(workspace_ref, cursor_or_initial)` returns rows with canonical inventory item reference, definition/provenance, Workspace-local declaration, inherited/effective enablement, editability and reason, plus one opaque read revision. Optional applied observations are separately World-qualified/freshness-labeled. Pages belong to that revision; if it cannot be maintained, return stale cursor and require a new read, not a mixed-revision list.

`SetRequirement(workspace_ref, item_ref, locally_enabled, expected_revision, mutation_key)` changes exactly one known editable Workspace-local declaration. Bind equality to principal/Workspace/operation, exact item, boolean and expected revision. Lookup an existing key under current read authorization **before** treating its old expected revision as stale: an ACK-lost success returns its original result even after later edits. A changed expected revision is a new intent with a new key, not a way to overwrite the prior result.

| Outcome | Meaning |
|---|---|
| Committed | Declaration and recoverable mutation result durably committed; return that operation's revision and authoritative row/readback. |
| NoChange | Expected revision validated; requested local state already held. Persist the operation result, but do not pretend an environment change or require a new declaration revision. |
| Conflict | Expected dependency/configuration cut is stale; no mutation. Return an authorized refresh reference, not an automatic retry against latest. |
| Refused | Invalid/unknown/uneditable item, denied action or unsupported scope/version; no declaration mutation. |
| KeyConflict | Original key is bound to different inputs; original operation unchanged. |
| PendingOrUnknown | Commit/result could have occurred; reconcile the original key. Never infer success or non-execution from the current boolean. |
| NotRecorded / ExpiredHistory / Unavailable | Distinct authorized lookup/coverage results; none means “safe to replay the write.” |

**Writer boundary:** include the Workspace patch and every effective inventory/parent input used to decide item validity/editability in the revision contract. All supported changes to those preconditions must be serialized or guarded by validated version/fence checks through commit. The intended supported writer set includes Workspace deps add/remove and any config set/reset/list replacement reaching `world.deps.enabled`; do not silently exclude existing supported CLI writers to make the new endpoint's CAS appear correct. Read-modify-write of the shared YAML must also preserve unrelated configuration edits. Direct arbitrary host edits are not transaction participants; detect changed preimages where possible and refuse/reconcile, without claiming atomic protection against them. Complete the writer inventory before implementation; this target does not assert that inventory is already exhaustive.

**Inspected source:** [deps surfaces](../../../crates/shell/src/builtins/world_deps/surfaces.rs) `run_workspace_add/remove` modify the normalized patch and call `write_atomic_patch_yaml`; the helper uses a temp-file persist, with no consumer expected-revision/mutation-result protocol in the inspected path. [Config commands](../../../crates/shell/src/execution/config_cmd.rs) `run_workspace_set/reset` are additional writers; [config model](../../../crates/shell/src/execution/config_model.rs) supports set/append/remove/reset of `world.deps.enabled`. The deps JSON branch also emits a human note; do not parse it as the new single machine-response contract. Reuse owner semantics, adding a deliberately versioned exposure/transaction boundary rather than changing legacy output by implication. All three files byte-matched clean Linux `088f62d…` during this pass.

**Planning disposition:** Collider can design revision-bearing rows, one-item edits, conflict refresh and unknown-result reconciliation now. Coordinated writer transactions and durable result retention are required implementation work, not currently supplied by atomic rename. Installed read/edit/readback proof remains outstanding.

## 8. Consumer mapping requirements

Same-key reconciliation preserves the request, not its first response: preparing/uncertain/material-incomplete progress may advance, but accepted identity and original inputs never change. The [Collider lifecycle mapping](../../../../collider/docs/agent-native/controller-ownership-plan/controller-owner-lifecycle-contract.md#cancellation-races-and-observation) carries the proposed lossless B4-to-client table. A local cancellation `failed` may mean unknown owner outcome; it is not evidence of refusal, non-delivery, or safe retry with a new key. Client generation is only a local race guard.

| Owner fact | Collider interpretation / prohibited shortcut |
|---|---|
| Preparing/submission unresolved; no known acceptance | Request UI only; not running, failed work, or a fabricated queue state. |
| `Pending(AcceptedMaterialIncomplete)` | Show acceptance reported / receipt completion pending; retain the original request key. No fabricated actionable receipt or accepted/running protocol transition before the complete join. Reconciliation may advance straight to terminal. |
| Exact accepted work | Accepted-first projection; running only when owner evidence supports it. |
| Receipt + historical Lane/World binding | Same work can appear in Control Plane and Lane projections; opening two views does not create two behavioral owners. |
| Cancel-request acknowledgement | Pending cancellation; continue observing until exact terminal evidence. Client `cancelRequest.generation` is not owner cancellation identity. |
| Pending-admission control target/outcome | Explicit unsupported-path result in the bounded accepted-work surface. Any later supported extension preserves `CancelledBeforeTransport`, `AlreadyRoutable`, and B4 admission progress separately from task terminality. |
| Owner terminal / unusual result class | Preserve the exact owner outcome separately from protocol settlement. `NeedsRetainedFollowup`/`Invalidated` still need a lossless segment-ending binding; unsupported is valid only if the selected profile truly cannot return them. No success/cancel fabrication or dropping an observed terminal. Terminal time remains unavailable when absent. |
| Owner cursor and read cut | Adapter checkpoint/projection sequence is separate; filtering cannot introduce unexplained sequence holes. |
| Declared dependency change | Configuration result, not installed/applied confirmation. Applied status must name the observed World. |

Collider's checked-in correction is a patched local client candidate, not a demand that Substrate adopt AgentKit protocol names. The existing consumer lifecycle mapping must be reviewed against the owner contracts once approved; it was not rewritten or approved here. C-OQ-027, native/deployment proof, licensing/distribution, and adoption are not closed by a contract draft.

## 9. Reconciliation and readiness register

This is a proposal-review checklist, not a new canonical status ledger. Link to actual owners for status changes.

**E3-F chronology clarification (user, 2026-09-21):** E3-F was started, then paused when gaps/prerequisites needing correction were discovered. The subsequent need for Substrate input to unblock Collider led to the present contract-design work. This agrees with the historical dossier's dispatched-but-paused, unaccepted/unclosed account. The older authority-map and E3 slice status text still says not admitted/dispatched; record that as unsynchronized historical/status evidence, not a reason to erase the later work. This clarification neither certifies that the prerequisite gaps are now fixed nor resumes/re-admits E3-F. Any canonical status correction and implementation continuation require their own scoped reconciliation; the historical dossier and R1–R34 remain unchanged.

| Item | Freshly read document evidence | Remaining boundary before implementation-ready claim |
|---|---|---|
| B01 | B2.2 owner records landed/review-clean receipt return and E2-RM use, with separate closure unlocated. The source pass confirms durable B1 proposal/ACK joins and early immutable retry, not a complete consumer authorization boundary. | Satisfy the runtime-profile readiness gate; review §3.1–3.4's result mapping and joint protocol; close concrete ingress/delegation and stale-sender/revocation fencing, protected correlation storage, recovery, retention/expiry, exact wire types and call chain. |
| B02 | Existing Supervisor and cancellation contracts define exact ownership and outcomes. B4 owner records landed source but unverified exact terminal proof/closure. | Public authenticated read/observe/cancel, cut/cursor/retention/restart contract, result mapping and exact-path proof. |
| B03 | Source enforces session/caller/World equality; shared physical ownership is session-keyed. These are existing bounded authorities, not the newly confirmed multi-World Lane model. | Review companion §4.6–4.9/§7 recommendation: Workspace association authority, proposed owner interfaces and dispatch correlation/recovery, backend custody reconciliation, exact wire/storage schemas and crash protocol. No plural HSA binding or implicit session rebind is selected. |
| B04 | Collider context requirements plus Handbook's HCM-4.1 consumer boundary, SDK packet operations and engine Context Resolution/grounding types were inspected. No arbitrary selected-reference-to-Substrate-delivery route was established. | Agree the concrete Handbook operation/reference-kind mapping for §6.1; implement authorized resolution/assembly/delivery with lineage and exact-path proof. |
| B05 | World deps docs and the inspected add/remove/config writers distinguish declaration editing from application; the inspected atomic replacement does not supply the target CAS/retry-result contract. | Complete the writer inventory and revision dependency set; implement §7.1's coordinated transaction/result persistence and versioned machine exposure; prove real read/edit/readback. |

**Cross-cutting successor obligations:** (1) session-bound World authority versus Workspace multi-World Lanes; (2) reconcile existing filesystem-sync/history and guest-image plans with stable project state; (3) immutable worker cap versus authorized same-worker expansion; (4) ownership-only auto-attach versus separately authorized bounded reasoning; (5) global/Workspace dependency configuration versus Lane baselines and World overrides. These are recorded design differences, not permission to remove current safeguards. The [auto-attach invariant](../../../llm-last-mile/runtime-refactor/architecture/invariants/07-auto-attach-restores-ownership-only.md) stays unchanged; a future trigger owns its own action authorization.

**Required research dependency:** [World state and environment research/experiments](../../ideas/world-state-environment-research.md) must compare alternatives and produce evidence before selecting the new filesystem/environment mechanisms or asserting their guarantees. It references ADR-0009/0010 and recoverable historical plans, including the older image/Workspace overlay keying. Guest OS isolation and project baseline isolation are separate questions. The older guest-rootfs initiative is not automatically adopted, made V1, or deferred with alternative-Lane backing. Research execution and experiments remain unstarted.

Continue semantic decisions for B01–B05; keep mechanism-dependent B03 state operations and B05 environment realization explicit dependencies. The narrow B05 declaration mutation is not per-World realization proof. A protocol-level test or unchanged declaration revision cannot demonstrate actual dependency isolation or stable project files.

Some obligations are outside the selected proof but not outside the product. B03 needs real binding; B05 can prove Workspace mutation separately. Stable filesystem behavior and V1 overrides must be satisfied before claiming complete V1, even if a smaller experiment does not exercise them. Worker fork, lane-orchestrator delivery, active-work transition, broad scheduling, and same-World editing mechanisms remain separately scoped. No optional non-E3 lease or E3 exclusion removal is selected here.

### 9.1 Planning handoff disposition

| Contract | Defined planning answer | Existing foundation / required change | Proof status |
|---|---|---|---|
| B01 | §§2.1/3: caller-scoped request, explicit destination, pending/refused/accepted separation, immutable retries, read-only reconciliation and current access. | B1/E2/B2.2 exist internally; consumer ingress/delegation, key correlation, guarded submission and complete exposure require work. | No certified consumer runtime profile or installed handoff proof. |
| B02 | §4: exact-work cuts, cursor/replay/gap/expiry semantics, observer lifetime, caller-keyed cancellation and monotonic owner terminality. | Supervisor/runtime/B4 primitives exist; authenticated consumer subscription/read/cancel and advertised limits require integration/extensions. | No GUI reconnect/restart/cancel-race proof claimed. |
| B03 | Companion: distinct Workspace/Lane/World/actor/custody, revision-guarded existing destination, durable joins and historical attribution. | Existing exact-session authority is narrower; Workspace association responsibility and general selected-World authority need explicit owner changes. | No genuine consumer Lane-binding implementation or multi-World runtime proof claimed. |
| B04 | §6.1: owner-qualified exact selections, required/optional refusals, no silent continuation, resolved versus injected lineage. | Real Handbook SDK/kernel/grounding foundations; exact selected-reference consumer mapping and Substrate delivery integration are named external-owner dependencies. | No end-to-end typed-context delivery proof claimed. |
| B05 | §7.1: local declaration versus effective/applied state, revision and mutation-key equality, no-op/conflict/uncertainty, coordinated writers. | Existing deps/config writers; expected-revision serialization and durable operation results need owner changes. | No all-writer concurrency/crash proof or installed mutation proof claimed. |

**Three distinct checkpoints:** target-mapping readiness specifies consumer meaning with named open bindings; exact binding readiness selects supported operations/types, runtime profile and cross-owner joins; runtime proof exercises those implementations on the selected installed path. B04 has a planning target but its positive binding is still open. None of these labels substitutes for owner acceptance.

**Handoff review exit:** Collider can identify every input it must retain, every outcome it must display/reconcile, the authoritative owner, unsupported paths, and the implementation/proof dependencies without inventing backend semantics. Those target answers are now written; consumer/owner acceptance is pending. Exact transport bindings, advertised numerical limits, the B04 owner mapping and runtime profile must be settled before live integration, but their absence is explicit and does not turn planning into a claim of runtime readiness. No milestone here certifies the complete Substrate architecture or replaces its R1–R34 inventory.

## 10. Review, validation, and next owner work

The next **planning** review is this bounded B01–B05 target handoff against Collider's consumer mapping, including the explicit Handbook integration dependency. Do not delay that review for a documentation reset, E3-F completion or filesystem mechanism selection. After target acceptance, detailed **implementation** closure proceeds through B01/B03 together, then B02; B04/B05 owner integration can be planned independently using the same identity conventions. This is not an implementation or parallel-agent dispatch.

Before implementing each mapping, owner review must choose exact schemas and supported entrypoints, resolve its named engineering obligations, identify production callers/enforcement, and attach its negative/race/recovery test plan. These are stronger gates than the planning handoff. Runtime acceptance additionally requires the actual installed Linux path; fixture tests or documentation approval are insufficient.

Document-review checks for these two drafts:

```bash
git diff --check
git diff --no-index --check /dev/null docs/contracts/draft/lane-world-binding.md
git diff --no-index --check /dev/null docs/contracts/draft/collider-b01-b05-handoff.md
shasum -a 256 SUBSTRATE_CORE_ARCHITECTURE_SESSION_DECISIONS_2026-09-20.md RUNTIME_REFACTOR_SESSION_DECISIONS_2026-09-18.md
git status --short
```

For new files, no-index checks can report differences; inspect whitespace diagnostics separately from a normal difference exit status. Also validate Markdown link targets/fragments, source pins, the active authoring fence, and absence of accidental status promotion. No Rust source changed, so no Cargo test result or runtime proof is claimed. Implementation test commands must be bound to the later admitted source slice rather than guessed in this draft.

**Initial authoring validation (before research follow-up):** root-agent consistency review and mechanical checks covered both drafts, 27 Markdown links/fragments, unchanged dossier hashes, and the two-new-document fence. Those results apply to the earlier draft bytes.

**Research follow-up validation:** root-agent delta review and mechanical checks passed for 53 local Markdown link occurrences (all draft/brief links and newly added dossier/backlog targets), seven historical Git paths, whitespace, the updated dossier source pin, and the five-document scope. HEAD and index remained unchanged; the historical runtime dossier remained byte-identical. No independent reviewer, owner approval, commit, integration, or runtime test is claimed.

**B03/B01 semantic follow-up evidence (2026-09-20):** reread the focused authority map, HSA binding invariant, receipt/supervision seam, ephemeral receipt, B2.2 owner, and relevant E2 commitment/E2-RM sections. All six files were byte-compared with the freshly rebound clean Linux checkout at the same `088f62d81c182b92cac404983b8bf512becc829c` revision. This pass edits only these two proposal drafts; the dossier, research brief/backlog, historical dossier, canonical owners, source, and Collider files are unchanged. Proposed revision scopes, receipt composition and result semantics are not owner approval or runtime proof. Earlier validation counts describe earlier bytes.

Root review and mechanical checks for this semantic follow-up passed: 32 draft link targets/fragments, whitespace, dossier source pin, unchanged HEAD/index and protected documents, and the two-file edit fence. Root review also distinguishes stale-destination rejection for a new request from immutable original-binding return for an accepted retry. These are documentation checks, not execution tests or independent approval.

**Drift rule:** before owner acceptance or implementation admission, recheck the dossier hashes, relevant Substrate owner documents and shared dependencies, and consumer mapping at the recorded bases. A moving branch tip alone is not grounds for a complete reread; a changed relevant authority or dependency is. Never treat a file hash alone as proof that its transitive semantics stayed unchanged.

**2026-09-21 review correction:** the independent source/document review requested changes for the runtime-profile gate and pending-admission outcome coverage, and qualified the HSA redesign inference. These draft corrections address that scope without approving a runtime profile or adding implementation authority. Existing bytes were first checkpointed at local `f461ac3f0` on `docs/substrate-collider-contracts`; the earlier authoring/source bindings above remain historical evidence, not that branch's current HEAD. This correction does not update the dossiers, research scope, canonical owners, or runtime source.

**2026-09-21 authority-design pass:** the reviewed corrections were committed separately at `9161d290d` before this proposal iteration. Authoritative Linux remained clean at `088f62d81c182b92cac404983b8bf512becc829c`; eleven inspected source files matched local bytes. B03 §7 records the affirmative source constraints, recommended owner changes, alternatives and validation limits. Only the two drafts change in this iteration; no canonical owner amendment, runtime implementation, research experiment or production proof is implied.

**2026-09-21 joint owner-interface pass:** starts from clean local `f43173ff31ef2c037a7183c577d05d22b4eba94a`; the user clarified E3-F history and authorized continuing design. Linux was freshly rebound clean at `088f62d81c182b92cac404983b8bf512becc829c`, tree `b40da914c1e5d73fc79f13c0946f6af9c97455d1`. Direct reads included `orchestrator_world_dispatch.rs` (`prepare_task_acceptance_submission`, `enforce_world_dispatch_steering_policy`, `launch_task_supervision_and_handoff_receipt`, `observe_run_world_task_stream`) and `state_store.rs` proposal/acceptance schemas; both files plus `dispatch_contract.rs` and the authority map byte-matched Linux. The edit fence is the two drafts and the core dossier, whose pin above is updated. The previous Pro review covers the submitted `f43173ff` bytes, not these new interface/recovery candidates. No independent review of this delta, runtime test, E3-F continuation, canonical amendment, or implementation is claimed.

**2026-09-21 bounded planning completion pass:** preserves the prior uncommitted design edits on the same HEAD and three-file fence. Collider remains at `75631d432099fa18f1a320fc93863207ac7bcd2b` with only its pre-existing untracked bundle ZIP; its boundary plan and foundations §11 were refreshed read-only. Handbook was inspected read-only at clean `8605d6539355f350c55b921d95fb450bca30ee18`; §6.1 records the relevant owner/SDK/source paths, not new publication or downstream proof. Linux remains clean at `088f62d…`; deps surfaces, config commands/model, cancel-outcome contract and deps reference byte-matched local copies. GitNexus provided Handbook symbol leads; Substrate query reported degraded keyword indexing, so direct source reads—not graph absence—ground the writer findings. Added planning targets and the stronger implementation gates remain distinct. No code, sibling repository, canonical runtime owner, historical dossier, research scope or index is changed; no tests, implementation, commit or push are claimed.

**2026-10-21 admission-contract design:** joint B01/B03 selects proposed authenticated ingress, original semantic/material joins, per-owner persistence/recovery and service close-before-effective-commit/consume/fence semantics. Minimal proposed responsibility notes are added to the existing authority map, receipt/supervision seam and B1/B2.1 contracts; five existing Markdown files change. Source correspondence, exact scope/byte checks, links/anchors, tables and auth/acceptance/retry/association matrices are documentation obligations. No runtime code, tests, builds, Linux host writes, component-owner signoff, installed proof, commit or publication is claimed; independent review/owner acceptance remain separate. Prior dated validation entries below/above concern their original bytes.

**2026-09-22 bounded consumer reconciliation:** clarified complete B04 request equality, known acceptance with incomplete receipt composition, cancellation uncertainty, mixed local/inherited editability and the still-open nonstandard terminal binding. Added the explicit selected-case B04 mapping dependency and corrected the core dossier resume sequence/pin. The existing Collider boundary/lifecycle plans now map these proposed targets and reference Packet A's recorded client correction; they do not reverify its patch or claim callable owner paths. No historical runtime dossier, canonical owner, runtime source or Handbook change; no new tests/experiments, implementation, commit or push. Consumer/owner acceptance and exact binding/proof remain separate.
