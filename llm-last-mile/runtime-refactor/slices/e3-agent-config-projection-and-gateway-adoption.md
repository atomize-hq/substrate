**Kind:** controlling numbered specification and preserved slice row
**Stable ID:** `e3-agent-config-projection-and-gateway-adoption`
**Canonical for:** controlling E3 authority correction, contract/file/symbol fence, dependency and gate disposition, and preserved historical row
**Status:** canonical specification; not admitted, dispatched, or implemented
**Authority scope:** documentation-only E3 authority; no product/test change, admission, dispatch, implementation, or green gate
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 222
**Correction source baseline:** commit `81cfd33d4c5d16c31c837eeddff769995c566570`, tree `d40f6663a2aa01996ce8b4957f4aae0d30830961`, parent `138864a26dbc4721366c6cc8934d464d1929a189`
**Landed prerequisite fact:** `E2-RM` is landed at `2012bb8b5562a73ed0ee45238c19252c16b25065`; that landing permits this documentation correction but is not an E3 runtime dependency
**Supersedes:** implementation-shaping E3 statements in the extracted row where corrected below; the row remains historical chronology
**Superseded by:** none
**Projection consumers:** [`track-e-dispatch-policy-and-config-projection.md`](track-e-dispatch-policy-and-config-projection.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md), [`../contracts/agent-config-projection-v1.md`](../contracts/agent-config-projection-v1.md), [`../contracts/managed-gateway-adoption-v1.md`](../contracts/managed-gateway-adoption-v1.md)

# E3 — AgentConfigProjectionService and gateway adoption

> **Authority boundary:** This file is the controlling numbered E3 specification. It consumes the
> landed E2-RM read behavior as serialization context without reopening E2, E2-RM, or B2.2. It
> preserves the exact historical Track E row below and grants documentation authority only. E3
> remains unadmitted, undispatched, and unimplemented.

## Controlling authority correction

E3 owns one accepted-home config-projection registry, deterministic non-secret Codex 0.125
rendering, adoption of the already-landed gateway secret-FD carrier, and the strict additive V2
member-dispatch carrier. Because baseline inventory V2 has no model/MCP/feature source, E3 adds
strict inventory V3 with explicit `model: codex` and explicit MCP/feature lists; V1/V2 remain
compatibility-only for E3. The controlling V1 effective capabilities/model are exact logical values,
MCP/features are exact empty lists, and the eight-key landed config/explain input includes
`llm.enabled`. Its canonical contracts are:

- [`AgentConfigProjectionRecordV1`](../contracts/agent-config-projection-v1.md), including
  `ConfigProjectionIdentityV1`, `ConfigProjectionRefV1`, canonical hashes, exact reference/equality
  rules, trusted persistence, typed failures, Codex ambient-config closure, and the product fence; and
- [`Managed gateway adoption V1`](../contracts/managed-gateway-adoption-v1.md), including preallocated
  gateway identity, dormant access, one-time handoff join, non-secret ACK/ref, kernel access boundary,
  activation, child barrier, and revocation.

The immutable series subject is permanently bound to accepted store/root, workspace, session,
retained participant, bootstrap run, backend/runtime family, exact world generation, descriptor-
pinned Codex/wrapper/gateway artifacts, and the immutable E2 launch/fork cap. Only projection-owned
config, rendering, gateway-session/handoff identity, and adoption evidence may advance monotonically.
Every series begins with a launch-inhibiting zero-live closed fence. The active record is resolved
from the root-installed bootstrap record's accepted-home authority, never a carrier-selected path,
and is revalidated with the
E2 link, intent, fence, ACK, access boundary, gateway, and artifacts immediately before activation
and two-stage wrapper final-exec release.
Immutable accepted-home native source bytes and the separate rebuildable per-fence `/run`
realization have exact copy/ownership/fsync/readback/recovery rules; mutable Codex state never flows
back into authority. Each gateway or Codex child derives one Landlock layer containing the exact
authenticated E2 plan plus only descriptor-bound E3 support roots and then stacks a role-narrowing
layer, drops to the bootstrap-authenticated UID/GID with no
supplementary groups or capabilities, and proves Landlock/no-new-privileges/seccomp posture. The
wrapper and probe must prove `dumpable=0`; world-service is non-dumpable with zero core limits before
accepting E3 secret bytes, and the credential-bearing gateway reasserts and attests that same posture
after exec and before delivery. Only non-secret Codex may remain `Dumpable=1`, gated by boot-stable
Yama `ptrace_scope=3`, `TracerPid=0`, and same-UID ptrace/process-vm/pidfd-getfd denial before prompt
release.

