# Spec: R7 Bounded Delegated-Session Support

Canonical path:
`docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-spec.md`

Status: **IMPLEMENTATION-READY / R7-PROMOTE, R7-0, R7-1, AND R7-2 COMPLETE /
CHECKPOINT-DOC COMMIT `78a168c09` FRESH INDEPENDENT REVIEW CLEAN / `CTX-R7-03` PROVEN / R7-3
SOLE ACTIVE PHASE AT ENTRY ONLY / ACTIVE PACKET NONE / TRANSITION/FIX SERIES `e27d82580` + `305e40bf2` FRESH INDEPENDENT BUILT-IN `default` `REVIEW CLEAN` AFTER FIX `305e40bf2` CORRECTED THE FIRST REVIEW'S STALE R7 PLAN PARAGRAPH / R7-3.1 NEXT, UNCHECKED, AND UNSTARTED / R7-3 ANALYZER/PRODUCTION
IMPLEMENTATION UNSTARTED / R7-4..R7-6 AND R8 BLOCKED / PROMPT 1 SELECTORS `PHASE_ID: R7-3` /
`ACTIVE_PACKET: none` PREPARED AND ELIGIBLE BUT NOT INVOKED**

## Assumptions I'm Making

1. R6 is **CLOSED**; `R6-CLOSE` and `CTX-R6-17` are complete. Promotion series `455d0ed90` +
   `876ac55de` received fresh independent built-in `default` `REVIEW CLEAN`, so `R7-PROMOTE` is
   complete. Transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh independent
   built-in `default` `REVIEW CLEAN`. `R7-0.1` series `a9e75f149` + `55bea5fa5` + `faff68ac6` and
   fixture-only `R7-0.2` commit `fa85cd4b8` are fresh independent built-in `default` `REVIEW CLEAN`,
   completing `R7-0`. Transition/fix series `339744dff` + `d20cac6a9` also received fresh
   independent built-in `default` `REVIEW CLEAN`. `R7-1.1` series `e65127720` + `685cf843b`,
   `R7-1.2` commit `4d122cd9f`, and `R7-1.3` commit `e865eee13` are fresh independent built-in
   `default` `REVIEW CLEAN`. Checkpoint-doc commit `1cae7d693` received fresh independent built-in
   `default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task commits
   `c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` + `75a353e46`, are fresh independent
   built-in `default` `REVIEW CLEAN`; the fix reconciled the summary-vs-checkpoint blocker. R7-2.1,
   R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc commit
   `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-2
   exit gate and proving `CTX-R7-03`. R7-2 is complete. Only R7-3 is active at entry with packet
   `none`; transition/fix series `e27d82580` + `305e40bf2` received fresh independent built-in `default` `REVIEW CLEAN` after fix `305e40bf2` corrected the first review's stale R7 plan paragraph. `R7-3.1` is next, unchecked,
   and unstarted; R7-3 analyzer/production implementation has not started. `R7-4..R7-6` and R8
   remain blocked. Prompt 1 selectors `PHASE_ID: R7-3` / `ACTIVE_PACKET: none` are prepared and
   eligible but have not been invoked.
2. The first supported delegated model is one parent plus directly spawned children. Nested child
   descendants remain visible as bounded residue rather than being recursively joined in the first
   R7 implementation.
3. Raw Codex rollout metadata is untrusted input. A semantic parent/child link requires reciprocal
   agreement between the parent spawn result and child `session_meta` source metadata.
4. Parent and child progress remain separate trajectory facts. The analyzer must never manufacture
   child progress from parent waits, orchestration, summaries, or prose.
5. The compactor remains the only layer that parses raw rollout JSONL. The analyzer and sentinel
   consume typed bundle/checkpoint contracts and do not add their own raw collaboration parser.
6. R8 still owns broad sentinel interpretation consolidation. R7 may add only compatibility and
   compact presentation required to carry the new analyzer semantics end to end.

## Implementation Promotion Gate — R7-2 Behavior Complete / Receipt Review Pending

