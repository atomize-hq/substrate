# R5 Follow-ups and Known Limitations

Status: living follow-up register for post-packet R5 notes that are real, grounded, and worth
tracking, but not currently severe enough to reopen an already-landed packet.

## Why This Doc Exists

R5 packet docs capture what must land for the feature to be considered implemented. They are not a
good place to accumulate every residual robustness note, review caveat, or future hardening idea.

This doc is the reusable place for those items.

Use it for:

1. non-blocking limitations confirmed against live code or tests,
2. review findings that were judged real but not packet-blocking,
3. robustness gaps that should inform later packeting or hardening work,
4. follow-up test ideas that should not be forgotten.

Do not use it for:

1. unresolved packet blockers that must be fixed before calling a packet done,
2. vague speculation without a concrete code/doc/test anchor,
3. superseded notes that no longer describe current repo reality.

## Entry Template

Each follow-up item should record:

- **ID**
- **Area**
- **Status**
- **Concern level**
- **Current repo reality**
- **Why it is not a current blocker**
- **Potential failure mode**
- **Suggested follow-up**
- **Evidence**

## Follow-up Items

### R5-FU-001: Material-objective reset is still heuristic

- **Area:** progress-window comparability resets
- **Status:** open
- **Concern level:** moderate robustness follow-up; not a blocker for landed `R5-4`

#### Current repo reality

The current reset logic in `crates/agent-drift-analyzer/src/checkpoint/progress.rs` still uses
mostly lexical heuristics for objective-change detection:

1. `explicit_replan_boundary(...)` checks for objective text changes plus keywords such as
   `replan`, `pivot`, `switch`, and `instead`.
2. `material_objective_delta(...)` tokenizes objective text through `objective_terms(...)` and
   resets when the term sets are mostly unrelated.
3. `objective_continues_current_line_of_work(...)` suppresses resets when the objective text
   contains continuity markers such as `same`, `re-run`, `rerun`, `follow-up`, `latest`,
   `continue`, or `again`.

This means the reset logic is not yet doing deeper semantic comparison of objective continuity vs
objective change.

#### Why it is not a current blocker

This limitation does **not** currently justify reopening `R5-4` because objective text is only one
comparability input. The same reset path is also guarded by:

1. strong archetype changes,
2. truth-artifact shifts,
3. working-set pivots,
4. delegation visibility changes.

So even when the objective-text heuristic is imperfect, other analyzer-local signals still catch
many real boundary changes.

#### Potential failure mode

Two classes of mistakes remain plausible:

1. a real material pivot with many shared words could fail to reset the progress window, and
2. an unusually phrased continuity objective could reset too aggressively.

The likely impact is incorrect comparability windows, which can in turn cause occasional false
`stalled`, `regressing`, or `insufficient_evidence` outcomes. This is a robustness issue, not a
known catastrophic contract break.

#### Suggested follow-up

If this becomes important enough to harden, the next bounded steps should be:

1. add synthetic tests for paraphrased objective pivots that still share substantial vocabulary,
2. add synthetic tests for continuity phrasing that does not use the current marker list,
3. consider a more structured comparability signal that weights objective text alongside working
   set, truth artifacts, and archetype continuity instead of relying primarily on lexical overlap,
4. keep any future change conservative and fail-closed so weak evidence does not overclaim
   continuity.

#### Evidence

- Code:
  - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `explicit_replan_boundary(...)`
    - `objective_or_truth_artifacts_shifted(...)`
    - `material_objective_delta(...)`
    - `objective_continues_current_line_of_work(...)`
    - `objective_terms(...)`
- Existing synthetic coverage:
  - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - `checkpoints_reset_comparability_after_an_explicit_replan_and_working_set_pivot()`
    - `checkpoints_reset_comparability_after_a_material_objective_pivot_without_replan_keywords()`
- Gap still not directly covered:
  - paraphrased pivots with heavy shared vocabulary,
  - continuity phrasing outside the current marker list.
