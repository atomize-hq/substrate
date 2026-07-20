# Runtime Refactor Control Pack

**Status:** canonical control pack for future runtime-refactor slices
**Scope:** planning, contracts, sequencing, and proof gates; not implementation history
**Source directive:** [`../../substrate-runtime-refactor-directive-revised.md`](../../substrate-runtime-refactor-directive-revised.md)
**Repo-truth snapshot:** 2026-07-17 at `6ab2a515e13946324d0aac25b144e1c3408cb2c1`; re-check live code before every slice
**Current authorized increment:** `A1.1d-5R2-2F — Authenticated world-deps and truthful doctor composition`; R2-2E is implementation-, proof-, and review-complete, while R2-2 remains incomplete.

## Canonical repo location

This pack's canonical location is:

```text
llm-last-mile/runtime-refactor/
```

The `../../...` links to repo-root directives and debug memos assume that placement. If this directory moves, update every affected relative link in the same PR.

## Purpose

This directory compresses the runtime-refactor directive into a selective-load control surface. It exists to prevent a recurring failure mode:

> An artifact in the tree is not evidence that its architecture seam has landed.

A seam is landed only when **all four** conditions are true:

1. the correct authority boundary owns the decision;
2. the real production call path routes through that boundary;
3. the intended policy is enforced at that boundary; and
4. smoke/e2e/regression proof exercises that exact path.

Unit tests, type names, persisted rows, helper functions, process liveness, socket reachability, and successful launch are useful evidence. None is sufficient by itself.

## Control-pack map

| File | Load when | Canonical content |
|---|---|---|
| [`01-target-architecture.md`](01-target-architecture.md) | deciding ownership or reviewing a boundary | target layers, authority map, non-negotiable invariants |
| [`02-seam-crosswalk.md`](02-seam-crosswalk.md) | scoping a slice or assessing current landing status | current artifacts, semantic classification, required action, sibling context, and the canonical A1.1d-5R2 propagation inventory |
| [`03-phase-slice-map.md`](03-phase-slice-map.md) | planning or executing a slice | five tracks, bounded slices, allowed areas, non-goals, exit and regression gates |
| [`04-contracts-and-gates.md`](04-contracts-and-gates.md) | changing schemas, receipts, supervisor behavior, policy, or UAA execution | concrete V1 contracts and acceptance rules |
| [`05-debug-regression-ledger.md`](05-debug-regression-ledger.md) | writing tests, smoke plans, or closeout evidence | resolved baselines, open debug seams, permanent regression gates |

Do not load the full historical design/debug stack by default. Start with the applicable crosswalk row, slice row, contract section, and regression row. Follow only the named must-read links. Slice A0 records its authority-leak inventory inside `02-seam-crosswalk.md`; it must not create an untracked seventh control-pack file.

## Per-slice context assembly protocol

Treat context assembly as part of every implementation and review slice. Do not hand an agent the entire directive, design family, debug history, control pack, and source tree at once.

Assemble three bounded packets:

1. **Authority packet — what must be true:** the exact `03` slice row, affected `02` seam rows, applicable `01` invariants, exact `04` contract sections, and only the design sections named by the slice.
2. **Repo-truth packet — what is true now:** the current production call path, files allowed by the slice, related types and tests, one relevant precedent when available, and fresh call-graph/impact evidence. Separate artifact existence, semantic correctness, real-path adoption, and runtime proof.
3. **Proof packet — how completion is judged:** exact `05` gate rows, targeted tests, negative/fail-closed cases, required smoke/e2e evidence, and the classification change permitted by that evidence.

Target fewer than 2,000 focused lines per implementation task. Historical debug documents are regression provenance, not current implementation authority. Conversation history and prior summaries are discovery hints only until revalidated against the current tree.

Use this capsule at slice start:

```text
SLICE / OBJECTIVE:
TARGET AUTHORITY BOUNDARY:
CURRENT PRODUCTION PATH / SEMANTIC STATUS:
MUST-READ SECTIONS:
LIVE SOURCE / TESTS / PRECEDENT:
SIBLING SEAMS IN CONTEXT:
ALLOWED CODE AREAS / EXPLICIT NON-GOALS:
APPLICABLE CONTRACTS / REGRESSION GATES:
KNOWN CORRECTIONS OR CONFLICTS:
EXIT PROOF / STOP CONDITIONS:
```

