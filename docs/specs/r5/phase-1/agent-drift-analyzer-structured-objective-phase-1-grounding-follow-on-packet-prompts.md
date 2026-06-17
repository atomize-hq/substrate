# Structured Objective Phase 1 Grounding Follow-On Packet Prompts
Status: draft orchestration prompts created on 2026-06-16 for the then-active unchecked packets in
`docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md`;
reconciled on 2026-06-17 after the grounding follow-on family landed through `SO-G6`. These
prompts now remain as the historical orchestration artifact for packets `SO-G1` through `SO-G6`.
Already-landed docs-lock packet `SO-G0.*` is excluded, and deferred ask-first packets `SO-GX.*`
remain excluded until separately approved.

Shared authority for all packets:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md
- AGENTS.md

Shared orchestration rules for all packets:
- Spawn a fresh implementation subagent first on GPT-5.4 high.
- The implementation subagent prompt must begin with `/goal ` and must explicitly tell the subagent
  to use `$incremental-implementation`.
- After implementation finishes, inspect the diff and verification results, then commit the packet
  before review if files changed. Do not invent empty commits.
- Before each commit, run `gitnexus_detect_changes()`.
- Then spawn a fresh review subagent on GPT-5.4 high.
- The review subagent prompt must begin with `/goal ` and must explicitly tell the subagent to use
  `$code-review-and-quality`.
- If review flags issues, spawn a fresh fix subagent on GPT-5.4 high. Its prompt must begin with
  `/goal ` and must explicitly tell the subagent to use `$incremental-implementation`.
- Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
- Before editing any indexed Rust symbol, the implementation/fix subagent must run GitNexus impact
  analysis first and report any HIGH or CRITICAL blast radius before proceeding.
- Do not start the next packet until the current packet is review-clean.

---

## Packet 1 Prompt — SO-G1 Restore Additive Grounding Identifiers

````text
/goal Land Packet `SO-G1` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-G1` only, assuming `SO-G0` is already landed and no later `SO-G2+` packet has started.

Packet `SO-G1` scope only:
- add optional `section_index` / `clause_index` back to `ObjectiveEvidenceSpan`
- restore a local per-section clause index in the decomposition substrate
- populate those identifiers from the live clause-backed evidence path
- keep `start_char` / `end_char` optional and conservatively unset unless already cheap and real

Primary files:
- crates/agent-drift-analyzer/src/checkpoint/schema.rs
- crates/agent-drift-analyzer/src/context/objective.rs

Out of scope:
- new heading/adversarial regressions from `SO-G2`
- phase-1 doc reconciliation from `SO-G3+`
- real character offsets
- downstream migration or `comparison_key` derivation
- any change to `src/input.rs` or the harness-fix path

Required verification wall:
```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-G1` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming `SO-G0` is already landed and no later `SO-G2+` packet has started.

Use the `$incremental-implementation` skill.

Land only Packet `SO-G1`:
- add additive optional `section_index` / `clause_index` to `ObjectiveEvidenceSpan`
- restore a local clause index in the current decomposition substrate
- populate section/clause identifiers from the live clause-backed evidence path
- keep `start_char` / `end_char` optional and honest; do not invent fake offsets

Read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/src/checkpoint/schema.rs
- crates/agent-drift-analyzer/src/context/objective.rs

GitNexus rules:
- before modifying any indexed Rust symbol, run impact analysis first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly inside Packet `SO-G1`
- keep the patch additive and backward-compatible
- do not broaden into new tests beyond the minimum needed to prove this packet
- run the required verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `SO-G1` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-G1`:
- additive optional `section_index` / `clause_index`
- clause-backed population path
- honest handling of `start_char` / `end_char`
- no widening into `SO-G2+`, downstream migration, or input-contract changes

Review the verification story first, then the implementation diff.
List findings by severity and state clearly whether Packet `SO-G1` is review-clean or requires changes.
```

Fix subagent prompt template if review flags issues:

```text
/goal Fix the review findings for already-landed Packet `SO-G1` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` without widening beyond the packet boundary.

Use the `$incremental-implementation` skill.

Fix only the concrete Packet `SO-G1` findings from the latest review:
- keep the work scoped to additive grounding identifiers and their clause-backed population
- do not widen into `SO-G2+`, real char offsets, downstream migration, or input-contract changes
- rerun the packet verification wall after fixes

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```
````

---

## Packet 2 Prompt — SO-G2 Focused Grounding And Heading Regressions

