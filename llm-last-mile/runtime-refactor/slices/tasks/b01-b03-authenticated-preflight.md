**Kind:** packet
**Stable ID:** `B01-B03-P1`
**Status:** TASK04 committed ad98ea44b; TASK05 committed 3eac45580; TASK07 committed 29cf8e822; TASK10/13 readers accepted in isolation, not adopted; full P1 incomplete; current documentation-only reconciliation, implementation held
**Canonical for:** initial same-host Substrate Workspace open/load/initialize boundary and exact-destination no-send preflight contract, proposed source fence, proof gates and admission assessment only
**Authority scope:** original P1 contract plus the selected P1-A component below; no real setup operation, full P1 qualification, B1 acceptance or later-send permission
**Projection consumers:** [`current`](../../index/current.md), [`by-packet`](../../index/by-packet.md), [`by-id`](../../index/by-id.md), [`by-kind`](../../index/by-kind.md)

# B01-B03-P1 — authenticated exact-destination no-send preflight

<!-- linux-preview-overlay:A05:start -->
## Linux preview applicability (2026-11-05)

[accepted M1 source record](../../milestones/milestone-1-source-completion-2026-11-05.md)/[M1 source/proof manifest](../../milestones/milestone-1-source-manifest-2026-11-05.json) now accept integrated native Workspace open/approved absent-only init, genuine reader adoption, configured V3 selection/defaults and exact no-send source proof. Old proposal-pending/not-adopted/implementation-HOLD wording below is historical for this narrower preview; [preview decision](../../linux-first-codex-release/DECISION.md) and [successor roadmap](../../linux-first-codex-release/ROADMAP.md) control current priority without changing the P1 contract.

The five `workspace-source-proof-current-11c1` populations are `preflight`, `preflight-resume`, `preflight-owner`, `preflight-custody`, `preflight-legacy`, all exit 0 with nonzero PF-OPEN/INIT/AUTH/EXACT/REV/ZERO/LATER observations in their applicable populations. Recorded source is `11c1c7eb388fcfdc2ad21264defac83fd221dde9ff7da8fe1e77b0ca9f96ef14`, binding `3ac6d40b02c92a3b6a5a94a3d5163015a9b0e1346c2442cad4514b7b027c0674`; results SHA-256 `7dea4aa324c509818a5188d9cb48898678581b74b5c12b459411bfd6257e975a`, with unchanged production PF routes independently confirmed to M1 tested map `46055570…`. This is private-source genuine native publishers/readers and typed Root UDS with a private cgroup metadata mask: **no installed/kernel/provider qualification**. Source correspondence is not a fresh run. Approved init writes remain separate from zero-effect open/preflight/denied reads. Full-send, B1 acceptance, association/reservation/history mutation and Collider/frontend adoption remain outside this proof; installed full-PF remains L3.

<!-- linux-preview-overlay:A05:end -->

## Configuration proposal and Collider alignment — 2026-10-28

