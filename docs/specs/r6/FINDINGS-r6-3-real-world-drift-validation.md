# Findings: R6-3 Rolling / Semantic Goal Drift — Real-World Validation And Next-Steps Charter

Status: findings record created 2026-07-03, after `R6-3` (rolling / previous-checkpoint semantic goal drift)
landed and passed a two-round codex review. This document captures a real-world validation investigation of
the `semantic_goal_drift` signal (both the `R6-2` kickoff-anchored and the `R6-3` rolling comparisons) and
is written to be a **self-contained charter for a fresh session** to execute the next steps. It does not
change any code; it records evidence, a locked decision, and a sequenced plan.

Authority / cross-references:
- `docs/specs/r6/MAP.md` — item 6 (R6-3) plus the "Open validation debt (rolling)" and "Diagnostic batch
  scan" notes, which summarize this document.
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md` — the `R6-3.X.3`
  ("loosen the eligibility bar") deferred task now carries the full "Batch scan outcome" and the revised
  verdict; `R6-3.X.2` is the graduated-distance debt.
- Live code: `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` (scorer + `eligible_current_goal`),
  `crates/agent-drift-analyzer/src/context/objective.rs` (objective decomposition / target extraction).
- Codex consults (OpenAI Codex CLI, read-only): `019f2927` (eligibility diagnosis), `019f2964` (batch review).

## TL;DR

On 110 real Codex sessions (43 repos, 11 months, 882 checkpoints), the `semantic_goal_drift` signal fired
6 times, **all in one session, and all false positives caused by garbage target extraction** — observed
live precision `0%`. Rolling fired exactly once, also on that garbage. The strict `unknowns.is_empty()`
eligibility bar is not the primary problem; it is accidentally suppressing over-fire. The two real levers,
in order, are (1) **objective-extraction robustness** in `context/objective.rs` (reject fragment / coordinate
/ model-name / non-path "targets"), then (2) the **graduated-distance metric** (`R6-3.X.2`) to stop reading
narrowing/progression as drift. **Do not loosen the eligibility bar (`R6-3.X.3`) until both land.** Until
then the signal should be treated as **experimental / known-over-fire**.

## Background: The Debt This Investigation Chased

`R6-3` added rolling / previous-checkpoint semantic drift as tagged evidence on the existing
`DriftClass::SemanticGoalDrift` class. It landed test-green, but with an honest, recorded gap: the
`R6-3.1` corpus check found `0 of 23` eligible adjacent `Medium+` `TaskStatement` pairs disjoint, so rolling
never fired on the committed corpus. Its only positive witness was a synthetic acceptance fixture
(`synthetic-rolling-mid-session-pivot`). The open question was whether the signal actually fires — and
whether it over-fires — on real sessions. This investigation answered that with real data.

(Separately, during this pass a real operator-visibility bug was found and fixed: on co-fire the redundant
`rolling ... current goal:` evidence line pushed the informative `rolling ... previous goal:` line past the
default `max_evidence_lines = 3` cap. The scorer now de-dups the shared current-goal line on co-fire. That
fix is unrelated to the eligibility findings below and already landed with tests.)

## Method

Two probes, escalating.

1. **Single-session probes.** Two live rollouts (`019e9864-…` exploratory/review; `019f2837-…`
   concrete-goal) were run through the real pipeline: `agent-session-compactor` (rollout JSONL -> bundle)
   then `agent-drift-analyzer` (bundle -> `checkpoints.jsonl`). Both produced `0` eligible checkpoints and
   `0` drift. A codex consult (`019f2927`) diagnosed the cause as the eligibility bar, upstream of the
   distance metric.

2. **Diagnostic batch scan.** 110 sessions were sampled ~10/month across the 11 analyzable months
   (2025-09 → 2026-07), maximizing distinct repos per month, and run individually through the same pipeline.
   The per-checkpoint results were tabulated for the coverage-diagnostic metrics codex specified. A second
   codex consult (`019f2964`) reviewed the results.

### Reproducing the pipeline (for the fresh session)

```bash
# one session -> checkpoints (isolated codex-home avoids scanning the full ~/.codex/sessions)
mkdir -p $H/codex-home/sessions/YYYY/MM/DD $H/bundle $H/analysis
cp <rollout.jsonl> $H/codex-home/sessions/YYYY/MM/DD/
cargo run -q -p agent-session-compactor -- --codex-home $H/codex-home --output-dir $H/bundle
cargo run -q -p agent-drift-analyzer   -- --input-dir  $H/bundle     --output-dir $H/analysis
# then inspect $H/analysis/checkpoints.jsonl: structured_objective.{objective_class,confidence,target,unknowns}
# and drift_scores[class==semantic_goal_drift].{flagged,evidence[].reason}
```

These commands are now packaged as reusable, parameterized tooling at `scripts/dev/drift-batch-scan/`
(`sample_sessions.py` → `run_batch.py` → `tabulate.py` / `inspect_targets.py` / `filter_junk.py`, with a
`README.md`). The generated batch data and session manifest are intentionally not committed (regenerable;
they embed private `~/.codex/sessions` paths).

Sampling notes: rollout date is in the path (`sessions/YYYY/MM/DD/`); the repo/cwd is in line 1
(`session_meta.payload.cwd`), normalized by collapsing `/worktrees/<hash>/` segments. Run each session in
its own temp codex-home to avoid any cross-session exact-dedupe contamination in the compactor.

## Findings

### F0 — Format cutoff: pre-`session_meta` rollouts are not analyzable

Rollouts before ~2025-09 use an older format whose first line is `{id, timestamp, instructions}` with no
`session_meta` and no `cwd`. They compact into rows but carry no session id the analyzer recognizes, so the
analyzer rejects the bundle ("does not contain any session-scoped rows"). The analyzable window therefore
starts ~2025-09; 2025-07/08 (and earlier) cannot be run through this pipeline as-is. This bounds any
historical validation to ~the last 11 months.

### F1 — The eligibility bar is not inert (coverage is ~18%, not 0%)

Across 882 checkpoints: `TaskStatement + confidence>=Medium` = `714` (81%). Current-bar eligible
(`TaskStatement + confidence>=Medium + unknowns.is_empty()`) = `156` (**17.7%** of all checkpoints, 21.8% of
TS+confident). The earlier two-session `0/0` probe was unrepresentative — real sessions do clear the bar
about one checkpoint in six. Eligibility-failure blockers among TS+confident (a checkpoint can carry several
unknowns): `target` = 366, `deliverables` = 284, `success_conditions` = 202.

### F2 — The only real firings are false positives (0% live precision)

The signal flagged `6` checkpoints total, with `1` rolling evidence line and `6` kickoff-anchor lines. **All
6 flags are one session** (`0199f9ec`, docs repo, 2025-10, ordinals 36-41). The extracted "current goal"
term set is garbage — `implement|file_or_directory|0_0_0_0_4000_n|5_n_n|5s_n|…`, i.e. coordinate / viewport
/ fragment noise (plausibly a diff-hunk header or screenshot dimensions) — while the kickoff anchor
(`architecture_overview_md|authentication_security_md`) is the real doc goal. The single rolling fire is the
same session: previous goal `readme_md` -> current garbage -> disjoint -> fires. So every real-data firing in
the batch is a false positive driven by bad target extraction, not a goal pivot. **Observed live precision:
0/6 = 0%.**

### F3 — Loosening the bar would add mostly-spurious firings

A hypothetical target-resolved bar (`TaskStatement + confidence>=Medium + grounded target present`, ignoring
`success_conditions`/`deliverables`) would admit `348` checkpoints (39.5%), a **+192 (2.2×)** coverage gain
(`target` is the single biggest current blocker at 366; 192 checkpoints have a resolved target but are
blocked only by `success_conditions`/`deliverables`). Rolling's firing surface under that bar: of `299`
adjacent pairs both target-eligible, `286` (95.7%) keep the same target (correct no-drift), `13` change, and
`12` are term-disjoint — i.e. ~12 rolling firing candidates vs the 1 the current bar produced.

But eyeballing all 12 disjoint pairs, they are dominated by garbage/fragment targets and legitimate
narrowing/progression, **not** real pivots. Representative examples:

- `isolated.\n-` → `\n-` (whitespace/fragment garbage)
- `README.md` → `5\n\n` (fragment garbage; same `0199f9ec` session)
- `…SKILL.md` → `GPT-5.4` (model-name fragment)
- `audit-trio.report.json` → `audit-trio.model-selection/cohesion-audit.report.json` (narrowing within the
  same artifact family)
- `PLAN-04.md` → `exec.rs:1537` → `{agent findings blob}` → `PLAN-04.md` (a normal plan→code→plan work
  cycle, flagged as drift at each step)
- `/Users/…/Library/Application` → `ui://orders-fixture/dashboard-v4.html` (truncated path fragment → real)

