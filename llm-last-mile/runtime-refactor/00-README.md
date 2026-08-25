# Runtime Refactor Control Pack

**Status:** canonical control pack for future runtime-refactor slices
**Scope:** planning, contracts, sequencing, and proof gates; not implementation history
**Source directive:** [`../../substrate-runtime-refactor-directive-revised.md`](../../substrate-runtime-refactor-directive-revised.md)
**Repo-truth snapshot:** 2026-08-02 at
`4ceecd50e20d822dda7cbd8f0e1bef4ccad65d8e` / tree
`8ed5dc7a354b731016a103b68091864b6a09223a`; re-check live code before every slice.
**Current scheduling projection:** [`index/current.md`](index/current.md) is the visibly non-authoritative current-state projection; canonical decisions, packets, and gates remain at their linked path-stable owners.
**Historical scheduling state (superseded for active scheduling on 2026-08-19):**
`A1.1d-5R3-PLAN` was complete planning-only authority that froze the
R3 implementation graph at `19c40d41679e843e3e524f64fb9827959849d33e` /
`d7f6b84c9efc8ad03d98ad55c4e1a31611b96335` with terminal planning fingerprint
`sha256:8f4cf54640443dbeb82fffbef68fac8d03eeaa6c72cf4e44f645044bc2b210e7`. R3 implementation is
`PARKED_BY_USER` and no R3 implementation task has been dispatched. R2-4 remains the terminal
predecessor and closes only context propagation; its evidence is unchanged in the
[R2-4 closeout evidence record](review-control/r2-4-closeout-evidence.md). The B1/B2.1 joint
production integration closeout is now complete on the frozen 2026-08-03 production source at
`f37943eb917285a044c5e12a05b481572c8d0a09` / tree
`54ae7b2a2d467a568b575665b99dcb94ef2893a2`, with packet evidence recorded in
[`review-control/b1-b2-1-joint-closeout-differential-evidence.json`](review-control/b1-b2-1-joint-closeout-differential-evidence.json)
and
[`review-control/b1-b2-1-joint-closeout-linux-evidence.md`](review-control/b1-b2-1-joint-closeout-linux-evidence.md).
This closeout adds no new product or test bytes and dispatches no successor. The canonical B3.1/C1 source-candidate completion projection moved to
[`b3-1-c1/current-state.md#b31-and-c1-source-candidate-completion-projection`](b3-1-c1/current-state.md#b31-and-c1-source-candidate-completion-projection).
The canonical A1.2b source-candidate completion projection moved to
[`a1-2-earlier-histories/current-state.md#a12b-source-candidate-completion-projection`](a1-2-earlier-histories/current-state.md#a12b-source-candidate-completion-projection). That checkpoint's
`AUTHORITY_REQUIRED:R3_RESUME` sequence is preserved as historical chronology. The macOS-parity
decision removed protected macOS lifecycle and Windows predecessors, but its former global
blocking order is superseded by the Linux-first scheduling decision in
[`linux-first-runtime-resumption/DECISION.md`](linux-first-runtime-resumption/DECISION.md). The
reentry gate is closed as a documentation-only historical selection. The active next
implementation packet is
[`linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md`](linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md);
the held
[`linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md`](linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md)
and
[`linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md`](linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md)
records remain preserved as non-executable historical fences, and macOS parity remains a separate
lane under `AUTHORITY_REQUIRED:MACOS_DEV_PARITY`.

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
| [`contracts/development-review-and-remediation-contract.md`](contracts/development-review-and-remediation-contract.md) | implementing, reviewing, remediating, or closing a packet | development review and remediation contract |
| [`gates/authority-required-macos-dev-parity.md`](gates/authority-required-macos-dev-parity.md) | evaluating or activating the macOS developer-parity lane | canonical `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` gate |
| [`gates/authority-required-runtime-refactor-reentry.md`](gates/authority-required-runtime-refactor-reentry.md) | interpreting the closed global reentry selection | canonical `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` gate |
| [`05-debug-regression-ledger.md`](05-debug-regression-ledger.md) | writing tests, smoke plans, or closeout evidence | resolved baselines, open debug seams, permanent regression gates |
| [`index/README.md`](index/README.md) | resolving current scheduling authority or supersession | decision and packet-owner index; navigation only |
| [`index/current.md`](index/current.md) | checking the current global packet, held packets, lane-local gate, or deferred work | visibly non-authoritative current-state projection |
| [`06-review-finding-inventory.md`](06-review-finding-inventory.md) | classifying, retaining, deduplicating, or resolving non-blocking review findings | the single `P3`/`P4` review and process-debt inventory; never a `P1`/`P2` waiver |
| [`foundations/semantic-status-labels.md`](foundations/semantic-status-labels.md) | classifying a seam or reviewing a promotion claim | semantic status labels and the promotion rule |
| [`foundations/authority-vocabulary.md`](foundations/authority-vocabulary.md) | interpreting authority terms or reviewing a control-plane boundary | authority vocabulary |
| [`foundations/reading-and-update-rules.md`](foundations/reading-and-update-rules.md) | reading or updating the control pack | reading and update rules |
| [`foundations/per-slice-context-assembly-protocol.md`](foundations/per-slice-context-assembly-protocol.md) | assembling implementation or review context for a slice | per-slice context assembly protocol |
| [`foundations/normative-conventions.md`](foundations/normative-conventions.md) | interpreting record, identity, revision, timestamp, commitment, or ref conventions | normative conventions |
| [`review-control/`](review-control/) | opening, closing, or extending a review cycle | small standard-library cycle record, validator, example, and focused tests |

Do not load the full historical design/debug stack by default. Start with the applicable crosswalk
row, slice row, contract section, regression row, and review contract. Follow only the named
must-read links. Slice A0's authority-leak inventory remains inside `02-seam-crosswalk.md`; `06` is
only the cross-slice non-blocking review-finding inventory and does not absorb A0 authority truth.

## Per-slice context assembly protocol

Canonical content is maintained in [`foundations/per-slice-context-assembly-protocol.md`](foundations/per-slice-context-assembly-protocol.md).

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

Canonical authority for these labels and their promotion rule is now in [`foundations/semantic-status-labels.md`](foundations/semantic-status-labels.md).

## Authority vocabulary

Canonical authority for this vocabulary is now in [`foundations/authority-vocabulary.md`](foundations/authority-vocabulary.md).

## Reading and update rules

Canonical authority for these rules is now in [`foundations/reading-and-update-rules.md`](foundations/reading-and-update-rules.md).

## Current control conclusion

The canonical B1/B2.1 control conclusion moved to [`b1-b2-1/current-state.md#current-b1b21-control-conclusion`](b1-b2-1/current-state.md#current-b1b21-control-conclusion).

The canonical A1.2a/A1.2a-WB/A1.2a-S completion projection moved to
[`a1-2-earlier-histories/current-state.md#a12aa12a-wba12a-s-completion-projection`](a1-2-earlier-histories/current-state.md#a12aa12a-wba12a-s-completion-projection).

The canonical B1/B2.1 completion projection moved to [`b1-b2-1/current-state.md#b1b21-completion-projection`](b1-b2-1/current-state.md#b1b21-completion-projection).

The canonical mixed B3.1/C1 predecessor and A1.2b completion projection moved to
[`a1-2-earlier-histories/current-state.md#a12b-mixed-predecessor-completion-projection`](a1-2-earlier-histories/current-state.md#a12b-mixed-predecessor-completion-projection).

Historical user-authorized scheduling disposition, superseded for active scheduling on
2026-08-19: R3 planning was complete at this checkpoint
(`19c40d41679e843e3e524f64fb9827959849d33e` / `d7f6b84c9efc8ad03d98ad55c4e1a31611b96335`,
planning fingerprint `sha256:8f4cf54640443dbeb82fffbef68fac8d03eeaa6c72cf4e44f645044bc2b210e7`), but R3
implementation is `PARKED_BY_USER` and no R3 implementation task has been dispatched. The
previously active authority wall `AUTHORITY_REQUIRED:B1_B2_1_JOINT_CLOSEOUT` is now closed, B3.1,
C1, and the bounded internal A1.2b packet are complete on the bound Tuesday, August 4, 2026
candidate, and the then-current gate was `AUTHORITY_REQUIRED:R3_RESUME`. The former replacement
was `AUTHORITY_REQUIRED:MACOS_DEV_PARITY`, which still owns the macOS lane, but the current global
replacement was the reentry gate under
[`linux-first-runtime-resumption/DECISION.md`](linux-first-runtime-resumption/DECISION.md), now
closed first by the selected
[A1.3 Linux-first packet](linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md), then narrowed
by the held
[A1.3-P0 Linux-first preparatory packet](linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md),
and finally corrected to the active
[A1.3-P1 Linux-first atomic public-adoption packet](linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md).
Protected macOS lifecycle and Windows R3 are not current product predecessors. No
`cargo test --workspace`
expected-failure inventory is frozen at this checkpoint; approximate workspace-failure counts are
not authority.

The internal **A1.1d-5I installer/bootstrap compatibility audit** is now complete at the
`b29897e0` baseline. It is evidence beneath A1.1d-5, not an A1.1d-6 checkpoint, and it changes no
broader architecture owner or sequence. The audit proves that the current private-home ancestor
ACL rule conflates non-writing traversal with replacement authority, custom-prefix authority is
not propagated across every install/uninstall child, and partial-install cleanup is not
convergent. Remediation remains bounded to **A1.1d-5R1 → A1.1d-5R2 → A1.1d-5R3**. R1 is
implemented and review-clean through `4d0acff68e20d86b97fe5367b8a4617554f33ef4`.
**A1.1d-5R2-0 is planning-complete, A1.1d-5R2-1 is implementation- and review-complete through
`2653c2ef20ae2e119a444811e6fb46e86d1a6ec6`, and Routes A–F are complete.** At that audit
checkpoint, the live R2 sequence was **Routes A–F complete → remediation planning → R1 → P1 →
fresh canonical baseline → renewed R2-2 production-fix-free integration closeout → final six-file
closeout docs → one ordinary fast-forward source publication → R2-3 → R2-4**, followed by R3.
The earlier failed integration closeout proved that R2-2 remained incomplete and unpublished; the
completed E, harness, and F packets did not replace the renewed closeout. R2-3, R2-4, and R3 were
then unstarted. Until the renewed closeout and later assigned lifecycle/product smoke pass, both
A1.1d Linux closeout and only the Linux product-smoke portion of the B1/B2.1 joint closeout
remained blocked. B1 receipt and B2.1 supervisor semantics did not regress, and native
macOS A1.1d proof is not added as a dependency of the B1/B2.1 corridor.

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
owned; at that historical pre-F checkpoint, R2-1 was complete and R2-2 was in progress through
review-clean Routes A, B, C, and D. The failed Routes A–D integration closeout was blocker
evidence rather than a completed packet; R2-2E was implementation-, proof-, and review-complete.
The combined F0/F0a/F0b/F0-HC harness corridor is
implemented, proof-complete, review-clean, committed, and preserved. At that historical pre-F
checkpoint, R2-2F was the exact next packet; the completed-F closeout below supersedes that
next-task status. Renewed closeout, R2-3/R2-4/R3 remained unstarted there. Static macOS/Windows
inspection in R2-0 is not native platform proof.

R2-1 implements the 19 owned rows PI-001–PI-004, PI-010–PI-011, PI-032–PI-034, PI-061–PI-063,
PI-067, PI-082–PI-085, PI-104, and PI-117. Its shared IH framing, hidden authenticated bootstrap
action, Unix principal binding, A-over-B dev/shim/generated projection matrix, and shell-only
explicit trace/policy binding are proven without changing the neutral trace setter, physical shim,
replay, world-deps production, lifecycle behavior, or any later packet owner.

The active planning packet at that checkpoint remained **A1.1d-5R2-2 — Unix release, sudo, Linux
service, and runtime propagation**. Routes A–D remained individually review-clean at their final
replayed commits `3bf59b30e4c7348b8ff6315e3eb3658d74af2552`,
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
projections. The later historical F5-PD correction, whose boundary the completed-F record below
retains, removed production World Doctor fixture selection and changed that child to the hidden
passive path; it did not change the separately F5-owned world-deps fixture. Ambient B, generated
files, or a contextless repository binary cannot
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

The canonical B1/B2.1 retained-target and dispatch-prerequisite projection moved to [`b1-b2-1/current-state.md#retained-target-and-dispatch-prerequisite-projection`](b1-b2-1/current-state.md#retained-target-and-dispatch-prerequisite-projection).


## A1.1d-5R2-2F0-HC shell-harness closure audit and environment correction

Canonical content: [`a1.1d-5r2-2f/current-state.md#a11d-5r2-2f0-hc-shell-harness-closure-audit-and-environment-correction`](a1.1d-5r2-2f/current-state.md#a11d-5r2-2f0-hc-shell-harness-closure-audit-and-environment-correction).

## A1.1d-5R2-2F0 historical differential authority correction

Canonical content: [`a1.1d-5r2-2f/current-state.md#a11d-5r2-2f0-historical-differential-authority-correction`](a1.1d-5r2-2f/current-state.md#a11d-5r2-2f0-historical-differential-authority-correction).

## F0/F0a/F0b/F0-HC canonical closeout

Canonical content: [`a1.1d-5r2-2f/current-state.md#f0f0af0bf0-hc-canonical-closeout`](a1.1d-5r2-2f/current-state.md#f0f0af0bf0-hc-canonical-closeout).

## A1.1d-5R2-2F historical readiness-boundary correction

Canonical content: [`a1.1d-5r2-2f/current-state.md#a11d-5r2-2f-historical-readiness-boundary-correction`](a1.1d-5r2-2f/current-state.md#a11d-5r2-2f-historical-readiness-boundary-correction).

## A1.1d-5R2-2F5-PD canonical correction

Canonical content: [`a1.1d-5r2-2f/current-state.md#a11d-5r2-2f5-pd-canonical-correction`](a1.1d-5r2-2f/current-state.md#a11d-5r2-2f5-pd-canonical-correction).

## A1.1d-5R2-2F canonical closeout

Canonical content: [`a1.1d-5r2-2f/current-state.md#a11d-5r2-2f-canonical-closeout`](a1.1d-5r2-2f/current-state.md#a11d-5r2-2f-canonical-closeout).
## A1.1d-5R2-2 renewed closeout publication authority (historical pre-RP4 checkpoint)

Canonical content: [`a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-renewed-closeout-publication-authority-historical-pre-rp4-checkpoint`](a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-renewed-closeout-publication-authority-historical-pre-rp4-checkpoint).

## Canonical broad-wall invocation authority correction

Canonical content: [`a1.1d-5r2-2-renewed-closeout/current-state.md#canonical-broad-wall-invocation-authority-correction`](a1.1d-5r2-2-renewed-closeout/current-state.md#canonical-broad-wall-invocation-authority-correction).

## A1.1d-5R2-2 closeout-remediation planning authority (historical RP0 checkpoint)

Canonical content: [`a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-closeout-remediation-planning-authority-historical-rp0-checkpoint`](a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-closeout-remediation-planning-authority-historical-rp0-checkpoint).

## A1.1d-5R2-2 RP3/RP4/RP5 closeout status

Canonical content: [`a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-rp3rp4rp5-closeout-status`](a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-rp3rp4rp5-closeout-status).
## A1.1d-5R2-3 closeout status

Canonical content: [`a1.1d-5r2-3/current-state.md#a11d-5r2-3-closeout-status`](a1.1d-5r2-3/current-state.md#a11d-5r2-3-closeout-status).
## A1.1d-5R3-PLAN authoritative planning status (archived for active scheduling)

Canonical content: [`a1.1d-5r3/current-state.md#a11d-5r3-plan-authoritative-planning-status-archived-for-active-scheduling`](a1.1d-5r3/current-state.md#a11d-5r3-plan-authoritative-planning-status-archived-for-active-scheduling).
## R3 implementation status append

Canonical content: [`a1.1d-5r3/status-append.md#r3-implementation-status-append`](a1.1d-5r3/status-append.md#r3-implementation-status-append).

### `A1.1d-5R3-MANIFEST`

Canonical content: [`a1.1d-5r3/status-append.md#a11d-5r3-manifest`](a1.1d-5r3/status-append.md#a11d-5r3-manifest).

### `A1.1d-5R3-LINUX`

Canonical content: [`a1.1d-5r3/status-append.md#a11d-5r3-linux`](a1.1d-5r3/status-append.md#a11d-5r3-linux).

### `A1.1d-5R3-LINUX-CLOSEOUT`

Canonical content: [`a1.1d-5r3/status-append.md#a11d-5r3-linux-closeout`](a1.1d-5r3/status-append.md#a11d-5r3-linux-closeout).

### `A1.1d-5R3-MAC`

Canonical content: [`a1.1d-5r3/status-append.md#a11d-5r3-mac`](a1.1d-5r3/status-append.md#a11d-5r3-mac).
## A1.1d-5R3-MAC attempt-4 remediation status (2026-08-06)

Canonical content: [`r3-mac-evidence-recovery/current-state.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](r3-mac-evidence-recovery/current-state.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN status (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/current-state.md#aux-r3-mac-evidence-recovery-plan-status-2026-08-07`](r3-mac-evidence-recovery/current-state.md#aux-r3-mac-evidence-recovery-plan-status-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN current authority correction (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/current-state.md#aux-r3-mac-evidence-recovery-plan-current-authority-correction-2026-08-07`](r3-mac-evidence-recovery/current-state.md#aux-r3-mac-evidence-recovery-plan-current-authority-correction-2026-08-07).
## macOS developer-parity lane (2026-08-19; scoped)

Canonical content: [`macos-dev-parity/current-state.md#macos-developer-parity-lane-2026-08-19-scoped`](macos-dev-parity/current-state.md#macos-developer-parity-lane-2026-08-19-scoped).

## Linux-first runtime-refactor scheduling decision (2026-08-20; controlling)

> **Projection status:** non-authoritative current-state projection. Canonical owners are linked from [`index/current.md`](index/current.md); this legacy section grants no authority.

[`linux-first-runtime-resumption/DECISION.md`](linux-first-runtime-resumption/DECISION.md)
supersedes the macOS decision's former global blocking order. The reentry gate has closed as a
live-source historical selection of
[A1.3](linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md), later narrowed by the held
[A1.3-P0](linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md), and now
corrected so the active Linux-first implementation packet is
[A1.3-P1](linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md).
The older A1.3 and A1.3-P0 records remain held as historical fences only. macOS parity may
proceed in its separate lane and Windows remains deferred. Neither lane's closure dispatches the
other's successor.
