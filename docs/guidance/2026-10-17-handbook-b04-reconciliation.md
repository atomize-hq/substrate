# Handbook → Substrate → Collider: current-state and B04 reconciliation

**Date:** 2026-10-17; updated 2026-10-19 (America/Indiana/Indianapolis).

**Status:** Investigation plus the human-selected SDK-first prototype direction (§8, §11). Source-qualified SDK/owner change recommendation is in §13; §14 reconciles the complete Collider-to-owner workflow and separates integration glue from substantive owner changes. §15 corrects Collider readiness (application shell versus retained isolated proof), reconciles cumulative assertions and names the smallest missing delta under the existing staged proof. Implementation and integrated proof remain open. This is not owner-contract approval or implementation authorization.

**Purpose:** Reconstruct Handbook's accepted direction and newer Windows landings, then map them against the existing [B04 handoff](../contracts/draft/collider-b01-b05-handoff.md#6-b04--typed-selection-semantic-resolution-and-delivered-context). This is one review document, not another control plane or status ledger.

## 1. Executive finding

The September Mac checkout is **not the current Handbook implementation baseline**. Recorded Windows commands identify `feat/handbook-contract-membrane-clean`, with substantial later native SDK, governance, Contracts, and work-registration work. Some important foundations already landed August 29 but were absent from the Mac checkout used in September; they are not all newly built in the last three weeks. Separately, file-native document/graph experiments explore reusable infrastructure; their presence on the same branch does not adopt them into production owners.

The three-owner direction remains coherent:

1. **Handbook:** semantic applicability, Scope/Resolution, exact source identity, domain checks, and context materialization.
2. **Substrate:** execution authorization, destination visibility, execution-specific packaging and actual delivery, accepted-work identity, and execution evidence.
3. **Collider:** selection, interaction, and presentation of owner results—not a substitute resolver or execution authority.

However, B04's September evidence pin and operation inventory are historical. A new mapping must consider Handbook's later supported SDK foundations, not design a generic resolver merely because the old `generate_packet(packet_id)` route was insufficient.

**October 18 source follow-up:** the snapshot-only candidate cannot carry arbitrary selected document bodies. Flow remains the existing materialization candidate, but is **not yet qualified for B04**. The later Git refresh (§12.9) removes the partial-session-source access blocker: the published Windows baseline is now checked out locally, and its concrete ordinary Flow request, admission, materialization and result path have been inspected. That route supplies content and structured outcomes, but not the complete B04 selection/authority/currentness contract. §12.8's adapter responsibilities remain; §12.9 supersedes its source-access limitation. **Current conclusion (§13): the inspected public paths do not compose into complete B04 unchanged.** Recommend a bounded supported SDK materialization composition with exact-reference/authority inputs and owner-backed results, plus successor-aware Context Resolution admission for continuing local projects. Existing SDK shapes are changeable, not constraints on intended product behavior. Initial-only authority remains a narrower supported case, not a substitute for the continuing-project path.

**Collider flow reconciliation (§14):** the connecting layer is already anticipated by Collider’s plans but not fully bound. Collider owns intent/default/request and presentation coordination; Substrate owns durable execution coordination and runtime context custody/delivery; Handbook owns semantic resolution. Exact references can be supplied through authorized owner reads, and Handbook may resolve applicable dependencies—users need not manually enumerate every contextual document. This does not authorize an unversioned latest-content shortcut or move owner authority into the UI.

**Collider readiness correction (§15, updated October 19):** the main UI is a shell; the isolated corrected-client proof already has reusable Composer controls, coordinated CP/Lane/tab consumers, view-local Sticky and Toolkit grid interactions. Their exact assertions are narrower than complete routing, context workflow or B05. C-OQ-027 and the proof specification already prescribe actual-client fixtures then real-owner proof. Resume that staged experiment with the missing delta; no new fixture-first strategy decision or rebuild of Packet A is needed. Full V1 and the file-native engine are not prerequisites.

**No recovered evidence closes the complete B04 positive path:** exact selected context → supported Handbook operation → authorized Substrate execution → recorded actual injection. This is an evidence finding, not a claim that Handbook lacks useful resolution/grounding operations. Unfinished Context Resolution authority-management work and unfinished storage experiments must not automatically become prerequisites for every possible supported context route.

## 2. Evidence boundaries and authoritative baselines

### 2.1 What was actually inspected

Three read-only investigation agents covered chronological product intent, implementation/SDK evidence, and research/experiments. The parent reconciled those findings against the local Substrate/Collider contracts, Git state, remote advertised refs, and selected Windows command outputs. Windows sessions were read through Codex's connected-host history tools, including recorded source reads, diffs, Git output and human messages—not just sidebar summaries.

Evidence labels used below:

- **Live local/remote-ref observation:** checked during this investigation.
- **Recorded source/command evidence:** actual Windows command output recovered from a dated session; stronger than an assistant recap, but not a fresh execution in the Windows checkout.
- **Recorded completion/review:** a task's reported result; tests were not rerun here.
- **Accepted intent:** direct human approval/correction recovered chronologically.
- **Proposal/experiment:** advice, research, or an isolated implementation without production adoption.

Initially, direct Windows filesystem access was not established: noninteractive SSH authentication failed, and no further credentials were tried. The accessible GitNexus indexes pointed to stale Mac checkouts; no graph-absence inference was used and no index was rebuilt. **Later Git refresh:** the published Windows checkpoint is now locally available and its ancestry and focused source were inspected (§12.9). This supersedes the session-output-only limitation for those committed files. It does not certify the Windows machine's current dirty tree, installed binary, or a fresh test pass. Unpublished/untracked research remains session evidence, not part of the fetched snapshot. The subsequent §13 pass refreshed the Handbook index with `--index-only` and attempted focused tests; those later actions and their limits are recorded there.

### 2.2 Repository identities

| Repository / surface | Baseline and status | Interpretation |
|---|---|---|
| Substrate Mac | `docs/substrate-collider-contracts` at `aa5ef73ae087aa82d1f952ba6dd5c92361b8e24f`; clean before this document | Current local B01–B05 target handoff, not runnable integration. |
| Collider Mac | `spike/agentkit-controller-correction` at `a491f98724e539a3d7a23aa0661b6b99af79c657`; only pre-existing untracked bundle ZIP | Current consumer mapping and historical Packet A verification. No new test execution here. |
| Handbook historical Mac branch / `origin/feat/handbook-contract-membrane` | `8605d6539355f350c55b921d95fb450bca30ee18`; preserved unchanged | Historical inspection baseline used by B04 §6.1. Commit itself dates to August 11; “September baseline” means the September inspection, not September implementation. |
| Handbook refreshed Mac review checkout | `feat/handbook-b04-source-review`, tracking `origin/codex/file-native-baseline-379af4e`, at `379af4eda9b95ee82be9d3afcfdcdca2d6d132f9`; clean | Fetched October 18. Contains 170 commits beyond the historical branch, including the HCM-4.6 checkpoint. Full committed source is now available; the remote branch name does not mean it contains only the storage experiment. |
| Handbook advertised `origin/main` | `a3babd20329027afacdcee9d8b7b9d638d15af5b` | Not evidence that Windows local work is absent. No `feat/handbook-contract-membrane-clean` branch was advertised by origin during this read. |
| Handbook Windows | `C:/Users/spmcc/Documents/__Project_Code/handbook`, recorded branch `feat/handbook-contract-membrane-clean` | Relevant newer local implementation/research stream. Do not silently replace it with Mac or GitHub truth. |
| October 17 recovered committed Windows checkpoint | `2cec91857fb49139ecdeeef4d723b4ce40940ed3`, publication experiment; predecessor `b6b3aed9beb3f5c832d2cd10978e71fdce68fdcf` | Recorded Git commit output and later `rev-parse` confirm it. Active follow-up remained untracked/in progress at observation. This is a recovered checkpoint, not a promise the branch will stay there. |

October 18 follow-up: T8 command `exec-c8bbe97d-cfcf-4286-8ead-c374b9ecac28` records newer short HEAD `379af4e`, “research: execute native file-engine and lazy COW follow-up,” and an untracked next-experiment note. This supersedes the earlier latest-checkpoint observation, not the source-specific HCM evidence pins. Its full SHA, complete tree and research results were not audited in this SDK pass.

Windows source root is abbreviated **`HB-WIN/`** below. Host ID for recovering every Windows session is `remote-control:env_e_6a3d9cd24b5483238ba55b699a98be35`.

Numerous relevant research/proposal directories are untracked on Windows. A commit-only bundle would omit parts of the human decision history. Do not clean, blanket-stage, or infer approval from their tracked/untracked status.

### 2.3 Authority is not just a branch name

Use the current accepted semantic amendments and bounded implementation selections for intended behavior; use source and evidence for what is implemented; use later explicit human corrections to interpret supersession. A commit, passing fixture, Pro recommendation, or sibling feature does not itself select a new product contract. The newer Handbook control-pack entry is `HB-WIN/docs/specs/handbook-contract-membrane/README.md`; the old Mac `00-README.md` is historical navigation, not the sole current status source.

## 3. Chronological reconstruction

Dates below identify the recovered decision/landing sequence; cited session timestamps may be UTC and cross local midnight. Full source/session identifiers are in §9.

| Period | Decision / landing | What it establishes—and does not |
|---|---|---|
| Aug 29, revalidated in the Sept 24 restart | HCM-4.6 closed at `4750aa22`; Snapshot Memory/grounding and fixed session-boundary composition exist beyond the Mac baseline | Recheck those real operations before adding a new consumer abstraction. Internal adapter proofs are not installed external Substrate integration. See §4. |
| Sept 24–Oct 1 | Resumed HCM validation uses the reset control pack (`8c93d6f3eacf3312c935a6f83ff62244450dbf1b`); Sept 30 human semantic approval recorded in `f186db6` Oct 1 | Current semantic owners/navigation supersede relying on the old pack alone. Human rejects complex IAM drift and favors attainable explicit project configuration/local instruction authority. This does not date the reset itself to September or remove exact currentness/owner enforcement. |
| Sept 28–30 | HR-001 and HR-004–006 semantic selections | Explicit establishment/replacement under previously established authority; diagnostic versus decision-seeking evaluation; positive inapplicability accounting rather than fabricated runs; fresh checks may authorize launch under the same committed basis, not silently replace fixed inputs. |
| Oct 1 | Dock correction `b0e73fc`; coverage map `9c69b17`; human checkpoints `d271b56` | Do not resurrect stronger Dock confinement as a prerequisite. Dock adapts established tools, first concrete adapter generic JSON Schema. Review/source comparison is not a fresh test run or implementation authority. |
| Oct 2 | Governance establishment `aa793c1`, completion documentation `f62b1f4` | Exact fresh-project/Contracts freshness scope and supplied ceiling; a 24-hour example is configurable, not universal policy. |
| Oct 2–4 | Human corrects mandatory-governance and “Draft” assumptions; approves amendment at `31e7746` | Complete useful independent Contracts may exist without governance. Once applicable governance is established it is enforced. Defaults are replaceable; adaptive setup and skill guidance do not create deterministic permission. |
| Oct 4–5 | Explicit deferral `8b13505`; later same-project establishment `9f28f99` | Deferral is a recorded decision, not absence-as-permission. Later establishment changes current applicability while original recovery retains history; no project identity recreation. |
| Oct 5–8 | Independent local-check/native SDK packets; recorded completion through `be7d040` | Real local-advisory Contracts/evidence/evaluation without mandatory hardware authentication. No implicit CLI/skill workflow or Substrate injection proof. |
| Oct 6 | SDK-first delivery clarification `2da2760` | Human prioritizes SDK usability/features; CLI and skills follow the relevant SDK work. Do not invent a new CLI dependency merely for B04 design. |
| Oct 6–8 | Context Resolution lifecycle design `70019b7`; extension investigation `2740e042d1e161b8e80fb84435d0775eda4bc5d4` | Reuse of local-advisory mechanisms is favored for investigation and possible adoption. Exact successor-native root binding was **not adopted or implemented** by these reports. |
| Oct 8 | Joint CR/Work Progression reconciliation `41b4e2e`; work semantics `2ce0871` | Binding management, root-envelope use and work registration are separate permissions. Context Resolution is not a universal predecessor of all work registration. |
| Oct 9 | Native registration foundation `cbfbf76` following selections `cb00b9d` | Durable open leaf registration, exact request/authority/history/recovery. Registration completion is not work completion. |
| Oct 9–10 | Unified composition-aware selection `277674e`; implementation `7129b5c`; immutable kind versions `ae1af38`; reconciliation `5363e0a` | Ordinary/planning-output/requirements-input compositions and per-version approved kinds landed; human reviewed completion. Hierarchy, eligibility, closing and CR selections remain separate future work. |
| Oct 11 | Phase refactor `83fc8b6`; investigation `b08ea31` | Prepare binds exact request; invocation establishes current permission/conflicts; committed-original recovery preserved. Four invocation scans and 32 lifetime transactions still remain at that product checkpoint. |
| Oct 12–16 | Isolated membership prototype `c7f54e63a17adaf4c826eeb5aad797d090d79f05` → transaction experiment `b6b3aed9beb3f5c832d2cd10978e71fdce68fdcf` | COW/buffering evidence, not production work-state migration or a B04 API. EchoDB-inspired patterns, not imported EchoDB source/dependency. |
| Oct 14–17 | Human emphasizes meaningful readable files and selective constituent-document composition; broadens to Handbook-independent engine | Accepted research direction, not universal YAML migration or adoption of literal graph-export examples. |
| Oct 17 | Publication experiment `2cec91857fb49139ecdeeef4d723b4ce40940ed3`; active follow-up | Experimental Windows evidence; later follow-up also uses WSL2. Both readable-view options remain open; Dolt comparator-only. No production backend selected. |

## 4. Current implementation: useful foundations and sharp boundaries

### 4.1 Handbook has more than the old packet-ID route

The local historical source confirms `HandbookSdkV1::generate_packet` / `inspect_packet` use `packet_id`, and the old `FlowRouteError` is diagnostic-text based. These are **historical source facts**, not an exhaustive inventory of the current Windows SDK.

Recorded source identifies completed HCM-4.6 Snapshot Memory/grounding owners, durable repository support, four ordinary operations, and fixed SDK session-boundary/recomposition. Execution/review lineage recovered from T15–T17 is `72f2b35` → `f7ab6b8` → `3c7ed255` → `e789fddb` → `8ffc63b8` → `622f01b4` → `4750aa22` (closeout full identity `4750aa22404ff750035250e1f9f1aed2b4ca32d7`). Do not interpret the old B04 paragraph as proving these foundations absent.

| Implemented surface | Actual source-backed shape | B04 fit and limitation |
|---|---|---|
| `snapshot.capture` | `workspace: ExactBinding`, bounded `host_session_ref`, boundary `Start` or `End`, bounded `idempotency_key`; trusted `SnapshotSourcePortV1` required | Exact boundary capture foundation, not arbitrary caller-selected references. A client cannot simply declare itself the trusted source. |
| `snapshot.read` | `SnapshotTargetRequestDtoV1` with Workspace and expected boundary `{exact_ref, fingerprint}` | Descriptor/accounting read, not a demonstrated full-content/prompt export. |
| `snapshot.delta` | Same exact target request family | Owner delta evidence; not a Substrate filesystem diff or automatic context mutation. |
| `snapshot.project` | Same exact target; result carries grounding, capture reference/fingerprint, definition, disclosure, lossiness, omissions, refusals, required rechecks, Resolution stack/values, handoff references and `current` | Strong typed provenance/currentness foundations. Not demonstrated as an arbitrary `ContextSelection` resolver or a provider-ready context-body renderer. |
| `handbook.use-case.session-grounding-boundary@1.0.0` | `SessionGroundingBoundaryRequestV1`: Workspace, `host_session_ref`, Start/End, idempotency key. SDK `run_session_grounding_boundary_use_case` takes request, trusted source port and limits | Fixed direct Rust SDK/generic Tauri composition; not an ordinary discoverable operation or special CLI command. Existing source must be reused where it fits, rather than adding a parallel boundary. |
| `recompose_session_grounding_boundary_use_case` | Takes committed `SnapshotCaptureOutcomeV1` and limits | Receipt-based recomposition without recapture. It is not automatic permission to recapture/relaunch an uncertain Substrate request. |

The composed result includes terminal/sequence/predecessor, exact boundary/accounting/delta/grounding/receipt bindings, replay status, Flow/pipeline status and namespace, handoff references and recovery action. Its outcome distinguishes `Composed`, `OwnerFailed`, and `CaptureCommittedCompositionFailed`; the latter preserves the committed receipt. A failed composition cannot be silently treated as no capture.

Ordinary operations use exact definition/body-schema envelopes and independent owner admission. Recorded coverage includes positive SDK/Tauri capture and CLI read/delta/project seeded by SDK. **CLI capture refuses when the trusted source port is absent**; the evidence does not establish an ordinary CLI host capable of supplying that trust.

`read_current_snapshot` checks exact expected reference/fingerprint against the current stream head, rechecks the head for `SourceDrift`, and validates accounting coverage. That `current` has a specific snapshot-head meaning. It is not blanket proof that every underlying source, disclosure permission, or later Substrate destination is still current. Nor is a Handbook session-boundary operation an instruction to continue a provider-native session; the external host/session identity mapping remains explicit.