`MemberDispatchRequestV1` remains strict with byte-identical codec, fields, and post-admission
semantics. E3 adds strict
`MemberDispatchRequestV2` with the non-optional projection activation carrier under an untagged
version wrapper. Because the baseline execute request has no integrated-auth field, the shell first
publishes only immutable nonsecret authoring inputs and sends the existing transient
`GatewayIntegratedAuthPayloadV1` through the strict E3 preparation route. World-service reserves the
listener and deny-all boundary, publishes the independently valid Dormant/no-ACK predecessor, and
returns its carrier without spawning a child. V2 carries no secret; world-service alone joins the
still-live sealed preparation and advances the predecessor through gateway ACK, ReadyClosed, and
Active in the launch request.
D1 later adds strict V3 with its owned `WorldRuntimeAdapterExecutionEnvelopeV1`.
E3 publishes a valid projection before D1 and neither constructs nor partially owns that envelope.
World-service retains the non-cloneable projection capability, active ref, lease, native descriptors,
gateway handle, and descriptor-pinned adapter in `ActiveMemberRuntime` across sequential resumed
turns. The unchanged retained-turn request relies on that state and the exact current E2 turn carrier;
it cannot reconstruct E3 from `binary_path` or fall back to UAA. One combined lifecycle/turn mutex
linearizes resumed-turn reservation and final release against terminal revocation.

Because baseline non-E3 children can inherit powerful ambient capabilities, a process-wide child-
exclusion lease is mandatory during every credential-bearing E3 epoch. Entry requires zero live
non-E3 or unclassified descendants; all ordinary execute/PTY, V1/UAA, and compatibility-gateway
or GC-helper spawns fail before child side effects until every E3 sibling in the exact world generation is revoked
and reaped. V1 remains strict and behavior-identical whenever that exclusion is idle; it is never
silently upgraded or allowed to coexist with E3 secrets.

The gateway's `X-Substrate-*` request headers are identity/audit metadata, not authorization.
Credential-requiring Linux E3 uses an exact participant-process cgroup plus nftables rule to keep the
loopback listener deny-all while dormant, allow only the bound member tree after final validation,
and revoke before teardown. E3 adopts exactly one inherited listener, disables the auxiliary OAuth
bind, and exposes only nonce-gated `GET /health` plus identity-gated `POST /v1/responses`. Readiness
is probed by the descriptor-pinned, zero-capability wrapper child inside the exact probe cgroup, never
by a world-service thread. The existing
gateway alone receives and consumes the fresh one-time FD after the pinned wrapper proves privilege
descent and that same unprivileged PID descriptor-execs the gateway;
no secret is persisted, inherited through the member environment, placed in argv, or leaked to
Codex/UAA descendants. Copied host auth/config is separately policy-granted, named/logged,
non-promotable compatibility only.

## Projection namespace and HSA serialization

The E3 registry remains exactly
`<accepted-home>/authority-v1/agent-config-projection-v1/`. It is an intentional new member of the
existing closed `authority-v1` namespace, not a new authority hierarchy. The later E3-B storage
owner is `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/layout.rs`,
specifically `StoreLayout::validate_closed_layout`: it may add only the literal top-level directory
`agent-config-projection-v1`. It must not weaken the closed-layout check, accept arbitrary unknown
entries, or introduce a wildcard. Landed E2-RM commit
`2012bb8b5562a73ed0ee45238c19252c16b25065` repeats the same closed manifest in
`crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/transaction.rs`,
specifically `validate_e2_rm_authority_manifest`, and otherwise returns `UnsafeNamespaceEntry`. A
fresh E3-B admission must bind that second validator for the identical sole literal recognition, as
well as the primary layout owner and every other required product path. Neither product path is
changed or admitted here. The second literal is HSA layout compatibility only; it grants no ownership
of E2-RM authority data, read semantics, history, reconciliation, schema, or namespace.

