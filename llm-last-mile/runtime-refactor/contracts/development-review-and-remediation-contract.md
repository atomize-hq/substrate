**Kind:** contract
**Status:** canonical
**Canonical for:** development review and remediation contract

## Development review and remediation contract

This contract governs implementation, documentation, proof, remediation, and closeout work driven
by this control pack. It does not define a Substrate product-runtime review engine.

### Required workflow skills and selected outcome

Every packet begins by loading `using-agent-skills` and then the skills applicable to its phase.
Multi-file implementation or documentation uses `incremental-implementation`; every independent
review uses `code-review-and-quality`. Other skills remain conditional on the work rather than
becoming automatic acceptance requirements.

Before implementation, the packet freezes:

1. the selected integrated outcome and the exact completion claim;
2. the controlling contracts, acceptance criteria, and required proof gates;
3. the allowed files/symbols and explicit non-goals;
4. the subject-fingerprint method and review lenses; and
5. the review budget and stop conditions below.

A reviewer may discover a defect but may not create a new acceptance requirement. A property of
agent-created proof, controller, supervisor, dispatch, or reporting tooling blocks the selected
packet only when the controlling authority explicitly requires that property or the property is
demonstrably necessary for a named proof claim. Otherwise the concern is classified against its
actual effect on the selected integrated outcome and, when valid, retained as `P3` or `P4` in
[`06-review-finding-inventory.md`](../06-review-finding-inventory.md).

### Review priority

| Priority | Reviewer label | Required evidence and effect |
|---|---|---|
| `P1` | Critical | Demonstrated severe safety, security, data-loss, destructive-mutation, or authority-integrity failure. Blocks completion. |
| `P2` | Required | Demonstrated failure of the selected contract, acceptance criterion, required gate, authorized scope boundary, or completion claim. Blocks completion. |
| `P3` | Optional / Consider | Useful hardening or improvement without demonstrated failure of the selected integrated outcome. Non-blocking and inventoried when unfixed. |
| `P4` | Nit | Minor polish, naming, formatting, or consistency issue with no correctness effect. Non-blocking and inventoried when unfixed. |

`CLEAN` means no unresolved valid `P1` or `P2`; it may include `P3` or `P4` advisories. A findings
verdict contains at least one valid `P1` or `P2`. Reviewer wording does not set priority by itself:
the parent validates each finding against current authority and live truth, records any evidence-
based reclassification, and preserves the raw review unchanged. Uncertainty alone does not elevate
defense-in-depth or speculative robustness to `P2`; missing evidence is blocking only when that
evidence is required for the selected completion claim.

Priority follows the selected integrated outcome, not the most severe isolated component
observation. An independently observable wrapper or attestation weakness is `P2` only when it makes
the selected proof unable to support its claimed status. A concern that leaves the required proof
independently evaluable is normally `P3`, even when hardening the wrapper would be worthwhile.

### Bounded review cycles

The default automatic budget is:

1. one complete-subject discovery review or same-fingerprint review burst;
2. one consolidated remediation covering every validated `P1` and `P2` from that cycle;
3. one different-fresh, delta-focused closure review; and
4. at most two supplemental causal remediation/closure cycles for new `P1` or `P2` findings
   demonstrated to have been directly caused or unmasked by the immediately preceding remediation.

A review burst uses disjoint lenses over the same subject fingerprint and is consolidated before
one remediation pass. A closure review verifies the remediation, affected contracts/call paths,
invalidated proof, aggregate subject identity, and absence of remediation-caused regression; it
does not restart open-ended discovery. A new observation outside that boundary is `P3` unless the
parent demonstrates its `P1` or `P2` effect on the selected integrated outcome.

Every supplemental cycle remains within the frozen scope, authority, and risk ceiling, cites the
immediately preceding `P1`/`P2` IDs, and records evidence for the causal claim. An unrelated
blocker, material scope/risk expansion, or exhausted two-cycle allowance produces a bounded non-
completed stop for explicit authority. Budget exhaustion never waives a valid `P1` or `P2`, and a
`CLEAN` cycle is terminal: no further review or remediation cycle may be launched.

Reviewers are read-only and fresh after material remediation. Give them the exact authority,
subject/delta, gates, raw verification, unavailable proof, and non-goals. Do not provide
implementation reasoning, remediation discussion, prior reviewer conclusions, or a success-
asserting summary. The parent retains cycle/finding lineage separately from the reviewer's isolated
context.

### Machine-auditable cycle record

For every new runtime-refactor review sequence, the parent owns one JSON record shaped like
[`review-control/review-cycle-record.example.json`](../review-control/review-cycle-record.example.json).
Each consolidated cycle records its kind (`discovery`, `closure`, or `supplemental_causal`), stable
ID, exact subject fingerprint, review evidence refs, verdict, findings, and immediate causal
lineage. The record is process evidence only; it does not replace the packet's contracts, tests,
raw reviews, or proof artifacts.

After every returned review cycle, validate the updated record:

```bash
python llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py <record.json>
```

Before launching a closure or supplemental cycle, keep the record `in_progress` and require the
candidate next kind to pass:

```bash
python llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py \
  <record.json> --next-cycle closure
python llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py \
  <record.json> --next-cycle supplemental_causal \
  --causal-evidence-ref <evidence-ref>
```

The standard-library validator rejects invalid order, a cycle after `CLEAN`, inexact triggering
`P1`/`P2` IDs, unchanged post-remediation subject identity, missing supplemental causal evidence,
a third supplemental cycle, and false `complete`/`bounded_stop` status. It intentionally does not
collect evidence, hash live files, judge whether a causal claim is true, launch an agent, or mutate
a checkout. Those remain parent/reviewer responsibilities under the frozen packet authority; do
not add a bespoke supervisor to satisfy this contract.

### Mechanical changes and completion

Before the discovery fingerprint, run formatting, `git diff --check`, the packet allowlist/scope
check, and applicable focused verification so deterministic cleanup does not create a late review
round. A mechanical-only delta is limited to deterministically proved whitespace/formatting,
generated fingerprint or ledger bytes, or exact `P3`/`P4` inventory transcription. Record the diff
and deterministic checks without another reviewer; any semantic uncertainty makes the delta
material. A packet that explicitly requires exact reviewed bytes remains stricter and must perform
the mechanical work before review or follow its named re-review rule.

A packet completes only when its final material cycle is `CLEAN`, all required proof gates pass,
and every valid unfixed `P3`/`P4` is added to or deduplicated against `06`. Historical review
verdicts and packet-specific stricter gates remain immutable evidence, but no future packet inherits
a blanket reviewer-count or all-findings-block rule unless its authority states that requirement
explicitly.

**Source provenance:** extracted from [`04-contracts-and-gates.md#development-review-and-remediation-contract`](../04-contracts-and-gates.md#development-review-and-remediation-contract), baseline lines 16–139; only the two repository-relative Markdown targets for `06-review-finding-inventory.md` and `review-control/review-cycle-record.example.json` were rebased by one parent directory to preserve their original root-level targets after relocation under `contracts/`
**Baseline span SHA-256:** `0aecf3573e59f401729b999d04a0b923863ce811de8fec5f525d6ca8cddb2a0d`