If target docs, live code, tests, or fresh runtime evidence conflict, record the conflict in `KNOWN CORRECTIONS OR CONFLICTS` and resolve it before implementation. Never silently select the source that makes the slice appear easiest or most complete.

For B2.1-3, keep three facts separate in every capsule and review: the durable supervisor claim
and cursor, the process-memory world-service producer replay registry, and the shell startup hook
that invokes the canonical supervisor recovery operation. Only `WorldWorkExecutionSupervisor`
interprets durable claims or performs restart discovery and reconciliation. Producer replay only
retains and transports exact B0 frames, and a startup surface only activates the canonical owner.

## Current gateway carrier correction

Keep this split explicit in every D1, D3, or E3 context capsule:

- The managed in-world gateway auth carrier is a landed positive primitive: `world-service` creates a validated `GatewayAuthBundleV1` pipe handoff, launches `substrate-gateway` with `SUBSTRATE_LLM_AUTH_BUNDLE_FD`, scrubs raw secret env vars, and the gateway consumes and validates the bundle once.
- Direct world Codex/member execution still uses the isolated seed-home compatibility bridge. It is not yet consistently pointed at the managed gateway with a per-worker, Substrate-owned `CODEX_HOME`/`config.toml` projection derived from accepted policy and logical config.
- Therefore, do not rebuild or describe the secure-FD carrier as missing. Preserve it under `RG-CONFIG-03`. The unresolved adoption/projection seam is `RG-CONFIG-04`, and the complete world-Codex path remains below `ContractCorrectAndProven` until production-path smoke/e2e closes that gate.

R2-2E did not change that split. The managed gateway secure-FD path is landed, regression-proven,
and unchanged by R2-2E. Direct-member Codex/UAA gateway adoption remains unresolved transitional
compatibility, is non-promotable, and stays owned by E3/D1/D3. `RG-CONFIG-02`, `RG-CONFIG-04`,
`RG-UAA-02`, and `RG-UAA-03` remain open.

## Semantic status labels

These labels describe the **target seam as a whole**, not the quality of individual functions.

| Label | Meaning |
|---|---|
| `ContractCorrectAndProven` | Correct owner, real path, enforcement point, and runtime proof all exist. |
| `UsefulFootholdButWrongBoundary` | Reusable logic/data exists, but ownership or call-path placement is wrong. |
| `DefensiveScaffoldingOnly` | The artifact reduces risk or enables transition, but does not implement the target authority contract. |
| `MislandedWrongModel` | The implementation encodes semantics that conflict with the target model and must be replaced or inverted. |
| `MissingSeam` | No meaningful implementation of the target boundary exists, even if neighboring primitives do. |

Promotion to `ContractCorrectAndProven` requires explicit evidence for all four landing conditions. A component test cannot promote a seam whose production path bypasses it.

## Authority vocabulary

- **Authority:** decides durable meaning and validates state transitions.
- **Host transition intent:** a durable, revision-bound, single-application request for `Start`, `Attach`, or `ResumeOneTurn`; helper plans and episodes transport it but never constitute its claim/application or erase its authority state.
- **Persistence:** stores authority decisions; it does not invent them.
- **Transport:** delivers requests/events; reachability is a signal, not durable truth.
- **Projection:** derives a view or runtime-native artifact from canonical truth.
- **Enforcement:** makes the policy unavoidable on the side-effecting path.
- **Receipt:** durable accepted-work identity returned before terminal completion.
- **Runtime event carrier:** producer-assigned stable stream/frame/event/terminal identity and
  monotonic ordering; it transports fact but owns neither durable observation nor semantics.
- **Producer replay registry:** a bounded, process-memory world-service index that retains exact
  B0 frames for one exact acceptance-record/stream/cursor lookup; it is transport availability,
  not durable supervisor or lifecycle truth.
- **Supervisor:** restart-safe owner of post-acceptance observation and closeout.
- **Supervisor recovery activation hook:** a production startup call that invokes one canonical
  supervisor recovery entry point and retains its observation tasks; it owns no discovery,
  reconciliation, journal interpretation, or terminal decision.
