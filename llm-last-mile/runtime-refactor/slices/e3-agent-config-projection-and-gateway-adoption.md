**Kind:** controlling numbered specification and preserved slice row
**Stable ID:** `e3-agent-config-projection-and-gateway-adoption`
**Canonical for:** controlling E3 authority correction, contract/file/symbol fence, dependency and gate disposition, and preserved historical row
**Status:** canonical specification; not admitted, dispatched, or implemented
**Authority scope:** documentation-only E3 authority; no product/test change, admission, dispatch, implementation, or green gate
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 222
**Authority baseline:** commit `138864a26dbc4721366c6cc8934d464d1929a189`, tree `881393e8fc6db7d4422f97cceadfa550d076a14f`, parent `0457e90137da5fb3c3a50f4a531d8828864bd1e3`
**Supersedes:** implementation-shaping E3 statements in the extracted row where corrected below; the row remains historical chronology
**Superseded by:** none
**Projection consumers:** [`track-e-dispatch-policy-and-config-projection.md`](track-e-dispatch-policy-and-config-projection.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md), [`../contracts/agent-config-projection-v1.md`](../contracts/agent-config-projection-v1.md), [`../contracts/managed-gateway-adoption-v1.md`](../contracts/managed-gateway-adoption-v1.md)

# E3 — AgentConfigProjectionService and gateway adoption

> **Authority boundary:** This file is the controlling numbered E3 specification. It consumes the
> completed documentation-only E2-RM authority correction as baseline context without reopening E2,
> E2-RM, or B2.2. It preserves the exact historical Track E row below and grants documentation
> authority only. E3 still requires a later fresh admission and explicit dispatch.

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

## E2 and E2-RM linkage

E3 consumes the exact immutable E2 launch or fork cap ref, created/application revisions, and E1
snapshot identity already carried before member launch. It validates those values through E2's
existing authenticated capability and equality-joins every shared binding. E3 never reconstructs a
cap from current parent policy, changes an E2 record/index/hash, calls the historic receipt-material
projection to invent launch authority, or makes B2.2 receipt work part of E3.

The E2-RM authority correction completed in this baseline remains separately specified and
unadmitted. E2-RM/B2.2 integration and E3 may coordinate shared source windows and differential
baselines, but neither is a prerequisite for the other and no owner semantics move.

## Corrected product/file/symbol fence

The extracted `crates/codex` path is absent at the exact baseline. `codex::CodexHomeLayout` resolves
to external package `unified-agent-api-codex = 0.3.7`; the executable is the separately provisioned,
SHA-256-pinned official OpenAI Codex `0.125.0` Linux archive. A later E3 admission may add a new shared
`crates/config-projection` rather than fabricating `crates/codex`.

Only the exact symbol-level fence in
[`agent-config-projection-v1.md#admission-fence-and-required-proof`](../contracts/agent-config-projection-v1.md#admission-fence-and-required-proof)
is eligible for later admission: the new shared crate; strict inventory V3 projection adapter;
strict V2/version-wrapper transport plumbing through every currently V1-typed world-service helper;
exact Spawn/Fork/toolbox/continue-fork carrier paths; the V2-only local Codex adapter and installed
`substrate-world-entry` two-stage descriptor helper while V1 keeps UAA; descriptor-pinned gateway
start/readiness/access-boundary/ACK symbols; the existing gateway one-time consumer/identity metadata
points; content-addressed root-owned bootstrap/artifact-source installer surfaces; and the exact Codex placement/version
assertion. File-wide changes, a new local Codex helper crate, D1 envelope code, receipts, retained
manifests, E2/E2-RM/B2.2, broker policy, D2, E4 sync, unrelated providers, and non-Linux product work
are outside E3.

## Required later acceptance wall

A fresh admission must require tests using candidate Substrate binaries built from source on Linux
for canonical codecs and every typed failure;
first-writer/CAS and all authority/native-realization crash/recovery/fsync boundaries; zero-live
fencing and exact retry; privilege/capability/Landlock/seccomp/control-path denial; retained
capability revalidation and revocation across resumed turns; sibling
home/gateway isolation; secret canaries across disk, proc, FDs, logs, traces, receipts, and manifests;
descriptor substitution/TOCTOU; deterministic Codex 0.125 reconstruction and all ambient config
layers; strict V3 inventory sources; confined deterministic output paths; install-manifest provenance;
strict V1/V2 mixed versions; exact E2 cap linkage; managed gateway readiness/access/revocation;
and an exact-baseline differential against `138864a26dbc4721366c6cc8934d464d1929a189` with no
candidate-only failure.

Documentation validation is not implementation evidence. No `RG-CONFIG-*`, `RG-UAA-*`,
`RG-OBS-01`, promotion, D1, or D3 gate becomes green here. After a separately admitted implementation
lands independently review-clean, E4 may seek its own admission, D1 may consume the opaque capability
in V3, and D3 remains the final integration owner.

## Explicit nonownership

E3 does not own D1 envelopes; B1/B2.1 observations; B2.2/B3.2 receipts or retained manifests; E1/E2
policy or E2-RM history; gateway provider/upstream policy; D2 command brokerage; E4 workspace
synchronization/reconciliation; host credential authority; or legacy migration. No E3, E4, D1, D3,
E2-RM, B2.2, or successor implementation is admitted or dispatched by this correction.

## Preserved pre-correction row (chronology only)

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **E3 — AgentConfigProjectionService and gateway adoption** | Make logical/effective/native config projection first-class per retained worker and point world Codex at the existing in-world gateway secure-FD credential/session boundary. | `02` config projection + envelope + realization rows; `04` envelope projection, existing-carrier adoption note, and `LaunchTimeSecretHandoffV1`; existing gateway auth-handoff internals/verification docs; config projection, Codex mapping, and workspace overlay designs | Envelope; retained manifest; EffectivePolicyResolver; in-world gateway; RuntimeFamilyRealizationAdapter | `execution/agent_inventory.rs`; `crates/codex`; `world-service/member_runtime.rs`; existing in-world gateway endpoint/session integration points; bounded projection module; projection tests | No reimplementation of the landed FD carrier; no ambient runtime file as authority; no secret payload in projected files, manifests, traces, or UAA child FDs; no workspace-sync policy decision. | Sibling workers get isolated non-secret projections; `CODEX_HOME`, `config.toml`, and provider wiring can be rebuilt from Substrate truth plus accepted policy; Codex uses the exact managed gateway whose credentials arrived through the existing one-time FD carrier; copied auth/config runs only as named/logged/non-promotable compatibility; narrowed hints never substitute for enforcement. | `RG-CONFIG-01`, `RG-CONFIG-02`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-UAA-02`; E3 credential-handoff clause of `RG-OBS-01` |
