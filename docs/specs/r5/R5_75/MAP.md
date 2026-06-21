# R5.75 Map: Sequential Pre-R6 Hardening And Validation

Status: draft map created on 2026-06-12 to turn the adopted post-`R5.5` fix list into a one-issue-at-a-time landing order with explicit promotion gates and manual smoke checks between landings; reconciled on 2026-06-17 against the live `R5.75-1` structured-objective phase-1 stack and updated on 2026-06-18 after `SO-2.3B-refine` closeout. On 2026-06-20 the `R5.75-1` named smoke gate was re-run and promotion was first HELD (a structured-objective failure on `019eb47f` pulled Issue 1/2/3 forward as a blocker); the Issue 1/2/3 anchoring fix (plus Issue 8 corpus lock) then landed and the gate was re-run green, so `R5.75-1` is now PROMOTED and the active seam is `R5.75-2` (sparse readable session fail-open). The map reflects the current active seam and next-packet order honestly.

## Objective

Finish the remaining analyzer-semantic hardening required before `R6` scorer work by landing one bounded issue at a time, validating between each landing, and refusing to advance to the next issue until the current issue is both test-green and smoke-proven against the known native and adapted repro sessions.

## Assumptions

1. The user’s requested new spec directory belongs under the existing `R5` authority stack, so this map lives at `docs/specs/r5/R5_75/` rather than a new top-level `docs/specs/R5_75/` tree.
2. Native `.codex/sessions/rollout-*.jsonl` sessions remain the primary behavior authority.
3. The adapted Hugging Face export corpus remains secondary robustness evidence only; it is useful for hardening but does not redefine native Codex rollout semantics.
4. `R5.5` landed meaningful improvements, but the validation handoff proved the family is not yet ready to declare “fully landed and R6-ready.”

## Current Live Routing Note (2026-06-20)

- `R5.75-0` is landed history.
- `R5.75-1` is landed history as of 2026-06-20 (routed through
  `docs/specs/r5/R5_75/phase-1/SO/`). The current active seam is `R5.75-2` (sparse readable session
  fail-open, `crates/agent-drift-analyzer/src/input.rs`).
- **Promotion PROMOTED (2026-06-20):** the named promotion smoke first surfaced a structured-objective
  failure on a condensation gate session (`019eb47f`): the structured `Goal` spans anchored to
  boilerplate (system-instruction rows + the pasted `$code-review-and-quality` skill body), yielding
  `primary_intent=implement` on a review/eval ask and `success_conditions` pooled from the `AGENTS.md`
  cargo ladder. This held the gate and pulled Issues 1/2/3 forward to blocker. The anchoring fix then
  landed in `context/objective.rs` (scope goal selection + evidence spans + field assembly to the
  selected goal's mission surface; exclude pasted-boilerplate rows from goal candidacy; source-gate
  the `Goal` role to user/goal surfaces; synthesize a structural goal for the top user prompt when its
  phrasing misses the keyword heuristics), with Issue 8 corpus coverage. The gate was re-run green on
  every named session, so `R5.75-1` closed and `R5.75-2` begins. See the `R5.75-1` Remaining Bug
  Ledger and promotion gate below.
- The live crate already has real section/clause decomposition, preliminary structured assembly,
  and grounding follow-on work through `SO-G6`, so this is no longer just a narrow
  `normalized_objective_text(...)` stopgap.
- `SO-2.3B-refine` is landed inside `R5.75-1`: checkpoint narrowing preserves the richer
  structured objective, weak goal clauses no longer fabricate `target`, verification-command
  extraction is role-backed, the packet verification wall is green, and the packet closeout notes
  record those facts plus residual risks explicitly.
- `SO-3`, `SO-4`, `SO-5`, and `SO-6` are now landed as well (verified against the live crate):
  `ObjectiveSummary.text` renders from structured state when safe
  (`compatibility_text_from_structured`), `comparison_key` is derived from structured semantic
  state (`comparison_key_from_structured`, no longer a display-text echo), and the committed
  `objective_acceptance` harness + locked-acceptance corpus exist and are green. The earlier
  bullet wording that called those items "still absent / still mirrors display text" was stale and
  has been corrected here.
- Two further packets surfaced by structured-objective validation are now **also landed**:
  1. **`SO-Observability`** — the promotion-gate smoke previously inspected only the
     `task_frame.objective` string and was blind to `primary_intent`, `target`, `success_conditions`,
     `deliverables`, and `unknowns`. This packet exports an additive `structured_objective` on each
     `Checkpoint`, renders a per-checkpoint objective line in `summary.md`, and rewrote the `R5.75-1`
     gate to assert structured semantics (see `R5.75-1` gate below).
  2. **`SO-2.3D`** — fixed the two semantic-honesty defects the now-observable output exposed: review
     prompts no longer misclassify as `Implement` on the noun "implementation" (`#4`, intent is the
     request action via whole-word matching), and `success_conditions`/`deliverables` are scoped to the
     active goal surface with symmetric `unknowns` instead of being pooled from boilerplate (`#6`). The
     two regressions are live (no longer `#[ignore]`d) and two locked acceptance cases were added.