The user requests authoritative control-plane/Collider alignment. This is documentation only
while implementation is held. The [current E3 direction and hold](../e3-agent-config-projection-and-gateway-adoption.md#configuration-direction-and-planning-hold--2026-10-28),
[TASK20 v2 roadmap](../../../../.codex/orchestration/e3f-retained-next-20261022/evidence/task20/v2/serial-completion-roadmap.md) and
[configuration audit](../../../../.codex/orchestration/e3f-retained-next-20261022/evidence/task20/v2/configuration-source-audit.md) retain human
configuration intent. C1–C5, schema/settings, whole-agent replacement, route and catalog deferral
remain pending review. These links authorize no P1 contract or source-fence expansion.

At the existing same-host native boundary, Collider invokes Substrate; selected paths and UI
fields are not authority. Valid configured state loads with current authorization, without
enrollment, pairing or new setup permission. Only genuine absence offers explicit approved
initialization; invalid, disabled or incomplete state is diagnosed. Substrate owns identities,
configuration, policy, Lanes and destinations; presentation cannot invent or write them.

Exact no-send preflight requires the authentic existing HSA caller/session/backend/custody and
exact Workspace/Lane/World-generation/association. It does not rebind sessions, reserve, accept
or send. Initiating principal, execution actor and backend owner remain distinct. Existing PF
gates are unchanged. A new model selection neither selects a World/Lane nor grants execution
or read rights.

If proposed selected-agent, inventory, route or profile revisions contribute to eligibility,
the later approved design must identify their real owners and complete monotonic invalidator
coverage in the read cut, including same-bytes A→B→A. No implementation coverage is claimed.
Running sessions remain pinned to the original selection; read/load must not silently reselect
or repair.

The [handoff same-host opening](../../../../docs/contracts/draft/collider-b01-b05-handoff.md#1-scope-and-source-binding) and
[§3.2 native binding and separate later ingress](../../../../docs/contracts/draft/collider-b01-b05-handoff.md#32-immutable-request-identity-and-the-admission-boundary),
B03 [§4.6 actor/selection boundary](../../../../docs/contracts/draft/lane-world-binding.md#46-recommended-model-authorized-selection-not-session-rebinding),
[§4.7 consistency/recovery](../../../../docs/contracts/draft/lane-world-binding.md#47-associationadmission-consistency-and-recovery),
[§4.8 owner-interface candidate](../../../../docs/contracts/draft/lane-world-binding.md#48-joint-b01b03-owner-interface-candidate--2026-09-21) and
[§4.9 durable request/history joins](../../../../docs/contracts/draft/lane-world-binding.md#49-durable-request-join-and-recovery-candidate)
are review candidates, not exported APIs or approved extensions. P1's
[current-read stable cut](#current-read-schema-and-stable-cut) and
[PF proof gates](#dependencies-and-required-proof) remain controlling. B01 send, reconciliation,
reservation and fencing; B04 context/Handbook; B05 dependency mutation; frontend adapter adoption;
and installed runtime readiness remain separately scoped and proved. Completing all B01–B05 is
not required to finish no-send P1. The handoff's Collider repository pin is historical; the
current frontend is unverified.

## Admission disposition

### Selected first increment — 2026-10-26

Following the [honestly stopped E3 return](../e3-agent-config-projection-and-gateway-adoption.md#bounded-return-and-next-task-sequence--2026-10-26),
the user authorized bounded TASK dispatch. META selects `B01-B03-P1-A`: implement the strict
read-only selected-location inspection component of the already proposed
`WorkspaceOpenAdapter::inspect_existing`. This section supersedes earlier no-source-dispatch
wording only for this component; the complete P1 contract and remaining proof stay binding.

**Outcome and claim ceiling.** A native internal caller can classify a selected location and its
nearest applicable Workspace boundary without initialization, repair, owner allocation or
execution. A valid marker is only a discovered candidate, never authenticated Workspace/Lane
identity, a `Loaded` owner view, exact-destination eligibility or permission to send. Component A
must not manufacture absent owner state to implement the later load/preflight operations.

**Source fence.** New `crates/shell/src/execution/workspace_open.rs`, minimum registration in
`execution/mod.rs`, directly necessary use/extraction of `workspace.rs` path helpers, and focused
tests for this component. Consume existing marker/config validation where suitable. Preserve
existing public CLI/discovery behavior; do not silently replace legacy discovery callers. No
initializer, HSA/store/backend IPC, config/policy writer, service, gateway or installer changes.
Any necessary change beyond that fence returns to META with the concrete dependency.

**Required behavior/proof.** An ordinary nested directory may resolve its enclosing candidate;
an intervening disabled, malformed, incomplete, incompatible, unreadable or unsafe boundary must
diagnose/refuse rather than fall through to an unrelated ancestor or defaults. Distinguish true
absence from I/O failure and partial `.substrate` state. Establish filesystem/symlink handling
from the existing security contract before implementation. Typed internal observations do not
establish a public unauthenticated inspection endpoint or disclose private owner records.
Exercise positive/nested, absence, disabled, missing marker, malformed/legacy/incompatible,
unreadable and path-alias/boundary cases as applicable, including a valid ancestor behind a
blocking boundary. Use before/after evidence for entries, content and modes plus decisive call-path
inspection to prove no read-side writes, initializer/reconciliation, identity allocation, processes,
cgroups, acceptance, service adoption or provider traffic. Tests use isolated temporary filesystem
fixtures and explicitly make no real owner-state/installed-runtime claim.

The TASK freezes its exact types and tests before edits, delegates implementation, and returns
focused tests with nonzero selected cases, formatting, relevant shell regression/build checks,
original implementation review and META strategic disposition. The authorized endpoint is a
reviewed working-tree component and remaining-obligation report; no commit, push, merge,
installation or live model call. This is partial PF-OPEN/PF-ZERO component evidence only. The
full P1 gates, real-owner IPC proof and publication checks below are not waived or declared passed.
Stop at this component; subsequent owner-backed load, absent initialization and exact no-send
read-cut composition require their own bounded assignments.

### Component acceptance — 2026-10-26

META accepted TASK04 at its reviewed-working-tree endpoint: new internal Linux inspection and
minimal module registration, on HEAD `623e01aab` plus the scoped uncommitted delta. Existing CLI
callers remain unchanged. Candidate results leave policy/owner validity unchecked; no authenticated
loading, initialization, production integration or full PF qualification is claimed.

The [final return](../../../../.codex/orchestration/e3f-retained-next-20261022/evidence/task04/final-return.md)
and [META acceptance](../../../../.codex/orchestration/e3f-retained-next-20261022/evidence/task04-meta-acceptance.md)
retain source correspondence, original paired observations and independent focused confirmations.
Focused 14-case tests, formatting, build and one existing config unit test passed. The named
`workspace_init` Cargo suite remains exit 101 (one guard-masked pass, five failures); a separately
recorded supported-prefix replay of all six behaviors and original assertions supplies accepted
preservation evidence. Both original P2 proof findings were resolved through that new evidence and
fresh independent confirmations, not a claimed suite fix or baseline runtime equivalence.

No commit, push, merge or installation is part of acceptance. Before any next task, META must bound
a concrete genuine owner-state/loading outcome, including missing schemas, writers and native read
access. No successor is dispatched; installer restoration remains separately owned by Task03.

### Selected owner-backed settings increment — 2026-10-26

The user authorized committing TASK04, then defining and dispatching the next bounded owner-backed
Workspace loading task. TASK04 and the preceding control-plane refresh are committed at
`ad98ea44b853956cdce31ba6a0b7ac8dce1ac6fe`. META selects `B01-B03-P1-B` / TASK05:
**internal native settings loading from the existing configuration and policy owners**. This is a
useful partial load, not `WorkspaceOpenReplyV1::Loaded` or authenticated consumer disclosure.

**Observable outcome.** From a fresh selected-directory inspection and existing trusted native
invocation/product-home context, return effective configuration and policy with truthful source
provenance. Return explicit unsupported states for Workspace identity, Lanes, destinations and
consumer read authorization: no supported owner readers/writers exist for those components.
Unsupported must not assert a particular owner record is absent. Plausible identity fixture bytes
must not become genuine state. Missing/invalid product home fails without scaffolding. Results and
errors stay internal; no consumer endpoint, serialization, blanket sensitive Debug output or logs.

**Source fence.** Extend `execution/workspace_open.rs`; add a fresh explicit-source resolver in
`execution/config_model.rs`, a minimal `execution/policy_model.rs` bridge, and explicit-source
composition in `crates/broker/src/effective_policy.rs` with minimum necessary `api.rs`/`lib.rs`
exports and focused tests. Existing native principal binding and `OpenedBootstrapHomeV1` reads
are the context footholds, not a new Workspace read grant. Do not modify HSA storage/trusted-home
policy, bootstrap layout, config/policy writers, initializer, persistence/revisions, service,
transport, gateway or public CLI callers. Return a concrete dependency before exceeding this fence.

**Required semantics.** Repeat strict inspection; never trust an old path-only Candidate. Collect
Workspace marker and optional policy from the same held boundary, without legacy ancestor
rediscovery or cache hits. Preserve genuine config/policy composition, environment/CLI precedence,
validation and provenance. Capture actual override inputs once. Preserve supported missing-policy
inheritance; malformed/unreadable/unsafe/replaced policy blocks regardless of gateway mode. Validate
regular files before reads and revalidate named/held associations and observed absence. The result
is a bounded observation, not an atomic snapshot, monotonic revision cut, ABA guarantee or authority
for later use. No invented identity, initialization offer beyond discovery absence, eligibility,
capability, reservation, hidden repair or runtime/provider effect.

**Proof and endpoint.** Freeze exact typed outcomes and native-context construction before delegated
edits. Nonzero focused tests cover distinct global/Workspace values and provenance, fresh reopen,
precedence/inheritance and legacy parity, nearer invalid/disabled boundaries, invalid policy in all
relevant modes, aliases/special files/substitution, wrong principal/invalid home, unsupported owner
components despite plausible fixture IDs, and unchanged before/after entries/content/modes. Inspect
the call path for no startup scaffolding, tracing, writes, reconciliation, allocation or execution.
Run applicable shell/broker tests, formatting and substrate build; bind commands, source and counts.
Retain TASK04's failed original CLI suite and alternative replay honestly; reuse only still-applicable
proof. Refresh/bind the exact GitNexus index and run symbol impacts before edits. Paired independent
implementation and strategic review govern acceptance. Endpoint: reviewed working-tree changes;
no commit/push/merge/install/live call or successor dispatch is authorized for TASK05.

**Next decision and concurrency.** Full `Loaded` needs genuine Workspace owner persistence and a
supported publication lifecycle for already-configured Workspaces; absent-only initialization cannot
silently migrate them. Select that outcome explicitly after this return. Only after TASK05 starts
will META identify a concurrent task and present its scope/conflict plan for human approval. No
concurrent dispatch is authorized by this selection; installer restoration remains separate.

### Settings-component acceptance — 2026-10-27

META accepted TASK05 candidate `0bb301583149c694eb5bac51d919368f26609cfd5e6721b6443d2390f023e6f6`
at the reviewed-working-tree endpoint on HEAD `ad98ea44b`. Both independent observations found no
blocking finding or unmet selected proof; the owner established implementation CLEAN and verified
final correspondence. The six-file delta remains uncommitted. [META acceptance](../../../../.codex/orchestration/e3f-retained-next-20261022/evidence/task05-meta-acceptance.md)
retains the exact endpoint and original review references.

The accepted outcome is internal config/policy observation with unsupported identity, Lanes,
destinations and consumer read authorization. Settings17, inspection14, config29 (private umask077),
policy14, broker-focused3, doctest1, formatting and build passed. The original broker suite remains
78 passed/8 failed with invocation-guard limitations; a separately source-bound supported-prefix
replay passed86 and was accepted by both observers as alternative preservation evidence. No original
suite fix, baseline runtime equivalence or full PF qualification is claimed.

The next dependent selection must establish genuine Workspace publication/lifecycle and owner-issued
reopen, including already-configured Workspaces without implicit migration. No successor is selected.
The B2.2 closure-recovery concurrency proposal still awaits human approval; installer Task03 remains
separate. Full P1/PF/E3/F14 remains open. No commit, push, install or deployment is included here.

### Publication and scheduling update — 2026-10-27

The user authorized committing TASK05 and dispatching B2.2 closure recovery. Accepted TASK05 source
and its control-plane updates are committed at `3eac4558034e9bf4c1f09dbdff07c1d639165c46`; no push or
deployment occurred. TASK06 now owns bounded B2.2 retained-evidence recovery, not product repair or
automatic terminal closure. The user requested a proposal for genuine Workspace owner publication;
that proposed successor is not authorized for implementation or dispatched. Earlier acceptance text
above preserves the original working-tree endpoint and then-pending approval chronology.

### Selected owner publication and reopen — 2026-10-27

**Accepted component endpoint (2026-10-27):** TASK07's reviewed working tree is accepted by
META on HEAD `3eac4558034e9bf4c1f09dbdff07c1d639165c46`, exact eight-file source set
`e32e2b39bf2e09465af35499e1d1b888cf5dd4bdf7a65aa7227e76e0593d9864`.
Genuine explicit owner publication, fresh-process status/reopen and partial settings identity
composition are implemented. Original P2 CLI effect-reporting finding was repaired and confirmed
by a different focused reviewer; independent strategic Continue remains applicable. Final CLI9,
owner6, settings17, five-checkpoint handler regression, build and formatting passed; applicable
prior regressions are retained. Original failed initialization/broker suites remain distinct from
supported-prefix alternatives. Linux fixture and fault-injection limitations remain explicit.
No commit, installation, live-user publication or full P1/PF/E3/F14 closure follows. Config/policy
revisions, Lane/destination and consumer-read authorization remain Unsupported. The selected
contract below remains the fence; [acceptance evidence](</home/spenser/__Active_code/.worktrees/substrate-runtime-refactor-product-authority-integration/.codex/orchestration/e3f-retained-next-20261022/evidence/task07-meta-acceptance.md>)
retains the exact reviews, repair and source correspondence.


The user explicitly approved META's publication proposal and its boundaries. META selects TASK07:
**genuine owner identity publication for an already-configured Workspace, followed by read-only
native reopen**. TASK06 continues independently as immutable B2.2 evidence recovery.

**Observable surface.** Add `substrate workspace init [PATH] --publish-owner` and
`substrate workspace status [PATH] --json`. Publication is an explicit native operator action on an
existing enabled, strictly validated Workspace; it does not invoke ordinary initialization, change
settings/Git storage/gitignore/permissions, enroll a consumer or grant execution authority. Reject
`--force`/`--examples` combinations. Before publication status diagnoses `OwnerStateMissing`; after
publication a fresh process returns the same validated owner-issued Workspace reference and state
revision. Idempotent publication preserves identity. Bind actual principal, existing trusted product
home and exact physical Workspace/control roots before disclosure. No generic consumer read grant.

**Approved partial schema.** Freeze strict `.substrate/workspace-state.v1.json` owner format before
edits, in new `execution/agent_runtime/workspace_admission_authority.rs`. Allocate opaque independent
Workspace and store-incarnation identities through the production publisher; never derive them from
path/session or fixture input. Include native owner/root binding and initial state revision. Reject
unknown/duplicate fields, unsupported versions and empty identities. Config/policy revision coverage
is explicitly **Unsupported**, not invented current revision values or a complete owner revision
vector. This selected partial identity schema does not satisfy the full revision contract below;
writer hooks, Lane associations and ABA/current-cut proof remain open.

**Write and read safety.** Serialize publication using a descriptor-bound per-Workspace lock; validate
owner/type/mode/ACL/device/link posture and named/held associations without changing permissions.
Exclusive staging, file sync, no-replace publication and directory sync must preserve any existing
identity. Conflicting/corrupt/unsafe state or partial publication residue refuses replacement or new
identity allocation; report known/uncertain effects on interruption. Read-only status/open never
repairs, cleans, migrates or completes publication. No move/copy/recovery rebinding is supported;
invalid physical/principal/home bindings fail closed. Do not imply universal restore/ABA detection
beyond demonstrated binding checks. Integrate the same validated owner reader into TASK05's internal
settings observation; keep absent/corrupt owner state explicit and Lanes/destinations/consumer
read authorization unsupported. No full Loaded, eligibility or provider/runtime claim.

**Source fence.** New owner module and minimal parent registration; `workspace_open.rs` for held
owner reads/composition; `workspace_cmd.rs` and `cli.rs` for the exact new modes; `routing.rs` for
new-action-only dispatch after native binding and before scaffold/trace; focused Linux tests and
corresponding documentation. Touch `invocation/plan.rs` only if the context handoff demonstrably
requires it. Reuse applicable security primitives without changing HSA/private-home policy. Existing
Workspace init/status-independent CLI behavior stays unchanged. Return other dependencies before
expanding this fence. No HSA/backend/custody/Lane/execution or global config/policy writer changes.

**Proof and endpoint.** Freeze exact schema, command behavior, security checks and effects before
delegated implementation. Tests use the actual publisher and fresh-process status with isolated
native-owned fixtures: publish/reopen/idempotence, concurrent one-identity outcome, binding/alias/
special-file/ACL/mode rejection, corrupt/conflicting/partial states and injected interruption points,
no read-side mutation, exact declared publication effects and preserved settings. Genuine positive
records originate from the production publisher; handwritten JSON is negative input only. Run
applicable TASK04/05 regressions, fmt and build; retain original failed suites versus alternative
proof distinctions. Paired independent reviews govern acceptance. Endpoint is reviewed working-tree
implementation and command/effect evidence, no commit/push/install/deployment or live user Workspace
publication. No successor is dispatched. Full P1/PF/E3/F14 remains open.

### Original admission clarification — 2026-10-21

The human product clarification on 2026-10-21 supersedes **B01-B03-P1-B01**, the earlier
setup-issuer prerequisite. Initially Collider opens Substrate Workspaces on the same host.
An existing configured Workspace loads through Substrate with its current known configuration,
policy, Lanes and destinations. There is no separate Connect Collider, enrollment, pairing,
client-key registration or issuer-approval ceremony. An absent Workspace offers Substrate
initialization/onboarding; that is where the user chooses and approves initial settings once.
Folder selection is discovery input, never Workspace identity or general operation permission.

**Admission contract bound** means the proposed owner entrypoints, minimal source fence, schemas
and required proof below delimit future implementation work. It does not mean source dispatched,
APIs implemented, owner signoff obtained or runtime ready. Final source-admission assessment is
separate from independent documentation review and TASK/META acceptance. No implementation,
initialization, installed IPC or consumer send is authorized by this documentation task.

The prior packet was independently reviewed with **no findings** and accepted as documentation
with NOT ADMITTED status under its earlier external-client setup assumption. The human correction
supersedes that product assumption and its blocker; it is not dismissal of a reviewer P2 and does
not rewrite prior acceptance records. Later external/third-party ingress may need a separate
protocol; it is outside this initial native path, not a prerequisite to ordinary Workspace open.

## Integrated outcome and authority set

For the initial native same-host Collider adapter, Substrate discovers and validates the selected
location, loads an existing Workspace, or returns an explicit initialization offer for genuinely
absent state. It supplies current authorized configuration/policy and owner-backed Lanes and
known destinations with capability/availability qualifications. The separate no-send operation
checks one **existing** exact HSA session/caller/backend/World and genuine Workspace-owned
Lane/World association. Its eligibility observation describes a checked read cut; it is never
an execution capability, reservation, accepted request or permission for a later send.

Load only the [selective-load protocol](../../foundations/per-slice-context-assembly-protocol.md),
[authority vocabulary](../../foundations/authority-vocabulary.md),
[reading/update rules](../../foundations/reading-and-update-rules.md),
[semantic status rules](../../foundations/semantic-status-labels.md),
[authority amendment](../../architecture/authority-map.md#proposed-b01b03-consumer-admission-responsibility-amendment--2026-10-21),
[composition boundary](../../seams/runtime-event-receipt-supervision-and-retained-runtime.md#proposed-b01b03-composition-boundary--2026-10-21),
[B1/B2.1 extension boundary](../../b1-b2-1/contracts-and-gates.md#proposed-b01b03-extension-boundary--2026-10-21),
[B01 §§3.2–3.4](../../../../docs/contracts/draft/collider-b01-b05-handoff.md#32-immutable-request-identity-and-the-admission-boundary)
and [B03 §§4.6–4.9](../../../../docs/contracts/draft/lane-world-binding.md#46-recommended-model-authorized-selection-not-session-rebinding).
Those accepted design responsibilities are target extensions, not current APIs or component-owner
signoff. This packet adds a distinct **preflight** operation before Submit/Reconcile; it does not
reuse their request/disposition family or redefine the completed B1/B2.1 acceptance contract.

**Hard exclusions:** no client request key, consumer correlation, proposal identity allocation,
association reservation/hold, historical acceptance publication, context resolution/material seal
or injection, `MaySend`, runtime admission gate/consume/fence, execute, task launch, delivery,
provider traffic or credentials. Initialization is a separately approved Workspace operation, never an implicit preflight effect.
No retained route, E3-F continuation, scheduler, Lane move,
custody transfer, filesystem/merge/backing selection, plural HSA binding, Mac/Windows source work,
Collider or Handbook changes. Lanes remain durable Workspace entities with multiple Worlds and
one current association per associated World; a session, path or branch is never a Lane identity.

## Source correspondence and current status

Authoring baseline: Mac branch `docs/substrate-collider-contracts`, HEAD
`aa5ef73ae087aa82d1f952ba6dd5c92361b8e24f`. The preceding five dirty documentation changes
are accepted prior-phase bytes, separate from this packet's delta. The present correction changes only their directly affected opening/ingress paragraphs;
request, full-send, retention and recovery rules remain intact. The [author observation](../../../../.codex/orchestration/2026-10-21-b01-b03-preflight-packet/evidence/author-preedit-observation.json)
records the protected baseline, narrow fence, owner release and attributed fresh Linux source
correspondence. The separately retained [source correction](../../../../.codex/orchestration/2026-10-21-b01-b03-preflight-packet/evidence/author-source-correction.json)
supersedes the original snapshot-integration inference using actual production callers. These
are evidence, not product authority or runtime results.

Fresh read-only Linux relay `/root/bind_linux_source` binds D
`/home/spenser/__Active_code/substrate` and W
`/home/spenser/__Active_code/.worktrees/substrate-runtime-refactor-product-authority-integration`
to HEAD `088f62d81c182b92cac404983b8bf512becc829c`, tree
`b40da914c1e5d73fc79f13c0946f6af9c97455d1`. D is clean; W retains exactly its eight unstaged
files. W binding `cd89226b984def497b8a67f92a329e430f9d2899f16cfd743d4f7db07c55f427` and diff
`efcd97c433e33bc8a5182d7fc4a8f7db4a1946625dfeed3ab252291e6785c8c5` match the retained stop.
This retained correspondence resolves the earlier source-location question only; it is not a new
Linux visit or IPC/runtime proof. Current narrow local Workspace source inspection is recorded in
[the author observation](../../../../.codex/orchestration/2026-10-21-b01-b03-workspace-preflight-correction/evidence/author-preedit-observation.json).

| Existing source owner / locator | Actual foothold and constraint |
| --- | --- |
| HSA [`facade.rs::resolve_current_exact`](../../../../crates/shell/src/execution/agent_runtime/host_session_authority/facade.rs), local line 1310 | Supplies strict exact session/caller/root/store/policy facts, but calls `store::read_opened_root_v2_or_v3` and semantic preflight. That path reconciles temps under exclusive root lock (`store.rs:1653`, `store/platform/transaction.rs:3682–3684`); it is **not** a nonreconciling read. |
| HSA [`store.rs::read_existing_accepted_work_authority_snapshot`](../../../../crates/shell/src/execution/agent_runtime/host_session_authority/store.rs), lines 83 and 862–877 | Linux-only raw HSA/E2/B1 snapshot via `with_opened_existing_versioned_read_only_snapshot` (`store/platform/transaction.rs:547+`), reused by the current accepted-work projection path (`orchestrator_world_dispatch.rs:1740`, `dispatch_policy_commitment.rs:2031`). The old dead-code annotation describing future integration is stale. This existing production use does not supply complete Workspace/current-authority or custody validation; B2.2 is not reopened. |
| [`StateStore::resolve_world_work_registry_authority`](../../../../crates/shell/src/execution/agent_runtime/state_store.rs), line 5321 | Resolves HSA then binds receipt registry and supervisor; preflight cannot call it and claim zero persistent effects. |
| Backend [`session.rs`](../../../../crates/world/src/session.rs), `ownership_from_metadata:672–717`, `read_canonical_metadata:1146–1158` | Existing private custody decoding is the narrow reader foothold. `find_compatible_session` recovery reaches `ensure_cgroup_attach_target:575–578`; it is not a pure custody lookup. |
| [`world-service/service.rs::exact_bound_world_ownership_adoption`](../../../../crates/world-service/src/service.rs), local line 3264 | Constructs ownership adoption used on execute; adoption publishes ownership. Neither execute nor adoption can bootstrap a read-only eligibility check. |
| HSA [`stop.rs::issue_stop` / `validate_issue_request`](../../../../crates/shell/src/execution/agent_runtime/host_session_authority/stop.rs), lines 108 / 392–419; broker [`api.rs::evaluate`](../../../../crates/broker/src/api.rs), line 40 | Exact PublicCli/current HSA/lineage plus explicit `session_stop` capability is an operation-specific authorization precedent. Command evaluation and that Stop capability grant no Workspace initialization or arbitrary execution permission. |

Line numbers are source locators, not cross-host byte identity claims. The Linux relay and Mac
baseline are separate source states; a later implementation must bind exact files/symbols at its
actual checkout, include fresh impact analysis, and retain current raw command/source evidence.
No seam is promoted to `ContractCorrectAndProven` by this packet.

## Substrate Workspace open, load and initialize

These are **proposed native owner adapters**, not existing exported APIs or a new authority
framework. Collider invokes Substrate; it does not parse/write `.substrate` or synthesize policy,
Workspace identity, Lanes or destinations. Authentication uses the existing same-host native
Substrate caller context; caller identity and trusted root/store context are constructed inside
Substrate, not accepted from browser/request fields. No custom enrolled-client/key handshake is
required for this initial route. A future cross-process binding must preserve that context and
privacy; generic World execute and the E3 preparation socket are not substitutes.

| Existing owner / actual source entrypoint | Supported foothold and minimum proposed change |
| --- | --- |
| [`execution/workspace.rs`](../../../../crates/shell/src/execution/workspace.rs), `find_workspace_root`, `find_workspace_root_any`, marker/policy/disabled path helpers | Existing discovery looks for `.substrate/workspace.yaml`, canonicalizes with fallback and searches ancestors; enabled lookup skips disabled roots. New `execution/workspace_open.rs::WorkspaceOpenAdapter::inspect_existing` must inspect the selected boundary first, preserve actual root identity and distinguish disabled, missing-marker, malformed, incomplete and absent states before ancestor/default fallback. A present but invalid `.substrate` cannot become an unrelated parent Workspace or an initialization offer. An ordinary subdirectory may resolve its genuine enclosing Workspace only when no intervening invalid/disabled boundary exists. |
| [`workspace_cmd.rs`](../../../../crates/shell/src/execution/workspace_cmd.rs), `handle_workspace_command`, private `run_workspace_init`; [`cli.rs`](../../../../crates/shell/src/execution/cli.rs), `WorkspaceAction::Init`, `WorkspaceInitArgs` | Current `substrate workspace init [PATH]` creates directories/internal Git storage, fills missing YAML and edits `.gitignore`, even without `--force`. It does not implement this product onboarding contract or create the complete Lane model. Extract only the minimum initializer writer for `WorkspaceOpenAdapter::initialize_absent`; require strict absence plus the user's exact approved settings, expose partial outcomes, never invoke repair/enable/force during open. |
| [`config_model.rs`](../../../../crates/shell/src/execution/config_model.rs), `parse_config_patch_yaml`, `resolve_effective_config_with_explain` / explicit-bootstrap-home counterpart | Existing sparse config parsing, precedence, validation and provenance are reused. New `WorkspaceOpenAdapter::load_existing` reads fresh validated owner inputs; stat-key cache hits are not a coherent preflight cut. Preserve existing config semantics and reject unsupported legacy state. |
| [`policy_model.rs`](../../../../crates/shell/src/execution/policy_model.rs), `resolve_effective_policy_with_explain_for_bootstrap_home`; [`broker/effective_policy.rs`](../../../../crates/broker/src/effective_policy.rs), `resolve_effective_policy_with_explain_from_global_source` | Reuse the selected trusted bootstrap home's global policy plus current Workspace patch and existing precedence. Existing `policy_cmd.rs::run_current_show` is a command-level precedent; a machine adapter returns typed authorized provenance/denial rather than parsing printed output. Do not change broker precedence or treat an `Allow` as general permission. |
| New `agent_runtime/workspace_admission_authority.rs`, `read_workspace_identity`, `read_current_lanes_and_destinations`, `read_current_world_association`, `resolve_current_read_access` | Proposed Workspace-owned identity/association read adapter alongside HSA; **absent as a complete supported source path**. Return genuine owner records and explicit partial/unsupported availability. Existing YAML marker proves discovery only; never derive a Workspace/Lane ID from path/session or seed fixture records. Missing genuine state is diagnosed; it cannot be silently initialized, migrated or regranted. |

Proposed native versioned semantics (names/layout are future source work, not current wire APIs):

```text
OpenWorkspaceV1 = { version, selected_location }
WorkspaceOpenReplyV1 = Loaded(current_owner_view, component_statuses)
                    | InitializationOffered(selected_location, proposed_settings)
                    | Denied | Disabled | Invalid | Incomplete | Unsupported | Unavailable
InitializeWorkspaceV1 = { version, selected_location, approved_initial_settings }
WorkspaceReadViewV1 = { workspace_ref, root_store_binding, config_provenance,
                        policy_provenance, lanes, destinations, owner_revision_vector }
```

`selected_location` is an untrusted discovery hint. The owner constructs root/store binding,
initiator and every returned reference. `Loaded` is useful current owner-backed state with
explicit component coverage, not destination eligibility; unavailable Lane/destination support
cannot be filled by UI fixtures. An invalid/incomplete required identity or configuration fails
load; an unsupported optional component remains visibly unsupported. Policy may prohibit reads;
unauthorized callers receive no target existence, private path, identity or configuration data.

- **Existing valid state:** load current known settings and owner-backed objects without another
  setup approval. Reopen repeats current validation and disclosure checks; it never replays setup
  or extends permission. Read-only open performs no hidden recovery or runtime launch.
- **Genuinely absent state:** offer initial settings and effects for one user approval. Denial or
  cancellation writes nothing. Substrate's initializer alone applies the approved settings; it
  does not obtain provider credentials, start execution or treat onboarding as a blanket grant.
  Any initial durable Workspace/Lane records must use their genuine Substrate owner schema;
  initialization need not manufacture a World or an execution destination.
- **Failed/interrupted initialization:** report effects known to have occurred and incomplete
  state. Reopen diagnoses that state; it does not call init again, fill missing files, infer
  success, regrant permission or silently migrate. Any later completion/repair is a separate
  explicit Substrate operation outside ordinary open/preflight and outside this packet's proof.
- **Existing invalid/disabled/incompatible state:** diagnose and refuse the affected load/read;
  no enabling, `--force`, parent/default-policy fallback, regenerated identity or automatic
  onboarding. Missing marker within existing `.substrate`, bad/unknown schema, unsupported legacy
  files, substituted/symlinked root, unreadable policy and partial owner state stay distinct.

## Current-read schema and stable cut

The bounded TASK22 current-actor component is a distinct private increment: explicit
native owner requests, independent current read/publication decisions under existing
policy precedence, an immutable separately versioned first Workspace association,
actual service backend custody, and a mandatory nonreconciling HSA observation. Native
before/after checks release Workspace locks before IPC; the service joins its own local
owner/backend/HSA observations. Its nonsecret actor projection is not the complete
`EligibleObservation` or the all-invalidators monotonic/ABA cut specified below. Strict
identity-only owner-v1 bytes and serialized execution policies remain unchanged; no
real permission is activated by this facility. Full PF/init/read-cut/later-send and
installed qualification obligations remain open.

The proposed Workspace adapter returns an opaque owner-qualified `WorkspaceReadCutV1` containing
root/store incarnation, config/policy source revisions and protected content commitments, genuine
Workspace/association revisions, HSA snapshot identity and backend custody incarnation/revision.
This is internal evidence, not a bearer capability or exposed secret-derived digest. Config and
policy provenance includes the exact defaults/global/Workspace/environment/CLI inputs actually
used; the adapter neither invents new precedence nor assumes UI state is current authority.

The proposed strict `WorkspaceStateV1` record is Workspace-owned, separate from HSA: version,
opaque stable `workspace_ref`, canonical root/store binding, monotonic state/config/policy
revisions, Lane refs and exact World/generation→Lane association entries with monotonic association
revision and backend-custody ref. Proposed persistence is `.substrate/workspace-state.v1.json`;
this is a **new schema**, not an existing file or enrollment/grant directory. Initialization may
publish its genuinely new empty Lane/association collections with the approved initial settings;
it does not create an existing World or permission. Existing owner records need a supported exact
reader; absent/unknown/conflicting state is incomplete/unsupported, never synthesized or silently
migrated on open. Strict decoding rejects unknown/duplicate fields and unsupported versions.
The new Workspace owner serializes revision publication and config/policy updates with its
per-Workspace lock and recoverable atomic replacement; crash/partial state refuses eligibility.
Global-policy/config revisions remain with those existing owners, not copied into a second grant
store. No lock spans backend IPC. This specifies the minimum future record responsibility; actual
production support and interruption behavior require the PF proof below.

Use a bounded two-collect protocol: read fresh validated config/policy/Workspace/HSA snapshots,
read existing custody, then collect the complete owner revision/incarnation vector and current
read authorization again. Return eligibility only when every relevant invalidator is covered by
its owner's monotonic revision and both collects join unchanged. Config/policy owner writers
must advance the corresponding cut revision under their own persistence lock; new minimum
`config_cmd.rs` / `policy_cmd.rs` writer hooks are confined to that bookkeeping. Existing stat
caches, timestamps or equal final YAML bytes cannot prove no intervening A→B→A. Unmediated edits,
missing revision coverage or unsupported owner snapshot protection must produce changed,
unsupported or integrity failure; never repair metadata during a read. Exact revision storage
must stay in the Workspace owner's namespace, not add arbitrary files to HSA's closed layout.
A stable observation can become stale immediately afterward; later use reauthorizes independently.

## Proposed no-send operation and failure semantics

Proposed `WorkspaceOpenAdapter::preflight_exact_destination` consumes the native Substrate caller
context and an owner-qualified Workspace reference from loading, plus an exact expected Lane,
World/generation/association revision and existing session/caller/backend expectation. All client
selectors remain lookup hints. Reject unknown/duplicate fields, unsupported versions, empty
identities and execution/context/payload/request-key fields. The caller cannot supply authoritative
principal, root/store identity or grant. The initial route has no enrolled-client challenge state.

| Responsible owner / proposed read interface | No-send responsibility |
| --- | --- |
| Native Substrate caller boundary and Workspace `read_workspace_identity` | Bind actual native initiator and trusted root/store; validate genuine Workspace identity. No client enrollment or path-derived identity. |
| Workspace `resolve_current_read_access` with existing policy owners | Check current operation/resource and disclosure authority, exact initiator and HSA actor. Existing execution/history permissions remain distinct; loading does not mint delegation. |
| HSA `read_current_exact_nonreconciling` (new adapter) | Apply existing strict current exact session/caller/backend/World/root/store and policy validation to one verified read-only snapshot. No semantic-preflight reconciliation, registry/supervisor bind, bootstrap or repair. |
| Workspace `read_current_world_association` | Authenticate genuine association and monotonic revision/incarnation; compare expected Lane/World and revision without reserving or holding it. A→B→A cannot recreate an old revision. Siblings remain independent. |
| Existing backend owner `read_existing_custody_nonreconciling` (new export) | Read/authenticate exact physical World/generation, owning session/backend and custody revision/incarnation from existing protected metadata; no recovery/attach/adopt/create/cleanup. Missing/conflicting custody fails. |

Read current authorized owner snapshots, validate all exact joins, then revalidate the complete
relevant owner revision/incarnation vector and current read authorization. A positive result
requires a supported stable read cut: every relevant invalidator must advance its owner's
monotonic revision, and a bounded double collect must establish unchanged revisions across the
checked interval, or an equivalent owner-supported coherent snapshot must do so. No global lock
crosses backend IPC. Drift, unresolved temps/transactions, unsupported currentness coverage or
unreadable authority cannot yield eligibility. Return changed/unavailable/unsupported rather
than reconciling or repeatedly retrying until a desired answer appears. Changes after the cut
remain possible; observations carry owner revisions, never frozen permission.

Proposed semantic replies (not B01 request dispositions):

| Reply | Meaning and allowed disclosure |
| --- | --- |
| `EligibleObservation` | Authorized exact owner-qualified destination/actor and read-cut revision vector checked now. No bearer token, request/work identity, hold or permission for later use. |
| `Denied` / `InvalidRequest` | Privacy-filtered authentication/authorization or strict request failure. Unauthorized callers learn no target existence, principal match, association, refs or material. |
| `IneligibleObservation` / `ChangedDuringRead` | Authorized caller receives bounded stale destination/association, denied current delegation, incompatible posture or changed-cut observation where disclosure itself is authorized. |
| `Unsupported` / `Unavailable` / `IntegrityFailure` | Unsupported profile/owner evidence, unreadability/transport failure, or authenticated inconsistency. None means absent destination, safe resubmit, no past work or successful setup. |

No durable observation lookup, request reconciliation, background recovery, audit-triggered task
allocation or automatic retry follows a reply. Adapter/service loss requires a fresh current
read; it cannot execute. Bounded nonsecret diagnostics may name authorized opaque refs/revisions
but exclude private roots, prompts/material, private authentication material and raw
credential-derived hashes. Later Submit must authenticate and authorize again, allocate
its own original identity, reserve through owners and satisfy every then-current effect fence.

## Proposed later implementation fence

The following is a bounded source-admission contract, **not a source dispatch now**. New symbols,
records and test launchers are proposed and absent; existing entrypoints are identified above.

| Source owner / exact proposed entrypoints | Minimum work and later proof launcher |
| --- | --- |
| New `crates/shell/src/execution/workspace_open.rs::WorkspaceOpenAdapter::{inspect_existing,load_existing,initialize_absent,preflight_exact_destination}`; minimum registration in `execution/mod.rs` | Native typed open/init/read binding above, no external issuer/client-key framework. Proposed `cargo test -p shell --lib workspace_open_ -- --nocapture` must discover positive/reopen/denied/malformed/incomplete cases. |
| Existing `execution/workspace.rs` discovery/path helpers; `workspace_cmd.rs::{handle_workspace_command,run_workspace_init}`; `cli.rs::{WorkspaceAction,WorkspaceInitArgs}` | Strict selected-boundary classification and minimum approved-absent initializer extraction; preserve unrelated CLI/sync/checkpoint/rollback behavior. No implicit repair, World creation or provisioning. Proposed `workspace_open_` tests include partial init and cancelled/failed/reopen outcomes. |
| Existing `config_model.rs` fresh parse/effective resolvers; `policy_model.rs` trusted-home resolver; minimum `config_cmd.rs::{run_global_init,run_global_set,run_global_reset,run_workspace_set,run_workspace_reset}` and `policy_cmd.rs` corresponding init/set/reset writers | Reuse exact precedence/validation, provide cut-bound fresh inputs and minimum monotonic config/policy revision bookkeeping only. Existing broker resolver is consumed, not rewritten. Missing/unmediated revision coverage refuses eligibility. Proposed `cargo test -p shell --lib workspace_read_cut_ -- --nocapture` covers same-bytes ABA, cache invalidation, current-read denial and concurrent writes. |
| New `agent_runtime/workspace_admission_authority.rs::{read_workspace_identity,read_current_lanes_and_destinations,read_current_world_association,resolve_current_read_access,collect_read_cut}`; minimum parent registration | Typed existing genuine owner reads, no scheduler, grant issuer or duplicate HSA authority. Implement strict `WorkspaceStateV1` / `WorkspaceReadCutV1` and the named Workspace-owned revision storage; bind exact bytes at source start; no broad HSA-layout exception. Missing production state stays incomplete/unsupported. Proposed `workspace_read_cut_` tests plus real-owner IPC proof below. |
| HSA `facade.rs::read_current_exact_nonreconciling` (new), `store.rs` / `store/platform/transaction.rs` minimum snapshot access | Reuse production-adopted `read_existing_accepted_work_authority_snapshot` and `with_opened_existing_versioned_read_only_snapshot`; apply existing exact validation without temp reconciliation, B1/Supervisor bind, bootstrap or repair. Preserve transitions/codecs/root layout. Proposed `cargo test -p shell --lib preflight_ -- --nocapture`. |
| `crates/world/src/session.rs::read_existing_custody_nonreconciling` (new) and minimum `world/src/lib.rs` export | Export authenticated existing canonical metadata/ownership read under owner-supported snapshot protection; never call `find_compatible_session`, attach/create cgroup, adopt or cleanup. Proposed `cargo test -p world --lib custody_read -- --nocapture`. |
| Only if actual existing backend IPC requires it: `world-service` router/service plus versioned agent API model/client | Freeze one typed custody-read route and transport symbols before touching them; consume the export above with the same native caller/owner binding. No generic execute fallback/service gate/ownership transfer. Proposed real Linux test target below must bind this route if selected; wider access/layout changes return as exact source-scope conflicts. |

`B01-B03-P1-B02` remains **implementation and proof work**, not missing product authority: a
complete nonreconciling joined Workspace/current-authority/custody path does not exist yet.
These concrete adapters and exact read-cut coverage are what implementation must supply. They
do not revive B01 under an owner-TBD, renamed issuer or general management-framework prerequisite.

## Dependencies and required proof

| Item | Current disposition | Required implementation evidence |
| --- | --- | --- |
| B01-B03-P1-B01 separate setup issuer | **SUPERSEDED by human product clarification**, not a reviewer finding disposition. | Initial same-host configured open works without new enrollment/approval; absent initialization obtains settings approval once. |
| B01-B03-P1-B02 owner adapters/readers | **BOUNDED PROPOSED IMPLEMENTATION / PROOF REQUIRED.** No source dispatch or runtime readiness. | Actual Workspace identity/Lane support, strict root/config/policy joins, complete monotonic read-cut coverage, nonreconciling HSA/custody export and PF gates below. |
| Linux source correspondence | Retained resolved documentation evidence; no new host audit. | Rebind exact checkout/source/allowlist before later symbol edits; reuse valid evidence and obtain actual IPC proof. |

At a later authorized source start, bind exact candidate/schema/owner revision layout, actual
backend custody transport and test launchers to this fence; run required symbol impact analysis.
A concrete unsupported owner/export/layout conflict returns its path/symbol/required change for
assessment. API absence is assigned implementation work. No implementation follows this docs
correction automatically, and no fixture is accepted as genuine production Workspace/Lane truth.

| Gate | Required positive and negative proof on the exact source candidate |
| --- | --- |
| PF-OPEN | Real configured Workspace opens/reopens through Substrate with current known config/policy/Lanes/destinations and explicit component availability, with no Connect/enroll/pair/issuer approval or hidden repair. Nested/displaced/disabled/missing-marker/legacy/malformed/incompatible/unreadable states diagnose/refuse; no wrong-parent/default fallback or caller/path-derived identity. |
| PF-INIT | Truly absent state offers initial settings approval once. Denied/cancelled setup writes nothing; successful approved init returns genuine owner state and reopen does not repeat approval. Failed/interrupted/partial init reports known effects, remains incomplete on reopen and never auto-repairs/regrants. No runtime/World creation or arbitrary `.substrate` client write. |
| PF-AUTH | Native initiator/root/store binding is owner-constructed; spoofed principal/Workspace/context, wrong root/store and current policy/read denial fail without existence/material disclosure. Folder access does not grant execution, history, mutation or delegation. |
| PF-EXACT | Existing session/caller/backend/World/root/store/custody and genuine association joins succeed; mismatch, ambiguity, missing ownership and current operation/resource denial refuse eligibility. No session-as-Lane or fixture identity. |
| PF-REV | Genuine selected-World revision succeeds; stale A→B→A, config/policy/current-read/HSA/custody invalidation and incomplete state refuse eligibility. Prove bounded double collect plus every owner invalidator; unchanged sibling remains independent. Equal bytes/cache stats alone cannot pass. |
| PF-ZERO | Snapshot authority/config/policy/association/B1/B2.1/E2/backend metadata, directory entries/modes and process/cgroup/runtime effects before/after positive and every failed open/preflight. Zero durable lookup repair, setup/reconcile, identity/key/proposal allocation, reservation, acceptance, `MaySend`, execute, delivery, provider traffic, adoption/custody/cgroup change. Separately measure approved initialization's listed Workspace writes; they are not counted as read-only success. |
| PF-LATER | Change/revoke destination or permission after eligibility; observation grants no later send. No accepted-work/request ID or capability is returned; future submission reauthenticates/authorizes and satisfies all full-send fences. |

All following commands are **proposed later proof, not execution in this documentation task**.
The new filters above must discover nonzero intended cases. New integration target/launcher
`cargo test -p shell --test consumer_admission_preflight_linux -- --nocapture` does not exist yet;
it must exercise real Substrate-owned state and the actual native adapter/backend IPC route.
The designated ignored real-profile case is also absent:

```sh
SUBSTRATE_PREFLIGHT_PROOF_PROFILE=/absolute/protected/no-send-profile.json cargo test -p shell --test consumer_admission_preflight_linux test_real_workspace_open_init_and_existing_world_no_send -- --ignored --exact --nocapture
```

This test-only proposed profile selects selected location, trusted native adapter context, genuine
owner-issued Workspace/Lane/association references and an existing HSA session/caller/backend/World.
It contains no provider credentials, enrollment/issuer grant or client signing key. Bind its actual
parser, fixture distinction and installed launcher at source admission. Missing real owner state,
profile or selected IPC route must fail visibly, never skip or seed a production fixture. Exercise
PF-OPEN–PF-LATER with nonzero case counts and measured zero read-side effects. Initialization is
separately approved/source-bound and its partial writes are reported; no provisioning is assumed.

Retain command, cwd, baseline/candidate hashes, platform, actual exit and raw output for these
checks, `cargo fmt --all -- --check`, `cargo build -p substrate --bin substrate` and, if service IPC
changes, `cargo build -p world-service --bin world-service` plus `cargo test -p world-service`.
Before later source publication include applicable workspace build/clippy/tests and Linux
`substrate world doctor --json`, `substrate shim doctor --json`, `substrate health --json`.
Doctor output cannot replace PF gates; missing installed/platform evidence remains explicit.

**Later full-send dependencies:** preserve H1–H3, conditional H4/S1 context/material/use/delivery;
B1/B2.1/E2/B2.2 composition, B03 reservations/history and runtime consume/close/fence before real
submission. Applicable security/profile and D1–D3 requirements remain; E3-F/I1/I2 still govern
retained routes. These are not automatic predecessors to normal open/no-send reads. Existing
later external-client enrolled ingress remains outside this initial route. Preserve close-before-
effective-commit revocation and resolved B01B03-D01 original-material/unresolved-evidence custody
permission until positive resolution; opening/onboarding grants neither execution nor later use.

## Stop, review and publication endpoint

Stop for a newly demonstrated authority/layout/ownership conflict, drift in frozen source/protected
bytes, required API scope beyond the proposed fence, or inability to prove genuine owner state,
approved absent-only initialization and current exact read with zero read-side effects. Report
the exact owner, unsupported input/event, consequence and minimum choice/evidence; do not widen scope or weaken a
gate to manufacture eligibility.

The authorized present endpoint is independently reviewed **working-tree documentation** with
admitted-or-blocked disposition, exact delta/source correspondence and link/anchor/table/status/
fence/whitespace/protected-byte checks. Apply the
[development review contract](../../contracts/development-review-and-remediation-contract.md)
through the task's governing causal protocol; original findings and owner assessment remain
separate. No source dispatch, runtime test, installation, component-owner signoff, commit,
publication to a remote, integration, installed proof or Gate C follows from documentation
completion. TASK assessment and META acceptance remain separate decisions.

The [dated E3 correction](../e3-agent-config-projection-and-gateway-adoption.md#current-e3-f-status-correction--2026-10-21)
supersedes inherited “undispatched” current projections. E3-E stays closed; E3-F is paused after
dispatch, its W8 result unaccepted/unclosed, I1/I2 unresolved and `c381` published-unintegrated.
Older extracted/as-of status text remains chronology. Inherited navigation wording in
`slices/README.md`, Track E and E3 contract headers is nonblocking projection drift under that
canonical dated correction; it grants no continuation and is not rewritten by this packet.


### Selected HSA typed-object snapshot reads — 2026-10-27

Under the user's instruction to keep runtime work moving in parallel with installer integration
and live proof, META selects TASK10 as the next P1 component after Workspace commit
`29cf8e8223c9d166263e56de219428d2b3858aa9`. The existing P1 fence requires a
nonreconciling current-exact read; its storage transaction already holds and validates the existing
versioned root but lacks authenticated typed-object reads. Existing typed reads enter semantic
preflight, so using them would leave reconciliation reachable.

**Outcome:** HSA-private authenticated typed-object reads within the existing nonreconciling
snapshot transaction. Bind the frozen root revision/store, present index entry, exact kind/schema/ref,
canonical length/commitment and existing HMAC parent/key rules. Retain/revalidate actual object/key
files and held namespace bindings on successful and error exits. Missing, unsafe, changed, pending
or conflicting state refuses without repair, initialization, secret export or new logging.

**Fence:** `host_session_authority/store.rs`, `store/platform/transaction.rs`, and only necessary
verification extraction in `store/platform/object_persistence.rs`, under
`crates/shell/src/execution/agent_runtime/`, plus directly necessary colocated tests. Preserve codecs,
layouts, writers, current reconciling callers and E2-RM/B1 behavior. No facade migration, complete
current-exact resolver, Workspace revision persistence, consumer endpoint, service IPC or general
storage framework. Any additional file requires a demonstrated dependency returned to META.

**Proof:** genuine production-published isolated V2/V3 object reads, canonical/context-HMAC cases,
stale/absent/wrong/retired/corrupt/unsafe/substituted/pending material, revalidation on both exits,
unchanged entries/content/modes and call-path inspection excluding reconciliation and side effects.
Run focused meaningful HSA/E2-RM regressions, formatting and substrate build; retain proof limits.
Freeze API/contract before delegated changes and source/proof before paired review. Endpoint is
reviewed working tree only: no commit, integration, installation, full read-cut, Loaded or PF claim.

TASK10 uses a separate checkout/build/test state at the named Workspace commit. Installer TASK08
owns only integration/commit of the accepted three-script repair; fresh TASK09 then owns the
clean reinstall and exact-model normal-command proof. TASK10's unintegrated source is excluded
from that installation. Actual task identities and delivery states remain in the shared workflow.

**Accepted finite endpoint (2026-10-27):** META accepted TASK10's reviewed working tree on
`29cf8e8223c9d166263e56de219428d2b3858aa9`, full patch SHA256
`3cb6d32b09e302b470af70c0663146e6cc31c521b5fac99f8454dcb6035d28f0`.
Original implementation discovery found a canonical-payload/reference schema mismatch; the local
repair and three added tests received confirmation from a different fresh reviewer. Implementation
is CLEAN and the original strategic Continue remains applicable. All 17 focused tests and 16 E2-RM
tests pass, as do formatting and the substrate build. The HSA suite remains 264 passed, 1 failed,
2 ignored; the inventory diagnostic failure also occurs on the exact baseline. No full-suite green
claim is made. Original results, raw commands and source correspondence are retained under
`evidence/task10/` in the shared workflow.

The component authenticates a supplied reference within the held snapshot; selecting the current
graph edge and authoritative context remains the resolver's responsibility. Commit/integration,
current-exact resolver adoption, full read-cut, Loaded and PF completion remain pending. The
accepted source remains isolated from the installer candidate.

### Selected HSA current-exact reader — 2026-10-27

META selects TASK13 under the continuing parallel-runtime authorization. Its isolated checkout
starts at `33ebdf0d6a277f45dbc5603063bbcdbe521252ce` plus the exact accepted TASK10 patch,
retained separately from the new delta. Publication of TASK10 is not a prerequisite to this isolated
dependent work; neither component becomes an installer input.

The new HSA-private Linux reader selects graph references and verification contexts from the held
authority root and enforces existing exact session, caller, lineage, descriptor, attach, policy,
bound-store and expected-observation checks. Its claim is limited to roots supported by the strict
nonreconciling snapshot. Before material implementation, demonstrate a genuinely production-published
useful V3 state and freeze the support matrix. Particular continuation/interleaved histories may
require the existing external-history validator; do not relax the shared decoder or silently deliver
a V2-only substitute if that dependency prevents useful V3 support.

The fence is HSA `facade.rs`, signature-preserving reader injection in `transition.rs`, and minimum
bridge/captured-root access in `store.rs` and `store/platform/transaction.rs`, with colocated tests.
Existing production resolver callers, schemas, codecs, writers, runtime ownership, StateStore,
Workspace and backend custody remain outside this increment. Required proof covers production
V2/V3 positives, exact graph/context checks, stale and conflicting authority, corruption and pending
refusal, unconditional exit revalidation, and no durable or runtime effects. Run affected focused
regressions, TASK10 tests, E2-RM, formatting and the substrate build within an allocated private budget.

Endpoint: paired-reviewed working tree only. No full support parity, production adoption, complete
read cut, Loaded, exact-destination eligibility or P1/PF completion is claimed. Original advice,
assignment, actual owner identity and readiness/return states are retained in the shared workflow.

**Accepted TASK13 endpoint (2026-10-27):** META accepted the paired-reviewed working tree.
TASK13-only patch SHA256 is `fcb37a94f77336a643adbc388a27f42639350f6ef85fc1f01a2d6bc7aebf8154`;
combined with the separately retained TASK10 dependency, the patch SHA256 is
`f348248e3c48d9cb92eb70a17549c8e2aca4973d559190e47fdfb4759b4e952b` on `33ebdf0d6`.
Independent implementation discovery and strategic Continue found no blocking issue. Production
positives cover V2 applied Start, V3 Start revision 1 and one registered V3 Start continuation at
revision 2, ActiveAttached with the original caller and no interleaved retained ancestry. Ten focused,
72 Start/current, 17 typed-object and 16 E2-RM tests pass; formatting, substrate build and source
inspection pass. Original proof and exact source correspondence remain under `evidence/task13/`.

The known baseline HSA inventory failure remains failed and unrerun; broader workspace, clippy,
platform and installed checks remain unproved. Broader settled/successor/fork/Stop support,
production caller adoption, full read cut and P1/PF completion remain pending. No commit or
integration is selected by this acceptance.

### R09–R10 private source implementation (2026-11-01)

The Linux candidate now supplies native `workspace open`, exact approved-absent
initialization, and distinct `/v1/workspace/preflight` composition. The existing R08
current-actor wire remains separate. Local settings/component loading uses the independent
default-deny `workspace_authority.workspace_load` policy projection. Exact actor read,
initialization and association publication retain separate decisions under the existing
precedence. An initialization offer also binds its actual native carrier and held-home
physical identity; it cannot cross native contexts or attest Root-installed authority.
Global/Workspace settings owner records live
beside their sources, outside HSA; immutable identity/first-association V1 records are
preserved. Per-session selected-World association publication has its own incarnation
and monotonic revision, including explicit active-state changes. Current reads do not
publish, repair, execute, reserve or send.

The source proof target `consumer_admission_preflight_linux` and its designated
`test_real_workspace_open_init_and_existing_world_no_send` launcher are present. Its
protected profile binds genuine owner-published state and the actual native CLI/root
UDS route, separately approved initialization, measured effects and owner invalidators.
Missing profile/state/IPC fails. Private namespace source proof does not discharge
R17 installed/kernel qualification, R15 integration or later full-send fences. Existing
native WorldFork and trailing-retained Stop producer limitations remain explicit.
