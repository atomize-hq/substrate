**Kind:** historical record
**Stable ID:** `runtime-refactor-cross-cutting-control-checkpoints-history`
**Status:** canonical historical record
**Canonical for:** the exact two cross-cutting root-00 historical checkpoint bodies extracted by D12
**Authority scope:** historical chronology and checkpoint qualification only; never current scheduling, gate, implementation, evidence-satisfaction, or successor authority
**Source relationship:** two noncontiguous source bodies are preserved independently below and are not represented as one contiguous source span
**Historical scheduling source provenance:** [`../00-README.md#runtime-refactor-control-pack`](../00-README.md#runtime-refactor-control-pack), baseline lines 10–40 inclusive (`2778` bytes)
**Historical scheduling source SHA-256:** `4da52e7004f22ad7a3ae3e0b641f290eb24728891e269b6c8ddc8a6f88c52ded`
**Control conclusion source provenance:** [`../00-README.md#current-control-conclusion`](../00-README.md#current-control-conclusion), baseline residual body lines 135–343 inclusive (`16979` bytes)
**Control conclusion source SHA-256:** `24d5b750651639e4fdd1afde259a57143d938ecbb30d511b2e5e3461d53a753a`
**Link rebase:** only repository-relative Markdown targets in the two preserved bodies are rebased by one parent directory so they continue to resolve from `history/`; link labels, fragments, prose, dates, IDs, hashes, status words, failures, stops, limitations, and ordering are unchanged
**Supersedes:** canonical ownership of only these two exact historical bodies; root headings and former locations remain compatibility routes
**Superseded by:** none
**Projection consumers:** [`../00-README.md`](../00-README.md), [`../index/README.md`](../index/README.md), [`../index/by-id.md`](../index/by-id.md), [`../index/by-kind.md`](../index/by-kind.md), [`../index/by-packet.md`](../index/by-packet.md)

# Cross-cutting control-pack historical checkpoints

> **Authority boundary:** This file preserves cross-packet historical checkpoints that cannot be divided among narrower owners without rewriting chronology. Read [`../index/current.md`](../index/current.md) and its linked decisions, packets, and gates for current state. Nothing here authorizes work or changes any current status.

## Historical scheduling state (superseded for active scheduling on 2026-08-19)

<!-- exact-extracted-body:historical-scheduling-state:start -->
**Historical scheduling state (superseded for active scheduling on 2026-08-19):**
`A1.1d-5R3-PLAN` was complete planning-only authority that froze the
R3 implementation graph at `19c40d41679e843e3e524f64fb9827959849d33e` /
`d7f6b84c9efc8ad03d98ad55c4e1a31611b96335` with terminal planning fingerprint
`sha256:8f4cf54640443dbeb82fffbef68fac8d03eeaa6c72cf4e44f645044bc2b210e7`. R3 implementation is
`PARKED_BY_USER` and no R3 implementation task has been dispatched. R2-4 remains the terminal
predecessor and closes only context propagation; its evidence is unchanged in the
[R2-4 closeout evidence record](../review-control/r2-4-closeout-evidence.md). The B1/B2.1 joint
production integration closeout is now complete on the frozen 2026-08-03 production source at
`f37943eb917285a044c5e12a05b481572c8d0a09` / tree
`54ae7b2a2d467a568b575665b99dcb94ef2893a2`, with packet evidence recorded in
[`review-control/b1-b2-1-joint-closeout-differential-evidence.json`](../review-control/b1-b2-1-joint-closeout-differential-evidence.json)
and
[`review-control/b1-b2-1-joint-closeout-linux-evidence.md`](../review-control/b1-b2-1-joint-closeout-linux-evidence.md).
This closeout adds no new product or test bytes and dispatches no successor. The canonical B3.1/C1 source-candidate completion projection moved to
[`b3-1-c1/current-state.md#b31-and-c1-source-candidate-completion-projection`](../b3-1-c1/current-state.md#b31-and-c1-source-candidate-completion-projection).
The canonical A1.2b source-candidate completion projection moved to
[`a1-2-earlier-histories/current-state.md#a12b-source-candidate-completion-projection`](../a1-2-earlier-histories/current-state.md#a12b-source-candidate-completion-projection). That checkpoint's
`AUTHORITY_REQUIRED:R3_RESUME` sequence is preserved as historical chronology. The macOS-parity
decision removed protected macOS lifecycle and Windows predecessors, but its former global
blocking order is superseded by the Linux-first scheduling decision in
[`linux-first-runtime-resumption/DECISION.md`](../linux-first-runtime-resumption/DECISION.md). The
reentry gate is closed as a documentation-only historical selection. The active next
implementation packet is
[`linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md`](../linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md);
the held
[`linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md`](../linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md)
and
[`linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md`](../linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md)
records remain preserved as non-executable historical fences, and macOS parity remains a separate
lane under `AUTHORITY_REQUIRED:MACOS_DEV_PARITY`.
<!-- exact-extracted-body:historical-scheduling-state:end -->

## Current control conclusion

<!-- exact-extracted-body:current-control-conclusion-history:start -->
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
[`linux-first-runtime-resumption/DECISION.md`](../linux-first-runtime-resumption/DECISION.md), now
closed first by the selected
[A1.3 Linux-first packet](../linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md), then narrowed
by the held
[A1.3-P0 Linux-first preparatory packet](../linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md),
and finally corrected to the active
[A1.3-P1 Linux-first atomic public-adoption packet](../linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md).
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

<!-- exact-extracted-body:current-control-conclusion-history:end -->