````text
/goal Land Packet `SO-G2` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-G2` only, assuming `SO-G1` is already landed and no later `SO-G3+` packet has started.

Packet `SO-G2` scope only:
- add focused regressions proving goal and verification evidence spans carry populated `section_index` / `clause_index`
- add the duplicate-ish scope vs checklist ambiguity control
- add adversarial heading-classification controls for `Task constraints`, `Verification task`, `Output request`, `Questions to ask`, `Implementation steps`, and `What I need`

Primary files:
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- new schema or extractor changes unless strictly required to make the packet’s regression intent honest
- phase-1 doc reconciliation from `SO-G3+`
- acceptance harness implementation

Required verification wall:
```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-G2` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming `SO-G1` is already landed and no later `SO-G3+` packet has started.

Use the `$incremental-implementation` skill.

Land only Packet `SO-G2`:
- add focused regressions proving section/clause identifiers are populated for grounded goal + verification spans
- add the duplicate-ish grounding ambiguity control
- add the adversarial heading-classification controls

Read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/tests/checkpoints.rs
- crates/agent-drift-analyzer/src/context/objective.rs

GitNexus rules:
- if you must modify any indexed Rust symbol while making these regressions honest, run impact analysis first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay test-first and packet-scoped
- keep any code changes minimal and directly justified by the new regressions
- do not broaden into docs reconciliation or acceptance harness work
- run the required verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `SO-G2` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-G2`:
- whether the new tests actually prove the grounding identifiers matter
- whether the duplicate-ish ambiguity case would fail without correct section/clause grounding
- whether the adversarial heading cases are well targeted and not implementation-detail tests
- whether any code changes stayed minimal and packet-scoped

Review the tests first, then any implementation diff. List findings by severity and say clearly whether Packet `SO-G2` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Fix the review findings for already-landed Packet `SO-G2` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` without widening beyond the packet boundary.

Use the `$incremental-implementation` skill.

Fix only the concrete Packet `SO-G2` findings from the latest review:
- keep the work centered on checkpoint regressions and any minimal supporting changes
- do not widen into docs reconciliation, acceptance harness work, or downstream migration
- rerun the packet verification wall after fixes

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```
````

---

## Packet 3 Prompt — SO-G3 Reconcile Existing Phase-1 Docs To Landed Reality

````text
/goal Land Packet `SO-G3` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-G3` only, assuming `SO-G2` is already landed and no later `SO-G4+` packet has started.

Packet `SO-G3` scope only:
- reconcile the existing phase-1 tasks doc so landed work is not still presented as open
- reconcile the existing phase-1 plan so it names the real remaining seams honestly
- preserve auditability instead of pretending the old packet structure never existed

Primary files:
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md

Out of scope:
- code changes
- packet-prompt regeneration unless absolutely necessary and explicitly justified
- new acceptance harness design beyond what is needed to sequence the next packet honestly

Required verification wall:
```bash
git diff --stat
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-G3` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming `SO-G2` is already landed and no later `SO-G4+` packet has started.

Use the `$incremental-implementation` skill.

Land only Packet `SO-G3`:
- update the existing phase-1 tasks doc so landed schema bridge, sidecar exposure, section/clause decomposition, and preliminary assembly are not still presented as wholly pending
- update the existing phase-1 plan so the remaining seams are honest: grounding restoration, comparison-key derivation, acceptance harness, and deferred downstream migration
- preserve historical auditability rather than rewriting the story as if the original plan never existed

Read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md
- AGENTS.md

Execution rules:
- docs-only unless a tiny clarification elsewhere is strictly required
- do not silently claim later packets are landed if they are not
- do not widen into code or packet-prompt generation
- verify with manual doc review plus `git diff --stat`

Return with: changed files, verification performed, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `SO-G3` doc reconciliation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is accurate, honest, and ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-G3`:
- whether the phase-1 plan/tasks now match landed crate reality
- whether already-landed work is distinguished from remaining work
- whether the docs preserve auditability instead of silently flattening history
- whether the packet stayed docs-only and packet-scoped

List findings by severity and say clearly whether Packet `SO-G3` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Fix the review findings for already-landed Packet `SO-G3` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` without widening beyond the packet boundary.

Use the `$incremental-implementation` skill.

Fix only the concrete Packet `SO-G3` findings from the latest review:
- keep the work docs-only
- do not widen into code changes or packet-prompt generation
- recheck the affected docs manually before handing back

Return with: changed files, verification performed, residual risks if any, and the exact commit message you recommend.
```
````

---

## Packet 4 Prompt — SO-G4 Lock The Comparison-Key Guardrail

````text
/goal Land Packet `SO-G4` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-G4` only, assuming `SO-G3` is already landed and no later `SO-G5+` packet has started.

Packet `SO-G4` scope only:
- record explicitly that current `comparison_key` is provisional
- make clear it still mirrors display text and is not yet the approved semantic bridge
- block downstream migration from treating the current key as ready

Primary files:
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md

Out of scope:
- implementing actual `comparison_key` derivation
- downstream consumer migration
- code changes

Required verification wall:
```bash
git diff --stat
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-G4` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming `SO-G3` is already landed and no later `SO-G5+` packet has started.

Use the `$incremental-implementation` skill.

Land only Packet `SO-G4`:
- update the phase-1 docs so they explicitly say current `comparison_key` is provisional
- state clearly that it still mirrors display text and must not anchor downstream migration yet
- keep the work docs-only and tightly scoped to the comparison-key guardrail

Read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md
- AGENTS.md

Execution rules:
- docs-only
- no new code, no hidden downstream migration, no packet-prompt regeneration
- verify with manual doc review plus `git diff --stat`

Return with: changed files, verification performed, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `SO-G4` doc guardrail in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is accurate, unambiguous, and ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-G4`:
- whether the docs clearly state current `comparison_key` is provisional
- whether the docs block downstream reliance until real derivation lands
- whether the packet stayed docs-only and narrowly scoped