Every E3 read, publication, CAS, recovery, retirement, and GC transaction must use one
cross-process order:

1. Open and validate the accepted HSA authority root through the existing trusted-root capability.
2. Acquire the existing exclusive `authority-v1/lock/root.lock`.
3. Revalidate the HSA root identity and exact closed layout while holding the parent lock.
4. Only then open and exclusively acquire
   `authority-v1/agent-config-projection-v1/lock`.
5. Perform the complete E3 transaction while retaining both locks and the trusted-root capability.
6. Revalidate the HSA root, closed layout, namespace manifests, and affected E3 objects; release the
   child lock and then the parent lock.

Parent root lock before E3 child lock is mandatory. Reverse acquisition, child-only mutation, lock
upgrading, or releasing the parent during the transaction is forbidden. E2-RM reads and ordinary HSA
operations that acquire the existing parent root lock are thereby serialized against E3 publication
and recovery. E3 does not modify E2, E2-RM, B1, B2.1, retained-worker, or any other namespace
authority, and no reconciliation may mutate another owner's namespace. Process death releases
kernel locks without making incomplete E3 state authoritative; the next E3 transaction must recover
its own namespace or fail closed while holding both locks. Crash-boundary tests and concurrent
HSA/E2-RM/E3 tests, including a successful landed E2-RM read after E3 namespace creation plus
deadlock, unknown-entry, and cross-owner-mutation negatives, are mandatory.

## Selected-only inventory provenance

E3 V1 authenticates and persists provenance only for the selected contributing inventory source. A
valid workspace descriptor shadows and replaces the same-ID global descriptor before projection
construction; the global descriptor is then not a contributor. Shadowed bytes enter no projection
record, hash preimage, revision, source identity, or retention set, and E3 V1 promises no durable
shadow-history reconstruction. Discovery must still fail closed on malformed or ambiguous global or
workspace material before selection; nonretention after a valid shadow selection is not permission
to ignore invalid source input. Any future durable shadow-history feature requires separate
versioned authority and cannot be inferred into V1. No shadow-provenance schema or new hash field is
owned here.

## UAA and the local V2 adapter

External `unified-agent-api-codex` 0.3.7 remains unchanged. It is not claimed to provide
descriptor-pinned execution or confined output-path construction. Strict E3 V2 uses the already
specified local `E3Codex0125LaunchAdapterV1`; V1 compatibility continues through its existing UAA
path unchanged. E3 must not create or patch a local `crates/codex` replacement. This is a
source-grounded clarification of existing ownership, not a new product primitive.

## Serial bounded E3 work packets

The following labels decompose E3 internally; they are headings, not new stable IDs, separate slice
documents, implementation admissions, or automatic dispatches. Every packet requires its own later
fresh admission and explicit dispatch. A packet's success makes only its direct successor eligible
to seek admission and does not dispatch that successor. One combined E3-A-through-E3-F candidate is
forbidden.

### E3-A — strict V2 wire carrier and V1 compatibility

- **Bounded behavior:** add the strict additive V2 member-dispatch carrier and version wrapper, thread
  it through the exact shell/client/world-service transport reachability path, and preserve strict,
  byte-identical V1 behavior.
- **Source owner:** `crates/transport-api-types/src/lib.rs` plus only the exact client, shell routing,
  orchestration, Lima V1-wrapper, and world-service decode/validation call sites cataloged by the
  projection contract.
- **Direct predecessor:** pushed and live-verified E3-AC1 authority correction.
- **Consumes:** the existing E2 launch/fork carrier and the corrected E3 ownership/compatibility
  boundary; it consumes no projection implementation.