No clear "agent abandoned goal A for an unrelated goal B" pivot appeared in the 12. `sanctioned_replan` is
not exported per-checkpoint, so 12 is an upper bound on genuine firings.

### F4 — Coverage varies by era and repo

Current-bar eligibility ranges from `0` (2026-04, 2026-07) to `56` (2025-10) across months, and by repo from
`0/18` (buildconnectors) to `41/41` (docs). This reinforces that the dominant factor is
objective-decomposition coverage, which depends on session/workflow shape, not on the scorer.

## Codex Second Opinions

Two read-only Codex consults converged with the analysis.

- `019f2927` (eligibility diagnosis): confirmed rolling never reaches its comparison because
  `eligible_current_goal` gates on `unknowns.is_empty()`; corrected one point — empty
  `success_conditions`/`deliverables` do not disqualify on their own, they become `unknowns` only when the
  extractor saw those cues off the goal surface and rejected them; and neither field participates in
  divergence. Initial lean: loosen the bar to a target-resolved gate.

- `019f2964` (batch review): after the batch data, **reversed the "loosen first" lean** — "do not loosen the
  bar in isolation … correct, not overcorrecting." Highest-leverage fix is extraction hardening in
  `context/objective.rs`, not the scorer (the poison enters via `explicit_target_anchor_for_text`,
  `extract_inline_paths`, `looks_like_explicit_named_target`, `comparison_key_from_structured`). Judged the
  signal **not trustworthy as-is** (0% live precision) — downgrade to experimental / known-over-fire.
  Recommended a shadow-eval probe (extraction hardening only, scorer unchanged) to confirm extraction is the
  dominant fix before touching the distance metric or the bar.

