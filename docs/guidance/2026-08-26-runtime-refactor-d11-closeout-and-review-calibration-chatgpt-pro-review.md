# ChatGPT Pro advisory review: runtime-refactor D11 closeout rule and review-process calibration

- Date: 2026-08-26
- Bound baseline commit/tree: `d02168ada079b79c3a12709058f4db893bddabdb` / `fef8ee7849f446a898cb250792cb3dde0a900079`
- Candidate implementation subagent (`gpt-5.4`, Extra High): `/root/d11_batch2_closeout_review_calibration` (display name not exposed to this subagent runtime and therefore not asserted here)
- Closeout subagent (`gpt-5.4`, Extra High): `/root/d11_batch2_closeout` (display name not exposed to this subagent runtime and therefore not asserted here)
- Root/orchestrator review owner: `/root`
- Independent review chat: https://chatgpt.com/c/6a8f993d-ea90-83ea-842c-14deaf5fd41c
- Review mode: `initial-range`
- Review context: fresh ChatGPT conversation
- ChatGPT account surface: `Pro`
- Visible reasoning-effort control: `Extra High`
- Exact model label: not exposed by the current visible ChatGPT UI
- Candidate patch: `sha256:bdb220d10ff9187c62aadda68116f623825ea6bb24c721fade71b61febb8882a` (`22279 bytes`) at `/private/tmp/d11-closeout-review-calibration-candidate.patch`
- Review prompt: `sha256:d3dc95042d2a97103c22ba85158126502cb65324ce091a9e0d799a4adbd26439` (`32446 bytes`) at `/private/tmp/d11-closeout-review-calibration-chatgpt-pro-initial-review-prompt.txt`
- Review answer: `sha256:685bccb128046282f9dde499058e56be8afe9074cbeae9736ccb2e272c7b3e7c` (`2276 bytes`) at `/private/tmp/d11-closeout-review-calibration-chatgpt-pro-initial-review-answer.txt`
- Imported validation log: `sha256:74f8d5c8c003bbe4def5fd04ff8f6093a7b11b2f96665a725343068eeb6eb982` (`2888 bytes`) at `/private/tmp/d11-closeout-review-calibration-validation.log`
- Exact embedded source body (`Closeout rule`): baseline lines `193–201`, `347 bytes`, `sha256:f81142a2bf6d0cbd6b31a27011260da25108cc518b582dd1be81268f94635460`
- Exact embedded source body (`Review-process calibration`): baseline lines `332–353`, `1382 bytes`, `sha256:284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e`
- Review verdict: `APPROVED`
- Review finding summary: `No qualifying findings.`
- Remediation rounds: `0`
- Review adjudication posture: root agent owns review/adjudication; subagents do not self-approve
- Commit posture: the candidate remains uncommitted and unstaged during closeout; candidate plus this closeout are not yet committed
- Push posture: `not pushed`
- D12 posture: `blocked until D11 is exhaustive; separately unauthorized`
- Successor authority posture: no successor dispatch authority is granted

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Candidate scope and outcome

This independently reviewed D11 Batch 2 evidence family extracts the exact `## Closeout rule` and `## Review-process calibration` bodies into `llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md`, replaces only those two root bodies with truthful compatibility anchors, adds the single Batch 2 navigation row in `llm-last-mile/runtime-refactor/evidence/README.md`, adds the single truthful `Closeout rule + Review-process calibration` global-index row, and adds the matching two-row D11 extraction-ledger coverage. The batch preserves the noncontiguous source provenance, exact byte-preserved bodies, ordered bullets, literals, IDs, negative requirements, exceptions, review-governance reference-only boundary to the D3 development-review contract, and the no-controller/supervisor-remediation and no-proof-result-change statements while leaving Batch 1, prior D3–D10 owners/review artifacts, `index/current.md`, and `review-control/` unchanged. Batch 2 is complete locally and independently approved after initial-range review with zero remediation and is landed pending one atomic commit. Batch 1 remains committed complete at detached HEAD `d02168ada079b79c3a12709058f4db893bddabdb`. The exact next D11 unit is Batch 3 shared `Baseline behaviors` plus `Cross-gate smoke scenarios`; Batch 4 reading model/current-state/receipt-navigation remains queued; D11 is not complete. This documentation decomposition is not implementation, proof/evidence satisfaction, gate promotion, remediation, dispatch, or successor authority. D12 remains blocked until D11 is exhaustive and is separately unauthorized; no successor dispatch authority is granted.

## Prospective seven-path landing manifest

Prospective landing manifest for this independently reviewed batch:

- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
- `llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md`
- `llm-last-mile/runtime-refactor/evidence/README.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `docs/guidance/2026-08-26-runtime-refactor-d11-closeout-and-review-calibration-chatgpt-pro-review.md`

## Candidate path digests and imported review artifacts

Candidate path digests:

- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`: `sha256:6472f8c541ec90df039772870dcc588b6cb2d657ea57e746d7e676c703894eae`
- `llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md`: `sha256:a79982f11f66fc826f1f32904988826a3cf6ba508b7e32f5cd419c58179042b6`
- `llm-last-mile/runtime-refactor/evidence/README.md`: `sha256:e43e02a65b75383d3af10c6fa09dfd42a1af06af646bfafbe193f4daf4bbb0db`
- `llm-last-mile/runtime-refactor/index/README.md`: `sha256:ca9332773a0387c6ae1d3806a319f56a9dba8e9e11cb149df932191fbd5893ca`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`: `sha256:8823b2ec55a209c39c75656d16560eac2e9eddce53d4cc3e1b2857d416f4f7e1`

Imported review artifacts:

- Candidate patch: `sha256:bdb220d10ff9187c62aadda68116f623825ea6bb24c721fade71b61febb8882a` (`22279 bytes`)
- Review prompt: `sha256:d3dc95042d2a97103c22ba85158126502cb65324ce091a9e0d799a4adbd26439` (`32446 bytes`)
- Review answer: `sha256:685bccb128046282f9dde499058e56be8afe9074cbeae9736ccb2e272c7b3e7c` (`2276 bytes`)
- Imported validation log: `sha256:74f8d5c8c003bbe4def5fd04ff8f6093a7b11b2f96665a725343068eeb6eb982` (`2888 bytes`)
- Exact `Closeout rule` source body: `347 bytes`, `sha256:f81142a2bf6d0cbd6b31a27011260da25108cc518b582dd1be81268f94635460`
- Exact `Review-process calibration` source body: `1382 bytes`, `sha256:284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e`

## Exact source-body proof

- Baseline `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md` lines `193–201` are exactly `347` bytes with SHA-256 `f81142a2bf6d0cbd6b31a27011260da25108cc518b582dd1be81268f94635460`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:closeout-rule:start -->` and `<!-- exact-extracted-body:closeout-rule:end -->` in `llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md` are also exactly `347` bytes with SHA-256 `f81142a2bf6d0cbd6b31a27011260da25108cc518b582dd1be81268f94635460`.
- Baseline `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md` lines `332–353` are exactly `1382` bytes with SHA-256 `284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:review-process-calibration:start -->` and `<!-- exact-extracted-body:review-process-calibration:end -->` in `llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md` are also exactly `1382` bytes with SHA-256 `284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e`.
- Validation confirmed root bytes outside the two replaced spans remain byte-stable versus `HEAD`; the preserved root headings `## Closeout rule` and `## Review-process calibration` each remain exactly once as compatibility anchors, and the owner headings remain exactly once.
- `ROW_COUNT_OK` preserved exactly one Batch 2 `evidence/README.md` navigation row, exactly one `Closeout rule + Review-process calibration` global-index row, and exactly two D11 Batch 2 extraction-ledger rows.
- `BODY_LITERAL_BOUNDARY_OK` preserved the ordered bullets, literals, IDs, negative requirements, exceptions, review-governance reference-only boundary, and the no-controller/supervisor-remediation and no-proof-result-change statements without collapsing the two noncontiguous source bodies into one contiguous extraction.
- The two D11 extraction-ledger rows preserve separate noncontiguous provenance while sharing one atomic rollback unit: restore both exact source bodies in original locations/order, remove `evidence/closeout-and-review-calibration.md`, remove the sole Batch 2 row from `evidence/README.md`, remove the exact `Closeout rule + Review-process calibration` row from `index/README.md`, and remove both D11 Batch 2 ledger rows together. The review-process row retains the development-review contract relationship as reference-only and does not reopen D3.

## Validation evidence

- The supplied review answer states `No qualifying findings.` and ends with `VERDICT: APPROVED`.
- The imported validation log confirms the exact five-path candidate fence, no staged files, exact owner-body equality for both noncontiguous source spans, baseline-root reconstruction after replacing the compatibility stubs, unique preserved root/owner anchors, `ROW_COUNT_OK evidence_navigation=1 global_index=1 d11_ledger_batch2=2`, `OWNER_REVIEW_STABILITY_OK snapshot_paths=251`, `LINK_SCAN_SUMMARY checked=1916 excluded=4 failures=0`, and clean candidate-patch forward/reverse application.
- The candidate patch changes exactly the five imported candidate paths and no others.
- Candidate file hashes, candidate-patch SHA-256, prompt SHA-256, answer SHA-256, validation-log SHA-256, and both exact source-body SHA-256 digests were all reverified during this closeout.
- The rendered prompt and rendered answer below are whitespace-clean: any trailing literal spaces or tabs from the imported UTF-8 prompt or answer are rendered as `&#32;` or `&#09;` so this repository-local Markdown artifact stays clean under `git diff --check` while the adjacent base64 blocks preserve the exact bytes.
- The rendered prompt and rendered answer were both revalidated against the imported files, and the embedded base64 blocks decode exactly back to the imported UTF-8 bytes without silently correcting any captured whitespace, diff fences, or wording.
- Final changed-path fence including untracked files is exactly these seven paths:
  - `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
  - `llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md`
  - `llm-last-mile/runtime-refactor/evidence/README.md`
  - `llm-last-mile/runtime-refactor/index/README.md`
  - `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
  - `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
  - `docs/guidance/2026-08-26-runtime-refactor-d11-closeout-and-review-calibration-chatgpt-pro-review.md`
- `git diff --check` passed for the final closeout state.
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md` and `git diff --no-index --check /dev/null docs/guidance/2026-08-26-runtime-refactor-d11-closeout-and-review-calibration-chatgpt-pro-review.md` reported no whitespace diagnostics.
- Tracker truth validation passed for Batch 1 committed complete at detached HEAD `d02168ada079b79c3a12709058f4db893bddabdb`, shared Batch 2 approved and landed pending one atomic commit, the exact next D11 unit Batch 3 shared `Baseline behaviors` plus `Cross-gate smoke scenarios`, Batch 4 reading model/current-state/receipt-navigation still queued, D11 incomplete, D12 blocked until D11 is exhaustive and separately unauthorized, and no successor dispatch authority.
- Prior D3/D5–D10 owner/review-artifact stability passed via the exact seven-path fence.
- Runtime-refactor Markdown link/anchor resolution remains covered by the imported candidate validation (`checked=1916 excluded=4 failures=0`); additional tracker/review-record local link checks passed during this closeout for their two repository-local Markdown links, and plain-text external review URLs were not part of the local Markdown-link set.
- Complete combined candidate+closeout patch was generated at `/private/tmp/d11-closeout-review-calibration-closeout.patch` with a SHA-256 sidecar, and forward/reverse apply against disposable clean baseline exports both passed.
- No commit, stage, or push is performed during this closeout.

## Atomic rollback instructions

1. If the prospective seven-path batch has been committed, reverse that single batch commit atomically (for example, `git revert --no-commit <batch-commit>`) and verify that only the seven paths above are touched.
2. If the batch is still uncommitted, restore the tracked paths from `HEAD` and remove the two untracked files: `git restore --source=HEAD -- llm-last-mile/runtime-refactor/05-debug-regression-ledger.md llm-last-mile/runtime-refactor/evidence/README.md llm-last-mile/runtime-refactor/index/README.md llm-last-mile/runtime-refactor/migration/extraction-ledger.md docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md` then `rm -f llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md docs/guidance/2026-08-26-runtime-refactor-d11-closeout-and-review-calibration-chatgpt-pro-review.md`.
3. After either rollback path, confirm together that the exact `347`-byte `Closeout rule` body and exact `1382`-byte `Review-process calibration` body are restored to `05-debug-regression-ledger.md` in original order, the untracked owner and review record are removed, the sole Batch 2 `evidence/README.md` row is removed, the exact `Closeout rule + Review-process calibration` global-index row is removed, the two D11 Batch 2 extraction-ledger rows are removed, and the tracker summary/queue/closeout-row additions are reverted.
4. Leave Batch 1, prior D3–D10 owners and review artifacts, `index/current.md`, `review-control/`, remaining D11 batches, and every path outside this seven-path batch unchanged.
5. Documentation rollback must not be rewritten as implementation, proof/evidence satisfaction, gate promotion, remediation, dispatch, or successor authority. D11 remains incomplete, and D12 remains blocked until D11 is exhaustive and separately unauthorized.

## Final verdict

Shared `Closeout rule` + residual `Review-process calibration` is complete locally and **APPROVED** after initial-range review with zero remediation and is landed pending one atomic commit. Batch 1 remains committed complete at detached HEAD `d02168ada079b79c3a12709058f4db893bddabdb`. The exact next D11 unit is Batch 3 shared `Baseline behaviors` plus `Cross-gate smoke scenarios`; Batch 4 reading model/current-state/receipt-navigation remains queued; D11 is not complete. This documentation decomposition does not complete evidence, satisfy a gate, authorize implementation, gate promotion, remediation, dispatch, or successor work, and it does not replace review-control receipts. D12 remains blocked until D11 is exhaustive and is separately unauthorized. No successor dispatch authority is granted.

## Preserved review prompt

<details>
<summary>Initial-range review prompt (rendered copy)</summary>

<pre>
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: exact baseline commit d02168ada079b79c3a12709058f4db893bddabdb with tree fef8ee7849f446a898cb250792cb3dde0a900079, plus the complete content-addressed candidate patch below. Candidate patch SHA-256: bdb220d10ff9187c62aadda68116f623825ea6bb24c721fade71b61febb8882a.
Review target: the complete bounded D11 Batch 2 documentation-decomposition candidate. Review every changed line and every new-file byte in the patch; do not infer or request excluded repository content.

Manifest (complete changed-path fence):
1. llm-last-mile/runtime-refactor/05-debug-regression-ledger.md (modified root compatibility projection)
2. llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md (new compound canonical evidence owner)
3. llm-last-mile/runtime-refactor/evidence/README.md (modified non-authoritative navigation)
4. llm-last-mile/runtime-refactor/index/README.md (modified non-authoritative global index)
5. llm-last-mile/runtime-refactor/migration/extraction-ledger.md (modified provenance/rollback ledger)
No tracker or review record is part of this initial candidate; those are intentionally deferred until approval.

Supported inputs and behavior: repository-local UTF-8 Markdown using GitHub-style heading anchors, tables, inline code, relative links/fragments, and HTML comment boundary markers. Legacy root headings remain resolvable compatibility anchors. Boundary comments are non-rendered delimiters; the exact bytes strictly between each named start/end marker are the preservation subjects. Relative Markdown links are resolved from the containing file. Four predeclared external/opaque links elsewhere in the unchanged corpus are excluded from local resolution and are not candidate-caused.
Supported platforms and dialects: documentation-only, repository-local Markdown/GitHub-style anchor behavior on the current repository tree. No Rust/runtime/platform execution behavior, alternative Markdown dialect, case-insensitive filesystem guarantee, URL reachability, or D12 root-cutover behavior is supported or in scope.

In-scope invariants:
- The exact baseline source span at root lines 193-201 is independently preserved as 347 bytes with SHA-256 f81142a2bf6d0cbd6b31a27011260da25108cc518b582dd1be81268f94635460 between its matching owner markers.
- The exact baseline source span at root lines 332-353 is independently preserved as 1382 bytes with SHA-256 284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e between its matching owner markers. The spans are noncontiguous and must not be represented as a single contiguous extraction.
- Root headings `## Closeout rule` and `## Review-process calibration` remain unique legacy anchors with truthful compatibility pointers; all root bytes outside the two replaced spans stay stable.
- All ordered bullets, literals, issue IDs, negative requirements, exceptions, and semantic/authority boundaries remain exact, including RP3/RP4/RP5, P1/P2, CLEAN, raw REQUEST_CHANGES preservation, `06` debt separation, RR-RF-0001 through RR-RF-0003, and the no-controller/supervisor-remediation and no-proof-result-change statements.
- Exactly one canonical owner exists for this extracted compound family. The owner must not reopen D3 review governance or convert evidence into gate satisfaction, implementation, promotion, remediation, dispatch, or successor authority.
- Navigation/index additions are non-authoritative, unique, and resolve to the owner and preserved root anchors.
- Exactly two extraction-ledger rows truthfully record the two distinct source spans and complete atomic rollback: restore both exact bodies in original locations/order; remove the owner and exact navigation/index/two-ledger additions; preserve Batch 1, D3-D10 owners, index/current.md, review-control/, and all paths outside the five-file fence.
- The candidate changes exactly the five manifested paths and introduces no broken supported local link/fragment, whitespace error, or instability in prior D3/D5-D10 canonical owners/review artifacts.

Required material consequence: a blocking finding must demonstrate a reachable patch-caused loss or alteration of preserved evidence semantics/bytes/order; a broken supported legacy or owner navigation path; false provenance or incomplete/non-atomic rollback; duplicate or inflated canonical authority; unauthorized scope/path mutation; or another defect capable of making maintainers rely on materially wrong D11 state. Pure wording/style preference, harmless metadata phrasing, or unsupported Markdown behavior is not material.
Blocking threshold: only P1/P2-equivalent findings that are reachable through the supported behavior, violate an in-scope invariant, have the required material consequence, and are caused or materially worsened by this patch. For every blocker, identify exact file/line or patch hunk, reachability, violated invariant, material consequence, and patch causality. Do not block on pre-existing or excluded behavior.
Accepted prior findings: none; this is the first independent review of this candidate.
Deferred or out-of-scope concerns: global formatting/normalization; D3 review-governance changes; edits or re-extraction of D5/D6 packet-local evidence; D9/D10 owners; review-control contents; tracker/closeout record; D12 cutover; runtime, Rust, scripts, platform behavior, implementation, evidence satisfaction, promotion, dispatch, successor work; roadmap or reviewed ZIP changes; external validation of the four unchanged opaque links; unrelated or pre-existing repository defects.