List findings by severity and say clearly whether Packet `SO-G4` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Fix the review findings for already-landed Packet `SO-G4` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` without widening beyond the packet boundary.

Use the `$incremental-implementation` skill.

Fix only the concrete Packet `SO-G4` findings from the latest review:
- keep the work docs-only
- do not implement comparison-key derivation or downstream migration
- recheck the affected docs manually before handing back

Return with: changed files, verification performed, residual risks if any, and the exact commit message you recommend.
```
````

---

## Packet 5 Prompt — SO-G5 Prepare The Objective-Acceptance Harness Seam

````text
/goal Land Packet `SO-G5` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-G5` only, assuming `SO-G4` is already landed and no later `SO-G6+` packet has started.

Packet `SO-G5` scope only:
- make the objective-acceptance harness the explicit next packet after grounding
- require it to validate structured fields, role spans, grounding refs, forbidden promotions, compatibility rendering, and unknown-field correctness
- keep this packet as sequencing/authority work, not actual acceptance-harness implementation

Primary files:
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-plan.md

Out of scope:
- implementing the acceptance harness
- adding fixtures
- code changes

Required verification wall:
```bash
git diff --stat
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-G5` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming `SO-G4` is already landed and no later `SO-G6+` packet has started.

Use the `$incremental-implementation` skill.

Land only Packet `SO-G5`:
- update the docs so the objective-acceptance harness is the explicit next packet after grounding restoration
- require that future harness to validate structured fields, role spans, grounding refs, forbidden promotions, compatibility rendering, and unknown-field correctness
- keep the work strictly sequencing/authority-only

Read first:
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md
- AGENTS.md

Execution rules:
- docs-only
- do not implement the harness or fixtures
- do not widen into other follow-on planning beyond what this packet requires
- verify with manual doc review plus `git diff --stat`

Return with: changed files, verification performed, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `SO-G5` sequencing update in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is accurate, explicit, and ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-G5`:
- whether the next acceptance packet is explicit rather than vague
- whether the expected harness scope is concrete and matches the evaluation authority
- whether the packet stayed docs-only and did not smuggle in implementation work

List findings by severity and say clearly whether Packet `SO-G5` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Fix the review findings for already-landed Packet `SO-G5` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` without widening beyond the packet boundary.

Use the `$incremental-implementation` skill.

Fix only the concrete Packet `SO-G5` findings from the latest review:
- keep the work docs-only
- do not implement acceptance harness code or fixtures
- recheck the affected docs manually before handing back

Return with: changed files, verification performed, residual risks if any, and the exact commit message you recommend.
```
````

---

## Packet 6 Prompt — SO-G6 Validation And Closeout

````text
/goal Land Packet `SO-G6` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-G6` only, assuming `SO-G1` through `SO-G5` are already landed.

Packet `SO-G6` scope only:
- run the focused checkpoint wall and the full analyzer wall on the grounding patch
- if validation exposes a real packet-scoped defect in already-landed `SO-G1` or `SO-G2` work, fix only what is required to make the validation story honest and green
- keep closeout honest; do not claim broader phase completion beyond this packet

Primary files:
- no file changes required unless validation exposes a packet-scoped defect

Out of scope:
- new feature work
- acceptance harness implementation
- downstream migration
- unrelated cleanup

Required verification wall:
```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-G6` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming `SO-G1` through `SO-G5` are already landed.

Use the `$incremental-implementation` skill.

Land only Packet `SO-G6`:
- run the focused checkpoint wall and the full analyzer wall
- if validation exposes a real packet-scoped defect in the already-landed grounding patch, fix only what is required to make the packet honest and green
- keep any fixes tightly bounded to previously approved packet scope

Read first:
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

GitNexus rules:
- if a validation-discovered fix requires editing any indexed Rust symbol, run impact analysis first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit if files changed

Execution rules:
- validation first
- no opportunistic broadening
- if no files changed, do not invent a commit; report verification-only completion honestly
- rerun the verification wall after any fix batch

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `SO-G6` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether the packet is honestly closed out and ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-G6`:
- whether the stated verification wall actually ran and is green
- whether any fixes stayed tightly packet-scoped
- whether the closeout language is honest and does not overclaim broader phase completion

Review the verification story first, then any fix diff. List findings by severity and say clearly whether Packet `SO-G6` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Fix the review findings for already-landed Packet `SO-G6` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` without widening beyond the packet boundary.

Use the `$incremental-implementation` skill.

Fix only the concrete Packet `SO-G6` findings from the latest review:
- keep the work limited to validation-discovered defects inside the already approved grounding patch scope
- do not widen into new feature work, acceptance harness implementation, or downstream migration
- rerun the full packet verification wall after fixes

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```
````