## Decision (locked)

1. **Do not loosen the eligibility bar (`R6-3.X.3`) in isolation.** The strict `unknowns.is_empty()` gate is
   an accidental over-fire suppressor; loosening alone would take rolling from ~1 spurious fire to ~12
   mostly-spurious fires.
2. **Order of work: extraction hardening first, then graduated distance, then (only if warranted) loosen the
   bar.** The garbage-target problem dominates and is upstream of everything else.
3. **Treat `semantic_goal_drift` as experimental / known-over-fire** until extraction and distance are fixed.
   Its observed real-world precision is 0% on this sample; the operator surface should not present it as a
   trustworthy signal yet.

## Gate Result (executed 2026-07-03): Analysis-Only Junk-Target Filter

The charter's Step 2 "cheaper first cut" — an analysis-only junk-target filter over the existing
batch data, before writing any extraction code — has now been run. The 110-session batch and its
pipeline scripts were recovered intact and preserved as reusable tooling at
`scripts/dev/drift-batch-scan/` (`filter_junk.py` is this gate; see that directory's `README.md`).
Re-running the tabulator reproduced the F1–F4 baseline exactly (882 checkpoints, 156 current-bar
eligible, 6 flagged, 348 target-resolved, 12 disjoint pairs, 43 repos).

A term-level junk classifier matching the charter's named extraction defects — escaped-newline
residue (a stray `n` token from literal `\n`), number/coordinate runs, bare model/version tokens
(`GPT-5.4`), and prose-`etc` enumerations — was applied and survivors re-counted:

- **Actual fires: `0 of 6` survive.** All six flagged checkpoints carry the same garbage target (a
  GraphQL server-startup log — `0.0.0.0:4000`, `supergraph:`, `/graphql`, `5s` — parsed as "paths");
  suppressing junk empties their term set, so none reaches a comparison. This includes the single
  rolling fire (`README.md → log-garbage`). Extraction hardening alone removes 100% of the real-data
  over-fire.
- **Current-bar eligible resting on junk-only targets: `8 of 156` (5.1%)** had a target term set that
  is entirely garbage.
- **Target-resolved disjoint adjacent pairs: `12 → 9`** (3 killed outright: `isolated.\n- → \n-`,
  `…SKILL.md → GPT-5.4`, `README.md → log-garbage`).

Hand-verifying the 9 survivors — **none is an "abandoned goal A for unrelated goal B" pivot**:
- `2` are residual extraction garbage the conservative gate missed (prose fragments
  `closeout/review-ready` and `linux/mac/windows`); a slightly stronger prose/path guard rejects those
  too, taking extraction to effectively ~`5 of 12` killed.
- the remaining ~`7` are legitimate narrowing / progression / work-cycles: the canonical narrowing
  `audit-trio.report.json → audit-trio.model-selection/cohesion-audit.report.json`; the
  `PLAN-04.md → exec.rs → async_repl.rs → PLAN-04.md` plan→code→plan cycle (3 pairs);
  `status/risks.md → sprint-planning.md` doc progression; and two more real doc/step changes still
  carrying markdown-link / space-split-path mangling.

**Gate verdict.** Extraction hardening (Step 1) is confirmed the dominant lever: it eliminates every
real firing and the bulk of the disjoint firing surface, and the only residue is exactly the
narrowing / progression that the graduated-distance metric (`R6-3.X.2`, Step 3) is designed to absorb.
No genuine pivot surfaces, so the eligibility bar (`R6-3.X.3`, Step 4) stays deferred. This
quantitatively confirms the locked ordering. The ground truth also hands Step 1 a witness-backed
defect list: escaped-`\n` blobs ingested as paths (the dominant poison), unparsed `[text](path)`
markdown links, space-split truncated paths, JSON agent-status blobs, prose enumerations, and bare
model tokens.

## R6-3.5 Result (executed 2026-07-04): Extraction Hardening Landed + Full Re-Run

The charter's Step 1 (extraction hardening) landed as packet `R6-3.5` (see
`docs/specs/r6/R6-3.5/` spec/plan/tasks), and Step 2's full shadow-eval — the real batch re-run with
the new extraction code applied, scorer distance metric unchanged — has now been run.

