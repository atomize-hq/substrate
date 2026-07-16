# R7-5.2 Findings: Delegated Session Validation

Status: **PASS on 2026-07-15 at `afb10827d`**. The named R3.75/R5.75 parent
witnesses, the verified `019e93f8-...` -> `019e93fa-...` pair, and the committed
`da59436e63915185` derivative were re-run through compactor -> analyzer. The
results preserve trajectory-local progress and scoring, fail closed without
reciprocal child evidence, and commit no private corpus material.

This report is the only R7-5.2 tracked artifact. Generated bundles, analyzer
outputs, command logs, and the bounded private input mirror remain ignored under
`target/r7-5-real-corpus/`.

## Scope And Input Receipt

The five named native parent sessions are:

| Authority | Parent session | Claimed direct children |
| --- | --- | ---: |
| R3.75 | `019e93f8-a5e9-7490-ac1a-955b74c92ad0` | 2 |
| R3.75 | `019e9406-6736-79a2-946b-8a603e557422` | 2 |
| R5.75 | `019eb907-95c4-73e1-843e-e337d1e93cb9` | 4 |
| R5.75 | `019eb917-9531-74e0-897d-ad8d362138ec` | 6 |
| R5.75 | `019eb970-3543-7ab1-a5d6-2a62c00c7185` | 8 |

The ignored bounded input mirror contained exactly the five parents plus all 22
unique child artifacts named by their matched spawn results. Every claimed child
had exactly one artifact and reciprocal depth-1 origin metadata. There were zero
missing child artifacts, duplicate session artifacts, conflicting parent claims,
or depth-greater-than-one descendants in this named native corpus.

The bounded mirror was used because an unbounded linked-closure scan of the full
local history exceeded a 120-second recovery-command limit. That timed-out scan
is not evidence for this report. The bounded mirror retained every artifact that
could affect these roots: each named root, each artifact whose session id matched
a claimed child, each artifact with a matched claim for one of those children,
and any deeper origin attached to one of those children. The resulting 27-file
receipt is complete for the compactor's direct-closure selection rule; no product
or test script was needed.

## Reproducible Commands

`R7_REAL_CORPUS_CODEX_HOME` is a local, ignored Codex home containing the bounded
input receipt above. It must never be committed. The actual compactor -> analyzer
invocation was:

```bash
cargo build -p agent-session-compactor -p agent-drift-analyzer

export R7_REAL_CORPUS_CODEX_HOME="<local ignored Codex-home snapshot>"
export R7_REAL_CORPUS_OUT="target/r7-5-real-corpus"

parents=(
  019e93f8-a5e9-7490-ac1a-955b74c92ad0
  019e9406-6736-79a2-946b-8a603e557422
  019eb907-95c4-73e1-843e-e337d1e93cb9
  019eb917-9531-74e0-897d-ad8d362138ec
  019eb970-3543-7ab1-a5d6-2a62c00c7185
)

for session_id in "${parents[@]}"; do
  for mode in default linked; do
    run_root="$R7_REAL_CORPUS_OUT/native/$session_id/$mode"
    rm -rf "$run_root"
    mkdir -p "$run_root"

    linked_args=()
    if [[ "$mode" == linked ]]; then
      linked_args+=(--include-linked-children)
    fi

    target/debug/agent-session-compactor \
      --codex-home "$R7_REAL_CORPUS_CODEX_HOME" \
      --session-id "$session_id" \
      "${linked_args[@]}" \
      --output-dir "$run_root/compactor"

    target/debug/agent-drift-analyzer \
      --input-dir "$run_root/compactor" \
      --output-dir "$run_root/analyzer"
  done
done
```

The committed adapted derivative was copied byte-for-byte into the ignored proof
area, checked with SHA-256, and analyzed separately:

```bash
case_id="da59436e63915185"
source_dir="crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/adapted-parent-visible-$case_id"
run_root="target/r7-5-real-corpus/adapted/$case_id"

rm -rf "$run_root"
mkdir -p "$run_root/compactor"
cp "$source_dir"/{manifest.json,rows.archival.jsonl,rows.compact.jsonl,dedupe-audit.jsonl} \
  "$run_root/compactor/"

target/debug/agent-drift-analyzer \
  --input-dir "$run_root/compactor" \
  --output-dir "$run_root/analyzer"

(cd "$source_dir" && shasum -a 256 manifest.json rows.archival.jsonl rows.compact.jsonl dedupe-audit.jsonl) \
  > "$run_root/source.sha256"
(cd "$run_root/compactor" && shasum -a 256 manifest.json rows.archival.jsonl rows.compact.jsonl dedupe-audit.jsonl) \
  > "$run_root/copied.sha256"
diff -u "$run_root/source.sha256" "$run_root/copied.sha256"
```