- The named promotion smoke (condensation repros `019eb430/47f/98e` + adapted `05a56cc5…` + the
  `019edd98` structured-stressing session) was re-run on 2026-06-20. The first pass (pre-fix) held the
  gate: `019eb47f` anchored its goal to boilerplate and pooled `intent`/`target`/`success_conditions`
  from non-goal rows (Issue 1/2/3). The **Issue 1/2/3 anchoring fix then landed** in
  `context/objective.rs` and the gate was re-run **green** on every named session — `019eb47f` now
  anchors to the evaluate/review ask (`review` intent, grounded `docs/specs/r5` target, weak fields
  unknown, zero boilerplate `Goal` spans), and the adapted `05a56cc5…` anchors to the concrete
  `add this skill to @shared-cab-app` steer (grounded `repo_slice` target) instead of the pasted skill
  template. Issue 8 locked the fix into the corpus. Issue 5 (prose/broad target precedence) remains a
  non-blocking follow-on; Issue 7 (downstream migration) is now unblocked but deferred.

## Required Fixes Adopted Into R5.75

This map includes only the fixes that were explicitly adopted as required for pre-`R6` closure:

1. `R5.75-0` — reconcile stale `R5.5` status/docs so the repo authority matches landed code plus remaining work
2. `R5.75-1` — objective condensation / target extraction for giant pasted user prompts
3. `R5.75-2` — sparse readable session fail-open instead of analyzer hard-abort
4. `R5.75-3` — delegated parent-visible stabilization
5. `R5.75-4` — zero-verifier anti-flap gating for long exploratory sessions
6. `R5.75-5` — adapted external robustness fixture family

Not adopted into this required set by default:

- parent-visible comparability fingerprint redesign beyond what live repros prove necessary
- adapted-trace mirrored-row compactor widening unless analyzer hardening still cannot hold without it

## Authority And Evidence Inputs

Primary native evidence bundle:

- `target/manual-r55-validation/`
- key session ids:
  - `019eb430-6f9a-7a03-9a63-cb451b654795`
  - `019eb47f-0118-7e90-8291-30a1fb93769e`
  - `019eb98e-3c16-7ba0-92f9-0085654b470c`
  - `019eb907-95c4-73e1-843e-e337d1e93cb9`
  - `019eb917-9531-74e0-897d-ad8d362138ec`
  - `019eb970-3543-7ab1-a5d6-2a62c00c7185`

Secondary adapted-external evidence bundle:

- dataset root: `raw/RangaPrasath/`
- adapted home: `target/ranga-validation/codex-home/`
- run outputs: `target/ranga-validation/runs/`
- sample ids:
  - `05a56cc51632982b`
  - `f47b81f39f2495dd`
  - `9c1512861c25cef1`
  - `097d97e914ca220f`
  - `da59436e63915185`

## Shared Packet-Prompt Precondition Rule

Any packet prompt in this `R5.75` stack that names earlier packets/tasks as already landed must
make that prerequisite operational rather than prose-only:

- verify the named prerequisite packets/tasks against live repo state before editing,
- stop and report the missing prerequisite if the earlier landing is absent or incomplete, and
- do not let a later packet silently absorb or repair missing earlier-packet work.

## Shared Verification Ladder

Every implementation issue in this map must use the same broad verification ladder before promotion:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

If an issue directly changes replay/operator compatibility or committed checkpoint fixtures in ways that could affect downstream consumption, add the minimal sentinel spot-checks before promotion:

```bash
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

## Shared Manual Smoke Harness

Use the repo’s existing static Hybrid Drift smoke path for each named session.

Native-session smoke template:

```bash
export CODEX_HOME="$HOME/.codex"
export SESSION_ID="<native-session-id>"
export SMOKE_ROOT="target/r5_75-smoke/<issue-id>/$SESSION_ID"
export COMPACTOR_OUT="$SMOKE_ROOT/compactor"
export ANALYZER_OUT="$SMOKE_ROOT/analyzer"

rm -rf "$SMOKE_ROOT"

cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT"

sed -n '1,80p' "$ANALYZER_OUT/summary.md"
sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"
```

Adapted-external smoke template:

```bash
export CODEX_HOME="$(pwd)/target/ranga-validation/codex-home"
export SESSION_ID="<adapted-session-id>"
export SMOKE_ROOT="target/r5_75-smoke/<issue-id>/$SESSION_ID"
export COMPACTOR_OUT="$SMOKE_ROOT/compactor"
export ANALYZER_OUT="$SMOKE_ROOT/analyzer"