Design (deterministic, no model): a shared `TargetAnchorQuality{Stable,Weak,Junk}` classifier in
`context/objective.rs`, grounded in two established literatures — log-template variable masking
(Drain/LogPai "Preprocessing is All You Need"; LogPPT) for the noise masks, and schema-guided
dialogue typed slots (SGD/DSTC8/FastSGT) for the typed-anchor grammars. Grammar-first: a token
validating a typed anchor grammar (repo-relative path, recognized-extension file, well-known rootless
file, Windows path, Rust symbol ref, bare crate name, work-item id, workspace ref, instruction
surface) is `Stable` and never re-masked; otherwise variable masks (URL/endpoint, `host:port`,
numeric/coordinate run, hex, duration/timestamp, escaped-control residue) decide `Junk` vs
plausible-but-untyped `Weak`. Only `Stable` seeds `target`/`comparison_key`. The same taxonomy backs a
scorer stable-term backstop (`eligible_current_goal` requires ≥1 stable, non-constraint distinguishing
term) and a bounded opaque-delegated-parent guardrail. The design was reviewed read-only by Codex
before implementation (widen Stage-B grammars, keep `Weak` non-scoring, one shared classifier, hard-gate
only `Opaque` parents) and all its deltas were folded in.

Batch re-run (110 sessions, 43 repos, seeded sample; `877` checkpoints — a fresh draw from the grown
`~/.codex/sessions` store, so this is a directional before→after, not the identical corpus):

| metric | before (F1–F4 / Gate) | after (R6-3.5) |
| --- | --- | --- |
| `semantic_goal_drift` fired checkpoints | `6` (all false positives) | **`0`** |
| rolling + kickoff evidence lines | `1` + `6` | **`0` + `0`** |
| current-bar eligible | `156` (17.7%) | `137` (15.6%) |
| current-bar eligible resting on 100%-junk targets | `8 / 156` | **`0 / 137`** |
| target-resolved eligible | `348` (39.5%) | `234` (26.7%) |
| target-resolved disjoint adjacent pairs | `12` | `5` |
| …that survive a stable-only junk filter | `9` | **`5` (0 killed as junk)** |
| …that are genuine "abandoned goal A for goal B" pivots | `0` | **`0`** |