- **Explicit nonownership:** projection persistence or authoring, inventory V3, local Codex process
  launch, Linux child security, managed-gateway lifecycle, retained-runtime adoption, D1, and E4.
- **Successor eligibility proof:** strict V2 canonical/unknown/newer/malformed negatives, complete
  Spawn/Fork/toolbox/continue-fork reachability, old/new mixed-version behavior, and a byte-identical
  V1 differential with no V2 construction on compatibility paths. Only then may E3-B seek admission.

### E3-B — projection codec, registry, CAS, recovery, and retirement

- **Bounded behavior:** implement canonical projection codec/reference validation and the accepted-
  home registry's first-writer publication, CAS, recovery, leases, retirement, and GC under the exact
  parent-before-child lock protocol.
- **Source owner:** the packet-admitted codec/registry surfaces in `crates/config-projection`, the
  existing trusted-root facade needed to open the accepted HSA authority, and only
  `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/layout.rs`,
  specifically `StoreLayout::validate_closed_layout`, plus only
  `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/transaction.rs`,
  specifically `validate_e2_rm_authority_manifest`, for the same single literal directory
  recognition.
- **Direct predecessor:** independently landed and proof-clean E3-A.
- **Consumes:** E3-A wire reference types and the existing HSA trusted-root/root-lock capability.
- **Explicit nonownership:** inventory selection, source/artifact authoring, Codex rendering or
  launching, Linux child security, gateway preparation/adoption, retained-runtime launch/resume,
  and every non-E3 namespace's reconciliation.
- **Successor eligibility proof:** canonical codec/hash negatives; two-process first-writer/CAS;
  every fsync/rename/readback/recovery/retirement crash boundary; exact parent/child release order;
  concurrent ordinary HSA/E2-RM/E3 access without deadlock; successful landed E2-RM reads after E3
  namespace creation; closed-layout rejection of every other unknown entry; and proof that recovery
  mutates only the E3 namespace. Only then may E3-C seek admission.

### E3-C — authenticated authoring, V3 inventory, artifacts, and Codex rendering

- **Bounded behavior:** construct descriptor-pinned effective-config and selected-only inventory
  sources, strict inventory V3, authenticated artifact manifests/native sources, and deterministic
  non-secret Codex 0.125 rendering.
- **Source owner:** the packet-admitted `crates/config-projection` service/Codex/artifact surfaces;
  shell `config_model`, `agent_inventory`, and world-dependency installer surfaces; the one Codex
  placement descriptor; and exact Linux lifecycle/provision publication helpers cataloged by the
  contract.
- **Direct predecessor:** independently landed and proof-clean E3-B.
- **Consumes:** E3-B registry/codec capability and the pre-existing E2 immutable launch/fork link.
- **Explicit nonownership:** shadow-history authority, policy recomposition, child privilege descent,
  process-wide exclusion, gateway activation or secrets, retained launch/resume, D1, and E4.
- **Successor eligibility proof:** strict V3 parse/discovery negatives before selection; deterministic
  selected-only provenance with no shadowed bytes retained or hashed; descriptor/source substitution
  negatives; artifact/source-manifest and native-source crash proof; and byte-identical Codex 0.125
  reconstruction with ambient-layer closure. Only then may E3-D seek admission.

### E3-D — Linux child security, capability parking, and process-wide exclusion

- **Bounded behavior:** install the synchronous internal-exec-first capability parking sequence,
  descriptor-pinned wrapper security, cgroup/nftables/Landlock/seccomp/credential descent, and one
  process-wide exclusion shared by every child/helper path.
- **Source owner:** the packet-admitted world-service `main`, child-security, internal-exec, wrapper,
  service, PTY, observation-handler, GC, and exclusion surfaces cataloged by the contract.
- **Direct predecessor:** independently landed and proof-clean E3-C.
- **Consumes:** E3-C pinned artifacts, native roots, and enforcement inputs, plus the landed E2
  filesystem plan without changing E2.