rm -rf "$SMOKE_ROOT"

cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT"

sed -n '1,80p' "$ANALYZER_OUT/summary.md"
sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"
```

Manual smoke review rule:

- Do not treat green commands alone as promotion proof.
- Inspect `summary.md` plus the first checkpoints and confirm the issue-specific expectation below.
- `R5.75-0` is the docs-only exception: it may intentionally skip native control smoke when its
  packet closeout notes record the cross-doc audit plus baseline `agent-drift-analyzer` test run.
- Do not advance if the current issue appears fixed but any earlier landed issue regresses on its named smoke sessions.

## Sequential Landing Map

## R5.75-0: R5.5 Status Reconciliation And Authority Cleanup

### Problem

The current `R5.5` docs still read like most implementation packets are open follow-up work even though several packets landed and the remaining work has shifted to a narrower post-validation hardening family. That stale authority risks reopening already-landed work and makes later packet closure dishonest.

### Required Change

- add an explicit landed-status section to the `R5.5` authority docs
- mark already-landed `R5.5` work as landed/history rather than still-open implementation debt
- move the remaining pre-`R6` work into this new `R5.75` map/family
- keep root landing-order authority honest about `R5.75` being the next gate before `R6`

### Primary Files

- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- any matching root-order authority that names the next family

### Automated Gate

Manual review only for the doc changes themselves, plus:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

### Manual Smoke Check Before Promoting To R5.75-1

- Manual authority review across the edited docs must show:
  - landed `R5.5` work is not still presented as unchecked implementation work
  - remaining issues now point at `R5.75`, not back at stale `R5.5` wording
  - `R6` is not named as current/next until `R5.75` is complete
- Optional control smoke is intentionally skippable for this docs-only packet. If skipped, record
  the skip explicitly in the packet closeout notes together with the cross-doc audit and baseline
  `cargo test -p agent-drift-analyzer -- --nocapture` result.

### Promotion Gate

Do not begin `R5.75-1` until the docs are internally consistent and no stale `R5.5` checkbox language remains for already-landed work.

## R5.75-1: Objective Condensation And Target Extraction

### Problem

Objective selection is still too row-level. Giant pasted user prompts can still win as the checkpoint objective even when the concrete ask is embedded later in the same row or prompt body.

### Required Change

- condense long user prompts to the shortest concrete task ask when possible
- prefer `/goal`, short imperative asks, and clear workspace/action-target phrases over whole pasted bodies
- remove the current bias that can let longer same-priority candidates win just because they are longer
- preserve user-requested boilerplate targets when the actual task is to analyze or edit the boilerplate itself
- carry the richer structured-objective sidecar through checkpoint narrowing instead of erasing it
- keep weak target evidence unknown instead of fabricating a conceptual target
- align verification-command extraction with grounded verification-role spans before acceptance is locked

### Primary Files

- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/context/objective.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `docs/specs/r5/R5_75/phase-1/SO/*`

### Automated Gate

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Reconciled Live Status (2026-06-19)

The original condensation work is no longer the whole story for `R5.75-1`. The live crate now
contains additive structured-objective state, section/clause decomposition, unknown preservation,
grounding follow-on work, the `SO-2.3B-refine` honesty bridge, and the landed `SO-3`/`SO-4`/`SO-5`
compatibility + comparison-key + acceptance-wall work. The following items, previously listed as
open, are now **landed** (verified against the live crate):

- checkpoint narrowing preserves the richer `assemble_context(window)` structured objective and uses
  legacy narrowing only as fallback/display layering (`SO-2.3B-refine`),
- `target` assembly leaves weak goal clauses unknown instead of fabricating a target (`SO-2.3B-refine`),
- `comparison_key` is derived from structured semantic state, not a display-text echo (`SO-3.2`),
- compatibility text renders from structured state when safe (`SO-3.1`),
- the committed `objective_acceptance` harness + locked-acceptance corpus exist and are green
  (`SO-4`/`SO-5`/`SO-6`).

What actually remains open inside `R5.75-1` is narrower and was surfaced by validating the structured
work against unseen real sessions:

- **Gate observability (landed, `SO-Observability`):** the promotion-gate smoke surfaces
  (`summary.md`, `checkpoints.jsonl`) previously exported only the `task_frame.objective` string and
  could not see the structured fields, so a green gate proved nothing about the structured semantics.
  This packet exports an additive `structured_objective` per `Checkpoint`, renders a per-checkpoint
  objective line in `summary.md`, and rewrote the gate to assert structured semantics.
- **Two semantic-honesty defects (fixed, `SO-2.3D`):** review prompts no longer misclassify as
  `Implement` on the noun "implementation" (`#4`, intent is the request action via whole-word
  matching), and `success_conditions`/`deliverables` are scoped to the active goal surface with
  symmetric `unknowns` instead of being pooled from boilerplate (`#6`). The two regressions are live
  and two locked acceptance cases were added.
- **What remains for closeout:** re-run the named promotion smoke (condensation repros + `019edd98`)
  and confirm the structured-aware expected outcomes, then `R5.75-1` closes and `R5.75-2` may begin.

### Remaining `R5.75-1` Landing Order Before `R5.75-2`

1. **`SO-2.3B-refine` / `SO-Bridge-1`** `[landed]`
   - preserved the structured `assemble_context(window)` objective through checkpoint narrowing,
   - kept legacy narrowed text fallback-only and display-only when needed,
   - stopped weak goal clauses from fabricating `target`, and
   - aligned verification-role grounding with verifier-command extraction.
2. **`SO-3.1` / `SO-3.2`** `[landed]`
   - render compatibility text from structured state when safe,
   - derive deterministic `comparison_key` from structured semantic state.
3. **`SO-4.1` / `SO-4.2`** `[landed]`
   - added `tests/objective_acceptance.rs`,
   - locked the expected-shape contract for structured fields, grounding, forbidden promotions,
     compatibility rendering, and unknown-field correctness.
4. **`SO-5.*`** `[landed]`
   - seeded the locked acceptance corpus: WDAP, preserved boilerplate-target cases, concise `/goal`,
     review/no-code, and planning/docs families.
5. **`SO-Observability` (structured-output + structured-aware gate)** `[landed]`
   - exported an additive optional `structured_objective` on each `Checkpoint`
     (`checkpoint/schema.rs`, populated in `checkpoint/mod.rs`) and rendered a per-checkpoint objective
     line in `summary.md` (`checkpoint/export.rs`), without bumping the `v0.6` schema or migrating
     `TaskFrame`/`infer_task_frame`,
   - rewrote the `R5.75-1` gate (below) to assert structured semantics and added a structured-stressing
     smoke session,
   - captured `#4`/`#6` as committed pending specs in `tests/checkpoints.rs`.
6. **`SO-2.3D` (semantic-honesty fix)** `[landed]`
   - fixed `#4` — `intent_for_text` derives intent from the goal clause's request action via whole-word
     matching (the noun "implementation" no longer trips `Implement`); this also corrected the WDAP
     fixtures `plan`→`validate` to match the evaluation authority,
   - fixed `#6` — `success_conditions`/`deliverables`/`constraints` assembly is scoped to the active
     goal surface (`clause_is_on_active_goal_surface`), with symmetric `unknowns` recorded when an
     off-surface cue is rejected,
   - made the two regressions live (no longer `#[ignore]`d) and added the
     `review-implementation-noun-review-intent` + `orchestration-scaffolding-field-honesty` locked
     acceptance cases. `R5.75-2` is now unblocked pending the named promotion smoke.

Ordering note: even though earlier local packet docs briefly put `SO-4` ahead of `SO-3`, the
current architecture and migration authorities make `SO-3` the better prerequisite. Compatibility
rendering and `comparison_key` derivation are Phase-1 semantics that the acceptance harness should
validate, not a stopgap the harness silently defines after the fact.

### Manual Smoke Check Before Promoting To R5.75-2

This gate is **structured-aware**: with `structured_objective` now exported per checkpoint and
rendered in `summary.md` (the `objective:` line per checkpoint), the smoke must inspect the
structured fields, not just the legacy `task_frame.objective` string. Re-run the analyzer on each
session after building this packet so the new fields appear.

Run native condensation smoke on:

- `019eb430-6f9a-7a03-9a63-cb451b654795`
- `019eb47f-0118-7e90-8291-30a1fb93769e`
- `019eb98e-3c16-7ba0-92f9-0085654b470c`

Run adapted condensation smoke on:

- `05a56cc51632982b`

Run structured-stressing smoke on (review/orchestration shape that exercises intent + field honesty):

- `019edd98-8a79-7b53-a90b-94cd0d32329b`

Expected smoke outcome:

- **Condensation (string surface):** first-checkpoint objective resolves to the concrete task, not
  the pasted skill/spec/profile body; the adapted `05a56cc51632982b` session condenses to the
  workspace action request; explicit requests to analyze/edit instruction/skill/AGENTS material stay
  preserved when that is the real target.
- **Structured semantics (now observable):** for the review-shaped session `019edd98`,
  `primary_intent` resolves to `review` (not `implement`); `success_conditions` and `deliverables`
  contain only goal-grounded items, not skill/memory/safety/output-format boilerplate; and `target`
  is either a grounded anchor or `unknown` with an `ObjectiveUnknown`, never a fabricated conceptual
  topic.

### Remaining Bug Ledger (Structured Objective)

This is the **single canonical status set** of structured-objective bugs for `R5.75-1`: their
classification (blocker / follow-on / landed) and the promotion gate. The full root-cause detail
(per-issue code paths, evidence, research order) lives in
`docs/specs/r5/R5_75/structured-objective-bug-map.md`; this table is the authority for *status*,
that file is the reference for *diagnosis*. Numbering is unified
on `Issue 1`–`Issue 8`; the older `#3`–`#7` labels used elsewhere in this map are legacy aliases
mapped in the table. Statuses are current as of the 2026-06-20 gate re-run. **Promotion is gated on
every `blocker` row clearing.**

| Issue | Bug (one line) | Legacy alias | Primary code path | Status |
|-------|----------------|--------------|-------------------|--------|
| 1 | Extraction runs over the full session compact-row pool instead of an active-objective surface | — | `context/mod.rs::assemble_context` → `extract_objective` | **Landed (R5.75-1 anchoring fix, 2026-06-20)** |
| 2 | Structured fields assembled by pooling clauses from all candidate rows | — | `context/objective.rs::assemble_structured_objective`, `*_from_decomposition` | **Landed (R5.75-1 anchoring fix, 2026-06-20)** |
| 3 | Goal-role detection too permissive: boilerplate/system rows receive `Goal` evidence spans | `#3` | `context/objective.rs::role_candidates_for_clause`, `looks_like_goal_text` | **Landed (R5.75-1 anchoring fix, 2026-06-20)** |
| 4 | Intent substring-driven (review→implement on the noun "implementation") | `#4` | `context/objective.rs::intent_for_text` | **Landed (SO-2.3D)** — now unconditional (1/2/3 landed) |
| 5 | Target prefers broad repo/dir paths over packet/doc anchors | `#5` | `context/objective.rs::explicit_target_*` | OPEN — non-blocking follow-on |
| 6 | `success_conditions`/`deliverables` over-upgrade weak/boilerplate evidence | `#6` | `context/objective.rs::success_conditions_from_decomposition`, `deliverables_from_decomposition`, `unknowns_for_objective` | **Landed (SO-2.3D)** — now unconditional (1/2/3 landed) |
| 7 | Legacy narrowing/compat overlay can overwrite good structure with the wrong imperative line; downstream consumers not yet structured-native | `#7` | `checkpoint/mod.rs::narrowed_objective_summary`, `normalized_objective_text`; deferred `SO-X.1`–`X.4` | Deferred follow-on — **now ungated** (1/2/3 landed); not landed here |
| 8 | Acceptance corpus lacks real orchestration-shaped session shapes | — | `tests/objective_acceptance.rs`, `tests/fixtures/objective_acceptance/**` | **Landed (R5.75-1 anchoring fix, 2026-06-20)** |

**Blocker set for `R5.75-1` closure (cleared 2026-06-20):** Issues 1, 2, 3, and 8 are landed. The
anchoring fix scopes goal selection + evidence spans + field assembly to the selected goal's mission
surface, excludes pasted-boilerplate rows (negative `objective_score`) from goal candidacy,
source-gates the `Goal` role to user/goal surfaces, and synthesizes a structural goal for the top
user prompt when its phrasing misses the keyword heuristics. Issue 7 is now *ungated* (its hard
prerequisite — 1/2/3 — is landed) but remains a deferred follow-on, not landed in this packet. Issue 5
is the only purely non-blocking follow-on still open.

Notes:

- **Issues 1/2/3 landed (R5.75-1 anchoring fix).** `context/objective.rs`:
  `selected_goal_clause` now excludes boilerplate candidates (`candidate_is_boilerplate_surface`,
  keyed to the row scorer's negative `objective_score`) and picks the best clause-level goal among
  the survivors; `role_candidates_for_clause` source-gates `Goal` to `ThreadGoal`/`UserPrompt` rows;
  `inject_structural_goal_if_absent` promotes the top genuine user surface's primary actionable
  clause (gated by `clause_states_a_request_action`) when no keyword goal exists; and
  `evidence_spans_from_decomposition` grounds spans to the selected goal's candidate. Verified on the
  gate sessions (below).
- **Issue 4 / Issue 6 now hold unconditionally.** `SO-2.3D` fixed the narrow defects (whole-word
  intent derivation; active-goal-surface field scoping with symmetric `unknowns`); they were
  conditional on a correct goal anchor, which 1/2/3 now guarantee. On `019eb47f` the goal anchors to
  the real evaluate/review ask, so `primary_intent` resolves to `review` and `success_conditions` /
  `deliverables` stay empty with `ObjectiveUnknown`s rather than re-filling from the `AGENTS.md` cargo
  ladder. Live regressions in `tests/checkpoints.rs`:
  `so_2_3d_review_prompt_with_implementation_noun_stays_review_intent`,
  `so_2_3d_boilerplate_scaffolding_does_not_populate_success_or_deliverables`,
  `checkpoints_anchor_structured_objective_to_evaluate_ask_over_boilerplate_pool`; locked acceptance
  cases `review-implementation-noun-review-intent`, `orchestration-scaffolding-field-honesty`,
  `orchestration-evaluate-ask-anchor`.
- **Core diagnosis (confirmed).** The extractor used to treat a broad *session text soup* as the
  surface; the fix makes it *grounded extraction from the active mission surface* with conservative
  unknowns. Issue 1 (scope) + Issue 2 (pooling) were the spine; 3/4/6 fell out once anchoring was
  correct, exactly as predicted.
- **Issue 8 corpus.** Locked acceptance case `orchestration-evaluate-ask-anchor` (the minimized
  `019eb47f` shape) plus the `checkpoints.rs` full-pipeline regression
  `checkpoints_anchor_structured_objective_to_evaluate_ask_over_boilerplate_pool` lock the fix.
- **Residual (non-blocking, Issue 5).** On `019eb47f` checkpoints 2/3 (the later "perform a series of
  manual smoke checks … evaluate …" ask) the structured `target` resolves to `signal/classification/etc`
  — a prose token misread as a path. It comes from the genuine user ask (not boilerplate, not a
  pasted-body path, not a fabricated conceptual topic), so it satisfies the gate; tightening
  prose-path target extraction stays tracked as Issue 5.
- **Evidence sessions.** `019eb47f` — now anchors the goal to the evaluate ask (`review` intent,
  grounded `docs/specs/r5` target, weak fields unknown, zero boilerplate `Goal` spans); `019eddaa` —
  wrong imperative "optional nit" line wins (Issue 7); `019edd98` — passes, the shape where 4/6 held
  even before the anchoring fix.

### Promotion Gate

Do not begin `R5.75-2` until all of the following are true:

- the named condensation smoke repros still resolve to the true concrete task instead of pasted
  scaffold bodies,
- preserved-boilerplate targets still hold in targeted regressions,
- checkpoint narrowing no longer erases the structured sidecar (landed, `SO-2.3B-refine`),
- weak target evidence stays unknown instead of being fabricated into `target` (landed, `SO-2.3B-refine`),
- compatibility text and `comparison_key` come from structured state (landed, `SO-3`),
- `objective_acceptance` is a committed, green wall (landed, `SO-4`/`SO-5`),
- the exported checkpoint surfaces `structured_objective` and the gate smoke asserts structured
  semantics (landed, `SO-Observability`),
- the `SO-2.3D` semantic-honesty fix is landed so Issue 4/Issue 6 are fixed and their live regressions
  plus the structured-stressing smoke are green (now unconditional — Issue 1/2/3 landed; see ledger),
- **[CLEARED 2026-06-20]** the structured objective is correct on every named condensation gate
  session, not only `019edd98`: the goal anchors to the real ask with no boilerplate-pooled
  `intent`/`target`/`success_conditions` (Issues 1/2/3; see the Remaining Bug Ledger above). This is
  now satisfied on `019eb47f`; see the promotion decision below.

The operational gate was re-run on 2026-06-20 — first against the pre-fix pipeline (which held
`R5.75-1`), then against the fixed pipeline after the Issue 1/2/3 anchoring fix landed
(`cargo test -p agent-drift-analyzer` and `-p agent-drift-sentinel` both green — 486 tests, 0
failures; all five smoke pipelines compactor→analyzer→sentinel clean). Recorded outcomes after the fix:

- **Condensation (string surface) — pass:** all four first-checkpoint objectives resolve to the
  concrete task ask, not a pasted scaffold body. `019eb430` → the `/goal Review the already-landed
  Packet R5-7 …` ask; `019eb47f` → `use the $code-review-and-quality skill to evaluate if what was
  implemented landed correctly and completely`; `019eb98e` → `review what landed … and
  validate/invalidate …`; adapted `05a56cc5…` → `add this skill to @shared-cab-app` (grounded
  `repo_slice` target).
- **Structured semantics (`019edd98`) — pass (unchanged):** `primary_intent=review` on every
  checkpoint, goal-scoped `success_conditions`/`deliverables` (empty, with symmetric
  `ObjectiveUnknown`s), and a grounded `file_or_directory` target, never a fabricated conceptual topic.
- **Structured semantics (`019eb47f`) — now PASS:** the goal anchors to the real evaluate/review ask
  on every checkpoint. Checkpoints 0/1 (the named defect): `primary_intent=review` (was `implement`),
  `target=docs/specs/r5` grounded spec doc (was `/run/substrate.sock`), `success_conditions` /
  `deliverables` empty with symmetric `ObjectiveUnknown`s (was 24 cargo-ladder conditions), a single
  goal evidence span on the user prompt, and **zero `Goal` spans on system-instruction or pasted-skill
  rows** (was 22 system-instruction goal spans out of 541 pooled spans). Checkpoints 2/3 (the later
  "perform … smoke checks … evaluate …" ask) likewise resolve `review` with no boilerplate-pooled
  fields; the only residual is the `signal/classification/etc` prose-path target (Issue 5,
  non-blocking — grounded in the genuine ask, not a pasted body).
- **Adapted `05a56cc5…` structured — pass:** the goal anchors to the concrete steer
  `add this skill to @shared-cab-app` with a grounded `repo_slice` target, no longer the earlier
  pasted frontend-pro skill template.

**Promotion decision: PROMOTED (2026-06-20). `R5.75-1` is closed; `R5.75-2` may begin.** The Issue
1/2/3 anchoring fix landed in `crates/agent-drift-analyzer/src/context/objective.rs` (additive,
analyzer-local; `checkpoint/mod.rs` narrowing and the deferred `SO-X.*` consumers untouched). The
structured-aware gate now proves structured semantics on every named gate session, the automated
analyzer + sentinel walls are green, and the fix is locked into the corpus (Issue 8). The deferred
Issue 7 migration is now **unblocked** (its hard prerequisite — correct anchoring — is landed) but is
intentionally not part of this packet.

Phase 1 remains additive: the sidecar is still not read by the downstream drift/progress path, which
runs off the legacy `TaskFrame`/working-set bridge. The anchoring fix therefore corrects the
structured sidecar (the gate surface) without changing live drift output; it is the hard prerequisite
that makes the deferred Issue 7 migration safe to wire in next.

## R5.75-2: Sparse Readable Session Fail-Open

### Problem

The analyzer still hard-fails readable sessions when tool-call payloads or working-set inference are weak, even when literal objective rows remain available and a conservative checkpoint would be more honest than aborting.

### Required Change

- split structurally invalid bundle failures from semantically sparse-but-readable bundles
- keep hard-fail behavior for corrupt inputs only
- emit at least one conservative low-confidence checkpoint for sparse readable sessions
- use existing insufficient-evidence surfaces before widening public schema/contract

### Primary Files

- `crates/agent-drift-analyzer/src/input.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- any analyzer acceptance test that proves sparse readable fail-open behavior

### Automated Gate

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Manual Smoke Check Before Promoting To R5.75-3

Run adapted smoke on:

- `f47b81f39f2495dd`

Run native control smoke on:

- `019eb430-6f9a-7a03-9a63-cb451b654795`

Expected smoke outcome:

- `f47b81f39f2495dd` no longer aborts the analyzer pipeline
- analyzer emits at least one checkpoint for the sparse readable session
- resulting output stays conservative: low-confidence / insufficient-evidence rather than fabricated strong progress
- the native control session still produces normal output after the contract change

### Promotion Gate

Do not begin `R5.75-3` until the sparse adapted repro fail-opens cleanly and the native control session proves the relaxed contract did not break ordinary analyzer runs.

## R5.75-3: Delegated Parent-Visible Stabilization

### Problem

Delegated parent-visible progress remains unstable. Strong parent orchestration evidence can still collapse back into generic planning noise, especially when planning artifacts are edited or child visibility is partial/opaque.

### Required Change

- stop discarding parent-visible orchestration solely because planning/spec/handoff artifacts were edited
- preserve conservative parent-visible orchestration when delegation markers are strong but child visibility is limited
- only widen comparability/fingerprint logic if the named repros prove that reset behavior is still blocking stability after the earlier fix

### Primary Files

- `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- any targeted acceptance fixture proving delegated-parent stability

### Automated Gate

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Manual Smoke Check Before Promoting To R5.75-4

Run native smoke on:

- `019eb907-95c4-73e1-843e-e337d1e93cb9`
- `019eb917-9531-74e0-897d-ad8d362138ec`
- `019eb970-3543-7ab1-a5d6-2a62c00c7185`

Run adapted smoke on:

- `da59436e63915185`

Expected smoke outcome:

- delegated parent sessions remain in a stable parent-visible orchestration lane instead of dropping into generic planning-only noise
- limited child visibility remains visible as limiting evidence rather than being hidden
- `019eb970-3543-7ab1-a5d6-2a62c00c7185` stays a positive proof that the parent-visible path still works
- if stability still fails only because comparability resets remain too broad, capture that as the bounded follow-up inside this issue before promoting

### Promotion Gate

Do not begin `R5.75-4` until the three native delegated repros and the one adapted delegated repro hold a conservative but stable parent-visible interpretation.

## R5.75-4: Zero-Verifier Anti-Flap Gate

### Problem

Long browse/read/tool-output-heavy sessions with zero verifier density can still escalate into troubleshooting or dead-end-ish output without decisive evidence.

### Required Change

- cap or suppress troubleshooting/implementation escalation when verifier attempts, concrete source-edit progress, and explicit failure evidence are absent
- prefer planning-convergence / insufficient-evidence for long exploratory sessions unless decisive signals appear
- keep this analyzer-local; do not widen into `R6` scorer retuning yet

### Primary Files

- `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/progress_acceptance.rs`

### Automated Gate

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Manual Smoke Check Before Promoting To R5.75-5

Run adapted smoke on:

- `097d97e914ca220f`
- `da59436e63915185`

Expected smoke outcome:

- zero-verifier exploratory sessions stay boring and conservative
- no troubleshooting-frontier or equivalent strong failure posture appears unless verifier/failure evidence truly exists
- low-confidence planning / insufficient-evidence remains the default outcome when the session mostly reads, browses, or emits tool output without proof work

### Promotion Gate

Do not begin `R5.75-5` until the named exploratory adapted sessions stop flapping into overclaiming progress/failure lanes.

## R5.75-5: Adapted External Robustness Fixture Family

### Problem

The adapted external corpus is currently useful evidence but not yet a committed, bounded, secondary robustness wall. Without a committed robustness family, later regressions can quietly reappear before `R6`.

### Required Change

- add a separate adapted-external acceptance family or fixture lane
- keep it explicitly secondary to the native rollout corpus
- cover at least the adopted external repro classes:
  - giant pasted prompt body
  - sparse readable / no parseable tool-call payloads
  - long exploratory zero-verifier session
  - delegated opaque-parent session

### Primary Files

- `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**`
- any separate adapted-external acceptance test/module chosen during implementation
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- any narrow design note needed to record the secondary-fixture contract

### Automated Gate

```bash
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Add touched sentinel spot-checks only if the committed fixtures alter downstream replay expectations.

### Manual Smoke Check Before Declaring R5.75 Complete

Rerun the full named smoke set:

Native:

- `019eb430-6f9a-7a03-9a63-cb451b654795`
- `019eb47f-0118-7e90-8291-30a1fb93769e`
- `019eb98e-3c16-7ba0-92f9-0085654b470c`
- `019eb907-95c4-73e1-843e-e337d1e93cb9`
- `019eb917-9531-74e0-897d-ad8d362138ec`
- `019eb970-3543-7ab1-a5d6-2a62c00c7185`

Adapted:

- `05a56cc51632982b`
- `f47b81f39f2495dd`
- `097d97e914ca220f`
- `da59436e63915185`

Expected smoke outcome:

- all earlier issue expectations still hold together
- adapted sessions are clearly recorded as secondary robustness proof, not native authority replacement
- no current adopted repro class is left unguarded by either native fixtures, adapted fixtures, or manual smoke evidence

### Promotion Gate

Do not declare `R5.75` complete until the full smoke set above is rerun after the fixture-family landing and all prior issue expectations still hold.

## R6 Readiness Gate

Do not open `R6` until all of the following are true:

- `R5.75-0` authority docs are honest about landed `R5.5` work and remaining `R5.75` scope
- giant prompt objective repros condense to the concrete task instead of pasted scaffold bodies
- the `R5.75-1` structured-objective subfamily is closed honestly: checkpoint narrowing preserves
  the richer assembled structured objective instead of replacing it with compatibility-only state,
  weak targets remain unknown when evidence is weak, compatibility rendering and `comparison_key`
  come from structured semantics, and the objective-acceptance wall is committed
- sparse readable sessions fail open conservatively instead of hard-aborting the analyzer
- delegated parent-visible sessions stay stable and conservative under limited child visibility
- zero-verifier exploratory sessions no longer flap into troubleshooting/dead-end overclaim
- the adapted external robustness family is committed as a secondary acceptance wall
- `cargo test -p agent-drift-analyzer -- --nocapture` is green
- any touched sentinel spot-checks are green
- root landing-order authority names `R6` as next only after `R5.75`

## Out Of Scope For This Map

These can be considered only if the required issues above prove insufficient:

- scorer retuning or `dead_end_thrash` scoring redesign (`R6` scope)
- broad compactor normalization changes for native traces
- opportunistic refactors not required to land the named issue
- generalized parent-visible fingerprint redesign unless the named delegated repros prove it is still necessary after the narrower stabilization work
