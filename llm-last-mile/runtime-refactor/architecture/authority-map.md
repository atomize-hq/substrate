**Kind:** architecture
**Stable ID:** `shared-authority-map`
**Canonical for:** shared authority map plus the current E2 policy-commitment and E3 config-projection ownership corrections and proposed B01/B03 responsibility amendment
**Status:** canonical shared-architecture record
**Authority scope:** exact extracted shared authority-map source body plus the documentation-only E2 and E3 owner corrections and proposed B01/B03 responsibility amendment; no implementation authority
**Source span:** [`../01-target-architecture.md#authority-map`](../01-target-architecture.md#authority-map) lines 62–83
**Supersedes:** canonical ownership of the extracted source body; source heading remains a compatibility anchor
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md), [`../01-target-architecture.md`](../01-target-architecture.md)

# Authority map

## Proposed B01/B03 consumer admission responsibility amendment — 2026-10-21

The authorized joint [B01 contract design](../../../docs/contracts/draft/collider-b01-b05-handoff.md#32-immutable-request-identity-and-the-admission-boundary) and [B03 effect/recovery protocol](../../../docs/contracts/draft/lane-world-binding.md#47-associationadmission-consistency-and-recovery) select the following **target extensions**, not current exports, runtime implementation authority, component-owner signoff or seam promotion. The historical owner rows and their evidence states below remain unchanged.

| Boundary | Proposed additional responsibility | Preserved boundary |
| --- | --- | --- |
| WorldDispatchControl | Authenticated consumer-key correlation, immutable initiating/executing identity and original B04 seal joins, possible-send progress and explicitly activated recovery composition; separate non-mutating request read. | No second acceptance registry, scheduler, policy invention, context re-resolution on retry, task terminality or physical custody authority. |
| Workspace authority alongside HSA, with existing policy/authorization owners | Native Substrate Workspace discovery/load and explicitly approved absent-only initialization; current config/policy/identity/Lane/destination reads and separate operation/resource/read checks. Later full-send delegation and selected-World association reservation/revision/history remain distinct; external-client enrollment is outside the initial native opening path. | HSA alone resolves canonical session/caller/root facts; Lane identity is not session identity; association is not execution permission or backend ownership. |
| Runtime/backend authority | Authenticated service epoch/attempt consume, close/fence tombstones and original producer-evidence read; exact existing-custody validation and cleanup/state evidence. Every applicable invalidator closes the old gate before authoritative effect-facing change commits. | Consume is neither B0 ACK/B1 acceptance nor delivery. No stale permit, global lock across I/O, replacement task identity or relaxed physical ownership. Emergency invalidation/cancellation and later delivery/current-use guards remain separately owned. |
| B1 / B2.1 / E2 / B2.2 / S1 | B1 exposes original proposal reverse join and positive no-send close; existing acceptance/claim/immutable policy/receipt owners supply exact historical composition; S1 retains original material and independently proves authorized actual delivery. | No truth transfer among registry, Supervisor, policy or association owners; no fixture/receipt-to-injection inference. H1–H3 and conditional H4 remain Handbook responsibilities. |

Proposed module/symbol placement, lock/transaction boundaries and all failure transitions are explicit in B03 §§4.7–4.9; B01 §3.4 binds later profile/security proof and the narrow source catalog. Unsupported current ingress/profile or a non-participating policy/delegation/source/disclosure/current-use/destination/custody invalidator keeps execution unsupported/design-only. Close-before-effective-commit preserves immediate-at-authoritative-commit revocation semantics; delaying a response cannot postpone an already-effective revocation. This amendment changes no E3 ownership/status, retained-runtime fence, I1/I2 disposition or Linux candidate evidence.

[`B01-B03-P1`](../slices/tasks/b01-b03-authenticated-preflight.md) owns the initial same-host Substrate Workspace open/load/approved-initialize boundary and separate exact-destination **no-send preflight**. Its bounded admission contract names proposed native adapters, nonreconciling readers and proof; source dispatch is not authorized and runtime remains unproved. The human correction supersedes the separate setup-issuer prerequisite; configured opening needs no enrollment/pairing/new approval. Initialization writes are separate from zero-effect open/preflight. Eligibility grants no future send or B1 acceptance; later full-send request/hold/fence/retention responsibilities remain unchanged.

## Current E3 config-projection ownership correction

| Boundary | Owns | Must not own |
|---|---|---|
| AgentConfigProjectionService | accepted-home `AgentConfigProjectionRecordV1` series/record/ref authority; immutable subject binding to workspace/session/participant/bootstrap/backend/runtime artifacts/world generation/E2 cap; monotonic projection revisions; deterministic non-secret Codex 0.125 rendering and ambient-layer closure; strict secret-bearing preparation request with nonsecret response and strict secret-free `MemberDispatchRequestV2` carrier; gateway activation intent, zero-live fence, and exact pre-release validation | caller-selected roots; host config/auth files as authority; persisted or execute-carried secrets; current-parent or E2 cap reconstruction; E2/E2-RM mutation; D1 envelope or V3; receipts/manifests; broker policy; workspace sync/reconciliation; compatibility promotion |
| ManagedGatewayAdoption | preallocated gateway identity; deny-all dormant access boundary; exact one-time handoff join; non-secret readiness ACK/ref; exact-member activation and revocation; descriptor-pinned gateway identity; original exclusion/cgroup/namespace custody and sealed Codex per-turn accounting history feeding exact whole-attempt terminal cleanup | credential ownership or persistence; request-header authorization; copied-secret fallback; provider policy; Codex/UAA FD inheritance; D1/D3 evidence ownership |

The controlling schemas and nonownership wall are
[`agent-config-projection-v1.md`](../contracts/agent-config-projection-v1.md) and
[`managed-gateway-adoption-v1.md`](../contracts/managed-gateway-adoption-v1.md). This correction leaves
the extracted `AgentConfigProjectionService` and `WorldRuntimeAdapterExecutionEnvelope` rows below as
historical architecture projection. D1 still owns the envelope and later strict V3. E3-A through
E3-D are landed; the [E3-E closure](../slices/e3-agent-config-projection-and-gateway-adoption.md#e3-e-terminal-closure-2026-09-17) records terminal completion/product landing on a separate
lineage from authority closure. Enclosing E3 remains incomplete; the [dated E3-F correction](../slices/e3-agent-config-projection-and-gateway-adoption.md#current-e3-f-status-correction--2026-10-21) records dispatched-then-paused work, with W8 unaccepted/unclosed and `c381` published-unintegrated; no continuation is authorized.
The [I1/I2 contract binding](../contracts/agent-config-projection-v1.md#e3-f-i1i2-ownership-and-accounting-reconciliation)
now replaces the mandatory common non-E3 lease/additive E3 field with required tagged ownership.
The retained E3 owner arbitrates child/adapter custody and turn lifecycle; gateway owns original
kernel/exclusion custody and accepts sealed terminal-child contributions; projection/registry owns
exact durable whole-attempt terminal and original lease readback. None acquires B0/B1/B2.1 semantic
completion authority. These are design bindings, not implemented/proven source. The
[canonical return sequence](../slices/e3-agent-config-projection-and-gateway-adoption.md#i1i2-design-binding-and-runtime-return-sequence--2026-10-21)
places separately authorized P1 after the bounded E3-F proof return as scheduling priority, with no
universal E3-F dependency for independent Workspace reads or blanket successor admission.

## Current E2 policy-commitment ownership correction

| Boundary | Owns | Must not own |
|---|---|---|
| DispatchPolicyCommitmentRegistry | immutable `DispatchPolicyCommitmentV1` construction from owner-supplied exact current-parent/immutable-cap/patch inputs; exact E1 `serde_json::to_vec(PolicySnapshotV3)` bytes/ref/hash validation; exact source-owned B2.1 `WorldWorkExecutionClaimV1` linkage; immutable pre-B3.2a Fresh-Spawn reservation/ref with a keyed complete-request commitment, authenticated preallocated-identity proof, a distinct opaque narrowing attestation available only for authenticated nonempty E1 narrowing, the exact B3.2a `allow_capability_narrowing` validation exception for that attestation, stable source-owned admission linkage, and post-admission CAS publication retaining the reservation and request commitment in the committed index; exact retry/conflict handling; E2 fork-dispatch linkage | retained-target identity/routing or current-parent resolution; B1 acceptance; B2.1 observation, cursor, interruption, or resumability; B3.2a fingerprint, plan/registry/admission schema, other validation, cap accounting, admission/routability/lifecycle semantics, or identity generation outside the bounded E2 input seam; prompt/payload preimage or raw secret-derived request hash persistence; fork admission/lifecycle; receipt construction/return; retained lifecycle or full-manifest construction; D1 execution-envelope identity; E3 config-projection identity; policy enforcement; migration; or cap reconstruction |

## Authority map

| Boundary | Owns | Must not own |
|---|---|---|
| SurfaceAdapter / HostExecutionEpisode | input normalization, live channels, rendering, episode-local cancellation, readiness observations; the current bounded production activation call into canonical supervisor recovery | durable posture, world binding, retained continuity, successor allocation, restart discovery/reconciliation, supervisor-record interpretation, terminal truth, transition-intent issuance/claim/application |
| HostSessionAuthority | exact session/caller/lineage/session-world-binding resolution; durable posture transitions; revision-bound host-transition-intent issuance, claim validation, replay-safe application, and reconciliation | physical world realization or ownership metadata, runtime-process placement, transport loops, provider mechanics, compatibility projection |
| StateStore | bounded atomic physical persistence, migrations, schema evolution | lifecycle, receipt, supervisor, routing, or liveness-derived semantic authority; generic activated-store writes |
| CompatibilityReadModel | legacy reads, torn-root diagnostics, compatibility projection/migration | new authority writes or overriding newer revisions |
| WorldDispatchControl | typed world verbs and orchestration of authority/policy/receipt/runtime boundaries; blocking compatibility inspect/wait/cancel routing that consumes exact receipt/supervisor truth | physical world ownership adoption, provider-specific execution, direct policy invention, or ownership of accepted-work/observation truth |
| SteeringPolicyEngine | deny-by-default action/mode/backend/session/world/autonomy decisions | effective policy materialization or runtime launch |
| EffectivePolicyResolver | parent-policy composition and immutable `PolicySnapshotV3` materialization | enforcement by advisory flags alone |
| WorldWorkReceiptRegistry | proposed acceptance-record identity/request context before submission; durable immutable accepted task/turn identity only after runtime acknowledgement; immutable recording of any owner-supplied host-transition correlation | runtime acceptance itself, active observation claims, frame/event journals, terminal reconciliation, stream ownership, host-transition interpretation, or worker lifecycle policy |
| RuntimeEventTransport | producer-assigned stable stream/frame/event/terminal identity and monotonic ordering; bounded process-memory retention and exact replay transport for supplied acceptance-record/stream/cursor identity | receipt acceptance, durable observation, stream enumeration, fuzzy lookup, lifecycle or terminal truth, retained-message semantics, obligation semantics, or completeness |
| WorldWorkExecutionSupervisor | sole ownership of durable active-observation claims, no-gap post-acceptance frame/event journal, exact acceptance joins, opaque correlation-byte retention, duplicate/reorder rejection, caller-drop survival, restart discovery/recovery/reconciliation, unresolved replay state, and monotonic terminal closeout | proposal or immutable acceptance-record truth, foreground tool or startup-surface semantics, model-facing identity, producer-frame retention, retained-message or host-transition semantics, or obligation classification/materialization |
| WorldWorkerMessagingProtocol | fail-closed producer-side normalization of provider events plus exact retained target/source, active-run, thread, typed event class, attention, and request/message/event causation semantics | transport ordering, receipt acceptance, observation ownership, or obligation materialization |
| RetainedWorkerRuntime | worker create/continue/park/cancel/stop/fork/inspect/invalidate lifecycle; retained admission, R0-registration join, transport-claim, routability, and exact terminal truth | HSA world binding, physical world ownership metadata, host-session posture, or obligation projection |
| ObligationLedger | obligation classification, idempotent materialization, canonical revisions and records, completeness watermarks/cuts, closed snapshots, and attention/review/deferred-action truth | runtime identity generation, stream observation, host rendering, prompt replay, or direct worker continuation |
| Inbox / AutoAttach / Router | derived review view, attach eligibility, sanctioned host ownership restoration | approving, answering, forking, or continuing workers |
| AgentConfigProjectionService | logical inventory and non-secret effective/native projection per worker identity; launch-time secret-handoff intent | treating `.codex`, `CODEX_HOME`, `config.toml`, auth files, or workspace files as credential authority |
| WorldRuntimeAdapterExecutionEnvelope | guest-realizable launch contract bound to world, worker, config, policy snapshot, credential posture, secret-handoff ref, and equality-only exact world-ownership prerequisite | raw credential payloads, HSA or backend ownership authority, provider-specific parsing, or unrestricted side effects |
| WorldCommandExecutionBroker | every UAA shell/edit/write/tool/process/network side effect under the accepted policy snapshot | bypassing world-service because the initial process is in-world |
| RuntimeFamilyRealizationAdapter | provider launch, resume, output parsing, non-secret native config format, gateway endpoint wiring, provider cancellation mechanics; runtime-family/world-backend physical world realization and exact ownership-metadata publication under validated HSA proof | raw host credentials; minting or changing HSA binding, admission, policy, receipt, obligation, routability, or terminal semantics |