- **Explicit nonownership:** inventory or projection schema changes, gateway secret preparation or
  adoption, retained V2 Codex lifecycle, provider policy, V1/UAA semantics while exclusion is idle,
  D1, and E4.
- **Successor eligibility proof:** sole-thread transition-capability parking/readback; exact child
  UID/GID/zero-capability, Landlock, seccomp, dumpability, Yama, and control-path denial evidence;
  cgroup/nftables crash recovery; and races proving no ordinary/PTY/UAA/compatibility-gateway/GC
  helper can spawn during E3 exclusivity while idle compatibility behavior is unchanged. Only then
  may E3-E seek admission.

### E3-E — dormant managed-gateway preparation and adoption

- **Bounded behavior:** accept authenticated local preparation, reserve one listener and deny-all
  boundary, publish independently valid Dormant/no-ACK authority without spawning a child, then
  descriptor-start, attest, deliver the one-time secret, probe readiness, adopt, allow, and revoke
  the exact managed gateway.
- **Source owner:** the packet-admitted shell integrated-auth extraction, world-service preparation,
  authenticated local transport and gateway-runtime surfaces, and existing gateway launch/server
  adoption points cataloged by the projection and managed-gateway contracts.
- **Direct predecessor:** independently landed and proof-clean E3-D.
- **Consumes:** E3-B durable authority, E3-C pinned gateway artifacts/rendered inputs, E3-D child
  security and exclusion, and the already-landed secure-FD consumer.
- **Explicit nonownership:** a new secret carrier, UAA changes, provider/upstream policy, retained V2
  Codex launch/resume, D1 envelopes, compatibility-gateway promotion, and E4.
- **Successor eligibility proof:** authenticated-UDS/peer negatives before body read; Dormant/no-ACK
  publication and retry/restart/crash proof; exact listener/boundary/readiness cgroup proof; secret
  canary absence; gateway privilege and zero-core attestation; one-time FD consumption; and complete
  revocation with V1 compatibility unchanged. Only then may E3-F seek admission.

### E3-F — retained V2 Codex launch/resume adoption and integrated proof

- **Bounded behavior:** join the sealed preparation to the V2 carrier, advance the exact projection
  lineage through ReadyClosed/Active, retain the non-cloneable local Codex adapter and gateway
  capability, and revalidate/release/revoke them across initial and resumed turns.
- **Source owner:** the packet-admitted world-service member-runtime and prompt-fulfillment surfaces,
  the exact shell retained-runtime startup call sites that consume E3-A's carrier and E3-C's
  authoring outputs without changing the carrier codec, and the two bounded E3 integration test
  surfaces cataloged by the contract.
- **Direct predecessor:** independently landed and proof-clean E3-E.
- **Consumes:** the complete proof-clean outputs of E3-A through E3-E and the current E2 turn carrier.
- **Explicit nonownership:** D1/D2/D3 envelopes or brokerage/integration ownership, E4 workspace
  synchronization, receipts, retained manifests, compatibility promotion, non-Linux E3, and any
  mutation of E2, E2-RM, B1, B2.1, or retained-worker authority.
- **Completion proof:** initial plus at least two sequential resumed turns; deterministic confined
  output paths and descriptor execution; lifecycle/turn linearization; crash/retry/revocation and
  sibling isolation; complete security/secret/inventory/registry/gateway integrated negatives; and an
  exact baseline differential. This completes only E3's bounded implementation proof and does not
  admit any successor.

## E2 and E2-RM linkage

E3 consumes the exact immutable E2 launch or fork cap ref, created/application revisions, and E1
snapshot identity already carried before member launch. It validates those values through E2's
existing authenticated capability and equality-joins every shared binding. E3 never reconstructs a
cap from current parent policy, changes an E2 record/index/hash, calls the historic receipt-material
projection to invent launch authority, or makes B2.2 receipt work part of E3.

`E2-RM` is already landed at `2012bb8b5562a73ed0ee45238c19252c16b25065`. That landing satisfies
the prerequisite for landing this E3-AC1 documentation correction because the corrected namespace
and parent-lock protocol must serialize with landed E2-RM reads; it does not make E2-RM an E3
runtime dependency. E3 owns no E2-RM write, history, reconciliation, schema, or behavior change.

