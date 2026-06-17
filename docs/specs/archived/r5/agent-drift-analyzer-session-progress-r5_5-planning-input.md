# Agent Drift Analyzer Session Progress R5.5 Planning Input

Status: planning-input inventory created on 2026-06-11 after post-landing R5 review, third-party
correctness review, and bounded unseen-session manual smoke.

## Why This Doc Exists

`R5` is landed, but follow-on review surfaced a bounded set of real issues and hardening gaps that
should shape a later `R5.5` planning session.

This doc is **not** an implementation plan and **not** a packet ledger. It is the durable source
for:

1. what still appears wrong or underfit in live repo reality,
2. which items are highest priority,
3. what file/code anchors a planning session should start from,
4. what acceptance tests a later `R5.5` plan should require.

## Current Repo Reality

As of 2026-06-11:

1. `R5` is landed in code and docs.
2. The original formatting blocker is no longer open; the worktree includes
   `5073ef445 chore: run fmt --all`.
3. The remaining work is primarily analyzer-semantic hardening and acceptance-coverage deepening,
   not unfinished `R5` surface-area landing.

## Sources For This Inventory

This planning input is grounded in three post-landing checks:

1. live code review of the landed `R5` analyzer/sentinel implementation,
2. a third-party correctness review focused on progress semantics and scorer-readiness,
3. bounded manual smoke on unseen real sessions through:
   - `agent-session-compactor`
   - `agent-drift-analyzer`
   - `agent-drift-sentinel --mode replay`

## Priority Summary

### P0: Must fix before treating `R5` progress as safe `R6` scorer input

1. Troubleshooting can overclaim `advancing` when the same failure repeats after an overlapping
   edit.

### P1: High-value next fixes that materially improve real-session trust

1. Objective extraction can still be polluted by developer/system boilerplate instead of the true
   `/goal`.
2. Real-rollout acceptance coverage is still thin for implementation and closeout/review flows.
3. JS/TS verifier role coverage remains incomplete in the attempt classifier.

### P2: Consistency and observability hardening

1. `parent_visible_orchestration_progress(...)` bypasses `finalize_progress(...)`.
2. Delegation caps add a limiting signal but do not always surface limiting evidence as
   `counter_evidence`.

### P3: Cleanup / deeper design follow-ups

1. `diagnostics.rs` still starts with `#![allow(dead_code)]`.
2. The progress window remains implicit rather than being represented as a named internal seam.
3. Some `R5` task-doc checklist text is intentionally stale and should be cleaned when the next
   follow-up doc pass happens.

## Follow-up Inventory

---

## R5.5-FU-001: Troubleshooting repeated-failure path can overclaim advancement

- **Priority:** P0
- **Area:** troubleshooting progress semantics
- **Status:** open
- **Why it matters:** this is the main correctness risk before any `R6` scorer consumes
  `troubleshooting_frontier = advancing`.

### Current repo reality

The current troubleshooting path still allows this sequence to be labeled too optimistically:

1. a verifier fails,
2. the agent edits an overlapping scope,
3. the same normalized failure signature repeats,
4. `FailingScopeEdited` can still contribute enough positive evidence for an `advancing` result if
   the repeated-signature negative path was skipped because overlap was not `None`.

### Code anchors

- `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
  - repeated-signature negative path: around `380-412`
  - overlapping-edit positive path: around `413-431`
  - generic progress outcome classifier: around `1263-1288`

### Intended `R5.5` direction

`FailingScopeEdited` should remain useful evidence, but it should not by itself justify clean
troubleshooting advancement.

For repeated exact/strong-fuzzy signatures with overlapping edits, the next implementation should
conservatively yield `mixed` or `stalled` unless there is real verifier progress such as:

1. `VerificationClean`,
2. `FailureFrontierAdvanced`,
3. `FailureCountReduced`,
4. a real `BlockedBeforeTarget -> TargetExercised` improvement,
5. a later-stage comparable failure on the same target.

### Acceptance seeds for later `R5.5` planning

Add a focused regression proving:

1. same troubleshooting signature,
2. scoped overlapping edit,
3. same troubleshooting signature again,
4. result is **not** `advancing`,
5. result includes:
   - `FailureSignatureRepeated`
   - `FailingScopeEdited`
6. result is `mixed` or `stalled`.

### Planning note

This item should likely be its own first packet in any future `R5.5` sequence.

---

## R5.5-FU-002: Objective extraction can still be polluted by boilerplate

- **Priority:** P1
- **Area:** task frame / progress window comparability / real-session archetype selection
- **Status:** open

### Current repo reality

In bounded manual smoke on unseen real sessions, at least some first-checkpoint objectives still
resolved to injected developer/system boilerplate instead of the true user `/goal`.

Examples observed during the 2026-06-11 manual smoke pass included sessions like:

1. `019eb430-6f9a-7a03-9a63-cb451b654795`
2. `019eb47f-0118-7e90-8291-30a1fb93769e`

The effect is that planning/troubleshooting classification can be skewed before the real packet
objective becomes dominant.

### Code anchors

- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
  - phase/objective row logic: around `1889-1974`
  - current focus filtering already excludes `AGENTS.md instructions`, `<skill>`, and
    `Available skills`, but not all visible boilerplate classes

### Intended `R5.5` direction

Tighten objective-row filtering and selection so the analyzer prefers:

1. literal `/goal` rows,
2. explicit user requests,
3. thread-goal / goal-creation objective text,

over developer/system scaffolding such as:

1. permissions blocks,
2. app/plugin/skills boilerplate,
3. memory-system boilerplate,
4. similar non-task instruction frames.

### Acceptance seeds for later `R5.5` planning

Add coverage where:

1. boilerplate appears before the true `/goal`,
2. the checkpoint objective still resolves to the true task objective,
3. comparability and progress classification are anchored to the real goal rather than the
   instruction scaffold.

---

## R5.5-FU-003: Real-rollout acceptance coverage is still thin for implementation and closeout

- **Priority:** P1
- **Area:** semantic acceptance wall / corpus authority
- **Status:** open

### Current repo reality

The bounded `R5` semantic wall is real and useful, but its committed corpus still only includes:

1. one annotated troubleshooting real-rollout case,
2. one annotated closeout real-rollout case,
3. one annotated planning real-rollout case,
4. one synthetic implementation-advancing case,
5. one synthetic parent-visible opaque case,
6. one synthetic planning-advancing case.

That means implementation advancement remains synthetic-only in the dedicated acceptance corpus.

### Code/doc anchors

- `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- `docs/specs/r5/DESIGN-r5-validation-and-rollout-protocol.md`