The reported external-adopter obligations remain material: a real trigger host; trusted repository/Workspace/session binding; start/end; cancellation and delivery behavior; UX/deployment proof. The host must preserve start/final-delta/handoff evidence and recover a receipt without recapture. It may not manufacture predecessor, currentness or grounding truth. The reserved surfaces `projection.definition.read`, `projection.create`, `snapshot.verify_current`, and `snapshot.resolve_applicable` are deliberately absent; their absence does not establish that their internal semantics are unimplemented or that all four must become public APIs for B04.

### 4.2 Context authority lifecycle is not the same operation as content resolution

Current native **initial RootCreation** and the investigated **successor-native reusable root binding** are different:

- Recorded source uses `NativeLocalAuthorityServiceV1::admit_envelope` with `LocalCurrentAuthorityWitnessV1` for the bounded initial local path.
- The initial path applies its approved level/reach/constraints/ceiling and excludes parent/escalation authority.
- Its currentness rules stop at the local-check successor boundary; it is **not** evidence that reusable root admission after successor adoption is implemented.
- The older credential-backed path remains a distinct route. Public access to a Rust method does not approve swapping its authority model.
- The October 8 investigation proposes reusable native root bindings, exact human establishment and bounded delegated replacement, revocation/history/recovery, and additive native records. That is a proposal, not an accepted API.

The human already favored investigating local-advisory reuse. Do not reopen that general preference. If the selected B04 path requires the proposed successor binding, present only the remaining exact adoption decisions. Conversely, do not demand the whole lifecycle extension before demonstrating whether an existing supported content path already fits.

### 4.3 Optional governance does not mean optional authorization

The accepted direction separates complete independent Contracts from governed project qualification. Explicit deferral/later establishment preserves current and historical truth. B04 must still honor whatever authority, disclosure, Scope, currentness and required context apply to its selected route. A skill may recommend an action; neither the skill nor an LLM reading of a document grants it.

The September 30 approved semantic package also distinguishes diagnostic checks from decision-seeking evaluation: a weaker development contract passing is not evidence that independently governed completion requirements were satisfied. No-observation results require positive inapplicability accounting, not fabricated execution. Fresh verification can authorize a later attempt under the same committed basis; it cannot silently substitute new fixed inputs. These are accepted semantics, not assertions that every corresponding implementation is complete.

Keep at least these distinct:

```text
Handbook binding-management permission
Handbook root-envelope/context-use permission
Handbook work-registration permission
Substrate execution/destination permission
```

A successful check after releasing its guard is not automatically permission for a later write or launch. The cross-owner boundary must identify where currentness is revalidated and what remains protected through the effect.

### 4.4 Work registration is real, but not B04 by another name

The newer product supports explicit composition and optional immutable approved kind versions. Requirements-input work names an exact Contracts definition, not another work item. Registration records an open item and a completed registration transaction; it does not assess the requirement, close the work, or deliver agent context.

Do not force the first B04 task through an unimplemented hierarchy/eligibility/closing system merely to display a Work Item. Nor may a registration receipt serve as a context-resolution receipt or Substrate acceptance receipt.

## 5. Research direction and what has not been selected

### 5.1 Accepted direction to investigate

- Meaningfully named readable files/folders let humans or ordinary agents find useful content without installing Handbook or a storage engine.
- Stable identity is separate from title/path.
- Small constituent documents can be selected and ordered into readable context with exact revision/provenance.
- A reusable generic document/graph core could serve Handbook, memory and research without encoding Handbook policy semantics.
- Preserve exact owner/versioned bytes. The structured-format standard is proposed; Contracts remain JSON under their existing contracts. No blanket migration to YAML was approved.
- Preserve the original Python pipeline's selective-composition experience, not its weak safety behavior: the historical pipeline could annotate missing required material instead of refusing it.

### 5.2 Both readable-view contracts remain open

| Candidate | Actual promise | Not implied |
|---|---|---|
| A: immutable originals plus incrementally refreshed readable current tree | Exact API snapshots and retained packets coherent; ordinary reads/copies may be mixed/stale; published/view-pending is distinct from mutation failure; recovery renders without replaying mutation | Mixed ordinary reads are not automatic failure. The user did not reject A by failing to formally accept it. |
| B: complete retained named generations | Reader selects one positively selected exact path; its reads/copies retain the promised closure while successors publish | Following a mutable current alias separately for every file is not snapshot binding. Complete closure has real materialization/storage costs. |

The human explicitly corrected the earlier rejection framing. Neither candidate is production-selected. Dolt remains a comparator unless practical alternatives fail a concrete requirement; a faster benchmark is not adoption. COW is a promising experimental candidate, not a selected production dependency.

### 5.3 Evidence ceiling

- The first publication experiment landed 60 files under its isolated research directory. Recorded verifier output covers 120 publication-race plus 480 semantic-race observations and 233 fault rows; 10 growth configurations completed and two 100k initializations were censored before selection.
- That experiment used whole-state JSON manifests and Windows process-fault fixtures. It did not integrate COW/DoltLite/SQLite, establish power-loss durability, or qualify production owners.
- Earlier membership/transaction measurements used Windows and WSL2/ext4. WSL2 evidence is not independent-host Linux or macOS qualification; fresh process does not mean device-cold cache.
- The current follow-up is separately authorized experimental work. Its final outcome/review/commit was not available at the investigation cutoff; progress is not completion.
- The latest human clarification replaces GUI browsing gates with PowerShell/Bash file navigation/extraction. Do not keep an obsolete GUI prerequisite alive.

This research does not prove a B04 supported reference kind, consumer operation, semantic authorization bridge, or actual agent injection. No demonstrated dependency currently requires replacing Handbook's storage engine before mapping one correct B04 route.

## 6. Mapping against B04

This table separates supported foundations, needed contract reconciliation and unproven integration. “Not established” means not demonstrated by the recovered evidence, not global proof of absence.

| B04 obligation | What exists / accepted direction | What needs change or precise mapping | Still missing for the integrated proof |
|---|---|---|---|
| Supported semantic consumer boundary | Four ordinary snapshot operations plus fixed `handbook.use-case.session-grounding-boundary@1.0.0`, trusted source port and receipt-based recomposition (§4.1) | Bind the selected case to exact current supported operation and version; do not invent `resolve_context` or use Rust visibility as approval | A verified concrete consumer request/result mapping at the selected source baseline |
| Typed selected references | Exact identities, versions, packet and semantic-owner foundations; configurable work kinds are separate | Enumerate which actual source/reference kinds the selected operation accepts, including purpose, Scope, posture, required/optional meaning | One positive selection that satisfies the real experiment without raw-prompt/transcript substitution |
| Authority/currentness | Native initial authority, credential route, independent local-check/currentness/recovery mechanisms | State the selected route's authority basis; distinguish existing support from proposed successor binding; no work-grant/context-grant alias | Safe applicable revalidation across preparation, admission and delivery |
| Required/optional inputs and refusals | Typed native outcomes and `snapshot.project` omission/refusal/required-recheck fields exist; old packet-route diagnostic error is not the entire current API | Map actual structured outcomes; retain committed capture on composition failure; do not parse diagnostic strings or import historical Python annotation-as-success | Missing/stale/denied/wrong-binding controls on the selected route |
| Semantic assembly versus runtime assembly | Handbook/engine may select, order, project and materialize meaningful context; file-native experiments explore exact recipes/closure | Clarify the two assembly layers below, rather than move semantic resolution into Substrate | Exact resolved-material-to-runtime-envelope mapping |
| Exact content identity and retention | Snapshot/grounding foundations; experimental retained revisions/publication lineage reinforce direction | Define content/reference retention and current access separately; path or immutable digest is not a permission grant | Proven availability/readability for the advertised recovery horizon |
| Request and accepted-work joins | Substrate B01/B03 target semantics define exact immutable request/destination history | Join Handbook material/evidence to those owner references without equating Handbook session, Work Item, Lane, World or request IDs | Durable, authenticated consumer join and uncertain-result recovery |
| Actual injection | Substrate is the agreed delivery owner; Handbook fixed boundary supplies owner evidence, not an external trigger host | Select the trusted source port and the material-content read/render path; define what runtime event establishes delivery. Snapshot descriptor/grounding bindings alone are not a prompt body or injection | Installed-path actual delivery plus distinguishable Resolved/Injected lineage |
| New storage mechanism | Readable constituents, COW/transactions and two view models investigated | Keep mechanism independent unless a required capability demonstrably depends on it | No production adoption or general engine qualification claimed; not automatically a B04 blocker |
| Consumer UI and continuity | Collider's resolution-reconstructed default and lineage model already documented | Present actual capability/refusal/lineage; never silently continue a native session or resolve content in the frontend | Real Control Plane/Lane projection of the same accepted work, reconnect/cancel evidence under B02 |

### 6.1 Recommended ownership wording—not yet an owner amendment

> Handbook resolves semantic applicability and may select, order, project and materialize versioned source content into a provenance-bearing context result. Substrate checks execution visibility and current execution authority, packages the admitted material for the selected runtime, and records actual delivery to the exact accepted work. Collider selects supported references and presents the resulting owner evidence.

This clarifies “assembly” rather than adding another assembler subsystem. A generic file engine can implement mechanisms underneath Handbook without owning Handbook policy. Substrate should not repeat Handbook's semantic graph traversal or recipe policy; Handbook's rendered packet should not certify Substrate injection.

### 6.2 Concrete mapping candidate and its remaining blanks

The initial grounding candidate was the existing fixed SDK boundary plus ordinary exact-reference reads/projection. The deeper source pass in §12 establishes that these do not supply arbitrary selected document bodies; evaluate the distinct existing Flow content route rather than assume the snapshot seam is the complete resolver. The user has now selected the existing-SDK direction (§11); this particular operation composition remains a candidate, not a verified complete integration:

| Proposed B04 meaning | Existing candidate / unresolved bridge |
|---|---|
| Exact Workspace | `ExactBinding` supplies a real reference/fingerprint carrier. Mapping a Substrate Workspace to the trusted Handbook repository/Workspace binding must be owner-established, not copied from a UI label. |
| Invocation identity and retries | Existing host-session reference, boundary and idempotency key carriers are available. Their relationship to a Substrate invocation/request/accepted work must be specified; equal strings or a Lane ID are not proof. |
| Selected source kind/revision, purpose/action, Scope/posture and requiredness | Not corresponding fields in the bounded Start/End capture DTO. Determine which supported source-port/owner route supplies and validates these meanings. Do not add undeclared fields or silently treat capture as arbitrary reference selection. |
| Resolved evidence | Capture/grounding/projected bindings, accounting, omissions/refusals and rechecks can contribute. Select which exact owner result is the B04 resolution evidence and how original request equality retains its caller inputs. |
| Context bytes | Descriptor/projection reference metadata is insufficient. Identify the supported content retrieval/materialization route and exact admitted bytes/closure; this remains a genuine mapping gap. |
| Delivery/recovery | Preserve the committed capture/receipt and use supported recomposition after composition failure. Add the actual Substrate delivery/result join; no recapture or task relaunch merely because the reply or injection report is uncertain. |
| End/finalization | The existing End/final-delta/handoff mechanism is useful. Define the external trigger and what cancellation/failure means without fabricating an End success or conflating it with Substrate terminality. |

**Change classification:** refreshing stale documentation and clarifying ownership are justified now. Building a new Handbook resolver, publishing reserved operations, adopting successor-native root management, or migrating storage is **not yet justified merely by this comparison**. External host/source binding, exact content delivery and proof are still unestablished; source changes should follow the selected concrete route, not precede it.

## 7. Proposed changes by repository

**These are recommendations, not edits made by this investigation.**

### Handbook

1. Select and document the exact supported SDK/advanced-owner route for one representative B04 case, using the current Windows owner baseline.
2. Supply its actual allowed references, authority prerequisites, request/result/refusal fields, version/currentness and retention behavior. Prefer existing supported composition; only specify a new operation for a demonstrated gap.
3. If that route requires successor-native root bindings, turn the already-researched direction into an explicit bounded adoption decision first. Do not silently implement the investigation proposal.
4. Keep work registration and file-native storage adoption outside the integration dependency chain unless the selected case actually needs them.

### Substrate

1. Refresh B04 §6.1's historical evidence pin/navigation and operation inventory once the chosen current Handbook route is verified. Preserve the old inspection as history rather than rewriting it as current truth.
2. Clarify semantic materialization versus execution packaging/injection.
3. Finish the concrete B01/B03 authority/request/binding/recovery join around that route, then B02 observation/cancellation. No implicit rebinding or second semantic/acceptance owner.
4. Implement and prove actual context delivery only under a separate authorized implementation slice. B05 coordinated dependency-declaration writers remain a separate unresolved obligation.

### Collider

1. Update the existing consumer mapping with the selected supported kinds/operation and real result categories; do not create a second planning stack.
2. Preserve distinct selected/resolved/injected evidence and exact request/work identity across Control Plane and Lane views.
3. Do not equate optional governance with incomplete Contracts; do not equate registration with completion or context delivery.
4. Preserve Packet A's existing client/fixture verification; do not restart it or claim it proves owner/native integration.

## 8. Shortest justified route to the real-owner prototype

1. **Pin a current, reproducible Handbook source/evidence subset.** The Windows branch and local-only/untracked evidence need an explicit captured baseline; blindly pulling the old GitHub branch will not do this. Do not move or clean the active experiment checkout.
2. **Map one positive context case through the strongest existing supported SDK route.** Recommended scenario remains one small code change/test with required versioned context in a disposable repository, but this exact scenario has not been ratified. Determine whether initial/established authority and available references fit before declaring a new lifecycle extension mandatory.
3. **Close only demonstrated owner gaps.** Present any exact successor-native authority adoption choice if necessary. Finish Substrate B01/B03, then B02; plan B05 writer coordination independently. Broad file-native-engine selection is not automatically on this path.
4. **Implement and run the real selected Linux experiment.** One ephemeral World task; actual Handbook-derived context delivery; exact owner observation in Collider's Control Plane/Lane views; dependency declaration read/add/remove. No E3-F restart, retained continuation, full scheduler, or complete architecture reset is implied.

**Sequencing clarification:** this section describes real-owner qualification, not a requirement to finish it before any Collider UI experiment. The corrected readiness and recommended fixture-first route are in §15. Fixtures do not satisfy the SDK-first/full-B04 acceptance condition below.

### Recorded scope decision — 2026-10-18

The user selected **existing Handbook SDK capabilities for the first prototype, conditional on fully satisfying B04**, and explicitly kept the file-native engine separate. This resolves the scope question raised by the original investigation. It does not assert that the available SDK operations already close every B04 obligation, select a new storage backend, or approve a reduced proof. No raw-prompt bypass or partial test may be presented as full Gate C.

The linked Windows research session's latest response (T8, turn `01a14f62-529b-71c0-95f7-3a9ceafe0bbd`) supports preserving the SDK/domain-owner boundary while storage evolves. It explicitly identifies concrete current storage dependencies, adapter/refactoring work, and potentially versioned public changes. It does **not** establish a configurable universal storage plugin, zero SDK changes, or B04 compatibility. The concrete integration mapping and remaining verification gates are in §11.

The human already clarified optional governance, local-advisory preference, flexible work composition, both readable-view options, and Dolt's comparator role. Do not ask those again. Exact source access, API mapping, locking, wire types and proof coverage are engineering/evidence work, not questions to offload to the user.

## 9. Evidence index and recovery instructions

Recover Windows threads with `read_thread(threadId, hostId)` using the host ID in §2.2. Cursor pages are newest-first; human messages and later corrections control intent. `includeOutputs` is needed for the recorded source/Git evidence. A task's final answer is not a fresh source check.

### 9.1 Sessions