- **Materialization cut:** the ObligationLedger-owned proof that canonical obligation
  materialization covers an exact terminal event identity and sequence for one scoped run.
- **Secret handoff:** one-time secure-FD delivery from host credential authority to the in-world Substrate gateway; never a UAA-native credential file projection.
- **Install bootstrap context:** the one normalized, principal-bound host prefix selected at a public
  install/uninstall entry point and transported without child reinterpretation; in V1 its selected
  prefix, `SUBSTRATE_HOME`, and `SUBSTRATE_ROOT` are identical.
- **Platform bootstrap mapping:** an explicit commitment-preserving realization of that host context
  inside one exact Lima or WSL instance; it does not imply host/guest path or principal equality.
- **Runtime-family adapter:** provider mechanics only; never Substrate lifecycle or policy semantics.
- **Runtime placement versus session binding:** `AgentDescriptorV1.execution_scope` and the matching
  launch knob select where the runtime process executes. `DurableSessionAuthorityV1.world_binding`
  records the durable parent session's exact available world substrate. These are independent
  authority dimensions, not a bijection.

## Reading and update rules

1. Treat this pack as canonical for refactor intent, slice boundaries, contracts, and gates.
2. Treat live code plus fresh runtime evidence as canonical for current artifact truth.
3. If code truth changes, update the affected crosswalk row and regression row in the same implementation PR.
4. Keep a seam below `ContractCorrectAndProven` until its actual production path is proven.
5. Do not use helper/PID/socket liveness as authority in new contracts.
6. Do not create one crate per named seam. A seam may be a module, facade, trait, type, or extracted function set.
7. Keep slice boundaries hard. Adjacent sibling seams stay in context but are not implicit scope.
8. Preserve resolved debug behavior while replacing the model that produced it.

## Current control conclusion

The current tree contains important constraints and footholds, but this pack does not classify any
required seam as `ContractCorrectAndProven`. That is intentional. A1.1e is landed and supplies
necessary exact-read primitives, but the production-ingress audit found that it is not sufficient
by itself for the B1/B2.1 joint closeout: no production path creates a current authority before
A1.2, and the shared prepared dispatch still requires noncanonical compatibility records plus a
legacy live-retained count. The corrected bounded corridor is:

```text
A1.1e -> B0 -> B1-3a/B1-3b receipt core -> B2.1-1/2/3 -------------------------+
       \-> A1.2a current-authority protocol -> A1.2a-WB binding correction       |
           -> A1.2a-S bounded Start adoption                                    |
           -> B1/B2.1-R0 canonical retained target protocol                     |
           -> B3.2a retained creation/admission bridge
           -> B3.2a-WA exact bound-world ownership adoption ---------------------+
                                                                                -> B1/B2.1-0
                                                                                -> joint closeout
                                                                                -> B3.1 -> C1 -> A1.2b
```