## Native Run Results

`Links` is the compactor manifest's link-state count. Strata are analyzer
checkpoint counts, not session counts.

| Parent | Mode | Files / sessions | Links | Compact rows | Checkpoints | Analyzer strata |
| --- | --- | ---: | --- | ---: | ---: | --- |
| `019e93f8-...` | default | 1 / 1 | `parent_only=2` | 169 | 3 | opaque parent 3 |
| `019e93f8-...` | linked | 3 / 3 | `verified=2` | 487 | 10 | linked parent 3; delegated child 7 |
| `019e9406-...` | default | 1 / 1 | `parent_only=2` | 114 | 2 | opaque parent 2 |
| `019e9406-...` | linked | 3 / 3 | `verified=2` | 332 | 8 | linked parent 2; delegated child 6 |
| `019eb907-...` | default | 1 / 1 | `parent_only=4` | 136 | 4 | opaque parent 4 |
| `019eb907-...` | linked | 5 / 5 | `verified=4` | 490 | 12 | linked parent 4; delegated child 8 |
| `019eb917-...` | default | 1 / 1 | `parent_only=6` | 179 | 3 | opaque parent 3 |
| `019eb917-...` | linked | 7 / 7 | `verified=6` | 784 | 17 | linked parent 3; delegated child 14 |
| `019eb970-...` | default | 1 / 1 | `parent_only=8` | 341 | 6 | opaque parent 6 |
| `019eb970-...` | linked | 9 / 9 | `verified=8` | 1,054 | 22 | linked parent 6; delegated child 16 |
| `da59436e63915185` | adapted | 1 / 1 | none | 871 | 28 | single agent 23; opaque parent 5 |

The 11 runs produced 115 checkpoint observations. Required R7 proof strata,
including explicit zeroes, were:

| Required stratum | Checkpoints |
| --- | ---: |
| `single_agent` | 23 |
| `delegating_parent_linked` | 18 |
| `delegating_parent_partial` | 0 |
| `delegating_parent_opaque` | 23 |
| `delegated_child` | 51 |
| `mixed_or_ambiguous` | 0 |

Zero real-corpus partial or ambiguous checkpoints is expected for this receipt:
all 22 claimed native children were present, unique, reciprocal, and depth 1.
Missing, one-sided, conflicting, multi-child-with-missing, and nested-residue
behavior remains covered by the committed R7-5.1 acceptance matrix rather than
being fabricated from complete real evidence.

## Manual Audit

### Topology, visibility, and confidence

- Every default parent checkpoint was `delegating_parent / opaque / low`; none
  was falsely promoted to linked visibility from parent-only evidence.
- Every linked parent checkpoint was `delegating_parent / linked / high`.
- Every checkpoint belonging to an included child was
  `delegated_child / linked / high` with its exact parent id.
- The adapted derivative remained `single_agent / none / high` for its first 23
  checkpoints, then conservatively became `delegating_parent / opaque / low`
  for the final five. It never invented a linked child id.

### Verified `019e93f8-...` -> `019e93fa-...` pair

The compactor manifest recorded the pair as one verified depth-1 link with two
parent-side provenance references and one reciprocal child-side provenance
reference. The child emitted four independent checkpoints, each
`delegated_child / linked / high` with parent
`019e93f8-a5e9-7490-ac1a-955b74c92ad0`:

1. stalled / planning convergence;
2. stalled / planning convergence;
3. regressing / troubleshooting frontier; and
4. advancing / troubleshooting frontier with clean verification.

All four child checkpoints kept every drift class cleared. The parent's three
progress signatures (status, dimension, confidence, and signal) and scorer
signatures (class, state, raw score, confidence, and flagged bit) matched between
default and linked analysis; only the delegation context widened.
The linked bundle also retained the root's second verified direct child, so this
pair proof exercised the real multi-child closure instead of an isolated
synthetic join.

### Progress and scorer ownership

- All 3,126 analyzer evidence references were structurally valid.
- Progress and scorer evidence produced **zero** cross-session ownership
  violations. Parent progress/scores referenced parent rows; child
  progress/scores referenced that child's rows. Cross-trajectory references
  appeared only in the typed delegation context, where they are required.