| Key | Thread ID / title | Scope read and decisive evidence |
|---|---|---|
| T1 | `01a0fa1d-e246-72a1-944b-000987fe1896` — Meta Orch Main | Full available Oct 2–6 chronology. Human optional-governance correction `01a0fe8a-6624-7a70-9f75-beef0a80e6ed`; complete-not-Draft correction `01a10300-86fd-76a0-9863-f324c642f1b0`; actual amendment approval `01a1052a-3be3-7471-875b-4ccd49f78912`. |
| T2 | `01a10837-4c30-75a0-adc3-3b922f7dc6f0` — Meta Orch side | Focused Oct 6 native SDK/local-check completion evidence; older turns not all read. |
| T3 | `01a112f2-2a27-7703-8380-1fcc13ea9542` — Meta Side - Context Resolution | Full available fork history Oct 6–8. Human SDK-first `01a11271-d329-7a31-9701-d1c509ea12bc`; design authorization `01a11312-2e7d-78d3-bff5-7237d50438fd`; extension investigation-only `01a11cef-adb5-7002-919c-4994520b5be3`. |
| T4 | `01a11daf-85c5-7341-9f35-ed042fd480ea` — joint collection | Complete collection turn `01a11daf-8a34-7fe1-b05e-54e528c77144`, source-grounded state and separate permissions. |
| T5 | `01a11db8-c303-7762-8e5e-a71f963dca70` — joint reconciliation | Complete turn `01a11db8-c7c6-7a23-b223-ec1a881f9a40`, `41b4e2e`, remaining CR selections and independent Work Progression. |
| T6 | `01a11312-4fce-7e22-ae11-50ff5edf2d15` — Meta Side - Work Progression | Latest ten Oct 7–8 turns; human planning/output/requirements/hierarchy intent and `2ce0871`. |
| T7 | `01a11d53-45f8-7da2-9d77-d192042ca083` — Meta - Work Progression and Context… | Full available Oct 8–17 chronology, eight pages. Registration, compositions/kinds, human acceptance, phase refactor, research dispatches. Oct 10 reconciliation `01a123d5-5a7c-7ea0-b0dc-dc99841adbb8`; Oct 11 refactor `01a12bd1-eeea-7873-8620-58b66379baa0`. |
| T8 | `01a12df3-39a5-7a52-a8fe-71217078691e` — Research file-based durable membership… | Research chronology through the initial investigation, plus the SDK/storage-boundary follow-up read on October 18 (turn `01a14f62-529b-71c0-95f7-3a9ceafe0bbd`). Constituent-composition correction `01a145ad-21c8-7b73-9146-8bc7a2e7097d`; generic-engine direction `01a145ca-080b-7131-a5b6-aadce1017f20`; both-options correction `01a14a74-422b-7fc3-a219-2d2781449dcc`; Dolt comparator `01a14a79-233a-75e2-bff9-c8a04f9bfce8`; terminal walkthrough replacement `01a14bab-dec2-7ab1-982e-cbc7863b86ae`. |
| T9 | `01a130fd-cc2e-7e60-b09f-07993a6f9448` — Prototype durable membership: chunkmap… | Completion turn `01a130fd-cfa5-7c32-83c9-06f10b07d51f`, `c7f54e63`. |
| T10 | `01a13169-e42b-77d0-9fcb-bc9e4fdb5e01` — Complete transaction-layer experiment… | Completion turn `01a13169-e819-7303-90fc-3e8bc71a1d19`, `b6b3aed9`; repaired earlier experimental findings are not current unresolved defects. |
| T11 | `01a14553-3158-7483-9802-1283c097da4b` — Trace original Python pipeline | Turn `01a14555-ca5e-7430-826d-f6cf933ac078`; root commit `a3245951c49bd8ad59b2b8f56fb98b157520f2c7`, reconstructed selective assembly and its safety limits. |
| T12 | `01a147d4-f753-7643-9148-7965a98c78f1` — Research Dolt and GitButler for file-native… | Focused completed research findings; comparison/reuse, not production adoption. |
| T13 | `01a14a83-90fe-7380-9daf-ab876a01ada9` — Run file-native document and graph publication… | Turn `01a14a83-9353-7df2-97ea-365def59f207`. Commit output `exec-812952db-4bcc-41ac-8639-4ea160fc260b`; postcommit hashes/status `exec-7c0f9fcb-9861-48bc-8c27-67ce4ce34432`; verifier `exec-903b9553-ac2f-4d9d-95df-73a9d9564d05`. |
| T14 | `01a14bb1-0b19-7a43-aa13-3bc8704d943e` — Complete file-native engine follow-up… | Active turn `01a14bb1-1076-7591-b016-2cce876bd604`, scope and recorded `2cec918` HEAD/status; unfinished. Parent directly re-read selected Git output. |
| T15 | `01a04f96-2944-70b1-ab0b-87acfb7f5f4c` — Execute HCM-4.6 sequential packets | Aggregate source/review/closeout history, four ordinary operations and fixed boundary through `4750aa22`. Meta source read `exec-d4fc9c1c-f69c-400b-96de-4ab4e69e0743` at the `622f01b4`/closeout stage establishes exact-head/currentness bounds. |
| T16 | `01a0501b-3f3a-7531-94ec-4194855a7ea8` — HCM-4.6 Packet 4 | Recorded request/facade and SDK source at `e789fddb`: fixed request, trusted port, composed/owner-failed/committed-composition-failed results. |
| T17 | `01a05098-c23b-7700-9bb4-7869ba3dec11` — HCM-4.6 Packet 6 — Consumers and Integrated… | Command `exec-20415d09-7d2e-483f-b14c-6d5c9affb1d9` reads snapshot DTO/operation source at predecessor `8ffc63b8`; parent independently re-read the returned source for requests, results, operation admission and trusted-port forwarding. |
| T18 | `01a0dbba-fa90-7f41-ac0f-ef2a3bb70b94` — Resume HCM phase validation (2) | Full Sept 24–Oct 1 restart chronology. Sept 30 human approval `01a0f3df-2ef7-7a53-aa61-1640b618a9f5` approved the combined package; individual amendments had earlier explicit acceptance. |
| T19 | `01a0f54b-9a36-72d3-b740-d9c146ef2672` — Load session handoff context | Full Oct 1–2 successor chronology; `f186db6` approved semantic checkpoint, corrected Dock obligation, source-backed coverage comparison and bounded human checkpoint requirements. |

### 9.2 Important Windows documents

Paths below are recovery locators, not claims that every complete file was independently read on disk in this investigation. Relevant content was recovered through session reads/outputs; preserve each source's stated approval status.

- `HB-WIN/docs/specs/handbook-contract-membrane/README.md` — current control-pack navigation and bounded completion.
- `HB-WIN/docs/specs/handbook-contract-membrane/semantic-model/amendments/governance-defaults-and-delegated-operation.md` — accepted governance correction.
- `HB-WIN/docs/specs/handbook-contract-membrane/semantic-model/amendments/work-progression-kinds-and-relations.md` — work semantics, not completion implementation.
- `HB-WIN/docs/research/hcm-coverage-landing-2026-10-01/future-obligations.md` — HCM completion versus external adoption obligations.
- `HB-WIN/crates/sdk/src/contract_membrane/snapshot_operation.rs` — recorded lines 21–26 operation references; 32–37 capture request; 60–63 target request; 138–165 projection/problem results; 294 onward owner-admitted dispatch.
- `HB-WIN/crates/sdk/src/lib.rs` — recorded lines 330–379 identify fixed boundary/recomposition; line numbers belong to the inspected historical Windows revision, not a claim about today's line positions.
- `HB-WIN/crates/sdk/src/contract_membrane/facade/request.rs` — recorded `SessionGroundingBoundaryRequestV1` near line 579.
- `HB-WIN/crates/sdk/src/contract_membrane/session_grounding.rs` — fixed composed result and receipt-preserving failure outcomes.
- `HB-WIN/crates/engine/src/snapshot_grounding_operations.rs` — recorded lines 113–190, descriptor/accounting and current-head checks.
- `HB-WIN/docs/research/context-resolution-native-authority-lifecycle-design-2026-10-06.md` — design, not adoption.
- `HB-WIN/docs/research/context-resolution-local-advisory-extension-investigation-2026-10-08.md` — initial/successor distinction and proposed reuse.
- `HB-WIN/docs/specs/handbook-contract-membrane/slices/work-registration-phase-refactor/README.md` and `HB-WIN/docs/research/work-registration-phase-refactor-2026-10-11/README.md` — product refactor contract/evidence.
- `HB-WIN/docs/research/durable-membership-prototype-2026-10-12/` — comparison, transaction contract/review/verification/platforms.
- `HB-WIN/docs/research/original-python-pipeline-2026-10-16/README.md` — recovered intent and historical mechanism limits.
- `HB-WIN/docs/research/dolt-gitbutler-file-engine-2026-10-16/` and `HB-WIN/docs/research/file-engine-experiment-pro-consult-2026-10-17/` — revised proposal plus historical advice; later human corrections supersede earlier framing.
- `HB-WIN/docs/research/file-native-publication-experiment-2026-10-17/` — committed protocol/comparison/verification/review and raw evidence.
- `HB-WIN/docs/research/file-native-engine-followup-2026-10-17/` — active, not completed at cutoff.

### 9.3 Local handoff sources and preservation