This document is now implementation-ready authority. The R6 prerequisites audited during the
now-complete `R7-PROMOTE` phase are satisfied:

1. the R6 scorer-by-context applicability audit is complete;
2. every material scoring surface has exactly one terminal disposition: **Cutover complete**,
   **Fit-for-purpose exception**, **Merged/deprecated**, or **Explicitly deferred outside R6 with
   justification**; ordinary “still open” does not qualify;
3. the broad R6 acceptance claims have behavioral proof or have been narrowed honestly;
4. the named R6 closure controls are resolved and the R6 finding is `CLOSED`; and
5. root landing-order authority, the R6 MAP, root SPEC/tasks, and all R7 gate/status sections agree.

Promotion series `455d0ed90` + `876ac55de` completed the content/gate audit, reconciled the
canonical mirrors, and received fresh independent built-in `default` `REVIEW CLEAN`.
`R7-PROMOTE` is complete. Transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh
independent built-in `default` `REVIEW CLEAN`; it made `R7-0` active at entry only with packet
`none`. The exact docs-only `R7-0.1` contract verification passed, and series `a9e75f149` +
`55bea5fa5` + `faff68ac6` is fresh independent built-in `default` `REVIEW CLEAN`. Fixture-only
`R7-0.2` commit `fa85cd4b8` covers reciprocal, parent-only, child-only, conflict, multi-child,
nested-depth residue, and single-agent raw shapes; focused parser/privacy proof is `2 / 2`, full
compactor proof is `25 / 25` including end-to-end `2 / 2`, privacy scans over `24` rows found zero
private markers and zero raw UUIDs, and fresh independent review returned `REVIEW CLEAN`. `R7-0` is
complete. Transition/fix series `339744dff` + `d20cac6a9` is fresh independent built-in `default`
`REVIEW CLEAN`. R7-1 task series `e65127720` + `685cf843b`, `4d122cd9f`, and `e865eee13` are fresh
independent built-in `default` `REVIEW CLEAN`. Focused delegation-link proof passes `6 / 6`; direct-
closure end-to-end and CLI proof pass `6 / 6` and `2 / 2`; the full compactor wall passes `36` unit/
integration tests plus `3` doctests; formatting, clippy, diff, and staged GitNexus gates are green.
Link/session/file ordering is deterministic, and no raw private rollout data was added.
Checkpoint-doc commit `1cae7d693` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-1 exit gate. R7-1 is complete. Operator decision
`R7-2-HIGH-IMPACT-ANALYZER-CONTRACT-01: A` authorized the bounded high-impact analyzer seam. R7-2
task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 implementation/fix series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN` after the fix reconciled
the summary-vs-checkpoint blocker. At implementation HEAD `75a353e46`, input passes `16 / 16`,
delegation matches pass `39` total, checkpoint matches pass `172` total, full analyzer passes `417 /
417`, and format, analyzer clippy `-D warnings`, and diff checks are green. Staged GitNexus gates
reported R7-2.1 LOW / `0` affected processes, R7-2.2 MEDIUM / `1`, R7-2.3 HIGH / `9` within the
authorized decision, and the fix MEDIUM / `2`. Public v0.8 with v0.7 compatibility, graph-derived
roles and ids, `Linked`/`Partial` visibility, fail-closed conflicts, deterministic `RowRef` evidence,
JSON-summary parity, and separate trajectories are proven. No R7-3, R7-4, sentinel, or R8 work
leaked into the series. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Only R7-3 is active at entry with packet
`none`; transition/fix series `e27d82580` + `305e40bf2` received fresh independent built-in `default` `REVIEW CLEAN` after fix `305e40bf2` corrected the first review's stale R7 plan paragraph. `R7-3.1` is next, unchecked,
and unstarted; R7-3 analyzer/production implementation has not started. `R7-4..R7-6` and R8 remain
blocked. Prompt 1 selectors `PHASE_ID: R7-3` / `ACTIVE_PACKET: none` are prepared and eligible but
have not been invoked. R7 must continue to preserve the stable
ordinary single-session baseline and must not absorb baseline scorer semantics.

## Objective

Add bounded, evidence-backed delegated-session semantics to the hybrid drift stack so a verified
parent/child run is represented as linked but separate trajectories.

Success means:

- the compactor can explicitly include directly linked child rollouts for a requested parent;
- reciprocal linkage is validated and conflicting or missing evidence fails closed;
- the analyzer classifies verified parents and children from the link graph rather than prose alone;
- parent-visible progress and child-visible progress remain separately attributable;
- existing drift scorers run against the trajectory whose evidence they actually observe;
- opaque or partial traces do not produce claims about unseen child implementation;
- non-delegated behavior remains unchanged; and
- replay/live sentinel paths accept the analyzer-owned representation without absorbing R8.

## Tech Stack

- Rust 2021 workspace with MSRV and commands defined by the root `Cargo.toml` and `AGENTS.md`
- Raw rollout parsing through `unified-agent-api-codex` in `agent-session-compactor`
- Compactor bundle contract currently `v0.2`
- Analyzer checkpoint contract public at `v0.8` with readable `v0.7` compatibility
- JSON/JSONL artifacts serialized with `serde`
- Existing analyzer `SessionProgress`, `SessionArchetype`, `TurnContext`, typed outcome evidence,
  `DelegationContext`, `DelegationTopology`, and `ChildWorkVisibility`
- Existing sentinel replay/live compatibility and per-session cursor maps

## Commands

Baseline and code-intelligence capture:

```bash
git status --short --branch
npx gitnexus status
npx gitnexus query -r 97a0-substrate \
  "delegated session parent child linkage child work visibility orchestration progress"