Validation evidence (locally produced and independently spot-checked before this review):
- detached baseline HEAD/tree matched; clean before edits
- exact five-path dirty fence; nothing staged
- git diff --check passed
- git diff --no-index --check /dev/null for the new owner passed
- both pre-edit source spans matched frozen artifacts; both owner bodies match their separate artifacts byte-for-byte and by SHA-256/byte count
- replacing the two root compatibility stubs with the frozen bodies reconstructs baseline root exactly
- required literals/boundaries and unique legacy/owner anchors passed
- exactly one evidence row, one global index row, and two D11 Batch 2 ledger rows added
- 251 prior D3/D5-D10 owner/review-artifact paths were hash-stable
- Markdown scan checked 1916 local links/fragments, excluded only four declared opaque links, and found zero failures
- complete patch forward- and reverse-applies against a clean baseline export

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return findings first by severity. For every finding, include the exact file/hunk, supported-input reachability, violated invariant, material consequence, and patch causality. State explicitly when there are no qualifying findings. End with exactly one verdict line: `VERDICT: APPROVED` if there are no qualifying blocking findings, otherwise `VERDICT: REQUEST_CHANGES`.

Complete candidate patch (SHA-256 bdb220d10ff9187c62aadda68116f623825ea6bb24c721fade71b61febb8882a):
```diff
diff --git a/llm-last-mile/runtime-refactor/05-debug-regression-ledger.md b/llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
index 71bfc438e..13012d850 100644
--- a/llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
+++ b/llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
@@ -192,12 +192,7 @@ Covers: `RG-CONFIG-02`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-UAA-02`, `RG-UAA-03`
&#32;
 ## Closeout rule