- [Substrate B01–B05 handoff](../contracts/draft/collider-b01-b05-handoff.md), SHA-256 `4c6b66bfb9cd6d41f53b45d6ac9bf827925351ca9092478c8b69a091bddb2a24`.
- [B03 binding](../contracts/draft/lane-world-binding.md), SHA-256 `8ae534dea0d9cfbc8d147454e1aa83534a8c99ea50d17c119fb9f840451c8f5c`.
- [Core dossier](../../SUBSTRATE_CORE_ARCHITECTURE_SESSION_DECISIONS_2026-09-20.md), SHA-256 `6ff7f560a8d0c02178192051c207ef132a34aaa8b5e17d2c725a94a4ec43968f`.
- [Historical runtime dossier](../../RUNTIME_REFACTOR_SESSION_DECISIONS_2026-09-18.md), SHA-256 `d3538ad11d215ac801ee3ba6c21605d98d4c376ef6033c174fd485765e1dcfca`.
- [Collider existing consumer mapping](../../../collider/docs/agent-native/controller-ownership-plan/substrate-accepted-work-boundary-plan.md) and [foundations §11](../../../collider/docs/agent-native/canonical/01-collider-v1-foundations.md#11-context-and-continuity-model).
- Saved September Pro/local reconciliation: `.codex/chatgpt-pro-guidance/2026-09-22-published-cross-repo-consult/local-review-validation.md`. It supports target-design coherence, not current Handbook or runtime readiness.

Only this review document is authored by the investigation. Existing handoffs, dossiers, canonical owners, source, research inputs, and sibling repositories are preserved. No commit, push, implementation, benchmark, installed-path test, E3-F resumption, or production adoption is implied.

## 10. Review and validation of this synthesis

For the original October 17 synthesis, the three investigators separately read the complete compiled document against their evidence tracks. No blocking semantic, intent or experimental-scope findings were reported. The parent corrected exact reserved-operation names, one Markdown table delimiter and the distinction between individual amendment approvals and whole-package approval; it also independently re-read the recorded snapshot DTO/dispatch source and Windows Git evidence.

Mechanical validation checked local Markdown file targets, table delimiter consistency, whitespace including the new untracked file, and unchanged hashes for both Substrate drafts and both dossiers. The only new repository file is this document; it remains uncommitted. These are document/evidence checks—not an independent fresh Windows code audit or an integration test. Active Windows experimental work may advance after this report's observation cutoff.


## 11. Selected SDK-first prototype: concrete integration mapping

### 11.1 Decision and supported-boundary recommendation

**Accepted scope:** existing SDK first, complete B04 behavior required, file-native engine separate. **Proposed integration:** a trusted Substrate host integration calls Handbook's supported SDK boundary; Collider supplies typed selections and displays owner evidence. Prefer the Rust SDK composition where it fits, rather than introducing a CLI prerequisite or accessing private owner storage. Deployment/process transport is not selected by this document.

The candidate call sequence below is intentionally explicit about gaps. It is not executable pseudocode or a new serialization contract. In particular, snapshot metadata is not material content, an internal source port is not a public arbitrary-reference resolver, and SDK use alone does not prove delivery.

```text
Collider typed selection + original Substrate request
  -> authenticated/authorized Substrate preparation and exact destination
  -> trusted Handbook repository/Workspace/source binding          [adapter gap]
  -> supported semantic selection + grounding/materialization     [mapping gap]
  -> exact admitted content + Handbook provenance
  -> Substrate visibility/currentness checks and runtime packaging
  -> B01/B03 acceptance with immutable context/destination linkage  [owner gap]
  -> actual runtime delivery evidence -> B02 observation            [proof gap]
```

### 11.2 Stage-by-stage operation mapping

| Stage / owner | Concrete candidate and required input | Result, refusal and unresolved work |
|---|---|---|
| 0. Pin the usable capability / Handbook | Rebind the current Windows source to an exact reproducible revision plus needed local changes; verify the supported SDK version and reference-kind inventory. The old Mac pin is insufficient. | Produce a bounded capability mapping, not a claim that every public owner method is supported for ordinary consumers. Existing recorded sources in §4 are evidence leads, not a fresh build. |
| 1. Prepare / Substrate | Authenticate caller, authorize preparation, retain the original B01 request identity and full typed selection, validate the expected Workspace/Lane/World association under B03. | No accepted work yet. A selected reference or destination label grants neither content access nor execution authority. Preserve purpose, requested action, Scope/posture and requiredness even though capture DTOs lack those fields. |
| 2. Bind trusted inputs / Handbook integration | Map the Substrate Workspace to Handbook's real repository and `ExactBinding`. Bind a host-session reference and boundary-specific idempotency keys. Supply `SnapshotSourcePortV1` only through a supported trusted adapter. | Exact source-port construction, permitted source kinds and validation remain unverified. Its `observe_slot` / `read_bounded_slot` methods are owner source mechanisms, not permission for Collider to supply arbitrary JSON as trusted state. |
| 3. Capture/compose when applicable / Handbook | Candidate: `HandbookSdkV1::run_session_grounding_boundary_use_case` with `SessionGroundingBoundaryRequestV1 { workspace, host_session_ref, boundary: Start, idempotency_key }`, trusted port and limits. Exact fixed use case: `handbook.use-case.session-grounding-boundary@1.0.0`. | Preserve owner-derived predecessor, boundary/accounting/grounding/receipt and recovery result. `Composed`, `OwnerFailed`, and `CaptureCommittedCompositionFailed` differ. Do not separately call ordinary capture and then the capturing composition for the same boundary. Verify that this lifecycle fits invocation preparation before selecting it. |
| 4. Inspect exact grounding / Handbook | Where needed, `snapshot.read@1.0.0` and `snapshot.project@1.0.0` use `SnapshotTargetRequestDtoV1 { workspace, expected_boundary: { exact_ref, fingerprint } }` from the returned owner result. `snapshot.delta@1.0.0` supplies owner delta evidence when needed. | Retain accounting, omissions/refusals, disclosure, required rechecks and Resolution provenance. These current-head operations may refuse an older boundary; do not replace it with latest or assume historical content retrieval is supported. They do not themselves export the complete context body. |
| 5. Resolve/materialize required content / Handbook | Identify the supported route from exact selected kind/revision, purpose/action and Scope/posture to actual admitted content, ordering and provenance. Reuse an existing SDK operation if verified; do not invent `resolve_context`. | **Critical open mapping:** no recovered source demonstrates this full selection-to-bytes route. If unsupported, refuse preparation and specify only the demonstrated missing adapter/SDK capability. Empty selections, raw prompts and metadata-only packets do not satisfy B04. |
| 6. Assemble and admit / Substrate | Bind the original selection, Handbook evidence and exact admitted material to the original B01 request and expected B03 destination. Revalidate effective permissions, source rechecks and destination binding at the applicable effect boundary. | Define the authority guard/revalidation protocol across owners; a prior successful read is not a launch grant. Request equality, durable correlation and the acceptance transaction remain B01/B03 design work. Changed required inputs cannot silently reuse the same accepted request. |
| 7. Deliver and observe / Substrate, then Collider | Deliver the exact admitted material through the selected real execution path. Record `Injected` only from runtime-path evidence bound to accepted work; expose distinct selected/resolved/not-yet-delivered/injected or failed states through the authorized observation path. | Acceptance is not delivery. Lost delivery acknowledgement remains uncertain until owner reconciliation. Failure after acceptance retains accepted-work identity and terminal/recovery semantics. Collider neither manufactures an injected record nor resolves the semantic content. |
| 8. Finalize and recover / respective owners | If the selected Handbook lifecycle requires End, use its supported End boundary and exact host binding; define triggers for completion, cancellation and failed preparation. Recover committed capture through supported exact replay/recomposition. | Handbook boundary finalization is not Substrate terminal truth. End failure does not reverse task completion; task failure does not prove End ran. The selected host must define outstanding finalization recovery without inventing a successful boundary. |

Stage 3 remains a grounding/provenance candidate, **not the content-materialization path and not a newly mandatory prerequisite for every supported content route**. The source verification in §12 supersedes any interpretation that stages 3–4 already yield selected document bodies. If a different existing SDK route fully satisfies the selected B04 case without this boundary lifecycle, document that supported mapping instead. Do not add artificial Start/End sessions merely to make a DTO fit.

### 11.3 Identity and provenance joins that must remain distinct

| Identity / evidence | Required relationship |
|---|---|
| Authenticated caller and current authorization | Resolve through each applicable owner's authority rules. A user-facing selection, skill or file path is not proof of permission. |
| Original B01 request and typed selection | Preserve exact equality inputs and original retry key. Record preparation evidence without pretending an accepted-work identity already exists. |
| Handbook repository and Workspace `ExactBinding` | Trusted mapping from the Substrate Workspace; not equality of display names, directory strings or IDs. |
| Handbook host session and Start/End request keys | Stable, explicit host lifecycle binding. Different boundary requests have distinct request identity; exact replay preserves the original request. Do not equate these with Lane, World, logical worker or provider-native session IDs. |
| Snapshot/grounding/receipt bindings | Exact owner results and applicable currentness/retention rules. Their fingerprints do not grant content access or certify source freshness indefinitely. |
| Resolved material identity | Exact included source revisions, omissions and ordered admitted content. Select the owner-supported byte/content identity and serialization rather than inventing a hash convention here. |
| Accepted work and immutable destination provenance | Join the preserved preparation result to actual B01 acceptance and the authorized B03 binding; later association changes do not rewrite history. |
| Actual delivery evidence | Join exact delivered material to that accepted work and execution. Neither an SDK success nor a successful transport write without the selected runtime evidence is sufficient by itself. |

Retain only authorized necessary material in receipts/logs; lineage inspection needs its own access checks. Immutable identity and retained bytes are separate from current permission to disclose them.

### 11.4 Failure and recovery mapping

These are required cross-owner behaviors, not claims that corresponding Substrate APIs already exist. Actual structured Handbook outcomes must map to the B04 categories; never derive authorization or retries by parsing diagnostic text.

| Situation | Required handling |
|---|---|
| Unsupported kind/operation, malformed selection, incompatible Scope/posture, required missing/stale/denied material | Refuse before B01 submission on the chosen pre-acceptance path; redact denial detail. Optional omission requires explicit optionality, owner permission and recorded reason. Do not select latest as a fallback. |
| Capture response lost or composition fails after capture committed | Preserve the exact request and committed receipt. Reconcile/replay the original owner operation or use `recompose_session_grounding_boundary_use_case` with its committed outcome and limits as supported; no blind new capture. Composition failure is not evidence of noncommit. |
| Snapshot head advances before read/project | Honor currentness refusal; preserve historical capture evidence. Reprepare only through an explicit permitted path with newly bound inputs, not by substituting the newer head in the old request. Historical recovery support must be verified separately. |
| Permission, required source or destination association changes between preparation and delivery | Recheck through the relevant owner and stop/hold/refuse under the selected protocol. Do not reuse a cached pack as authority. If already accepted, preserve the work and report its actual blocked/failure state. |
| B01 submission acknowledgement lost | Reconcile the original accepted request through the acceptance owner. Handbook receipt replay cannot establish whether Substrate accepted or launched work; it cannot authorize duplicate dispatch. |
| Accepted work has missing/uncertain injection evidence | Report not-yet-delivered, failed or uncertain according to authoritative evidence. No fabricated `Injected`, erased acceptance or automatic relaunch. |
| Task cancellation/completion races End or its acknowledgement | Keep B02 terminal truth and Handbook finalization separate, preserve both records and recover the outstanding operation. Do not recapture or revive completed execution to repair an End receipt. |

The SDK recovery action named `DeliveryRecoveredByExactReplay` is **Handbook's own recovery vocabulary** (T16, `session_grounding.rs`); its name does not establish Substrate runtime injection. Likewise `Started`/`Finalized` in that composed result are Handbook boundary outcomes, not World-task lifecycle states.

### 11.5 Completion gates and immediate next deliverable

The scope decision is settled; engineering verification is not. The next bounded deliverable is **one source-backed Handbook adapter mapping for a real required-context case**, answering all of:

1. Exact current source baseline and supported SDK entry point, with every selection input mapped to a real validated field or owner mechanism.
2. Trusted repository/Workspace/source-port construction and authority prerequisites, including whether the proposed successor-native binding is actually needed. Do not presume it is.
3. Supported content retrieval/materialization yielding exact admitted bytes, provenance, retention and structured refusal behavior—not only descriptor/projection metadata.
4. Start/End applicability, revalidation timing and original-request recovery, demonstrated with the actual supported operations.
5. The precise Substrate preparation/acceptance/delivery join and any required bounded owner changes, clearly separate from Handbook storage migration.

Only after this mapping is verified can the existing B04 handoff and Collider consumer mapping replace their historical operation inventory with the selected binding. Any genuinely missing capability becomes a bounded owner-contract change, not a reason to quietly weaken B04 or automatically wait for the file-native engine.

Full B04 proof still needs real execution tests: positive exact-context delivery; required missing/stale/malformed/denied refusals; wrong Lane/World; source/permission/binding drift; permitted optional omission; uncertain capture/submission/delivery recovery without duplicate work; distinct resolved/injected lineage; and no implicit provider-native continuation from a shared UI thread. Check original-input retention and recovery after restart against the advertised limits. SDK fixtures alone are insufficient. Complete Gate C additionally needs the applicable B01/B02/B03/B05 implementation and proof obligations.

Future storage work may preserve this SDK boundary, but must re-prove authorization, exact identities/bytes, original recovery, currentness, retirement/corruption distinctions and any changed public types. No future engine compatibility is certified here.

**October 18 delta validation:** the parent re-read the linked Windows final response and the complete B04 target section, mapped the recovered SDK evidence, and checked document scope/links/whitespace. The earlier three-agent review in §10 covers the original synthesis, not this new mapping. No runtime code, canonical handoff, dossier or sibling repository was changed; no implementation or integration test was run.


## 12. Windows source verification: bytes, trust and B04 closure — 2026-10-18

### 12.1 Verdict and evidence ceiling

**The existing snapshot/grounding path is not a complete selected-reference-to-context-body adapter.** This is now a source-backed limitation of that particular path, not merely an unanswered question. It provides real bounded capture, currentness/accounting, durable provenance and replay. Existing SDK Flow packet generation is a separate, content-bearing candidate that must not be overlooked. Neither finding establishes a complete authorized B04 path into Substrate.

This pass re-read recorded Windows source/patches in T15/T17 and newly inspected T20 below, plus the newer source reads in T8. Final Packet 3 source reads and later Packet 6/meta reads control over intermediate patches. Truncated tool outputs were used only for visible spans, never treated as complete files. A separate historical Mac source inspection is explicitly labeled below. The available GitNexus inventory still identifies Mac indexes, not Windows; no graph query or absence claim was made from them. No fresh Windows source execution, current full-tree audit or tests were possible through session-history reads. No new task was dispatched or existing task messaged.

### 12.2 What the trusted source path actually does

1. **Trusted host supplies the repository and source implementation.** `FacadeState::open_with_snapshot_source` accepts a `SnapshotSourcePortV1 + Send + 'static` supplied by host code, stores it under a mutex, and canonicalizes the repository root. Its source comments explicitly exclude repository/source injection from IPC. Generic Tauri invocation forwards that host-owned port through `invoke_tauri_with_snapshot_source`. The direct Rust SDK has `invoke_with_snapshot_source`; the fixed use case also takes the port explicitly. These are real injection points, not a ready-made Substrate adapter. [W1]
2. **The SDK delegates to the owner.** `CaptureOwnerBridge` binds the actual SDK repository root to the supplied exact Workspace ref/fingerprint, derives the stream, constructs the bounded request, and calls `capture_boundary`. Owner binding includes an exact retained physical root, production policy and engine-authority fingerprints. That validates/binds owner identities; it does not independently authenticate a Collider caller or authorize its claimed Substrate Workspace. The trusted host must establish that external mapping. [W2, W3]
3. **Four external families plus one owner-derived family are captured.** Evidence, Git, Handbook and Work read the supplied port. Session `boundary_context` is derived by the owner from request, stream, sequence and predecessor—not trusted from the port. The 15 fixed slots are listed below. [W4]
4. **The owner checks observation consistency and bounded payloads.** For readable observations it does observe → bounded read → observe, compares reference and observation identity, then rechecks non-Session observations before producing the capture. It checks exactly-once slot coverage, admitted payload shapes, size/safety and the canonical JSON fingerprint. These checks detect disagreement with the trusted port; they do not prove that an arbitrary port obtained a real authorized source or that permission remains valid until later execution. [W4, W5]
5. **Immutable records contain only admitted bounded values.** `capture_value` includes each slot's bounded value, reference, fingerprint and disposition; the owner canonicalizes accounting/delta/grounding into the repository transaction. The SDK capture/composition results expose record bindings. `snapshot.project` exposes grounding/Resolution/disclosure/accounting metadata, not the stored bounded-value payloads or arbitrary document bodies. [W2, W5]

| Family | Exact slots in the inspected production contract |
|---|---|
| Evidence | `contracts_definition_lifecycle`, `contracts_evidence_root`, `contracts_decision`, `dock_manifest_bundle`, `dock_execution_candidate` |
| Git | `head`, `index`, `worktree` |
| Handbook | `engine_authority`, `profile`, `schema_artifact_resolution` |
| Session | `boundary_context` — owner-derived |
| Work | `flow_state`, `pipeline_state`, `handoff` |

**Critical payload limitation:** `slot_payload_shape_is_admitted` accepts exactly `{ref}` for ordinary slots; `schema_artifact_resolution` accepts exactly `{resolution: {stack_fingerprint, stack_ref, values}}`; Work handoff accepts `{refs}`; Session has its closed owner-derived boundary fields. Adding a `contents` field to any ordinary slot is not an adapter-only solution: it fails the inspected shape check. The 16 KiB slot cap does not authorize arbitrary content up to that size. A fingerprint of `{ref}` is a fingerprint of that admitted observation, not by itself a fingerprint of the referenced document's bytes. [W5]

The recovered concrete `CompletePort` implementations populate in-memory observations/values for tests. The Packet 6 search finds those fixtures and forwarding adapters, not a demonstrated production Substrate host port. This establishes **missing demonstrated host integration**, not global proof that no later adapter exists anywhere. The real host must bind owner-backed source observations, disclosure decisions and applicable Scope/Resolution without manufacturing them from frontend input. [W6]

### 12.3 The separate existing route toward actual content

The newer Windows SDK source still exports `generate_packet` and `inspect_packet`, forwarding to `resolver::resolve_packet`; it also exports `PacketResult`, `PacketSection` and `PacketSourceSummary`. Packet 6's ordinary inventory includes `flow.resolve@1.0.0`, `artifact.read@1.0.0` and `artifact.validate@1.0.0`. Windows artifact source exposes `ArtifactDocument` and an `ArtifactSelector` containing kind ref and instance ID. These are existing capabilities to inspect before proposing a new resolver. [W7]

**Additional Windows evidence:** the HCM-4.6 source read shows distinct Flow modules for `resolver`, `packet_result`, `flow_resolve_admission`, `operation_admission` and `grounded_packet`. The visible `adopt_grounding_outcome` implementation copies typed grounding, delta, provenance, omissions and evidence from an engine outcome; it does not read canonical document bodies or call the packet resolver. Its `Ready` result means no grounding omissions, not that selected document bytes have been materialized. The SDK search also shows separate closed Flow operation request/result, section, budget, refusal and blocker types. Consequently the legacy `FlowRouteError` text is not sufficient reason to invent a new machine-error API: qualify the existing ordinary operation first. These recorded reads do not expose that operation's full decoding, admission or projection implementation. [W9, W10]

**Historical source trace, not a current Windows implementation claim:** at clean Mac `8605d653...`, the concrete chain is `generate_packet` → SDK `resolver::resolve_packet` → `handbook_flow::resolve` → `resolve_with_contract` → `CanonicalArtifacts::load_with_contract` → baseline validation/rendered projections → manifest and budget decisions → `build_packet_result` → ordered `PacketSection.contents`. SDK mapping projects owner values; it does not supply an independent semantic resolver. [W8, W11]

That historical trace establishes useful, specific qualification checks:

- `GeneratePacketRequest` supplies only a packet ID; the Flow request adds a budget policy. Neither inspected historical request expresses B04's complete exact-reference selection and caller authority. The fixed recipe's requiredness cannot silently replace the caller's requiredness.
- Rendering uses the retained artifact/rendered bytes from that resolution pass. A section is `Verbatim`, `Rendered`, or `Summary`; source identity, rendered identity and actual final delivered-byte identity are different. For `Summary`, the inspected implementation emits a **“full contents omitted” budget notice**, not a semantic summary of the source. It cannot satisfy a requirement for the full selected content.
- `included_sources` is computed even when the packet is blocked; sections are empty unless selection succeeds with no refusal/blockers. A nonempty source list, successful Rust `Result`, or `execution.live.packet` label is not proof of usable context or execution.
- The result includes source and rendered fingerprints, manifest fingerprint, readiness, refusals and budget information. No evidence here proves a coherent current Windows authorization/source cut across all reads. Fingerprints describe identity; they do not grant disclosure or freeze currentness.

The old source is a comparison checklist, **not a defect report against the newer Windows implementation**. In particular, it must not resurrect mandatory Charter/governance requirements superseded by later accepted Handbook semantics. `ArtifactSdk::validate` returning a document/fingerprint likewise does not make validation alone a substitute for Flow applicability and assembly. [W8, W11]

What is **not** verified by that old source or the current wrapper read:

- A supported mapping from the exact B04 selection (kind/revision, purpose/action, Scope/posture, requiredness) to a current packet recipe or artifact request. A packet ID or kind/instance pair is not automatically an exact content revision.
- The current Windows resolver's full content-read/render implementation, supported reference set, source/authority checks, structured failures and concurrent-change behavior. Wrapper availability is not evidence those details match the old Mac implementation.
- A supported join between Flow-produced content and the snapshot/grounding record. Reading both independently does not prove they used the same revisions or authority basis.
- A canonical runtime envelope and identity for the final ordered delivered content. Source hashes, rendered-section hashes and final envelope identity are different objects; do not infer one from another.

**Refined recommendation:** first qualify the existing Flow packet/materialization route for the selected case; use snapshot grounding only where it provides needed lifecycle/provenance. Do not try to turn `snapshot.project` into a body export or require snapshot Start/End merely to use Flow. A bounded typed adapter or versioned SDK amendment may be needed, but this pass does not justify building a new general resolver or changing storage.

**Qualification outcome:** existing route identified; B04 integration not qualified. Exact adapter obligations can be specified now (§12.8). Whether each needs SDK code, an owner extension, or merely an existing supported binding cannot be settled from wrapper and search excerpts. Attempts to recover Packet 5 with source outputs hit the thread tool's single-turn size ceiling; a metadata-only read succeeded but is not source verification. Direct Windows filesystem access remains unavailable. No current Windows inner implementation is silently inferred from the Mac checkout.

### 12.4 Lifecycle and request constraints newly identified

- `bind_session_stream` derives its identity from repository binding, Workspace, policy and stream class—not `host_session_ref`. Host session participates in boundary/request identity. Together with strict Start/End sequencing, this means different host-session strings must **not** be assumed to provide independent concurrent streams in one bound Workspace. Define a supported lifecycle/serialization mapping, or use another supported route; no private stream-key invention. [W3]
- The inspected capture owner caps idempotency key and host-session reference at **256 bytes**; slot JSON at **16 KiB**, depth 64 and 4,096 nodes; handoff refs at 32. Wider outer DTO bounds do not establish owner acceptance. The actual capability descriptor must publish effective intersection limits and refusal behavior at the pinned revision. [W5]
- Exact replay in the Packet 4 fixture reuses the same receipt with **zero new source observations or payload reads**. This is useful historical recovery, not a fresh authorization/currentness check before another launch. A current-head read refusing an old snapshot is not permission to recapture it under the original accepted request. [W6]
- `open_repository` is public under test/test-support but `pub(crate)` in the inspected production owner. Internal `read_record`/canonical bytes are not automatically a supported production SDK export. Enabling test-support to bypass that boundary is not the prototype integration plan. [W3]

### 12.5 Join to Substrate acceptance and actual delivery

This follows the existing B01 §3.2 and B03 §§4.8–4.9 target contracts; it does not claim their proposed interfaces are implemented.

| Boundary | Required concrete join and failure behavior |
|---|---|
| Original request | Authenticated ingress preserves all caller-specified context inputs in protected request equality. Resolve the existing key before new preparation; a retry must not resolve latest content to redefine accepted work. Derived Handbook evidence is linked at admission under owner rules, not substituted into caller identity after the fact. |
| Pre-submission preparation | Obtain supported admitted content and exact provenance, not just snapshot metadata. Required context failure stops submission; Handbook capture may already have committed, so this is not “no effects anywhere.” Preserve that receipt and its recovery/finalization obligations. |
| Guarded admission | WorldDispatchControl obtains and enforces authorization; HSA resolves actor/session facts; association authority reserves the exact destination revision; Handbook supplies applicable semantic/disclosure/currentness results. Bind the material and original request to the B1 proposal before uncertain effects. The cross-owner revalidation/fencing protocol remains required; a snapshot read or association reservation cannot freeze permissions. |
| Acceptance | B1/runtime evidence establishes acceptance. Complete original B2.1/E2/B2.2/B03 joins as required; missing consumer material stays `AcceptedMaterialIncomplete`, never false pre-submission refusal. Handbook receipts are not B1 acceptance receipts. |
| Delivery | Substrate packages the admitted content without changing Handbook semantics and delivers it to the exact execution. Only the selected runtime-path evidence establishes `Injected`; record the material-to-accepted-work link and distinguish not-yet-delivered/failed/uncertain outcomes. No existing runtime event was qualified as that evidence in this pass. |
| Recovery and observation | Keep Handbook receipt replay/recomposition separate from B01 submission reconciliation and B02 terminal observation. Never replay `execute_stream` to repair an uncertain context/acceptance response. Historical reads need current access authorization, and retained content/recovery horizons must be defined. Collider displays these distinctions rather than inferring them. |

### 12.6 Exact remaining B04 gaps and closure evidence

**Current disposition:** §13 supersedes the earlier uncertainty about the inspected supported adapter/owner binding. G1 source access is closed at the published pin; concrete G1–G3 implementation changes are now identified. G4–G7 remain integration/proof obligations, not claimed capabilities.

| Gap | Classification / owner | What closes it |
|---|---|---|
| G1 — Current content-bearing SDK binding | Evidence + contract mapping; Handbook | Pin current Windows `flow_api.rs`, `resolver.rs` and its actual materialization dependencies, `contract_membrane/flow_operation.rs` and applicable artifact operations/tests. Demonstrate one real required-context selection through supported inputs to content bytes, exact revisions, structured outcomes and effective limits. The snapshot-only candidate is ruled out for arbitrary bodies; the broader SDK is not ruled out. |
| G2 — Trusted external source/authority binding | Integration unproven; Handbook + Substrate host | Concrete supported construction binding the real repository/Workspace and source owners to the caller's authorized Scope/posture. If snapshot grounding is used, implement/identify the production port and prove pre-read redaction, real owner observations, source drift/refusal and host-session lifecycle. Fixtures and path normalization alone do not close it. |
| G3 — Exact resolution-to-material correspondence | Contract/integration unproven; Handbook | Bind included bytes and ordering to the selected revisions, omissions and semantic evidence; prove no latest substitution and no torn read. If combining Flow and grounding, prove their common basis rather than joining unrelated successes. Define content retention versus later disclosure authorization. |
| G4 — Pre-acceptance and delivery authority join | Existing target needs concrete owner protocol; Substrate + Handbook | Guard/revalidate source, disclosure, policy and B03 destination through the relevant effect; bind the exact material to original request/B1 proposal. Prove revoked/stale/wrong-destination refusal and unchanged accepted retry history. |
| G5 — Actual execution delivery and lineage | Implementation/proof unestablished; Substrate, presented by Collider | Supported runtime packaging/delivery hook and authoritative evidence for exact material received by exact accepted work; authenticated lineage reads distinguish pending, absent, unavailable, resolved and injected. Handle post-acceptance failure without erasing acceptance. |
| G6 — Independent recovery/lifecycle semantics | Cross-owner contract + proof; respective owners | Original capture replay/recomposition, request uncertainty, delivery uncertainty, restart/retention and any End finalization have explicit independent outcomes. Prove no recapture/relaunch on a lost acknowledgement and no implicit provider continuation. |
| G7 — Installed positive and adversarial test | Proof; all three repos | Real Linux ephemeral task consumes required admitted content; stale/missing/malformed/denied references, allowed optional omissions, wrong Lane/World, permission/source/binding races and acknowledgement loss produce the specified results. The same accepted work and provenance appear in both Collider views. Fixtures or metadata success are not completion. |

G1–G3 determine whether this is adapter-only work or requires a bounded Handbook SDK/owner amendment. G4–G6 require the existing Substrate owner design to be made concrete; G7 is the end-to-end exit. These are evidence/engineering gaps, not a new broad product interview. B05 remains an additional complete-Gate-C obligation, not a hidden B04 storage requirement. No evidence found makes the file-native engine a prerequisite.

### 12.7 Reproducible evidence locators and checks

Use the Windows host from §2.2 and the command IDs below. These are recorded source observations, not fresh Windows test execution.

| Key | Source and recovery locator |
|---|---|
| T20 | `01a04ff0-67ac-7f71-bd3e-a625fc3ffe1f` — HCM-4.6 Packet 3 — Capture and Grounding; implementation checkpoint `3c7ed255d2228da256dd71ca568bd06f9b63a7ab`. |
| W1 | T17 `exec-df17d5e5-06f3-494f-90b5-d3f3cb69bcc3`: `crates/tauri/src/lib.rs`, host construction and generic dispatch; `exec-edf46477-3b16-401b-99ef-8078bf2ac267`: SDK invocation dispatch. |
| W2 | T17 `exec-20415d09-7d2e-483f-b14c-6d5c9affb1d9`: `snapshot_operation.rs`, request/result DTOs, definition admission and `CaptureOwnerBridge`; visible source through capture handling, not the truncated entire file. |
| W3 | T15 `exec-d4fc9c1c-f69c-400b-96de-4ab4e69e0743`: owner binding and production/test repository visibility, followed by current-head operation checks. |
| W4 | T17 `exec-29f0bebd-3295-4c65-b4fe-8a782d84f435`: source contracts/slots; T20 `exec-b6a62cc2-e7db-4c3d-9726-1c091cf08211`: production capture, owner-derived Session, observation/read/recheck and fingerprint checks. |
| W5 | T20 `exec-c589ce98-c6b4-4e7e-b2d8-d429beb2db6e`: final source's constants, trait and closed payload validator; `exec-c453a648-c14c-4e60-9c4f-ad368589b1f5`: capture identity patch; `exec-02b3fa4f-8761-4015-8f7b-6cc48dcb87ea`: stored bounded values; `exec-bc33049a-fff1-4f84-a3b6-6b88691d7f92`: final accounting-error preservation delta. |
| W6 | T17 `exec-202d7689-ab3a-439e-bd7f-240c41201347`: port implementation inventory at that revision; `exec-ceaf4ee5-ed8f-4759-b353-f00ec44019e9`: complete Packet 4 test source, fixture port and replay assertions. |
| W7 | T8 latest turn `01a14f62-529b-71c0-95f7-3a9ceafe0bbd`: `exec-73285263-b9ce-4b75-93df-1758a27979fc` current SDK wrappers/exports; `exec-e2d7213b-2aa4-4584-b17b-752645479055` artifact document/selector source; T17 `exec-938e9b0f-4843-44fe-a8cb-9f0acfdd51c2` visible ordinary operation inventory. |
| W8 | Supplemental historical Mac `8605d6539355f350c55b921d95fb450bca30ee18`: `crates/sdk/src/flow_api.rs`, `resolver.rs` (request, result fields and forwarding), `artifact.rs` (validation projection). Clean HEAD rechecked. Not a replacement for current Windows source. |
| W9 | T16 `exec-16e856bd-2921-46b3-8a41-b3fa5901b698`: visible complete `crates/flow/src/grounded_packet.rs`, Flow module/export list and typed grounding test. Later portions of the combined output are truncated and are not relied on. |
| W10 | T15 `exec-8ed827f3-7596-4e34-892b-955d9b1d4d96`: recorded search results locate `FlowResolveRequestV1`, `FlowPacketResultV1`, `FlowPacketSectionV1`, refusal/blocker/budget families in `crates/sdk/src/contract_membrane/flow_operation.rs`. Matching lines prove presence, not complete fields or enforcement. |
| W11 | Historical Mac `8605d653...`: `crates/flow/src/resolver.rs`, especially `resolve_with_contract`, `build_packet_result`, `packet_artifact_disposition_for`, `packet_sections_for`, `summarize_artifact`; `packet_result.rs` and SDK `resolver.rs`. Direct source read, no test execution. |

Only this reconciliation document changes. Local links, Markdown table shape, whitespace and unchanged draft/dossier hashes are checked. No code, canonical target contract, sibling repo, source index or remote branch is modified; no commit/push or live integration test is performed. This pass is parent-reviewed source analysis, not a new independent review of §11/§12.

### 12.8 Concrete Flow adapter work and conditional SDK amendments

**Historical qualification matrix:** §13 supplies the subsequent direct-source decision. The fixed-recipe option below is not a requirement to preserve the existing SDK or narrow the product around it.

**Recommended bounded design, not yet an approved callable API:** use the existing ordinary Flow operation through Handbook's SDK for a capability-declared reference/recipe subset that genuinely satisfies the chosen experiment. Keep semantics in Handbook and delivery in Substrate. Unsupported selections refuse explicitly; narrowing the supported subset must not drop mandatory context or disguise a partial test as full B04. The names below describe responsibilities, not new operation identifiers or edits to strict V1 schemas.

| Work item | Concrete adapter responsibility | When Handbook SDK/owner changes are necessary |
|---|---|---|
| A1 — Supported selection mapping | Publish a version-pinned capability mapping: admitted reference kinds/identities, content-revision domain, purpose/action, Scope/posture, requiredness, recipe and limits. Validate every B04 input; no raw path or display-name substitution. | If the existing ordinary request cannot express or faithfully bind the selected case, add one bounded versioned SDK composition/request using existing owners. Do not put arbitrary selections into a packet-ID string, alter repository configuration to simulate a request, or invent a general resolver. |
| A2 — Trusted source and disclosure | Trusted Substrate host binds authenticated caller/Workspace to the authorized Handbook repository and applicable owner authority. Authorize the complete source closure the recipe may read before materialization, not just the requested display item. Keep destination authorization separate. | If current Flow admission cannot enforce the necessary Scope/Resolution/disclosure for that closure, expose a supported owner-backed admission binding. Post-filtering an over-broad packet cannot repair unauthorized reads or leaked diagnostics. `SnapshotSourcePortV1` is not the Flow content source interface. |
| A3 — Exact material and semantic evidence | Compare the selected fingerprints in their owner-defined domains to the actual owner result; retain the exact returned bytes, order, render mode, owner evidence, recipe/version and omissions. Require each required selection to be satisfied at its promised fidelity. Reject substitution, unexplained additions/omissions and non-ready results. | If the current result omits exact input identities, policy/Resolution basis, per-reference dispositions or correspondence to returned bytes, add a bounded owner-produced result projection. If coherent reads cannot be established by existing owner facilities, add the necessary owner read/currentness guard—not a consumer-created semantic manifest. |
| A4 — Structured outcomes and limits | Preserve the ordinary operation's typed admission, refusal, blocker and unavailable outcomes. Map them losslessly to B04 with disclosure-safe detail. Enforce owner and transport limits; no silent truncation, automatic summary or required-to-optional downgrade. | Extend SDK projection only for distinctions the existing operation cannot expose. Do not parse `FlowRouteError`/decision-log strings or assume missing files, denied disclosure, stale revisions and unavailable owners are interchangeable. Pin actual numeric limits from the current schemas/owners before implementation. |
| A5 — Durable prepared material | Substrate retains authorized material and provenance under the original preparation/request join before possible submission. Define canonical framing/encoding, media types, section order and a digest of the **actual complete envelope**, distinct from source hashes and Handbook semantic identity. | No Handbook storage rewrite is established as necessary. Add an SDK retention/retrieval capability only if the selected owner contract requires Handbook-owned historical material unavailable through an existing supported route. Consumer cache IDs must not masquerade as owner resolution receipts. |
| A6 — Revalidation, acceptance and delivery | Use §12.5's guarded B01/B03 admission and runtime delivery joins. A stale/denied new preparation refuses; an accepted retry recovers original material/work without rerunning Flow against latest. Record actual delivery separately, including post-acceptance failure/uncertainty. | Handbook must provide the supported currentness/disclosure check or evidence needed by that protocol if absent. Substrate still owns the execution fence, acceptance and injected evidence; an SDK success cannot implement those responsibilities. |

**Minimal correspondence record:** retain (a) the caller's original typed selection; (b) the owner-validated exact input/dependency and authority basis; (c) each admitted output's bytes, order, mode and omissions; (d) the Substrate envelope identity and destination expectation; and (e) the later accepted-work/delivery joins. This is a cross-owner mapping requirement, not a new global registry. Digest validation after reading does not replace authorization before reading. An adapter lock protects only participating writers; it is not proof against arbitrary external edits. An immutable revision can remain historically readable without remaining authorized for a new delivery.

**Adapter-only is acceptable only if all of A1–A4 are already expressible and enforced by the pinned supported SDK/owner path.** A1–A6 require a concrete integration implementation, but this investigation has not established that all six require Handbook source changes. Conversely, “the SDK can return strings” is insufficient to claim adapter-only feasibility. Known wrapper limitations justify the qualification gates, not a fabricated exact patch list against unseen current code.

#### Bounded remaining source qualification — not another architecture interview

Obtain a read-only Windows source snapshot at one full commit/tree identity, including relevant dirty-file hashes if any, for:

1. `crates/sdk/src/flow_api.rs`, `resolver.rs`, `contract_membrane/flow_operation.rs`, the Flow branch of `invocation_dispatch.rs`, and the exact operation request/result schemas and capability entry.
2. `crates/flow/src/resolver.rs`, `packet_result.rs`, `flow_resolve_admission.rs`, `operation_admission.rs`, `budget.rs`; follow their actual canonical read/render and authority/currentness dependencies (historical entry: `CanonicalArtifacts::load_with_contract`). Do not assume the old dependency list remains exhaustive.
3. `crates/sdk/tests/contract_flow_operation.rs` and the actual Flow resolver/admission tests and fixtures for the selected reference/recipe. Record source hashes, command, result and installed SDK/build identity; historical HCM aggregate test success is not this B04 test.

This small source closure—not all Handbook sessions or the file-native experiment—is what decides each conditional SDK amendment above. Until obtained, G1 remains partially evidenced and G2–G3 unqualified. No claim is made that a later implementation lacks an extension merely because its source was not returned by the history tool.

#### Required qualification cases and stop conditions

- **Positive:** a real supported required reference at a pinned revision resolves through the ordinary SDK operation; assert the expected material bytes/mode and complete provenance, not just a nonempty packet or `Ready` flag. Substrate then consumes that exact material on the selected installed execution path and records the accepted-work/delivery join.
- **Selection and fidelity:** unsupported kind/recipe; wrong fingerprint; missing/empty/malformed required input; optional omission under permitted policy; budget summary/exclusion of required full content; unexpected recipe-added source. None may silently become a successful smaller request.
- **Trust and races:** wrong repository/Workspace; denied source or destination; symlink/path substitution where applicable; source/authority change during assembly and between assembly/admission/delivery. Assert no unauthorized bytes or sensitive failure details escape. A collection of independently successful reads is not a coherent authority cut.
- **Recovery:** crash after preparation, lost submission acknowledgement, later source edits, and delivery failure/uncertainty after acceptance. Original request reconciliation retains original accepted identity/material; it does not rerun Flow, recapture grounding, or relaunch work to manufacture certainty.

No live qualification cases were run in this pass. **Result: a concrete adapter/change decision matrix, not full Flow qualification or B04 closure.** Complete the bounded Windows source qualification, select only demonstrated SDK amendments, then implement/prove the adapter and Substrate joins. File-native storage, a new generic resolver, snapshot body export, mandatory session Start/End and provider-session continuation remain outside this proposal.

### 12.9 Published Windows baseline fetched and inspected — 2026-10-18

The user identified that the newer source could be obtained through Git. `git fetch origin` discovered `origin/codex/file-native-baseline-379af4e` at full SHA `379af4eda9b95ee82be9d3afcfdcdca2d6d132f9`. The old tracking branch still points to `8605d653...`; simply pulling that old upstream would not advance it. Ancestry checks show old HEAD is an ancestor (0 local-only / 170 remote-only commits), and `4750aa22404ff750035250e1f9f1aed2b4ca32d7` is also an ancestor. The clean checkout was switched to a new local tracking branch, `feat/handbook-b04-source-review`, without resetting or moving the historical branch. No Handbook source was edited or pushed.

**This resolves the source-access blocker, not B04.** Earlier “historical only” and “current internals unavailable” findings above describe the preceding pass. For the following findings, source is now directly inspected at the published Windows commit:

| Finding | Exact source at `379af4e...` | Consequence |
|---|---|---|
| Ordinary Flow input is closed and recipe-only | `crates/sdk/src/contract_membrane/flow_operation.rs:28–60,1229–1273`; `crates/sdk/schemas/2020-12/handbook.operation.flow-resolve-request/1.0.0.schema.json` | Exactly three packet IDs; unknown fields forbidden. There is no body field for selected references, expected content revisions, purpose/action or caller Scope/Resolution. Passing extra B04 fields to V1 is not a compatible integration. |
| “Owner admission” here admits an operation definition | `crates/flow/src/operation_admission.rs:70–100`; `flow_resolve_admission.rs`; SDK executor above | It matches operation/owner/schema fingerprints, not the authenticated external caller's permission to read selected context. Reuse it, but do not confuse this token with B04 source/disclosure authorization. |
| Existing content route is real, not merely named | SDK executor calls `resolver::resolve_packet` (`crates/sdk/src/resolver.rs:210–229`), which calls Flow; `crates/flow/src/resolver.rs:726–954,1361–1459` | The six inspected materialization/projection files below byte-match the old baseline. The previously historical ordered sections, full-contents-omitted summary notice, source/rendered hashes and readiness findings are confirmed at this newer commit. |
| Source loading has real per-file safeguards | `crates/engine/src/canonical_artifacts.rs:207–278,440–540` | Shipped profile decisions select descriptors; reads use normalized paths, no-follow checks, trusted handles and bounded stable reads. This is stronger than arbitrary file reads, but does not itself establish an external-caller authorization binding or a single cross-file authority/currentness cut through delivery. |
| Structured ordinary outcomes already exist | `crates/sdk/src/contract_membrane/flow_operation.rs:337–520,1247–1673`; `crates/sdk/tests/contract_flow_operation.rs` | Complete result projection preserves contents, mode, hashes, budget and refusal/blocker data; typed terminal/problem/recheck mapping exists. Do not build an error-string parser or duplicate this response system. Test source covers closed requests, schema contradictions and missing repository truth; tests were read, not rerun. |

The byte comparison used `git diff 8605d653... HEAD --` for `crates/sdk/src/flow_api.rs`, `crates/sdk/src/resolver.rs`, `crates/flow/src/resolver.rs`, `crates/flow/src/packet_result.rs`, `crates/flow/src/budget.rs` and `crates/engine/src/canonical_artifacts.rs`; it returned no changes. This does not assert that every transitive dependency, profile or owner is unchanged.

**Narrowed change disposition:**

1. **Confirmed reuse:** ordinary `flow.resolve@1.0.0`, SDK content projection, typed outcomes, current render/budget logic and per-file safety mechanisms. No new generic renderer, error system or file-native engine is justified.
2. **Required integration:** A1–A6 still need an explicit adapter. For a fixed supported recipe, it may validate caller expectations outside the existing V1 request only if the complete read closure is authorized beforehand, exact returned material is checked and retained, and Handbook's actual applicable semantics are faithfully preserved. This is not permission to filter unauthorized content after reading.
3. **Required new SDK/owner seam if the selected experiment needs inputs that recipe-only V1 cannot represent:** a bounded supported composition/operation binding exact selections and their semantic/authority basis. Do not add fields to the strict existing V1 request. An SDK-level adapter may reuse existing owner functionality; source changes to the Flow resolver itself are required only where those owners cannot supply the needed semantics. This review establishes the V1 mismatch, not absence of every possible alternative composition elsewhere in Handbook.
4. **Still to close before claiming adapter-only feasibility:** the supported external authority/Scope binding; coherent exact input/authority evidence and currentness protocol; retention/recovery binding; and the actual Substrate acceptance/delivery proof. The read path's stable-file checks must be retained, not replaced with an unsupported claim that a consumer mutex freezes all writers. G1's source-access sub-gap is closed for the inspected route; G1's supported positive binding and G2–G7 remain open as applicable.

No fresh Cargo or installed integration tests were run. The Handbook checkout is clean and exactly aligned with the fetched tracking ref; no claim is made about unpublished later Windows work. The reconciliation document remains uncommitted, and the two target drafts and dossiers are unchanged.


## 13. Supported adapter/owner binding qualification — 2026-10-18

### 13.1 Decision and evidence boundary

**Source-qualified result: reuse Handbook's native mechanisms, but amend the supported SDK/owner composition; do not claim an unchanged SDK adapter satisfies B04.** The user explicitly clarified that SDK/contracts were still evolving before the file-native investigation. The intended B04 behavior controls this design, not incidental limitations of `flow.resolve@1.0.0`. A versioned bounded composition is the recommended change vehicle; there is no instruction to permanently preserve every old interface or build a parallel permission platform. Existing strict requests must not silently acquire new semantics before an explicit contract change.

Pin: Handbook `379af4eda9b95ee82be9d3afcfdcdca2d6d132f9`, local `feat/handbook-b04-source-review`, published as `origin/codex/file-native-baseline-379af4e`. Origin still does not advertise `feat/handbook-contract-membrane-clean`. The user subsequently supplied Windows `git rev-parse HEAD`: `379af4eda9b95ee82be9d3afcfdcdca2d6d132f9`, matching the freshly rechecked Mac HEAD exactly. Combined with the supplied clean tracked-file status, this closes the reported committed-source parity gap despite different branch names. The listed untracked Windows research is still outside this snapshot; commit equality does not establish identical untracked files, dependencies, installed binaries or runtime behavior.

The current [coverage reconciliation](../../../handbook/docs/specs/handbook-contract-membrane/coverage-and-landing-map.md#joint-authority-and-recovery-reconciliation) and [October 8 investigation](../../../handbook/docs/research/context-resolution-local-advisory-extension-investigation-2026-10-08.md) agree: reusable local Context Resolution root binding is proposed, not adopted. A fresh read of T3's latest turn `01a11cef-adb5-7002-919c-4994520b5be3` recovers the human's authorization to **investigate and possibly adopt**, and the returned explicit non-adoption status. T7's latest turns concern storage experiments, not adoption of this authority extension. Current source diff from `2740e04` to the fetched pin leaves `local_setup.rs`, `context_resolution.rs` and `projection.rs` unchanged; `local_authority.rs` only adds the work-registration module declaration. Work registration's newer authority does not grant context use.

### 13.2 What exists, and why it is not yet the full binding

All source locators below are relative to the pinned Handbook repository, not a hypothetical new API.

| Supported capability / owner | Verified source | B04 disposition |
|---|---|---|
| Ordinary recipe materialization and typed result projection | §12.9; `crates/sdk/src/contract_membrane/flow_operation.rs`, SDK/Flow resolvers | Reuse contents, ordered sections, source/render identities and typed outcomes. Operation-definition admission is not authenticated caller/source authorization. Three closed recipe IDs cannot represent the complete selected-reference, expected-revision and semantic authority request. |
| Initial local root-envelope admission | `crates/sdk/src/local_setup.rs:395–401,469–490`; `crates/engine/src/context_resolution.rs:1757–1862` | A real public SDK path: `admit_local_envelope_request`. It requires positively current initial terms and current scoped deferral, exact approved level/constraints, no parent or escalation. It admits an envelope; it does not itself materialize selected context bytes. |
| Currentness and successor refusal | `crates/engine/src/local_authority.rs:783–816,935–954`; `local_authority/local_check.rs:520–565` | Initial witnesses intentionally refuse `successor_terms_selected`. Local-check guarded callbacks and current D/G basis patterns are reusable, but their existing action permissions are not reusable Context Resolution permission. Never delete the refusal, rerun initial setup, or pass a saved locator as authorization. |
| Native semantic projection | `crates/engine/src/context_resolution.rs:1966–2029`; `crates/engine/src/projection.rs`; `crates/engine/src/grounding.rs` | A current envelope can produce an internal projection authority view. That view and projection machinery are not a public arbitrary-document SDK materializer. Grounding uses exact snapshot/delta inputs; it does not turn snapshot metadata into selected document bodies. Wrap the needed owner behavior, not private/test-support access. |
| Artifact-owned content reads and validation | `crates/sdk/src/artifact.rs` (`ArtifactSelector`, `ArtifactDocument`, `read`, `validate`); `crates/engine/src/artifact_repository.rs:430–468,707–745` | Supported typed document and owner fingerprints exist. Selector identifies kind/instance, not expected content revision. Parsed `ArtifactDocument` is not the original raw bytes; serialization needs its own declared representation and digest. Artifact validation alone does not establish context applicability/disclosure or all B04 authority. |
| Guarded artifact read with recovery | `crates/engine/src/artifact_lineage_store.rs:2662–2696` | Read acquires owner locks, validates inventory and invokes persisted-transaction recovery before evaluation. Do not advertise an unrestricted side-effect-free read or “no Handbook effects before execution acceptance.” Reuse the owner's recovery semantics; don't substitute raw filesystem reads. |
| Selected authority/assurance rules | [Approval amendment](../../../handbook/docs/specs/handbook-contract-membrane/semantic-model/amendments/approval-authority-and-assurance.md), [objects/governance](../../../handbook/docs/specs/handbook-contract-membrane/semantic-model/objects-and-governance.md) | Local-advisory assertions are not authenticated identity or verified delegation. Substrate's authenticated caller and enforced host permissions remain separate. Not every governance family is mandatory; require exactly the applicable, positively resolved basis. Host agent profiles and Engine operating profiles remain distinct. |

### 13.3 Concrete recommended integration path

This is a **target composition**, not a claim of an existing callable end-to-end API:

1. **Collider selection → Substrate ingress:** preserve the original typed references, expected content revisions, requiredness, purpose/action, Scope/Resolution posture and destination expectation in original request equality. Authenticate the caller and authorize the operation. Resolve an existing accepted request before doing new materialization.
2. **Trusted host → Handbook repository/authority binding:** select the authorized Handbook repository from trusted Workspace configuration, not a caller-supplied path. Bind its actual owner identity/root to the allowed source closure and the exact applicable authority lineage. Keep authenticated host identity and local-advisory principal assertions separately attributable; neither upgrades the other's assurance.
3. **Handbook native admission → exact source material:** consume current applicable Context Resolution authority, validate the allowed source closure before reading, resolve exact owner inputs and apply semantic/disclosure rules inward. Read once into retained owner-validated inputs and render from those inputs. Do not authorize an artifact read and then let Flow independently reread a broader shipped recipe. No new Engine→Contracts dependency: resolve applicable outer governance at the supported SDK boundary and validate its exact basis inward using existing patterns.
4. **Handbook result → Substrate prepared envelope:** return exact source/dependency identities, semantic/authority basis, ordered content and representation, per-reference disposition and permitted omissions. Substrate frames and retains the actual execution envelope with a distinct digest. Required context failures stop new submission; budget summaries are acceptable only when the requested fidelity expressly permits them.
5. **Guarded B01/B03 acceptance → actual runtime delivery:** execute §12.5's cross-owner protocol with the retained material. A prior successful check is not a live permission grant. Define the effect's validation/linearization point, authority generations, revocation ordering and crash recovery; revalidate or fence through that point without assuming owner locks can safely be held across transport/runtime calls. Actual runtime delivery evidence, not a Handbook success, establishes `Injected` for the exact accepted work.
6. **Recovery/history:** recover the original prepared/accepted material and owner evidence after uncertain acknowledgement; never rerun latest Flow to redefine it. Historical retrieval requires current read authorization but does not re-authorize a new execution. Source edits, revocation or a delivery failure must not erase an already-established acceptance.

### 13.4 Exact change responsibilities and closure tests

Names identify responsibilities, not selected DTO spellings or another acceptance registry. H1–H3 are the concrete Handbook changes required for this recommended general B04 composition; H4 is required for the recommended continuing-local-project scope, not for every possible initial-only demonstration.

| ID / responsible boundary | Required change, reusable base and explicit limit | Closure evidence |
|---|---|---|
| H1 — Handbook SDK supported composition | Add or explicitly revise a typed materialization request binding selected owner refs plus expected content revisions, purpose/action, Scope/Resolution and requiredness to a trusted repository/authority context. Publish supported kinds/representations and effective limits. Use SDK composition and existing Flow/artifact/native owners; do not encode missing fields inside packet IDs or simulate requests by changing project config. Candidate boundary: `crates/sdk/src/flow_api.rs`, `contract_membrane/flow_operation.rs`, `artifact.rs`, or one narrowly owned new SDK module, chosen during owner design. | A normal external SDK dependency, without test-support/private entry points, requests one required exact reference and receives expected content plus typed stale/unsupported/denied/optional outcomes. Supported subset is explicit and satisfies every requirement of the selected experiment. |
| H2 — Handbook Engine admission and exact material source | Join current semantic/disclosure authority with the actual complete read/render closure. Reuse retained roots, safe bounded reads, artifact owner locks/recovery and Context Resolution/projection checks. Extend owner entry points where expected content revisions, coherent dependency cuts and protected consumption are not expressible. Flow must consume admitted retained inputs or an equivalent guarded owner result, not independently select/reread a different baseline. Candidate owners: `context_resolution.rs`, `projection.rs`, `artifact_repository.rs`, `canonical_artifacts.rs`, Flow `resolver.rs`. | Wrong repository, unapproved source closure, path substitution, stale required ref and concurrent authority/source changes refuse with disclosure-safe results. Exact source and rendered byte correspondence survives multi-input races. A consumer mutex or a post-read digest comparison alone cannot pass this test. |
| H3 — Handbook result/current-use contract | Project the owner-validated input/dependency and authority basis, exact selected dispositions, retained material identities, representation/fidelity and complete typed failures. Expose a supported current-use revalidation/admission binding needed by S1; distinguish historical resolution from permission now. Reuse ordinary structured outcomes and per-file hashes; extend missing B04 distinctions rather than parsing log strings. Persistence can reuse owner facilities or authorized Substrate retention; a new Handbook content database is not established as necessary. | Result completeness/strict decoding, required full-content budget refusal, explicitly permitted omission, source-vs-render-vs-envelope identity, reopened history and revoked new-use tests. No currentness token is accepted outside its owner-defined scope/lifetime. |
| H4 — Handbook continuing-project local authority | Adopt and implement the bounded successor-aware native root-envelope route recommended by October 8 research, using current local lineage and positively resolved supported D/G states. Separate binding management from root use; initial human establishment, delegated replacement only under prior exact permission, independent revocation/history and original-request recovery. Preserve initial refusal and credential-required legacy behavior. Candidate owners: `local_authority.rs` and its local-check mechanisms, `context_resolution.rs`, typed SDK local authority surface. Full child/escalation/memory grant administration is not required. | Reopen a continuing local-check project, establish/use an explicitly authorized root binding, refuse unauthorized replacement and revoked/stale use, recover original outcomes without fresh effects. Existing evaluation, correction or work-registration permissions cannot authorize this operation. |
| S1 — Substrate acceptance and delivery adapter | Implement trusted repository/caller binding, original-request material retention, live admission protocol with H3, exact B01/B03 owner joins and runtime delivery evidence. Keep existing Receipt Registry/Supervisor/HSA responsibilities. A Handbook receipt is not execution acceptance and content preparation is not injection. | Positive installed Linux task consumes the exact material; lost preparation/submission/delivery acknowledgements, revocation and destination races preserve truthful accepted/pending/failed/uncertain outcomes and immutable original bindings. |
| C1 — Collider workflow construction and consumer mapping | Build the bounded interaction/projection surfaces; they are not an existing product UI (§15). Use the supported selections/capabilities and show semantic refusal, preparation, acceptance and actual delivery separately. Carry owner-qualified references, not inferred filenames or regenerated context. | Both consumer views identify the same accepted work/material; no display-only `Injected`, silent optional downgrade, latest substitution or automatic provider continuation. |

**H4 scope choice remains proposed:** recommend continuing local projects, not a fresh-setup-only workaround. An explicitly agreed initial-only prototype could use the existing initial admission and omit H4, but would still need H1–H3/S1/C1 and must declare that limitation. Existing SDK-first approval does not itself select the new Context Resolution management rule or grant a runtime principal permission. Preserve legacy credential paths; no automatic fallback or mandatory credential acquisition for the proposed local route. The research's reusable binding lifecycle is the recommendation, not a universal B04 prerequisite or a pretext to finish all Handbook architecture.

**Real-owner order (Collider fixture work is separately scoped in §15):** select that bounded authority scope; design H1–H3 together with S1's exact admission/effect protocol; implement H4 where the selected scope requires it; then run SDK and installed integration proofs. H1–H4 can share existing owners without a separate general authority subsystem. B01/B03 and B02 remain necessary execution/observation contracts; B05 remains a separate complete-Gate-C requirement. File-native storage and complete Work Progression are not prerequisites established by this investigation.

### 13.5 Unpublished research and verification limits

The user-listed governance/defaults/approval, independent-local-check, HR-007 and replaceable-defaults consultation files are **absent in this Mac snapshot**. Their accepted effects are partly recoverable in the tracked semantic amendments, coverage reconciliation and Windows sessions; the raw consultations themselves have not been inspected here. Prioritize those exact files if they contain later human corrections. The missing work-state/registration consultations and original-Python-pipeline research can inform composition, but no evidence here makes obtaining all storage/comparator/platform experiment directories a prerequisite for B04. Do not claim untracked equals unimportant or unapproved. No Windows research was staged, committed, cleaned or published. A bounded read-only transfer is sufficient; committing is not required merely to review it.

Verification in this pass:

- Direct pinned source, meaningful test bodies, accepted amendments and current coverage/research inspected; T3/T7 session endpoints rechecked. No global claim about unseen Windows changes.
- Handbook GitNexus index was stale. `npx --no-install gitnexus analyze --index-only` completed successfully (42,267 nodes / 88,385 edges); focused query/context then recovered the actual local envelope/projection symbols. Direct source remains controlling; graph absence is not proof of missing functionality. Index-only refresh changed no tracked source or agent instructions.
- Attempted `cargo test -p handbook-sdk --test contract_flow_operation --test local_initial_setup --locked`. **Build blocked before tests:** `handbook-dock` panics because no frozen production manifest exists for `aarch64-apple-darwin` (`crates/dock/build.rs:9`). The pinned tree contains only the Windows MSVC manifest. No fabricated manifest, target spoofing or test-support bypass was used. Run the focused tests on the supported Windows build; installed Linux delivery still needs its real supported build/proof path. This build limitation is not a failed B04 behavioral assertion.
- No runtime/SDK code or canonical target contracts changed. Only this existing reconciliation document was updated; whitespace, local links/table shape and the protected draft/dossier hashes were checked. No new independent review, commit, push or installed integration proof is claimed.

**Exit of this investigation:** the supported capabilities and specific SDK/owner changes are identified at the published source pin. Full B04 remains open until the selected composition and authority protocol are implemented and the real delivery/recovery tests pass. Remaining uncertainty is unpublished research/corrections plus the explicit H4 scope selection; Windows/Mac committed-source parity is now confirmed—not whether the current recipe-only SDK should constrain the product.

## 14. Collider-to-owner end-to-end flow reconciliation — 2026-10-18

**Disposition:** documentation-only reconciliation requested by the user. The missing connecting workflow is partly **integration glue** and partly **real owner capability/contract work**. Do not call all of it a new Handbook resolver, all of it a Collider controller, or all of it transport. No implementation, owner adoption, new experiment, or fourth coordinating service is authorized.

### 14.1 Collider sources and what they already answer

Collider was rechecked at `a491f98724e539a3d7a23aa0661b6b99af79c657`, `spike/agentkit-controller-correction`; only the pre-existing bundle ZIP is untracked. Substrate remains at `aa5ef73ae087aa82d1f952ba6dd5c92361b8e24f`; Handbook remains at the Windows-matched `379af4e...`. The following are current local document reads, not new installed runtime tests or a refresh of Linux implementation truth. Older source pins inside these plans remain historical.

| Source | Controlling contribution to this flow |
|---|---|
| [Collider foundations §§3, 10, 11, 19](../../../collider/docs/agent-native/canonical/01-collider-v1-foundations.md) | Collider initiates owner actions and owns presentation/interaction. Handbook owns semantic resolution; Substrate owns runtime assembly/delivery. Work survives presentation closure. The downstream Execution and Binding specification is expressly intended to describe routing, binding, reconnection and provenance; this layer was anticipated, not absent from the product model. |
| [Glossary §§6, 8](../../../collider/docs/agent-native/canonical/00-collider-v1-glossary.md) | A local intention is not an accepted Invocation. Resolved Set, Context Pack, Injected, Acquired, Produced and Context Record are different concepts. A Workstream/view is not execution identity, permission or provider continuity. |
| [Consumer boundary plan, current B01–B05 mapping](../../../collider/docs/agent-native/controller-ownership-plan/substrate-accepted-work-boundary-plan.md#current-target-mapping-and-remaining-bindings--2026-09-22) | One Linux ephemeral World task in Control Plane/Lane views, plus the separate dependency declaration operation. The access boundary exposes existing owners; it is not another scheduler, acceptance registry, resolver or provider backend. Concrete context and execution bindings remain open. |
| [Controller/owner lifecycle contract](../../../collider/docs/agent-native/controller-ownership-plan/controller-owner-lifecycle-contract.md) | One shared behavioral controller, distinct intent/request/owner-work/observer/cancellation states; no Running from a returned ID; exact original-request reconciliation; no implicit queue; owner terminal is separate from protocol segment settlement. |
| [Packet A implementation record](../../../collider/docs/agent-native/controller-correction/implementation-record.md) | Maintained AgentKit 0.2.3 correction already exists in the isolated proof. Reuse its verified client direction rather than restart it. Local cancellation generation is not durable owner idempotency; local cancellation state is intentionally not restored into a new process. Fixture evidence does not supply the real owner bridge. |
| [Packet 2 identity/lifecycle map](../../../collider/docs/agent-native/packet-2/agentkit-owner-identity-lifecycle-map.md#context-and-queue), [scenario/evidence matrix](../../../collider/docs/agent-native/packet-2/proof-scenario-and-evidence-matrix.md) | Extract current intent and explicit references, not the accumulated transcript. Distinguish natural-language intent, raw shell, structured owner action and presentation action. Preserve exact source joins across two views; no direct provider handle, toolbox impersonation or duplicate dispatch on reopening. |
| [Substrate handoff §§3, 4, 6, 8](../contracts/draft/collider-b01-b05-handoff.md), [B03 §§4.8–4.9](../contracts/draft/lane-world-binding.md) | Proposed dispatch-owned durable request correlation, exact authorization/destination/acceptance joins, separately authorized owner recovery and read-only reconciliation. These are substantive owner changes, not an already implemented generic facade. |

**Consequence:** put product-interaction coordination in Collider's application/controller boundary and execution coordination in Substrate's existing orchestration/dispatch boundary. Keep semantic decisions inside Handbook. Physical placement (direct Rust SDK, IPC or other supported bridge) remains an engineering selection; repository/process placement does not transfer authority. A Tauri command can carry a request without becoming its durable execution owner.

### 14.2 Representative scenario and scope

Illustrative user interaction: from a Work Item or other supported semantic target, in a Lane view, the user says **“Implement this change here.”** The chosen target and view provide request inputs, not permission. The exact Handbook reference kind and installed Linux runtime profile still require the named owner binding; this example does not assert that arbitrary Work Item-to-packet resolution already exists.

For the bounded proof, resolve to **one existing Lane/World and one fresh ephemeral task**, with genuinely required Handbook context. Control Plane and Lane display the same accepted work. A lane orchestrator, worker fleet, automatic queue, approval continuation, provider-session resume, merge or file-native database is not a prerequisite. The broader confirmed product can route intent through host-bound Workspace/Lane orchestrators, but a reasoning/decomposition turn and the eventual World task must not be collapsed into one acceptance identity. Collider's Control Plane view is not that host orchestrator.

The human need not manually enumerate every contextual document. Preserve two different kinds of input:

- **Explicit selected reference:** carry the exact revision obtained through the authorized owner read/selection. If it is stale or unauthorized for the requested use, refuse or request an explicit refreshed selection; never silently substitute latest under the same protected request.
- **Owner-derived applicable material:** Handbook determines the permitted dependency/context closure from the supported target, purpose, Scope and Resolution. Record its exact resolved revisions and basis as derived preparation evidence before possible submission. It is not permission for Collider to discover and interpret documents itself.

The current B04 draft already requires exact selected references; this reconciliation does **not** add an unversioned “latest” selector to it. A UI may obtain exact target references during browsing/preparation without making users type hashes. If the desired automatic selection cannot be expressed through the selected supported owner operation, that is an explicit Handbook composition/contract change, not hidden adapter logic. Freeze caller inputs separately from owner-derived material; never replace original request equality with a hash of whichever content happens to resolve later. Any pre-submission re-preparation must follow an explicit owner transition/fencing rule; accepted or submission-uncertain work never re-resolves latest to manufacture a new original result.

### 14.3 One complete flow, with failure and ownership boundaries

This is a recommended composition of the existing targets, not callable API names or a newly approved schema. Steps describe logical obligations; they do not prescribe an unsupported cross-owner transaction or global lock.

| Stage | Responsible execution of the step | Retained fact / truthful user outcome |
|---|---|---|
| 1. Capture intent | **Collider application/controller:** identify current text/action, selected exact target refs, purpose, Scope/posture and requested destination. Use the Lane view default only as a new-request convenience; Control Plane has no implicit Lane. Pure presentation actions stay local. | Local intent/request key plus explicit inputs; show Draft/Pending, not accepted work. Distinct identical prompts have distinct keys. Do not send the full transcript or auto-resume a provider session. |
| 2. Normalize and cross ingress | **Collider adapter** maps its model to the supported owner request; **Substrate ingress** authenticates principal/delegation, validates version/inputs and authorizes request access. Look up an existing original request before new preparation. | Original caller-scoped key and protected inputs. Collider may persist its pending join if restart recovery is promised, but cannot mint owner acceptance or infer refusal from a transport timeout. |
| 3. Coordinate preparation | **Substrate dispatch/orchestration** obtains HSA actor facts, policy decisions and the real Lane/World association/custody through their owners. It binds the authorized Handbook repository and requests the supported semantic resolution. | Proposed request/preparation and destination evidence, not acceptance. References, lane defaults, an agent profile or a coordinator role grant no permission by themselves. Unsupported routes stop; no raw-shell/provider fallback. |
| 4. Resolve and materialize | **Handbook SDK composing native owners:** current authority, exact selected refs, semantic applicability, allowed source closure, disclosure, content fidelity and requiredness. | Owner-produced exact resolved material/basis and permitted omissions, or typed refusal. Required failure stops new task submission. An artifact read/validation or packet success alone does not satisfy all these propositions. Existing owner recovery may have effects before task submission; don't promise zero effects anywhere. |
| 5. Assemble and prepare durable joins | **Substrate:** add authorized runtime instructions and execution state without reinterpreting Handbook semantics; retain the exact final envelope and resolution-to-envelope linkage. Prepare the original request/B1 proposal/association/actor/policy/custody joins. | Prepared, not Injected or Running. Source bytes, rendered bytes and final envelope have distinct identities. Define actual retention/access and crash recovery; a frontend cache is not owner custody. |
| 6. Admit and submit | **Substrate dispatch plus policy/runtime owners**, with Handbook's supported current-use checks: enforce current authorization/source/destination rules at the defined effect boundary and fence stale senders. | Before-effect refusal is distinct from submission uncertainty. After a send may have occurred, retain original correlation/custody and reconcile evidence; don't launch again because the ACK is missing. A last-minute check without a valid ordering/fencing protocol is insufficient. |
| 7. Establish acceptance and delivery facts | **Existing B1/receipt, Supervisor, E2/B2.2 and association owners** establish/join their exact facts; **Substrate runtime path** supplies actual context delivery evidence. | Acceptance may precede complete consumer receipt composition. Show acceptance-reported/material-incomplete where necessary; do not invent an actionable receipt. Record `Injected` only from exact runtime evidence. Delivery failure after acceptance cannot erase acceptance. Do not assume a universal ordering of all runtime observations beyond the chosen owner's guarantees. |
| 8. Observe and project | **Substrate B02** supplies authorized exact-work read/observation; **Collider's adapter/shared controller** validates/mapping-version-binds events, maintains projection checkpoints and renders CP/Lane views. | Accepted, Running and terminal only from corresponding facts; gaps/expiry/interruption remain visible. A view's sequence and Activity Item ID are not owner cursor/work identity. Closing a view detaches observation; it does not cancel or redispatch. |
| 9. Report result and provenance | **Supervisor/runtime** establish the terminal result and supported Acquired/Produced records; **Collider** shows output and links to the exact work, historical binding and context evidence. | Completion is not automatic Handbook Evidence admission, Work Item completion, merge or environment realization. Any such later action goes through its owner. Missing acquired/produced telemetry is unavailable, not a fabricated empty record. Nonstandard terminal classes retain their meaning even if protocol settlement needs an additional mapping. |
| 10. Reconnect or cancel | **Collider** reconciles the original request/work and restores supported observations; **authorized Substrate recovery** repairs owner progress separately. Explicit cancellation uses B02's exact target/key and owner checks. | Read-only reconciliation is not a repair writer or retry launch. Cancel ACK is not Cancelled; completion may win. An unreachable owner means uncertainty, not terminality. Historical accepted material survives current-policy changes, subject to current read access; it is not permission to start new work. |

**Minimum inspectable join:** Collider intent/request → authenticated Substrate correlation → exact Handbook resolved basis/material → final runtime-envelope identity → original B1 acceptance and historical Lane/World binding → delivery evidence → owner observations/terminal → Collider projection checkpoint/view. Some links are prepared before acceptance and completed afterward; one missing link must remain explicit. Use authorized redacted references, not leaked credentials/private capability tokens or duplicated sensitive payloads in UI history.

### 14.4 Integration glue versus substantive owner changes

The following is a responsibility classification, not a complete UI estimate: Collider's application is a shell, and the bounded workflow itself must be built (§15). “Glue” below still requires implementation and tests; it means mapping/coordinating supported contracts without adding semantic, permission or execution authority. An adapter cannot implement a guarantee absent from the participating owners.

| Item | Classification and repository | What is actually needed; relation to §13 |
|---|---|---|
| Intent/default/selection mapping | **Integration glue — Collider** | One application-level mapper from Composer/Surface to explicit original request. Carry exact owner refs and view defaults, expose unsupported capabilities, preserve local pending identity. Do not create another behavioral controller or model-context resolver. Refines C1; not a new Handbook owner merely because the UI wants convenient defaults. |
| Transport and response mapping | **Integration glue — Collider + Substrate exposure** | Versioned supported transport, structured result decoding and owner-to-controller/projection mapping. Reuse Packet A; selected profiles, auth bootstrap, limits and unusual-terminal settlement still need concrete bindings/tests. Codec/IPC work alone does not close B01/B02. No protocol technology is selected here. |
| Calling owners in the right order | **Integration glue — Substrate composition** | Invoke Handbook and execution owners, translate only representations, preserve their references and surface failures. The durable guards, request records and recovery guarantees this coordination needs belong to the substantive rows below, not a new orchestration service. |
| Exact typed context materialization | **SDK/owner composition change — Handbook** | H1–H3: expose the bounded supported reference/purpose/Scope/authority-to-material path; enforce source/disclosure/revision correspondence and return complete evidence/current-use outcomes. Existing recipe resolution, artifact read, native authority and projection pieces are reusable, but an external mapper cannot supply missing owner semantics. Do not require a general arbitrary-reference resolver if the selected case only needs a bounded supported composition. |
| Continuing-project Context Resolution permission | **Conditional native-owner change — Handbook** | H4 remains needed for the recommended continuing-local-project scope because initial authority correctly refuses successor state. Not inherently required just because Collider initiates a request. An explicitly selected initial-only proof may omit it; that must not be silently substituted for the intended workflow. Management permission, root use and previously granted local-check/work-registration permission stay distinct. |
| Authenticated request/destination/acceptance recovery | **Substantive owner extension — Substrate** | Proposed B01/B03 ingress/delegation, durable caller-key/proposal joins, Lane association admission and stale-sender/recovery protocol. Existing B1/HSA/Supervisor/runtime foundations do not make these consumer guarantees automatic. Preserve existing authorities rather than duplicating them in Collider or a facade. This is a contract/source-bound requirement from the drafts, not a fresh claim about every Linux branch. |
| Context custody and delivery proof | **Substantive integration/owner work — Substrate, with Handbook** | S1: retained exact material, authorized historical read, Handbook current-use/permission join, runtime delivery hook and authoritative provenance. Reuse existing persistence/runtime facilities where sufficient; amend their contracts if necessary. Provider acceptance, task acceptance and actual model-visible context must not be conflated. Exact implementation extent requires the selected runtime path, not another frontend patch. |
| Observation, cancel and restart guarantees | **Owner exposure/extension — Substrate; mapping glue — Collider** | B02 needs the supported read cut/cursor/retention/auth/cancel-key binding; Collider maps it truthfully and preserves request/observer state separately. Existing raw runtime events or client generation guards do not provide durable owner reconciliation. |
| Result presentation and CP/Lane correlation | **Integration glue — Collider** | One shared work mapping and command owner with view-local projections; preserve lineage and current versus historical bindings. No new canonical history store, mandatory full Inspector, auto-promotion, or UI-owned execution lifetime. |
| Workspace requirement editor | **Separate B05 owner work — Substrate; consumer glue — Collider** | Preserve the real requirement read/add/remove, all-writer revision/CAS/recovery and authoritative readback obligations. It is a companion non-chat proof, not a mandatory step in every context request and not an implicit install/provision operation. |

**Refinement of §13:** H1–H3 remain a supported composition gap, not a mandate to invent all-new semantic mechanisms. H4 is conditional on chosen authority scope. S1 must be split into ordinary mapping and durable owner behavior as above. Collider is more than a passive renderer—it owns interaction, request construction and projection coordination—but it does not take over semantic resolution or background execution. No extra repository/service is justified by this flow.

### 14.5 Decisions resolved here, open bindings, and proof obligations

**Resolved at the responsibility level:** user-visible orchestration glue belongs partly in Collider; durable execution coordination belongs in Substrate; context meaning remains in Handbook. Users need not type revision hashes, and selecting a Work Item is not a requirement to manually enumerate its whole context. This interpretation follows existing Collider foundations and the confirmed Substrate host-orchestrator model; physical deployment and wire formats remain unselected. It does not approve an implemented interface or globally ratify the new Handbook authority proposal.

**Concrete next contract decisions, not a reopened product interview:**

1. Name one supported semantic target/reference kind and its authorized exact-read/selection mapping, plus the owner-derived context closure and required fidelity for the existing proof. Do not assume the illustrative Work Item already maps to a supported Flow recipe.
2. Select the applicable initial versus continuing-project authority population. Recommended continuing-project root binding still requires H4's explicit owner-direction selection; neither SDK-first preference nor UI placement supplies it.
3. Bind Collider's original request and derived preparation evidence to the actual Handbook operation and Substrate B01/B03 protocol. Define pre-submission refresh, source/authority/destination race handling and the delivery linearization point; do not weaken immutable accepted retry semantics.
4. Select the real supported runtime/ingress and exact delivery/observation evidence. The September Linux proof/security gates remain controlling until freshly verified; this Mac document pass does not resume E3-F, approve copied credentials, or prove mediation of agent side effects.

**End-to-end proof checklist:** one positive required-context task; stale explicit pin; owner-derived dependency drift; missing/denied required input and permitted optional omission; wrong destination or revoked permission; lost ACK before/after acceptance; delivery failure after acceptance; window close/reconnect with zero redispatch; exact cancellation/completion race; and the same work/result/context identity in CP and Lane. Pair the existing Packet 2 P01–P10/P15/P20–P21 cases with Handbook byte/provenance assertions, not just a nonempty packet. P18–P19/B05 remain additional complete-Gate-C obligations. Approvals, queue and continuation stay deferred unless separately selected. Source interpretation, fixture client success and a real owner-integrated installed proof are distinct evidence levels.

**Validation of the preceding flow update:** only this reconciliation document changed. Relevant Collider documents and the four Substrate draft/dossier baselines were hash-guarded; repository HEAD/index/status were rechecked, as were local links, section anchors, table shapes and whitespace. No source edits, dependency installation, test execution, independent subagent review, commit or push occurred. The document remains untracked/uncommitted. Current-source Handbook findings are from §13; this pass does not upgrade historical Substrate/Collider evidence into new runtime proof.

## 15. Corrected readiness and resumption of the existing proof — updated 2026-10-19

**Disposition:** resume the staged proof already prescribed by Collider
[C-OQ-027](../../../collider/docs/agent-native/canonical/02-collider-v1-open-architecture-questions.md#c-oq-027--frontend-foundation-and-reuse-strategy),
[open register §5.1](../../../collider/docs/agent-native/canonical/02-collider-v1-open-architecture-questions.md#51-current-frontend-evaluation-sequence)
and [proof specification §§4, 9–11](../../../collider/docs/agent-native/proof/agentkit-owner-boundary-proof-spec.md).
The earlier framing of fixture UI versus real-owner execution as a new strategy
choice is corrected here. Newer human direction selects a product-shaped fixture
Workspace milestone in place of a standalone F1 harness increment (§15.2).
Its hosting/reuse choices and implementation authorization remain separate.
This documentation update does not authorize its implementation,
select a frontend, close C-OQ-027, approve H4 or weaken the SDK-first/full-B04 real-path
condition. Packet A (controller correction) is distinct from Gate A (owner contract
inventory), and neither is complete Gate B or Gate C by name.

### 15.1 Readiness: retain assets, extend missing semantics, preserve proof limits

Current inspection used Collider `a491f98724e539a3d7a23aa0661b6b99af79c657`,
Substrate `aa5ef73ae087aa82d1f952ba6dd5c92361b8e24f` and the uncommitted reconciliation
baseline. The full canonical glossary, Foundations and open register, proof specification,
controller correction/lifecycle/boundary plans and actual corrected proof source/test
bodies were read. The consult [README](2026-10-18-collider-proof-continuation-pro-consult/README.md)
and [response](2026-10-18-collider-proof-continuation-pro-consult/response.md) are
advice; they do not establish coverage, owner truth or implementation authority.

| Surface | Current source and retained evidence | Required work / evidence limit |
|---|---|---|
| Main application | [Home](../../../collider/src/app/page.tsx) renders background and “Collider”; layout is basic, root scripts still use Next. | Product UI, main-app integration/migration and native-host proof remain work. Do not estimate isolated-proof extension as building all product UI. |
| Corrected client | [Packet A](../../../collider/docs/agent-native/controller-correction/implementation-record.md) implements the maintained AgentKit 0.2.3 correction/profile with Toolkit 0.20.3 unchanged. [Results](../../../collider/docs/agent-native/controller-correction/regression-results.md) report 80 Node tests, two invariants, 14 browser executions and artifact/type/build/clean-install checks. | Retain the patch, profile and reproducibility/invariant checks. These are historical runs, not fresh passes; local cancellation state is not durable owner reconciliation. Do not restart correction or claim ordinary upstream interchangeability. |
| Composer and shared views | [App](../../../collider/proofs/agentkit-controller-correction/ui/App.jsx), [runtime](../../../collider/proofs/agentkit-controller-correction/ui/runtime.mjs), [WorkViews](../../../collider/proofs/agentkit-controller-correction/ui/WorkViews.jsx) and [browser assertions](../../../collider/proofs/agentkit-controller-correction/ui-tests/composition.spec.mjs) contain current intent/Lane/posture controls, one injected client, CP/L1/tab, detach/reattach, inline anchors, view-local Sticky and keyboard/return focus. | Reuse these assets. Non-agent submits only call setPlacement('tab'); the zero-dispatch browser check does not prove structured owner-action or full Presentation Action semantics. CP-neutral versus Lane-default routing and complete Activity Projection/reconstruction are not established. |
| Non-chat Surface | Existing Toolkit DataGrid has fixed rows, boolean proposals, synthetic revisions, authorization/stale rejection, readback, placement/focus and renderer fallback. | Retain it. Fixed-row enabled toggles do not implement full B05 local declaration read/add/remove, mixed inherited/local edits, NoChange, original result versus current value or lost-ACK/unknown reconciliation. |
| Context and failure presentation | Existing P14/P20 test captures current intent + explicit fixture refs/session while omitting old transcript from the synthetic owner call. No reference-selection/context lifecycle UI exists. | Add only the selected missing composition. That assertion does not prove material bytes, required/optional handling, semantic authorization, stale session rejection or delivery. Unavailable source, stale reference, malformed mapping and missing renderer are distinct. |
| Product direction and real owners | Canonical V1 is broader; downstream Shell/Execution specification roles are anticipated but those detailed files were not found in this tree. §§13–14 retain H1–H4/S1 and B01–B05 obligations. | No complete Handbook→Substrate→Collider delivery path, native Tauri, distribution clearance or adoption is demonstrated. A bounded fixture increment need not settle every open question, full Shell, complete Inspector or file-native storage. |

The prior three chat reads in this investigation remain historical session evidence:
`01a0b27c-9a51-7d81-9cbf-1b56dce1f449` (planning),
`01a0b46f-a834-7101-8406-015b53af8945` (correction), and
`01a0cac9-b802-7691-855b-4896f29eb3b0` (publication to a491f987…); none is a fresh
runtime proof. The current source/assertion inventory is recorded inside Collider's
[existing scenario matrix](../../../collider/docs/agent-native/packet-2/proof-scenario-and-evidence-matrix.md#2026-10-19-cumulative-sourceassertion-reconciliation).
It preserves P01–P22 and distinguishes implementation presence, exact assertion,
historical run claim and remaining gap. Counts or control/menu presence are insufficient.

### 15.2 Smallest next separately authorizable delta

**Current direction — 2026-10-19:** the human-authorized
[minimal fixture-backed Workspace amendment](../../../collider/docs/agent-native/controller-ownership-plan/implementation-and-regression-plan.md#2026-10-19-amendment--minimal-fixture-backed-workspace)
supersedes implementing F1 as a separate extension of the diagnostic harness.
F1's original requirements remain; its isolated-host implementation order and
nine-file fence are historical, not the entry gate or edit authority for the new
milestone. Neither standalone F1 completion nor completion of all Gate B is a
prerequisite to this bounded product-shaped fixture milestone. The amendment is
reviewed planning, not implementation authorization or final frontend adoption.

The next proposed experience is one recognizable synthetic Workspace: useful
Navigator destinations, a canonical Control Plane and Lane Workstreams, one
Composer with distinct agent, finite simulated shell-shaped, structured fixture
owner-action and local Presentation Action routes, typed Blocks and useful
semantic/execution/non-chat Surfaces. Inline-to-Tab, close/reopen/reconstruction,
persistent background-work awareness and explicit cancellation belong to the
journey; a collection of diagnostic controls alone does not satisfy it. Arbitrary
raw shell/PTY and real process effects remain unavailable. The
[Workspace acceptance specifications](../../../collider/docs/agent-native/packet-2/proof-scenario-and-evidence-matrix.md#2026-10-19-minimal-workspace-acceptance-specifications)
define MW-01–MW-08 and the fixture inputs, independent barriers, expected material,
effect counts and failure cases. They are planned tests, not executed evidence.

**All six F1 requirements are retained in full; none is deferred.** Their
[Workspace mappings](../../../collider/docs/agent-native/packet-2/proof-scenario-and-evidence-matrix.md#retained-f1-workspace-test-mappings)
carry forward exact selection/preview and current-input joins (F1-01), every
required-input refusal with zero effects (F1-02), optional omission versus required
refusal (F1-03), acceptance distinct from independent delivery and failed/unknown
or wrong-work/material evidence (F1-04), same-work/context identity and reconstruction
across placements (F1-05), and immutable original inputs after later selection or
default changes (F1-06). The detailed Collider assertions remain controlling;
this summary does not narrow them.

Label output **simulated execution** and lineage **synthetic context evidence**
throughout the journey. Fixture references, material and authority fields remain
provisional, not approved Handbook SDK types or authentic receipts. A fixture
result never grants permission; resolution or submission does not prove delivery.
Reconstruction is bounded by a declared fixture lifetime/read cut, not an implied
disk, frontend-process or owner-service restart guarantee. The existing harness,
corrected client/profile and regressions remain protected assets. How to reuse
them and where to host the Workspace is the next discussion (step 3), not an
implementation decision made by this alignment. No direct model call, credentials,
owner runtime change, main-app migration or new production coordinator is selected.

### 15.3 Cumulative proof obligations and acceptance boundaries

The [P01–P22 crosswalk](../../../collider/docs/agent-native/packet-2/proof-scenario-and-evidence-matrix.md#cumulative-p01p22-crosswalk)
is the operational coverage account, subordinate to the canonical/proof requirements.
It is not a replacement acceptance contract. Each future result must point to exact
candidate/profile, source and assertion, executed command/output and evidence class.
Unknowns stay unknown; source inspection and historical runs do not become fresh passes.

| Boundary | Retain now; missing delta remains explicit | What fixture success cannot establish |
|---|---|---|
| Unified Composer | Preserve existing agent-route and zero-dispatch regression evidence. The Workspace milestone specifies four distinct routes and CP/Lane defaults, including finite simulated shell-shaped commands; their implementation/proof remains open. Arbitrary raw shell/PTY stays unavailable. | Full canonical shell semantics, a production command parser or owner operation from a menu label. |
| B01 | Retain pending/no receipt, accepted-not-running, distinct identical intents and lost-ACK exact message/request reconciliation tests. Incomplete receipt/unknown UI and broader protected-equality/recovery remain open. Retained F1 Workspace assertions require context refusal before submission. | Authenticated caller namespace, durable original-request correlation, crash/expiry/no-replay or actual acceptance. |
| B02 | Retain cancel ACK/terminal races, detach/reopen, duplicate/gap/cut/version/retention and completed settlement assertions. Preserve source cursor versus local sequence. Observer-failure UI, nonstandard terminal binding and restart remain gaps. | Provider survival, real retention/read cuts, cancellation idempotency or host/service restart. |
| B03 | Retain same-work CP/Lane/tab IDs and filtered output. Historical binding, current association/revision and stale A→B→A/new-request rejection remain unproved. Retained F1 assertions freeze original fixture request fields, not real association authority. | Genuine Lane/World authority, execution placement or association fencing. |
| B04 | Existing input-capture test is narrow. The Workspace carries all F1 requirements: exact required/optional refs, known material, typed refusals/omissions and independent post-acceptance delivery outcomes without erasing work. Implementation and execution evidence remain pending. | Supported SDK materialization, semantic/disclosure/current-use authority, real model-visible injection or full B04. |
| Approval/queue/continuity | Enabled synthetic approve/deny and negative cases plus authorized/removed/rebound/unsupported queue remain mandatory retained regressions, even though real features stay deferred. Crosswalk identifies missing segment/Inbox, context revocation and no-promotion assertions. P14 input capture does not prove stale continuity refusal. | Real human approval, scheduler, native suspended-request continuation or qualified session revalidation. Disabled upstream profile is not enabled synthetic coverage. |
| Capability/fallback | Retain five discovery states with separate support/authorization/readiness and read-only missing-renderer fallback. Stale reference, malformed owner mapping and unavailable owner source need their own cases. | Owner effect authorization, safe arbitrary renderer/provider behavior or shipping accessibility/security. |
| B05 | Retain actual Toolkit editing, normalization, synthetic stale/refusal/readback and placement/focus assertions. Preserve richer declaration semantics, mixed inherited/local editability, NoChange, original committed result versus later value, unknown/lost-ACK/key reconciliation and coherent revisions as remaining work. | All-writer owner CAS, persistence/crash safety, installation, realization or active-World mutation. |

Keep inline anchors, view-local Sticky, tab focus/close/reopen and one same-work
behavioral owner. Preserve existing Sticky behavior without requiring a new Sticky
system; mandatory placement evidence is not waived. A new fixture-backed owner-shaped
endpoint remains fixture evidence. The original Gate B negative control and Packet A
historical results stay intact; never weaken assertions or count import failures as
negative controls. Full Gate B is claimed only after the cumulative mandatory set has
applicable execution evidence, not merely because the Workspace or retained F1 tests pass.

### 15.4 Repository responsibilities and order

| Stage / repository | Bounded work and dependency |
|---|---|
| Coverage/scope — Collider, aligned here | The reviewed working-tree amendment supersedes standalone F1, specifies the minimal Workspace and maps all six F1 requirements into its acceptance tests. No runtime implementation is included. |
| Host/reuse discussion — Collider (step 3) | Choose host/entrypoint, exact edit fence, fixture lifetime/test binding and how the corrected client/regressions will be reused. Preserve the old harness as a regression asset, not a second UI to expand in parallel. This alignment does not select or implement those choices. |
| Minimal fixture-backed Workspace — Collider, only after separate implementation authorization | Implement the product journey and all retained F1 assertions under the newly selected fence, not the historical nine-file F1 fence. Exit with scoped fresh evidence, synthetic labels and cumulative open gaps, not complete Gate B or coherent V1. |
| Further fixture gaps — Collider | Account for remaining B01/B02/B03/B05/fallback/approval/queue and broader routing obligations beyond the milestone. Do not rebuild already proved behavior or silently drop required cases. |
| Real semantic path — Handbook | Design/implement H1–H3 with supported exact read/render/authority/current-use outcomes; H4 only for the selected continuing-project population. Qualify normal external SDK consumption on supported builds. No private/test-support or raw Flow workaround. |
| Real execution path — Substrate | Close concrete B01/B03 original-request/authority/acceptance/recovery, then B02 observation/cancel; join S1 retained material to actual runtime delivery. B05 all-writer transaction/result recovery is independently bounded work. Reuse existing owners and applicable Linux/security gates; no E3-F resumption here. |
| Real consumer join — Collider with both owners | Bind exact supported mappings and run the selected context/execution/observation scenario plus real B05 read/add/remove/stale-conflict/readback. Include §14.5 negatives/races. Partial owner evidence remains partial; fixture DTOs may change to match actual owners. |
| Deployment and adoption — Collider | Complete applicable Gate D native-host/artifact/security/licensing/economics evidence, then decide adoption explicitly. Implementation-neutral Shell design can progress where sufficiently settled; broader product construction and migration have separate scope. |

The existing order is actual-client fixtures → supported real-owner exact-path proof →
applicable deployment/artifact/economics evidence and a separate adoption decision.
Owner contract work can progress alongside separately authorized fixture work. No
need to finish every architecture/storage decision before interaction learning; no
fixture may substitute for real required context or grant approval to skip owners.
The complete real target remains one supported semantic selection, one Linux ephemeral
World task in CP/Lane and the separate B05 declaration operation. This update neither
selects a callable endpoint nor refreshes Linux runtime readiness.

### 15.5 Decisions versus engineering work

| Decision / work | Status and timing |
|---|---|
| Fixture-first versus owner-first strategy | Already supplied by C-OQ-027 and the staged proof. No new strategic decision requested. |
| Exact next implementation scope | Minimal product-shaped fixture Workspace, with all six F1 requirements retained. Host/entrypoint, edit fence, fixture retention and client/test reuse await step-3 discussion and a separately authorized implementation assignment. Standalone F1 is superseded; no main-app migration or production adoption selected. |
| Real Handbook authority population | Continuing-project use remains recommended; H4 successor binding is still proposed. An expressly initial-only demonstration is a narrower product/owner selection and cannot be substituted silently. Resolve before claiming that real route, not before neutral synthetic presentation. |
| SDK/ref encoding, validated fixtures, adapter location, deterministic tests, transport and admission/recovery mechanisms | Engineering/owner-contract work within separately authorized scopes. These do not justify reopening broad product choices or asking the user to design APIs. |

Existing SDK-first/full-B04, owner separation, fresh reconstruction and file-native
separation remain controlling. Continuing Handbook project authority is not provider
Session Continuity. No new owner API, authority rule, frontend framework, repository-wide
migration or wholesale cutover is adopted.

**Historical verification boundary for the preceding cross-repository update:** fresh source/assertion inspection and
non-mutating document/link/table/reference/whitespace/scope checks only. No runtime,
install, package generation, tests, browser execution, owner mutation or new pass is
claimed. Historical §15 validation described the earlier single-document correction;
this dated update additionally changes the two existing Collider planning documents
linked above. Independent causal review and owner acceptance remain separate task
steps; no self-approval or publication follows from authorship.

**Subsequent Substrate-only alignment — 2026-10-19:** the user authorized this
document to match the reviewed Collider Workspace amendment. §§15.2–15.5 now carry
that next-milestone order and preserve every F1 requirement and owner-proof ceiling.
The Collider amendment's “External alignment outstanding” paragraph describes its
earlier publication state; this later entry records the requested alignment here
without modifying that historical account. The two Collider working-tree documents
were read but not edited. Step 3 remains discussion, not implementation or an adopted
hosting/reuse decision. This pass checks the document delta, links/anchors, tables,
whitespace and scope; it does not claim a fresh independent review, runtime test,
commit, push, full Gate B, real B04 or frontend adoption.