### Why it matters

The unseen-session smoke suggested that real packet implementation and review flows often collapse
into:

1. `planning_convergence`, or
2. `troubleshooting_frontier`,

instead of surfacing the intended implementation / closeout dimensions with enough confidence.

This is an underfitting / acceptance-depth issue more than a proven contract-break issue, but it is
important before treating `R5` as semantically mature on live packet work.

### Intended `R5.5` direction

Expand the bounded semantic corpus with additional **real** rollout fixtures, especially for:

1. implementation advancement,
2. review/closeout narrowing,
3. review-findings / reopen / re-verify flows.

### Acceptance seeds for later `R5.5` planning

Require at least:

1. one annotated real implementation-progress case expected to hit
   `implementation_verification_wall`,
2. one annotated real closeout/review case expected to hit
   `verification_closeout_narrowing` or an intentionally conservative closeout result,
3. one real review-findings case that proves reopen / re-verify behavior is classified honestly.

---

## R5.5-FU-004: JS/TS verifier role coverage remains incomplete

- **Priority:** P1
- **Area:** attempt/role classification
- **Status:** open

### Current repo reality

Top-level command-role classification handles `npm`/`pnpm` verification forms better than the
attempt-role path, but there are still easy holes:

1. `yarn` is not treated as a first-class npm-like family,
2. `npm run lint`, `pnpm run lint`, and `yarn lint` are not all surfaced cleanly through the
   attempt-role classifier,
3. this can reduce verifier-attempt quality on JS/TS-heavy repositories.

### Code anchors

- `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`
  - npm-like attempt role handling: around `665-694`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
  - npm/pnpm command-role handling: around `1082-1115`

### Intended `R5.5` direction

Harden npm-like attempt-role coverage so common JS/TS verifier commands classify deterministically,
including:

1. `npm run lint`
2. `pnpm run lint`
3. `yarn lint`
4. `npm test`
5. `pnpm test`
6. `vitest ...`

### Acceptance seeds for later `R5.5` planning

Add:

1. command-role tests,
2. attempt-role tests,
3. at least one checkpoint-level progress test where JS/TS verifier attempts become comparable
   progress evidence rather than generic shell noise.

---

## R5.5-FU-005: Parent-visible orchestration bypasses final progress normalization

- **Priority:** P2
- **Area:** parent-visible orchestration fallback consistency
- **Status:** open

### Current repo reality

`build_session_progress(...)` still returns early when
`parent_visible_orchestration_progress(...)` matches, which means this branch bypasses
`finalize_progress(...)`.

### Code anchors

- `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
  - early return path: around `26-33`
  - finalizer: around `1247-1260`

### Why it matters

This is not known to break the current contract, but it means parent-visible progress does not
automatically share:

1. signal sorting,
2. evidence dedupe / limiting,
3. final normalization behavior.

### Intended `R5.5` direction

Route parent-visible progress through `finalize_progress(...)` for consistency unless a future
design explicitly justifies a separate path.

### Acceptance seeds for later `R5.5` planning

Add or update delegated-parent tests to prove:

1. the status stays conservative,
2. evidence remains present,
3. the final normalized shape matches non-parent-visible progress hygiene rules.

---

## R5.5-FU-006: Delegation caps do not always promote limiting evidence into counter-evidence

- **Priority:** P2
- **Area:** delegation guardrails / operator honesty
- **Status:** open

### Current repo reality

`apply_delegation_caps(...)` currently caps confidence and adds
`DelegationVisibilityLimited`, but archetype-native progress does not always surface that same
limiting context as explicit `counter_evidence`.

### Code anchors

- `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
  - `apply_delegation_caps(...)`: around `1202-1245`

