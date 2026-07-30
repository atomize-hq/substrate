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

# 1. freeze an exact CurrentNativeV2 inventory at an explicit cutoff, then select from it
python3 "$REPO/$S/sample_sessions.py" \
  --as-of 2026-07-29T23:59:59Z \
  --inventory-out candidate_inventory.jsonl \
  --out selected_sessions.jsonl \
  --receipt-out selection_receipt.json

# 2. run each session through compactor->analyzer in its own temp codex-home
#    (isolation avoids cross-session exact-dedupe contamination in the compactor)
python3 "$REPO/$S/run_batch.py" \
  --repo "$REPO" \
  --selected selected_sessions.jsonl \
  --selection-receipt selection_receipt.json \
  --batch-dir batch \
  --batch-receipt batch/batch_receipt.json

# 3. coverage diagnostic (F1-F4 plus the explicit R6-3.6 funnel: eligibility, firings,
#    target-resolved bar, adjacent pairs, diversity, delegation split, and honest export limits)
python3 "$REPO/$S/tabulate.py" \
  --checkpoints-dir batch/checkpoints \
  --batch-receipt batch/batch_receipt.json \
  --receipt-out batch/p7_private_receipt.json

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
- best-effort language / repo type, workflow type, and tooling type strata

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

## Best-effort validation strata (R6-3.X.2B)

Without widening the checkpoint export, `tabulate.py` also emits three additional checkpoint-level
strata, each with an explicit heuristic-source note printed alongside the table:

- **language / repo type** — inferred from `task_frame.working_set_paths`,
  `structured_objective.target.{paths,named_artifacts,workspace_refs,symbols,display}`, plus
  command/tool and file-extension hints; buckets: `rust`, `js_ts`, `python`, `docs_only`, `mixed`,
  `unknown`.
- **workflow type** — inferred from `session_archetype.label` and `session_progress.dimension`, with
  `structured_objective.primary_intent` / verification-command fallback; buckets: `implementation`,
  `docs_planning`, `verification`, `review_fix`, `mixed`, `unknown`.
- **tooling type** — inferred from `task_frame.command_families`, `task_frame.tools`,
  `task_frame.verification_commands`, with file-extension fallback when command evidence is absent;
  buckets: `cargo_rust`, `node_npm`, `python_pytest`, `generic_filesystem_doc`, `unknown`.

These are intentionally **best-effort reporting heuristics**, not new analyzer facts. If a whole
table resolves to `100% unknown`, `tabulate.py` prints an explicit unresolved warning so the packet
does not claim that stratum as validated.

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

## P7 frozen inventory contract

`sample_sessions.py` mirrors the repository's exact raw-adapter route: only sources whose
`session_meta.payload.multi_agent_version` markers select `CurrentNativeV2` enter the candidate
inventory. Missing, `disabled`, or `v1` markers remain Legacy; unsupported, non-string, or
conflicting markers are excluded.

The required `--as-of` value is an inclusive RFC3339 cutoff over the synthetic/session metadata
timestamp. The script sorts the accepted inventory canonically and hashes a record containing the
relative source location, source digest, session metadata, and cutoff. Enumeration order and the
absolute inventory root therefore do not affect the digest, while any candidate-content or
inventory change does.

Before selection begins, the script constructs one immutable quota contract. It freezes the seed,
the required `language_repo`, `workflow`, `tooling`, and `delegation` families, 3-session ordinary
bucket quotas, 5-session high-risk delegation quotas, and the population at which each quota becomes
mandatory. P7's overlapping-quota selector consumes this frozen value; it does not mutate quota
authority during selection.

Candidates receive every named label visible in task-owned user/directive text, typed tool calls,
current workspace metadata, and typed delegation origin. Global base instructions and raw tool
outputs are excluded from labeling so common prompt/tool-output vocabulary cannot make every
session look like every stratum. `unknown` remains visible but never earns a named-bucket credit.

Selection is deterministic greedy set cover. Each candidate's gain is the number of still-unfilled
named buckets it genuinely covers; the frozen seed is consulted only when gains tie. The selector
must return a non-empty set, emits a selected-set digest, and validates every bucket after
selection. A bucket with at least its mandatory population must meet quota. A scarce bucket may
remain underfilled only when every eligible candidate was selected; that limitation is emitted as
`permitted_inventory_scarcity` rather than hidden or backfilled with an unrelated session.

## Privacy-safe P7 receipts

The three P7 stages carry authority through two sanitized sidecars:

- `selection_receipt.json` contains only the inventory cutoff/digest/count, immutable quota
  configuration/digest, selected-set digest/count, and per-bucket eligible/selected/underfill
  results.
- `batch/batch_receipt.json` adds only attempted/succeeded/failed counts and coarse failure kinds.
- `batch/p7_private_receipt.json` adds aggregate analyzer/funnel counts and heuristic stratum counts.

The receipt builders recursively reject raw path, session, repository, and message fields or
absolute-path string values. The tabulator's repository table is rank-anonymized. Its language,
workflow, tooling, and delegation tables remain observable distribution heuristics only; the
receipt explicitly refuses scorer-internal containment, stable-target-hygiene, or sanctioned-replan
claims.

## Sampling notes & caveats

- Rollout cutoff/month comes from the selected `session_meta` timestamp; repo/cwd comes from its
  payload and is normalized by collapsing `/worktrees/<hash>/` and `/.conductor/<name>/` segments
  so worktrees of one repo count as that repo.
- **Format cutoff (F0):** rollouts before ~2025-09 predate `session_meta` (first line is
  `{id, timestamp, instructions}` with no `cwd`); the analyzer rejects those bundles, so the
  analyzable window is ~the last 11 months. `sample_sessions.py` still lists earlier months but they
  yield little.
- The seed (`--seed 42`) is only a stable tie-breaker. Reproduction requires the same explicit
  cutoff and inventory digest; a growing `~/.codex/sessions` store is not silently treated as the
  same authority.

## Not committed

The generated `candidate_inventory.jsonl`, `selected_sessions.jsonl`, checkpoint JSONL, and status
JSONL are intentionally **not** committed: they contain absolute paths into a private
`~/.codex/sessions` store, private identifiers, and derived session content. Keep them in a scratch
directory. Only a recursively validated digest/count/underfill aggregate receipt is eligible for
committed P7 evidence.
