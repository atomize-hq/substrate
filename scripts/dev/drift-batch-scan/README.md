# drift-batch-scan — real-world validation harness for `semantic_goal_drift`

Diagnostic tooling that runs a diverse sample of real Codex rollout sessions through the live
`agent-session-compactor` → `agent-drift-analyzer` pipeline and tabulates how the
`semantic_goal_drift` signal (both the `R6-2` kickoff-anchored and the `R6-3` rolling comparisons)
behaves on real data.

This is the harness behind the findings in
[`docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`](../../../docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md)
and the "Batch scan outcome" / gate re-count in
[`docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`](../../../docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md)
(task `R6-3.X.3`). Read the FINDINGS doc first; it holds the interpretation, the locked decision,
and the next-steps charter. These scripts just regenerate the evidence.

## Pipeline

Run from a scratch directory (outputs land in `./selected_sessions.jsonl` and `./batch/`).

```bash
# 0. build the two binaries the batch driver shells out to
cargo build -p agent-session-compactor -p agent-drift-analyzer

S=scripts/dev/drift-batch-scan          # from repo root
OUT=/tmp/drift-batch                     # any writable scratch dir
mkdir -p "$OUT" && cd "$OUT"

# 1. sample ~10 sessions/month across the analyzable months, maximizing distinct repos
python3 "$REPO/$S/sample_sessions.py" --out selected_sessions.jsonl

# 2. run each session through compactor->analyzer in its own temp codex-home
#    (isolation avoids cross-session exact-dedupe contamination in the compactor)
python3 "$REPO/$S/run_batch.py" --repo "$REPO" --selected selected_sessions.jsonl --batch-dir batch

# 3. coverage diagnostic (F1-F4 plus the explicit R6-3.6 funnel: eligibility, firings,
#    target-resolved bar, adjacent pairs, diversity, delegation split, and honest export limits)
python3 "$REPO/$S/tabulate.py" --checkpoints-dir batch/checkpoints

# 4a. ground truth: raw targets behind the fires and the disjoint pairs
python3 "$REPO/$S/inspect_targets.py" --checkpoints-dir batch/checkpoints

# 4b. the cheapest gate: suppress junk targets, re-count fires / eligible / disjoint pairs
python3 "$REPO/$S/filter_junk.py" --checkpoints-dir batch/checkpoints
```

Each row written to `batch/checkpoints/<session_id>.jsonl` is the analyzer's checkpoint record with
four added keys — `_month`, `_repo`, `_session_file_id`, and `_delegation` — used by the tabulators.

## R6-3.6 funnel coverage and limits

`tabulate.py` now prints an explicit funnel from:

- total sessions / checkpoints
- `structured_objective` coverage
- structured-target presence
- an **analysis-only stable-target proxy** (cheap heuristic over exported targets; not the exact Rust
  scorer decision)
- current-bar eligible checkpoints
- target-resolved eligible checkpoints
- total adjacent pairs
- target-resolved adjacent candidate pairs
- same-target exact-match suppressions
- remaining disjoint pairs
- emitted `semantic_goal_drift` fires
- delegation-category stratification

What it **cannot** report from the current checkpoint export without either duplicating the Rust scorer or
widening the analyzer export/schema:

- structural-containment suppressions
- scorer-true stable-target-hygiene suppressions
- `sanctioned_replan` suppressions

Those limits are printed by `tabulate.py` so a zero-fire or non-zero-fire outcome is not overstated as a
fully explained scorer funnel.

## Delegation stratification (R6-3.5)

`run_batch.py` tags each session with a coarse `_delegation` category by scanning the raw rollout for
the analyzer's delegation markers (`spawn_agent` / `wait_agent` / `close_agent` / `multi_agent_v1`
and child-visibility signals like `child rollout` / `subagent`): `single_agent`,
`delegated_parent_opaque`, `delegated_child_visible`, or `unknown`. `tabulate.py` then reports
checkpoints / eligibility / fires / disjoint-pairs **separately per category**, so precision claims
from the batch are not made by treating delegated or opaque parent-only traces as equal-weight
evidence for single-agent sessions.

This tag is a *reporting* aid and is intentionally coarser than the analyzer's own per-checkpoint
`DelegationContext` (topology + `child_work_visibility`), which is not serialized into the checkpoint
export. Opaque delegated sessions remain **secondary** evidence until R7-style parent/child semantic
support exists; see the FINDINGS delegation caveat.

## What `filter_junk.py` answers (the cheapest gate)

The FINDINGS charter's Step 2 calls for an "analysis-only junk-target filter over the existing batch
data before writing extraction code," to confirm objective-extraction robustness is the dominant
lever before touching the distance metric or the eligibility bar. `filter_junk.py` is that gate: it
suppresses obvious extraction garbage at the term level (escaped-newline residue, number/coordinate
runs, bare model/version tokens, prose-`etc` enumerations) and re-counts survivors. On the recorded
110-session batch it showed **0/6 actual fires survive** and **12 → 9 disjoint pairs**, with none of
the 9 survivors being a real "abandoned goal A for unrelated goal B" pivot — they are residual
extraction garbage plus legitimate narrowing / progression / work-cycles. See the FINDINGS "Gate
Result" section for the full breakdown.

The junk heuristic here is deliberately conservative — it is a diagnostic gate, **not** the
production extractor. The real fix lives in `crates/agent-drift-analyzer/src/context/objective.rs`
(charter Step 1).

## Sampling notes & caveats

- Rollout date comes from the path (`sessions/YYYY/MM/DD/`); repo/cwd from line 1
  (`session_meta.payload.cwd`), normalized by collapsing `/worktrees/<hash>/` and `/.conductor/<name>/`
  segments so worktrees of one repo count as that repo.
- **Format cutoff (F0):** rollouts before ~2025-09 predate `session_meta` (first line is
  `{id, timestamp, instructions}` with no `cwd`); the analyzer rejects those bundles, so the
  analyzable window is ~the last 11 months. `sample_sessions.py` still lists earlier months but they
  yield little.
- The sample is seeded (`--seed 42`) for reproducibility, but the underlying `~/.codex/sessions`
  store grows over time, so a re-run will not reproduce the exact same 110 sessions.

## Not committed

The generated data — `selected_sessions.jsonl` and `batch/` — is intentionally **not** committed:
it is regenerable from these scripts and contains absolute paths into a private
`~/.codex/sessions` store plus derived session content. Keep it in a scratch dir.
