# Runtime Refactor Control Pack

**Status:** canonical control pack for future runtime-refactor slices
**Scope:** planning, contracts, sequencing, and proof gates; not implementation history
**Source directive:** [`../../substrate-runtime-refactor-directive-revised.md`](../../substrate-runtime-refactor-directive-revised.md)
**Repo-truth snapshot:** 2026-07-17 at `6ab2a515e13946324d0aac25b144e1c3408cb2c1`; re-check live code before every slice
**Current authorized increment:** `A1.1d-5R2-2F5-PD — Non-mutating nested doctor boundary`; F1–F4 are complete local runtime increments, the blocked F5 candidate is preserved but rejected, F5-PD is documentation-authorized and unimplemented, F5 remains blocked, and R2-2 remains incomplete.

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
**R2-1 → R2-2 Routes A–E → F0/F0a/F0b/F0-HC complete → R2-2F → renewed
R2-2 integration closeout → R2-3 → R2-4**,
followed by R3. Routes A–D are individually review-clean, but the failed integration closeout proved
that R2-2 is incomplete and unpublished. R2-2E is now implementation-, proof-, and review-complete;
The combined harness corridor is now canonically complete at runtime commit
`770a6a9de9f537f7bc179c75421abbc3fff05b8d`. R2-2F, renewed
R2-2 closeout, R2-3, R2-4, and R3 remain
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
implementation-, proof-, and review-complete. The combined F0/F0a/F0b/F0-HC harness corridor is
implemented, proof-complete, review-clean, committed, and preserved. R2-2F is the exact next packet;
renewed
closeout, R2-3/R2-4/R3 remain unstarted. Static macOS/Windows
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

At the Route D checkpoint, the preimplementation source-closure audit found one incomplete
allowlist boundary inside the existing Health/shim-doctor report family. Before the typed
dependency collector, the report could read `world_deps.json`; its embedded world-doctor snapshot
could read `world_doctor.json` or launch the existing `world doctor --json` child. Route D therefore
carried the same authenticated typed IH through both branches: fixture lookup was rooted only at A,
and the nested Unix child received the canonical hidden argv carrier plus A-derived checked child
projections. The controlling F5-PD correction below removes production World Doctor fixture
selection and changes that child to the hidden passive path; it does not change the separately
F5-owned world-deps fixture. Ambient B, generated files, or a contextless repository binary cannot
select or supplement A. The historical mechanical closure was limited to
`gather_world_doctor_snapshot`, `gather_world_deps_section`, `try_load_health_fixture`,
`health_fixture_path`, and `run_json_subcommand` in the already-allowed report file, plus their
existing caller/signature/cfg/lint and focused test closure. It created no resolver, schema, module
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
A-bound truth. Before F, **A1.1d-5R2-2F0 — Deterministic world-socket test isolation** repairs the
proven test-harness-only `SUBSTRATE_WORLD_SOCKET` collision, while **A1.1d-5R2-2F0a —
SUBSTRATE_HOME test isolation** repairs the separate HOME collision exposed by F0's final broad
wall. At the authorization checkpoint F0/F0a were incomplete and focused proof was not closeout
proof. Their completed implementation uses one process-global authority-environment lock
for same-process HOME/socket mutation, and socket-owning async fixtures must abort, await confirmed
task termination, complete test-owned cleanup, restore exact prior environment state, and only then
release that lock. Neither increment changes any production byte, socket-selection behavior,
retained-registration validation, test name, assertion, capability, or policy.

**A1.1d-5R2-2F0b — deterministic renderer-output test isolation** is the bounded final
test-isolation prerequisite exposed by the F0/F0a broad wall. The stdout fallback test replaces
process fd 1 with `dup2`, so libtest's parallel reporter can write its progress byte into the
test's private pipe. The exact captured bytes were
`".[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n"`. The forced same-process
parallel matrix passed 376 and failed 124 of 500 runs with one normalized signature; isolated,
same-process serial, and separate-process controls each passed 100/100, while the pretty reporter
passed 99/100 parallel runs. Candidate introduction is unnecessary: the helper and target are
byte-identical to clean E. This is `TestIsolationDefectConfirmed`, not a product renderer defect.
The same-owner fd 2 helper has the identical structural race and is included in the bounded
migration even though stdout supplied the observed failure.