- For all five parents, default and linked runs had the same parent checkpoint
  count and exactly the same drift class, state, raw score, and flagged bit at
  every ordinal. Four parents also had exact progress equality.
- `019eb917-...` had one expected topology-driven progress refinement at parent
  ordinal 1: parent-only evidence reported stalled planning convergence at
  medium confidence, while verified linkage reported stalled
  parent-visible orchestration at low progress confidence. No score state, raw
  score, or flagged bit changed, and later parent ordinals matched.

### Fires and suppressions

These are checkpoint observations across all 11 runs; default and linked modes
deliberately observe the same parent trajectory twice.

| Drift class | Active | Recovered | Historical only | Cleared | Flagged |
| --- | ---: | ---: | ---: | ---: | ---: |
| `wrong_plan_branch` | 2 | 0 | 0 | 113 | 2 |
| `truth_grounding_gap` | 10 | 3 | 0 | 102 | 10 |
| `dead_end_thrash` | 4 | 4 | 17 | 90 | 4 |
| `semantic_goal_drift` | 0 | 0 | 0 | 115 | 0 |

There were zero active-but-unflagged observations. Manual attribution confirmed:

- the `019e9406-...` parent retained the same active
  `wrong_plan_branch` checkpoint in default and linked modes;
- the `019eb917-...` parent retained the same active then recovered
  `dead_end_thrash` sequence in both modes;
- linked-child `dead_end_thrash` and `truth_grounding_gap` fires stayed on the
  child trajectories that produced them and did not appear on their parents;
- the adapted derivative's single active `dead_end_thrash` remained on its own
  single-agent checkpoint; and
- historical/recovered states stayed unflagged rather than becoming new active
  claims.

### Fail-closed incomplete evidence and single-agent stability

- Default manifests preserved `parent_only` link states in counts
  `2 / 2 / 4 / 6 / 8`; analyzer output remained opaque and low-confidence.
  Parent-side spawn results alone never authorized child ids, child progress, or
  child drift.
- The ten-case R7-5.1 matrix separately exercised one verified child, multiple
  verified children, one verified plus one missing child, missing child file,
  child-only metadata, conflicting ids, nested residue, ordinary single-agent,
  parent wait while a child advances, and child stall while the parent stays
  clean. Its non-verified link assertions passed fail closed.
- The adapted derivative preserved 23 ordinary single-agent checkpoints before
  its late parent-visible boundary, and its committed progress contract passed.
  The dedicated ordinary single-agent matrix control also passed. No existing
  single-agent trajectory was promoted to a linked role.
- Adapted checkpoint 28 matched its committed contract: stalled
  `parent_visible_orchestration`, low confidence, required visibility-limiting
  signal present, forbidden strong-progress signals absent, and all drift
  classes unflagged (`dead_end_thrash` remained historical-only at score 20).

## Verification And Privacy

Focused proof at `afb10827d`:

```bash
cargo test -p agent-drift-analyzer --test delegated_acceptance -- --nocapture
# 2 passed; 0 failed

cargo test -p agent-drift-analyzer --test delegation_context -- --nocapture
# 8 passed; 0 failed

cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
# 4 passed; 0 failed; finished in 138.54s
```

The ignored proof receipt also ran a field-only aggregation over all 115
checkpoints and returned 3,126 valid evidence references, zero malformed
references, and zero progress/scorer ownership violations. The adapted source
and ignored copy had identical SHA-256 lists.

Privacy verification was run as:

```bash
python3 target/r7-5-real-corpus/privacy_check.py \
  docs/specs/r7/FINDINGS-r7-delegated-session-validation.md
# privacy_check: PASS

git check-ignore -q target/r7-5-real-corpus
git status --short
```

The tracked report contains no raw corpus rows, message text, user requests,
agent instructions, evidence-reason text, local private paths, credentials, or
secret material. Session ids and aggregate semantic fields are retained only
where required to identify and reproduce the named proof. R7-5.2 required no
production, fixture, script, canonical status, ledger, R7-6, or sentinel edits.

## Disposition

R7-5.2 acceptance is satisfied: the named real witnesses and adapted derivative
were re-run, the six required strata are explicit including zeroes, topology and
trajectory ownership were manually audited, incomplete evidence failed closed,
and no raw private input was committed. This finding is ready for fresh
independent review; it is not a self-approval or an R7-5 phase-transition claim.