The `6 → 0` fires and `8 → 0` junk-only eligible are the decisive, sample-robust signals: the junk
targets that produced 100% of the observed over-fire no longer become eligible. The lower
target-resolved count (39.5% → 26.7%) is the intended junk removal (garbage no longer counts as a
grounded target), not a false-negative regression — the surviving eligible targets are legitimate
typed anchors on hand inspection (`README.md`, `architecture-overview.md`, `sprint-planning.md`,
`test_azure_blob_client.py`, `templates.contract.ts`, `R6-2.2`, `tests/fixtures`), and the codex
false-negative watch-list (`Cargo.lock`, `.github/workflows/ci.yml`, bare crate names, `foo::bar`,
`v2.3.1/notes.md`) is covered by committed unit tests.

All `5` surviving disjoint pairs are legitimate narrowing / progression / work-cycles, not pivots:
`risks.md,status.md → sprint-planning.md` (doc progression); a `plan/spec/tasks → spec_plan` narrowing;
the canonical `audit-trio.report.json → audit-trio.model-selection/cohesion-audit.report.json`;
`…seam-6b…handoff-boundary.md → …harness-convergence-threading.md`; and the
`async_repl.rs:497 → llm-last-mile/PLAN-04.md` plan→code→plan cycle. This is exactly the residue the
graduated-distance metric (`R6-3.X.2`, Step 3) is designed to absorb, so the eligibility bar
(`R6-3.X.3`, Step 4) stays deferred. Locked ordering re-confirmed on real data.

**Delegation caveat (R6-3.5 stratification).** The batch is now reported stratified by a coarse
session-level delegation marker scan (`scripts/dev/drift-batch-scan/`): `single_agent` = `761` cp /
`130` current-bar eligible / **`0`** fired / `2` disjoint pairs; `delegated_child_visible` = `116` cp /
`7` eligible / **`0`** fired / `3` disjoint pairs (no `delegated_parent_opaque` surfaced in this
sample). Fires are `0` in every category. Precision claims are reported per category rather than
pooled; opaque delegated sessions remain **secondary** evidence until R7-style parent/child semantic
support exists. This tag is a reporting heuristic, coarser than the analyzer's per-checkpoint
`DelegationContext`, which is not serialized into the checkpoint export.

**R6-3.6 Result (2026-07-05): positive-control wall + corpus revalidation.** The acceptance corpus now
holds `10` bounded semantic-goal-drift cases: **`5/5` positive controls pass** (the two pre-existing
true-positive witnesses plus new hyphen-collision, same-basename-without-extension, and sibling-stem
boundary pivots) and **`5/5` negative controls stay quiet** (kickoff/rolling containment narrowing,
same-file `:line` narrowing, pure broadening into a containing directory, and same-target
review→verify progression). The live acceptance harness now fail-closes on a curated allowlist rather
than a frozen count of three and asserts full `unknowns.is_empty()` eligibility plus non-empty target
evidence for every checkpoint whose target it checks, so the positive controls prove real shipping-bar
eligibility instead of target-only extraction.

The corpus harness was then re-run on a fresh seed-42 draw from the grown store: `110` sessions /
`44` repos / `938` checkpoints, `110/110` analyzed clean. Fires remain **`0`** flagged checkpoints,
`0` rolling evidence lines, and `0` kickoff-anchor evidence lines — no over-fire regression after the
containment first cut. Export-derivable funnel counts: `938` checkpoints with `structured_objective`,
`263` with a structured target, `255` with an analysis-only stable-target proxy, `138`
current-bar-eligible checkpoints (14.7%), `263` target-resolved eligible checkpoints (28.0%), `828`
adjacent checkpoint pairs total, `221` adjacent pairs with both sides target-eligible, `213`
same-target exact-match suppressions, `8` changed-target candidate pairs, `7` remaining disjoint pairs,
and `0` emitted `semantic_goal_drift` fires. Honest export limits are now printed by
`scripts/dev/drift-batch-scan/tabulate.py`: structural-containment suppressions, scorer-true
stable-target-hygiene suppressions, and `sanctioned_replan` suppressions are **not derivable** from the
current checkpoint export without duplicating Rust scorer logic or widening the export surface.