### Why it matters

The current behavior is directionally correct but can still under-communicate why a stronger claim
was denied or confidence-capped, especially in delegated partial/opaque cases.

### Intended `R5.5` direction

Whenever delegation caps suppress a stronger claim or reduce confidence, the implementation should
consider promoting the limiting evidence into `counter_evidence` as well as a limiting signal.

### Acceptance seeds for later `R5.5` planning

Add delegated-partial and delegated-opaque cases that assert:

1. confidence is capped conservatively,
2. limiting signal is present,
3. limiting evidence is visible in `counter_evidence`.

---

## R5.5-FU-007: Progress-window representation remains implicit

- **Priority:** P3
- **Area:** internal design / debuggability / performance
- **Status:** open

### Current repo reality

The current code models progress windows implicitly by rescanning checkpoint analyses and applying
window-boundary predicates, rather than carrying a named internal progress-window structure.

### Code anchors

- `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
  - repeated `checkpoint_analyses(&analysis.current.window)` rescans around:
    - `1339`
    - `1367`
    - `1922`

### Why it is not an immediate blocker

This is not yet a correctness bug. The current behavior is understandable enough for `R5`, and the
main correctness work sits elsewhere.

### Intended `R5.5` or later direction

Consider promoting the implicit window into a named internal seam such as:

```text
ProgressWindow {
  start_checkpoint,
  end_checkpoint,
  reset_reason,
  comparable_attempts,
  best_frontier_so_far,
}
```

This would primarily help:

1. debug explainability,
2. reviewability of comparability decisions,
3. repeated-scan locality/perf hygiene.

---

## R5.5-FU-008: `diagnostics.rs` still uses `#![allow(dead_code)]`

- **Priority:** P3
- **Area:** diagnostics hygiene
- **Status:** open

### Current repo reality

`crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs` still begins with
`#![allow(dead_code)]`.

### Why it is not an immediate blocker

This is acceptable for a staged landing, but it weakens trust in the diagnostic seam if it remains
too long after the feature has landed.

### Intended `R5.5` or later direction

Either:

1. remove the attribute after wiring or deleting dead code, or
2. explicitly document why the remaining currently-unused pieces are intentionally staged for later
   work.

---

## R5.5-FU-009: R5 task doc still carries intentionally stale legacy checklist text

- **Priority:** P3
- **Area:** documentation hygiene
- **Status:** open

### Current repo reality

`docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md` explicitly says that the
remaining unchecked items are legacy checklist text rather than open R5 work.

This is honest, but it is still easy for later readers to misread.

### Intended direction

On the next doc-hygiene pass, either:

1. collapse the legacy unchecked global list into a resolved historical note, or
2. rewrite it so no unchecked boxes remain once the family is explicitly declared landed.

## Suggested Future Packeting Shape

This is **not** the plan, only a likely decomposition seed for the later `R5.5` planning session.

### Candidate Packet A: Troubleshooting overclaim fix

- scope: `R5.5-FU-001`
- goal: remove false `advancing` on repeated troubleshooting failures after overlapping edits

### Candidate Packet B: Objective extraction and real-rollout corpus deepening

- scope:
  - `R5.5-FU-002`
  - `R5.5-FU-003`
- goal: improve real-session task framing and back it with real acceptance fixtures

### Candidate Packet C: JS/TS verifier-role hardening

- scope: `R5.5-FU-004`
- goal: improve npm/pnpm/yarn/vitest classifier fidelity

### Candidate Packet D: Delegation normalization and evidence honesty

- scope:
  - `R5.5-FU-005`
  - `R5.5-FU-006`

### Candidate Packet E: Cleanup / design deepening

- scope:
  - `R5.5-FU-007`
  - `R5.5-FU-008`
  - `R5.5-FU-009`

## Non-Goals For The Later `R5.5` Planning Session

Unless fresh repo evidence changes the picture, a later `R5.5` plan should **not** silently widen
into:

1. `R6` scorer retuning,
2. sentinel policy changes,
3. broad compactor rewrites,
4. full parent/child semantic linkage beyond the existing `R5` delegation guardrails.

## Recommended Planning Entry Questions

When a later session turns this doc into `R5.5` spec / plan / tasks, it should answer:

1. Which of these items are true correctness blockers versus acceptance-depth gaps?
2. Does `R5.5` stay analyzer-only, or do any doc / sentinel fixture updates need their own packet?
3. Which real-rollout cases should become committed progress fixtures?
4. Can the objective-pollution fix be kept narrow to task-frame inference without destabilizing
   existing accepted fixtures?
5. Should delegation consistency work land together or remain separate from the P0/P1 fixes?