```

Focused compactor verification:

```bash
cargo test -p agent-session-compactor -- --nocapture
cargo test -p agent-session-compactor --test end_to_end -- --nocapture
```

Focused analyzer verification:

```bash
cargo test -p agent-drift-analyzer delegation -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer --test dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Sentinel compatibility verification:

```bash
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Workspace closeout:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
git diff --check
npx gitnexus detect-changes -r 97a0-substrate
```

## Project Structure

```text
docs/specs/r7/
  MAP.md                                              # R6 handoff and R7 packet authority
  agent-drift-analyzer-delegated-session-support-r7-spec.md
  agent-drift-analyzer-delegated-session-support-r7-plan.md
  agent-drift-analyzer-delegated-session-support-r7-tasks.md

crates/agent-session-compactor/src/
  ingest/codex_rollout.rs                             # raw parent/child metadata extraction
  discovery.rs                                        # requested-root and direct-child artifact discovery
  export/mod.rs                                       # additive bundle linkage contract
  normalize/mod.rs                                    # existing row normalization; no semantic joining

crates/agent-drift-analyzer/src/
  input.rs                                             # typed link validation and session graph loading
  inference/mod.rs                                     # existing heuristic boundary, narrowed by verified links
  checkpoint/schema.rs                                 # public v0.8 delegation trajectory contract
  checkpoint/mod.rs                                    # role/visibility derivation and per-session progress
  scoring/*.rs                                         # trajectory-local guardrails only

crates/agent-drift-sentinel/src/
  input.rs                                             # v0.8 compatibility
  real_session_live.rs                                 # linked-closure validation and per-session cursors
  operator_surface.rs                                  # compact analyzer-owned delegation presentation
```

## Interface Contract

### Compactor bundle extension

The existing `v0.2` manifest gains an additive, serde-defaulted `delegation_links` list. Existing
consumers and bundles remain valid. The linked-child closure is explicit, not a silent change to
ordinary `--session-id` behavior.

Conceptual contract:

```rust
pub struct DelegationLink {
    pub parent_session_id: String,
    pub child_session_id: String,
    pub depth: u32,
    pub state: DelegationLinkState,
    pub parent_evidence: Vec<RowRef>,
    pub child_evidence: Vec<RowRef>,
}

pub enum DelegationLinkState {
    Verified,
    ParentOnly,
    ChildOnly,
    Conflicting,
}
```