&#32;
-An implementation PR may mark a ledger row resolved only when:
-
-1. its owning crosswalk seam has the correct owner and call path for that behavior;
-2. the named permanent gate passes on the real path;
-3. adjacent resolved baselines remain green; and
-4. the evidence distinguishes durable success from transport/process success.
+Compatibility anchor only; canonical content: [`evidence/closeout-and-review-calibration.md#closeout-rule`](evidence/closeout-and-review-calibration.md#closeout-rule).
&#32;
 ### R2-2 historical failed integration closeout and remaining-seam correction
&#32;
@@ -331,26 +326,8 @@ Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evi
 Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#rp3rp4rp5-closeout-ledger`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#rp3rp4rp5-closeout-ledger).
 ## Review-process calibration
&#32;
-RP4 exposed a process failure without exposing a product or test regression: a blanket requirement
-for four clean reviews allowed findings about agent-created cache-only orchestration to expand the
-product-proof acceptance surface. Repeated fix/review attempts then improved bespoke evidence
-tooling rather than the selected Substrate outcome. Human disposition correctly preserved the raw
-`REQUEST_CHANGES` review while accepting the independently evaluable product proof.
-
-The prospective correction is owned by the development-review contract in `04`:
-
-- `P1`/`P2` block only on demonstrated impact to the selected contract, gate, scope, or completion
-  claim;
-- one discovery cycle, one consolidated remediation, one closure cycle, and at most two directly
-  causal supplemental cycles bound automatic work;
-- `CLEAN` is terminal, mechanical-only deltas do not create review cycles, and unrelated or expanded
-  blockers stop for authority rather than widening scope; and
-- valid non-blocking review/process debt is retained in `06`, separate from this product regression
-  ledger.
-
-The three RP4 persistence findings are registered as `RR-RF-0001` through `RR-RF-0003`. Their raw
-review files, hashes, and original verdict remain unchanged. This calibration authorizes no
-controller/supervisor remediation and changes no RP3/RP4/RP5 proof result.
+Compatibility anchor only; canonical content: [`evidence/closeout-and-review-calibration.md#review-process-calibration`](evidence/closeout-and-review-calibration.md#review-process-calibration).
+
&#32;
 ## A1.1d-5R2-4 terminal evidence ledger
&#32;
diff --git a/llm-last-mile/runtime-refactor/evidence/README.md b/llm-last-mile/runtime-refactor/evidence/README.md
index aaab355bc..30bc1c387 100644
--- a/llm-last-mile/runtime-refactor/evidence/README.md
+++ b/llm-last-mile/runtime-refactor/evidence/README.md
@@ -17,3 +17,4 @@
 | Evidence component | Canonical owner | Current extracted scope |
 |---|---|---|
 | Batch 1 — canonical issue ledger | [`canonical-issue-ledger.md#canonical-issue-ledger`](canonical-issue-ledger.md#canonical-issue-ledger) | Full extracted `## Canonical issue ledger` table plus the preserved `### A1.2a-WB gate assignment` body; exact issue IDs, chronology, limitations, obligations, literals, hashes, commands, and negative requirements remain unchanged. |
+| Batch 2 — closeout rule + review-process calibration | [`closeout-and-review-calibration.md`](closeout-and-review-calibration.md) | Exact extracted [`## Closeout rule`](closeout-and-review-calibration.md#closeout-rule) and [`## Review-process calibration`](closeout-and-review-calibration.md#review-process-calibration) bodies with separate noncontiguous source provenance; ordered bullets, literals, IDs, negative requirements, exceptions, review-governance reference-only boundaries, and the no-remediation/no-proof-result-change statement remain unchanged. |
diff --git a/llm-last-mile/runtime-refactor/index/README.md b/llm-last-mile/runtime-refactor/index/README.md
index 982f1c5ff..7754953d1 100644
--- a/llm-last-mile/runtime-refactor/index/README.md
+++ b/llm-last-mile/runtime-refactor/index/README.md
@@ -33,6 +33,7 @@
 | `shared-seam-crosswalk` | seam index | [`seams/README.md`](../seams/README.md) | `canonical shared seam-crosswalk rules plus extracted seam-family navigation for host/session, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation` | Supersedes only the extracted root reading-rule/classification spans and the extracted host/session authority, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation family rows; A0 and the existing D6 `HostSessionAuthority`, `WorldWorkerMessagingProtocol`, and `ObligationLedger` compatibility rows remain at their current owners. | [`02`](../02-seam-crosswalk.md#reading-rule), [`02`](../02-seam-crosswalk.md#a-host-authority-and-ingress), [`02`](../02-seam-crosswalk.md#b-dispatch-policy-receipts-and-retained-runtime), [`02`](../02-seam-crosswalk.md#c-obligations-and-host-re-engagement), [`02`](../02-seam-crosswalk.md#d-uaa-realization-projection-and-side-effect-mediation), [`02`](../02-seam-crosswalk.md#classification-consequences) |
 | `shared-slice-map` | slice index | [`slices/README.md`](../slices/README.md) | `canonical shared sequencing/dependency and slice-closeout owner plus non-authoritative D9 Track A–E navigation and A1/A1.4 projection index` | Supersedes only root-canonical ownership of the extracted shared sequencing/closeout spans plus the extracted Track A–E and A1/A1.4 projection spans; controlling schedule authority remains with decisions, packets, gates, and `current.md`. | [`03`](../03-phase-slice-map.md#sequencing-rules), [`03`](../03-phase-slice-map.md#track-a--authority-and-surface-neutrality), [`03`](../03-phase-slice-map.md#a1-bounded-packet-decomposition), [`03`](../03-phase-slice-map.md#track-b--world-dispatch-receipts-supervision-and-cancel), [`03`](../03-phase-slice-map.md#track-c--obligations-inbox-auto-attach-and-router-attach), [`03`](../03-phase-slice-map.md#track-d--uaa-execution-envelope-and-side-effect-mediation), [`03`](../03-phase-slice-map.md#track-e--dispatch-scoped-policy-narrowing-and-config-projection), [`03`](../03-phase-slice-map.md#slice-closeout-minimum) |
 | Canonical issue ledger | evidence/regression | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) | `canonical extracted owner for the full issue/regression-obligation ledger table plus the preserved A1.2a-WB gate-assignment body` | Supersedes only root-canonical ownership of the extracted `## Canonical issue ledger` body; the legacy `## Canonical issue ledger` and `### A1.2a-WB gate assignment` headings remain compatibility anchors until D12. | [`05`](../05-debug-regression-ledger.md#canonical-issue-ledger), [`05`](../05-debug-regression-ledger.md#a12a-wb-gate-assignment) |
+| Closeout rule + Review-process calibration | evidence/regression | [`evidence/closeout-and-review-calibration.md`](../evidence/closeout-and-review-calibration.md) | `canonical extracted compound owner for the exact noncontiguous Closeout rule and Review-process calibration evidence/regression bodies` | Supersedes only root-canonical ownership of the extracted `## Closeout rule` and `## Review-process calibration` bodies; the root headings remain compatibility anchors, and the development-review contract remains reference-only without reopening D3. | [`05`](../05-debug-regression-ledger.md#closeout-rule), [`05`](../05-debug-regression-ledger.md#review-process-calibration) |
 | `A1.1d-5R2-4` | packet index | [`a1.1d-5r2-4/README.md`](../a1.1d-5r2-4/README.md) | `closed only as the bounded R2 propagation join` | Assembles the five extracted canonical R2-4 component owners; existing `review-control/` records stay at their current paths. | [`01`](../01-target-architecture.md#r2-4-bounded-closeout-architecture-disposition), [`02`](../02-seam-crosswalk.md#r2-4-terminal-crosswalk-disposition), [`03`](../03-phase-slice-map.md#a11d-5r2-4--r2-integration-and-closeout), [`04`](../04-contracts-and-gates.md#a11d-5r2-4-terminal-gate-disposition), [`05`](../05-debug-regression-ledger.md#a11d-5r2-4-terminal-evidence-ledger) |
 | `A1.1d-5R3-family` | packet index | [`a1.1d-5r3/README.md`](../a1.1d-5r3/README.md) | `archived for active scheduling; non-authoritative navigation for the extracted R3 implementation-family owners` | Assembles the archived planning/current-state projections, lifecycle architecture, crosswalk, implementation index, contracts, proof ledger, and their status appendices without dispatching R3, native evidence, or recovery implementation. | [`00`](../00-README.md#a11d-5r3-plan-authoritative-planning-status-archived-for-active-scheduling), [`01`](../01-target-architecture.md#a11d-5r3-lifecycle-ownership-architecture), [`02`](../02-seam-crosswalk.md#a11d-5r3-canonical-ownership-and-source-closure-crosswalk), [`03`](../03-phase-slice-map.md#a11d-5r3-authoritative-implementation-index), [`04`](../04-contracts-and-gates.md#a11d-5r3-lifecycle-contracts-and-gates), [`05`](../05-debug-regression-ledger.md#a11d-5r3-planned-proof-and-regression-ledger-archived-for-active-scheduling) |
 | `r3-mac-evidence-recovery-family` | planning index | [`r3-mac-evidence-recovery/README.md`](../r3-mac-evidence-recovery/README.md) | `planning-only authority preserved; non-authoritative navigation for extracted recovery status and correction projections` | Assembles the existing `PLAN.md`/`SPEC.md`/`TASKS.md` packet together with the extracted 00/02/03/04/05 recovery status/correction projections without authorizing recovery implementation, native evidence, or MAC closeout. | [`00`](../00-README.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06), [`02`](../02-seam-crosswalk.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06), [`03`](../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-plan-packet-status-2026-08-07), [`04`](../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-plan-recovery-decision-2026-08-07), [`05`](../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-plan-regression-status-2026-08-07) |
diff --git a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
index 316e84c11..b9174cd1a 100644
--- a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
+++ b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
@@ -197,3 +197,5 @@ This ledger records content-preserving authority transfers while retaining requi
 | D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 13. Dispatch narrowing monotonicity rules | [`13-dispatch-narrowing-monotonicity-rules`](../04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules) | lines 181–214 (1930 bytes) | `b12ef2b8c8bcffad5139e196df49ba2f59094ddf024f59d2d2a645eb9be2210a` | [`../gates/dispatch-narrowing-monotonicity.md`](../gates/dispatch-narrowing-monotonicity.md) | gate | canonical destination; source compatibility anchor | none | restore the exact 1179-byte source span at `04-contracts-and-gates.md#10-cancel-outcome-categories`, the exact 1675-byte source span at `04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules`, the exact 1930-byte source span at `04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules`, and the exact 1106-byte source span at `04-contracts-and-gates.md#14-contract-promotion-gates`; remove `contracts/cancel-outcome-categories.md`, `gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`, `gates/dispatch-narrowing-monotonicity.md`, and `gates/contract-promotion.md`; remove the exact `Cancel outcome categories`, `Final-receipt immutable PolicySnapshotV3 acceptance rules`, `Dispatch narrowing monotonicity rules`, and `Contract promotion gates` rows from `index/README.md`; remove all four D10 ledger entries for this batch; leave `LaunchTimeSecretHandoffV1`, `Supervisor idempotency and restart rules`, `Differential-baseline transition gate`, `Remaining R2-2 same-process carrier closure`, all D3/D5–D10 owners, `index/current.md`, `review-control/`, and every path outside the seven-path fence unchanged |
 | D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 14. Contract promotion gates | [`14-contract-promotion-gates`](../04-contracts-and-gates.md#14-contract-promotion-gates) | lines 215–231 (1106 bytes) | `0fa77777b7f8ae85bc4e06082454b243677d89d070ff4c510bd9e148b61b2d43` | [`../gates/contract-promotion.md`](../gates/contract-promotion.md) | gate | canonical destination; source compatibility anchor | none | restore the exact 1179-byte source span at `04-contracts-and-gates.md#10-cancel-outcome-categories`, the exact 1675-byte source span at `04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules`, the exact 1930-byte source span at `04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules`, and the exact 1106-byte source span at `04-contracts-and-gates.md#14-contract-promotion-gates`; remove `contracts/cancel-outcome-categories.md`, `gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`, `gates/dispatch-narrowing-monotonicity.md`, and `gates/contract-promotion.md`; remove the exact `Cancel outcome categories`, `Final-receipt immutable PolicySnapshotV3 acceptance rules`, `Dispatch narrowing monotonicity rules`, and `Contract promotion gates` rows from `index/README.md`; remove all four D10 ledger entries for this batch; leave `LaunchTimeSecretHandoffV1`, `Supervisor idempotency and restart rules`, `Differential-baseline transition gate`, `Remaining R2-2 same-process carrier closure`, all D3/D5–D10 owners, `index/current.md`, `review-control/`, and every path outside the seven-path fence unchanged |
 | D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Canonical issue ledger + A1.2a-WB gate assignment | [`canonical-issue-ledger`](../05-debug-regression-ledger.md#canonical-issue-ledger); [`a12a-wb-gate-assignment`](../05-debug-regression-ledger.md#a12a-wb-gate-assignment) | lines 16–97 (41492 bytes) | `191a62ea0c77064941947280d87c539163ab69b772a8018f6b2f4e3d3e5aae3e` | [`../evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) | evidence/regression | canonical destination; source compatibility anchors for both preserved root headings | none; exact 41492-byte source body preserved between owner boundary markers | restore the exact 41492-byte source body at `05-debug-regression-ledger.md#canonical-issue-ledger` and `#a12a-wb-gate-assignment` in original order; remove `evidence/canonical-issue-ledger.md`; remove the sole Batch-1 row from `evidence/README.md` and delete `evidence/README.md` if no navigation rows remain; remove the exact `Canonical issue ledger` row from `index/README.md`; remove this D11 ledger entry; leave D3–D10 owners, `index/current.md`, and `review-control/` unchanged |
+| D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Closeout rule | [`closeout-rule`](../05-debug-regression-ledger.md#closeout-rule) | lines 193–201 (347 bytes) | `f81142a2bf6d0cbd6b31a27011260da25108cc518b582dd1be81268f94635460` | [`../evidence/closeout-and-review-calibration.md`](../evidence/closeout-and-review-calibration.md) | evidence/regression | canonical compound destination; source compatibility anchor; separate noncontiguous provenance preserved | none; exact 347-byte source body preserved between the `closeout-rule` owner boundary markers in the compound evidence owner | restore the exact 347-byte source body at `05-debug-regression-ledger.md#closeout-rule` and the exact 1382-byte source body at `05-debug-regression-ledger.md#review-process-calibration` in original order; remove `evidence/closeout-and-review-calibration.md`; remove the sole Batch-2 row from `evidence/README.md`; remove the exact `Closeout rule + Review-process calibration` row from `index/README.md`; remove both D11 ledger entries for this batch; leave Batch 1 plus D3–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
+| D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Review-process calibration | [`review-process-calibration`](../05-debug-regression-ledger.md#review-process-calibration) | lines 332–353 (1382 bytes) | `284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e` | [`../evidence/closeout-and-review-calibration.md`](../evidence/closeout-and-review-calibration.md) | evidence/regression | canonical compound destination; source compatibility anchor; separate noncontiguous provenance preserved | none; exact 1382-byte source body preserved between the `review-process-calibration` owner boundary markers in the compound evidence owner; the development-review contract relationship remains reference-only and does not reopen D3 | restore the exact 347-byte source body at `05-debug-regression-ledger.md#closeout-rule` and the exact 1382-byte source body at `05-debug-regression-ledger.md#review-process-calibration` in original order; remove `evidence/closeout-and-review-calibration.md`; remove the sole Batch-2 row from `evidence/README.md`; remove the exact `Closeout rule + Review-process calibration` row from `index/README.md`; remove both D11 ledger entries for this batch; leave Batch 1 plus D3–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
diff --git a/llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md b/llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md
new file mode 100644
index 000000000..1974b88fe
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md
@@ -0,0 +1,51 @@
+**Kind:** evidence/regression
+**Status:** canonical
+**Canonical for:** exact extracted `Closeout rule` and `Review-process calibration` bodies
+**Authority scope:** documentation decomposition only; exact extracted evidence/regression source bodies only
+**Source provenance:** extracted byte-for-byte from [`../05-debug-regression-ledger.md#closeout-rule`](../05-debug-regression-ledger.md#closeout-rule), baseline lines 193–201 inclusive (`347` bytes), and [`../05-debug-regression-ledger.md#review-process-calibration`](../05-debug-regression-ledger.md#review-process-calibration), baseline lines 332–353 inclusive (`1382` bytes); each exact source body is preserved independently between its matching boundary markers below; the bodies are noncontiguous in the source file and are not represented as one contiguous extraction
+**Closeout rule source SHA-256:** `f81142a2bf6d0cbd6b31a27011260da25108cc518b582dd1be81268f94635460`
+**Review-process calibration source SHA-256:** `284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e`
+**Supersedes:** canonical ownership of the extracted source bodies; the source headings remain compatibility anchors
+**Superseded by:** none
+**Relationship to D3 governance:** reference-only to [`../contracts/development-review-and-remediation-contract.md`](../contracts/development-review-and-remediation-contract.md); this owner does not reopen D3 governance
+**Authority boundary:** this evidence/regression owner does not satisfy a gate, change a proof result, authorize implementation, promotion, remediation, or dispatch
+**Projection consumers:** [`README.md`](README.md), [`../index/README.md`](../index/README.md)
+
+# Closeout and review calibration
+
+&lt;!-- exact-extracted-body:closeout-rule:start --&gt;
+## Closeout rule
+
+An implementation PR may mark a ledger row resolved only when:
+
+1. its owning crosswalk seam has the correct owner and call path for that behavior;
+2. the named permanent gate passes on the real path;
+3. adjacent resolved baselines remain green; and
+4. the evidence distinguishes durable success from transport/process success.
+
+&lt;!-- exact-extracted-body:closeout-rule:end --&gt;
+
+&lt;!-- exact-extracted-body:review-process-calibration:start --&gt;
+## Review-process calibration
+
+RP4 exposed a process failure without exposing a product or test regression: a blanket requirement
+for four clean reviews allowed findings about agent-created cache-only orchestration to expand the
+product-proof acceptance surface. Repeated fix/review attempts then improved bespoke evidence
+tooling rather than the selected Substrate outcome. Human disposition correctly preserved the raw
+`REQUEST_CHANGES` review while accepting the independently evaluable product proof.
+
+The prospective correction is owned by the development-review contract in `04`:
+
+- `P1`/`P2` block only on demonstrated impact to the selected contract, gate, scope, or completion
+  claim;
+- one discovery cycle, one consolidated remediation, one closure cycle, and at most two directly
+  causal supplemental cycles bound automatic work;
+- `CLEAN` is terminal, mechanical-only deltas do not create review cycles, and unrelated or expanded
+  blockers stop for authority rather than widening scope; and
+- valid non-blocking review/process debt is retained in `06`, separate from this product regression
+  ledger.
+
+The three RP4 persistence findings are registered as `RR-RF-0001` through `RR-RF-0003`. Their raw
+review files, hashes, and original verdict remain unchanged. This calibration authorizes no
+controller/supervisor remediation and changes no RP3/RP4/RP5 proof result.
+&lt;!-- exact-extracted-body:review-process-calibration:end --&gt;
```

Validation log (SHA-256 74f8d5c8c003bbe4def5fd04ff8f6093a7b11b2f96665a725343068eeb6eb982):
```text
PRECHECK_HEAD d02168ada079b79c3a12709058f4db893bddabdb
PRECHECK_TREE fef8ee7849f446a898cb250792cb3dde0a900079
PRECHECK_BRANCH (detached)
STATUS_LINES_START
 M llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
 M llm-last-mile/runtime-refactor/evidence/README.md
 M llm-last-mile/runtime-refactor/index/README.md
 M llm-last-mile/runtime-refactor/migration/extraction-ledger.md
?? llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md
STATUS_LINES_END
PATH_FENCE_OK 5 changed paths exactly match allowed fence; no staged files
GIT_DIFF_CHECK_OK
NEW_FILE_DIFF_CHECK_OK llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md
PRE_EDIT_SPAN_MATCH_OK closeout_rule bytes=347 sha256=f81142a2bf6d0cbd6b31a27011260da25108cc518b582dd1be81268f94635460 review_process_calibration bytes=1382 sha256=284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e
OWNER_BODY_EQUAL_OK closeout_rule bytes=347 sha256=f81142a2bf6d0cbd6b31a27011260da25108cc518b582dd1be81268f94635460
OWNER_BODY_EQUAL_OK review_process_calibration bytes=1382 sha256=284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e
ROOT_NONSPAN_STABILITY_OK replacing the two compatibility stubs with the frozen bodies reconstructs HEAD exactly
ROOT_STUB_AND_HEADING_OK legacy root headings preserved once each with owner-anchor pointers; owner headings preserved once each
BODY_LITERAL_BOUNDARY_OK ordered bullets, literals, IDs, negative requirements, exceptions, no-remediation/proof-result boundary, and separate noncontiguous provenance preserved
ROW_COUNT_OK evidence_navigation=1 global_index=1 d11_ledger_batch2=2
OWNER_REVIEW_STABILITY_OK snapshot_paths=251
LINK_SCAN_SUMMARY checked=1916 excluded=4 failures=0
PATCH_WRITTEN /private/tmp/d11-closeout-review-calibration-candidate.patch sha256=bdb220d10ff9187c62aadda68116f623825ea6bb24c721fade71b61febb8882a
PATCH_APPLY_OK forward_and_reverse_apply_match_clean_baseline
FILE_SHA llm-last-mile/runtime-refactor/05-debug-regression-ledger.md 6472f8c541ec90df039772870dcc588b6cb2d657ea57e746d7e676c703894eae
FILE_SHA llm-last-mile/runtime-refactor/evidence/README.md e43e02a65b75383d3af10c6fa09dfd42a1af06af646bfafbe193f4daf4bbb0db
FILE_SHA llm-last-mile/runtime-refactor/evidence/closeout-and-review-calibration.md a79982f11f66fc826f1f32904988826a3cf6ba508b7e32f5cd419c58179042b6
FILE_SHA llm-last-mile/runtime-refactor/index/README.md ca9332773a0387c6ae1d3806a319f56a9dba8e9e11cb149df932191fbd5893ca
FILE_SHA llm-last-mile/runtime-refactor/migration/extraction-ledger.md 8823b2ec55a209c39c75656d16560eac2e9eddce53d4cc3e1b2857d416f4f7e1
FILE_SHA /private/tmp/d11-closeout-review-calibration-candidate.patch bdb220d10ff9187c62aadda68116f623825ea6bb24c721fade71b61febb8882a
FILE_SHA /private/tmp/d11-closeout-review-calibration-candidate.patch.sha256 4c9a9aa885214a587c68ec2ced781c14230fb4f827761bbbcd77abc67dd29892
```
</pre>

</details>

<details>
<summary>Initial-range review prompt exact bytes (base64 UTF-8)</summary>

```text
UmV2aWV3IG1vZGU6IGluaXRpYWwtcmFuZ2UKUmV2aWV3IGNvbnRleHQ6IGluZGVwZW5kZW50IGZy
ZXNoIGNvbnZlcnNhdGlvbgpSZXZpZXcgYm91bmRhcnk6IGV4YWN0IGJhc2VsaW5lIGNvbW1pdCBk
MDIxNjhhZGEwNzliNzljM2ExMjcwOTA1OGY0ZGI4OTNiZGRhYmRiIHdpdGggdHJlZSBmZWY4ZWU3
ODQ5ZjQ0NmE4OThjYjI1MDc5MmNiM2RkZTBhOTAwMDc5LCBwbHVzIHRoZSBjb21wbGV0ZSBjb250
ZW50LWFkZHJlc3NlZCBjYW5kaWRhdGUgcGF0Y2ggYmVsb3cuIENhbmRpZGF0ZSBwYXRjaCBTSEEt
MjU2OiBiZGIyMjBkMTBmZjkxODdjNjJhYWRkYTY4MTE2ZjYyMzgyNWVhNmJiMjRjNzIxZmFkZTcx
YjYxZmViYjg4ODJhLgpSZXZpZXcgdGFyZ2V0OiB0aGUgY29tcGxldGUgYm91bmRlZCBEMTEgQmF0
Y2ggMiBkb2N1bWVudGF0aW9uLWRlY29tcG9zaXRpb24gY2FuZGlkYXRlLiBSZXZpZXcgZXZlcnkg
Y2hhbmdlZCBsaW5lIGFuZCBldmVyeSBuZXctZmlsZSBieXRlIGluIHRoZSBwYXRjaDsgZG8gbm90
IGluZmVyIG9yIHJlcXVlc3QgZXhjbHVkZWQgcmVwb3NpdG9yeSBjb250ZW50LgoKTWFuaWZlc3Qg
KGNvbXBsZXRlIGNoYW5nZWQtcGF0aCBmZW5jZSk6CjEuIGxsbS1sYXN0LW1pbGUvcnVudGltZS1y
ZWZhY3Rvci8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCAobW9kaWZpZWQgcm9vdCBjb21w
YXRpYmlsaXR5IHByb2plY3Rpb24pCjIuIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9l
dmlkZW5jZS9jbG9zZW91dC1hbmQtcmV2aWV3LWNhbGlicmF0aW9uLm1kIChuZXcgY29tcG91bmQg
Y2Fub25pY2FsIGV2aWRlbmNlIG93bmVyKQozLiBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0
b3IvZXZpZGVuY2UvUkVBRE1FLm1kIChtb2RpZmllZCBub24tYXV0aG9yaXRhdGl2ZSBuYXZpZ2F0
aW9uKQo0LiBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvaW5kZXgvUkVBRE1FLm1kICht
b2RpZmllZCBub24tYXV0aG9yaXRhdGl2ZSBnbG9iYWwgaW5kZXgpCjUuIGxsbS1sYXN0LW1pbGUv
cnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQgKG1vZGlmaWVk
IHByb3ZlbmFuY2Uvcm9sbGJhY2sgbGVkZ2VyKQpObyB0cmFja2VyIG9yIHJldmlldyByZWNvcmQg
aXMgcGFydCBvZiB0aGlzIGluaXRpYWwgY2FuZGlkYXRlOyB0aG9zZSBhcmUgaW50ZW50aW9uYWxs
eSBkZWZlcnJlZCB1bnRpbCBhcHByb3ZhbC4KClN1cHBvcnRlZCBpbnB1dHMgYW5kIGJlaGF2aW9y
OiByZXBvc2l0b3J5LWxvY2FsIFVURi04IE1hcmtkb3duIHVzaW5nIEdpdEh1Yi1zdHlsZSBoZWFk
aW5nIGFuY2hvcnMsIHRhYmxlcywgaW5saW5lIGNvZGUsIHJlbGF0aXZlIGxpbmtzL2ZyYWdtZW50
cywgYW5kIEhUTUwgY29tbWVudCBib3VuZGFyeSBtYXJrZXJzLiBMZWdhY3kgcm9vdCBoZWFkaW5n
cyByZW1haW4gcmVzb2x2YWJsZSBjb21wYXRpYmlsaXR5IGFuY2hvcnMuIEJvdW5kYXJ5IGNvbW1l
bnRzIGFyZSBub24tcmVuZGVyZWQgZGVsaW1pdGVyczsgdGhlIGV4YWN0IGJ5dGVzIHN0cmljdGx5
IGJldHdlZW4gZWFjaCBuYW1lZCBzdGFydC9lbmQgbWFya2VyIGFyZSB0aGUgcHJlc2VydmF0aW9u
IHN1YmplY3RzLiBSZWxhdGl2ZSBNYXJrZG93biBsaW5rcyBhcmUgcmVzb2x2ZWQgZnJvbSB0aGUg
Y29udGFpbmluZyBmaWxlLiBGb3VyIHByZWRlY2xhcmVkIGV4dGVybmFsL29wYXF1ZSBsaW5rcyBl
bHNld2hlcmUgaW4gdGhlIHVuY2hhbmdlZCBjb3JwdXMgYXJlIGV4Y2x1ZGVkIGZyb20gbG9jYWwg
cmVzb2x1dGlvbiBhbmQgYXJlIG5vdCBjYW5kaWRhdGUtY2F1c2VkLgpTdXBwb3J0ZWQgcGxhdGZv
cm1zIGFuZCBkaWFsZWN0czogZG9jdW1lbnRhdGlvbi1vbmx5LCByZXBvc2l0b3J5LWxvY2FsIE1h
cmtkb3duL0dpdEh1Yi1zdHlsZSBhbmNob3IgYmVoYXZpb3Igb24gdGhlIGN1cnJlbnQgcmVwb3Np
dG9yeSB0cmVlLiBObyBSdXN0L3J1bnRpbWUvcGxhdGZvcm0gZXhlY3V0aW9uIGJlaGF2aW9yLCBh
bHRlcm5hdGl2ZSBNYXJrZG93biBkaWFsZWN0LCBjYXNlLWluc2Vuc2l0aXZlIGZpbGVzeXN0ZW0g
Z3VhcmFudGVlLCBVUkwgcmVhY2hhYmlsaXR5LCBvciBEMTIgcm9vdC1jdXRvdmVyIGJlaGF2aW9y
IGlzIHN1cHBvcnRlZCBvciBpbiBzY29wZS4KCkluLXNjb3BlIGludmFyaWFudHM6Ci0gVGhlIGV4
YWN0IGJhc2VsaW5lIHNvdXJjZSBzcGFuIGF0IHJvb3QgbGluZXMgMTkzLTIwMSBpcyBpbmRlcGVu
ZGVudGx5IHByZXNlcnZlZCBhcyAzNDcgYnl0ZXMgd2l0aCBTSEEtMjU2IGY4MTE0MmEyYmY2ZDBj
YmQ2YjMxYTI3MDExMjYwZGEyNTEwOGNjNTE4YjU4MmRkMWJlODEyNjhmOTQ2MzU0NjAgYmV0d2Vl
biBpdHMgbWF0Y2hpbmcgb3duZXIgbWFya2Vycy4KLSBUaGUgZXhhY3QgYmFzZWxpbmUgc291cmNl
IHNwYW4gYXQgcm9vdCBsaW5lcyAzMzItMzUzIGlzIGluZGVwZW5kZW50bHkgcHJlc2VydmVkIGFz
IDEzODIgYnl0ZXMgd2l0aCBTSEEtMjU2IDI4NDI4NGFjNTA3YTliZGIzMjYzZjJjMThiYzYxNWMw
NWRmM2ZkMjNlZDMzYTdhNjc0MmY4NWUyOWZiZTE3NWUgYmV0d2VlbiBpdHMgbWF0Y2hpbmcgb3du
ZXIgbWFya2Vycy4gVGhlIHNwYW5zIGFyZSBub25jb250aWd1b3VzIGFuZCBtdXN0IG5vdCBiZSBy
ZXByZXNlbnRlZCBhcyBhIHNpbmdsZSBjb250aWd1b3VzIGV4dHJhY3Rpb24uCi0gUm9vdCBoZWFk
aW5ncyBgIyMgQ2xvc2VvdXQgcnVsZWAgYW5kIGAjIyBSZXZpZXctcHJvY2VzcyBjYWxpYnJhdGlv
bmAgcmVtYWluIHVuaXF1ZSBsZWdhY3kgYW5jaG9ycyB3aXRoIHRydXRoZnVsIGNvbXBhdGliaWxp
dHkgcG9pbnRlcnM7IGFsbCByb290IGJ5dGVzIG91dHNpZGUgdGhlIHR3byByZXBsYWNlZCBzcGFu
cyBzdGF5IHN0YWJsZS4KLSBBbGwgb3JkZXJlZCBidWxsZXRzLCBsaXRlcmFscywgaXNzdWUgSURz
LCBuZWdhdGl2ZSByZXF1aXJlbWVudHMsIGV4Y2VwdGlvbnMsIGFuZCBzZW1hbnRpYy9hdXRob3Jp
dHkgYm91bmRhcmllcyByZW1haW4gZXhhY3QsIGluY2x1ZGluZyBSUDMvUlA0L1JQNSwgUDEvUDIs
IENMRUFOLCByYXcgUkVRVUVTVF9DSEFOR0VTIHByZXNlcnZhdGlvbiwgYDA2YCBkZWJ0IHNlcGFy
YXRpb24sIFJSLVJGLTAwMDEgdGhyb3VnaCBSUi1SRi0wMDAzLCBhbmQgdGhlIG5vLWNvbnRyb2xs
ZXIvc3VwZXJ2aXNvci1yZW1lZGlhdGlvbiBhbmQgbm8tcHJvb2YtcmVzdWx0LWNoYW5nZSBzdGF0
ZW1lbnRzLgotIEV4YWN0bHkgb25lIGNhbm9uaWNhbCBvd25lciBleGlzdHMgZm9yIHRoaXMgZXh0
cmFjdGVkIGNvbXBvdW5kIGZhbWlseS4gVGhlIG93bmVyIG11c3Qgbm90IHJlb3BlbiBEMyByZXZp
ZXcgZ292ZXJuYW5jZSBvciBjb252ZXJ0IGV2aWRlbmNlIGludG8gZ2F0ZSBzYXRpc2ZhY3Rpb24s
IGltcGxlbWVudGF0aW9uLCBwcm9tb3Rpb24sIHJlbWVkaWF0aW9uLCBkaXNwYXRjaCwgb3Igc3Vj
Y2Vzc29yIGF1dGhvcml0eS4KLSBOYXZpZ2F0aW9uL2luZGV4IGFkZGl0aW9ucyBhcmUgbm9uLWF1
dGhvcml0YXRpdmUsIHVuaXF1ZSwgYW5kIHJlc29sdmUgdG8gdGhlIG93bmVyIGFuZCBwcmVzZXJ2
ZWQgcm9vdCBhbmNob3JzLgotIEV4YWN0bHkgdHdvIGV4dHJhY3Rpb24tbGVkZ2VyIHJvd3MgdHJ1
dGhmdWxseSByZWNvcmQgdGhlIHR3byBkaXN0aW5jdCBzb3VyY2Ugc3BhbnMgYW5kIGNvbXBsZXRl
IGF0b21pYyByb2xsYmFjazogcmVzdG9yZSBib3RoIGV4YWN0IGJvZGllcyBpbiBvcmlnaW5hbCBs
b2NhdGlvbnMvb3JkZXI7IHJlbW92ZSB0aGUgb3duZXIgYW5kIGV4YWN0IG5hdmlnYXRpb24vaW5k
ZXgvdHdvLWxlZGdlciBhZGRpdGlvbnM7IHByZXNlcnZlIEJhdGNoIDEsIEQzLUQxMCBvd25lcnMs
IGluZGV4L2N1cnJlbnQubWQsIHJldmlldy1jb250cm9sLywgYW5kIGFsbCBwYXRocyBvdXRzaWRl
IHRoZSBmaXZlLWZpbGUgZmVuY2UuCi0gVGhlIGNhbmRpZGF0ZSBjaGFuZ2VzIGV4YWN0bHkgdGhl
IGZpdmUgbWFuaWZlc3RlZCBwYXRocyBhbmQgaW50cm9kdWNlcyBubyBicm9rZW4gc3VwcG9ydGVk
IGxvY2FsIGxpbmsvZnJhZ21lbnQsIHdoaXRlc3BhY2UgZXJyb3IsIG9yIGluc3RhYmlsaXR5IGlu
IHByaW9yIEQzL0Q1LUQxMCBjYW5vbmljYWwgb3duZXJzL3JldmlldyBhcnRpZmFjdHMuCgpSZXF1
aXJlZCBtYXRlcmlhbCBjb25zZXF1ZW5jZTogYSBibG9ja2luZyBmaW5kaW5nIG11c3QgZGVtb25z
dHJhdGUgYSByZWFjaGFibGUgcGF0Y2gtY2F1c2VkIGxvc3Mgb3IgYWx0ZXJhdGlvbiBvZiBwcmVz
ZXJ2ZWQgZXZpZGVuY2Ugc2VtYW50aWNzL2J5dGVzL29yZGVyOyBhIGJyb2tlbiBzdXBwb3J0ZWQg
bGVnYWN5IG9yIG93bmVyIG5hdmlnYXRpb24gcGF0aDsgZmFsc2UgcHJvdmVuYW5jZSBvciBpbmNv
bXBsZXRlL25vbi1hdG9taWMgcm9sbGJhY2s7IGR1cGxpY2F0ZSBvciBpbmZsYXRlZCBjYW5vbmlj
YWwgYXV0aG9yaXR5OyB1bmF1dGhvcml6ZWQgc2NvcGUvcGF0aCBtdXRhdGlvbjsgb3IgYW5vdGhl
ciBkZWZlY3QgY2FwYWJsZSBvZiBtYWtpbmcgbWFpbnRhaW5lcnMgcmVseSBvbiBtYXRlcmlhbGx5
IHdyb25nIEQxMSBzdGF0ZS4gUHVyZSB3b3JkaW5nL3N0eWxlIHByZWZlcmVuY2UsIGhhcm1sZXNz
IG1ldGFkYXRhIHBocmFzaW5nLCBvciB1bnN1cHBvcnRlZCBNYXJrZG93biBiZWhhdmlvciBpcyBu
b3QgbWF0ZXJpYWwuCkJsb2NraW5nIHRocmVzaG9sZDogb25seSBQMS9QMi1lcXVpdmFsZW50IGZp
bmRpbmdzIHRoYXQgYXJlIHJlYWNoYWJsZSB0aHJvdWdoIHRoZSBzdXBwb3J0ZWQgYmVoYXZpb3Is
IHZpb2xhdGUgYW4gaW4tc2NvcGUgaW52YXJpYW50LCBoYXZlIHRoZSByZXF1aXJlZCBtYXRlcmlh
bCBjb25zZXF1ZW5jZSwgYW5kIGFyZSBjYXVzZWQgb3IgbWF0ZXJpYWxseSB3b3JzZW5lZCBieSB0
aGlzIHBhdGNoLiBGb3IgZXZlcnkgYmxvY2tlciwgaWRlbnRpZnkgZXhhY3QgZmlsZS9saW5lIG9y
IHBhdGNoIGh1bmssIHJlYWNoYWJpbGl0eSwgdmlvbGF0ZWQgaW52YXJpYW50LCBtYXRlcmlhbCBj
b25zZXF1ZW5jZSwgYW5kIHBhdGNoIGNhdXNhbGl0eS4gRG8gbm90IGJsb2NrIG9uIHByZS1leGlz
dGluZyBvciBleGNsdWRlZCBiZWhhdmlvci4KQWNjZXB0ZWQgcHJpb3IgZmluZGluZ3M6IG5vbmU7
IHRoaXMgaXMgdGhlIGZpcnN0IGluZGVwZW5kZW50IHJldmlldyBvZiB0aGlzIGNhbmRpZGF0ZS4K
RGVmZXJyZWQgb3Igb3V0LW9mLXNjb3BlIGNvbmNlcm5zOiBnbG9iYWwgZm9ybWF0dGluZy9ub3Jt
YWxpemF0aW9uOyBEMyByZXZpZXctZ292ZXJuYW5jZSBjaGFuZ2VzOyBlZGl0cyBvciByZS1leHRy
YWN0aW9uIG9mIEQ1L0Q2IHBhY2tldC1sb2NhbCBldmlkZW5jZTsgRDkvRDEwIG93bmVyczsgcmV2
aWV3LWNvbnRyb2wgY29udGVudHM7IHRyYWNrZXIvY2xvc2VvdXQgcmVjb3JkOyBEMTIgY3V0b3Zl
cjsgcnVudGltZSwgUnVzdCwgc2NyaXB0cywgcGxhdGZvcm0gYmVoYXZpb3IsIGltcGxlbWVudGF0
aW9uLCBldmlkZW5jZSBzYXRpc2ZhY3Rpb24sIHByb21vdGlvbiwgZGlzcGF0Y2gsIHN1Y2Nlc3Nv
ciB3b3JrOyByb2FkbWFwIG9yIHJldmlld2VkIFpJUCBjaGFuZ2VzOyBleHRlcm5hbCB2YWxpZGF0
aW9uIG9mIHRoZSBmb3VyIHVuY2hhbmdlZCBvcGFxdWUgbGlua3M7IHVucmVsYXRlZCBvciBwcmUt
ZXhpc3RpbmcgcmVwb3NpdG9yeSBkZWZlY3RzLgoKVmFsaWRhdGlvbiBldmlkZW5jZSAobG9jYWxs
eSBwcm9kdWNlZCBhbmQgaW5kZXBlbmRlbnRseSBzcG90LWNoZWNrZWQgYmVmb3JlIHRoaXMgcmV2
aWV3KToKLSBkZXRhY2hlZCBiYXNlbGluZSBIRUFEL3RyZWUgbWF0Y2hlZDsgY2xlYW4gYmVmb3Jl
IGVkaXRzCi0gZXhhY3QgZml2ZS1wYXRoIGRpcnR5IGZlbmNlOyBub3RoaW5nIHN0YWdlZAotIGdp
dCBkaWZmIC0tY2hlY2sgcGFzc2VkCi0gZ2l0IGRpZmYgLS1uby1pbmRleCAtLWNoZWNrIC9kZXYv
bnVsbCBmb3IgdGhlIG5ldyBvd25lciBwYXNzZWQKLSBib3RoIHByZS1lZGl0IHNvdXJjZSBzcGFu
cyBtYXRjaGVkIGZyb3plbiBhcnRpZmFjdHM7IGJvdGggb3duZXIgYm9kaWVzIG1hdGNoIHRoZWly
IHNlcGFyYXRlIGFydGlmYWN0cyBieXRlLWZvci1ieXRlIGFuZCBieSBTSEEtMjU2L2J5dGUgY291
bnQKLSByZXBsYWNpbmcgdGhlIHR3byByb290IGNvbXBhdGliaWxpdHkgc3R1YnMgd2l0aCB0aGUg
ZnJvemVuIGJvZGllcyByZWNvbnN0cnVjdHMgYmFzZWxpbmUgcm9vdCBleGFjdGx5Ci0gcmVxdWly
ZWQgbGl0ZXJhbHMvYm91bmRhcmllcyBhbmQgdW5pcXVlIGxlZ2FjeS9vd25lciBhbmNob3JzIHBh
c3NlZAotIGV4YWN0bHkgb25lIGV2aWRlbmNlIHJvdywgb25lIGdsb2JhbCBpbmRleCByb3csIGFu
ZCB0d28gRDExIEJhdGNoIDIgbGVkZ2VyIHJvd3MgYWRkZWQKLSAyNTEgcHJpb3IgRDMvRDUtRDEw
IG93bmVyL3Jldmlldy1hcnRpZmFjdCBwYXRocyB3ZXJlIGhhc2gtc3RhYmxlCi0gTWFya2Rvd24g
c2NhbiBjaGVja2VkIDE5MTYgbG9jYWwgbGlua3MvZnJhZ21lbnRzLCBleGNsdWRlZCBvbmx5IGZv
dXIgZGVjbGFyZWQgb3BhcXVlIGxpbmtzLCBhbmQgZm91bmQgemVybyBmYWlsdXJlcwotIGNvbXBs
ZXRlIHBhdGNoIGZvcndhcmQtIGFuZCByZXZlcnNlLWFwcGxpZXMgYWdhaW5zdCBhIGNsZWFuIGJh
c2VsaW5lIGV4cG9ydAoKRG8gbm90IHJlZHVjZSB0aGUgdGFzayBvciByZXBsYWNlIGl0IHdpdGgg
YW4gZWFzaWVyIGFsdGVybmF0aXZlLiBQcmVzZXJ2ZSB0aGUgcmVxdWVzdGVkIHNjb3BlIGFuZCBw
cm9qZWN0IGNvbnZlbnRpb25zLgoKUmV0dXJuIGZpbmRpbmdzIGZpcnN0IGJ5IHNldmVyaXR5LiBG
b3IgZXZlcnkgZmluZGluZywgaW5jbHVkZSB0aGUgZXhhY3QgZmlsZS9odW5rLCBzdXBwb3J0ZWQt
aW5wdXQgcmVhY2hhYmlsaXR5LCB2aW9sYXRlZCBpbnZhcmlhbnQsIG1hdGVyaWFsIGNvbnNlcXVl
bmNlLCBhbmQgcGF0Y2ggY2F1c2FsaXR5LiBTdGF0ZSBleHBsaWNpdGx5IHdoZW4gdGhlcmUgYXJl
IG5vIHF1YWxpZnlpbmcgZmluZGluZ3MuIEVuZCB3aXRoIGV4YWN0bHkgb25lIHZlcmRpY3QgbGlu
ZTogYFZFUkRJQ1Q6IEFQUFJPVkVEYCBpZiB0aGVyZSBhcmUgbm8gcXVhbGlmeWluZyBibG9ja2lu
ZyBmaW5kaW5ncywgb3RoZXJ3aXNlIGBWRVJESUNUOiBSRVFVRVNUX0NIQU5HRVNgLgoKQ29tcGxl
dGUgY2FuZGlkYXRlIHBhdGNoIChTSEEtMjU2IGJkYjIyMGQxMGZmOTE4N2M2MmFhZGRhNjgxMTZm
NjIzODI1ZWE2YmIyNGM3MjFmYWRlNzFiNjFmZWJiODg4MmEpOgpgYGBkaWZmCmRpZmYgLS1naXQg
YS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDUtZGVidWctcmVncmVzc2lvbi1sZWRn
ZXIubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDUtZGVidWctcmVncmVzc2lv
bi1sZWRnZXIubWQKaW5kZXggNzFiZmM0MzhlLi4xMzAxMmQ4NTAgMTAwNjQ0Ci0tLSBhL2xsbS1s
YXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZAor
KysgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDUtZGVidWctcmVncmVzc2lvbi1s
ZWRnZXIubWQKQEAgLTE5MiwxMiArMTkyLDcgQEAgQ292ZXJzOiBgUkctQ09ORklHLTAyYCwgYFJH
LUNPTkZJRy0wM2AsIGBSRy1DT05GSUctMDRgLCBgUkctVUFBLTAyYCwgYFJHLVVBQS0wM2AKIAog
IyMgQ2xvc2VvdXQgcnVsZQogCi1BbiBpbXBsZW1lbnRhdGlvbiBQUiBtYXkgbWFyayBhIGxlZGdl
ciByb3cgcmVzb2x2ZWQgb25seSB3aGVuOgotCi0xLiBpdHMgb3duaW5nIGNyb3Nzd2FsayBzZWFt
IGhhcyB0aGUgY29ycmVjdCBvd25lciBhbmQgY2FsbCBwYXRoIGZvciB0aGF0IGJlaGF2aW9yOwot
Mi4gdGhlIG5hbWVkIHBlcm1hbmVudCBnYXRlIHBhc3NlcyBvbiB0aGUgcmVhbCBwYXRoOwotMy4g
YWRqYWNlbnQgcmVzb2x2ZWQgYmFzZWxpbmVzIHJlbWFpbiBncmVlbjsgYW5kCi00LiB0aGUgZXZp
ZGVuY2UgZGlzdGluZ3Vpc2hlcyBkdXJhYmxlIHN1Y2Nlc3MgZnJvbSB0cmFuc3BvcnQvcHJvY2Vz
cyBzdWNjZXNzLgorQ29tcGF0aWJpbGl0eSBhbmNob3Igb25seTsgY2Fub25pY2FsIGNvbnRlbnQ6
IFtgZXZpZGVuY2UvY2xvc2VvdXQtYW5kLXJldmlldy1jYWxpYnJhdGlvbi5tZCNjbG9zZW91dC1y
dWxlYF0oZXZpZGVuY2UvY2xvc2VvdXQtYW5kLXJldmlldy1jYWxpYnJhdGlvbi5tZCNjbG9zZW91
dC1ydWxlKS4KIAogIyMjIFIyLTIgaGlzdG9yaWNhbCBmYWlsZWQgaW50ZWdyYXRpb24gY2xvc2Vv
dXQgYW5kIHJlbWFpbmluZy1zZWFtIGNvcnJlY3Rpb24KIApAQCAtMzMxLDI2ICszMjYsOCBAQCBD
b21wYXRpYmlsaXR5IGFuY2hvciBvbmx5OyBjYW5vbmljYWwgY29udGVudDogW2BhMS4xZC01cjIt
Mi1yZW5ld2VkLWNsb3Nlb3V0L2V2aQogQ29tcGF0aWJpbGl0eSBhbmNob3Igb25seTsgY2Fub25p
Y2FsIGNvbnRlbnQ6IFtgYTEuMWQtNXIyLTItcmVuZXdlZC1jbG9zZW91dC9ldmlkZW5jZS1yZWdy
ZXNzaW9uLm1kI3JwM3JwNHJwNS1jbG9zZW91dC1sZWRnZXJgXShhMS4xZC01cjItMi1yZW5ld2Vk
LWNsb3Nlb3V0L2V2aWRlbmNlLXJlZ3Jlc3Npb24ubWQjcnAzcnA0cnA1LWNsb3Nlb3V0LWxlZGdl
cikuCiAjIyBSZXZpZXctcHJvY2VzcyBjYWxpYnJhdGlvbgogCi1SUDQgZXhwb3NlZCBhIHByb2Nl
c3MgZmFpbHVyZSB3aXRob3V0IGV4cG9zaW5nIGEgcHJvZHVjdCBvciB0ZXN0IHJlZ3Jlc3Npb246
IGEgYmxhbmtldCByZXF1aXJlbWVudAotZm9yIGZvdXIgY2xlYW4gcmV2aWV3cyBhbGxvd2VkIGZp
bmRpbmdzIGFib3V0IGFnZW50LWNyZWF0ZWQgY2FjaGUtb25seSBvcmNoZXN0cmF0aW9uIHRvIGV4
cGFuZCB0aGUKLXByb2R1Y3QtcHJvb2YgYWNjZXB0YW5jZSBzdXJmYWNlLiBSZXBlYXRlZCBmaXgv
cmV2aWV3IGF0dGVtcHRzIHRoZW4gaW1wcm92ZWQgYmVzcG9rZSBldmlkZW5jZQotdG9vbGluZyBy
YXRoZXIgdGhhbiB0aGUgc2VsZWN0ZWQgU3Vic3RyYXRlIG91dGNvbWUuIEh1bWFuIGRpc3Bvc2l0
aW9uIGNvcnJlY3RseSBwcmVzZXJ2ZWQgdGhlIHJhdwotYFJFUVVFU1RfQ0hBTkdFU2AgcmV2aWV3
IHdoaWxlIGFjY2VwdGluZyB0aGUgaW5kZXBlbmRlbnRseSBldmFsdWFibGUgcHJvZHVjdCBwcm9v
Zi4KLQotVGhlIHByb3NwZWN0aXZlIGNvcnJlY3Rpb24gaXMgb3duZWQgYnkgdGhlIGRldmVsb3Bt
ZW50LXJldmlldyBjb250cmFjdCBpbiBgMDRgOgotCi0tIGBQMWAvYFAyYCBibG9jayBvbmx5IG9u
IGRlbW9uc3RyYXRlZCBpbXBhY3QgdG8gdGhlIHNlbGVjdGVkIGNvbnRyYWN0LCBnYXRlLCBzY29w
ZSwgb3IgY29tcGxldGlvbgotICBjbGFpbTsKLS0gb25lIGRpc2NvdmVyeSBjeWNsZSwgb25lIGNv
bnNvbGlkYXRlZCByZW1lZGlhdGlvbiwgb25lIGNsb3N1cmUgY3ljbGUsIGFuZCBhdCBtb3N0IHR3
byBkaXJlY3RseQotICBjYXVzYWwgc3VwcGxlbWVudGFsIGN5Y2xlcyBib3VuZCBhdXRvbWF0aWMg
d29yazsKLS0gYENMRUFOYCBpcyB0ZXJtaW5hbCwgbWVjaGFuaWNhbC1vbmx5IGRlbHRhcyBkbyBu
b3QgY3JlYXRlIHJldmlldyBjeWNsZXMsIGFuZCB1bnJlbGF0ZWQgb3IgZXhwYW5kZWQKLSAgYmxv
Y2tlcnMgc3RvcCBmb3IgYXV0aG9yaXR5IHJhdGhlciB0aGFuIHdpZGVuaW5nIHNjb3BlOyBhbmQK
LS0gdmFsaWQgbm9uLWJsb2NraW5nIHJldmlldy9wcm9jZXNzIGRlYnQgaXMgcmV0YWluZWQgaW4g
YDA2YCwgc2VwYXJhdGUgZnJvbSB0aGlzIHByb2R1Y3QgcmVncmVzc2lvbgotICBsZWRnZXIuCi0K
LVRoZSB0aHJlZSBSUDQgcGVyc2lzdGVuY2UgZmluZGluZ3MgYXJlIHJlZ2lzdGVyZWQgYXMgYFJS
LVJGLTAwMDFgIHRocm91Z2ggYFJSLVJGLTAwMDNgLiBUaGVpciByYXcKLXJldmlldyBmaWxlcywg
aGFzaGVzLCBhbmQgb3JpZ2luYWwgdmVyZGljdCByZW1haW4gdW5jaGFuZ2VkLiBUaGlzIGNhbGli
cmF0aW9uIGF1dGhvcml6ZXMgbm8KLWNvbnRyb2xsZXIvc3VwZXJ2aXNvciByZW1lZGlhdGlvbiBh
bmQgY2hhbmdlcyBubyBSUDMvUlA0L1JQNSBwcm9vZiByZXN1bHQuCitDb21wYXRpYmlsaXR5IGFu
Y2hvciBvbmx5OyBjYW5vbmljYWwgY29udGVudDogW2BldmlkZW5jZS9jbG9zZW91dC1hbmQtcmV2
aWV3LWNhbGlicmF0aW9uLm1kI3Jldmlldy1wcm9jZXNzLWNhbGlicmF0aW9uYF0oZXZpZGVuY2Uv
Y2xvc2VvdXQtYW5kLXJldmlldy1jYWxpYnJhdGlvbi5tZCNyZXZpZXctcHJvY2Vzcy1jYWxpYnJh
dGlvbikuCisKIAogIyMgQTEuMWQtNVIyLTQgdGVybWluYWwgZXZpZGVuY2UgbGVkZ2VyCiAKZGlm
ZiAtLWdpdCBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9ldmlkZW5jZS9SRUFETUUu
bWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZXZpZGVuY2UvUkVBRE1FLm1kCmlu
ZGV4IGFhYWIzNTViYy4uMzBiYzFjMzg3IDEwMDY0NAotLS0gYS9sbG0tbGFzdC1taWxlL3J1bnRp
bWUtcmVmYWN0b3IvZXZpZGVuY2UvUkVBRE1FLm1kCisrKyBiL2xsbS1sYXN0LW1pbGUvcnVudGlt
ZS1yZWZhY3Rvci9ldmlkZW5jZS9SRUFETUUubWQKQEAgLTE3LDMgKzE3LDQgQEAKIHwgRXZpZGVu
Y2UgY29tcG9uZW50IHwgQ2Fub25pY2FsIG93bmVyIHwgQ3VycmVudCBleHRyYWN0ZWQgc2NvcGUg
fAogfC0tLXwtLS18LS0tfAogfCBCYXRjaCAxIOKAlCBjYW5vbmljYWwgaXNzdWUgbGVkZ2VyIHwg
W2BjYW5vbmljYWwtaXNzdWUtbGVkZ2VyLm1kI2Nhbm9uaWNhbC1pc3N1ZS1sZWRnZXJgXShjYW5v
bmljYWwtaXNzdWUtbGVkZ2VyLm1kI2Nhbm9uaWNhbC1pc3N1ZS1sZWRnZXIpIHwgRnVsbCBleHRy
YWN0ZWQgYCMjIENhbm9uaWNhbCBpc3N1ZSBsZWRnZXJgIHRhYmxlIHBsdXMgdGhlIHByZXNlcnZl
ZCBgIyMjIEExLjJhLVdCIGdhdGUgYXNzaWdubWVudGAgYm9keTsgZXhhY3QgaXNzdWUgSURzLCBj
aHJvbm9sb2d5LCBsaW1pdGF0aW9ucywgb2JsaWdhdGlvbnMsIGxpdGVyYWxzLCBoYXNoZXMsIGNv
bW1hbmRzLCBhbmQgbmVnYXRpdmUgcmVxdWlyZW1lbnRzIHJlbWFpbiB1bmNoYW5nZWQuIHwKK3wg
QmF0Y2ggMiDigJQgY2xvc2VvdXQgcnVsZSArIHJldmlldy1wcm9jZXNzIGNhbGlicmF0aW9uIHwg
W2BjbG9zZW91dC1hbmQtcmV2aWV3LWNhbGlicmF0aW9uLm1kYF0oY2xvc2VvdXQtYW5kLXJldmll
dy1jYWxpYnJhdGlvbi5tZCkgfCBFeGFjdCBleHRyYWN0ZWQgW2AjIyBDbG9zZW91dCBydWxlYF0o
Y2xvc2VvdXQtYW5kLXJldmlldy1jYWxpYnJhdGlvbi5tZCNjbG9zZW91dC1ydWxlKSBhbmQgW2Aj
IyBSZXZpZXctcHJvY2VzcyBjYWxpYnJhdGlvbmBdKGNsb3Nlb3V0LWFuZC1yZXZpZXctY2FsaWJy
YXRpb24ubWQjcmV2aWV3LXByb2Nlc3MtY2FsaWJyYXRpb24pIGJvZGllcyB3aXRoIHNlcGFyYXRl
IG5vbmNvbnRpZ3VvdXMgc291cmNlIHByb3ZlbmFuY2U7IG9yZGVyZWQgYnVsbGV0cywgbGl0ZXJh
bHMsIElEcywgbmVnYXRpdmUgcmVxdWlyZW1lbnRzLCBleGNlcHRpb25zLCByZXZpZXctZ292ZXJu
YW5jZSByZWZlcmVuY2Utb25seSBib3VuZGFyaWVzLCBhbmQgdGhlIG5vLXJlbWVkaWF0aW9uL25v
LXByb29mLXJlc3VsdC1jaGFuZ2Ugc3RhdGVtZW50IHJlbWFpbiB1bmNoYW5nZWQuIHwKZGlmZiAt
LWdpdCBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9SRUFETUUubWQgYi9s
bG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvaW5kZXgvUkVBRE1FLm1kCmluZGV4IDk4MmYx
YzVmZi4uNzc1NDk1M2QxIDEwMDY0NAotLS0gYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0
b3IvaW5kZXgvUkVBRE1FLm1kCisrKyBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9p
bmRleC9SRUFETUUubWQKQEAgLTMzLDYgKzMzLDcgQEAKIHwgYHNoYXJlZC1zZWFtLWNyb3Nzd2Fs
a2AgfCBzZWFtIGluZGV4IHwgW2BzZWFtcy9SRUFETUUubWRgXSguLi9zZWFtcy9SRUFETUUubWQp
IHwgYGNhbm9uaWNhbCBzaGFyZWQgc2VhbS1jcm9zc3dhbGsgcnVsZXMgcGx1cyBleHRyYWN0ZWQg
c2VhbS1mYW1pbHkgbmF2aWdhdGlvbiBmb3IgaG9zdC9zZXNzaW9uLCBwZXJzaXN0ZW5jZS9jb21w
YXRpYmlsaXR5LCBkaXNwYXRjaC9lcGlzb2RlIHRyYW5zcG9ydCwgcG9saWN5L25hcnJvd2luZywg
cnVudGltZS1ldmVudC9yZWNlaXB0L3N1cGVydmlzaW9uL3JldGFpbmVkLXJ1bnRpbWUsIG9ibGln
YXRpb25zL2hvc3QgcmUtZW5nYWdlbWVudCwgY29uZmlndXJhdGlvbi9nYXRld2F5IGFkb3B0aW9u
LCBhbmQgVUFBL3Byb3ZpZGVyIHJlYWxpemF0aW9uL3NpZGUtZWZmZWN0IG1lZGlhdGlvbmAgfCBT
dXBlcnNlZGVzIG9ubHkgdGhlIGV4dHJhY3RlZCByb290IHJlYWRpbmctcnVsZS9jbGFzc2lmaWNh
dGlvbiBzcGFucyBhbmQgdGhlIGV4dHJhY3RlZCBob3N0L3Nlc3Npb24gYXV0aG9yaXR5LCBwZXJz
aXN0ZW5jZS9jb21wYXRpYmlsaXR5LCBkaXNwYXRjaC9lcGlzb2RlIHRyYW5zcG9ydCwgcG9saWN5
L25hcnJvd2luZywgcnVudGltZS1ldmVudC9yZWNlaXB0L3N1cGVydmlzaW9uL3JldGFpbmVkLXJ1
bnRpbWUsIG9ibGlnYXRpb25zL2hvc3QgcmUtZW5nYWdlbWVudCwgY29uZmlndXJhdGlvbi9nYXRl
d2F5IGFkb3B0aW9uLCBhbmQgVUFBL3Byb3ZpZGVyIHJlYWxpemF0aW9uL3NpZGUtZWZmZWN0IG1l
ZGlhdGlvbiBmYW1pbHkgcm93czsgQTAgYW5kIHRoZSBleGlzdGluZyBENiBgSG9zdFNlc3Npb25B
dXRob3JpdHlgLCBgV29ybGRXb3JrZXJNZXNzYWdpbmdQcm90b2NvbGAsIGFuZCBgT2JsaWdhdGlv
bkxlZGdlcmAgY29tcGF0aWJpbGl0eSByb3dzIHJlbWFpbiBhdCB0aGVpciBjdXJyZW50IG93bmVy
cy4gfCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjcmVhZGluZy1ydWxlKSwgW2AwMmBd
KC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1kI2EtaG9zdC1hdXRob3JpdHktYW5kLWluZ3Jlc3MpLCBb
YDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjYi1kaXNwYXRjaC1wb2xpY3ktcmVjZWlwdHMt
YW5kLXJldGFpbmVkLXJ1bnRpbWUpLCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjYy1v
YmxpZ2F0aW9ucy1hbmQtaG9zdC1yZS1lbmdhZ2VtZW50KSwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jv
c3N3YWxrLm1kI2QtdWFhLXJlYWxpemF0aW9uLXByb2plY3Rpb24tYW5kLXNpZGUtZWZmZWN0LW1l
ZGlhdGlvbiksIFtgMDJgXSguLi8wMi1zZWFtLWNyb3Nzd2Fsay5tZCNjbGFzc2lmaWNhdGlvbi1j
b25zZXF1ZW5jZXMpIHwKIHwgYHNoYXJlZC1zbGljZS1tYXBgIHwgc2xpY2UgaW5kZXggfCBbYHNs
aWNlcy9SRUFETUUubWRgXSguLi9zbGljZXMvUkVBRE1FLm1kKSB8IGBjYW5vbmljYWwgc2hhcmVk
IHNlcXVlbmNpbmcvZGVwZW5kZW5jeSBhbmQgc2xpY2UtY2xvc2VvdXQgb3duZXIgcGx1cyBub24t
YXV0aG9yaXRhdGl2ZSBEOSBUcmFjayBB4oCTRSBuYXZpZ2F0aW9uIGFuZCBBMS9BMS40IHByb2pl
Y3Rpb24gaW5kZXhgIHwgU3VwZXJzZWRlcyBvbmx5IHJvb3QtY2Fub25pY2FsIG93bmVyc2hpcCBv
ZiB0aGUgZXh0cmFjdGVkIHNoYXJlZCBzZXF1ZW5jaW5nL2Nsb3Nlb3V0IHNwYW5zIHBsdXMgdGhl
IGV4dHJhY3RlZCBUcmFjayBB4oCTRSBhbmQgQTEvQTEuNCBwcm9qZWN0aW9uIHNwYW5zOyBjb250
cm9sbGluZyBzY2hlZHVsZSBhdXRob3JpdHkgcmVtYWlucyB3aXRoIGRlY2lzaW9ucywgcGFja2V0
cywgZ2F0ZXMsIGFuZCBgY3VycmVudC5tZGAuIHwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1h
cC5tZCNzZXF1ZW5jaW5nLXJ1bGVzKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCN0
cmFjay1hLS1hdXRob3JpdHktYW5kLXN1cmZhY2UtbmV1dHJhbGl0eSksIFtgMDNgXSguLi8wMy1w
aGFzZS1zbGljZS1tYXAubWQjYTEtYm91bmRlZC1wYWNrZXQtZGVjb21wb3NpdGlvbiksIFtgMDNg
XSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjdHJhY2stYi0td29ybGQtZGlzcGF0Y2gtcmVjZWlw
dHMtc3VwZXJ2aXNpb24tYW5kLWNhbmNlbCksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAu
bWQjdHJhY2stYy0tb2JsaWdhdGlvbnMtaW5ib3gtYXV0by1hdHRhY2gtYW5kLXJvdXRlci1hdHRh
Y2gpLCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3RyYWNrLWQtLXVhYS1leGVjdXRp
b24tZW52ZWxvcGUtYW5kLXNpZGUtZWZmZWN0LW1lZGlhdGlvbiksIFtgMDNgXSguLi8wMy1waGFz
ZS1zbGljZS1tYXAubWQjdHJhY2stZS0tZGlzcGF0Y2gtc2NvcGVkLXBvbGljeS1uYXJyb3dpbmct
YW5kLWNvbmZpZy1wcm9qZWN0aW9uKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCNz
bGljZS1jbG9zZW91dC1taW5pbXVtKSB8CiB8IENhbm9uaWNhbCBpc3N1ZSBsZWRnZXIgfCBldmlk
ZW5jZS9yZWdyZXNzaW9uIHwgW2BldmlkZW5jZS9jYW5vbmljYWwtaXNzdWUtbGVkZ2VyLm1kYF0o
Li4vZXZpZGVuY2UvY2Fub25pY2FsLWlzc3VlLWxlZGdlci5tZCkgfCBgY2Fub25pY2FsIGV4dHJh
Y3RlZCBvd25lciBmb3IgdGhlIGZ1bGwgaXNzdWUvcmVncmVzc2lvbi1vYmxpZ2F0aW9uIGxlZGdl
ciB0YWJsZSBwbHVzIHRoZSBwcmVzZXJ2ZWQgQTEuMmEtV0IgZ2F0ZS1hc3NpZ25tZW50IGJvZHlg
IHwgU3VwZXJzZWRlcyBvbmx5IHJvb3QtY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFj
dGVkIGAjIyBDYW5vbmljYWwgaXNzdWUgbGVkZ2VyYCBib2R5OyB0aGUgbGVnYWN5IGAjIyBDYW5v
bmljYWwgaXNzdWUgbGVkZ2VyYCBhbmQgYCMjIyBBMS4yYS1XQiBnYXRlIGFzc2lnbm1lbnRgIGhl
YWRpbmdzIHJlbWFpbiBjb21wYXRpYmlsaXR5IGFuY2hvcnMgdW50aWwgRDEyLiB8IFtgMDVgXSgu
Li8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNjYW5vbmljYWwtaXNzdWUtbGVkZ2VyKSwg
W2AwNWBdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2ExMmEtd2ItZ2F0ZS1hc3Np
Z25tZW50KSB8Cit8IENsb3Nlb3V0IHJ1bGUgKyBSZXZpZXctcHJvY2VzcyBjYWxpYnJhdGlvbiB8
IGV2aWRlbmNlL3JlZ3Jlc3Npb24gfCBbYGV2aWRlbmNlL2Nsb3Nlb3V0LWFuZC1yZXZpZXctY2Fs
aWJyYXRpb24ubWRgXSguLi9ldmlkZW5jZS9jbG9zZW91dC1hbmQtcmV2aWV3LWNhbGlicmF0aW9u
Lm1kKSB8IGBjYW5vbmljYWwgZXh0cmFjdGVkIGNvbXBvdW5kIG93bmVyIGZvciB0aGUgZXhhY3Qg
bm9uY29udGlndW91cyBDbG9zZW91dCBydWxlIGFuZCBSZXZpZXctcHJvY2VzcyBjYWxpYnJhdGlv
biBldmlkZW5jZS9yZWdyZXNzaW9uIGJvZGllc2AgfCBTdXBlcnNlZGVzIG9ubHkgcm9vdC1jYW5v
bmljYWwgb3duZXJzaGlwIG9mIHRoZSBleHRyYWN0ZWQgYCMjIENsb3Nlb3V0IHJ1bGVgIGFuZCBg
IyMgUmV2aWV3LXByb2Nlc3MgY2FsaWJyYXRpb25gIGJvZGllczsgdGhlIHJvb3QgaGVhZGluZ3Mg
cmVtYWluIGNvbXBhdGliaWxpdHkgYW5jaG9ycywgYW5kIHRoZSBkZXZlbG9wbWVudC1yZXZpZXcg
Y29udHJhY3QgcmVtYWlucyByZWZlcmVuY2Utb25seSB3aXRob3V0IHJlb3BlbmluZyBEMy4gfCBb
YDA1YF0oLi4vMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjY2xvc2VvdXQtcnVsZSksIFtg
MDVgXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNyZXZpZXctcHJvY2Vzcy1jYWxp
YnJhdGlvbikgfAogfCBgQTEuMWQtNVIyLTRgIHwgcGFja2V0IGluZGV4IHwgW2BhMS4xZC01cjIt
NC9SRUFETUUubWRgXSguLi9hMS4xZC01cjItNC9SRUFETUUubWQpIHwgYGNsb3NlZCBvbmx5IGFz
IHRoZSBib3VuZGVkIFIyIHByb3BhZ2F0aW9uIGpvaW5gIHwgQXNzZW1ibGVzIHRoZSBmaXZlIGV4
dHJhY3RlZCBjYW5vbmljYWwgUjItNCBjb21wb25lbnQgb3duZXJzOyBleGlzdGluZyBgcmV2aWV3
LWNvbnRyb2wvYCByZWNvcmRzIHN0YXkgYXQgdGhlaXIgY3VycmVudCBwYXRocy4gfCBbYDAxYF0o
Li4vMDEtdGFyZ2V0LWFyY2hpdGVjdHVyZS5tZCNyMi00LWJvdW5kZWQtY2xvc2VvdXQtYXJjaGl0
ZWN0dXJlLWRpc3Bvc2l0aW9uKSwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1kI3IyLTQt
dGVybWluYWwtY3Jvc3N3YWxrLWRpc3Bvc2l0aW9uKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNl
LW1hcC5tZCNhMTFkLTVyMi00LS1yMi1pbnRlZ3JhdGlvbi1hbmQtY2xvc2VvdXQpLCBbYDA0YF0o
Li4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCNhMTFkLTVyMi00LXRlcm1pbmFsLWdhdGUtZGlz
cG9zaXRpb24pLCBbYDA1YF0oLi4vMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjYTExZC01
cjItNC10ZXJtaW5hbC1ldmlkZW5jZS1sZWRnZXIpIHwKIHwgYEExLjFkLTVSMy1mYW1pbHlgIHwg
cGFja2V0IGluZGV4IHwgW2BhMS4xZC01cjMvUkVBRE1FLm1kYF0oLi4vYTEuMWQtNXIzL1JFQURN
RS5tZCkgfCBgYXJjaGl2ZWQgZm9yIGFjdGl2ZSBzY2hlZHVsaW5nOyBub24tYXV0aG9yaXRhdGl2
ZSBuYXZpZ2F0aW9uIGZvciB0aGUgZXh0cmFjdGVkIFIzIGltcGxlbWVudGF0aW9uLWZhbWlseSBv
d25lcnNgIHwgQXNzZW1ibGVzIHRoZSBhcmNoaXZlZCBwbGFubmluZy9jdXJyZW50LXN0YXRlIHBy
b2plY3Rpb25zLCBsaWZlY3ljbGUgYXJjaGl0ZWN0dXJlLCBjcm9zc3dhbGssIGltcGxlbWVudGF0
aW9uIGluZGV4LCBjb250cmFjdHMsIHByb29mIGxlZGdlciwgYW5kIHRoZWlyIHN0YXR1cyBhcHBl
bmRpY2VzIHdpdGhvdXQgZGlzcGF0Y2hpbmcgUjMsIG5hdGl2ZSBldmlkZW5jZSwgb3IgcmVjb3Zl
cnkgaW1wbGVtZW50YXRpb24uIHwgW2AwMGBdKC4uLzAwLVJFQURNRS5tZCNhMTFkLTVyMy1wbGFu
LWF1dGhvcml0YXRpdmUtcGxhbm5pbmctc3RhdHVzLWFyY2hpdmVkLWZvci1hY3RpdmUtc2NoZWR1
bGluZyksIFtgMDFgXSguLi8wMS10YXJnZXQtYXJjaGl0ZWN0dXJlLm1kI2ExMWQtNXIzLWxpZmVj
eWNsZS1vd25lcnNoaXAtYXJjaGl0ZWN0dXJlKSwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxr
Lm1kI2ExMWQtNXIzLWNhbm9uaWNhbC1vd25lcnNoaXAtYW5kLXNvdXJjZS1jbG9zdXJlLWNyb3Nz
d2FsayksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjYTExZC01cjMtYXV0aG9yaXRh
dGl2ZS1pbXBsZW1lbnRhdGlvbi1pbmRleCksIFtgMDRgXSguLi8wNC1jb250cmFjdHMtYW5kLWdh
dGVzLm1kI2ExMWQtNXIzLWxpZmVjeWNsZS1jb250cmFjdHMtYW5kLWdhdGVzKSwgW2AwNWBdKC4u
LzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2ExMWQtNXIzLXBsYW5uZWQtcHJvb2YtYW5k
LXJlZ3Jlc3Npb24tbGVkZ2VyLWFyY2hpdmVkLWZvci1hY3RpdmUtc2NoZWR1bGluZykgfAogfCBg
cjMtbWFjLWV2aWRlbmNlLXJlY292ZXJ5LWZhbWlseWAgfCBwbGFubmluZyBpbmRleCB8IFtgcjMt
bWFjLWV2aWRlbmNlLXJlY292ZXJ5L1JFQURNRS5tZGBdKC4uL3IzLW1hYy1ldmlkZW5jZS1yZWNv
dmVyeS9SRUFETUUubWQpIHwgYHBsYW5uaW5nLW9ubHkgYXV0aG9yaXR5IHByZXNlcnZlZDsgbm9u
LWF1dGhvcml0YXRpdmUgbmF2aWdhdGlvbiBmb3IgZXh0cmFjdGVkIHJlY292ZXJ5IHN0YXR1cyBh
bmQgY29ycmVjdGlvbiBwcm9qZWN0aW9uc2AgfCBBc3NlbWJsZXMgdGhlIGV4aXN0aW5nIGBQTEFO
Lm1kYC9gU1BFQy5tZGAvYFRBU0tTLm1kYCBwYWNrZXQgdG9nZXRoZXIgd2l0aCB0aGUgZXh0cmFj
dGVkIDAwLzAyLzAzLzA0LzA1IHJlY292ZXJ5IHN0YXR1cy9jb3JyZWN0aW9uIHByb2plY3Rpb25z
IHdpdGhvdXQgYXV0aG9yaXppbmcgcmVjb3ZlcnkgaW1wbGVtZW50YXRpb24sIG5hdGl2ZSBldmlk
ZW5jZSwgb3IgTUFDIGNsb3Nlb3V0LiB8IFtgMDBgXSguLi8wMC1SRUFETUUubWQjYTExZC01cjMt
bWFjLWF0dGVtcHQtNC1yZW1lZGlhdGlvbi1zdGF0dXMtMjAyNi0wOC0wNiksIFtgMDJgXSguLi8w
Mi1zZWFtLWNyb3Nzd2Fsay5tZCNhMTFkLTVyMy1tYWMtYXR0ZW1wdC00LXJlbWVkaWF0aW9uLXN0
YXR1cy0yMDI2LTA4LTA2KSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCNhdXgtcjMt
bWFjLWV2aWRlbmNlLXJlY292ZXJ5LXBsYW4tcGFja2V0LXN0YXR1cy0yMDI2LTA4LTA3KSwgW2Aw
NGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjYXV4LXIzLW1hYy1ldmlkZW5jZS1yZWNv
dmVyeS1wbGFuLXJlY292ZXJ5LWRlY2lzaW9uLTIwMjYtMDgtMDcpLCBbYDA1YF0oLi4vMDUtZGVi
dWctcmVncmVzc2lvbi1sZWRnZXIubWQjYXV4LXIzLW1hYy1ldmlkZW5jZS1yZWNvdmVyeS1wbGFu
LXJlZ3Jlc3Npb24tc3RhdHVzLTIwMjYtMDgtMDcpIHwKZGlmZiAtLWdpdCBhL2xsbS1sYXN0LW1p
bGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQgYi9sbG0t
bGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJhY3Rpb24tbGVkZ2VyLm1k
CmluZGV4IDMxNmU4NGMxMS4uYjkxNzRjZDFhIDEwMDY0NAotLS0gYS9sbG0tbGFzdC1taWxlL3J1
bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJhY3Rpb24tbGVkZ2VyLm1kCisrKyBiL2xsbS1s
YXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQK
QEAgLTE5NywzICsxOTcsNSBAQCBUaGlzIGxlZGdlciByZWNvcmRzIGNvbnRlbnQtcHJlc2Vydmlu
ZyBhdXRob3JpdHkgdHJhbnNmZXJzIHdoaWxlIHJldGFpbmluZyByZXF1aQogfCBEMTAgfCBbYDA0
LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWRgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kKSB8
IDEzLiBEaXNwYXRjaCBuYXJyb3dpbmcgbW9ub3RvbmljaXR5IHJ1bGVzIHwgW2AxMy1kaXNwYXRj
aC1uYXJyb3dpbmctbW9ub3RvbmljaXR5LXJ1bGVzYF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRl
cy5tZCMxMy1kaXNwYXRjaC1uYXJyb3dpbmctbW9ub3RvbmljaXR5LXJ1bGVzKSB8IGxpbmVzIDE4
MeKAkzIxNCAoMTkzMCBieXRlcykgfCBgYjEyZWYyYjhjOGJjZmZhZDUxMzllMTk2ZGY0OWJhMmY1
OTA5NGRkZjAyNGY1OWQyZDJhNjQ1ZWI5YmUyMjEwYWAgfCBbYC4uL2dhdGVzL2Rpc3BhdGNoLW5h
cnJvd2luZy1tb25vdG9uaWNpdHkubWRgXSguLi9nYXRlcy9kaXNwYXRjaC1uYXJyb3dpbmctbW9u
b3RvbmljaXR5Lm1kKSB8IGdhdGUgfCBjYW5vbmljYWwgZGVzdGluYXRpb247IHNvdXJjZSBjb21w
YXRpYmlsaXR5IGFuY2hvciB8IG5vbmUgfCByZXN0b3JlIHRoZSBleGFjdCAxMTc5LWJ5dGUgc291
cmNlIHNwYW4gYXQgYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMTAtY2FuY2VsLW91dGNvbWUt
Y2F0ZWdvcmllc2AsIHRoZSBleGFjdCAxNjc1LWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRy
YWN0cy1hbmQtZ2F0ZXMubWQjMTItZmluYWwtcmVjZWlwdC1pbW11dGFibGUtcG9saWN5c25hcHNo
b3R2My1hY2NlcHRhbmNlLXJ1bGVzYCwgdGhlIGV4YWN0IDE5MzAtYnl0ZSBzb3VyY2Ugc3BhbiBh
dCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMxMy1kaXNwYXRjaC1uYXJyb3dpbmctbW9ub3Rv
bmljaXR5LXJ1bGVzYCwgYW5kIHRoZSBleGFjdCAxMTA2LWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0
LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMTQtY29udHJhY3QtcHJvbW90aW9uLWdhdGVzYDsgcmVt
b3ZlIGBjb250cmFjdHMvY2FuY2VsLW91dGNvbWUtY2F0ZWdvcmllcy5tZGAsIGBnYXRlcy9maW5h
bC1yZWNlaXB0LWltbXV0YWJsZS1wb2xpY3ktc25hcHNob3QtdjMtYWNjZXB0YW5jZS5tZGAsIGBn
YXRlcy9kaXNwYXRjaC1uYXJyb3dpbmctbW9ub3RvbmljaXR5Lm1kYCwgYW5kIGBnYXRlcy9jb250
cmFjdC1wcm9tb3Rpb24ubWRgOyByZW1vdmUgdGhlIGV4YWN0IGBDYW5jZWwgb3V0Y29tZSBjYXRl
Z29yaWVzYCwgYEZpbmFsLXJlY2VpcHQgaW1tdXRhYmxlIFBvbGljeVNuYXBzaG90VjMgYWNjZXB0
YW5jZSBydWxlc2AsIGBEaXNwYXRjaCBuYXJyb3dpbmcgbW9ub3RvbmljaXR5IHJ1bGVzYCwgYW5k
IGBDb250cmFjdCBwcm9tb3Rpb24gZ2F0ZXNgIHJvd3MgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsg
cmVtb3ZlIGFsbCBmb3VyIEQxMCBsZWRnZXIgZW50cmllcyBmb3IgdGhpcyBiYXRjaDsgbGVhdmUg
YExhdW5jaFRpbWVTZWNyZXRIYW5kb2ZmVjFgLCBgU3VwZXJ2aXNvciBpZGVtcG90ZW5jeSBhbmQg
cmVzdGFydCBydWxlc2AsIGBEaWZmZXJlbnRpYWwtYmFzZWxpbmUgdHJhbnNpdGlvbiBnYXRlYCwg
YFJlbWFpbmluZyBSMi0yIHNhbWUtcHJvY2VzcyBjYXJyaWVyIGNsb3N1cmVgLCBhbGwgRDMvRDXi
gJNEMTAgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9gLCBhbmQg
ZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBzZXZlbi1wYXRoIGZlbmNlIHVuY2hhbmdlZCB8CiB8IEQx
MCB8IFtgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0
ZXMubWQpIHwgMTQuIENvbnRyYWN0IHByb21vdGlvbiBnYXRlcyB8IFtgMTQtY29udHJhY3QtcHJv
bW90aW9uLWdhdGVzYF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMxNC1jb250cmFjdC1w
cm9tb3Rpb24tZ2F0ZXMpIHwgbGluZXMgMjE14oCTMjMxICgxMTA2IGJ5dGVzKSB8IGAwZmE3Nzc3
N2I3ZjhhZTg1YmM0ZTA2MDgyNDU0YjI0MzY3N2Q4OWQwNzBmZjRjNTEwYmQ5ZTE0OGI2MWIyZDQz
YCB8IFtgLi4vZ2F0ZXMvY29udHJhY3QtcHJvbW90aW9uLm1kYF0oLi4vZ2F0ZXMvY29udHJhY3Qt
cHJvbW90aW9uLm1kKSB8IGdhdGUgfCBjYW5vbmljYWwgZGVzdGluYXRpb247IHNvdXJjZSBjb21w
YXRpYmlsaXR5IGFuY2hvciB8IG5vbmUgfCByZXN0b3JlIHRoZSBleGFjdCAxMTc5LWJ5dGUgc291
cmNlIHNwYW4gYXQgYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMTAtY2FuY2VsLW91dGNvbWUt
Y2F0ZWdvcmllc2AsIHRoZSBleGFjdCAxNjc1LWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRy
YWN0cy1hbmQtZ2F0ZXMubWQjMTItZmluYWwtcmVjZWlwdC1pbW11dGFibGUtcG9saWN5c25hcHNo
b3R2My1hY2NlcHRhbmNlLXJ1bGVzYCwgdGhlIGV4YWN0IDE5MzAtYnl0ZSBzb3VyY2Ugc3BhbiBh
dCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMxMy1kaXNwYXRjaC1uYXJyb3dpbmctbW9ub3Rv
bmljaXR5LXJ1bGVzYCwgYW5kIHRoZSBleGFjdCAxMTA2LWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0
LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMTQtY29udHJhY3QtcHJvbW90aW9uLWdhdGVzYDsgcmVt
b3ZlIGBjb250cmFjdHMvY2FuY2VsLW91dGNvbWUtY2F0ZWdvcmllcy5tZGAsIGBnYXRlcy9maW5h
bC1yZWNlaXB0LWltbXV0YWJsZS1wb2xpY3ktc25hcHNob3QtdjMtYWNjZXB0YW5jZS5tZGAsIGBn
YXRlcy9kaXNwYXRjaC1uYXJyb3dpbmctbW9ub3RvbmljaXR5Lm1kYCwgYW5kIGBnYXRlcy9jb250
cmFjdC1wcm9tb3Rpb24ubWRgOyByZW1vdmUgdGhlIGV4YWN0IGBDYW5jZWwgb3V0Y29tZSBjYXRl
Z29yaWVzYCwgYEZpbmFsLXJlY2VpcHQgaW1tdXRhYmxlIFBvbGljeVNuYXBzaG90VjMgYWNjZXB0
YW5jZSBydWxlc2AsIGBEaXNwYXRjaCBuYXJyb3dpbmcgbW9ub3RvbmljaXR5IHJ1bGVzYCwgYW5k
IGBDb250cmFjdCBwcm9tb3Rpb24gZ2F0ZXNgIHJvd3MgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsg
cmVtb3ZlIGFsbCBmb3VyIEQxMCBsZWRnZXIgZW50cmllcyBmb3IgdGhpcyBiYXRjaDsgbGVhdmUg
YExhdW5jaFRpbWVTZWNyZXRIYW5kb2ZmVjFgLCBgU3VwZXJ2aXNvciBpZGVtcG90ZW5jeSBhbmQg
cmVzdGFydCBydWxlc2AsIGBEaWZmZXJlbnRpYWwtYmFzZWxpbmUgdHJhbnNpdGlvbiBnYXRlYCwg
YFJlbWFpbmluZyBSMi0yIHNhbWUtcHJvY2VzcyBjYXJyaWVyIGNsb3N1cmVgLCBhbGwgRDMvRDXi
gJNEMTAgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9gLCBhbmQg
ZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBzZXZlbi1wYXRoIGZlbmNlIHVuY2hhbmdlZCB8CiB8IEQx
MSB8IFtgMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWRgXSguLi8wNS1kZWJ1Zy1yZWdyZXNz
aW9uLWxlZGdlci5tZCkgfCBDYW5vbmljYWwgaXNzdWUgbGVkZ2VyICsgQTEuMmEtV0IgZ2F0ZSBh
c3NpZ25tZW50IHwgW2BjYW5vbmljYWwtaXNzdWUtbGVkZ2VyYF0oLi4vMDUtZGVidWctcmVncmVz
c2lvbi1sZWRnZXIubWQjY2Fub25pY2FsLWlzc3VlLWxlZGdlcik7IFtgYTEyYS13Yi1nYXRlLWFz
c2lnbm1lbnRgXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNhMTJhLXdiLWdhdGUt
YXNzaWdubWVudCkgfCBsaW5lcyAxNuKAkzk3ICg0MTQ5MiBieXRlcykgfCBgMTkxYTYyZWEwYzc3
MDY0OTQxOTQ3MjgwZDg3YzUzOTE2M2FiNjliNzcyYTgwMThmNmIyZjRlM2QzZTVhYWUzZWAgfCBb
YC4uL2V2aWRlbmNlL2Nhbm9uaWNhbC1pc3N1ZS1sZWRnZXIubWRgXSguLi9ldmlkZW5jZS9jYW5v
bmljYWwtaXNzdWUtbGVkZ2VyLm1kKSB8IGV2aWRlbmNlL3JlZ3Jlc3Npb24gfCBjYW5vbmljYWwg
ZGVzdGluYXRpb247IHNvdXJjZSBjb21wYXRpYmlsaXR5IGFuY2hvcnMgZm9yIGJvdGggcHJlc2Vy
dmVkIHJvb3QgaGVhZGluZ3MgfCBub25lOyBleGFjdCA0MTQ5Mi1ieXRlIHNvdXJjZSBib2R5IHBy
ZXNlcnZlZCBiZXR3ZWVuIG93bmVyIGJvdW5kYXJ5IG1hcmtlcnMgfCByZXN0b3JlIHRoZSBleGFj
dCA0MTQ5Mi1ieXRlIHNvdXJjZSBib2R5IGF0IGAwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5t
ZCNjYW5vbmljYWwtaXNzdWUtbGVkZ2VyYCBhbmQgYCNhMTJhLXdiLWdhdGUtYXNzaWdubWVudGAg
aW4gb3JpZ2luYWwgb3JkZXI7IHJlbW92ZSBgZXZpZGVuY2UvY2Fub25pY2FsLWlzc3VlLWxlZGdl
ci5tZGA7IHJlbW92ZSB0aGUgc29sZSBCYXRjaC0xIHJvdyBmcm9tIGBldmlkZW5jZS9SRUFETUUu
bWRgIGFuZCBkZWxldGUgYGV2aWRlbmNlL1JFQURNRS5tZGAgaWYgbm8gbmF2aWdhdGlvbiByb3dz
IHJlbWFpbjsgcmVtb3ZlIHRoZSBleGFjdCBgQ2Fub25pY2FsIGlzc3VlIGxlZGdlcmAgcm93IGZy
b20gYGluZGV4L1JFQURNRS5tZGA7IHJlbW92ZSB0aGlzIEQxMSBsZWRnZXIgZW50cnk7IGxlYXZl
IEQz4oCTRDEwIG93bmVycywgYGluZGV4L2N1cnJlbnQubWRgLCBhbmQgYHJldmlldy1jb250cm9s
L2AgdW5jaGFuZ2VkIHwKK3wgRDExIHwgW2AwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZGBd
KC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kKSB8IENsb3Nlb3V0IHJ1bGUgfCBbYGNs
b3Nlb3V0LXJ1bGVgXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNjbG9zZW91dC1y
dWxlKSB8IGxpbmVzIDE5M+KAkzIwMSAoMzQ3IGJ5dGVzKSB8IGBmODExNDJhMmJmNmQwY2JkNmIz
MWEyNzAxMTI2MGRhMjUxMDhjYzUxOGI1ODJkZDFiZTgxMjY4Zjk0NjM1NDYwYCB8IFtgLi4vZXZp
ZGVuY2UvY2xvc2VvdXQtYW5kLXJldmlldy1jYWxpYnJhdGlvbi5tZGBdKC4uL2V2aWRlbmNlL2Ns
b3Nlb3V0LWFuZC1yZXZpZXctY2FsaWJyYXRpb24ubWQpIHwgZXZpZGVuY2UvcmVncmVzc2lvbiB8
IGNhbm9uaWNhbCBjb21wb3VuZCBkZXN0aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgYW5j
aG9yOyBzZXBhcmF0ZSBub25jb250aWd1b3VzIHByb3ZlbmFuY2UgcHJlc2VydmVkIHwgbm9uZTsg
ZXhhY3QgMzQ3LWJ5dGUgc291cmNlIGJvZHkgcHJlc2VydmVkIGJldHdlZW4gdGhlIGBjbG9zZW91
dC1ydWxlYCBvd25lciBib3VuZGFyeSBtYXJrZXJzIGluIHRoZSBjb21wb3VuZCBldmlkZW5jZSBv
d25lciB8IHJlc3RvcmUgdGhlIGV4YWN0IDM0Ny1ieXRlIHNvdXJjZSBib2R5IGF0IGAwNS1kZWJ1
Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNjbG9zZW91dC1ydWxlYCBhbmQgdGhlIGV4YWN0IDEzODIt
Ynl0ZSBzb3VyY2UgYm9keSBhdCBgMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjcmV2aWV3
LXByb2Nlc3MtY2FsaWJyYXRpb25gIGluIG9yaWdpbmFsIG9yZGVyOyByZW1vdmUgYGV2aWRlbmNl
L2Nsb3Nlb3V0LWFuZC1yZXZpZXctY2FsaWJyYXRpb24ubWRgOyByZW1vdmUgdGhlIHNvbGUgQmF0
Y2gtMiByb3cgZnJvbSBgZXZpZGVuY2UvUkVBRE1FLm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgQ2xv
c2VvdXQgcnVsZSArIFJldmlldy1wcm9jZXNzIGNhbGlicmF0aW9uYCByb3cgZnJvbSBgaW5kZXgv
UkVBRE1FLm1kYDsgcmVtb3ZlIGJvdGggRDExIGxlZGdlciBlbnRyaWVzIGZvciB0aGlzIGJhdGNo
OyBsZWF2ZSBCYXRjaCAxIHBsdXMgRDPigJNEMTAgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAs
IGByZXZpZXctY29udHJvbC9gLCBhbmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmaXZlLXBhdGgg
ZmVuY2UgdW5jaGFuZ2VkIHwKK3wgRDExIHwgW2AwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5t
ZGBdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kKSB8IFJldmlldy1wcm9jZXNzIGNh
bGlicmF0aW9uIHwgW2ByZXZpZXctcHJvY2Vzcy1jYWxpYnJhdGlvbmBdKC4uLzA1LWRlYnVnLXJl
Z3Jlc3Npb24tbGVkZ2VyLm1kI3Jldmlldy1wcm9jZXNzLWNhbGlicmF0aW9uKSB8IGxpbmVzIDMz
MuKAkzM1MyAoMTM4MiBieXRlcykgfCBgMjg0Mjg0YWM1MDdhOWJkYjMyNjNmMmMxOGJjNjE1YzA1
ZGYzZmQyM2VkMzNhN2E2NzQyZjg1ZTI5ZmJlMTc1ZWAgfCBbYC4uL2V2aWRlbmNlL2Nsb3Nlb3V0
LWFuZC1yZXZpZXctY2FsaWJyYXRpb24ubWRgXSguLi9ldmlkZW5jZS9jbG9zZW91dC1hbmQtcmV2
aWV3LWNhbGlicmF0aW9uLm1kKSB8IGV2aWRlbmNlL3JlZ3Jlc3Npb24gfCBjYW5vbmljYWwgY29t
cG91bmQgZGVzdGluYXRpb247IHNvdXJjZSBjb21wYXRpYmlsaXR5IGFuY2hvcjsgc2VwYXJhdGUg
bm9uY29udGlndW91cyBwcm92ZW5hbmNlIHByZXNlcnZlZCB8IG5vbmU7IGV4YWN0IDEzODItYnl0
ZSBzb3VyY2UgYm9keSBwcmVzZXJ2ZWQgYmV0d2VlbiB0aGUgYHJldmlldy1wcm9jZXNzLWNhbGli
cmF0aW9uYCBvd25lciBib3VuZGFyeSBtYXJrZXJzIGluIHRoZSBjb21wb3VuZCBldmlkZW5jZSBv
d25lcjsgdGhlIGRldmVsb3BtZW50LXJldmlldyBjb250cmFjdCByZWxhdGlvbnNoaXAgcmVtYWlu
cyByZWZlcmVuY2Utb25seSBhbmQgZG9lcyBub3QgcmVvcGVuIEQzIHwgcmVzdG9yZSB0aGUgZXhh
Y3QgMzQ3LWJ5dGUgc291cmNlIGJvZHkgYXQgYDA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1k
I2Nsb3Nlb3V0LXJ1bGVgIGFuZCB0aGUgZXhhY3QgMTM4Mi1ieXRlIHNvdXJjZSBib2R5IGF0IGAw
NS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNyZXZpZXctcHJvY2Vzcy1jYWxpYnJhdGlvbmAg
aW4gb3JpZ2luYWwgb3JkZXI7IHJlbW92ZSBgZXZpZGVuY2UvY2xvc2VvdXQtYW5kLXJldmlldy1j
YWxpYnJhdGlvbi5tZGA7IHJlbW92ZSB0aGUgc29sZSBCYXRjaC0yIHJvdyBmcm9tIGBldmlkZW5j
ZS9SRUFETUUubWRgOyByZW1vdmUgdGhlIGV4YWN0IGBDbG9zZW91dCBydWxlICsgUmV2aWV3LXBy
b2Nlc3MgY2FsaWJyYXRpb25gIHJvdyBmcm9tIGBpbmRleC9SRUFETUUubWRgOyByZW1vdmUgYm90
aCBEMTEgbGVkZ2VyIGVudHJpZXMgZm9yIHRoaXMgYmF0Y2g7IGxlYXZlIEJhdGNoIDEgcGx1cyBE
M+KAk0QxMCBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYHJldmlldy1jb250cm9sL2AsIGFu
ZCBldmVyeSBwYXRoIG91dHNpZGUgdGhlIGZpdmUtcGF0aCBmZW5jZSB1bmNoYW5nZWQgfApkaWZm
IC0tZ2l0IGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2V2aWRlbmNlL2Nsb3Nlb3V0
LWFuZC1yZXZpZXctY2FsaWJyYXRpb24ubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0
b3IvZXZpZGVuY2UvY2xvc2VvdXQtYW5kLXJldmlldy1jYWxpYnJhdGlvbi5tZApuZXcgZmlsZSBt
b2RlIDEwMDY0NAppbmRleCAwMDAwMDAwMDAuLjE5NzRiODhmZQotLS0gL2Rldi9udWxsCisrKyBi
L2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9ldmlkZW5jZS9jbG9zZW91dC1hbmQtcmV2
aWV3LWNhbGlicmF0aW9uLm1kCkBAIC0wLDAgKzEsNTEgQEAKKyoqS2luZDoqKiBldmlkZW5jZS9y
ZWdyZXNzaW9uCisqKlN0YXR1czoqKiBjYW5vbmljYWwKKyoqQ2Fub25pY2FsIGZvcjoqKiBleGFj
dCBleHRyYWN0ZWQgYENsb3Nlb3V0IHJ1bGVgIGFuZCBgUmV2aWV3LXByb2Nlc3MgY2FsaWJyYXRp
b25gIGJvZGllcworKipBdXRob3JpdHkgc2NvcGU6KiogZG9jdW1lbnRhdGlvbiBkZWNvbXBvc2l0
aW9uIG9ubHk7IGV4YWN0IGV4dHJhY3RlZCBldmlkZW5jZS9yZWdyZXNzaW9uIHNvdXJjZSBib2Rp
ZXMgb25seQorKipTb3VyY2UgcHJvdmVuYW5jZToqKiBleHRyYWN0ZWQgYnl0ZS1mb3ItYnl0ZSBm
cm9tIFtgLi4vMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjY2xvc2VvdXQtcnVsZWBdKC4u
LzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2Nsb3Nlb3V0LXJ1bGUpLCBiYXNlbGluZSBs
aW5lcyAxOTPigJMyMDEgaW5jbHVzaXZlIChgMzQ3YCBieXRlcyksIGFuZCBbYC4uLzA1LWRlYnVn
LXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI3Jldmlldy1wcm9jZXNzLWNhbGlicmF0aW9uYF0oLi4vMDUt
ZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjcmV2aWV3LXByb2Nlc3MtY2FsaWJyYXRpb24pLCBi
YXNlbGluZSBsaW5lcyAzMzLigJMzNTMgaW5jbHVzaXZlIChgMTM4MmAgYnl0ZXMpOyBlYWNoIGV4
YWN0IHNvdXJjZSBib2R5IGlzIHByZXNlcnZlZCBpbmRlcGVuZGVudGx5IGJldHdlZW4gaXRzIG1h
dGNoaW5nIGJvdW5kYXJ5IG1hcmtlcnMgYmVsb3c7IHRoZSBib2RpZXMgYXJlIG5vbmNvbnRpZ3Vv
dXMgaW4gdGhlIHNvdXJjZSBmaWxlIGFuZCBhcmUgbm90IHJlcHJlc2VudGVkIGFzIG9uZSBjb250
aWd1b3VzIGV4dHJhY3Rpb24KKyoqQ2xvc2VvdXQgcnVsZSBzb3VyY2UgU0hBLTI1NjoqKiBgZjgx
MTQyYTJiZjZkMGNiZDZiMzFhMjcwMTEyNjBkYTI1MTA4Y2M1MThiNTgyZGQxYmU4MTI2OGY5NDYz
NTQ2MGAKKyoqUmV2aWV3LXByb2Nlc3MgY2FsaWJyYXRpb24gc291cmNlIFNIQS0yNTY6KiogYDI4
NDI4NGFjNTA3YTliZGIzMjYzZjJjMThiYzYxNWMwNWRmM2ZkMjNlZDMzYTdhNjc0MmY4NWUyOWZi
ZTE3NWVgCisqKlN1cGVyc2VkZXM6KiogY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFj
dGVkIHNvdXJjZSBib2RpZXM7IHRoZSBzb3VyY2UgaGVhZGluZ3MgcmVtYWluIGNvbXBhdGliaWxp
dHkgYW5jaG9ycworKipTdXBlcnNlZGVkIGJ5OioqIG5vbmUKKyoqUmVsYXRpb25zaGlwIHRvIEQz
IGdvdmVybmFuY2U6KiogcmVmZXJlbmNlLW9ubHkgdG8gW2AuLi9jb250cmFjdHMvZGV2ZWxvcG1l
bnQtcmV2aWV3LWFuZC1yZW1lZGlhdGlvbi1jb250cmFjdC5tZGBdKC4uL2NvbnRyYWN0cy9kZXZl
bG9wbWVudC1yZXZpZXctYW5kLXJlbWVkaWF0aW9uLWNvbnRyYWN0Lm1kKTsgdGhpcyBvd25lciBk
b2VzIG5vdCByZW9wZW4gRDMgZ292ZXJuYW5jZQorKipBdXRob3JpdHkgYm91bmRhcnk6KiogdGhp
cyBldmlkZW5jZS9yZWdyZXNzaW9uIG93bmVyIGRvZXMgbm90IHNhdGlzZnkgYSBnYXRlLCBjaGFu
Z2UgYSBwcm9vZiByZXN1bHQsIGF1dGhvcml6ZSBpbXBsZW1lbnRhdGlvbiwgcHJvbW90aW9uLCBy
ZW1lZGlhdGlvbiwgb3IgZGlzcGF0Y2gKKyoqUHJvamVjdGlvbiBjb25zdW1lcnM6KiogW2BSRUFE
TUUubWRgXShSRUFETUUubWQpLCBbYC4uL2luZGV4L1JFQURNRS5tZGBdKC4uL2luZGV4L1JFQURN
RS5tZCkKKworIyBDbG9zZW91dCBhbmQgcmV2aWV3IGNhbGlicmF0aW9uCisKKzwhLS0gZXhhY3Qt
ZXh0cmFjdGVkLWJvZHk6Y2xvc2VvdXQtcnVsZTpzdGFydCAtLT4KKyMjIENsb3Nlb3V0IHJ1bGUK
KworQW4gaW1wbGVtZW50YXRpb24gUFIgbWF5IG1hcmsgYSBsZWRnZXIgcm93IHJlc29sdmVkIG9u
bHkgd2hlbjoKKworMS4gaXRzIG93bmluZyBjcm9zc3dhbGsgc2VhbSBoYXMgdGhlIGNvcnJlY3Qg
b3duZXIgYW5kIGNhbGwgcGF0aCBmb3IgdGhhdCBiZWhhdmlvcjsKKzIuIHRoZSBuYW1lZCBwZXJt
YW5lbnQgZ2F0ZSBwYXNzZXMgb24gdGhlIHJlYWwgcGF0aDsKKzMuIGFkamFjZW50IHJlc29sdmVk
IGJhc2VsaW5lcyByZW1haW4gZ3JlZW47IGFuZAorNC4gdGhlIGV2aWRlbmNlIGRpc3Rpbmd1aXNo
ZXMgZHVyYWJsZSBzdWNjZXNzIGZyb20gdHJhbnNwb3J0L3Byb2Nlc3Mgc3VjY2Vzcy4KKworPCEt
LSBleGFjdC1leHRyYWN0ZWQtYm9keTpjbG9zZW91dC1ydWxlOmVuZCAtLT4KKworPCEtLSBleGFj
dC1leHRyYWN0ZWQtYm9keTpyZXZpZXctcHJvY2Vzcy1jYWxpYnJhdGlvbjpzdGFydCAtLT4KKyMj
IFJldmlldy1wcm9jZXNzIGNhbGlicmF0aW9uCisKK1JQNCBleHBvc2VkIGEgcHJvY2VzcyBmYWls
dXJlIHdpdGhvdXQgZXhwb3NpbmcgYSBwcm9kdWN0IG9yIHRlc3QgcmVncmVzc2lvbjogYSBibGFu
a2V0IHJlcXVpcmVtZW50Citmb3IgZm91ciBjbGVhbiByZXZpZXdzIGFsbG93ZWQgZmluZGluZ3Mg
YWJvdXQgYWdlbnQtY3JlYXRlZCBjYWNoZS1vbmx5IG9yY2hlc3RyYXRpb24gdG8gZXhwYW5kIHRo
ZQorcHJvZHVjdC1wcm9vZiBhY2NlcHRhbmNlIHN1cmZhY2UuIFJlcGVhdGVkIGZpeC9yZXZpZXcg
YXR0ZW1wdHMgdGhlbiBpbXByb3ZlZCBiZXNwb2tlIGV2aWRlbmNlCit0b29saW5nIHJhdGhlciB0
aGFuIHRoZSBzZWxlY3RlZCBTdWJzdHJhdGUgb3V0Y29tZS4gSHVtYW4gZGlzcG9zaXRpb24gY29y
cmVjdGx5IHByZXNlcnZlZCB0aGUgcmF3CitgUkVRVUVTVF9DSEFOR0VTYCByZXZpZXcgd2hpbGUg
YWNjZXB0aW5nIHRoZSBpbmRlcGVuZGVudGx5IGV2YWx1YWJsZSBwcm9kdWN0IHByb29mLgorCitU
aGUgcHJvc3BlY3RpdmUgY29ycmVjdGlvbiBpcyBvd25lZCBieSB0aGUgZGV2ZWxvcG1lbnQtcmV2
aWV3IGNvbnRyYWN0IGluIGAwNGA6CisKKy0gYFAxYC9gUDJgIGJsb2NrIG9ubHkgb24gZGVtb25z
dHJhdGVkIGltcGFjdCB0byB0aGUgc2VsZWN0ZWQgY29udHJhY3QsIGdhdGUsIHNjb3BlLCBvciBj
b21wbGV0aW9uCisgIGNsYWltOworLSBvbmUgZGlzY292ZXJ5IGN5Y2xlLCBvbmUgY29uc29saWRh
dGVkIHJlbWVkaWF0aW9uLCBvbmUgY2xvc3VyZSBjeWNsZSwgYW5kIGF0IG1vc3QgdHdvIGRpcmVj
dGx5CisgIGNhdXNhbCBzdXBwbGVtZW50YWwgY3ljbGVzIGJvdW5kIGF1dG9tYXRpYyB3b3JrOwor
LSBgQ0xFQU5gIGlzIHRlcm1pbmFsLCBtZWNoYW5pY2FsLW9ubHkgZGVsdGFzIGRvIG5vdCBjcmVh
dGUgcmV2aWV3IGN5Y2xlcywgYW5kIHVucmVsYXRlZCBvciBleHBhbmRlZAorICBibG9ja2VycyBz
dG9wIGZvciBhdXRob3JpdHkgcmF0aGVyIHRoYW4gd2lkZW5pbmcgc2NvcGU7IGFuZAorLSB2YWxp
ZCBub24tYmxvY2tpbmcgcmV2aWV3L3Byb2Nlc3MgZGVidCBpcyByZXRhaW5lZCBpbiBgMDZgLCBz
ZXBhcmF0ZSBmcm9tIHRoaXMgcHJvZHVjdCByZWdyZXNzaW9uCisgIGxlZGdlci4KKworVGhlIHRo
cmVlIFJQNCBwZXJzaXN0ZW5jZSBmaW5kaW5ncyBhcmUgcmVnaXN0ZXJlZCBhcyBgUlItUkYtMDAw
MWAgdGhyb3VnaCBgUlItUkYtMDAwM2AuIFRoZWlyIHJhdworcmV2aWV3IGZpbGVzLCBoYXNoZXMs
IGFuZCBvcmlnaW5hbCB2ZXJkaWN0IHJlbWFpbiB1bmNoYW5nZWQuIFRoaXMgY2FsaWJyYXRpb24g
YXV0aG9yaXplcyBubworY29udHJvbGxlci9zdXBlcnZpc29yIHJlbWVkaWF0aW9uIGFuZCBjaGFu
Z2VzIG5vIFJQMy9SUDQvUlA1IHByb29mIHJlc3VsdC4KKzwhLS0gZXhhY3QtZXh0cmFjdGVkLWJv
ZHk6cmV2aWV3LXByb2Nlc3MtY2FsaWJyYXRpb246ZW5kIC0tPgpgYGAKClZhbGlkYXRpb24gbG9n
IChTSEEtMjU2IDc0ZjhkNWM4YzAwM2JiZTRkZWY1ZmQwNGZmOGY2MDkzYTdiMTFiMmY5NjY2NWE3
MjUzNDMwNjhlZWI2ZWI5ODIpOgpgYGB0ZXh0ClBSRUNIRUNLX0hFQUQgZDAyMTY4YWRhMDc5Yjc5
YzNhMTI3MDkwNThmNGRiODkzYmRkYWJkYgpQUkVDSEVDS19UUkVFIGZlZjhlZTc4NDlmNDQ2YTg5
OGNiMjUwNzkyY2IzZGRlMGE5MDAwNzkKUFJFQ0hFQ0tfQlJBTkNIIChkZXRhY2hlZCkKU1RBVFVT
X0xJTkVTX1NUQVJUCiBNIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci8wNS1kZWJ1Zy1y
ZWdyZXNzaW9uLWxlZGdlci5tZAogTSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZXZp
ZGVuY2UvUkVBRE1FLm1kCiBNIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9S
RUFETUUubWQKIE0gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRy
YWN0aW9uLWxlZGdlci5tZAo/PyBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZXZpZGVu
Y2UvY2xvc2VvdXQtYW5kLXJldmlldy1jYWxpYnJhdGlvbi5tZApTVEFUVVNfTElORVNfRU5EClBB
VEhfRkVOQ0VfT0sgNSBjaGFuZ2VkIHBhdGhzIGV4YWN0bHkgbWF0Y2ggYWxsb3dlZCBmZW5jZTsg
bm8gc3RhZ2VkIGZpbGVzCkdJVF9ESUZGX0NIRUNLX09LCk5FV19GSUxFX0RJRkZfQ0hFQ0tfT0sg
bGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2V2aWRlbmNlL2Nsb3Nlb3V0LWFuZC1yZXZp
ZXctY2FsaWJyYXRpb24ubWQKUFJFX0VESVRfU1BBTl9NQVRDSF9PSyBjbG9zZW91dF9ydWxlIGJ5
dGVzPTM0NyBzaGEyNTY9ZjgxMTQyYTJiZjZkMGNiZDZiMzFhMjcwMTEyNjBkYTI1MTA4Y2M1MThi
NTgyZGQxYmU4MTI2OGY5NDYzNTQ2MCByZXZpZXdfcHJvY2Vzc19jYWxpYnJhdGlvbiBieXRlcz0x
MzgyIHNoYTI1Nj0yODQyODRhYzUwN2E5YmRiMzI2M2YyYzE4YmM2MTVjMDVkZjNmZDIzZWQzM2E3
YTY3NDJmODVlMjlmYmUxNzVlCk9XTkVSX0JPRFlfRVFVQUxfT0sgY2xvc2VvdXRfcnVsZSBieXRl
cz0zNDcgc2hhMjU2PWY4MTE0MmEyYmY2ZDBjYmQ2YjMxYTI3MDExMjYwZGEyNTEwOGNjNTE4YjU4
MmRkMWJlODEyNjhmOTQ2MzU0NjAKT1dORVJfQk9EWV9FUVVBTF9PSyByZXZpZXdfcHJvY2Vzc19j
YWxpYnJhdGlvbiBieXRlcz0xMzgyIHNoYTI1Nj0yODQyODRhYzUwN2E5YmRiMzI2M2YyYzE4YmM2
MTVjMDVkZjNmZDIzZWQzM2E3YTY3NDJmODVlMjlmYmUxNzVlClJPT1RfTk9OU1BBTl9TVEFCSUxJ
VFlfT0sgcmVwbGFjaW5nIHRoZSB0d28gY29tcGF0aWJpbGl0eSBzdHVicyB3aXRoIHRoZSBmcm96
ZW4gYm9kaWVzIHJlY29uc3RydWN0cyBIRUFEIGV4YWN0bHkKUk9PVF9TVFVCX0FORF9IRUFESU5H
X09LIGxlZ2FjeSByb290IGhlYWRpbmdzIHByZXNlcnZlZCBvbmNlIGVhY2ggd2l0aCBvd25lci1h
bmNob3IgcG9pbnRlcnM7IG93bmVyIGhlYWRpbmdzIHByZXNlcnZlZCBvbmNlIGVhY2gKQk9EWV9M
SVRFUkFMX0JPVU5EQVJZX09LIG9yZGVyZWQgYnVsbGV0cywgbGl0ZXJhbHMsIElEcywgbmVnYXRp
dmUgcmVxdWlyZW1lbnRzLCBleGNlcHRpb25zLCBuby1yZW1lZGlhdGlvbi9wcm9vZi1yZXN1bHQg
Ym91bmRhcnksIGFuZCBzZXBhcmF0ZSBub25jb250aWd1b3VzIHByb3ZlbmFuY2UgcHJlc2VydmVk
ClJPV19DT1VOVF9PSyBldmlkZW5jZV9uYXZpZ2F0aW9uPTEgZ2xvYmFsX2luZGV4PTEgZDExX2xl
ZGdlcl9iYXRjaDI9MgpPV05FUl9SRVZJRVdfU1RBQklMSVRZX09LIHNuYXBzaG90X3BhdGhzPTI1
MQpMSU5LX1NDQU5fU1VNTUFSWSBjaGVja2VkPTE5MTYgZXhjbHVkZWQ9NCBmYWlsdXJlcz0wClBB
VENIX1dSSVRURU4gL3ByaXZhdGUvdG1wL2QxMS1jbG9zZW91dC1yZXZpZXctY2FsaWJyYXRpb24t
Y2FuZGlkYXRlLnBhdGNoIHNoYTI1Nj1iZGIyMjBkMTBmZjkxODdjNjJhYWRkYTY4MTE2ZjYyMzgy
NWVhNmJiMjRjNzIxZmFkZTcxYjYxZmViYjg4ODJhClBBVENIX0FQUExZX09LIGZvcndhcmRfYW5k
X3JldmVyc2VfYXBwbHlfbWF0Y2hfY2xlYW5fYmFzZWxpbmUKRklMRV9TSEEgbGxtLWxhc3QtbWls
ZS9ydW50aW1lLXJlZmFjdG9yLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kIDY0NzJmOGM1
NDFlYzkwZGYwMzk3NzI4NzBkY2M1ODhiNmNiMmQ2NTdlYTU3ZTc0NmQ3ZTY3NmM3MDM4OTRlYWUK
RklMRV9TSEEgbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2V2aWRlbmNlL1JFQURNRS5t
ZCBlNDNlMDJhNjViNzUzODNkM2FmMTBjNmZhMDlkZmQ0MmExYWYwNmFmNjQ2YmZhZmJlMTkzZjRk
YWY0YmJiMGRiCkZJTEVfU0hBIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9ldmlkZW5j
ZS9jbG9zZW91dC1hbmQtcmV2aWV3LWNhbGlicmF0aW9uLm1kIGE3OTk4MmYxMWY2NmZjODI2ZjFm
MzI5MDQ5ODg4MjZhM2NmNmJhNTA4YjdlMzJmNWNkNDE5YzU4MTc5MDQyYjYKRklMRV9TSEEgbGxt
LWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZCBjYTkzMzI3NzNhMDM4
N2M2YWUxZDM4MDZhMzE5ZjU2YTlkYmE4ZTllMTFjYjE0OWRmOTMyMTkxZmJkNTg5M2NhCkZJTEVf
U0hBIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1s
ZWRnZXIubWQgODgyM2IyZWM1NWEyMDljMzljNzU2NTZkMTY1NjBlYWMyZTllZGRjZTUzZDRjYzNl
MWIyODU3ZDQxNmY0ZjdlMQpGSUxFX1NIQSAvcHJpdmF0ZS90bXAvZDExLWNsb3Nlb3V0LXJldmll
dy1jYWxpYnJhdGlvbi1jYW5kaWRhdGUucGF0Y2ggYmRiMjIwZDEwZmY5MTg3YzYyYWFkZGE2ODEx
NmY2MjM4MjVlYTZiYjI0YzcyMWZhZGU3MWI2MWZlYmI4ODgyYQpGSUxFX1NIQSAvcHJpdmF0ZS90
bXAvZDExLWNsb3Nlb3V0LXJldmlldy1jYWxpYnJhdGlvbi1jYW5kaWRhdGUucGF0Y2guc2hhMjU2
IDRjOWE5YWE4ODUyMTRhNTg3YzY4ZWMyY2VkNzgxYzE0MjMwZmI0ZjgyNzc2MWJiYmNkNzdhYmM2
N2RkMjk4OTIKYGBgCg==
```

</details>

## Preserved review answer

<details>
<summary>Initial-range review answer (rendered copy)</summary>

<pre>
No qualifying findings.

I reviewed the complete bounded D11 Batch 2 candidate against the stated P1/P2 blocking threshold and five-path fence. The two noncontiguous extraction subjects, legacy-anchor requirements, single-owner authority boundary, navigation requirements, and atomic rollback obligations are explicitly defined by the packet.&#32;


The candidate satisfies those requirements:

The root retains ## Closeout rule and ## Review-process calibration as compatibility anchors pointing to the corresponding headings in the new compound owner.&#32;


The new owner expressly limits itself to documentation/evidence-regression authority, records the two spans separately, identifies their individual hashes, points reference-only to the existing D3 development-review contract, and explicitly disclaims gate satisfaction, proof-result changes, implementation, promotion, remediation, and dispatch authority.&#32;


Both preserved bodies are byte-exact within their respective marker-delimited regions, including the closeout conditions and the RP4/P1/P2/CLEAN/RR-RF-0001–0003/no-remediation/no-proof-result-change material.&#32;


The evidence navigation and global index each add one bounded, non-authoritative projection to the new owner.&#32;


The two ledger rows preserve separate provenance for the 347-byte and 1382-byte source spans while specifying the same atomic rollback: restore both bodies, remove the compound owner and the exact Batch-2 navigation/index additions, remove both Batch-2 ledger rows, and preserve Batch 1 and the excluded prior owners/artifacts.&#32;


The supplied validation independently records exact owner-body byte/hash equality, root reconstruction, unique anchors, required literal/boundary preservation, the expected row counts, prior-owner stability, zero failures across 1,916 supported local links/fragments, and clean forward/reverse patch application.&#32;


I also checked the preserved phrase referring to the development-review contract “in 04.” In this candidate it does not create a qualifying authority defect: changing it would violate the byte-exact preservation requirement, while the new owner&#x27;s enclosing metadata explicitly identifies the current canonical D3 contract and makes the relationship reference-only.&#32;


VERDICT: APPROVED
</pre>

</details>

<details>
<summary>Initial-range review answer exact bytes (base64 UTF-8)</summary>

```text
Tm8gcXVhbGlmeWluZyBmaW5kaW5ncy4KCkkgcmV2aWV3ZWQgdGhlIGNvbXBsZXRlIGJvdW5kZWQg
RDExIEJhdGNoIDIgY2FuZGlkYXRlIGFnYWluc3QgdGhlIHN0YXRlZCBQMS9QMiBibG9ja2luZyB0
aHJlc2hvbGQgYW5kIGZpdmUtcGF0aCBmZW5jZS4gVGhlIHR3byBub25jb250aWd1b3VzIGV4dHJh
Y3Rpb24gc3ViamVjdHMsIGxlZ2FjeS1hbmNob3IgcmVxdWlyZW1lbnRzLCBzaW5nbGUtb3duZXIg
YXV0aG9yaXR5IGJvdW5kYXJ5LCBuYXZpZ2F0aW9uIHJlcXVpcmVtZW50cywgYW5kIGF0b21pYyBy
b2xsYmFjayBvYmxpZ2F0aW9ucyBhcmUgZXhwbGljaXRseSBkZWZpbmVkIGJ5IHRoZSBwYWNrZXQu
IAoKClRoZSBjYW5kaWRhdGUgc2F0aXNmaWVzIHRob3NlIHJlcXVpcmVtZW50czoKClRoZSByb290
IHJldGFpbnMgIyMgQ2xvc2VvdXQgcnVsZSBhbmQgIyMgUmV2aWV3LXByb2Nlc3MgY2FsaWJyYXRp
b24gYXMgY29tcGF0aWJpbGl0eSBhbmNob3JzIHBvaW50aW5nIHRvIHRoZSBjb3JyZXNwb25kaW5n
IGhlYWRpbmdzIGluIHRoZSBuZXcgY29tcG91bmQgb3duZXIuIAoKClRoZSBuZXcgb3duZXIgZXhw
cmVzc2x5IGxpbWl0cyBpdHNlbGYgdG8gZG9jdW1lbnRhdGlvbi9ldmlkZW5jZS1yZWdyZXNzaW9u
IGF1dGhvcml0eSwgcmVjb3JkcyB0aGUgdHdvIHNwYW5zIHNlcGFyYXRlbHksIGlkZW50aWZpZXMg
dGhlaXIgaW5kaXZpZHVhbCBoYXNoZXMsIHBvaW50cyByZWZlcmVuY2Utb25seSB0byB0aGUgZXhp
c3RpbmcgRDMgZGV2ZWxvcG1lbnQtcmV2aWV3IGNvbnRyYWN0LCBhbmQgZXhwbGljaXRseSBkaXNj
bGFpbXMgZ2F0ZSBzYXRpc2ZhY3Rpb24sIHByb29mLXJlc3VsdCBjaGFuZ2VzLCBpbXBsZW1lbnRh
dGlvbiwgcHJvbW90aW9uLCByZW1lZGlhdGlvbiwgYW5kIGRpc3BhdGNoIGF1dGhvcml0eS4gCgoK
Qm90aCBwcmVzZXJ2ZWQgYm9kaWVzIGFyZSBieXRlLWV4YWN0IHdpdGhpbiB0aGVpciByZXNwZWN0
aXZlIG1hcmtlci1kZWxpbWl0ZWQgcmVnaW9ucywgaW5jbHVkaW5nIHRoZSBjbG9zZW91dCBjb25k
aXRpb25zIGFuZCB0aGUgUlA0L1AxL1AyL0NMRUFOL1JSLVJGLTAwMDHigJMwMDAzL25vLXJlbWVk
aWF0aW9uL25vLXByb29mLXJlc3VsdC1jaGFuZ2UgbWF0ZXJpYWwuIAoKClRoZSBldmlkZW5jZSBu
YXZpZ2F0aW9uIGFuZCBnbG9iYWwgaW5kZXggZWFjaCBhZGQgb25lIGJvdW5kZWQsIG5vbi1hdXRo
b3JpdGF0aXZlIHByb2plY3Rpb24gdG8gdGhlIG5ldyBvd25lci4gCgoKVGhlIHR3byBsZWRnZXIg
cm93cyBwcmVzZXJ2ZSBzZXBhcmF0ZSBwcm92ZW5hbmNlIGZvciB0aGUgMzQ3LWJ5dGUgYW5kIDEz
ODItYnl0ZSBzb3VyY2Ugc3BhbnMgd2hpbGUgc3BlY2lmeWluZyB0aGUgc2FtZSBhdG9taWMgcm9s
bGJhY2s6IHJlc3RvcmUgYm90aCBib2RpZXMsIHJlbW92ZSB0aGUgY29tcG91bmQgb3duZXIgYW5k
IHRoZSBleGFjdCBCYXRjaC0yIG5hdmlnYXRpb24vaW5kZXggYWRkaXRpb25zLCByZW1vdmUgYm90
aCBCYXRjaC0yIGxlZGdlciByb3dzLCBhbmQgcHJlc2VydmUgQmF0Y2ggMSBhbmQgdGhlIGV4Y2x1
ZGVkIHByaW9yIG93bmVycy9hcnRpZmFjdHMuIAoKClRoZSBzdXBwbGllZCB2YWxpZGF0aW9uIGlu
ZGVwZW5kZW50bHkgcmVjb3JkcyBleGFjdCBvd25lci1ib2R5IGJ5dGUvaGFzaCBlcXVhbGl0eSwg
cm9vdCByZWNvbnN0cnVjdGlvbiwgdW5pcXVlIGFuY2hvcnMsIHJlcXVpcmVkIGxpdGVyYWwvYm91
bmRhcnkgcHJlc2VydmF0aW9uLCB0aGUgZXhwZWN0ZWQgcm93IGNvdW50cywgcHJpb3Itb3duZXIg
c3RhYmlsaXR5LCB6ZXJvIGZhaWx1cmVzIGFjcm9zcyAxLDkxNiBzdXBwb3J0ZWQgbG9jYWwgbGlu
a3MvZnJhZ21lbnRzLCBhbmQgY2xlYW4gZm9yd2FyZC9yZXZlcnNlIHBhdGNoIGFwcGxpY2F0aW9u
LiAKCgpJIGFsc28gY2hlY2tlZCB0aGUgcHJlc2VydmVkIHBocmFzZSByZWZlcnJpbmcgdG8gdGhl
IGRldmVsb3BtZW50LXJldmlldyBjb250cmFjdCDigJxpbiAwNC7igJ0gSW4gdGhpcyBjYW5kaWRh
dGUgaXQgZG9lcyBub3QgY3JlYXRlIGEgcXVhbGlmeWluZyBhdXRob3JpdHkgZGVmZWN0OiBjaGFu
Z2luZyBpdCB3b3VsZCB2aW9sYXRlIHRoZSBieXRlLWV4YWN0IHByZXNlcnZhdGlvbiByZXF1aXJl
bWVudCwgd2hpbGUgdGhlIG5ldyBvd25lcidzIGVuY2xvc2luZyBtZXRhZGF0YSBleHBsaWNpdGx5
IGlkZW50aWZpZXMgdGhlIGN1cnJlbnQgY2Fub25pY2FsIEQzIGNvbnRyYWN0IGFuZCBtYWtlcyB0
aGUgcmVsYXRpb25zaGlwIHJlZmVyZW5jZS1vbmx5LiAKCgpWRVJESUNUOiBBUFBST1ZFRAo=
```

</details>