F0b authorizes only a private explicit-writer rendering core or equivalent private sink adapter in
`crates/shell/src/execution/agent_runtime/control.rs`. `PublicPromptRenderer::render` remains
the production entry point and delegates through real stdout/stderr with byte-for-byte identical
stream choice, order, newlines, flushing, write-error treatment, redaction, and bounded fallback.
The two exact fallback tests instead provide private in-memory writers and assert the complete exact
bytes, including an empty nonselected stream. Raw fd replacement, a public output API,
process-global writer lock/registry, environment-selected sink, reporter filtering, sleeps,
retries, thread reduction, ignored tests, and assertion weakening are forbidden. No production
caller or product behavior changes, and no seam is promoted.

The exact sequence is **Routes A–E → F0/F0a/F0b/F0-HC complete → F → renewed R2-2
integration closeout → R2-3 → R2-4 → R3**. E and F are logically
separable but use this
deterministic order so
F can reuse E's canonical projection entrypoint without overlapping ownership. No seam is promoted,
and this docs-only correction begins no runtime increment. The authenticated guarantees
are bounded to the Unix/Linux route in the reviewed allowlists. macOS, Windows, fallback, and other
non-Unix compatibility paths remain R2-3-owned and unproven; they either report unavailable/fail
closed before an A-bound claim or retain explicitly labeled ambient compatibility that cannot
satisfy E/F. Non-Unix cfg proof means build/static preservation only, never authenticated product
proof.

Baseline terms are intentionally distinct. The **R2-2 historical starting baseline** is
`1089 passed / 149 failed`; the **clean Route D comparison baseline** is
`1101 passed / 149 failed / 0 ignored`; and the genuine **post-E pre-F0 success observation** is
`1114 passed / 149 failed / 0 ignored`. That post-E result is nondeterministic before F0: the proven
shared-socket interference can instead produce `1113 passed / 150 failed / 0 ignored`. The F0
candidate then produced `1118/150` and `1119/149` while exposing the separate HOME interference.
The post-fork F0/F0a candidate's three exact parallel broad walls were
`1134 passed / 146 failed / 0 ignored`, `1134 passed / 146 failed / 0 ignored`, and
`1133 passed / 147 failed / 0 ignored`; the third wall alone added
`public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails`. That variance
invalidated canonical closeout. Combined F0/F0a/F0b closeout has now established the deterministic
F comparison baseline at `1,280 discovered / 1,235 passed / 45 failed / 0 ignored`, identically
across three parallel walls and one serial wall. F may start only from that recorded baseline.
Neither F nor renewed closeout may overwrite the historical values or erase any isolation record.

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

## A1.1d-5R2-2F0-HC shell-harness closure audit and environment correction

`A1.1d-5R2-2F0-HC` is corrected and authorized as a bounded evidence prerequisite inside canonical
slice A1. It is not a new top-level slice, micro-packet, or product seam. The correction started
from published control-pack commit `9a3b54edcf1560d50b6aafda7293699fce85c719` (tree
`740c19fa4099161f243220c2c25916c76485d7bf`) and audited immutable replayed-E source
`a6c4a5a52c7e33d4efcc18f70dc2b7301ab0fcca` (tree
`17b83d26b3f9a6258d0234461543528c269338bb`). It changes documentation and evidence only.