## Corrected product/file/symbol fence

The extracted `crates/codex` path is absent at the exact baseline. `codex::CodexHomeLayout` resolves
to external package `unified-agent-api-codex = 0.3.7`; the executable is the separately provisioned,
SHA-256-pinned official OpenAI Codex `0.125.0` Linux archive. A later E3-B or E3-C packet-specific
admission may add its bounded portion of a new shared `crates/config-projection` rather than
fabricating `crates/codex`.

Only packet-applicable portions of the ownership catalog in
[`agent-config-projection-v1.md#admission-fence-and-required-proof`](../contracts/agent-config-projection-v1.md#admission-fence-and-required-proof)
may be eligible for a later packet-specific admission: the single literal E3-B closed-layout addition
in `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/layout.rs`,
specifically `StoreLayout::validate_closed_layout`; the new shared crate; strict inventory V3
projection adapter; the identical single literal recognition in
`crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/transaction.rs`,
specifically `validate_e2_rm_authority_manifest`, without any E2-RM authority-data or semantic
ownership;
strict V2/version-wrapper transport plumbing through every currently V1-typed world-service helper;
exact Spawn/Fork/toolbox/continue-fork carrier paths; the V2-only local Codex adapter and installed
`substrate-world-entry` two-stage descriptor helper while V1 keeps UAA; descriptor-pinned gateway
start/readiness/access-boundary/ACK symbols; the existing gateway one-time consumer/identity metadata
points; content-addressed root-owned bootstrap/artifact-source installer surfaces; and the exact Codex placement/version
assertion. File-wide changes, a new local Codex helper crate, D1 envelope code, receipts, retained
manifests, E2/E2-RM/B2.2, broker policy, D2, E4 sync, unrelated providers, and non-Linux product work
are outside E3. The catalog does not authorize one combined E3-A-through-E3-F implementation
candidate; every additional product path required by a packet must be bound by that packet's fresh
admission or the work stops.

## Required later acceptance wall

No current-host property is presumed to satisfy an E3 packet. Each packet's fresh admission must bind
its exact outcome and nonownership, source and toolchain identities, actual proof commands, host
facilities required by those commands, and realistic resource needs. The retained evidence must make
the bounded proof reproducible without treating unfinished successor-owned behavior as a pre-edit
prerequisite. E3-A therefore requires Linux `x86_64`, the repository-required Rust toolchain (at this
baseline, channel `1.89.0` and MSRV `1.89`), and the other environment needed for its strict wire-
transport and V1-compatibility tests; musl tooling, privileged facilities, gateway or Codex execution,
and full E3 security attestation are not prerequisites solely because later packets need them. When
an actual retained test requires one of those facilities, it must be available before the dependent
operation, and a false green, silent skip, or invalid invocation never counts as product proof.
Missing or drifting required facilities are environmental stops for the dependent work.

Candidate-implemented security behavior is established by its owning packet's proof, not presumed
before its code exists. The required existing `CAP_SYS_ADMIN`, `CAP_NET_ADMIN`, `CAP_DAC_OVERRIDE`,
and `CAP_SYS_PTRACE` service-authority posture and all capability-parking and readback, privilege-
descent, boot-stable Yama, namespace/cgroup/nftables/Landlock/seccomp confinement, credential,
activation/revocation, and control-path denial predicates remain mandatory before the operations they
protect and before the owning packet may be claimed complete. Adding `CAP_SETUID`, `CAP_SETGID`, or
`CAP_SETPCAP` to an otherwise unmodified service cannot substitute for implementing and proving
E3-D's synchronous transition-capability parking, and no security fallback or substitute privilege
model is authorized.

Admission requires reproducible commands and sufficient persistent evidence, not a bespoke wrapper
or six fresh directories for every packet. Existing valid build targets, roots, and evidence may be
reused when causal inputs are unchanged. Filesystem use must not collide with repository or accepted
authority state; must use short paths for Unix sockets; must preserve proof-required device
relationships; and must place neither heavy builds nor durable state on tmpfs. Separation and
isolation are required where the proof needs them, including matched baseline/candidate invocations
that could otherwise share causal state, but incidental harness structure is not a universal gate.
Applicable differential proof begins with the minimum valid matched commands that can reach the
changed behavior, compares canonical semantic outcomes rather than scheduling or emission order,
permits unchanged valid evidence to be reused, and still rejects every candidate-only semantic
failure. The required comparison baseline remains
`2b2fc6c50b40046dbeaeb5b316562fbd96480a2a`.

Once a packet's applicable prerequisites are available, its fresh admission must require its bounded
tests using candidate Substrate binaries built from source on Linux. Across the serial sequence,
proof must
cover canonical codecs and every typed failure;
first-writer/CAS and all authority/native-realization crash/recovery/fsync boundaries; zero-live
fencing and exact retry; privilege/capability/Landlock/seccomp/control-path denial; retained
capability revalidation and revocation across resumed turns; sibling
home/gateway isolation; secret canaries across disk, proc, FDs, logs, traces, receipts, and manifests;
descriptor substitution/TOCTOU; deterministic Codex 0.125 reconstruction and all ambient config
layers; strict V3 inventory sources; confined deterministic output paths; install-manifest provenance;
strict V1/V2 mixed versions; exact E2 cap linkage; managed gateway readiness/access/revocation;
and an exact-baseline differential against the pushed and live-verified E3-AC1 landing commit with no
candidate-only failure.

Documentation validation is not implementation evidence. No `RG-CONFIG-*`, `RG-UAA-*`,
`RG-OBS-01`, promotion, or other E3 gate becomes green here. This correction grants no admission,
dispatch, implementation, security exception, or successor eligibility by implication.

## Explicit nonownership

E3 does not own D1 envelopes; B1/B2.1 observations; B2.2/B3.2 receipts or retained manifests; E1/E2
policy or E2-RM history; gateway provider/upstream policy; D2 command brokerage; E4 workspace
synchronization/reconciliation; host credential authority; or legacy migration. `E2-RM` is already
landed at `2012bb8b5562a73ed0ee45238c19252c16b25065` but is not an E3 runtime dependency and is not
modified here. E3 remains unadmitted, undispatched, and unimplemented. No D1, D2, D3, E4,
compatibility-promotion, non-Linux, B2.2, or successor work is admitted or dispatched; B4 remains
separate and is neither modified nor adjudicated. No E3 gate becomes green. A fresh E3-A admission
may be considered only after this correction is pushed and its live origin identity is verified.

## Preserved pre-correction row (chronology only)

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **E3 — AgentConfigProjectionService and gateway adoption** | Make logical/effective/native config projection first-class per retained worker and point world Codex at the existing in-world gateway secure-FD credential/session boundary. | `02` config projection + envelope + realization rows; `04` envelope projection, existing-carrier adoption note, and `LaunchTimeSecretHandoffV1`; existing gateway auth-handoff internals/verification docs; config projection, Codex mapping, and workspace overlay designs | Envelope; retained manifest; EffectivePolicyResolver; in-world gateway; RuntimeFamilyRealizationAdapter | `execution/agent_inventory.rs`; `crates/codex`; `world-service/member_runtime.rs`; existing in-world gateway endpoint/session integration points; bounded projection module; projection tests | No reimplementation of the landed FD carrier; no ambient runtime file as authority; no secret payload in projected files, manifests, traces, or UAA child FDs; no workspace-sync policy decision. | Sibling workers get isolated non-secret projections; `CODEX_HOME`, `config.toml`, and provider wiring can be rebuilt from Substrate truth plus accepted policy; Codex uses the exact managed gateway whose credentials arrived through the existing one-time FD carrier; copied auth/config runs only as named/logged/non-promotable compatibility; narrowed hints never substitute for enforcement. | `RG-CONFIG-01`, `RG-CONFIG-02`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-UAA-02`; E3 credential-handoff clause of `RG-OBS-01` |