This corridor does not close A1.1d, bypass A2/A3 ownership, enable foreground early return, or
promote any seam. B0's runtime-owned identity carrier is landed with its producer clauses proven;
the B1 receipt and B2.1 supervisor cores are recovered and review-clean, and B1/B2.1-0 is
review-clean, but B1 and B2.1 remain below complete until their later joint production integration
closeout. A1.2a is landed and independently review-clean through
`b5f2b4f8dd7d9f650c462cd4626a562cacc1d27f`; it remains limited to production Start after one
strict greenfield-only V1-to-V2 root upgrade, reservation/issuance/application, initial authority
birth, exact retry, and the typed read surface required to resolve that already-current authority.
It has no Attach/Resume, obligation, correlation-supply, or public-consumer adoption.
A1.2a-WB is landed and independently review-clean through `275f9fa2`. It corrects only the Start
write/read matrix so issuance, application/persistence, and exact current-authority resolution all
accept Host runtime placement with either no session world binding or an exact session world
binding, while World placement still requires an exact binding. It changes no schema, canonical
JSON bytes, golden vectors, persisted objects, participant placement, or world capability/policy
semantics. A1.2a-S is landed and independently review-clean through `2f2fecb3`. It adopts only the
ordinary internal greenfield host Start path: a distinct identity-free proposal is applied after
the real dormant-launch adapter has the exact optional world binding and only then becomes the
existing fully materialized `PreparedAgentRuntime`. The authority-managed path performs zero
activated legacy session/participant/snapshot writes, carries the exact bound capability into the
live toolbox context, and leaves startup ownership Pending. It does not change fork/member prepared
runtime construction or adopt helper plans, public Attach/Resume, startup outcome reconciliation,
or any post-turn behavior. B1/B2.1-R0 is now landed and independently review-clean through
`bb3eefba`. B3.2a plus its B3.2a-WA prerequisite are independently review-clean through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda`. The B1 receipt core is recovered through
`6436289fd9dd55ea516b96ef3299e4055d1ea718`; the B2.1 supervisor and replay/startup cores are
recovered through `c519024bd91b6ca6e332d0b8881f7d13ded940e0` and
`de727091a39c884044179a89135df3db5d566778`, with versioned authority-store binding corrected by
`717579b0744154d343985ad439fb8756158f376f`. B1/B2.1-0 is review-clean through
`83101dcbcc750e6e8fb8979bea19f1f777792188`. Its joint production integration closeout has not
begun, B3.1 is not dependency-ready, and no seam is promoted.

The internal **A1.1d-5I installer/bootstrap compatibility audit** is now complete at the
`b29897e0` baseline. It is evidence beneath A1.1d-5, not an A1.1d-6 checkpoint, and it changes no
broader architecture owner or sequence. The audit proves that the current private-home ancestor
ACL rule conflates non-writing traversal with replacement authority, custom-prefix authority is
not propagated across every install/uninstall child, and partial-install cleanup is not
convergent. Remediation remains bounded to **A1.1d-5R1 → A1.1d-5R2 → A1.1d-5R3**. R1 is
implemented and review-clean through `4d0acff68e20d86b97fe5367b8a4617554f33ef4`.
**A1.1d-5R2-0 is planning-complete, and A1.1d-5R2-1 is implementation- and review-complete through
`2653c2ef20ae2e119a444811e6fb46e86d1a6ec6`.** R2 remains sequenced as
**R2-1 → R2-2 Routes A–D → R2-2E → R2-2F → R2-2 integration closeout → R2-3 → R2-4**,
followed by R3. Routes A–D are individually review-clean, but the failed integration closeout proved
that R2-2 is incomplete and unpublished. R2-2E is now implementation-, proof-, and review-complete;
R2-2F is the exact next authorized increment. Renewed R2-2 closeout, R2-3, R2-4, and R3 remain
unstarted. Until the
remaining implementation packets are review-clean and their Linux lifecycle/product smoke passes,
both A1.1d Linux closeout and only the Linux product-smoke portion of the B1/B2.1 joint closeout
remain blocked. B1 receipt and B2.1 supervisor semantics did not regress, and native macOS A1.1d
proof is not added as a dependency of the B1/B2.1 corridor.

Before R2-1 runtime work, the control pack resolves one historical trigger conflict without changing
scope or ownership: parse failures, help, `--version`, and `--version-json` are non-mutating, while
installer-managed and focused private-home regression callers use the hidden
`--install-bootstrap-home-v1` action together with the authenticated
`--install-bootstrap-context-v1 <carrier>`. The carrier argv remains the sole internal-child
discriminator; current-principal and checked H/R validation precede the existing explicit-context
private-home/dependency bootstrap, after which the process returns without entering normal dispatch.
This correction adds no user-facing feature, inventory row, seam promotion, world-deps production
ownership, cleanup authority, or R2-2/R2-3/R2-4/R3 work.

A second preimplementation sequencing correction narrows trace binding without changing the R2-1
file allowlist. `set_global_trace_context` remains the neutral, set-once registration primitive with
its existing signature and behavior. R2-1 may add only an explicit product-bound `TraceContext`
posture carrying `A/trace.jsonl` and policy Git directory A after IH/current-principal validation,
plus an explicit-directory policy hash lookup. Existing physical-shim, replay, platform, test, and
other compatibility callers retain a named `LegacyAmbientCompatibility` posture that is explicitly
not contract-correct and cannot satisfy R2-1 shell proof. PI-117 remains shell/common additive work;
PI-118 and the already-frozen replay/platform migration own removal of that compatibility posture
and the final global unbound-initialization rule in R2-3. No caller-identity table, setter semantic
change, trace lifecycle change, physical-shim edit, replay edit, or new capability is authorized.

The canonical **A1.1d-5R1 Case A** decision corrects only the Linux security contract: V1 requires
**no effective other-principal authority**, not unprovable physical ACL-xattr absence. A descriptor-
bound `ENODATA` result means only that the kernel returned no ACL data; it is accepted under the
authoritative safe mode/owner/type/identity proof and is never called ACL absence. Strictly parsed
ancestor access ACLs may grant masked non-writing `--x` or `r-x`, while any effective write,
ancestor default ACL, observable access/default ACL on the final root, malformed or unsupported
model, or distinguishable read failure fails closed. Exact final-root `0700` and sensitive
descendant `0700`/`0600` contracts remain mandatory. This correction adds no seam, privileged
broker, world capability, policy, gateway, receipt, supervisor, worker, replay, or command change;
the contract correction is recorded by `17ea3a839345cd47a5b2409cde0d4facdde09446` and its bounded
runtime by `4d0acff68e20d86b97fe5367b8a4617554f33ef4`. `RG-HOME-01` and `RG-INSTALL-01` remain open,
A1.1d and A1 remain incomplete, B3.1 remains blocked, and the R2 packets/R3 remain separately
owned; R2-1 is complete and R2-2 is in progress through review-clean Routes A, B, C, and D. The
failed Routes A–D integration closeout is blocker evidence rather than a completed packet; R2-2E is
implementation-, proof-, and review-complete, R2-2F is the only authorized next implementation
increment, and renewed closeout, R2-3/R2-4/R3 remain unstarted. Static macOS/Windows
inspection in R2-0 is not native platform proof.

R2-1 implements the 19 owned rows PI-001–PI-004, PI-010–PI-011, PI-032–PI-034, PI-061–PI-063,
PI-067, PI-082–PI-085, PI-104, and PI-117. Its shared IH framing, hidden authenticated bootstrap
action, Unix principal binding, A-over-B dev/shim/generated projection matrix, and shell-only
explicit trace/policy binding are proven without changing the neutral trace setter, physical shim,
replay, world-deps production, lifecycle behavior, or any later packet owner.

The active planning packet remains **A1.1d-5R2-2 — Unix release, sudo, Linux service, and runtime
propagation**. Routes A–D remain individually review-clean at their final replayed commits
`3bf59b30e4c7348b8ff6315e3eb3658d74af2552`,
`1b5219d5c7471f492865ab55e4efc0d1ab0cac49`,
`0290b829ebaf222fcdc942678178a5e4545de62c`, and
`6452d3a0650df4075c3ef720bad37920a8e5859d`; none is reopened by this correction. Their named
preservation branches retain patch-equivalent reviewed commits, with Route D's replay preservation
pointing at the listed commit exactly. Route B retains
exact ordinary/binary patch SHA-256
`28e6b9da35f96b5f93c49369cbde0eda77e9a145b54f9c412bae8e5a83871674` across its reviewed six
files. Route C remains preserved by branch
`feat/preserve-a1-1d-5r2-2-route-c-final-5b436d5d`; its full-index binary patch remains
`5b436d5dfeaf6e513cbaa306de65848cbe3d5b003fa8c00589be09e54b630277`: 182 insertions and five
deletions across only `crates/shell/src/execution/platform/linux.rs`,
`crates/shell/src/execution/platform/mod.rs`, `crates/shell/tests/doctor_scopes_ds0.rs`,
`crates/transport-api-types/src/lib.rs`, and `crates/world-service/src/handlers.rs`. Those bytes are
a preserved completed increment, not authority to expand scope. Route C projects only
already-authenticated typed context through the existing Host/World doctor paths; its non-secret
optional output fields cannot select, construct, replace, or mutate authority.

The Route D preimplementation source-closure audit found one incomplete allowlist boundary inside
the existing Health/shim-doctor report family. Before the typed dependency collector, the report
may read `world_deps.json`; its embedded world-doctor snapshot may read `world_doctor.json` or launch
the existing `world doctor --json` child. Route D therefore carries the same authenticated typed IH
through both branches: fixture lookup is rooted only at A, and the nested Unix child receives the
canonical hidden argv carrier plus A-derived checked child projections. Ambient B, generated files,
or a contextless repository binary cannot select or supplement A. The mechanical closure is limited
to `gather_world_doctor_snapshot`, `gather_world_deps_section`, `try_load_health_fixture`,
`health_fixture_path`, and `run_json_subcommand` in the already-allowed report file, plus their
existing caller/signature/cfg/lint and focused test closure. It creates no resolver, schema, module
owner, execution family, authority seam, or parent-process environment mutation. The Unix
`collect_report` compatibility collector remains behavior-frozen until R2-3, and physical-shim,
replay, platform-native mapping, installation, cleanup, service, world, policy, capability,
credential, receipt, supervisor, retained-worker, and lifecycle behavior remain unchanged.

The attempted integration closeout after Route D stopped without a runtime or documentation
closeout commit. It proved three remaining authenticated-context gaps: PI-111 world-gateway
config/effective-policy/network-policy/runtime-family projection still selected ambient state;
PI-106/PI-107 normal current/global/workspace/runtime world-deps paths still dropped A; and doctor
composition could label a report with A while configuration, policy, inventory, or dependency
constituents came from ambient B. The source branch therefore remains unpublished for runtime, and
`RG-HOME-01` plus `RG-INSTALL-01` remain open.

Two bounded increments close those remaining seams. **A1.1d-5R2-2E — Authenticated world-gateway
projection** owns PI-111 and has established the single explicit config/policy/network projection
path: existing explicit-bootstrap-home config and effective-policy owners remain unchanged, while
`execution/policy_snapshot.rs` adds only the sole explicit-bootstrap-home policy-snapshot/network-
policy entrypoint. Its implementation/proof clause is complete at
`7e8e83802885c0ece93efcaacccc26503eeb6715`, tree
`02a1a2f6b7e4be47ed9ec38537c805ae348c96b6`, ordinary patch
`fb7b65b02cdac46857750b64ebd5ceab651e8e99910c05240b187c3e729444ff`, and full-index binary
patch `c712f467ac92efb0de7b524272479741ac6b318201cf45dbd994116ec0e8b862`, preserved at
`feat/preserve-a1-1d-5r2-2e-c712f467`. Gateway credentials remain launch-time handoff and network
allow/deny semantics do not change. **A1.1d-5R2-2F — Authenticated world-deps and truthful
doctor composition** owns the unresolved PI-106/PI-107 production paths, one shared typed
world-deps context, authenticated runtime-request construction, and constituent-coherence
validation. Mixed A/B diagnostics fail closed or report unavailable/incoherent, never healthy
A-bound truth. The exact sequence is **Routes A–D → R2-2E → R2-2F → renewed R2-2 integration
closeout → R2-3 → R2-4 → R3**. E and F are logically separable but use this deterministic order so
F can reuse E's canonical projection entrypoint without overlapping ownership. No seam is promoted,
and this docs-only correction begins no runtime increment. The authenticated guarantees
are bounded to the Unix/Linux route in the reviewed allowlists. macOS, Windows, fallback, and other
non-Unix compatibility paths remain R2-3-owned and unproven; they either report unavailable/fail
closed before an A-bound claim or retain explicitly labeled ambient compatibility that cannot
satisfy E/F. Non-Unix cfg proof means build/static preservation only, never authenticated product
proof.

Baseline terms are intentionally distinct. The **R2-2 historical starting baseline** is
`1089 passed / 149 failed`; the **clean Route D comparison baseline** is
`1101 passed / 149 failed / 0 ignored`; and the **current post-E and F comparison baseline** is
`1114 passed / 149 failed / 0 ignored`. R2-2F and its proof wall compare against the third value;
neither it nor the renewed closeout may overwrite the two historical values.

B1/B2.1-R0 lets RetainedWorkerRuntime create the immutable retained object graph and requires
HostSessionAuthority first to reserve the ingress idempotency key, validate the exact participant
identity supplied by its caller, and fix the replay-stable registration/object identities before
object publication, then atomically append exactly its participant to lineage, add its validated
object ref, advance the authority revision, and persist a distinct non-transition registration
proof. It then exact-resolves the canonical target; a still-Pending Start keeps its original
expected revision and A1.2b later accepts only the unique contiguous registration-proof ancestry.
R0 remains a registration protocol and does not claim a production caller, messaging,
accepted-turn observation, park/cancel/stop/fork, or live-count semantics. B3.2a is the separate
RetainedWorkerRuntime-owned production bridge: before R0 it atomically checks the durable
admission count/cap and reserves one exact participant slot and full canonical request fingerprint
under its own crash-stable admission key (never an HSA commitment key) across processes. A durable
per-session registration head alone may then fix the current authority revision. A queued
`SlotReserved` record plus no current head is valid: after the current head reconciles R0 it
releases the head without automatically promoting another record. Only exact re-presentation of
the complete canonical request for the lowest-sequence queued slot may acquire the next head;
later requests cannot overtake it, and an abandoned earliest slot remains conservatively live.
Remaining B3.2 owns exact, restart-safe durable resolution and reconciliation of that abandoned
admission; B4 owns the user/tool-facing exact inspect/cancel verb and distinct outcomes. B3.2a
implements neither protocol.
The admission record stores only the keyed commitment and non-secret fixed fields, never the
request/prompt/payload preimage. The bridge passes that
slot-fixed participant to R0 instead of allocating a retry-local ID, exact-joins R0, commits the
proof before opening the member stream, and carries a transport-neutral typed equality proof through
both the direct dispatcher transport and live internal-toolbox Spawn adapter via the real
transport-api `Service::execute_stream` member branch to the world-service launch boundary. Activated-
store legacy session/participant writes are replaced by exact proof validation. Unknown or
interrupted state stays nonterminal and counted; only exact B0 terminal truth removes it from the
live count. Active caller, posture, workspace, world, policy, spawn steering/outcome, and transport
event behavior remain unchanged. The bounded Linux live Spawn proof then exposed one remaining
physical-realization prerequisite: an authority-managed request could exact-bind HSA and admission
truth to the already-running generic world while world-service `AttachOrCreate` created a different
shared-owner world before launch validation. **B3.2a-WA** therefore runs before B3.2a closeout and
B1/B2.1-0. It gives the runtime-family/world backend one internal, exact, durable adoption operation
for the already-HSA-bound generic world. Adoption preserves the HSA-owned world ID and generation,
changes no HSA or RetainedWorkerRuntime record, exact-joins retry, rejects conflicting ownership,
and completes durable ownership publication before member process creation. It is authorized only
for authority-managed `Some(exact proof)`; compatibility `None` and ordinary world execution keep
their current behavior. The operation adds no world-api field or persisted wire-schema version,
does not persist request/prompt bytes, and does not prove member launch, Registered, routability, or
terminal success. At authorization time no seam was promoted, and B3.2a remained incomplete until
this prerequisite and the full live proof were clean. That prerequisite and proof are now
review-clean through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda`: the exact HSA-bound world was durably adopted with
unchanged ID/generation, the authority-managed member registered through the production toolbox,
and no alternate world or prompt persistence was observed. B1/B2.1-0 now partitions the shared
prepared state, review-clean through `83101dcbcc750e6e8fb8979bea19f1f777792188`,
for RunWorldTask, ordinary retained ContinueWorldWorker, and ephemeral accepted-task
Inspect/Cancel/Wait. Those paths
do not require the missing live-retained lifecycle count. This is the first point at which the
independent B1/B2.1 receipt/supervisor branch joins the authority/retained branch. B1 and B2.1 share one production
integration closeout only after those prerequisites. A1.2b remains after B3.1/C1 and retains all
successor and obligation-dependent post-turn work; it begins by freezing the later strict V3
root/intent/state extension, so A1.2a's V2 Start schema imports no B1-owned accepted-work type.
Retained Inspect/Cancel/Stop remain on unchanged compatibility paths for B3.2/B4 and cannot count
as a joint-closeout failure-to-pass transition.
B2.2/B3.2 retain the deferred receipt-UX and broader retained-lifecycle work after A1 and the named
A2/A3 boundaries. Similarly named event, span, or payload fields are not closure evidence.