Rules:

- only `Verified` links authorize child semantic import;
- reciprocal equality is required: parent result child id equals child metadata id, and child
  `parent_thread_id` equals the parent session id;
- self-links, duplicate contradictory parents, cycles, malformed ids, and mismatched depth are
  rejected or exported as non-semantic conflict evidence;
- the first implementation includes direct children only; deeper descendants are recorded as
  unsupported residue, not silently dropped or recursively interpreted;
- one-sided links may improve opacity diagnostics but never authorize child progress or drift.

### Analyzer checkpoint extension

Checkpoint schema `v0.8` promotes delegation into an analyzer-owned required field. Progress remains
on the checkpoint's own session trajectory; it is not nested or copied across sessions.

Conceptual contract:

```rust
pub struct DelegationContext {
    pub topology: DelegationTopology,
    pub parent_session_id: Option<String>,
    pub child_session_ids: Vec<String>,
    pub child_work_visibility: ChildWorkVisibility,
    pub confidence: Confidence,
    pub supporting_evidence: Vec<EvidenceRef>,
    pub counter_evidence: Vec<EvidenceRef>,
}

pub enum DelegationTopology {
    SingleAgent,
    DelegatingParent,
    DelegatedChild,
    MixedOrAmbiguous,
}

pub enum ChildWorkVisibility {
    None,
    Linked,
    Partial,
    Opaque,
}
```

`Linked` means every direct child claimed by the parent and included in the bounded closure has a
verified reciprocal link and its own analyzer trajectory. `Partial` means at least one direct child
is verified but another claimed child is missing, unsupported, or conflicting. `Opaque` means
delegation is visible but no child trajectory is semantically authorized.

### Error semantics

- malformed raw metadata is input evidence, not an analyzer panic;
- contradictory reciprocal linkage is surfaced as `Conflicting` and cannot become `Linked`;
- missing child artifacts leave the parent `Opaque` or `Partial` and do not fail ordinary parent
  analysis unless the caller explicitly required a fully linked closure;
- an explicit linked-child mode may return a typed error for ambiguous duplicate child artifacts,
  self-links, or graph cycles;
- legacy compactor bundles and analyzer checkpoints remain readable through existing compatibility
  paths.

## Code Style

Prefer typed, additive contracts and deterministic ordering. Do not encode semantic relationships in
reason strings and then parse those strings downstream.

```rust
match link.state {
    DelegationLinkState::Verified => authorize_child_trajectory(link),
    DelegationLinkState::ParentOnly
    | DelegationLinkState::ChildOnly
    | DelegationLinkState::Conflicting => preserve_as_limited_evidence(link),
}
```

Conventions:

- sort links by `(parent_session_id, child_session_id)` before export;
- deduplicate evidence by stable `RowRef` plus reason;
- use `anyhow::Context` or typed crate errors at file and contract boundaries;
- keep role derivation pure and unit-testable;
- prefer additive optional fields for the compactor `v0.2` extension;
- use a checkpoint schema bump for required public analyzer semantics;
- never use prose, timestamps, nickname, or nearby filenames as sole link authority.

## Testing Strategy

### Contract tests

- parse current structured child `session_meta.source.subagent.thread_spawn` metadata;
- preserve parent spawn-result child ids and child parent ids in the compactor contract;
- round-trip legacy manifests with no `delegation_links`;
- reject or limit self-links, conflicting parents, duplicate artifacts, and mismatched reciprocal ids;
- prove deterministic link and session ordering.

### Analyzer tests

- verified parent becomes `DelegatingParent` with a linked child id;
- verified child becomes `DelegatedChild` with the parent id;
- parent-only, child-only, missing-artifact, and conflict cases remain `Partial`, `Opaque`, or
  `MixedOrAmbiguous` without child semantic claims;