The `7` remaining disjoint pairs are still **all non-pivots** on hand inspection: doc progression
(`status.md,risks.md → sprint-planning.md`), generic `spec/plan/tasks → spec_plan` narrowing, family-stem
`audit-trio.report.json → audit-trio.model-selection/cohesion-audit.report.json`, sibling work under one
directory (`…handoff-boundary.md → …threading.md`), the longstanding plan→code→plan residue
(`async_repl.rs:497 → llm-last-mile/PLAN-04.md`), an absolute-vs-repo-relative docs-survey progression
(`docs/legacy` / `docs/contracts` → specific `handbook` legacy/contract files), and a broad
spec→examples progression (`spec/task → examples/*`). None is a subtree narrowing the containment
carve-out should have absorbed. The newly-observed absolute-vs-repo-relative residue is still in the
conservative direction (over-fire, never drift-masking), and unlike the bare-`CrateOrPackage`
over-fire it is plausibly closable under the remaining graduated-distance work because the bundle
already carries `session_meta.payload.cwd`.

Delegation split on this re-run: `single_agent` = `760` cp / `130` current-bar eligible / `186`
target-resolved eligible / `0` fired / `2` disjoint pairs; `delegated_child_visible` = `178` cp / `8`
current-bar eligible / `77` target-resolved eligible / `0` fired / `5` disjoint pairs; no
`delegated_parent_opaque` or `unknown` sessions surfaced in this draw. Conclusion: the shipping bar is
still precise on real data, recall now has an explicit positive-control wall, and the next justified
packet remains the **remaining graduated / weighted-distance work in `R6-3.X.2`**. The
eligibility-bar revisit (`R6-3.X.3`) stays deferred behind it.

**R6-3.X.2B / R6-3.X.2C Result (2026-07-06): explicit drift-decision routing + honest rerun residue.**
`R6-3.X.2B` landed the analyzer-local relation taxonomy and supporting batch strata, but it did **not**
finish the routing story: the scorer still routed via relation-only `claims_drift()` even though the
packet docs described a richer relation + confidence + evidence contract. `R6-3.X.2C` reopened that
gap and landed the missing decision surface locally in
`crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`:
- explicit `Suppress` / `Fire` / `NoClaim` routing,
- relation family remains primary,
- numeric score is explanatory only,
- weaker suppressive families now require `confidence >= Medium`, decisive evidence, and no material
  counter-evidence,
- same-crate residue and broad work-item lineage no longer suppress on their own,
- doc-bundle behavior is covered scorer-locally plus through a live acceptance fixture, and
- code/test path markers now beat `spec` / `design` substrings during role classification.

The default seed-42 corpus rerun was repeated from the **current** session store with `--seed 42`, so
the result is again an explicit **directional** comparison against the published `2026-07-05`
`R6-3.6` baseline, not a claim of identical session identity. Post-`R6-3.X.2C` rerun totals:
- `110/110` sessions analyzed clean across `44` repos
- `929` checkpoints with `structured_objective`
- `242` structured-target checkpoints
- `238` analysis-only stable-target-proxy checkpoints
- `137` current-bar eligible checkpoints
- `242` target-resolved eligible checkpoints
- `819` adjacent checkpoint pairs total
- `203` adjacent pairs with both sides target-eligible
- `195` same-target exact-match suppressions
- `8` changed-target candidate pairs
- `7` remaining disjoint pairs
- `3` emitted `semantic_goal_drift` fires (`1` rolling evidence line, `3` kickoff-anchor evidence lines)

Those `3` fires are not a reopened relation-only routing bug. They are one docs-session residue family:
root-level **doc-bundle → anchored member-doc narrowing**
(`architecture-overview.md|README.md|…` → `README.md`) where the bundle lacks a stable shared prefix.
`R6-3.X.2C` intentionally suppresses doc-bundle movement only when the same-doc-family evidence is clear;
it does **not** suppress ambiguous root-level bundles on relation type alone anymore. The rerun therefore
remains conservative in the safe direction: over-fire, not drift masking.

The junk-filter post-pass confirms this is not garbage-target noise:
- `3/3` actual fires survive the junk suppression filter
- `0/137` current-bar eligible checkpoints rest on junk-only targets
- `7/7` target-resolved disjoint adjacent pairs survive the stable-only junk filter

