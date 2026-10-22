# E3-F Linux service-first implementation handoff — 2026-10-21

**Kind:** derived guidance; future implementation prompt, not canonical authority or current dispatch.
**Owners:** [E3-F slice](../../llm-last-mile/runtime-refactor/slices/e3-agent-config-projection-and-gateway-adoption.md#greenfield-handoff-and-service-first-sequence-correction--2026-10-21),
[projection contract](../../llm-last-mile/runtime-refactor/contracts/agent-config-projection-v1.md#e3-f-greenfield-retention-and-replacement--2026-10-21),
and [gateway cleanup](../../llm-last-mile/runtime-refactor/contracts/managed-gateway-adoption-v1.md#e3-f-retained-successor-and-cleanup-joins).
This file is corrected guidance derived from those owners. Saving, reviewing, integrating or mirroring
it does not execute it. E3-E remains closed; E3-F remains paused, unaccepted and unclosed; enclosing
E3 remains incomplete. Invoke the prompt below only for a separately selected implementation session.

$using-agent-skills

Begin by loading `/home/spenser/.agents/skills/using-agent-skills/SKILL.md`. The Linux worktree has no repository-local `.agents/skills/` directory; use the verified installed skill root `/home/spenser/.agents/skills/` for the `$skill-name` invocations below. Discover, announce and invoke each applicable skill by reading its actual `SKILL.md` when its phase starts. Listing a skill is not applying it. Do not preload the entire skill family or turn conditional skills into additional acceptance gates.

## Runtime parameters

Expected Linux worktree:
`/home/spenser/__Active_code/.worktrees/substrate-runtime-refactor-product-authority-integration`

Resolve `REPO_ROOT` with `git rev-parse --show-toplevel`; run repository commands there. Paths below are relative to `REPO_ROOT` unless explicitly absolute.

```text
SLICE_ID: E3-F
SELECTED_WORKSTREAM: service-side V2 retained launch prerequisites and admission
EXECUTION_LIMIT: one dependency-ready, independently reviewable implementation packet
PRESERVED_SOURCE_CHECKPOINT: eeb17d952edba00a00c336220ccfac7147ce06ae
PRESERVED_CHECKPOINT_TREE: 8b9ecfbac594d403e7c8eb38cb9357e4006ac8ce
BRANCH: feat/runtime-refactor-e3-f-retained-codex-implementation
UPSTREAM: origin/feat/runtime-refactor-e3-f-retained-codex-implementation
ORIGINAL_CHECKPOINT_PARENT: c7e08c3a39e69a6b6ba05853a37378a77a11afa6
PRESERVED_W8_DIFF_SHA256: efcd97c433e33bc8a5182d7fc4a8f7db4a1946625dfeed3ab252291e6785c8c5
```

When explicitly invoked for implementation, this prompt selects service-side V2 work before
completing the shell bridge and authorizes preflight, specification/planning and implementation/proof of one bounded packet inside the existing E3-F service-side authority. It does not authorize all of E3-F, automatic successors, shared-branch integration, push, deployment or persistent-service changes. Do not request another routine approval merely because the historical documentation says continuation was paused: this prompt supplies the selected bounded continuation authority. A real contract/scope conflict still requires an explicit stop.

## Mission and current truth

Use the existing `llm-last-mile/runtime-refactor/` control plane. Preserve the checkpoint and continue from it; do not restart the refactor or rewrite its architecture.

The identities above describe the immutable source checkpoint, not a required current documentation
HEAD/tree. After documentation integration, rebind the actual descendant HEAD/tree and verify the
checkpoint ancestry and unchanged eight-file source delta/modes; do not reset to the old tree.

At that source checkpoint:

- The eight W8 source edits are committed and pushed, and the Linux implementation worktree is clean. They remain WIP, unaccepted implementation.
- I1/I2 ownership/accounting design and the c381 producer/classification/custody-bridge documentation are incorporated in the documentation parent.
- `WorldService::execute_stream` still rejects member-dispatch V2 before its ordinary path.
- `MemberRuntimeManager::launch_v2` and `E3Codex0125LaunchAdapterV1` are absent.
- `ActiveMemberRuntime` still has the common non-E3 lease/ordinary execution fields that the I1 contract replaces.
- E3-E is closed. Enclosing E3 is incomplete. No prior WIP commit proves runtime adoption, installation or E3-F completion.

The service-side prerequisite chain must be built and proven before the shell bridge can obtain positive production transfer proof. Do not remove the V2 rejection and forward through V1/UAA, invent a Registered/Active result, or substitute helper success for the missing path.

The next workstream's integrated target is one authentic V2 request selecting the E3 branch before ordinary admission, consuming its original preparation once, and retaining one service-owned provisional/active runtime with correct launch and failure/cleanup custody.

That is a workstream target, not an assertion that all its absent prerequisites fit in one packet. Phases 0–3 must map the actual native-input, adapter, ownership and accounting dependencies and freeze the earliest green service-owned packet. Prefer the service admission/ownership join if its prerequisites are actually ready. If absent adapter/native/accounting implementation must precede it, select that exact prerequisite packet within this workstream and state its narrower completion claim. Do not invent placeholder resource owners to make the target appear independently implementable. Stop after the selected packet; do not automatically implement the entire prerequisite chain.

## Canonical control pack and precedence

Read `llm-last-mile/runtime-refactor/00-README.md` in full, then selectively follow its canonical-owner links from these files under `llm-last-mile/runtime-refactor/`:

- `01-target-architecture.md`
- `02-seam-crosswalk.md`
- `03-phase-slice-map.md`
- `04-contracts-and-gates.md`
- `05-debug-regression-ledger.md`

Required focused owners under `llm-last-mile/runtime-refactor/`:

- `foundations/per-slice-context-assembly-protocol.md`
- `contracts/development-review-and-remediation-contract.md`
- `slices/e3-agent-config-projection-and-gateway-adoption.md`: current E3-F correction, dated I1/I2 binding/return sequence, F1–F14 obligation table and E3-F row.
- `contracts/agent-config-projection-v1.md`: retained ownership/callable joins; I1/I2 ownership/accounting reconciliation; applicable native realization/registry/adapter and loader boundaries; future symbol fence; admission fence; deterministic pinned-Codex proof route for its actual dependent claims.
- `contracts/managed-gateway-adoption-v1.md`: retained successor/cleanup joins and the consumed E3-E transfer invariants.
- Applicable exact `05` gate owners and `06-review-finding-inventory.md`.

Read the focused greenfield/provenance findings in `RUNTIME_REFACTOR_SESSION_DECISIONS_2026-09-18.md`
and `SUBSTRATE_CORE_ARCHITECTURE_SESSION_DECISIONS_2026-09-20.md`. Their dirty-checkout and pending-c381
statements are historical; c381 is integrated and its bounded design review is not an unresolved blocker.

Use the reusable runtime-refactor prompt's Phase 0–9 structure. Current canonical owners supersede older generic prompt assumptions: use the existing decomposed owners, the bounded causal-review budget and P1/P2 completion rule. Do not restore an all-findings-block rule, unlimited review loop, mandatory duplicate closeout reviewer, or six-root-file-only amendment rule from older prompt text. Do not create a parallel SPEC/PLAN/TASKS control plane. Record this packet's specification, plan, tasks and proof references in its task evidence and update existing canonical owners only when warranted.

## Greenfield retention and replacement

Apply the canonical [greenfield rule](../../llm-last-mile/runtime-refactor/contracts/agent-config-projection-v1.md#e3-f-greenfield-retention-and-replacement--2026-10-21).
For **each mechanism retained by the selected packet**, record in the existing packet evidence:

| Mechanism | Required product capability | Existing owner | Independent architectural/security purpose, or replacement/removal disposition | Prerequisite and proof |
|---|---|---|---|---|
| Bind actual implicated paths during Phases 0–3 | Required outcome, not a historical caller | Canonical boundary | Compatibility alone is insufficient | Exact dependent claim and ordering |

Apply this to old V1 wire/construction paths, UAA realizations, copied credentials/configuration,
fallbacks, duplicate lifecycle mechanisms and compatibility-only adapters/tests where implicated.
Existing callers/tests, version names, prior support and support for Claude/UAA alone justify none
of them. Preserve required non-E3 capability and counted security admission; decide each underlying
mechanism separately. Preserve UAA common run/event/cancel/session/capability semantics and the
agent-neutral lifecycle/policy owners. Pinned Codex is a bounded realization/security lane.

Otherwise name the replacement/removal under its existing owner, with capability/security proof and
sequencing. Do not silently preserve it indefinitely or delete it blindly. Reconcile narrowly
conflicting canonical preservation/future-fence clauses before dependent source changes; this
prompt cannot supersede a contract or widen its source catalog. Do not repeat the capability audit,
create a parallel framework or reopen closed predecessors. A3 legacy read precedence, C2 migration/
compatibility ingress and D1 compatibility-posture/V3 reconciliation remain concrete follow-ups
with those owners, not completed work or unrelated new E3-F prerequisites.

## Core truth rules

Keep artifact existence, correct authority ownership, real production adoption, enforcement location and exact proof separate.
Preserve immutable identity/policy/E2 joins, accepted-parent/no-redelivery, secure credential FD/gateway
ownership and no authority-dropping V2-to-V1 conversion. Keep the current service exclusion/concurrency
boundary; do not invent coexistence or declare permanent serialization to be the final product. A compile, fixture, socket, fake executable or state-shaped struct cannot establish the production seam. Preserve unchanged E3-A–E proof with its original subject and claim ceiling.

## Phase 0 — Worktree and dependency preflight

Load applicable AGENTS.md instructions. Rebind host, root, branch, HEAD/tree, upstream/remote tip, index, tracked/untracked status and relevant ancestry. Expect a clean descendant containing the stated checkpoint and integrated guidance; verify the
checkpoint's original parent/tree, binary/full-index parent-to-checkpoint hash, and unchanged source
bytes/native modes in the descendant. Bind the actual start separately. Stop on unexplained overlapping drift. Do not reset, stash, discard, amend or silently rebase W8.

Verify Linux x86_64, Rust 1.89.0 and the tools/resources required by the actual selected proof commands. Capture historical versus freshly observed failures separately. Do not assume installed revision 12 matches the checkpoint, and do not reinstall solely to match a documentation HEAD.

Build a prerequisite table with: required operation, exact owner, present source, production adoption, retained proof, remaining work, and whether it blocks editing, enabling V2, or only a later installed claim. Include configured registry/service/exclusion injection, `take_for_v2`, tagged ownership, native inputs, local Codex adapter, gateway accounting/denial/cleanup, wrapper setup/final gate and registration order. Consume `take_for_v2` unchanged.

Do not require candidate-owned functionality to exist before its own implementation; instead bind its implementation and proof in the dependency plan. Do not treat the whole F14 runtime environment as a prerequisite to unrelated component work. Required environmental facilities must exist before the operations that depend on them.

## Phase 1 — Context assembly

Invoke `$context-engineering`. Assemble bounded authority, repo-truth, proof and review packets, targeting fewer than 2,000 focused lines. Use current source and code-intelligence evidence, not just a list of proposed contract names.

Produce this capsule before implementation:

```text
SLICE / OBJECTIVE:
SELECTED PACKET / INTEGRATED OUTCOME / EXACT COMPLETION CLAIM:
DEPENDENCY PROOF:
RETENTION / REPLACEMENT PURPOSE TABLE:
TARGET AUTHORITY BOUNDARY:
CURRENT PRODUCTION PATH / SEMANTIC STATUS:
  artifact existence:
  authority correctness:
  real-path adoption:
  enforcement correctness:
  smoke/e2e proof:
MUST-READ SECTIONS:
LIVE SOURCE / TESTS / PRECEDENT:
SIBLING SEAMS IN CONTEXT:
ALLOWED FILES / SYMBOLS / TESTS:
EXPLICIT NON-GOALS:
APPLICABLE CONTRACTS / REGRESSION GATES:
KNOWN CORRECTIONS OR CONFLICTS:
EXIT PROOF / STOP CONDITIONS:
REVIEW LENSES / BUDGET / CYCLE RECORD:
```

Before symbol edits, run the required upstream GitNexus impact analysis against this worktree, refresh its index when required without injecting unrelated files, and report callers, flows and HIGH/CRITICAL risk. Follow applicable independent pre-edit review requirements before multi-file changes. A stale Mac index is not this Linux candidate's source proof.

## Phase 2 — SPECIFY

Invoke `$spec-driven-development`. Freeze the one packet's authority decision, current/target path, internal interfaces, ownership transitions, immutable identities, required ordinary capabilities/security admission and per-mechanism
retention/replacement decisions, fail-closed behavior, retry/restart/cancellation semantics, exact positive/negative proof and non-goals. Invoke `$api-and-interface-design` only if the packet actually designs or changes an interface; consume settled callable definitions without redesigning them.

The controlling service path requires:

1. V2 selection before a non-E3 lease or helper/world side effect; missing/unequal configured authority rejects before transfer. Nonstream member dispatch stays rejected.
2. One `take_for_v2` and continuous original service-owned custody. No reconstructed capability, refreshed preparation/deadline or expiry cleanup after actual transfer.
3. I1's required `MemberRuntimeOwnershipV1::{NonE3,E3}`. Preserve required non-E3 agent capability and counted security admission; do not infer blanket
   retention of V1/UAA wire, construction, launcher or adapter implementations. No dummy/nullable common lease, mixed ownership, additive-only E3 field or E3 fallback.
4. I2's original adapter/gateway ownership: one original exclusion in the gateway; exact accounting-key/slot transfer and ACK-loss readback; no premature whole-attempt terminality while the gateway lives; early denial before late cleanup effects, complete historical process/cgroup registrations,
   namespace-before-cgroup removal and exclusion released last. Preserve the exact original adapter-to-
   gateway custody handoff; a Codex-only terminal object while the gateway is live is invalid.
5. Production registration/Active ordering and final-release gates from the exact owner contracts. No fabricated startup/registration evidence.

Apply the subset owned by this packet and record dependent obligations still unavailable. If a canonical contradiction or new callable/schema is genuinely necessary, stop source implementation and report the exact owner/span and smallest required control-plane correction. Do not quietly amend authority or dispatch a docs detour.

## Phase 3 — PLAN and TASKS

Invoke `$planning-and-task-breakdown` for the service dependency decomposition. Show the ordered packets needed to reach service-side V2 production proof, then select only the first dependency-ready green packet for this session.

For that packet freeze goal, exact owner/file/symbol/test allowlist, transitions, per-mechanism
purpose/disposition, non-goals, prerequisites, proof commands, exit gate and slice claims not yet permitted.
Choose the fence from actual dependencies, with no assumed three-file limit and no silent expansion
to all of E3-F. Derive its fence from the canonical E3-F catalog; do not treat the catalog as a blanket file-edit allowance. If no bounded green packet is supported, return the demonstrated blocker rather than switching back to shell bridge work.

The shell producer/REPL bridge is subsequent work. B01-B03-P1, D1/D2/D3/E4, E2/E2-RM/B2.2 changes, new provider paths, new schemas/authority/supervisors and non-Linux implementation remain out of scope.

## Phase 4 — Implementation

Invoke `$incremental-implementation` for this multi-file packet and `$test-driven-development` as its test phase begins. Keep the admitted increments small and green. Invoke `$debugging-and-error-recovery` on actual failures; reproduce/localize before patching. Invoke `$security-and-hardening` when changing the privilege, namespace, descriptor or cleanup boundary, rather than treating a generic checklist as proof.

Preserve all checkpoint work outside the frozen fence. No unrelated formatting/lint cleanup. Keep production V2 unavailable until its required real admission/launch/cleanup path and gates are ready; no success stub, bypass, fake Codex, fallback to V1/UAA or test-only activation of an unproved production path. No persistent daemon installation/restart, host policy/trust changes or external-provider credentials are authorized.

## Phase 5 — Packet verification and fresh review

Bind real selectors before running them; zero tests and silent skips are unavailable evidence. Starting launchers, only as applicable to the selected packet:

```bash
cargo +1.89.0 check -p world-service --locked
cargo +1.89.0 test -p world-service --lib --locked
cargo +1.89.0 test -p config-projection --lib --locked
cargo +1.89.0 test -p config-projection --test agent_config_projection_v1 --locked
cargo +1.89.0 test -p world-service --test e3_config_projection_activation_v1 --locked
```

Use the I1/I2 contract's proposed test groups only after implementing and binding their actual selectors. Test the selected transition and its negative, race, cancellation, partial-failure, retry/ACK-loss and cleanup consequences. Prove required ordinary capabilities, strict validation and counted admission, with each retained
V1/UAA implementation justified or replacement/removal explicitly selected under its owner. Preserve
unchanged E3-E assertions and historical evidence; compatibility-only tests do not dictate product
architecture, and safety assertions cannot be weakened to obtain green. Record exact commands, exits, counts, raw outputs and baseline/candidate bindings. Existing checkpoint failures remain evidence, not permission to weaken tests or claim green.

Invoke `$code-review-and-quality` when review starts. Use fresh, independent, read-only built-in `default` subagents with isolated context. Use the model, reasoning and standard/default speed settings required by the live user/AGENTS
instructions; do not infer an override from historical handoff text. Verify supported selection in the live environment and never manufacture effective-settings attestation. Supply exact authority, frozen subject/delta, gates, raw verification and unavailable proof; omit implementer reasoning, previous conclusions and success-asserting summaries. Reviewers cannot edit or expand scope. Use built-in spawn/wait/status/message/lifecycle tools; record actual agent IDs/statuses. No `codex exec`, shell reviewer, temporary-file transport or filesystem polling fallback. If the required built-in capability is unavailable, stop that gate honestly.

## Phase 6 — Verification wall and claim limits

Before the discovery fingerprint run deterministic formatting/scope checks, `git diff --check` and applicable focused verification. Run the packet's required lint/build/regression and real-path proof. Preserve unrelated inherited failures and stop if they invalidate required proof; do not declare acceptance with a required gate unavailable.

Removing the V2 rejection alone is not completion. Component tests establish their exact component
claim; production wiring requires the actual service path; installed proof additionally binds installed
artifacts/source and environment. Review, commit, integration, acceptance and closure are separate
status axes and none substitutes for another. The initial packet may prove a service primitive without
proving whole V2 launch; its capsule must name that narrower result. A service-side V2 production claim requires actual original preparation transfer, configured retained ownership, validated setup/release, authentic Active/registration and exact failure cleanup. Component fixtures do not replace that proof.

Bridge completion subsequently requires the real `take_for_v2 -> retained owner -> Active -> Registered` ordering through its actual direct/REPL paths. HTTP success, Start/Running or ReadyClosed alone never proves transfer. Do not undertake the shell bridge in this session merely because one service packet passes.

Full E3-F proof later retains the official pinned Codex 0.125.0, real gateway/wrapper and matching installed artifact/source provenance; one initial plus two sequential resumed turns of the same session; namespace-local HTTPS responder/test CA and synthetic credentials under the canonical containment rules; security/denial/cleanup/race/crash/ACK controls; and the applicable exact-baseline differential at `2b2fc6c50b40046dbeaeb5b316562fbd96480a2a`. The preserved checkpoint remains source ancestry, not a replacement for that differential baseline. Missing artifacts, privileges, namespace/TLS tools or containment stop the dependent claim. Do not provision/execute that full campaign without its separately selected scope.

## Phase 7 — Bounded review and remediation

Follow `llm-last-mile/runtime-refactor/contracts/development-review-and-remediation-contract.md`: one complete-subject discovery review/burst; one consolidated remediation of validated P1/P2; a different-fresh delta-focused closure review; at most two supplemental causal cycles for P1/P2 directly caused or unmasked by the preceding remediation. A CLEAN cycle is terminal; do not add another review just to satisfy older template reviewer counts. P3/P4 are nonblocking and inventoried, not an all-findings veto. Scope expansion, unrelated blockers or budget exhaustion produce a bounded non-completed stop.

Own a machine-auditable cycle record using `llm-last-mile/runtime-refactor/review-control/review-cycle-record.example.json`. Validate it after returns and before allowed next cycles with `llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py`. Preserve raw reviews independently from parent assessment. Do not invent a new review controller.

## Phase 8 — Control-plane closeout

Invoke `$documentation-and-adrs` when recording accepted changes. Update only the existing canonical owners/projections genuinely affected; do not create a parallel planning system or casually rewrite historical evidence.

Record the selected packet's authority owner, per-mechanism retention/replacement disposition,
real path/adoption status, exact proof and remaining dependencies. Keep all remaining F1–F14 visible
at the existing slice owner: F1–F4 classification/config/direct and REPL/toolbox producers/carriers;
F5–F7 service admission/transfer, retained ownership/resumes and events/run/cancel; F8–F10 pinned
adapter/native cleanup and registry/lineage/original-lease guards; F11–F12 gateway accounting,
namespace/security/revocation and wrapper/final gate; F13–F14 exact registry/native retry/crash/ACK
proof and real pinned-Codex initial plus two sequential resumed integrated turns. Mark only what
the selected packet actually proves; carry every remaining obligation forward. Do not close E3-F/E3 or mark the bridge proven from a prerequisite packet. E3-E remains closed. The bounded `E3F-RETAINED-PROOF-RETURN` and separately authorized P1 scheduling remain; no automatic successor admission follows.

## Phase 9 — Commit and report

Invoke `$git-workflow-and-versioning` when committing. Commit only the selected packet after its required proof and final CLEAN material review; do not amend/squash the preserved checkpoint. Run required GitNexus change detection and final scope/index checks before committing. No push, shared-branch merge or deployment is authorized by this prompt.

Report: actual starting/final identities; service dependency table; chosen first packet and why it precedes V2/bridge completion; exact changed files/symbols; proof results and limitations; reviewer identities/cycles/findings; canonical updates; commit or honest uncommitted stop state; remaining service/bridge/runtime obligations; and the next eligible packet without beginning it.

## Mandatory stop conditions

Stop dependent work for unexplained drift, contradictory authority, an out-of-fence required change, unprovable custody/identity, a required unavailable facility, invalid or missing required proof, unavailable mandatory fresh review, or a packet that cannot remain independently reviewable. Identify the exact issue and owner. Do not switch features, invent authority, relax proof or relabel a stopped candidate as complete.