- child checkpoints retain their own archetype, progress, and drift scores;
- parent checkpoints retain parent-visible orchestration semantics and do not absorb child progress;
- existing single-agent and delegated-parent guardrail fixtures remain stable.

### Acceptance matrix

- one parent / one verified child;
- one parent / multiple verified direct children;
- one verified child plus one missing child;
- parent spawn result with no child file;
- child metadata with no parent result;
- conflicting parent/child ids;
- nested depth greater than one, explicitly left as residue in the first R7 cut;
- ordinary single-agent control;
- parent wait loop while child advances;
- child stalls while parent orchestration remains clean.

### Real-session proof

Use sanitized derivatives of known delegated sessions already cited by R3.75/R5.75, including the
verified parent/child pair `019e93f8-...` -> `019e93fa-...`. Never commit raw private rollout text.
The proof report must stratify `single_agent`, `delegating_parent_linked`,
`delegating_parent_partial`, `delegating_parent_opaque`, `delegated_child`, and
`mixed_or_ambiguous`.

## Boundaries

### Always do

- run GitNexus impact analysis before editing every function/type implementation symbol;
- preserve raw provenance for both sides of every link;
- require reciprocal verification before child semantics are authorized;
- keep parent and child progress in separate session checkpoints;
- preserve conservative R3.75/R5 fallback behavior whenever linkage is absent;
- keep non-delegated output stable aside from the required v0.8 delegation field;
- update compactor, analyzer, sentinel compatibility, fixtures, and docs in packet order;
- run `gitnexus detect-changes` before each implementation commit.

### Ask first about

- supporting recursive linkage beyond direct children;
- making linked-child discovery implicit for every compactor `--session-id` run;
- adding or renaming a `DriftClass` variant;
- changing compactor bundle `schema_version` instead of using an additive field;
- aggregating child progress or drift into a parent score/status;
- changing scheduler or intervention policy.

### Never do

- infer child work from parent orchestration, waiting, or summaries alone;
- let a one-sided or conflicting link authorize child progress or drift;
- parse raw rollout JSON in the analyzer or sentinel;
- flatten parent and child rows into one synthetic session;
- silently include arbitrary descendants or unrelated sessions;
- reopen semantic-goal-drift, `R6-3.X.3`, or conditional `R6-4` without new evidence;
- perform R8 sentinel interpretation consolidation inside R7;
- commit raw private rollout content as fixtures.

## Success Criteria

1. R6 authority docs state that `dead_end_thrash` is already cut over and that R7 is next.
2. The compactor preserves deterministic, reciprocal direct parent/child linkage with row-level
   provenance and legacy-manifest compatibility.
3. Explicit linked-child compaction includes only the verified direct closure and reports unsupported
   or conflicting residue.
4. Analyzer checkpoint `v0.8` exposes required delegation topology, ids, visibility, confidence,
   and evidence.
5. Verified children are analyzed as separate `DelegatedChild` trajectories with their own
   `SessionProgress`, archetype, and drift scores.
6. Parent progress never includes or impersonates child implementation progress.
7. Parent waits/orchestration cannot be emitted as child thrash, child advancement, or child drift.
8. Missing or contradictory child evidence yields partial/opaque/ambiguous semantics, not a false
   linked claim.
9. The delegated acceptance matrix and named sanitized real-session witnesses pass.
10. Existing non-delegated acceptance, R5 progress, R6 scorer, analyzer, and sentinel walls pass.
11. Replay and live sentinel paths accept v0.8 linked trajectories with per-session cursor safety.
12. R8 consolidation, recursive execution graphs, and new drift taxonomy remain outside this family.

## Open Questions

No blocking design question remains for the first implementation packet. The following decisions are
intentionally evidence-gated later in the family:

- whether linked-child discovery should become the default after R7 acceptance proves it stable;
- whether any delegated-specific `DriftClass` is justified after per-trajectory scorer evidence;
- whether depth greater than one should become a later R7 follow-on or a separate phase.