The corrected shell-library process closure contains 38 resource rows: environment 4, fixed file
descriptors 3, process working state 3, global hooks/subscribers 4, static registries/singletons 11,
filesystem/sockets/ports 4, time/scheduling 3, background lifetime 3, and test-runner coordination
3. Every row has exactly one primary disposition: 7 `UnifiedProcessStateLock`, 4
`ExplicitDependencyInjection`, 1 `TerminationConfirmedTeardown`, 1
`DeterministicSynchronization`, 7 `SubprocessIsolation`, 17 `ProvenConcurrencySafe`, and 1
`SeparatelyOwnedDeferred`. The reconciled row ledger and proof are in
[04-contracts-and-gates.md](04-contracts-and-gates.md#a11d-5r2-2f0-hc-corrected-complete-process-resource-ledger),
and empirical transitions are in
[05-debug-regression-ledger.md](05-debug-regression-ledger.md#a11d-5r2-2f0-hc-empirical-closure-record).

The environment inventory is now exactly 86 names: 74 parent-process mutations and 12 child-only
projections or read-only names. The six names omitted by the prior 80-name ledger are
`XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME`, `SUBSTRATE_OVERRIDE_ANCHOR_MODE`,
`SUBSTRATE_OVERRIDE_ANCHOR_PATH`, and `SUBSTRATE_OVERRIDE_CAGED`. The first three are installed by
`AmbientSelectionGuard::set` in the existing authenticated-gateway negative-authority test; the
last three are arguments to the settings-test `EnvGuard::new`. Complete callsite closure corrects
the dependent total from 518 to 534 parent-mutating tests in the same 35 files. A2 is corrected
from 113 to 129 direct tests: 116 additive plus 13 existing-F0a tests.

The exact test correction is not a blind sixteen-test increment. The old scanner found 506 tests
by literal plus same-file helper traversal, then manually added twelve disjoint dynamic-wrapper
tests to publish 518. Corrected cross-file and duplicate-wrapper closure finds eighteen real
mutators the published union omitted and removes two false positives caused by ambiguous
same-file bare-name fanout, yielding 534. The added set is the named XDG gateway test,
`codex_auth_projection_errors_redact_committed_account_home`, both macOS
`update_world_env_sets_*_flags` tests, and fourteen `with_test_mode`-only PTY tests. The removed
non-mutators are the two `b1_*_acknowledgement_*` tests named in `05`.

Full wrapper closure also corrects the classification of already-listed `SUBSTRATE_SHELL`: the
host-replay test mutates it through shared `set_env`/`restore_env`, and `SpanBuilder::new` is an
overlapping stable reader. That test and file were already in the 116-test/35-file A2 manifest, so
they do not contribute to the correction delta. Its manual restoration can be bypassed by early return and is not exact
for non-Unicode prior values; the future unified-lane migration must make restoration RAII and
exact `OsString`/absence.

Complete dynamic-sink closure also adds the two existing
`execution/platform/macos.rs::platform_tests::update_world_env_sets_*_flags` tests to A2. They
were absent from the defective 518-test total, although their file already belonged to the
35-file universe, and their local `snapshot`/`restore` helper
captures only `String`/absence and restores only `SUBSTRATE_WORLD` and
`SUBSTRATE_WORLD_ENABLED` even though production-frozen `update_world_env` mutates six exact
names. Their future test-only migration must snapshot and restore all six as exact
`OsString`/absence under the existing unified environment coordinator; production
`update_world_env` remains frozen.

The prior audit resolved mutation-wrapper bodies but not every wrapper argument/callsite. Its
manual dynamic-wrapper repair likewise retained the settings tests without extracting their three
override-name arguments and omitted the gateway guard entirely. The correction therefore requires
two independent inventories—direct primitives and wrapper/callsite source closure—and fails if a
wrapper callsite, literal, constant, table entry, parameterized name, dynamic name, parent/child
classification, or mutating test lacks an explicit inventory and migration disposition. The
validator now freezes 43 dynamic mutation-sink owners, 1,005 resolved parent-mutation callsites,
and the exact 534-test manifest; source-parses the projection and local-key tables; checks every
cross-file/duplicate-name wrapper family; and runs negative perturbation checks for a missing
callsite, new table name, or `with_store` callsite.

The audit proves eight additional harness-interference families beyond the already-known HOME,
world-socket, and descriptor-capture families: ambient selector versus stable reader, current
directory versus stable path derivation, global trace-output retargeting, environment-dependent
socket-activation cache persistence, private retry-hook replacement, dispatch-tracker fixture-key
collision, mutable global-broker policy replacement, and active-PTY control replacement. It also
finds a tenth abort-without-awaited-termination test and predictable
private stop-socket paths that survive a failed test process. These findings are consolidated into
one future harness boundary; they do not create F0c/F0d/F0e packets.

At that checkpoint, the canonical sequence was:

`Routes A–E` → `F0/F0a/F0b planned` → `F0-HC corrected and authorized` →
`combined F0/F0a/F0b/Harness implementation and canonical closeout` → `F` →
`renewed R2-2 integration closeout` → `R2-3` → `R2-4` → `R3`.

That checkpoint did not complete F0, F0a, F0b, or their combined closeout. The canonical closeout
below now completes them. F, renewed R2-2 closeout, R2-3, R2-4, and R3 remain unstarted. No user-facing behavior change, production
registry, capability change, policy change, or seam promotion is authorized by F0-HC. The two
preserved F0/F0a candidates remain evidence only; the corrected post-fork-remediation full-index
SHA-256 is `9e012c68ea33107443cd38f2051aa40aae9f0f03d5c6abb4c630d418b0d7dee2`.

## A1.1d-5R2-2F0 historical differential authority correction

`HistoricalParallelArtifactUnavailable` is the bounded evidence result. The historical clean
parallel wall remains authenticated at source commit `06c928443a93579899e5e5827b151f530e1be933`
(tree `dfe15d93366655e9855bca40ad0607887f545637`) and aggregate outcome 1,263 discovered,
1,113 passed, 150 failed, and 0 ignored. Unioning the panic headers and final-summary tail from the
same retained transcript recovers all 150 names, but complete panic output for normalized
signatures survives for only 37. The run therefore lacks a complete authenticated name/signature
artifact. Its aggregate remains diagnostic evidence of prior parallel interference, not
transition-matrix authority. A rerun of the old,
already-proven nondeterministic harness would be a new observation rather than recovery of the
historical artifact.

The corrected proof contract is an evidence-authority correction, not a waiver:

- the complete historical serial artifact is semantic authority;
- three final-candidate parallel walls and one final-candidate serial wall with identical counts,
  failure-name sets, and normalized signatures are concurrency authority; and
- the historical parallel aggregate is diagnostic evidence only.

The authoritative serial transition matrix is exactly `PassToPass=1202`, `PassToFail=0`,
`FailToSameFailure=45`, `FailToChangedFailure=0`, `FailToPass=16`, `Removed=0`,
`RenamedOrSubstituted=0`, `NewPass=17`, `NewFail=0`, and `NewIgnored=0`. Every `FailToPass` remains
subject to causal audit, and all 17 `NewPass` rows are the authorized added tests listed in `02`.
The correction does not permit a historical-parallel failure-name/signature comparison, a
parallel `PassToFail` value, a claim that every current failure belonged to the historical
parallel set, or a claim that exactly 105 named historical parallel failures became passes.

The exact 45-file candidate is preserved remotely at branch
`feat/preserve-a1-1d-5r2-2-harness-final-baseline-blocker-7ab220a7`, commit
`86ed6f5620787121b1c2e5b033ee8d6f9ff369d3`, parent
`1e1be221ca8a9f4e94af93fbda7bda6e94901d01`, and tree
`f6480f3986d43bb41e5387fa1ba5b68ae53f598b`. Its manifest SHA-256 is
`b9e3a44dd671409f66e2d62d48cb494ab147069a74ed71f2308e031f56372ae6`; per-file fingerprint
aggregate is `41cb1a325add4efd8198872b456c3ae0fba73f8c4943e4b59c06e2bc76d8b479`; ordinary patch is
`7ab220a715f4ae3389be314da2e6b0e614fcffb7a92166f04fde999570801861`; full-index patch is
`adc1c5952e2ac4cc97881a1bf4df8e24b00d4d4ba337e692bf21965151d5c0c4`; and its diff is 45 files,
5,630 insertions, and 2,104 deletions. Three implementation reviews—environment closure,
renderer/injection, and lifecycle/subprocess—are `CLEAN`. Fresh reviewer
`/root/final_containment_corrected_authority`, thread
`019f8681-7957-7cc3-88fa-37ab3ad2fc87`, returned `CLEAN` under the corrected authority model.

The correction itself changed no product behavior and started no runtime packet. The subsequently
reviewed harness closeout also changes no product or user-facing behavior and promotes no seam.
F remains unstarted. The binding sequence is now:

`Routes A–E` → `F0/F0a/F0b/F0-HC complete` → `F` →
`renewed R2-2 closeout` → `R2-3` → `R2-4` → `R3`.

## F0/F0a/F0b/F0-HC canonical closeout

F0, F0a, F0b, and F0-HC are complete. The reviewed runtime commit is
`770a6a9de9f537f7bc179c75421abbc3fff05b8d`, tree
`61fdd2e9476f1ce3e041720ce106f7c3427895be`, preserved at
`feat/preserve-a1-1d-5r2-2-harness-review-clean-7ab220a7`. Its 45-file runtime patch is
byte-identical to the preserved candidate: 5,630 insertions and 2,104 deletions; manifest
`b9e3a44dd671409f66e2d62d48cb494ab147069a74ed71f2308e031f56372ae6`; per-file fingerprint
aggregate `41cb1a325add4efd8198872b456c3ae0fba73f8c4943e4b59c06e2bc76d8b479`; ordinary patch
`7ab220a715f4ae3389be314da2e6b0e614fcffb7a92166f04fde999570801861`; and full-index patch
`adc1c5952e2ac4cc97881a1bf4df8e24b00d4d4ba337e692bf21965151d5c0c4`.

The corrected inventory revalidated at 86 names, 74 parent-mutated, 12 child-only/read-only, 395
primitives, 73 dynamic calls, 43 sinks, 1,005 resolved callsites, 534 parent-mutating tests across
35 files, 38 resource rows, and zero unresolved dynamic or ownership rows. All 38 dispositions are
implemented or retained. Focused proof covers HOME/socket overlap, stdout/stderr injection,
XDG/account-home negative projection, environment/CWD restoration, trace/cache/hook/tracker/broker/
ACTIVE_PTY isolation, fork publication, async termination, subprocess isolation, exact absence and
non-Unicode restoration, panic/poison/nesting/reacquisition, and child failure propagation.

Three fresh parallel walls and one fresh serial wall each discovered 1,280 tests, passed 1,235,
failed 45, and ignored 0. Their failure-name set SHA-256 is
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`; their consistently
normalized signature set SHA-256 is
`33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`.
Historical serial semantic authority yields exactly 1,202 `PassToPass`, zero `PassToFail`, 45
`FailToSameFailure`, zero `FailToChangedFailure`, 16 causally audited `FailToPass`, zero removed or
renamed/substituted tests, 17 exact authorized `NewPass`, zero `NewFail`, and zero `NewIgnored`.

Historical parallel evidence remains diagnostic only: `1263/1113/150/0`, all 150 names recovered,
37 complete panic bodies, and 113 normalized signatures unavailable. No historical parallel
transition matrix, 105-transition claim, or reconstructed signature is asserted. GitNexus reports
45 files and 365 changed indexed symbols; its sole medium process attribution resolves to
`AuthorityEnvTestTempDir::new` under `#[cfg(test)]`, not a production flow. The only production hunk
is the authorized mechanical F0b writer delegation with unchanged output/error behavior. The prior
three implementation reviewers and the fresh containment reviewer are `CLEAN`. There is no product
or user-facing behavior change and no seam promotion. F was not started.

At that historical checkpoint the exact next packet was **A1.1d-5R2-2F — Authenticated world-deps
and truthful doctor composition**. The controlling F5-PD correction below supersedes this
next-task statement.

## A1.1d-5R2-2F historical readiness-boundary correction

This section records the historical F3/F4 readiness correction and superseded only the earlier
statements that F was wholly unstarted and the earlier F request-builder/readiness allowlist.
Historical F0/F0a/F0b/F0-HC evidence remains unchanged.

| F checkpoint | Canonical status |
|---|---|
| F1 | Complete and locally committed as `eae02af959f0b7066015bb242ffa45fc7a01d591` (`feat: bind authenticated world-deps context`), tree `7bd7e8b5ca2d563b2991b4fea62ca4ea26266b0c`. |
| F2 | Complete and locally committed as `d30d8cec764e2338fb48475747733091d3af22bf` (`feat: propagate authenticated world-deps scope`), tree `a6161c7664fa6ca1173302dd3570b5d8d7e09ecc`. |
| F3/F4 | Incomplete and blocked. The exact seven-file candidate is preserved at `a343f0796d19d66c168c5bb2797856710cff5708`, tree `a377daa6f41693454e38c39163cc9895bbf3e828`, on `feat/preserve-a1-1d-5r2-2f-f3-f4-blocked-7224dd30`. Its ordinary/full-index patch SHA-256 values are `7224dd30c04e5fcd85913bfdddb62deb497f0b3eaba416b2f689d392f47b9b0c` and `76fb0c5c183880dca8f31576647c8e9372d1ad026e2a731b48d5ef2642ef0a22`. Preservation is evidence, not approval. |
| F5 and final F walls | Unstarted. Renewed R2-2 integration closeout, R2-3, R2-4, and R3 remain blocked and unstarted. |

The blocking review found two coupled boundary defects. First, the candidate copied ambient
`SUBSTRATE_*` and `WORLD_*` values into authenticated world requests, allowing authority selectors,
credentials, tokens, configuration overrides, and conflicting B state to cross beside A. Second,
the candidate connected directly to the fixed Linux socket without invoking the existing
`world_ops.rs::ensure_world_service_ready` lifecycle owner. Duplicating its probe, activation,
stale-socket, spawn, timeout, or error logic is forbidden.

Future F3/F4 work may therefore add one private Linux explicit-target readiness core in
`world_ops.rs`. The existing no-argument owner remains the compatibility entry point and every
existing caller remains behaviorally unchanged. The authenticated F builder supplies the fixed
product socket `/run/substrate.sock` and an immutable installed-product service posture; it never
selects a socket or service binary from the environment, HOME/XDG state, CWD, or a global side
table. Request construction occurs only after authenticated context validation and successful
explicit readiness.

The outbound command environment is an exact generated projection, not inherited state:
`SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR`, `PATH`, `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`,
`XDG_CACHE_HOME`, and `TERM`, each set to the fixed guest-runtime value specified in `04`. No
ambient value is forwarded, including locale values. Structured request fields remain the sole
transport for CWD, profile, policy snapshot, network policy, and filesystem mode; authority,
policy, socket, carrier, credential, prompt, and request preimage values are excluded from the
command environment.

That correction changed no production capability or policy, promoted no seam, and authorized no
runtime implementation in that documentation packet. At that checkpoint F remained incomplete;
the exact next task was to resume **A1.1d-5R2-2F3/F4** under the explicit-readiness and
exact-environment contracts, and F5/final walls could not begin.

## A1.1d-5R2-2F5-PD canonical correction

This latest section supersedes the live-status and next-task wording above while retaining it as
historical readiness-boundary evidence. F3 and F4 are complete local runtime increments at the
pre-documentation-replay commits `54a5ce663cf2724b69b0c124acf2c2758ff622dd` and
`7802c44198625ea6849933140c390900ebe84190`. Their clean wall discovered 1,300 tests, passed 1,255,
failed 45, and ignored 0. The retained failure-name and normalized-signature SHA-256 values are
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70` and
`33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`. These facts complete only
F3/F4; they do not complete F.

The attempted F5 candidate proved 13 decoder tests and 17 of 18 shim-doctor integration tests, then
stopped at the nested-child lifecycle boundary. It is preserved remotely at
`feat/preserve-a1-f5-blocked-candidate-20260722`, commit
`c4b93506480ae5e9708417067980e08e0e754b7a`, parent
`7802c44198625ea6849933140c390900ebe84190`, tree
`03bec2f517866e5213b8a0bed9ad99058c25ebc3`. Preservation is archaeology, not authorization; the
candidate must not be restored wholesale. Its exact four-file manifest and hashes are recorded in
`05-debug-regression-ledger.md`.

The contradiction is now resolved by the bounded prerequisite
**A1.1d-5R2-2F5-PD — Non-mutating nested doctor boundary**. The normal public
`substrate world doctor --json` remains behaviorally unchanged and is not represented as
side-effect-free. Only the authenticated F5 composition child receives a hidden, typed passive
mode. That child validates the canonical hidden argv carrier and current principal before any
projection installation, home scaffold, trace initialization, config/policy selection, platform
resolution, readiness, socket, service, transport, world, or probe action. It then returns a
bounded fail-closed diagnostic. No existing durable runtime-health artifact is authoritative for
this purpose, so the initial passive result is honestly `unavailable`; it never fabricates
coherent success.

Whenever authenticated Linux composition has World enabled and reaches
`gather_world_doctor_snapshot`, it invokes that child and no longer selects
`A/health/world_doctor.json`. The existing World-disabled `build_report` short-circuit remains
unchanged: it returns `disabled_world_doctor_snapshot` without a child or fixture lookup. On the
enabled passive path World Doctor fixtures remain `cfg(test)` evidence only. Existing non-Linux
compatibility stays behavior-frozen,
including its legacy fixture/public-child mechanics; it is forbidden evidence for F5-PD/F5 and
cannot claim A-bound or native proof. The Linux parent
requires duplicate-rejecting raw JSON decode plus exact schema/exit/A identity, rejects mixed or
secret/request-bearing payloads, discards raw child JSON and stderr, and maps only to the existing bounded unavailable/incoherent
`NeedsAttention` representation. The hidden field lives on `WorldAction::Doctor`, leaving the
top-level `Cli` and its `auto_sync.rs` literal unchanged. This additive Rust enum layout does not
change normal public `world doctor --json` CLI behavior.

The binding sequence is now:

`Routes A–E` → `F0/F0a/F0b/F0-HC complete` → `F1/F2 complete` → `F3/F4 complete` →
`F5-PD` → `F5` → `final F walls` → `F closeout` →
`renewed R2-2 integration closeout` → `R2-3` → `R2-4` → `R3`.

F5-PD is a prerequisite beneath F5, not a new slice and not a seam promotion. It changes no world
execution capability, policy, network, filesystem, caging, placement, credential, service, or
lifecycle behavior. Public World doctor compatibility is frozen. Authenticated shim/Health
composition will no longer activate infrastructure merely to collect its nested diagnostic; when
passive evidence is absent, user-visible composition reports unavailable rather than success or
implicit activation. F5, the final F walls, F closeout, renewed R2-2 closeout, R2-3, R2-4, and R3
remain unstarted.

The exact next task is **Implement F5-PD, re-review it, then resume F5**.