Validation strata still report with explicit heuristic sources and without any added table collapsing to
`100% unknown`. `RepoRelativeEquivalentAfterCwdStrip` stays explicitly **deferred**: the batch still
shows absolute-vs-repo-relative residue, but this packet did not prove a scorer-local cwd/session-root
reachability seam, so widening into `context/objective.rs` or other deferred surfaces would have broken
packet scope. Final routing decision: **`R6-3.X.2` is now closed through `R6-3.X.2C`; `R6-3.X.3`
remains deferred** because the remaining residue is conservative family/progression over-fire, not
evidence that the strict eligibility bar is masking genuine pivots.

Source map (deterministic design inputs, not repo authority): log-template variable abstraction —
[Preprocessing is All You Need (arXiv 2412.05254)](https://arxiv.org/pdf/2412.05254),
[Drain3](https://github.com/logpai/Drain3), [LogPPT (arXiv 2302.07435)](https://arxiv.org/abs/2302.07435);
schema-guided typed slots — [SGD (arXiv 1909.05855)](https://arxiv.org/pdf/1909.05855),
[FastSGT (arXiv 2008.12335)](https://arxiv.org/pdf/2008.12335) (the R5 objective-classifier taxonomy
already cites this family).

## Next-Steps Charter (for a dedicated fresh session)

### Step 1 (primary) — Objective-extraction robustness in `context/objective.rs`

Reject non-goal "targets" so they never enter `target` / `comparison_key` state; fall back to
`target = None` / `unknown_target`. Concretely, guard the target-anchor path
(`explicit_target_anchor_for_text`, `extract_inline_paths`, `looks_like_explicit_named_target`,
`comparison_key_from_structured`) against: pure whitespace / newline fragments; coordinate/viewport/number
runs (`0_0_0_0_4000`, `5_n_n`); bare model names / version tokens (`GPT-5.4`); truncated path fragments; and
prose snippets that are not paths/symbols/named artifacts. Add a secondary backstop in the scorer:
`eligible_current_goal` (or `goal_specific_terms`) should require at least one **stable** target term after
normalization and reject junk-only term sets. Prefer the extraction fix as primary; the scorer backstop is
defense in depth. This must not regress the `R6-2`/`R6-3` acceptance fixtures.

### Step 2 — Shadow-eval probe (cheap, gates the rest)

With Step 1 applied, re-run the batch scan (same 110-session sample, or a fresh one) with the scorer
unchanged and measure, before vs after: eligible-checkpoint count, live flagged/firing count, and the fate
of the 12 target-resolved disjoint adjacent pairs from F3. If most of the 12 disappear, extraction was the
dominant fix. If many survive, they are the narrowing/progression cases that prove the graduated-distance
work (`R6-3.X.2`) is the next blocker. A cheaper first cut can be done as an analysis-only junk-target filter
over the existing batch data before writing extraction code. **Done (2026-07-03): see the "Gate Result"
section above — the cheap cut confirmed extraction is the dominant lever (0/6 fires survive, 12→9 pairs,
no genuine pivot). Full shadow-eval DONE (2026-07-04): extraction code landed as `R6-3.5` and the real
re-run confirmed `6 → 0` fires, `8 → 0` junk-only eligible, `12 → 5` disjoint pairs (all legitimate
progression). See the "R6-3.5 Result" section above.**

### Step 3 (conditional) — Graduated / weighted distance (`R6-3.X.2`)

Only if Step 2 shows narrowing/progression false positives survive extraction hardening. Replace the binary
disjoint-set overlap with a graduated distance across both the kickoff-anchored and rolling comparisons so a
narrowing (crate root → one file inside it) or a plan→code→plan cycle is not read as drift.

**Containment first cut DONE (2026-07-05).** Step 2's condition was met (all 5 post-`R6-3.5` disjoint
pairs are legitimate narrowing/progression), so the bounded first cut of this step landed: divergence in
`scoring/semantic_goal_drift.rs` now treats two goals as related when one structured target
structurally contains the other, absorbing the canonical crate/directory → contained-file narrowing
(and `foo::bar` → `foo::bar::baz`, and the broadening direction) on both the kickoff-anchored and rolling
comparisons. Containment is computed on the **raw target strings** split only on real structural
separators (`/`, `\`, `::`). A codex review of the first draft (which tested containment on the
normalized `_`-flattened terms) found that approach would treat `docs/specs/r6-map` and
`docs/specs/r6/map.md` as ancestor/descendant and silently drop a real pivot, because `normalize_goal_term`
collapses `/`, `-`, and `.` to the same `_`; splitting the raw path on structural separators only keeps
`-`/`.` inside a segment, so that pivot still fires. Sibling artifacts sharing a stem still fire and every
pinned true-positive fixture is preserved; a new acceptance case
(`synthetic-kickoff-narrowing-into-anchored-subtree`) pins the suppression through the live analyzer path,
and a scorer-level regression guard pins the hyphen/slash collision. Multi-anchor containment is symmetric,
not any-pair (codex re-review §P2, two rounds: a multi-target goal that adds an unrelated target — whether
narrowing or broadening the others — still fires, because every concrete anchor of both goals must be
structurally related to some anchor of the other). Segment comparison is case-sensitive
(codex re-review §P3: `Foo::Bar` ≠ `foo::bar` for Rust symbols and case-sensitive filesystems, so a
case-only difference stays a real pivot). Segmentation canonicalizes identity-preserving spellings first
(codex re-review round 4): a `./` current-dir segment is dropped and a trailing `:line`/`:line:col`
reference is stripped, so a narrowing that only adds a `./` prefix or pins a line (`exec.rs` →
`exec.rs:1537`) no longer fires — both forms are no-ops, so the canonicalization can only remove
over-fires, never mask a pivot (a Rust `a::b` tail is non-numeric and left intact; a dotfile dir like
`.github` stays a real segment). The line strip runs on the leaf segment after path splitting, not on
the whole string (codex re-review round 5): a whole-string strip stops at a Windows drive-letter colon
(`C:/repo/src/lib.rs:42` has non-numeric tail `/repo/src/lib.rs:42`), so the same-file narrowing would
still fire on Windows absolute paths; per-leaf application mirrors how the upstream `strip_line_ref` in
`context/objective.rs` is applied. Still open under this step: the genuinely graduated/weighted
metric for family-stem narrowing (`audit-trio.report.json → audit-trio.model-selection/…report.json`), doc
progression, plan→code→plan cycles, and dotted work-item narrowing (`R6-3 → R6-3.5`, no structural
separator); a known residual over-fire where a bare `CrateOrPackage` name is not matched as an ancestor of
the crate's path form (codex re-review §P2, deferred — closing it needs the package-root convention and
over-firing never masks drift); a known residual over-fire on case-insensitive filesystems (codex round 6,
deliberately declined): a case-only respelling of the same path (`C:/Repo/src` → `c:/repo/src/lib.rs`,
`MAP.md` → `map.md`) fails containment and still fires — codex round 2 (§P3) demanded the opposite
(case-insensitive comparison masks `Foo::Bar` → `foo::bar` symbol pivots and real path pivots on Linux, the
drift-masking direction), the analyzer cannot know the traced filesystem's case semantics from the bundle,
and over-firing never masks drift, so the case-sensitive §P3 decision stands until filesystem-semantics
metadata or an anchor-type-aware rule lands with the graduated metric; a known residual over-fire where an
absolute and a repo-relative spelling of the same subtree do not relate on raw segments (`docs/legacy` vs
`/Users/…/handbook/docs/legacy/HARNESS.md`; observed in the 2026-07-05 corpus re-check — closable via
`session_meta.payload.cwd` prefix-stripping under the graduated metric, and over-firing never masks drift);
plus the shared-constraint-masking false negative and the anchor comparison_key asymmetry. See the `R6-3`
TASKS ledger `R6-3.X.2` for the landed/open split.

### Step 4 (conditional) — Revisit loosening the eligibility bar (`R6-3.X.3`)

Only after Steps 1–3. With garbage suppressed and the distance metric graduated, re-measure whether a
target-resolved bar now yields genuine pivots at acceptable precision. This remains a cross-signal change
(shared `eligible_current_goal` helper affects `R6-2` kickoff), so it needs its own SPEC/PLAN/TASKS delta and
impact analysis.

## Non-Goals

- Re-opening the `R6-3` surfacing decision (evidence-on-`SemanticGoalDrift`, no variant, no `schema_version`
  bump) — that stands.
- Touching `progress.rs` comparability/reset (`R6-4`, still deferred).
- Shipping any eligibility-bar loosening before extraction hardening lands.
