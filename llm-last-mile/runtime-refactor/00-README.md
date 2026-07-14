# Runtime Refactor Control Pack

**Status:** canonical control pack for future runtime-refactor slices
**Scope:** planning, contracts, sequencing, and proof gates; not implementation history
**Source directive:** [`../../substrate-runtime-refactor-directive-revised.md`](../../substrate-runtime-refactor-directive-revised.md)
**Repo-truth snapshot:** 2026-07-13; re-check live code before every slice

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
| [`02-seam-crosswalk.md`](02-seam-crosswalk.md) | scoping a slice or assessing current landing status | current artifacts, semantic classification, required action, sibling context |
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

## Current gateway carrier correction

Keep this split explicit in every D1, D3, or E3 context capsule:

- The managed in-world gateway auth carrier is a landed positive primitive: `world-service` creates a validated `GatewayAuthBundleV1` pipe handoff, launches `substrate-gateway` with `SUBSTRATE_LLM_AUTH_BUNDLE_FD`, scrubs raw secret env vars, and the gateway consumes and validates the bundle once.
- Direct world Codex/member execution still uses the isolated seed-home compatibility bridge. It is not yet consistently pointed at the managed gateway with a per-worker, Substrate-owned `CODEX_HOME`/`config.toml` projection derived from accepted policy and logical config.
- Therefore, do not rebuild or describe the secure-FD carrier as missing. Preserve it under `RG-CONFIG-03`. The unresolved adoption/projection seam is `RG-CONFIG-04`, and the complete world-Codex path remains below `ContractCorrectAndProven` until production-path smoke/e2e closes that gate.

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
- **Supervisor:** restart-safe owner of post-acceptance observation and closeout.
- **Materialization cut:** the ObligationLedger-owned proof that canonical obligation
  materialization covers an exact terminal event identity and sequence for one scoped run.
- **Secret handoff:** one-time secure-FD delivery from host credential authority to the in-world Substrate gateway; never a UAA-native credential file projection.
- **Runtime-family adapter:** provider mechanics only; never Substrate lifecycle or policy semantics.

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
       \-> A1.2a current-authority protocol -> A1.2a-S bounded Start adoption   |
           -> B1/B2.1-R0 canonical retained target protocol                     |
           -> B3.2a retained creation/admission bridge --------------------------+
                                                                                -> B1/B2.1-0
                                                                                -> joint closeout
                                                                                -> B3.1 -> C1 -> A1.2b
```

This corridor does not close A1.1d, bypass A2/A3 ownership, enable foreground early return, or
promote any seam. B0's runtime-owned identity carrier is landed with its producer clauses proven;
the B1 receipt and B2.1 supervisor cores are preserved at the current joint-closeout stop. B1
cannot close until the accepted production paths enter the durable supervisor without attempting
the legacy active-task writer. After this corrected sequence is independently review-clean, the
exact next implementation packet is A1.2a. It is limited to production Start
after one strict greenfield-only V1-to-V2 root upgrade; then reservation/issuance/application, initial
authority birth, exact retry, and the typed read surface required to resolve that already-current
authority; it has no Attach/Resume, obligation, correlation-supply, or public-consumer adoption.
A1.2a-S then adopts only the ordinary internal greenfield host Start path: a distinct identity-free
proposal is applied after the real dormant-launch adapter has the optional world binding and only
then becomes the existing fully materialized `PreparedAgentRuntime`. It replaces that path's legacy
session/participant writes with the applied A1.2a result, carries the bound capability into the
live toolbox context, and leaves startup ownership Pending. It does not change fork/member prepared
runtime construction or adopt helper plans,
public Attach/Resume, startup outcome reconciliation, or any post-turn behavior.
B1/B2.1-R0 then lets RetainedWorkerRuntime create the immutable retained object graph and requires
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
per-session registration head alone may then fix the current authority
revision; later slots remain ordered without pinning a stale revision. The bridge passes that
slot-fixed participant to R0 instead of allocating a retry-local ID, exact-joins R0, commits the
proof before opening the member stream, and carries a transport-neutral typed equality proof through
both the direct dispatcher transport and live internal-toolbox Spawn adapter via the real
transport-api `Service::execute_stream` member branch to the world-service launch boundary. Activated-
store legacy session/participant writes are replaced by exact proof validation. Unknown or
interrupted state stays nonterminal and counted; only exact B0 terminal truth removes it from the
live count. Active caller, posture, workspace, world, policy, spawn steering/outcome, and transport
event behavior remain unchanged. B1/B2.1-0 then partitions the shared prepared state
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
