# ChatGPT Pro advisory review: runtime-refactor D11 baseline behaviors and smoke scenarios

- Date: 2026-08-26
- Bound baseline commit/tree: `c164ec0c7acb24abcb44211afa26a2630dfa86cf` / `3170c00c22c65b693299137de03404e57be3108b`
- Landing subagent (`gpt-5.4`, Extra High): `/root/d11_batch3_baseline_smoke`
- Remediation subagent (`gpt-5.4`, Extra High): `/root/d11_batch3_referent_remediation`
- Closeout subagent (`gpt-5.4`, Extra High): `/root/d11_batch3_closeout`
- Root/orchestrator review owner: `/root`
- Initial independent review chat: https://chatgpt.com/c/6a8fa3d0-b794-83e9-bd50-dfbf7150c612
- Initial review mode: `initial-range`
- Initial review context: fresh ChatGPT conversation
- Fresh remediation-follow-up review chat: https://chatgpt.com/c/6a8fa794-5da8-83ea-b3da-0aa1f4219b5a
- Remediation-follow-up review mode: `remediation-follow-up`
- Remediation-follow-up review context: fresh ChatGPT conversation
- ChatGPT account surface: `Pro`
- Visible reasoning-effort control: `Extra High`
- Exact model label: not exposed by the current visible ChatGPT UI
- Initially reviewed complete candidate patch: `sha256:3370ece1f87e3723d00bbff0b7ce6bd6af304cd7026a7dba99a28d70afb02094` (`51282 bytes`) at `/private/tmp/d11-baseline-smoke-candidate.patch`
- Initial review prompt: `sha256:14394bb2c6a85141044248475d1299fd398bf82fef671c1ccb22b1b4773aae71` (`63475 bytes`) at `/private/tmp/d11-baseline-smoke-chatgpt-pro-initial-review-prompt.txt`
- Initial review answer: `sha256:fb1ae13d141d0f9853304edb037804308acdfb68e5bd1e780576b285202692ba` (`2955 bytes`) at `/private/tmp/d11-baseline-smoke-chatgpt-pro-initial-review-answer.txt`
- Initial validation log: `sha256:ce422f69f954d848ec32fb986fd29f399cf3407b551cf58cbd5debf4460a1106` (`4318 bytes`) at `/private/tmp/d11-baseline-smoke-validation.log`
- Remediation delta: `sha256:54e5796515f1d5c77d335f9493cfdcd47f727c135c926c9261652b6bf360341b` (`888 bytes`) at `/private/tmp/d11-baseline-smoke-remediation-delta.patch`
- Ordered initial-patch-plus-remediation-delta digest: `sha256:e4ff934ea42b5495b8fa543d44197fb9a3f3e6d15506af6e11b4ff29c30317ee` (`52170 bytes`) at `/private/tmp/d11-baseline-smoke-ordered-initial-plus-delta.patchstream`
- Updated full remediated candidate patch: `sha256:ff7a759e8b559a84aacf3e4338e7195c6899c8afc26af652a70d631464726c28` (`51514 bytes`) at `/private/tmp/d11-baseline-smoke-remediated-candidate.patch`
- Remediation validation log: `sha256:eb7d77ccf76385fd3c981452760e3e5212fab8a03dd6d8b9ebd01ce609bc76b1` (`5063 bytes`) at `/private/tmp/d11-baseline-smoke-remediation-validation.log`
- Remediation-follow-up review prompt: `sha256:6a19ca3e8a2f72817fb159acf1819b530d119ea30ef6e963a9b5b77bfa0d6f70` (`12245 bytes`) at `/private/tmp/d11-baseline-smoke-chatgpt-pro-remediation-follow-up-prompt.txt`
- Remediation-follow-up review answer: `sha256:11621667cef0c8081a5a6e22742bd6087483813af5784fef57e56963192820de` (`1177 bytes`) at `/private/tmp/d11-baseline-smoke-chatgpt-pro-remediation-follow-up-answer.txt`
- Exact embedded source body (`Baseline behaviors that all tracks preserve`): baseline lines `80–92`, `3644 bytes`, `sha256:5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132`
- Exact embedded source body (`Cross-gate smoke scenarios`): baseline lines `93–192`, `11714 bytes`, `sha256:586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85`
- Review verdict: `APPROVED`
- Review finding summary: `Initial review returned one accepted P2 on RG-DIFF-01's unresolved source-relative inventory referent; remediation fixed it; follow-up review approved with no qualifying delta regression.`
- Remediation rounds: `1`
- Review adjudication posture: root agent owns review/adjudication; subagents do not self-approve
- Commit posture: Batch 1 and Batch 2 are already committed complete at detached HEAD `c164ec0c7acb24abcb44211afa26a2630dfa86cf`; Batch 3 candidate plus this closeout remain uncommitted and unstaged during closeout
- Push posture: `not pushed`
- D12 posture: `blocked until exhaustive D11; separately unauthorized`
- Successor authority posture: no successor dispatch authority is granted

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Candidate scope and outcome

This independently reviewed D11 Batch 3 evidence family extracts the exact `## Baseline behaviors that all tracks preserve` and `## Cross-gate smoke scenarios` bodies into `llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md`, replaces only those two root bodies plus the root scenario subheadings with truthful compatibility anchors, adds the single Batch 3 navigation row in `llm-last-mile/runtime-refactor/evidence/README.md`, adds the single truthful `Baseline behaviors + Cross-gate smoke scenarios` global-index row, and adds the matching two-row D11 extraction-ledger coverage. The batch preserves exact Gate IDs, statuses, table order/cells, ordered scenario steps, the deterministic parked-successor sequence, Covers lines, negative requirements, fail-closed and exception boundaries, and evidence-vs-satisfaction semantics while leaving Batch 1, Batch 2, prior D3–D10 owners/review artifacts, `index/current.md`, `review-control/`, and runtime/platform behavior unchanged. The initial review accepted one P2: RG-DIFF-01's source-relative phrase `inventory above` lost a truthful destination-local referent in the standalone owner. The sole remediation adds a destination-local context note above the exact-body marker that explicitly links the already-canonical packet-local B1/B2.1 inventory owner without changing either preserved source body or the packet-local owner. Batch 3 is complete locally and independently approved after one remediation round and is landed pending one atomic commit. Batch 1 and Batch 2 remain committed complete at detached HEAD `c164ec0c7acb24abcb44211afa26a2630dfa86cf`. The exact next/final planned batch is Batch 4 reading model/current-state projection/receipt navigation and exhaustive D11 closeout; D11 remains in progress and incomplete until that final batch and exhaustive closeout are complete. This documentation decomposition is not proof/evidence satisfaction, gate promotion, implementation, remediation, dispatch, or successor authority. D12 remains blocked until exhaustive D11 and is separately unauthorized; no push or successor dispatch authority is granted.

## Prospective seven-path landing manifest

Prospective landing manifest for this independently reviewed batch:

- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
- `llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md`
- `llm-last-mile/runtime-refactor/evidence/README.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `docs/guidance/2026-08-26-runtime-refactor-d11-baseline-behaviors-and-smoke-scenarios-chatgpt-pro-review.md`

## Candidate path digests and imported review artifacts

Candidate path digests:

- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`: `sha256:159f531dc05aef1afc4d5a26eb412c1330db75ef4db1a90503690f139c538bf6`
- `llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md`: `sha256:90c9fe4e20fed14a5d2c3a22cb697e56e1a29191d905172e0104e19a02b2c0d2`
- `llm-last-mile/runtime-refactor/evidence/README.md`: `sha256:2ed99e460ff84bb742c5d99b82df25b3b61d3e168ea47a5f6c0dcfe724e526f2`
- `llm-last-mile/runtime-refactor/index/README.md`: `sha256:7157b5ae1a36c5539788e8f39e32375585565a251da6386aaf1e60b51c2636e2`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`: `sha256:f1ddf43b54400a29d69ca8d39828ef53c4a502ddee7d20d5f2b24fd885bc0c46`

Imported review artifacts:

- Initial candidate patch: `sha256:3370ece1f87e3723d00bbff0b7ce6bd6af304cd7026a7dba99a28d70afb02094` (`51282 bytes`)
- Initial review prompt: `sha256:14394bb2c6a85141044248475d1299fd398bf82fef671c1ccb22b1b4773aae71` (`63475 bytes`)
- Initial review answer: `sha256:fb1ae13d141d0f9853304edb037804308acdfb68e5bd1e780576b285202692ba` (`2955 bytes`)
- Initial validation log: `sha256:ce422f69f954d848ec32fb986fd29f399cf3407b551cf58cbd5debf4460a1106` (`4318 bytes`)
- Remediation delta: `sha256:54e5796515f1d5c77d335f9493cfdcd47f727c135c926c9261652b6bf360341b` (`888 bytes`)
- Ordered initial-patch-plus-remediation-delta byte stream: `sha256:e4ff934ea42b5495b8fa543d44197fb9a3f3e6d15506af6e11b4ff29c30317ee` (`52170 bytes`)
- Updated full remediated candidate patch: `sha256:ff7a759e8b559a84aacf3e4338e7195c6899c8afc26af652a70d631464726c28` (`51514 bytes`)
- Remediation validation log: `sha256:eb7d77ccf76385fd3c981452760e3e5212fab8a03dd6d8b9ebd01ce609bc76b1` (`5063 bytes`)
- Remediation-follow-up review prompt: `sha256:6a19ca3e8a2f72817fb159acf1819b530d119ea30ef6e963a9b5b77bfa0d6f70` (`12245 bytes`)
- Remediation-follow-up review answer: `sha256:11621667cef0c8081a5a6e22742bd6087483813af5784fef57e56963192820de` (`1177 bytes`)
- Exact `Baseline behaviors that all tracks preserve` source body: `3644 bytes`, `sha256:5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132`
- Exact `Cross-gate smoke scenarios` source body: `11714 bytes`, `sha256:586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85`

## Exact source-body proof

- Baseline `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md` lines `80–92` are exactly `3644` bytes with SHA-256 `5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:baseline-behaviors:start -->` and `<!-- exact-extracted-body:baseline-behaviors:end -->` in `llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md` are also exactly `3644` bytes with SHA-256 `5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132`.
- Baseline `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md` lines `93–192` are exactly `11714` bytes with SHA-256 `586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:cross-gate-smoke-scenarios:start -->` and `<!-- exact-extracted-body:cross-gate-smoke-scenarios:end -->` in `llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md` are also exactly `11714` bytes with SHA-256 `586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85`.
- Validation confirmed root bytes outside the two replaced spans remain byte-stable versus `HEAD`; the preserved root headings `## Baseline behaviors that all tracks preserve` and `## Cross-gate smoke scenarios` plus the six root scenario subheadings remain exactly once each as compatibility anchors, and the owner headings remain exactly once.
- `ROW_COUNT_OK` preserved exactly one Batch 3 `evidence/README.md` navigation row, exactly one `Baseline behaviors + Cross-gate smoke scenarios` global-index row, and exactly two D11 Batch 3 extraction-ledger rows.
- `BODY_PRESERVATION_OK` preserved the exact Gate IDs, statuses, table order/cells, ordered scenario steps, deterministic parked-successor sequence, Covers lines, negative requirements, fail-closed cases, exceptions, platform limitations, and evidence-vs-satisfaction boundaries without collapsing the two source bodies into one contiguous extraction.
- The destination-local remediation note lives above `<!-- exact-extracted-body:baseline-behaviors:start -->`, outside both exact-body markers, and explicitly links RG-DIFF-01's source-relative `inventory above` referent to [`../../llm-last-mile/runtime-refactor/b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result`](../../llm-last-mile/runtime-refactor/b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result). It clarifies the referent only; it does not duplicate packet-local evidence, alter either preserved body, or change gate/implementation/authority status.

## Initial independent review

- The exact initial review prompt preserved below scoped supported behavior to repository-local UTF-8 Markdown with GitHub-style heading anchors, tables, inline code, relative links/fragments, and HTML comment boundary markers; it excluded runtime behavior, alternative Markdown dialects, URL reachability, D12 root-cutover behavior, code/scripts/platform work, broader repository audit, review-control content, and validation of the four declared opaque links.
- That prompt required the exact five-path candidate fence, exact source-body preservation for both extracted spans, truthful root compatibility anchors and owner uniqueness, the single Batch 3 evidence/index row additions, the paired atomic rollback semantics, and the absence of broken supported links or widened authority claims.
- Its blocking threshold allowed only P1/P2-equivalent findings that are reachable through supported behavior, violate an in-scope invariant, have the required material consequence, and are caused or materially worsened by the candidate patch.
- The initial review returned one accepted P2 and **`VERDICT: REQUEST_CHANGES`**.
- Accepted finding: in `llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md`, RG-DIFF-01 still said “The exact fourteen-test B1/B2.1 inventory above ...”, but the new standalone canonical owner contained no destination-local context identifying or linking that fourteen-test inventory because the preserved extraction starts at source line `80`. The byte-preserved row therefore lost a truthful source-relative referent in its new canonical destination.
- Material consequence accepted by review: a maintainer following the canonical Batch 3 owner could no longer resolve which exact fourteen historical B1/B2.1 names and production-level assertions RG-DIFF-01 governs, which could affect whether a failure-to-pass transition is accepted or keeps closeout open.
- The initial review found no additional qualifying blocker in the compatibility-heading projections, Batch 3 navigation/index rows, paired atomic rollback rows, authority boundaries, scenario ordering, or supported relative links.

## Bounded remediation

A fresh remediation subagent changed only `llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md`. It added a `540`-byte destination-local context note at line `18`, above the `baseline-behaviors` exact-body marker, explicitly identifying RG-DIFF-01's `inventory above` referent as the exact fourteen historical B1/B2.1 names and production routes and linking directly to their already-canonical packet-local owner. The exact preserved `Baseline behaviors` and `Cross-gate smoke scenarios` bodies were unchanged byte-for-byte, the packet-local B1/B2.1 owner remained unchanged byte-for-byte, and `05-debug-regression-ledger.md`, `evidence/README.md`, `index/README.md`, and `migration/extraction-ledger.md` did not change during remediation.

## Remediation validation

- The remediation validation log reverified the exact five-path candidate fence, detached baseline `HEAD`/tree, and zero staged files.
- Non-owner candidate hashes remained exact: root `159f531dc05aef1afc4d5a26eb412c1330db75ef4db1a90503690f139c538bf6`, `evidence/README.md` `2ed99e460ff84bb742c5d99b82df25b3b61d3e168ea47a5f6c0dcfe724e526f2`, `index/README.md` `7157b5ae1a36c5539788e8f39e32375585565a251da6386aaf1e60b51c2636e2`, and `migration/extraction-ledger.md` `f1ddf43b54400a29d69ca8d39828ef53c4a502ddee7d20d5f2b24fd885bc0c46`.
- The remediated owner hash changed only from the reviewed-candidate owner `023def64882e738264b072f26fb434397f53ea8a122a8d5fd45a104b06b37698` to the current remediated owner `90c9fe4e20fed14a5d2c3a22cb697e56e1a29191d905172e0104e19a02b2c0d2`.
- `OWNER_CONTEXT_NOTE_OK` recorded the new line `18` note and the exact target link. `B1_REFERENT_LINK_OK` confirmed the target anchor resolves and the packet-local owner remains unchanged at `sha256:6387b0a8548c542c9c985f69e5f6845eb1379c59534421009de7763a5ce3dbed`.
- Both extracted bodies retained exact byte counts and SHA-256 digests; root compatibility anchors, owner anchors, table semantics, scenario ordering, Covers lines, and separate-provenance semantics remained stable.
- The runtime-refactor Markdown link scan stayed clean with `checked=1942 excluded=4 failures=0`, and the four known opaque exclusions remained exactly the declared external paths.
- The remediation delta patch forward/reverse apply matched the reviewed candidate, and the full remediated candidate patch forward/reverse apply matched a clean baseline export.

## Fresh remediation-follow-up review

The fresh independent remediation-follow-up review returned:

- **`No blocker found.`**
- **`VERDICT: APPROVED`**

It confirmed that the accepted P2 is fully fixed: the new destination-local note sits above and outside the exact-body marker, explicitly identifies the `inventory above` referent as the exact fourteen historical B1/B2.1 names/production routes, and links directly to the already-canonical packet-local owner. It also confirmed that the one remediation delta introduces no qualifying regression in preserved semantics, links/anchors, authority boundaries, row counts, or path scope.

## Post-review closeout validation

- The rendered prompt/answer copies below are whitespace-clean: any trailing literal spaces or tabs from the imported UTF-8 prompt or answer are rendered as `&#32;` or `&#09;` so this repository-local Markdown artifact stays clean under `git diff --check` while the adjacent base64 blocks preserve the exact bytes.
- Final changed-path fence including untracked files is exactly these seven paths:
  - `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
  - `llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md`
  - `llm-last-mile/runtime-refactor/evidence/README.md`
  - `llm-last-mile/runtime-refactor/index/README.md`
  - `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
  - `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
  - `docs/guidance/2026-08-26-runtime-refactor-d11-baseline-behaviors-and-smoke-scenarios-chatgpt-pro-review.md`
- Candidate file hashes remained unchanged before and after closeout: root `159f531dc05aef1afc4d5a26eb412c1330db75ef4db1a90503690f139c538bf6`, owner `90c9fe4e20fed14a5d2c3a22cb697e56e1a29191d905172e0104e19a02b2c0d2`, `evidence/README.md` `2ed99e460ff84bb742c5d99b82df25b3b61d3e168ea47a5f6c0dcfe724e526f2`, `index/README.md` `7157b5ae1a36c5539788e8f39e32375585565a251da6386aaf1e60b51c2636e2`, and `migration/extraction-ledger.md` `f1ddf43b54400a29d69ca8d39828ef53c4a502ddee7d20d5f2b24fd885bc0c46`.
- `git diff --check` passed for the final closeout state.
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md` and `git diff --no-index --check /dev/null docs/guidance/2026-08-26-runtime-refactor-d11-baseline-behaviors-and-smoke-scenarios-chatgpt-pro-review.md` reported no whitespace diagnostics.
- The rendered initial prompt, initial answer, remediation-follow-up prompt, and remediation-follow-up answer below all round-tripped exactly back to the imported UTF-8 artifacts, and the embedded base64 blocks decode exactly back to the imported bytes without silently correcting whitespace, diff fences, or wording.
- Both exact source-body hashes remain true; root/owner headings and scenario subheadings remain unique; Gate IDs and statuses remain in the same order; scenario order and step counts remain `S1=5`, `S1A=18`, `S2=6`, `S3=5`, `S4=4`, `S5=8`; the deterministic parked-successor sequence remains `10` steps; Covers lines remain exact with the multiline `S2` continuation intact; and the RG-DIFF-01 destination-local referent note still resolves to the unchanged packet-local owner.
- Exactly one Batch 3 evidence-navigation row, one Batch 3 global-index row, and two Batch 3 extraction-ledger rows remain present and truthful.
- Runtime-refactor Markdown link/anchor validation remained covered by the imported remediation validation (`checked=1942 excluded=4 failures=0`), and additional tracker/review-record local link checks passed during this closeout. Plain-text external review URLs were not part of the local Markdown-link set.
- Prior D3/D5–D10 and D11 Batch 1/Batch 2 owners/review artifacts remained hash-stable outside the exact seven-path fence, and the packet-local B1/B2.1 referent target remained unchanged.
- No files were staged, committed, or pushed during this closeout.

## Atomic rollback instructions

1. If the prospective seven-path batch has been committed, reverse that single batch commit atomically (for example, `git revert --no-commit <batch-commit>`) and verify that only the seven paths above are touched.
2. If the batch is still uncommitted, restore the tracked paths from `HEAD` and remove the two untracked files: `git restore --source=HEAD -- llm-last-mile/runtime-refactor/05-debug-regression-ledger.md llm-last-mile/runtime-refactor/evidence/README.md llm-last-mile/runtime-refactor/index/README.md llm-last-mile/runtime-refactor/migration/extraction-ledger.md docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md` then `rm -f llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md docs/guidance/2026-08-26-runtime-refactor-d11-baseline-behaviors-and-smoke-scenarios-chatgpt-pro-review.md`.
3. After either rollback path, confirm together that the exact `3644`-byte `Baseline behaviors that all tracks preserve` body and exact `11714`-byte `Cross-gate smoke scenarios` body are restored to `05-debug-regression-ledger.md` in original order, the untracked owner and review record are removed, the sole Batch 3 `evidence/README.md` row is removed, the exact `Baseline behaviors + Cross-gate smoke scenarios` global-index row is removed, the two D11 Batch 3 extraction-ledger rows are removed, and the tracker summary/queue/closeout-row additions are reverted.
4. Leave Batch 1, Batch 2, prior D3–D10 owners and review artifacts, `index/current.md`, `review-control/`, remaining D11 batches, and every path outside this seven-path batch unchanged.
5. Documentation rollback must not be rewritten as proof/evidence satisfaction, gate promotion, implementation, remediation, dispatch, or successor authority. D11 remains incomplete, and D12 remains blocked until exhaustive D11 and separately unauthorized.

## Final verdict

Shared `Baseline behaviors` + `Cross-gate smoke scenarios` is complete locally and **APPROVED** after one remediation round and is landed pending one atomic commit. Batch 1 canonical issue ledger + A1.2a-WB gate assignment and shared Batch 2 `Closeout rule` + residual `Review-process calibration` remain committed complete at detached HEAD `c164ec0c7acb24abcb44211afa26a2630dfa86cf`. The exact next/final planned batch is Batch 4 reading model/current-state projection/receipt navigation and exhaustive D11 closeout; D11 remains in progress and incomplete until that final batch is complete. This documentation decomposition does not complete evidence, satisfy a gate, authorize implementation, promotion, remediation, dispatch, or successor work, and it does not replace review-control receipts. D12 remains blocked until exhaustive D11 and is separately unauthorized. No push or successor dispatch authority is granted.

## Preserved review prompts and answers

<details>
<summary>Initial-range review prompt (rendered copy)</summary>

<pre>
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: exact baseline commit c164ec0c7acb24abcb44211afa26a2630dfa86cf with tree 3170c00c22c65b693299137de03404e57be3108b, plus the complete content-addressed candidate patch below. Candidate patch SHA-256: 3370ece1f87e3723d00bbff0b7ce6bd6af304cd7026a7dba99a28d70afb02094.
Review target: the complete bounded D11 Batch 3 documentation-decomposition candidate. Review every changed line and every byte of the new file in the patch; do not infer or request excluded repository content.

Manifest (complete changed-path fence):
1. llm-last-mile/runtime-refactor/05-debug-regression-ledger.md (modified root compatibility projections)
2. llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md (new compound canonical evidence owner)
3. llm-last-mile/runtime-refactor/evidence/README.md (modified non-authoritative evidence navigation)
4. llm-last-mile/runtime-refactor/index/README.md (modified non-authoritative global index)
5. llm-last-mile/runtime-refactor/migration/extraction-ledger.md (modified provenance/rollback ledger)
The tracker and review record are intentionally absent until approval.

Supported inputs and behavior: repository-local UTF-8 Markdown using GitHub-style heading anchors, tables, ordered lists (including a nested semantic sub-sequence expressed as a second ordered list), inline code, relative links/fragments, and HTML comment boundary markers. Legacy root headings remain resolvable compatibility anchors. Marker comments are non-rendered delimiters; the exact UTF-8 bytes strictly between each named start/end marker are the preservation subjects. Relative links resolve from the containing file. Four already-declared opaque/external links elsewhere in the unchanged corpus are excluded from local resolution.
Supported platforms and dialects: documentation-only, repository-local Markdown/GitHub-style anchor behavior on this repository tree. No Rust/runtime/platform execution behavior, alternative Markdown dialect, case-insensitive-filesystem promise, remote URL availability, or D12 root-cutover behavior is in scope.

In-scope invariants:
- Exact baseline root lines 80-92 are independently preserved as 3644 bytes, SHA-256 5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132, inside the matching baseline-behaviors owner markers.
- Exact baseline root lines 93-192 are independently preserved as 11714 bytes, SHA-256 586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85, inside the matching cross-gate-smoke-scenarios markers. The two source spans have separate provenance and must not be falsely represented as one contiguous source span.
- All root bytes outside those two replaced spans remain stable. The root retains exactly once, in original order, these eight legacy headings/anchors with truthful owner pointers: Baseline behaviors; Cross-gate smoke scenarios; S1; S1A; S2; S3; S4; S5. Corresponding owner anchors are unique and resolvable.
- The permanent baseline table preserves exact row/cell/order and semantic state: RG-BASE-01, RG-BASE-02, RG-BASE-03, RG-BASE-04, RG-DIFF-01, including open/blocking/not-waived, RegressionMasked, BaselineRegressionAmbiguous, FailToPass, negative requirements, exceptions, and independent-review requirement. Definitions/scenarios must not become current pass/proof claims.
- Smoke scenarios S1/S1A/S2/S3/S4/S5 preserve exact order, all ordered steps, the ten-step deterministic parked-successor sub-sequence, all Covers lines/IDs, commands/literals/paths/hashes, fail-closed and negative cases, compatibility distinctions, platform limitations, and evidence-vs-satisfaction boundaries.
- Exactly one canonical owner exists for this compound evidence family. It is documentation/evidence-regression ownership only and must not satisfy a gate, change current-pass state, close proof, replace review-control receipts, authorize implementation/promotion/remediation/dispatch/successor work, or reopen D3 governance.
- Exactly one Batch 3 evidence-navigation row, one global-index owner row, and two D11 extraction-ledger rows are added. Navigation/index are non-authoritative and resolve correctly.
- The two ledger rows preserve distinct source provenance and one truthful atomic rollback: restore both exact bodies in original locations/order; remove the owner plus exact Batch 3 evidence/index/two-ledger additions; leave D11 Batches 1/2, D3-D10 owners, index/current.md, review-control/, and every outside-fence path unchanged.
- The patch changes exactly the five manifested paths, introduces no whitespace or supported local link/fragment failure, and keeps prior D3/D5-D10 and D11 Batch 1/2 owner/review artifacts hash-stable.

Required material consequence: a blocker must demonstrate a reachable patch-caused alteration/loss/reordering of preserved baseline or smoke semantics/bytes; a broken supported legacy/owner navigation path; false provenance or incomplete/non-atomic rollback; duplicate/inflated authority or proof/pass claim; unauthorized path/scope mutation; or another defect that could make maintainers materially rely on wrong D11 state. Pure style/wording preference, harmless metadata phrasing, or unsupported behavior is non-material.
Blocking threshold: only P1/P2-equivalent findings reachable through supported behavior, violating an in-scope invariant, meeting the material consequence threshold, and caused or materially worsened by this patch. Every blocker must identify exact file/hunk, reachability, invariant, material consequence, and patch causality. Do not block on pre-existing/excluded behavior.
Accepted prior findings: none; this is the first independent review of this candidate.
Deferred or out-of-scope concerns: global Markdown reformatting; D3 governance or review-control contents; re-extraction/edits of packet-local D5/D6 evidence or D11 Batch 1/2; D9/D10/slices/tasks owners; tracker and closeout record; D12 cutover; Rust/runtime/scripts/platform implementation; proof/evidence satisfaction, promotion, remediation, dispatch, successor work; roadmap/ZIP changes; the four unchanged opaque links; unrelated/pre-existing defects.

Validation evidence (locally produced and independently spot-checked before review):
- clean detached baseline matched before edits; exact five-path candidate fence; nothing staged
- git diff --check and new-file no-index whitespace check passed
- both pre-edit source spans matched their frozen artifacts; each owner body separately matches exact bytes, counts, and SHA-256
- root reconstruction and prefix/suffix stability passed
- eight legacy root headings remained unique/in-order with resolving unique owner anchors
- baseline gate order/state preservation and scenario order/step counts/ten-step nested sequence/Covers preservation passed
- exactly one evidence row, one index row, and two Batch 3 ledger rows added
- 323 prior owner/review files remained hash-stable
- Markdown scan checked 1941 supported local links/fragments, excluded only four declared opaque links, failures zero
- complete patch forward- and reverse-applies against a clean baseline export

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return findings first by severity. For every finding, include exact file/hunk, supported-input reachability, violated invariant, material consequence, and patch causality. State explicitly when there are no qualifying findings. End with exactly one verdict line: `VERDICT: APPROVED` if there are no qualifying blocking findings, otherwise `VERDICT: REQUEST_CHANGES`.

Complete candidate patch (SHA-256 3370ece1f87e3723d00bbff0b7ce6bd6af304cd7026a7dba99a28d70afb02094):
```diff
diff --git a/llm-last-mile/runtime-refactor/05-debug-regression-ledger.md b/llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
index 13012d850ad851e979de152e0dff2d4d5613b40f..11a9046ffb40b4f90984d7e4dc420a64f739bb61 100644
--- a/llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
+++ b/llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
@@ -79,116 +79,35 @@ Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a12b-recorded
&#32;
 ## Baseline behaviors that all tracks preserve
&#32;
-These rows define permanent behavior contracts, not current pass claims. Any row explicitly marked
-open remains a blocking regression gate until its named owner and real-path proof close it.
-
-| Gate ID | Baseline | Required proof |
-|---|---|---|
-| **RG-BASE-01** | Public world-scoped start → turn/reattach → stop; **open, blocking, and not waived** | A1.2 must supply exact parked-successor `Attach`/`ResumeOneTurn` application. The held A1.3-P0 and A1.3 records remain preserved but cannot be dispatched. A1.3-P1 must atomically land the new authority-managed public Start/turn/reattach transport path, exact startup/post-turn actor-event resolution on the real public CLI/helper/REPL path, and only the strictly mechanical projection of already HSA-authorized correlation through the exact retained-turn `orchestrator_world_dispatch.rs` acceptance-context construction/projection seam and into the existing B1 acceptance-context field, without legacy authority writes, readiness/process inference, auto-attach drift, B1 semantic ownership changes, or C1/B3.1 semantic changes. Exact session and world binding survive; stop reaches durable terminal truth even when transport posture changes. Stale lifecycle/world-binding overwrite remains rejected. Prove unsafe other-principal authority fails closed separately, then run the positive smoke against an owner-only private bootstrap home with exact type/`0700`, descriptor identity/replacement safety, qualified access/default `NoData` under authoritative mode bits, and no effective other-principal authority; `NoData` is not physical-absence proof. This gate is neither permanently expected to fail nor successful until that real-path wall is green. |
-| **RG-BASE-02** | REPL first-dispatch `run_world_task` binding repair | Correct binding succeeds; stale/mismatched world generation fails closed; no generic binding synthesis on unrelated surfaces. A host runtime may carry exact parent-session world binding while its descriptor and participant manifest remain host-scoped. |
-| **RG-BASE-03** | Parked host ordinary-command and continuity parity | Unprefixed `ls`/`pwd` remain usable; policy-required `cd ../` cage denial remains; later targeted host turn reuses session/UAA continuity; public CLI parity stays green. |
-| **RG-BASE-04** | Retained spawn/fork/exact continue/exact stop plus ambiguity close | Exact source and child handles route correctly; backend-only follow-up with multiple retained workers fails closed; source detached stop and child live-transport stop remain valid. |
-| **RG-DIFF-01** | Monotonic broad-suite differential; a historical failure becoming a pass is neither automatic success nor automatic regression | Apply the exact-name and normalized-signature transition gate in `04`. Preserve complete inventories and artifact hashes; prove every historical failure-to-pass transition through the exact baseline and current full production dispatcher; inspect test and assertion diffs; map the causal change to the owning slice and symbol; and prove no direct resolver, transport, receipt, supervisor, helper, bypass, weakened enforcement, renamed/replaced/ignored test, weakened assertion, removed behavior, or unrelated capability loss. The exact fourteen-test B1/B2.1 inventory above retains every historical name and original production-level assertion. A lower-level substitute is `RegressionMasked`; any otherwise uncertain transition is `BaselineRegressionAmbiguous`. Either classification keeps the owning closeout open, and neither can count as `FailToPass` proof. Independent review is mandatory. |
+Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#baseline-behaviors-that-all-tracks-preserve`](evidence/baseline-behaviors-and-smoke-scenarios.md#baseline-behaviors-that-all-tracks-preserve).
&#32;
 ## Cross-gate smoke scenarios
&#32;
-### S1 — Authority survives episode loss
+Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#cross-gate-smoke-scenarios`](evidence/baseline-behaviors-and-smoke-scenarios.md#cross-gate-smoke-scenarios).
&#32;
-1. Start a world-scoped host session and retain a worker.
-2. Kill the current helper/episode and remove its private socket.
-3. Read durable authority, worker manifest, and any active receipt.
-4. Reattach or perform sanctioned durable closeout.
-5. Prove no stale episode write regressed the authority revision.
+### S1 — Authority survives episode loss
&#32;
-Covers: `RG-AUTH-01`, `RG-AUTH-02`, `RG-CLOSE-01`, `RG-BASE-01`.
+Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#s1--authority-survives-episode-loss`](evidence/baseline-behaviors-and-smoke-scenarios.md#s1--authority-survives-episode-loss).
&#32;
 ### S1A — Revision-bound host transitions survive helper loss and replay
&#32;
-1. Before any production Start, classify genuinely absent, interrupted-initialization, valid-existing, unsupported-pre-A1, and corrupt/unsupported stores. Reject malformed, partial, unreadable, symlinked, permission-invalid, and unsupported roots without generating a replacement store ID. Persist the same normalized physical bootstrap home in the init marker, greenfield certificate, and root; pass its opened handle through the bounded StateStore/config/policy/inventory entry points; change/remove ambient home values between each call and prove no reread; reject a copied/rebound authority tree; and crash/restart after marker, key, root, and marker-cleanup steps without changing store/home identity.
-2. Completely and safely enumerate both pre-A1 session/participant authority collections before certificate publication and every semantic transaction. Prove component-by-component no-follow traversal from the trusted home rejects ancestor symlinks, unsafe owner/mode/ACL, cross-device rebinding, and scan-to-publication identity replacement. Prove validated empty collections permit greenfield certification, while any artifact is `UnsupportedLegacyState` without parsing/conversion and unreadable, permission-invalid, symlinked, or partially enumerable collections fail closed. Every pre-A1 read-decide-write-remove-rename transaction retains the opened trusted physical root, descendant handles, exact identity, activation observation, and root lock through final file/directory `fsync`; it never rereads `SUBSTRATE_HOME`, uses ambient CWD, reconstructs an absolute descendant path, or releases the lock early. In a real subprocess, rename/replace/rebind the lexical root after lock acquisition and prove original/replacement identities explicitly: the replacement receives zero writes, temps, removals, or `fsync`-dependent publication, and a lock on the former root is never represented as protection for the replacement. Serialize every in-repository pre-A1 writer on the same root lock before activation; after init-marker or root publication it rejects before mutation. No missing row or compatibility projection satisfies absence.
-3. Exercise every A1 object kind against its exact V1 schema, file bytes, typed path, and commitment variant/domain. Reject cross-kind/schema/domain substitution; require `run_present=0x01` and the exact non-empty parent-intent run ID for every sensitive domain, with golden negative fixtures for absent/empty/different runs. Crash before/after each temp-file fsync and rename: recognized operation-bound temps are removed and never promoted, while unsafe/unrecognized temps fail closed. For pending initialization, validate a reused key only against the init marker and re-fsync the file plus `keys/` before root publication. Crash between first-use kind/version directory creation and every directory/parent fsync; only safe empty closed directories survive without object authority. Crash after object rename but before root commit; restart must classify the safe unindexed file as a retained non-authoritative orphan, and only a retry with the complete ref, exact parent intent/run/store context, and verified bytes may adopt it. Exact retry must treat every reservation, tombstone, transition intent, issuer-request index, application-journal, and object-index entry as semantic authority occupancy and may join only complete matching committed state. Rotate and retire HMAC keys across crashes before/after key publication, root selection, retirement root commit, and key deletion; after reconciliation and immediately before root publication, validate the complete candidate against the exact locked root/revision, home/store identity, key registry/files, active/verification/retired constraints, reachable objects, semantic maps/indexes/journal, empty pre-A1 collections, and unchanged physical root. Old referenced keys remain verification-only, key loss or post-reconciliation invalidity fails before publication with zero semantic mutation, and crash/restart plus exact retry converge.
-4. Issue a public A1.2 `Start` intent against greenfield-certificate-proven `ExpectedAbsent`, then terminate delivery once while `Issued`, once after plan load/removal but before claim, once while `Claimed`, once after initial authority birth, and once after input acceptance but before success is observed.
-5. Restart and retry the exact intent; prove missing plan transport is reproduced from the retained payload, pre-application cases safely resume/reconcile, and committed/accepted cases join one immutable result without another namespace, authority revision, participant allocation, or input delivery. Prove payload cleanup occurs only after exact terminal handoff, with crashes/failures before and after the `ReleaseEligible` root commit, transport deletion, object-directory fsync, and `Released` root commit for both present and already-absent transport bytes.
-6. Repeat public CLI and real REPL `Attach` and `ResumeOneTurn` against exact revisions, including bounded auto-attach plan production, and prove the same bootstrap-home/workspace/world/descriptor/attach/resume/policy commitments reach application.
-7. Reject stale revision, substituted plan, cross-kind ref, payload/hash mismatch, wrong caller/source/target/lineage, wrong bootstrap home/workspace/world generation, missing/copied greenfield certificate, compatibility-derived absence, pre-A1 artifacts, missing key, expired intent, superseded claim, and conflicting replay before authority mutation.
-8. Run public world-scoped start → reattach → stop and prove exact session/world binding plus durable terminal truth are unchanged.
-
-The deterministic parked-successor portion of this scenario is:
-
-1. Persist current `Active` / `ParkedResumable` durable authority.
-2. Remove all episode-local PID, handle, helper-readiness, and prompt-stream state.
-3. Invoke the public prompt-bearing turn or explicit reattach path.
-4. Resolve the exact current authority revision/hash and authoritative lineage.
-5. Issue and apply `ResumeOneTurn` for the public turn or `Attach` for explicit reattach.
-6. Preserve the exact session identity and world binding.
-7. Revision-authorize exactly one successor participant and retain immutable input handoff when
-   present.
-8. Launch the helper/REPL execution episode only after durable application.
-9. Retry the exact request and join the committed intent/application without a second successor.
-10. Stop the same session and reach durable terminal truth.
-
-Covers: `RG-AUTH-01`, `RG-AUTH-02`, `RG-AUTH-03`, `RG-BASE-01`, `RG-BASE-02`.
+Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#s1a--revision-bound-host-transitions-survive-helper-loss-and-replay`](evidence/baseline-behaviors-and-smoke-scenarios.md#s1a--revision-bound-host-transitions-survive-helper-loss-and-replay).
&#32;
 ### S2 — Receipt, supervisor, obligation, cancel
&#32;
-1. Accept one ephemeral task from its exact pre-terminal B0 acknowledgement, durably claim the
-   stream in B2.1 before any subsequent frame, and prove the production path never attempts legacy
-   active-task registration. Blocking inspect/wait and narrowly adapted cancel consume exact
-   receipt/supervisor truth, and foreground drop does not erase the accepted or supervised work.
-2. Continue an exact retained worker, persist its B1 accepted active-run identity, hand the same B0
-   stream to B2.1 with no observation gap, and prove the foreground may remain a blocking waiter
-   without retaining observation ownership.
-3. Normalize one supported provider attention shape before `AgentEvent` construction, emit one
-   B3.1 event with exact target/run/thread/class/attention/causation identity, and prove B2.1
-   journals it durably and C1 materializes one obligation before the terminal event. Prove an
-   ambiguous or malformed attention-driving provider shape fails before emission rather than
-   becoming progress or no-attention, and prove host JSON-pointer parsing cannot repair it. For
-   every post-ack retained `Event` frame, prove either the typed member reaches the journal/cut or
-   the stream and Complete cut fail closed; no event is silently dropped or left untyped.
-4. Drop callers and restart the supervisor during both accepted families, then replay duplicate
-   frames/events and prove the claim, journal, receipt, obligation, ledger revision, watermark, and
-   terminal state remain idempotent. Missing exact terminal truth remains interrupted/nonterminal.
-5. Query C1 before and after the exact terminal cut; prove Pending first, then Complete for both
-   empty/no-attention and non-empty/attention dispositions without consulting projections. The
-   Complete snapshot&#x27;s ordered event vector must join exhaustively to every retained B2.1 `Event`
-   ref in scope, including typed non-attention events.
-6. After B2.2/B3.2, receive the accepted receipt before terminal exit and cancel by
-   `active_run_id` in the same host turn.
-
-Covers: `RG-EVENT-01`, `RG-RECEIPT-02`, `RG-RECEIPT-03`, `RG-SUP-01`, `RG-SUP-02`,
-`RG-MSG-01`, `RG-CANCEL-01`, `RG-OBL-01`, `RG-OBL-02`, `RG-OBS-01`.
+Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#s2--receipt-supervisor-obligation-cancel`](evidence/baseline-behaviors-and-smoke-scenarios.md#s2--receipt-supervisor-obligation-cancel).
&#32;
 ### S3 — World-UAA mediation under narrowed policy
&#32;
-1. Spawn/continue world Codex with write narrowed to `src/parser.rs`.
-2. Prove shell and provider-native edit can modify that file.
-3. Prove shell, direct edit, and write-capable tool cannot modify `README.md` or escape root.
-4. Repeat with broker unavailable and require fail-closed behavior.
-5. Join each operation to the receipt&#x27;s immutable policy hash.
-
-Covers: `RG-UAA-01`, `RG-UAA-02`, `RG-UAA-03`, `RG-POLICY-01`, `RG-POLICY-02`.
+Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#s3--world-uaa-mediation-under-narrowed-policy`](evidence/baseline-behaviors-and-smoke-scenarios.md#s3--world-uaa-mediation-under-narrowed-policy).
&#32;
 ### S4 — Host visibility and failure truth
&#32;
-1. Run equivalent writes under host-visible and full-isolation policies.
-2. Verify exact visibility/reconciliation semantics.
-3. Force a non-zero UAA turn after a permitted write.
-4. Prove durable failed active-run state, retained authority behavior, diagnostics, and file visibility outcome independently.
-
-Covers: `RG-SYNC-01`, `RG-WORKER-EXIT-01`, `RG-OBS-01`.
+Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#s4--host-visibility-and-failure-truth`](evidence/baseline-behaviors-and-smoke-scenarios.md#s4--host-visibility-and-failure-truth).
&#32;
 ### S5 — Existing gateway carrier preservation and direct Codex adoption
&#32;
-1. Run the existing launcher/consumer regression baseline: validated `GatewayAuthBundleV1`, raw-secret env scrubbing, one-time gateway consumption, and fresh bundle delivery on restart.
-2. Resolve a credential source on the host and launch the exact managed in-world gateway without copying the secret payload into a world-visible runtime home.
-3. Start direct world Codex with a per-worker Substrate-owned projection that points provider traffic at that gateway; do not seed a host auth file in contract-correct mode.
-4. Join the envelope, accepted policy snapshot, projection identity, exact gateway receiver, and non-secret handoff evidence for the same world generation.
-5. Prove the UAA child and descendants cannot inherit the secret FD or raw secret and that no secret-bearing runtime-native file is created.
-6. Exercise duplicate consume, wrong gateway/world generation, expiry, unavailable handoff, and unavailable gateway adoption; require fail-closed behavior.
-7. Exercise the explicitly named compatibility copy mode separately and prove it is logged, has retirement metadata, and cannot satisfy contract-promotion evidence.
-8. Inspect logs/traces/manifests for secret payloads, secret-bearing paths, and reusable secret-derived hashes; none may appear.
-
-Covers: `RG-CONFIG-02`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-UAA-02`, `RG-UAA-03`, `RG-OBS-01`.
+Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#s5--existing-gateway-carrier-preservation-and-direct-codex-adoption`](evidence/baseline-behaviors-and-smoke-scenarios.md#s5--existing-gateway-carrier-preservation-and-direct-codex-adoption).
&#32;
 ## Closeout rule
&#32;
diff --git a/llm-last-mile/runtime-refactor/evidence/README.md b/llm-last-mile/runtime-refactor/evidence/README.md
index 30bc1c387309722e75782a40aca8441ae6b24786..c5f5742ba3710229aebd6f5af52de8973d47b23b 100644
--- a/llm-last-mile/runtime-refactor/evidence/README.md
+++ b/llm-last-mile/runtime-refactor/evidence/README.md
@@ -18,3 +18,4 @@
 |---|---|---|
 | Batch 1 — canonical issue ledger | [`canonical-issue-ledger.md#canonical-issue-ledger`](canonical-issue-ledger.md#canonical-issue-ledger) | Full extracted `## Canonical issue ledger` table plus the preserved `### A1.2a-WB gate assignment` body; exact issue IDs, chronology, limitations, obligations, literals, hashes, commands, and negative requirements remain unchanged. |
 | Batch 2 — closeout rule + review-process calibration | [`closeout-and-review-calibration.md`](closeout-and-review-calibration.md) | Exact extracted [`## Closeout rule`](closeout-and-review-calibration.md#closeout-rule) and [`## Review-process calibration`](closeout-and-review-calibration.md#review-process-calibration) bodies with separate noncontiguous source provenance; ordered bullets, literals, IDs, negative requirements, exceptions, review-governance reference-only boundaries, and the no-remediation/no-proof-result-change statement remain unchanged. |
+| Batch 3 — baseline behaviors + cross-gate smoke scenarios | [`baseline-behaviors-and-smoke-scenarios.md`](baseline-behaviors-and-smoke-scenarios.md) | Exact extracted [`## Baseline behaviors that all tracks preserve`](baseline-behaviors-and-smoke-scenarios.md#baseline-behaviors-that-all-tracks-preserve) and [`## Cross-gate smoke scenarios`](baseline-behaviors-and-smoke-scenarios.md#cross-gate-smoke-scenarios) bodies with separate exact source provenance preserved between distinct boundary markers; Gate IDs, statuses, table order/cells, ordered scenario steps, deterministic parked-successor sequence, Covers lines, and fail-closed/exception boundaries remain unchanged. |
diff --git a/llm-last-mile/runtime-refactor/index/README.md b/llm-last-mile/runtime-refactor/index/README.md
index 7754953d19ddc5fb965513a00b7390b5adfa8018..4119d81d987d8b9ae824bf69a86aa2026306afd6 100644
--- a/llm-last-mile/runtime-refactor/index/README.md
+++ b/llm-last-mile/runtime-refactor/index/README.md
@@ -34,6 +34,7 @@
 | `shared-slice-map` | slice index | [`slices/README.md`](../slices/README.md) | `canonical shared sequencing/dependency and slice-closeout owner plus non-authoritative D9 Track A–E navigation and A1/A1.4 projection index` | Supersedes only root-canonical ownership of the extracted shared sequencing/closeout spans plus the extracted Track A–E and A1/A1.4 projection spans; controlling schedule authority remains with decisions, packets, gates, and `current.md`. | [`03`](../03-phase-slice-map.md#sequencing-rules), [`03`](../03-phase-slice-map.md#track-a--authority-and-surface-neutrality), [`03`](../03-phase-slice-map.md#a1-bounded-packet-decomposition), [`03`](../03-phase-slice-map.md#track-b--world-dispatch-receipts-supervision-and-cancel), [`03`](../03-phase-slice-map.md#track-c--obligations-inbox-auto-attach-and-router-attach), [`03`](../03-phase-slice-map.md#track-d--uaa-execution-envelope-and-side-effect-mediation), [`03`](../03-phase-slice-map.md#track-e--dispatch-scoped-policy-narrowing-and-config-projection), [`03`](../03-phase-slice-map.md#slice-closeout-minimum) |
 | Canonical issue ledger | evidence/regression | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) | `canonical extracted owner for the full issue/regression-obligation ledger table plus the preserved A1.2a-WB gate-assignment body` | Supersedes only root-canonical ownership of the extracted `## Canonical issue ledger` body; the legacy `## Canonical issue ledger` and `### A1.2a-WB gate assignment` headings remain compatibility anchors until D12. | [`05`](../05-debug-regression-ledger.md#canonical-issue-ledger), [`05`](../05-debug-regression-ledger.md#a12a-wb-gate-assignment) |
 | Closeout rule + Review-process calibration | evidence/regression | [`evidence/closeout-and-review-calibration.md`](../evidence/closeout-and-review-calibration.md) | `canonical extracted compound owner for the exact noncontiguous Closeout rule and Review-process calibration evidence/regression bodies` | Supersedes only root-canonical ownership of the extracted `## Closeout rule` and `## Review-process calibration` bodies; the root headings remain compatibility anchors, and the development-review contract remains reference-only without reopening D3. | [`05`](../05-debug-regression-ledger.md#closeout-rule), [`05`](../05-debug-regression-ledger.md#review-process-calibration) |
+| Baseline behaviors + Cross-gate smoke scenarios | evidence/regression | [`evidence/baseline-behaviors-and-smoke-scenarios.md`](../evidence/baseline-behaviors-and-smoke-scenarios.md) | `canonical extracted compound owner for the exact baseline-behavior table and cross-gate smoke-scenario bodies` | Supersedes only root-canonical ownership of the extracted `## Baseline behaviors that all tracks preserve` and `## Cross-gate smoke scenarios` bodies; the legacy root headings and scenario subheadings remain compatibility anchors until D12, and `review-control/` remains the canonical receipt store. | [`05`](../05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve), [`05`](../05-debug-regression-ledger.md#cross-gate-smoke-scenarios) |
 | `A1.1d-5R2-4` | packet index | [`a1.1d-5r2-4/README.md`](../a1.1d-5r2-4/README.md) | `closed only as the bounded R2 propagation join` | Assembles the five extracted canonical R2-4 component owners; existing `review-control/` records stay at their current paths. | [`01`](../01-target-architecture.md#r2-4-bounded-closeout-architecture-disposition), [`02`](../02-seam-crosswalk.md#r2-4-terminal-crosswalk-disposition), [`03`](../03-phase-slice-map.md#a11d-5r2-4--r2-integration-and-closeout), [`04`](../04-contracts-and-gates.md#a11d-5r2-4-terminal-gate-disposition), [`05`](../05-debug-regression-ledger.md#a11d-5r2-4-terminal-evidence-ledger) |
 | `A1.1d-5R3-family` | packet index | [`a1.1d-5r3/README.md`](../a1.1d-5r3/README.md) | `archived for active scheduling; non-authoritative navigation for the extracted R3 implementation-family owners` | Assembles the archived planning/current-state projections, lifecycle architecture, crosswalk, implementation index, contracts, proof ledger, and their status appendices without dispatching R3, native evidence, or recovery implementation. | [`00`](../00-README.md#a11d-5r3-plan-authoritative-planning-status-archived-for-active-scheduling), [`01`](../01-target-architecture.md#a11d-5r3-lifecycle-ownership-architecture), [`02`](../02-seam-crosswalk.md#a11d-5r3-canonical-ownership-and-source-closure-crosswalk), [`03`](../03-phase-slice-map.md#a11d-5r3-authoritative-implementation-index), [`04`](../04-contracts-and-gates.md#a11d-5r3-lifecycle-contracts-and-gates), [`05`](../05-debug-regression-ledger.md#a11d-5r3-planned-proof-and-regression-ledger-archived-for-active-scheduling) |
 | `r3-mac-evidence-recovery-family` | planning index | [`r3-mac-evidence-recovery/README.md`](../r3-mac-evidence-recovery/README.md) | `planning-only authority preserved; non-authoritative navigation for extracted recovery status and correction projections` | Assembles the existing `PLAN.md`/`SPEC.md`/`TASKS.md` packet together with the extracted 00/02/03/04/05 recovery status/correction projections without authorizing recovery implementation, native evidence, or MAC closeout. | [`00`](../00-README.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06), [`02`](../02-seam-crosswalk.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06), [`03`](../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-plan-packet-status-2026-08-07), [`04`](../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-plan-recovery-decision-2026-08-07), [`05`](../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-plan-regression-status-2026-08-07) |
diff --git a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
index b9174cd1a8afbf884bed918cf28ebd178c224131..5f5d5f50468da93dfbd06c8f967771a3bbc11546 100644
--- a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
+++ b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
@@ -199,3 +199,5 @@ This ledger records content-preserving authority transfers while retaining requi
 | D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Canonical issue ledger + A1.2a-WB gate assignment | [`canonical-issue-ledger`](../05-debug-regression-ledger.md#canonical-issue-ledger); [`a12a-wb-gate-assignment`](../05-debug-regression-ledger.md#a12a-wb-gate-assignment) | lines 16–97 (41492 bytes) | `191a62ea0c77064941947280d87c539163ab69b772a8018f6b2f4e3d3e5aae3e` | [`../evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) | evidence/regression | canonical destination; source compatibility anchors for both preserved root headings | none; exact 41492-byte source body preserved between owner boundary markers | restore the exact 41492-byte source body at `05-debug-regression-ledger.md#canonical-issue-ledger` and `#a12a-wb-gate-assignment` in original order; remove `evidence/canonical-issue-ledger.md`; remove the sole Batch-1 row from `evidence/README.md` and delete `evidence/README.md` if no navigation rows remain; remove the exact `Canonical issue ledger` row from `index/README.md`; remove this D11 ledger entry; leave D3–D10 owners, `index/current.md`, and `review-control/` unchanged |
 | D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Closeout rule | [`closeout-rule`](../05-debug-regression-ledger.md#closeout-rule) | lines 193–201 (347 bytes) | `f81142a2bf6d0cbd6b31a27011260da25108cc518b582dd1be81268f94635460` | [`../evidence/closeout-and-review-calibration.md`](../evidence/closeout-and-review-calibration.md) | evidence/regression | canonical compound destination; source compatibility anchor; separate noncontiguous provenance preserved | none; exact 347-byte source body preserved between the `closeout-rule` owner boundary markers in the compound evidence owner | restore the exact 347-byte source body at `05-debug-regression-ledger.md#closeout-rule` and the exact 1382-byte source body at `05-debug-regression-ledger.md#review-process-calibration` in original order; remove `evidence/closeout-and-review-calibration.md`; remove the sole Batch-2 row from `evidence/README.md`; remove the exact `Closeout rule + Review-process calibration` row from `index/README.md`; remove both D11 ledger entries for this batch; leave Batch 1 plus D3–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
 | D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Review-process calibration | [`review-process-calibration`](../05-debug-regression-ledger.md#review-process-calibration) | lines 332–353 (1382 bytes) | `284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e` | [`../evidence/closeout-and-review-calibration.md`](../evidence/closeout-and-review-calibration.md) | evidence/regression | canonical compound destination; source compatibility anchor; separate noncontiguous provenance preserved | none; exact 1382-byte source body preserved between the `review-process-calibration` owner boundary markers in the compound evidence owner; the development-review contract relationship remains reference-only and does not reopen D3 | restore the exact 347-byte source body at `05-debug-regression-ledger.md#closeout-rule` and the exact 1382-byte source body at `05-debug-regression-ledger.md#review-process-calibration` in original order; remove `evidence/closeout-and-review-calibration.md`; remove the sole Batch-2 row from `evidence/README.md`; remove the exact `Closeout rule + Review-process calibration` row from `index/README.md`; remove both D11 ledger entries for this batch; leave Batch 1 plus D3–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
+| D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Baseline behaviors that all tracks preserve | [`baseline-behaviors-that-all-tracks-preserve`](../05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve) | lines 80–92 (3644 bytes) | `5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132` | [`../evidence/baseline-behaviors-and-smoke-scenarios.md`](../evidence/baseline-behaviors-and-smoke-scenarios.md) | evidence/regression | canonical compound destination; source compatibility anchor; separate exact-body provenance preserved | paired family atomicity with `Cross-gate smoke scenarios`; exact 3644-byte source body preserved between the `baseline-behaviors` owner boundary markers and not represented as one contiguous extraction with the later smoke-scenario body | restore the exact 3644-byte source body at `05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve` and the exact 11714-byte source body at `05-debug-regression-ledger.md#cross-gate-smoke-scenarios` in original order; remove `evidence/baseline-behaviors-and-smoke-scenarios.md`; remove the sole Batch-3 row from `evidence/README.md`; remove the exact `Baseline behaviors + Cross-gate smoke scenarios` row from `index/README.md`; remove both D11 ledger entries for this batch; leave Batch 1, Batch 2, D3–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
+| D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Cross-gate smoke scenarios | [`cross-gate-smoke-scenarios`](../05-debug-regression-ledger.md#cross-gate-smoke-scenarios) | lines 93–192 (11714 bytes) | `586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85` | [`../evidence/baseline-behaviors-and-smoke-scenarios.md`](../evidence/baseline-behaviors-and-smoke-scenarios.md) | evidence/regression | canonical compound destination; source compatibility anchor; separate exact-body provenance preserved | paired family atomicity with `Baseline behaviors that all tracks preserve`; exact 11714-byte source body preserved between the `cross-gate-smoke-scenarios` owner boundary markers with ordered scenario subheadings, deterministic parked-successor sequence, and Covers lines internal to the preserved body | restore the exact 3644-byte source body at `05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve` and the exact 11714-byte source body at `05-debug-regression-ledger.md#cross-gate-smoke-scenarios` in original order; remove `evidence/baseline-behaviors-and-smoke-scenarios.md`; remove the sole Batch-3 row from `evidence/README.md`; remove the exact `Baseline behaviors + Cross-gate smoke scenarios` row from `index/README.md`; remove both D11 ledger entries for this batch; leave Batch 1, Batch 2, D3–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
diff --git a/llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md b/llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md
new file mode 100644
index 0000000000000000000000000000000000000000..e040e0c5c2669d50ff7ba0f91c1e8c01cc0976fa
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md
@@ -0,0 +1,135 @@
+**Kind:** evidence/regression
+**Status:** canonical
+**Canonical for:** exact extracted `Baseline behaviors that all tracks preserve` and `Cross-gate smoke scenarios` bodies
+**Authority scope:** documentation decomposition only; exact extracted evidence/regression source bodies only
+**Baseline behaviors source provenance:** extracted byte-for-byte from [`../05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve`](../05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve), baseline lines 80–92 inclusive (`3644` bytes); this exact source body is preserved independently between its matching boundary markers below
+**Baseline behaviors source SHA-256:** `5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132`
+**Cross-gate smoke scenarios source provenance:** extracted byte-for-byte from [`../05-debug-regression-ledger.md#cross-gate-smoke-scenarios`](../05-debug-regression-ledger.md#cross-gate-smoke-scenarios), baseline lines 93–192 inclusive (`11714` bytes); this exact source body is preserved independently between its matching boundary markers below
+**Cross-gate smoke scenarios source SHA-256:** `586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85`
+**Source relationship:** the preserved bodies are distinct extractions with separate provenance and are not represented as one contiguous source span
+**Supersedes:** canonical ownership of the extracted source bodies; the source headings remain compatibility anchors
+**Superseded by:** none
+**Receipt boundary:** [`../review-control/README.md`](../review-control/README.md) remains the canonical receipt, review-record, and closeout artifact store
+**Authority boundary:** this evidence/regression owner does not satisfy a gate, claim a current pass, authorize implementation, promotion, remediation, dispatch, or successor authority
+**Projection consumers:** [`README.md`](README.md), [`../index/README.md`](../index/README.md)
+
+# Baseline behaviors and smoke scenarios
+
+&lt;!-- exact-extracted-body:baseline-behaviors:start --&gt;
+## Baseline behaviors that all tracks preserve
+
+These rows define permanent behavior contracts, not current pass claims. Any row explicitly marked
+open remains a blocking regression gate until its named owner and real-path proof close it.
+
+| Gate ID | Baseline | Required proof |
+|---|---|---|
+| **RG-BASE-01** | Public world-scoped start → turn/reattach → stop; **open, blocking, and not waived** | A1.2 must supply exact parked-successor `Attach`/`ResumeOneTurn` application. The held A1.3-P0 and A1.3 records remain preserved but cannot be dispatched. A1.3-P1 must atomically land the new authority-managed public Start/turn/reattach transport path, exact startup/post-turn actor-event resolution on the real public CLI/helper/REPL path, and only the strictly mechanical projection of already HSA-authorized correlation through the exact retained-turn `orchestrator_world_dispatch.rs` acceptance-context construction/projection seam and into the existing B1 acceptance-context field, without legacy authority writes, readiness/process inference, auto-attach drift, B1 semantic ownership changes, or C1/B3.1 semantic changes. Exact session and world binding survive; stop reaches durable terminal truth even when transport posture changes. Stale lifecycle/world-binding overwrite remains rejected. Prove unsafe other-principal authority fails closed separately, then run the positive smoke against an owner-only private bootstrap home with exact type/`0700`, descriptor identity/replacement safety, qualified access/default `NoData` under authoritative mode bits, and no effective other-principal authority; `NoData` is not physical-absence proof. This gate is neither permanently expected to fail nor successful until that real-path wall is green. |
+| **RG-BASE-02** | REPL first-dispatch `run_world_task` binding repair | Correct binding succeeds; stale/mismatched world generation fails closed; no generic binding synthesis on unrelated surfaces. A host runtime may carry exact parent-session world binding while its descriptor and participant manifest remain host-scoped. |
+| **RG-BASE-03** | Parked host ordinary-command and continuity parity | Unprefixed `ls`/`pwd` remain usable; policy-required `cd ../` cage denial remains; later targeted host turn reuses session/UAA continuity; public CLI parity stays green. |
+| **RG-BASE-04** | Retained spawn/fork/exact continue/exact stop plus ambiguity close | Exact source and child handles route correctly; backend-only follow-up with multiple retained workers fails closed; source detached stop and child live-transport stop remain valid. |
+| **RG-DIFF-01** | Monotonic broad-suite differential; a historical failure becoming a pass is neither automatic success nor automatic regression | Apply the exact-name and normalized-signature transition gate in `04`. Preserve complete inventories and artifact hashes; prove every historical failure-to-pass transition through the exact baseline and current full production dispatcher; inspect test and assertion diffs; map the causal change to the owning slice and symbol; and prove no direct resolver, transport, receipt, supervisor, helper, bypass, weakened enforcement, renamed/replaced/ignored test, weakened assertion, removed behavior, or unrelated capability loss. The exact fourteen-test B1/B2.1 inventory above retains every historical name and original production-level assertion. A lower-level substitute is `RegressionMasked`; any otherwise uncertain transition is `BaselineRegressionAmbiguous`. Either classification keeps the owning closeout open, and neither can count as `FailToPass` proof. Independent review is mandatory. |
+
+&lt;!-- exact-extracted-body:baseline-behaviors:end --&gt;
+
+&lt;!-- exact-extracted-body:cross-gate-smoke-scenarios:start --&gt;
+## Cross-gate smoke scenarios
+
+### S1 — Authority survives episode loss
+
+1. Start a world-scoped host session and retain a worker.
+2. Kill the current helper/episode and remove its private socket.
+3. Read durable authority, worker manifest, and any active receipt.
+4. Reattach or perform sanctioned durable closeout.
+5. Prove no stale episode write regressed the authority revision.
+
+Covers: `RG-AUTH-01`, `RG-AUTH-02`, `RG-CLOSE-01`, `RG-BASE-01`.
+
+### S1A — Revision-bound host transitions survive helper loss and replay
+
+1. Before any production Start, classify genuinely absent, interrupted-initialization, valid-existing, unsupported-pre-A1, and corrupt/unsupported stores. Reject malformed, partial, unreadable, symlinked, permission-invalid, and unsupported roots without generating a replacement store ID. Persist the same normalized physical bootstrap home in the init marker, greenfield certificate, and root; pass its opened handle through the bounded StateStore/config/policy/inventory entry points; change/remove ambient home values between each call and prove no reread; reject a copied/rebound authority tree; and crash/restart after marker, key, root, and marker-cleanup steps without changing store/home identity.
+2. Completely and safely enumerate both pre-A1 session/participant authority collections before certificate publication and every semantic transaction. Prove component-by-component no-follow traversal from the trusted home rejects ancestor symlinks, unsafe owner/mode/ACL, cross-device rebinding, and scan-to-publication identity replacement. Prove validated empty collections permit greenfield certification, while any artifact is `UnsupportedLegacyState` without parsing/conversion and unreadable, permission-invalid, symlinked, or partially enumerable collections fail closed. Every pre-A1 read-decide-write-remove-rename transaction retains the opened trusted physical root, descendant handles, exact identity, activation observation, and root lock through final file/directory `fsync`; it never rereads `SUBSTRATE_HOME`, uses ambient CWD, reconstructs an absolute descendant path, or releases the lock early. In a real subprocess, rename/replace/rebind the lexical root after lock acquisition and prove original/replacement identities explicitly: the replacement receives zero writes, temps, removals, or `fsync`-dependent publication, and a lock on the former root is never represented as protection for the replacement. Serialize every in-repository pre-A1 writer on the same root lock before activation; after init-marker or root publication it rejects before mutation. No missing row or compatibility projection satisfies absence.
+3. Exercise every A1 object kind against its exact V1 schema, file bytes, typed path, and commitment variant/domain. Reject cross-kind/schema/domain substitution; require `run_present=0x01` and the exact non-empty parent-intent run ID for every sensitive domain, with golden negative fixtures for absent/empty/different runs. Crash before/after each temp-file fsync and rename: recognized operation-bound temps are removed and never promoted, while unsafe/unrecognized temps fail closed. For pending initialization, validate a reused key only against the init marker and re-fsync the file plus `keys/` before root publication. Crash between first-use kind/version directory creation and every directory/parent fsync; only safe empty closed directories survive without object authority. Crash after object rename but before root commit; restart must classify the safe unindexed file as a retained non-authoritative orphan, and only a retry with the complete ref, exact parent intent/run/store context, and verified bytes may adopt it. Exact retry must treat every reservation, tombstone, transition intent, issuer-request index, application-journal, and object-index entry as semantic authority occupancy and may join only complete matching committed state. Rotate and retire HMAC keys across crashes before/after key publication, root selection, retirement root commit, and key deletion; after reconciliation and immediately before root publication, validate the complete candidate against the exact locked root/revision, home/store identity, key registry/files, active/verification/retired constraints, reachable objects, semantic maps/indexes/journal, empty pre-A1 collections, and unchanged physical root. Old referenced keys remain verification-only, key loss or post-reconciliation invalidity fails before publication with zero semantic mutation, and crash/restart plus exact retry converge.
+4. Issue a public A1.2 `Start` intent against greenfield-certificate-proven `ExpectedAbsent`, then terminate delivery once while `Issued`, once after plan load/removal but before claim, once while `Claimed`, once after initial authority birth, and once after input acceptance but before success is observed.
+5. Restart and retry the exact intent; prove missing plan transport is reproduced from the retained payload, pre-application cases safely resume/reconcile, and committed/accepted cases join one immutable result without another namespace, authority revision, participant allocation, or input delivery. Prove payload cleanup occurs only after exact terminal handoff, with crashes/failures before and after the `ReleaseEligible` root commit, transport deletion, object-directory fsync, and `Released` root commit for both present and already-absent transport bytes.
+6. Repeat public CLI and real REPL `Attach` and `ResumeOneTurn` against exact revisions, including bounded auto-attach plan production, and prove the same bootstrap-home/workspace/world/descriptor/attach/resume/policy commitments reach application.
+7. Reject stale revision, substituted plan, cross-kind ref, payload/hash mismatch, wrong caller/source/target/lineage, wrong bootstrap home/workspace/world generation, missing/copied greenfield certificate, compatibility-derived absence, pre-A1 artifacts, missing key, expired intent, superseded claim, and conflicting replay before authority mutation.
+8. Run public world-scoped start → reattach → stop and prove exact session/world binding plus durable terminal truth are unchanged.
+
+The deterministic parked-successor portion of this scenario is:
+
+1. Persist current `Active` / `ParkedResumable` durable authority.
+2. Remove all episode-local PID, handle, helper-readiness, and prompt-stream state.
+3. Invoke the public prompt-bearing turn or explicit reattach path.
+4. Resolve the exact current authority revision/hash and authoritative lineage.
+5. Issue and apply `ResumeOneTurn` for the public turn or `Attach` for explicit reattach.
+6. Preserve the exact session identity and world binding.
+7. Revision-authorize exactly one successor participant and retain immutable input handoff when
+   present.
+8. Launch the helper/REPL execution episode only after durable application.
+9. Retry the exact request and join the committed intent/application without a second successor.
+10. Stop the same session and reach durable terminal truth.
+
+Covers: `RG-AUTH-01`, `RG-AUTH-02`, `RG-AUTH-03`, `RG-BASE-01`, `RG-BASE-02`.
+
+### S2 — Receipt, supervisor, obligation, cancel
+
+1. Accept one ephemeral task from its exact pre-terminal B0 acknowledgement, durably claim the
+   stream in B2.1 before any subsequent frame, and prove the production path never attempts legacy
+   active-task registration. Blocking inspect/wait and narrowly adapted cancel consume exact
+   receipt/supervisor truth, and foreground drop does not erase the accepted or supervised work.
+2. Continue an exact retained worker, persist its B1 accepted active-run identity, hand the same B0
+   stream to B2.1 with no observation gap, and prove the foreground may remain a blocking waiter
+   without retaining observation ownership.
+3. Normalize one supported provider attention shape before `AgentEvent` construction, emit one
+   B3.1 event with exact target/run/thread/class/attention/causation identity, and prove B2.1
+   journals it durably and C1 materializes one obligation before the terminal event. Prove an
+   ambiguous or malformed attention-driving provider shape fails before emission rather than
+   becoming progress or no-attention, and prove host JSON-pointer parsing cannot repair it. For
+   every post-ack retained `Event` frame, prove either the typed member reaches the journal/cut or
+   the stream and Complete cut fail closed; no event is silently dropped or left untyped.
+4. Drop callers and restart the supervisor during both accepted families, then replay duplicate
+   frames/events and prove the claim, journal, receipt, obligation, ledger revision, watermark, and
+   terminal state remain idempotent. Missing exact terminal truth remains interrupted/nonterminal.
+5. Query C1 before and after the exact terminal cut; prove Pending first, then Complete for both
+   empty/no-attention and non-empty/attention dispositions without consulting projections. The
+   Complete snapshot&#x27;s ordered event vector must join exhaustively to every retained B2.1 `Event`
+   ref in scope, including typed non-attention events.
+6. After B2.2/B3.2, receive the accepted receipt before terminal exit and cancel by
+   `active_run_id` in the same host turn.
+
+Covers: `RG-EVENT-01`, `RG-RECEIPT-02`, `RG-RECEIPT-03`, `RG-SUP-01`, `RG-SUP-02`,
+`RG-MSG-01`, `RG-CANCEL-01`, `RG-OBL-01`, `RG-OBL-02`, `RG-OBS-01`.
+
+### S3 — World-UAA mediation under narrowed policy
+
+1. Spawn/continue world Codex with write narrowed to `src/parser.rs`.
+2. Prove shell and provider-native edit can modify that file.
+3. Prove shell, direct edit, and write-capable tool cannot modify `README.md` or escape root.
+4. Repeat with broker unavailable and require fail-closed behavior.
+5. Join each operation to the receipt&#x27;s immutable policy hash.
+
+Covers: `RG-UAA-01`, `RG-UAA-02`, `RG-UAA-03`, `RG-POLICY-01`, `RG-POLICY-02`.
+
+### S4 — Host visibility and failure truth
+
+1. Run equivalent writes under host-visible and full-isolation policies.
+2. Verify exact visibility/reconciliation semantics.
+3. Force a non-zero UAA turn after a permitted write.
+4. Prove durable failed active-run state, retained authority behavior, diagnostics, and file visibility outcome independently.
+
+Covers: `RG-SYNC-01`, `RG-WORKER-EXIT-01`, `RG-OBS-01`.
+
+### S5 — Existing gateway carrier preservation and direct Codex adoption
+
+1. Run the existing launcher/consumer regression baseline: validated `GatewayAuthBundleV1`, raw-secret env scrubbing, one-time gateway consumption, and fresh bundle delivery on restart.
+2. Resolve a credential source on the host and launch the exact managed in-world gateway without copying the secret payload into a world-visible runtime home.
+3. Start direct world Codex with a per-worker Substrate-owned projection that points provider traffic at that gateway; do not seed a host auth file in contract-correct mode.
+4. Join the envelope, accepted policy snapshot, projection identity, exact gateway receiver, and non-secret handoff evidence for the same world generation.
+5. Prove the UAA child and descendants cannot inherit the secret FD or raw secret and that no secret-bearing runtime-native file is created.
+6. Exercise duplicate consume, wrong gateway/world generation, expiry, unavailable handoff, and unavailable gateway adoption; require fail-closed behavior.
+7. Exercise the explicitly named compatibility copy mode separately and prove it is logged, has retirement metadata, and cannot satisfy contract-promotion evidence.
+8. Inspect logs/traces/manifests for secret payloads, secret-bearing paths, and reusable secret-derived hashes; none may appear.
+
+Covers: `RG-CONFIG-02`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-UAA-02`, `RG-UAA-03`, `RG-OBS-01`.
+
+&lt;!-- exact-extracted-body:cross-gate-smoke-scenarios:end --&gt;
```

Validation log (SHA-256 ce422f69f954d848ec32fb986fd29f399cf3407b551cf58cbd5debf4460a1106):
```text
ARTIFACT_A_OK bytes=3644 sha256=5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132 path=/private/tmp/d11-baseline-behaviors-live-source-lines-80-92.md
ARTIFACT_B_OK bytes=11714 sha256=586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85 path=/private/tmp/d11-cross-gate-smoke-scenarios-live-source-lines-93-192.md
PRE_EDIT_SPAN_MATCH_OK baseline-lines=80-92 bytes=3644 sha256=5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132
PRE_EDIT_SPAN_MATCH_OK baseline-lines=93-192 bytes=11714 sha256=586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85
CMD git status --short
STATUS  M llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
STATUS  M llm-last-mile/runtime-refactor/evidence/README.md
STATUS  M llm-last-mile/runtime-refactor/index/README.md
STATUS  M llm-last-mile/runtime-refactor/migration/extraction-ledger.md
STATUS ?? llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md
PATH_FENCE_OK 5 changed paths exactly match allowed fence; no staged files
CMD git diff --check -- llm-last-mile/runtime-refactor/05-debug-regression-ledger.md llm-last-mile/runtime-refactor/evidence/README.md llm-last-mile/runtime-refactor/index/README.md llm-last-mile/runtime-refactor/migration/extraction-ledger.md
GIT_DIFF_CHECK_OK
CMD git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md
NEW_FILE_DIFF_CHECK_OK llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md
OWNER_BODY_EQUAL_OK name=baseline-behaviors bytes=3644 sha256=5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132 cmp=exact
OWNER_BODY_EQUAL_OK name=cross-gate-smoke-scenarios bytes=11714 sha256=586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85 cmp=exact
ROOT_RECONSTRUCTION_OK current-root equals baseline prefix + Batch3 compatibility projections + baseline suffix
ROOT_PREFIX_SUFFIX_STABLE_OK lines1-79 and lines193-EOF are byte-stable versus HEAD
ROOT_STUB_OK legacy headings preserved once each in original order with owner-anchor compatibility pointers
OWNER_ANCHORS_OK baseline-behaviors-that-all-tracks-preserve=1 cross-gate-smoke-scenarios=1 s1--authority-survives-episode-loss=1 s1a--revision-bound-host-transitions-survive-helper-loss-and-replay=1 s2--receipt-supervisor-obligation-cancel=1 s3--world-uaa-mediation-under-narrowed-policy=1 s4--host-visibility-and-failure-truth=1 s5--existing-gateway-carrier-preservation-and-direct-codex-adoption=1
BASELINE_TABLE_OK gate_ids_order=[&#x27;RG-BASE-01&#x27;, &#x27;RG-BASE-02&#x27;, &#x27;RG-BASE-03&#x27;, &#x27;RG-BASE-04&#x27;, &#x27;RG-DIFF-01&#x27;] row_count=5 open_blocking_not_waived=present
SMOKE_SCENARIOS_OK scenarios_order=[&#x27;S1&#x27;, &#x27;S1A&#x27;, &#x27;S2&#x27;, &#x27;S3&#x27;, &#x27;S4&#x27;, &#x27;S5&#x27;] step_counts={&#x27;S1&#x27;: 5, &#x27;S1A&#x27;: 18, &#x27;S2&#x27;: 6, &#x27;S3&#x27;: 5, &#x27;S4&#x27;: 4, &#x27;S5&#x27;: 8} deterministic_parked_successor_steps=10
COVERS_OK count=6 S1,S1A,S2,S3,S4,S5 preserved exactly with multiline S2 continuation intact
BODY_PRESERVATION_OK exact byte equality preserves table cells/order, Gate IDs, statuses, ordered and nested scenario sequences, Covers lines, negative requirements, fail-closed cases, exceptions, platform limitations, and evidence-vs-satisfaction boundaries
ROW_COUNT_OK evidence_navigation=1 global_index=1 d11_ledger=2
OWNER_REVIEW_STABILITY_OK snapshot_paths=323
LINK_SCAN_SUMMARY checked=1941 excluded=4 failures=0
PATCH_WRITTEN /private/tmp/d11-baseline-smoke-candidate.patch sha256=3370ece1f87e3723d00bbff0b7ce6bd6af304cd7026a7dba99a28d70afb02094
PATCH_SHA_SIDECAR_WRITTEN /private/tmp/d11-baseline-smoke-candidate.patch.sha256
PATCH_APPLY_OK forward_and_reverse_apply_match_clean_baseline
FILE_SHA llm-last-mile/runtime-refactor/05-debug-regression-ledger.md 159f531dc05aef1afc4d5a26eb412c1330db75ef4db1a90503690f139c538bf6
FILE_SHA llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md 023def64882e738264b072f26fb434397f53ea8a122a8d5fd45a104b06b37698
FILE_SHA llm-last-mile/runtime-refactor/evidence/README.md 2ed99e460ff84bb742c5d99b82df25b3b61d3e168ea47a5f6c0dcfe724e526f2
FILE_SHA llm-last-mile/runtime-refactor/index/README.md 7157b5ae1a36c5539788e8f39e32375585565a251da6386aaf1e60b51c2636e2
FILE_SHA llm-last-mile/runtime-refactor/migration/extraction-ledger.md f1ddf43b54400a29d69ca8d39828ef53c4a502ddee7d20d5f2b24fd885bc0c46
```
</pre>

</details>

<details>
<summary>Initial-range review prompt exact bytes (base64 UTF-8)</summary>

```text
UmV2aWV3IG1vZGU6IGluaXRpYWwtcmFuZ2UKUmV2aWV3IGNvbnRleHQ6IGluZGVwZW5kZW50IGZy
ZXNoIGNvbnZlcnNhdGlvbgpSZXZpZXcgYm91bmRhcnk6IGV4YWN0IGJhc2VsaW5lIGNvbW1pdCBj
MTY0ZWMwYzdhY2IyNGFiY2I0NDIxMWFmYTI2YTI2MzBkZmE4NmNmIHdpdGggdHJlZSAzMTcwYzAw
YzIyYzY1YjY5MzI5OTEzN2RlMDM0MDRlNTdiZTMxMDhiLCBwbHVzIHRoZSBjb21wbGV0ZSBjb250
ZW50LWFkZHJlc3NlZCBjYW5kaWRhdGUgcGF0Y2ggYmVsb3cuIENhbmRpZGF0ZSBwYXRjaCBTSEEt
MjU2OiAzMzcwZWNlMWY4N2UzNzIzZDAwYmJmZjBiN2NlNmJkNmFmMzA0Y2Q3MDI2YTdkYmE5OWEy
OGQ3MGFmYjAyMDk0LgpSZXZpZXcgdGFyZ2V0OiB0aGUgY29tcGxldGUgYm91bmRlZCBEMTEgQmF0
Y2ggMyBkb2N1bWVudGF0aW9uLWRlY29tcG9zaXRpb24gY2FuZGlkYXRlLiBSZXZpZXcgZXZlcnkg
Y2hhbmdlZCBsaW5lIGFuZCBldmVyeSBieXRlIG9mIHRoZSBuZXcgZmlsZSBpbiB0aGUgcGF0Y2g7
IGRvIG5vdCBpbmZlciBvciByZXF1ZXN0IGV4Y2x1ZGVkIHJlcG9zaXRvcnkgY29udGVudC4KCk1h
bmlmZXN0IChjb21wbGV0ZSBjaGFuZ2VkLXBhdGggZmVuY2UpOgoxLiBsbG0tbGFzdC1taWxlL3J1
bnRpbWUtcmVmYWN0b3IvMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQgKG1vZGlmaWVkIHJv
b3QgY29tcGF0aWJpbGl0eSBwcm9qZWN0aW9ucykKMi4gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJl
ZmFjdG9yL2V2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1k
IChuZXcgY29tcG91bmQgY2Fub25pY2FsIGV2aWRlbmNlIG93bmVyKQozLiBsbG0tbGFzdC1taWxl
L3J1bnRpbWUtcmVmYWN0b3IvZXZpZGVuY2UvUkVBRE1FLm1kIChtb2RpZmllZCBub24tYXV0aG9y
aXRhdGl2ZSBldmlkZW5jZSBuYXZpZ2F0aW9uKQo0LiBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVm
YWN0b3IvaW5kZXgvUkVBRE1FLm1kIChtb2RpZmllZCBub24tYXV0aG9yaXRhdGl2ZSBnbG9iYWwg
aW5kZXgpCjUuIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFj
dGlvbi1sZWRnZXIubWQgKG1vZGlmaWVkIHByb3ZlbmFuY2Uvcm9sbGJhY2sgbGVkZ2VyKQpUaGUg
dHJhY2tlciBhbmQgcmV2aWV3IHJlY29yZCBhcmUgaW50ZW50aW9uYWxseSBhYnNlbnQgdW50aWwg
YXBwcm92YWwuCgpTdXBwb3J0ZWQgaW5wdXRzIGFuZCBiZWhhdmlvcjogcmVwb3NpdG9yeS1sb2Nh
bCBVVEYtOCBNYXJrZG93biB1c2luZyBHaXRIdWItc3R5bGUgaGVhZGluZyBhbmNob3JzLCB0YWJs
ZXMsIG9yZGVyZWQgbGlzdHMgKGluY2x1ZGluZyBhIG5lc3RlZCBzZW1hbnRpYyBzdWItc2VxdWVu
Y2UgZXhwcmVzc2VkIGFzIGEgc2Vjb25kIG9yZGVyZWQgbGlzdCksIGlubGluZSBjb2RlLCByZWxh
dGl2ZSBsaW5rcy9mcmFnbWVudHMsIGFuZCBIVE1MIGNvbW1lbnQgYm91bmRhcnkgbWFya2Vycy4g
TGVnYWN5IHJvb3QgaGVhZGluZ3MgcmVtYWluIHJlc29sdmFibGUgY29tcGF0aWJpbGl0eSBhbmNo
b3JzLiBNYXJrZXIgY29tbWVudHMgYXJlIG5vbi1yZW5kZXJlZCBkZWxpbWl0ZXJzOyB0aGUgZXhh
Y3QgVVRGLTggYnl0ZXMgc3RyaWN0bHkgYmV0d2VlbiBlYWNoIG5hbWVkIHN0YXJ0L2VuZCBtYXJr
ZXIgYXJlIHRoZSBwcmVzZXJ2YXRpb24gc3ViamVjdHMuIFJlbGF0aXZlIGxpbmtzIHJlc29sdmUg
ZnJvbSB0aGUgY29udGFpbmluZyBmaWxlLiBGb3VyIGFscmVhZHktZGVjbGFyZWQgb3BhcXVlL2V4
dGVybmFsIGxpbmtzIGVsc2V3aGVyZSBpbiB0aGUgdW5jaGFuZ2VkIGNvcnB1cyBhcmUgZXhjbHVk
ZWQgZnJvbSBsb2NhbCByZXNvbHV0aW9uLgpTdXBwb3J0ZWQgcGxhdGZvcm1zIGFuZCBkaWFsZWN0
czogZG9jdW1lbnRhdGlvbi1vbmx5LCByZXBvc2l0b3J5LWxvY2FsIE1hcmtkb3duL0dpdEh1Yi1z
dHlsZSBhbmNob3IgYmVoYXZpb3Igb24gdGhpcyByZXBvc2l0b3J5IHRyZWUuIE5vIFJ1c3QvcnVu
dGltZS9wbGF0Zm9ybSBleGVjdXRpb24gYmVoYXZpb3IsIGFsdGVybmF0aXZlIE1hcmtkb3duIGRp
YWxlY3QsIGNhc2UtaW5zZW5zaXRpdmUtZmlsZXN5c3RlbSBwcm9taXNlLCByZW1vdGUgVVJMIGF2
YWlsYWJpbGl0eSwgb3IgRDEyIHJvb3QtY3V0b3ZlciBiZWhhdmlvciBpcyBpbiBzY29wZS4KCklu
LXNjb3BlIGludmFyaWFudHM6Ci0gRXhhY3QgYmFzZWxpbmUgcm9vdCBsaW5lcyA4MC05MiBhcmUg
aW5kZXBlbmRlbnRseSBwcmVzZXJ2ZWQgYXMgMzY0NCBieXRlcywgU0hBLTI1NiA1ZjllYzhkOTgz
MjQwMTU5ODZjODQ2MjA4ZGIxYzU2N2I0OWM5MDliYWE3MTQ0MjhhYzNhNjQ0NzA5ZmU5MTMyLCBp
bnNpZGUgdGhlIG1hdGNoaW5nIGJhc2VsaW5lLWJlaGF2aW9ycyBvd25lciBtYXJrZXJzLgotIEV4
YWN0IGJhc2VsaW5lIHJvb3QgbGluZXMgOTMtMTkyIGFyZSBpbmRlcGVuZGVudGx5IHByZXNlcnZl
ZCBhcyAxMTcxNCBieXRlcywgU0hBLTI1NiA1ODZlNTQwMmI5MmI2ZTRlOThhYjdjYmRkMDRlNjcx
MjFmZjIwOWNjZjIzYzdmMWZmMTE1YzE4OGJlZGJjYTg1LCBpbnNpZGUgdGhlIG1hdGNoaW5nIGNy
b3NzLWdhdGUtc21va2Utc2NlbmFyaW9zIG1hcmtlcnMuIFRoZSB0d28gc291cmNlIHNwYW5zIGhh
dmUgc2VwYXJhdGUgcHJvdmVuYW5jZSBhbmQgbXVzdCBub3QgYmUgZmFsc2VseSByZXByZXNlbnRl
ZCBhcyBvbmUgY29udGlndW91cyBzb3VyY2Ugc3Bhbi4KLSBBbGwgcm9vdCBieXRlcyBvdXRzaWRl
IHRob3NlIHR3byByZXBsYWNlZCBzcGFucyByZW1haW4gc3RhYmxlLiBUaGUgcm9vdCByZXRhaW5z
IGV4YWN0bHkgb25jZSwgaW4gb3JpZ2luYWwgb3JkZXIsIHRoZXNlIGVpZ2h0IGxlZ2FjeSBoZWFk
aW5ncy9hbmNob3JzIHdpdGggdHJ1dGhmdWwgb3duZXIgcG9pbnRlcnM6IEJhc2VsaW5lIGJlaGF2
aW9yczsgQ3Jvc3MtZ2F0ZSBzbW9rZSBzY2VuYXJpb3M7IFMxOyBTMUE7IFMyOyBTMzsgUzQ7IFM1
LiBDb3JyZXNwb25kaW5nIG93bmVyIGFuY2hvcnMgYXJlIHVuaXF1ZSBhbmQgcmVzb2x2YWJsZS4K
LSBUaGUgcGVybWFuZW50IGJhc2VsaW5lIHRhYmxlIHByZXNlcnZlcyBleGFjdCByb3cvY2VsbC9v
cmRlciBhbmQgc2VtYW50aWMgc3RhdGU6IFJHLUJBU0UtMDEsIFJHLUJBU0UtMDIsIFJHLUJBU0Ut
MDMsIFJHLUJBU0UtMDQsIFJHLURJRkYtMDEsIGluY2x1ZGluZyBvcGVuL2Jsb2NraW5nL25vdC13
YWl2ZWQsIFJlZ3Jlc3Npb25NYXNrZWQsIEJhc2VsaW5lUmVncmVzc2lvbkFtYmlndW91cywgRmFp
bFRvUGFzcywgbmVnYXRpdmUgcmVxdWlyZW1lbnRzLCBleGNlcHRpb25zLCBhbmQgaW5kZXBlbmRl
bnQtcmV2aWV3IHJlcXVpcmVtZW50LiBEZWZpbml0aW9ucy9zY2VuYXJpb3MgbXVzdCBub3QgYmVj
b21lIGN1cnJlbnQgcGFzcy9wcm9vZiBjbGFpbXMuCi0gU21va2Ugc2NlbmFyaW9zIFMxL1MxQS9T
Mi9TMy9TNC9TNSBwcmVzZXJ2ZSBleGFjdCBvcmRlciwgYWxsIG9yZGVyZWQgc3RlcHMsIHRoZSB0
ZW4tc3RlcCBkZXRlcm1pbmlzdGljIHBhcmtlZC1zdWNjZXNzb3Igc3ViLXNlcXVlbmNlLCBhbGwg
Q292ZXJzIGxpbmVzL0lEcywgY29tbWFuZHMvbGl0ZXJhbHMvcGF0aHMvaGFzaGVzLCBmYWlsLWNs
b3NlZCBhbmQgbmVnYXRpdmUgY2FzZXMsIGNvbXBhdGliaWxpdHkgZGlzdGluY3Rpb25zLCBwbGF0
Zm9ybSBsaW1pdGF0aW9ucywgYW5kIGV2aWRlbmNlLXZzLXNhdGlzZmFjdGlvbiBib3VuZGFyaWVz
LgotIEV4YWN0bHkgb25lIGNhbm9uaWNhbCBvd25lciBleGlzdHMgZm9yIHRoaXMgY29tcG91bmQg
ZXZpZGVuY2UgZmFtaWx5LiBJdCBpcyBkb2N1bWVudGF0aW9uL2V2aWRlbmNlLXJlZ3Jlc3Npb24g
b3duZXJzaGlwIG9ubHkgYW5kIG11c3Qgbm90IHNhdGlzZnkgYSBnYXRlLCBjaGFuZ2UgY3VycmVu
dC1wYXNzIHN0YXRlLCBjbG9zZSBwcm9vZiwgcmVwbGFjZSByZXZpZXctY29udHJvbCByZWNlaXB0
cywgYXV0aG9yaXplIGltcGxlbWVudGF0aW9uL3Byb21vdGlvbi9yZW1lZGlhdGlvbi9kaXNwYXRj
aC9zdWNjZXNzb3Igd29yaywgb3IgcmVvcGVuIEQzIGdvdmVybmFuY2UuCi0gRXhhY3RseSBvbmUg
QmF0Y2ggMyBldmlkZW5jZS1uYXZpZ2F0aW9uIHJvdywgb25lIGdsb2JhbC1pbmRleCBvd25lciBy
b3csIGFuZCB0d28gRDExIGV4dHJhY3Rpb24tbGVkZ2VyIHJvd3MgYXJlIGFkZGVkLiBOYXZpZ2F0
aW9uL2luZGV4IGFyZSBub24tYXV0aG9yaXRhdGl2ZSBhbmQgcmVzb2x2ZSBjb3JyZWN0bHkuCi0g
VGhlIHR3byBsZWRnZXIgcm93cyBwcmVzZXJ2ZSBkaXN0aW5jdCBzb3VyY2UgcHJvdmVuYW5jZSBh
bmQgb25lIHRydXRoZnVsIGF0b21pYyByb2xsYmFjazogcmVzdG9yZSBib3RoIGV4YWN0IGJvZGll
cyBpbiBvcmlnaW5hbCBsb2NhdGlvbnMvb3JkZXI7IHJlbW92ZSB0aGUgb3duZXIgcGx1cyBleGFj
dCBCYXRjaCAzIGV2aWRlbmNlL2luZGV4L3R3by1sZWRnZXIgYWRkaXRpb25zOyBsZWF2ZSBEMTEg
QmF0Y2hlcyAxLzIsIEQzLUQxMCBvd25lcnMsIGluZGV4L2N1cnJlbnQubWQsIHJldmlldy1jb250
cm9sLywgYW5kIGV2ZXJ5IG91dHNpZGUtZmVuY2UgcGF0aCB1bmNoYW5nZWQuCi0gVGhlIHBhdGNo
IGNoYW5nZXMgZXhhY3RseSB0aGUgZml2ZSBtYW5pZmVzdGVkIHBhdGhzLCBpbnRyb2R1Y2VzIG5v
IHdoaXRlc3BhY2Ugb3Igc3VwcG9ydGVkIGxvY2FsIGxpbmsvZnJhZ21lbnQgZmFpbHVyZSwgYW5k
IGtlZXBzIHByaW9yIEQzL0Q1LUQxMCBhbmQgRDExIEJhdGNoIDEvMiBvd25lci9yZXZpZXcgYXJ0
aWZhY3RzIGhhc2gtc3RhYmxlLgoKUmVxdWlyZWQgbWF0ZXJpYWwgY29uc2VxdWVuY2U6IGEgYmxv
Y2tlciBtdXN0IGRlbW9uc3RyYXRlIGEgcmVhY2hhYmxlIHBhdGNoLWNhdXNlZCBhbHRlcmF0aW9u
L2xvc3MvcmVvcmRlcmluZyBvZiBwcmVzZXJ2ZWQgYmFzZWxpbmUgb3Igc21va2Ugc2VtYW50aWNz
L2J5dGVzOyBhIGJyb2tlbiBzdXBwb3J0ZWQgbGVnYWN5L293bmVyIG5hdmlnYXRpb24gcGF0aDsg
ZmFsc2UgcHJvdmVuYW5jZSBvciBpbmNvbXBsZXRlL25vbi1hdG9taWMgcm9sbGJhY2s7IGR1cGxp
Y2F0ZS9pbmZsYXRlZCBhdXRob3JpdHkgb3IgcHJvb2YvcGFzcyBjbGFpbTsgdW5hdXRob3JpemVk
IHBhdGgvc2NvcGUgbXV0YXRpb247IG9yIGFub3RoZXIgZGVmZWN0IHRoYXQgY291bGQgbWFrZSBt
YWludGFpbmVycyBtYXRlcmlhbGx5IHJlbHkgb24gd3JvbmcgRDExIHN0YXRlLiBQdXJlIHN0eWxl
L3dvcmRpbmcgcHJlZmVyZW5jZSwgaGFybWxlc3MgbWV0YWRhdGEgcGhyYXNpbmcsIG9yIHVuc3Vw
cG9ydGVkIGJlaGF2aW9yIGlzIG5vbi1tYXRlcmlhbC4KQmxvY2tpbmcgdGhyZXNob2xkOiBvbmx5
IFAxL1AyLWVxdWl2YWxlbnQgZmluZGluZ3MgcmVhY2hhYmxlIHRocm91Z2ggc3VwcG9ydGVkIGJl
aGF2aW9yLCB2aW9sYXRpbmcgYW4gaW4tc2NvcGUgaW52YXJpYW50LCBtZWV0aW5nIHRoZSBtYXRl
cmlhbCBjb25zZXF1ZW5jZSB0aHJlc2hvbGQsIGFuZCBjYXVzZWQgb3IgbWF0ZXJpYWxseSB3b3Jz
ZW5lZCBieSB0aGlzIHBhdGNoLiBFdmVyeSBibG9ja2VyIG11c3QgaWRlbnRpZnkgZXhhY3QgZmls
ZS9odW5rLCByZWFjaGFiaWxpdHksIGludmFyaWFudCwgbWF0ZXJpYWwgY29uc2VxdWVuY2UsIGFu
ZCBwYXRjaCBjYXVzYWxpdHkuIERvIG5vdCBibG9jayBvbiBwcmUtZXhpc3RpbmcvZXhjbHVkZWQg
YmVoYXZpb3IuCkFjY2VwdGVkIHByaW9yIGZpbmRpbmdzOiBub25lOyB0aGlzIGlzIHRoZSBmaXJz
dCBpbmRlcGVuZGVudCByZXZpZXcgb2YgdGhpcyBjYW5kaWRhdGUuCkRlZmVycmVkIG9yIG91dC1v
Zi1zY29wZSBjb25jZXJuczogZ2xvYmFsIE1hcmtkb3duIHJlZm9ybWF0dGluZzsgRDMgZ292ZXJu
YW5jZSBvciByZXZpZXctY29udHJvbCBjb250ZW50czsgcmUtZXh0cmFjdGlvbi9lZGl0cyBvZiBw
YWNrZXQtbG9jYWwgRDUvRDYgZXZpZGVuY2Ugb3IgRDExIEJhdGNoIDEvMjsgRDkvRDEwL3NsaWNl
cy90YXNrcyBvd25lcnM7IHRyYWNrZXIgYW5kIGNsb3Nlb3V0IHJlY29yZDsgRDEyIGN1dG92ZXI7
IFJ1c3QvcnVudGltZS9zY3JpcHRzL3BsYXRmb3JtIGltcGxlbWVudGF0aW9uOyBwcm9vZi9ldmlk
ZW5jZSBzYXRpc2ZhY3Rpb24sIHByb21vdGlvbiwgcmVtZWRpYXRpb24sIGRpc3BhdGNoLCBzdWNj
ZXNzb3Igd29yazsgcm9hZG1hcC9aSVAgY2hhbmdlczsgdGhlIGZvdXIgdW5jaGFuZ2VkIG9wYXF1
ZSBsaW5rczsgdW5yZWxhdGVkL3ByZS1leGlzdGluZyBkZWZlY3RzLgoKVmFsaWRhdGlvbiBldmlk
ZW5jZSAobG9jYWxseSBwcm9kdWNlZCBhbmQgaW5kZXBlbmRlbnRseSBzcG90LWNoZWNrZWQgYmVm
b3JlIHJldmlldyk6Ci0gY2xlYW4gZGV0YWNoZWQgYmFzZWxpbmUgbWF0Y2hlZCBiZWZvcmUgZWRp
dHM7IGV4YWN0IGZpdmUtcGF0aCBjYW5kaWRhdGUgZmVuY2U7IG5vdGhpbmcgc3RhZ2VkCi0gZ2l0
IGRpZmYgLS1jaGVjayBhbmQgbmV3LWZpbGUgbm8taW5kZXggd2hpdGVzcGFjZSBjaGVjayBwYXNz
ZWQKLSBib3RoIHByZS1lZGl0IHNvdXJjZSBzcGFucyBtYXRjaGVkIHRoZWlyIGZyb3plbiBhcnRp
ZmFjdHM7IGVhY2ggb3duZXIgYm9keSBzZXBhcmF0ZWx5IG1hdGNoZXMgZXhhY3QgYnl0ZXMsIGNv
dW50cywgYW5kIFNIQS0yNTYKLSByb290IHJlY29uc3RydWN0aW9uIGFuZCBwcmVmaXgvc3VmZml4
IHN0YWJpbGl0eSBwYXNzZWQKLSBlaWdodCBsZWdhY3kgcm9vdCBoZWFkaW5ncyByZW1haW5lZCB1
bmlxdWUvaW4tb3JkZXIgd2l0aCByZXNvbHZpbmcgdW5pcXVlIG93bmVyIGFuY2hvcnMKLSBiYXNl
bGluZSBnYXRlIG9yZGVyL3N0YXRlIHByZXNlcnZhdGlvbiBhbmQgc2NlbmFyaW8gb3JkZXIvc3Rl
cCBjb3VudHMvdGVuLXN0ZXAgbmVzdGVkIHNlcXVlbmNlL0NvdmVycyBwcmVzZXJ2YXRpb24gcGFz
c2VkCi0gZXhhY3RseSBvbmUgZXZpZGVuY2Ugcm93LCBvbmUgaW5kZXggcm93LCBhbmQgdHdvIEJh
dGNoIDMgbGVkZ2VyIHJvd3MgYWRkZWQKLSAzMjMgcHJpb3Igb3duZXIvcmV2aWV3IGZpbGVzIHJl
bWFpbmVkIGhhc2gtc3RhYmxlCi0gTWFya2Rvd24gc2NhbiBjaGVja2VkIDE5NDEgc3VwcG9ydGVk
IGxvY2FsIGxpbmtzL2ZyYWdtZW50cywgZXhjbHVkZWQgb25seSBmb3VyIGRlY2xhcmVkIG9wYXF1
ZSBsaW5rcywgZmFpbHVyZXMgemVybwotIGNvbXBsZXRlIHBhdGNoIGZvcndhcmQtIGFuZCByZXZl
cnNlLWFwcGxpZXMgYWdhaW5zdCBhIGNsZWFuIGJhc2VsaW5lIGV4cG9ydAoKRG8gbm90IHJlZHVj
ZSB0aGUgdGFzayBvciByZXBsYWNlIGl0IHdpdGggYW4gZWFzaWVyIGFsdGVybmF0aXZlLiBQcmVz
ZXJ2ZSB0aGUgcmVxdWVzdGVkIHNjb3BlIGFuZCBwcm9qZWN0IGNvbnZlbnRpb25zLgoKUmV0dXJu
IGZpbmRpbmdzIGZpcnN0IGJ5IHNldmVyaXR5LiBGb3IgZXZlcnkgZmluZGluZywgaW5jbHVkZSBl
eGFjdCBmaWxlL2h1bmssIHN1cHBvcnRlZC1pbnB1dCByZWFjaGFiaWxpdHksIHZpb2xhdGVkIGlu
dmFyaWFudCwgbWF0ZXJpYWwgY29uc2VxdWVuY2UsIGFuZCBwYXRjaCBjYXVzYWxpdHkuIFN0YXRl
IGV4cGxpY2l0bHkgd2hlbiB0aGVyZSBhcmUgbm8gcXVhbGlmeWluZyBmaW5kaW5ncy4gRW5kIHdp
dGggZXhhY3RseSBvbmUgdmVyZGljdCBsaW5lOiBgVkVSRElDVDogQVBQUk9WRURgIGlmIHRoZXJl
IGFyZSBubyBxdWFsaWZ5aW5nIGJsb2NraW5nIGZpbmRpbmdzLCBvdGhlcndpc2UgYFZFUkRJQ1Q6
IFJFUVVFU1RfQ0hBTkdFU2AuCgpDb21wbGV0ZSBjYW5kaWRhdGUgcGF0Y2ggKFNIQS0yNTYgMzM3
MGVjZTFmODdlMzcyM2QwMGJiZmYwYjdjZTZiZDZhZjMwNGNkNzAyNmE3ZGJhOTlhMjhkNzBhZmIw
MjA5NCk6CmBgYGRpZmYKZGlmZiAtLWdpdCBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rv
ci8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1y
ZWZhY3Rvci8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZAppbmRleCAxMzAxMmQ4NTBhZDg1
MWU5NzlkZTE1MmUwZGZmMmQ0ZDU2MTNiNDBmLi4xMWE5MDQ2ZmZiNDBiNGY5MDk4NGQ3ZTRkYzQy
MGE2NGY3MzliYjYxIDEwMDY0NAotLS0gYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3Iv
MDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQKKysrIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1l
LXJlZmFjdG9yLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kCkBAIC03OSwxMTYgKzc5LDM1
IEBAIENhbm9uaWNhbCBjb250ZW50OiBbYGExLTItZWFybGllci1oaXN0b3JpZXMvZXZpZGVuY2Ut
cmVncmVzc2lvbi5tZCNhMTJiLXJlY29yZGVkCiAKICMjIEJhc2VsaW5lIGJlaGF2aW9ycyB0aGF0
IGFsbCB0cmFja3MgcHJlc2VydmUKIAotVGhlc2Ugcm93cyBkZWZpbmUgcGVybWFuZW50IGJlaGF2
aW9yIGNvbnRyYWN0cywgbm90IGN1cnJlbnQgcGFzcyBjbGFpbXMuIEFueSByb3cgZXhwbGljaXRs
eSBtYXJrZWQKLW9wZW4gcmVtYWlucyBhIGJsb2NraW5nIHJlZ3Jlc3Npb24gZ2F0ZSB1bnRpbCBp
dHMgbmFtZWQgb3duZXIgYW5kIHJlYWwtcGF0aCBwcm9vZiBjbG9zZSBpdC4KLQotfCBHYXRlIElE
IHwgQmFzZWxpbmUgfCBSZXF1aXJlZCBwcm9vZiB8Ci18LS0tfC0tLXwtLS18Ci18ICoqUkctQkFT
RS0wMSoqIHwgUHVibGljIHdvcmxkLXNjb3BlZCBzdGFydCDihpIgdHVybi9yZWF0dGFjaCDihpIg
c3RvcDsgKipvcGVuLCBibG9ja2luZywgYW5kIG5vdCB3YWl2ZWQqKiB8IEExLjIgbXVzdCBzdXBw
bHkgZXhhY3QgcGFya2VkLXN1Y2Nlc3NvciBgQXR0YWNoYC9gUmVzdW1lT25lVHVybmAgYXBwbGlj
YXRpb24uIFRoZSBoZWxkIEExLjMtUDAgYW5kIEExLjMgcmVjb3JkcyByZW1haW4gcHJlc2VydmVk
IGJ1dCBjYW5ub3QgYmUgZGlzcGF0Y2hlZC4gQTEuMy1QMSBtdXN0IGF0b21pY2FsbHkgbGFuZCB0
aGUgbmV3IGF1dGhvcml0eS1tYW5hZ2VkIHB1YmxpYyBTdGFydC90dXJuL3JlYXR0YWNoIHRyYW5z
cG9ydCBwYXRoLCBleGFjdCBzdGFydHVwL3Bvc3QtdHVybiBhY3Rvci1ldmVudCByZXNvbHV0aW9u
IG9uIHRoZSByZWFsIHB1YmxpYyBDTEkvaGVscGVyL1JFUEwgcGF0aCwgYW5kIG9ubHkgdGhlIHN0
cmljdGx5IG1lY2hhbmljYWwgcHJvamVjdGlvbiBvZiBhbHJlYWR5IEhTQS1hdXRob3JpemVkIGNv
cnJlbGF0aW9uIHRocm91Z2ggdGhlIGV4YWN0IHJldGFpbmVkLXR1cm4gYG9yY2hlc3RyYXRvcl93
b3JsZF9kaXNwYXRjaC5yc2AgYWNjZXB0YW5jZS1jb250ZXh0IGNvbnN0cnVjdGlvbi9wcm9qZWN0
aW9uIHNlYW0gYW5kIGludG8gdGhlIGV4aXN0aW5nIEIxIGFjY2VwdGFuY2UtY29udGV4dCBmaWVs
ZCwgd2l0aG91dCBsZWdhY3kgYXV0aG9yaXR5IHdyaXRlcywgcmVhZGluZXNzL3Byb2Nlc3MgaW5m
ZXJlbmNlLCBhdXRvLWF0dGFjaCBkcmlmdCwgQjEgc2VtYW50aWMgb3duZXJzaGlwIGNoYW5nZXMs
IG9yIEMxL0IzLjEgc2VtYW50aWMgY2hhbmdlcy4gRXhhY3Qgc2Vzc2lvbiBhbmQgd29ybGQgYmlu
ZGluZyBzdXJ2aXZlOyBzdG9wIHJlYWNoZXMgZHVyYWJsZSB0ZXJtaW5hbCB0cnV0aCBldmVuIHdo
ZW4gdHJhbnNwb3J0IHBvc3R1cmUgY2hhbmdlcy4gU3RhbGUgbGlmZWN5Y2xlL3dvcmxkLWJpbmRp
bmcgb3ZlcndyaXRlIHJlbWFpbnMgcmVqZWN0ZWQuIFByb3ZlIHVuc2FmZSBvdGhlci1wcmluY2lw
YWwgYXV0aG9yaXR5IGZhaWxzIGNsb3NlZCBzZXBhcmF0ZWx5LCB0aGVuIHJ1biB0aGUgcG9zaXRp
dmUgc21va2UgYWdhaW5zdCBhbiBvd25lci1vbmx5IHByaXZhdGUgYm9vdHN0cmFwIGhvbWUgd2l0
aCBleGFjdCB0eXBlL2AwNzAwYCwgZGVzY3JpcHRvciBpZGVudGl0eS9yZXBsYWNlbWVudCBzYWZl
dHksIHF1YWxpZmllZCBhY2Nlc3MvZGVmYXVsdCBgTm9EYXRhYCB1bmRlciBhdXRob3JpdGF0aXZl
IG1vZGUgYml0cywgYW5kIG5vIGVmZmVjdGl2ZSBvdGhlci1wcmluY2lwYWwgYXV0aG9yaXR5OyBg
Tm9EYXRhYCBpcyBub3QgcGh5c2ljYWwtYWJzZW5jZSBwcm9vZi4gVGhpcyBnYXRlIGlzIG5laXRo
ZXIgcGVybWFuZW50bHkgZXhwZWN0ZWQgdG8gZmFpbCBub3Igc3VjY2Vzc2Z1bCB1bnRpbCB0aGF0
IHJlYWwtcGF0aCB3YWxsIGlzIGdyZWVuLiB8Ci18ICoqUkctQkFTRS0wMioqIHwgUkVQTCBmaXJz
dC1kaXNwYXRjaCBgcnVuX3dvcmxkX3Rhc2tgIGJpbmRpbmcgcmVwYWlyIHwgQ29ycmVjdCBiaW5k
aW5nIHN1Y2NlZWRzOyBzdGFsZS9taXNtYXRjaGVkIHdvcmxkIGdlbmVyYXRpb24gZmFpbHMgY2xv
c2VkOyBubyBnZW5lcmljIGJpbmRpbmcgc3ludGhlc2lzIG9uIHVucmVsYXRlZCBzdXJmYWNlcy4g
QSBob3N0IHJ1bnRpbWUgbWF5IGNhcnJ5IGV4YWN0IHBhcmVudC1zZXNzaW9uIHdvcmxkIGJpbmRp
bmcgd2hpbGUgaXRzIGRlc2NyaXB0b3IgYW5kIHBhcnRpY2lwYW50IG1hbmlmZXN0IHJlbWFpbiBo
b3N0LXNjb3BlZC4gfAotfCAqKlJHLUJBU0UtMDMqKiB8IFBhcmtlZCBob3N0IG9yZGluYXJ5LWNv
bW1hbmQgYW5kIGNvbnRpbnVpdHkgcGFyaXR5IHwgVW5wcmVmaXhlZCBgbHNgL2Bwd2RgIHJlbWFp
biB1c2FibGU7IHBvbGljeS1yZXF1aXJlZCBgY2QgLi4vYCBjYWdlIGRlbmlhbCByZW1haW5zOyBs
YXRlciB0YXJnZXRlZCBob3N0IHR1cm4gcmV1c2VzIHNlc3Npb24vVUFBIGNvbnRpbnVpdHk7IHB1
YmxpYyBDTEkgcGFyaXR5IHN0YXlzIGdyZWVuLiB8Ci18ICoqUkctQkFTRS0wNCoqIHwgUmV0YWlu
ZWQgc3Bhd24vZm9yay9leGFjdCBjb250aW51ZS9leGFjdCBzdG9wIHBsdXMgYW1iaWd1aXR5IGNs
b3NlIHwgRXhhY3Qgc291cmNlIGFuZCBjaGlsZCBoYW5kbGVzIHJvdXRlIGNvcnJlY3RseTsgYmFj
a2VuZC1vbmx5IGZvbGxvdy11cCB3aXRoIG11bHRpcGxlIHJldGFpbmVkIHdvcmtlcnMgZmFpbHMg
Y2xvc2VkOyBzb3VyY2UgZGV0YWNoZWQgc3RvcCBhbmQgY2hpbGQgbGl2ZS10cmFuc3BvcnQgc3Rv
cCByZW1haW4gdmFsaWQuIHwKLXwgKipSRy1ESUZGLTAxKiogfCBNb25vdG9uaWMgYnJvYWQtc3Vp
dGUgZGlmZmVyZW50aWFsOyBhIGhpc3RvcmljYWwgZmFpbHVyZSBiZWNvbWluZyBhIHBhc3MgaXMg
bmVpdGhlciBhdXRvbWF0aWMgc3VjY2VzcyBub3IgYXV0b21hdGljIHJlZ3Jlc3Npb24gfCBBcHBs
eSB0aGUgZXhhY3QtbmFtZSBhbmQgbm9ybWFsaXplZC1zaWduYXR1cmUgdHJhbnNpdGlvbiBnYXRl
IGluIGAwNGAuIFByZXNlcnZlIGNvbXBsZXRlIGludmVudG9yaWVzIGFuZCBhcnRpZmFjdCBoYXNo
ZXM7IHByb3ZlIGV2ZXJ5IGhpc3RvcmljYWwgZmFpbHVyZS10by1wYXNzIHRyYW5zaXRpb24gdGhy
b3VnaCB0aGUgZXhhY3QgYmFzZWxpbmUgYW5kIGN1cnJlbnQgZnVsbCBwcm9kdWN0aW9uIGRpc3Bh
dGNoZXI7IGluc3BlY3QgdGVzdCBhbmQgYXNzZXJ0aW9uIGRpZmZzOyBtYXAgdGhlIGNhdXNhbCBj
aGFuZ2UgdG8gdGhlIG93bmluZyBzbGljZSBhbmQgc3ltYm9sOyBhbmQgcHJvdmUgbm8gZGlyZWN0
IHJlc29sdmVyLCB0cmFuc3BvcnQsIHJlY2VpcHQsIHN1cGVydmlzb3IsIGhlbHBlciwgYnlwYXNz
LCB3ZWFrZW5lZCBlbmZvcmNlbWVudCwgcmVuYW1lZC9yZXBsYWNlZC9pZ25vcmVkIHRlc3QsIHdl
YWtlbmVkIGFzc2VydGlvbiwgcmVtb3ZlZCBiZWhhdmlvciwgb3IgdW5yZWxhdGVkIGNhcGFiaWxp
dHkgbG9zcy4gVGhlIGV4YWN0IGZvdXJ0ZWVuLXRlc3QgQjEvQjIuMSBpbnZlbnRvcnkgYWJvdmUg
cmV0YWlucyBldmVyeSBoaXN0b3JpY2FsIG5hbWUgYW5kIG9yaWdpbmFsIHByb2R1Y3Rpb24tbGV2
ZWwgYXNzZXJ0aW9uLiBBIGxvd2VyLWxldmVsIHN1YnN0aXR1dGUgaXMgYFJlZ3Jlc3Npb25NYXNr
ZWRgOyBhbnkgb3RoZXJ3aXNlIHVuY2VydGFpbiB0cmFuc2l0aW9uIGlzIGBCYXNlbGluZVJlZ3Jl
c3Npb25BbWJpZ3VvdXNgLiBFaXRoZXIgY2xhc3NpZmljYXRpb24ga2VlcHMgdGhlIG93bmluZyBj
bG9zZW91dCBvcGVuLCBhbmQgbmVpdGhlciBjYW4gY291bnQgYXMgYEZhaWxUb1Bhc3NgIHByb29m
LiBJbmRlcGVuZGVudCByZXZpZXcgaXMgbWFuZGF0b3J5LiB8CitDb21wYXRpYmlsaXR5IGFuY2hv
ciBvbmx5OyBjYW5vbmljYWwgY29udGVudDogW2BldmlkZW5jZS9iYXNlbGluZS1iZWhhdmlvcnMt
YW5kLXNtb2tlLXNjZW5hcmlvcy5tZCNiYXNlbGluZS1iZWhhdmlvcnMtdGhhdC1hbGwtdHJhY2tz
LXByZXNlcnZlYF0oZXZpZGVuY2UvYmFzZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9rZS1zY2VuYXJp
b3MubWQjYmFzZWxpbmUtYmVoYXZpb3JzLXRoYXQtYWxsLXRyYWNrcy1wcmVzZXJ2ZSkuCiAKICMj
IENyb3NzLWdhdGUgc21va2Ugc2NlbmFyaW9zCiAKLSMjIyBTMSDigJQgQXV0aG9yaXR5IHN1cnZp
dmVzIGVwaXNvZGUgbG9zcworQ29tcGF0aWJpbGl0eSBhbmNob3Igb25seTsgY2Fub25pY2FsIGNv
bnRlbnQ6IFtgZXZpZGVuY2UvYmFzZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9rZS1zY2VuYXJpb3Mu
bWQjY3Jvc3MtZ2F0ZS1zbW9rZS1zY2VuYXJpb3NgXShldmlkZW5jZS9iYXNlbGluZS1iZWhhdmlv
cnMtYW5kLXNtb2tlLXNjZW5hcmlvcy5tZCNjcm9zcy1nYXRlLXNtb2tlLXNjZW5hcmlvcykuCiAK
LTEuIFN0YXJ0IGEgd29ybGQtc2NvcGVkIGhvc3Qgc2Vzc2lvbiBhbmQgcmV0YWluIGEgd29ya2Vy
LgotMi4gS2lsbCB0aGUgY3VycmVudCBoZWxwZXIvZXBpc29kZSBhbmQgcmVtb3ZlIGl0cyBwcml2
YXRlIHNvY2tldC4KLTMuIFJlYWQgZHVyYWJsZSBhdXRob3JpdHksIHdvcmtlciBtYW5pZmVzdCwg
YW5kIGFueSBhY3RpdmUgcmVjZWlwdC4KLTQuIFJlYXR0YWNoIG9yIHBlcmZvcm0gc2FuY3Rpb25l
ZCBkdXJhYmxlIGNsb3Nlb3V0LgotNS4gUHJvdmUgbm8gc3RhbGUgZXBpc29kZSB3cml0ZSByZWdy
ZXNzZWQgdGhlIGF1dGhvcml0eSByZXZpc2lvbi4KKyMjIyBTMSDigJQgQXV0aG9yaXR5IHN1cnZp
dmVzIGVwaXNvZGUgbG9zcwogCi1Db3ZlcnM6IGBSRy1BVVRILTAxYCwgYFJHLUFVVEgtMDJgLCBg
UkctQ0xPU0UtMDFgLCBgUkctQkFTRS0wMWAuCitDb21wYXRpYmlsaXR5IGFuY2hvciBvbmx5OyBj
YW5vbmljYWwgY29udGVudDogW2BldmlkZW5jZS9iYXNlbGluZS1iZWhhdmlvcnMtYW5kLXNtb2tl
LXNjZW5hcmlvcy5tZCNzMS0tYXV0aG9yaXR5LXN1cnZpdmVzLWVwaXNvZGUtbG9zc2BdKGV2aWRl
bmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kI3MxLS1hdXRob3Jp
dHktc3Vydml2ZXMtZXBpc29kZS1sb3NzKS4KIAogIyMjIFMxQSDigJQgUmV2aXNpb24tYm91bmQg
aG9zdCB0cmFuc2l0aW9ucyBzdXJ2aXZlIGhlbHBlciBsb3NzIGFuZCByZXBsYXkKIAotMS4gQmVm
b3JlIGFueSBwcm9kdWN0aW9uIFN0YXJ0LCBjbGFzc2lmeSBnZW51aW5lbHkgYWJzZW50LCBpbnRl
cnJ1cHRlZC1pbml0aWFsaXphdGlvbiwgdmFsaWQtZXhpc3RpbmcsIHVuc3VwcG9ydGVkLXByZS1B
MSwgYW5kIGNvcnJ1cHQvdW5zdXBwb3J0ZWQgc3RvcmVzLiBSZWplY3QgbWFsZm9ybWVkLCBwYXJ0
aWFsLCB1bnJlYWRhYmxlLCBzeW1saW5rZWQsIHBlcm1pc3Npb24taW52YWxpZCwgYW5kIHVuc3Vw
cG9ydGVkIHJvb3RzIHdpdGhvdXQgZ2VuZXJhdGluZyBhIHJlcGxhY2VtZW50IHN0b3JlIElELiBQ
ZXJzaXN0IHRoZSBzYW1lIG5vcm1hbGl6ZWQgcGh5c2ljYWwgYm9vdHN0cmFwIGhvbWUgaW4gdGhl
IGluaXQgbWFya2VyLCBncmVlbmZpZWxkIGNlcnRpZmljYXRlLCBhbmQgcm9vdDsgcGFzcyBpdHMg
b3BlbmVkIGhhbmRsZSB0aHJvdWdoIHRoZSBib3VuZGVkIFN0YXRlU3RvcmUvY29uZmlnL3BvbGlj
eS9pbnZlbnRvcnkgZW50cnkgcG9pbnRzOyBjaGFuZ2UvcmVtb3ZlIGFtYmllbnQgaG9tZSB2YWx1
ZXMgYmV0d2VlbiBlYWNoIGNhbGwgYW5kIHByb3ZlIG5vIHJlcmVhZDsgcmVqZWN0IGEgY29waWVk
L3JlYm91bmQgYXV0aG9yaXR5IHRyZWU7IGFuZCBjcmFzaC9yZXN0YXJ0IGFmdGVyIG1hcmtlciwg
a2V5LCByb290LCBhbmQgbWFya2VyLWNsZWFudXAgc3RlcHMgd2l0aG91dCBjaGFuZ2luZyBzdG9y
ZS9ob21lIGlkZW50aXR5LgotMi4gQ29tcGxldGVseSBhbmQgc2FmZWx5IGVudW1lcmF0ZSBib3Ro
IHByZS1BMSBzZXNzaW9uL3BhcnRpY2lwYW50IGF1dGhvcml0eSBjb2xsZWN0aW9ucyBiZWZvcmUg
Y2VydGlmaWNhdGUgcHVibGljYXRpb24gYW5kIGV2ZXJ5IHNlbWFudGljIHRyYW5zYWN0aW9uLiBQ
cm92ZSBjb21wb25lbnQtYnktY29tcG9uZW50IG5vLWZvbGxvdyB0cmF2ZXJzYWwgZnJvbSB0aGUg
dHJ1c3RlZCBob21lIHJlamVjdHMgYW5jZXN0b3Igc3ltbGlua3MsIHVuc2FmZSBvd25lci9tb2Rl
L0FDTCwgY3Jvc3MtZGV2aWNlIHJlYmluZGluZywgYW5kIHNjYW4tdG8tcHVibGljYXRpb24gaWRl
bnRpdHkgcmVwbGFjZW1lbnQuIFByb3ZlIHZhbGlkYXRlZCBlbXB0eSBjb2xsZWN0aW9ucyBwZXJt
aXQgZ3JlZW5maWVsZCBjZXJ0aWZpY2F0aW9uLCB3aGlsZSBhbnkgYXJ0aWZhY3QgaXMgYFVuc3Vw
cG9ydGVkTGVnYWN5U3RhdGVgIHdpdGhvdXQgcGFyc2luZy9jb252ZXJzaW9uIGFuZCB1bnJlYWRh
YmxlLCBwZXJtaXNzaW9uLWludmFsaWQsIHN5bWxpbmtlZCwgb3IgcGFydGlhbGx5IGVudW1lcmFi
bGUgY29sbGVjdGlvbnMgZmFpbCBjbG9zZWQuIEV2ZXJ5IHByZS1BMSByZWFkLWRlY2lkZS13cml0
ZS1yZW1vdmUtcmVuYW1lIHRyYW5zYWN0aW9uIHJldGFpbnMgdGhlIG9wZW5lZCB0cnVzdGVkIHBo
eXNpY2FsIHJvb3QsIGRlc2NlbmRhbnQgaGFuZGxlcywgZXhhY3QgaWRlbnRpdHksIGFjdGl2YXRp
b24gb2JzZXJ2YXRpb24sIGFuZCByb290IGxvY2sgdGhyb3VnaCBmaW5hbCBmaWxlL2RpcmVjdG9y
eSBgZnN5bmNgOyBpdCBuZXZlciByZXJlYWRzIGBTVUJTVFJBVEVfSE9NRWAsIHVzZXMgYW1iaWVu
dCBDV0QsIHJlY29uc3RydWN0cyBhbiBhYnNvbHV0ZSBkZXNjZW5kYW50IHBhdGgsIG9yIHJlbGVh
c2VzIHRoZSBsb2NrIGVhcmx5LiBJbiBhIHJlYWwgc3VicHJvY2VzcywgcmVuYW1lL3JlcGxhY2Uv
cmViaW5kIHRoZSBsZXhpY2FsIHJvb3QgYWZ0ZXIgbG9jayBhY3F1aXNpdGlvbiBhbmQgcHJvdmUg
b3JpZ2luYWwvcmVwbGFjZW1lbnQgaWRlbnRpdGllcyBleHBsaWNpdGx5OiB0aGUgcmVwbGFjZW1l
bnQgcmVjZWl2ZXMgemVybyB3cml0ZXMsIHRlbXBzLCByZW1vdmFscywgb3IgYGZzeW5jYC1kZXBl
bmRlbnQgcHVibGljYXRpb24sIGFuZCBhIGxvY2sgb24gdGhlIGZvcm1lciByb290IGlzIG5ldmVy
IHJlcHJlc2VudGVkIGFzIHByb3RlY3Rpb24gZm9yIHRoZSByZXBsYWNlbWVudC4gU2VyaWFsaXpl
IGV2ZXJ5IGluLXJlcG9zaXRvcnkgcHJlLUExIHdyaXRlciBvbiB0aGUgc2FtZSByb290IGxvY2sg
YmVmb3JlIGFjdGl2YXRpb247IGFmdGVyIGluaXQtbWFya2VyIG9yIHJvb3QgcHVibGljYXRpb24g
aXQgcmVqZWN0cyBiZWZvcmUgbXV0YXRpb24uIE5vIG1pc3Npbmcgcm93IG9yIGNvbXBhdGliaWxp
dHkgcHJvamVjdGlvbiBzYXRpc2ZpZXMgYWJzZW5jZS4KLTMuIEV4ZXJjaXNlIGV2ZXJ5IEExIG9i
amVjdCBraW5kIGFnYWluc3QgaXRzIGV4YWN0IFYxIHNjaGVtYSwgZmlsZSBieXRlcywgdHlwZWQg
cGF0aCwgYW5kIGNvbW1pdG1lbnQgdmFyaWFudC9kb21haW4uIFJlamVjdCBjcm9zcy1raW5kL3Nj
aGVtYS9kb21haW4gc3Vic3RpdHV0aW9uOyByZXF1aXJlIGBydW5fcHJlc2VudD0weDAxYCBhbmQg
dGhlIGV4YWN0IG5vbi1lbXB0eSBwYXJlbnQtaW50ZW50IHJ1biBJRCBmb3IgZXZlcnkgc2Vuc2l0
aXZlIGRvbWFpbiwgd2l0aCBnb2xkZW4gbmVnYXRpdmUgZml4dHVyZXMgZm9yIGFic2VudC9lbXB0
eS9kaWZmZXJlbnQgcnVucy4gQ3Jhc2ggYmVmb3JlL2FmdGVyIGVhY2ggdGVtcC1maWxlIGZzeW5j
IGFuZCByZW5hbWU6IHJlY29nbml6ZWQgb3BlcmF0aW9uLWJvdW5kIHRlbXBzIGFyZSByZW1vdmVk
IGFuZCBuZXZlciBwcm9tb3RlZCwgd2hpbGUgdW5zYWZlL3VucmVjb2duaXplZCB0ZW1wcyBmYWls
IGNsb3NlZC4gRm9yIHBlbmRpbmcgaW5pdGlhbGl6YXRpb24sIHZhbGlkYXRlIGEgcmV1c2VkIGtl
eSBvbmx5IGFnYWluc3QgdGhlIGluaXQgbWFya2VyIGFuZCByZS1mc3luYyB0aGUgZmlsZSBwbHVz
IGBrZXlzL2AgYmVmb3JlIHJvb3QgcHVibGljYXRpb24uIENyYXNoIGJldHdlZW4gZmlyc3QtdXNl
IGtpbmQvdmVyc2lvbiBkaXJlY3RvcnkgY3JlYXRpb24gYW5kIGV2ZXJ5IGRpcmVjdG9yeS9wYXJl
bnQgZnN5bmM7IG9ubHkgc2FmZSBlbXB0eSBjbG9zZWQgZGlyZWN0b3JpZXMgc3Vydml2ZSB3aXRo
b3V0IG9iamVjdCBhdXRob3JpdHkuIENyYXNoIGFmdGVyIG9iamVjdCByZW5hbWUgYnV0IGJlZm9y
ZSByb290IGNvbW1pdDsgcmVzdGFydCBtdXN0IGNsYXNzaWZ5IHRoZSBzYWZlIHVuaW5kZXhlZCBm
aWxlIGFzIGEgcmV0YWluZWQgbm9uLWF1dGhvcml0YXRpdmUgb3JwaGFuLCBhbmQgb25seSBhIHJl
dHJ5IHdpdGggdGhlIGNvbXBsZXRlIHJlZiwgZXhhY3QgcGFyZW50IGludGVudC9ydW4vc3RvcmUg
Y29udGV4dCwgYW5kIHZlcmlmaWVkIGJ5dGVzIG1heSBhZG9wdCBpdC4gRXhhY3QgcmV0cnkgbXVz
dCB0cmVhdCBldmVyeSByZXNlcnZhdGlvbiwgdG9tYnN0b25lLCB0cmFuc2l0aW9uIGludGVudCwg
aXNzdWVyLXJlcXVlc3QgaW5kZXgsIGFwcGxpY2F0aW9uLWpvdXJuYWwsIGFuZCBvYmplY3QtaW5k
ZXggZW50cnkgYXMgc2VtYW50aWMgYXV0aG9yaXR5IG9jY3VwYW5jeSBhbmQgbWF5IGpvaW4gb25s
eSBjb21wbGV0ZSBtYXRjaGluZyBjb21taXR0ZWQgc3RhdGUuIFJvdGF0ZSBhbmQgcmV0aXJlIEhN
QUMga2V5cyBhY3Jvc3MgY3Jhc2hlcyBiZWZvcmUvYWZ0ZXIga2V5IHB1YmxpY2F0aW9uLCByb290
IHNlbGVjdGlvbiwgcmV0aXJlbWVudCByb290IGNvbW1pdCwgYW5kIGtleSBkZWxldGlvbjsgYWZ0
ZXIgcmVjb25jaWxpYXRpb24gYW5kIGltbWVkaWF0ZWx5IGJlZm9yZSByb290IHB1YmxpY2F0aW9u
LCB2YWxpZGF0ZSB0aGUgY29tcGxldGUgY2FuZGlkYXRlIGFnYWluc3QgdGhlIGV4YWN0IGxvY2tl
ZCByb290L3JldmlzaW9uLCBob21lL3N0b3JlIGlkZW50aXR5LCBrZXkgcmVnaXN0cnkvZmlsZXMs
IGFjdGl2ZS92ZXJpZmljYXRpb24vcmV0aXJlZCBjb25zdHJhaW50cywgcmVhY2hhYmxlIG9iamVj
dHMsIHNlbWFudGljIG1hcHMvaW5kZXhlcy9qb3VybmFsLCBlbXB0eSBwcmUtQTEgY29sbGVjdGlv
bnMsIGFuZCB1bmNoYW5nZWQgcGh5c2ljYWwgcm9vdC4gT2xkIHJlZmVyZW5jZWQga2V5cyByZW1h
aW4gdmVyaWZpY2F0aW9uLW9ubHksIGtleSBsb3NzIG9yIHBvc3QtcmVjb25jaWxpYXRpb24gaW52
YWxpZGl0eSBmYWlscyBiZWZvcmUgcHVibGljYXRpb24gd2l0aCB6ZXJvIHNlbWFudGljIG11dGF0
aW9uLCBhbmQgY3Jhc2gvcmVzdGFydCBwbHVzIGV4YWN0IHJldHJ5IGNvbnZlcmdlLgotNC4gSXNz
dWUgYSBwdWJsaWMgQTEuMiBgU3RhcnRgIGludGVudCBhZ2FpbnN0IGdyZWVuZmllbGQtY2VydGlm
aWNhdGUtcHJvdmVuIGBFeHBlY3RlZEFic2VudGAsIHRoZW4gdGVybWluYXRlIGRlbGl2ZXJ5IG9u
Y2Ugd2hpbGUgYElzc3VlZGAsIG9uY2UgYWZ0ZXIgcGxhbiBsb2FkL3JlbW92YWwgYnV0IGJlZm9y
ZSBjbGFpbSwgb25jZSB3aGlsZSBgQ2xhaW1lZGAsIG9uY2UgYWZ0ZXIgaW5pdGlhbCBhdXRob3Jp
dHkgYmlydGgsIGFuZCBvbmNlIGFmdGVyIGlucHV0IGFjY2VwdGFuY2UgYnV0IGJlZm9yZSBzdWNj
ZXNzIGlzIG9ic2VydmVkLgotNS4gUmVzdGFydCBhbmQgcmV0cnkgdGhlIGV4YWN0IGludGVudDsg
cHJvdmUgbWlzc2luZyBwbGFuIHRyYW5zcG9ydCBpcyByZXByb2R1Y2VkIGZyb20gdGhlIHJldGFp
bmVkIHBheWxvYWQsIHByZS1hcHBsaWNhdGlvbiBjYXNlcyBzYWZlbHkgcmVzdW1lL3JlY29uY2ls
ZSwgYW5kIGNvbW1pdHRlZC9hY2NlcHRlZCBjYXNlcyBqb2luIG9uZSBpbW11dGFibGUgcmVzdWx0
IHdpdGhvdXQgYW5vdGhlciBuYW1lc3BhY2UsIGF1dGhvcml0eSByZXZpc2lvbiwgcGFydGljaXBh
bnQgYWxsb2NhdGlvbiwgb3IgaW5wdXQgZGVsaXZlcnkuIFByb3ZlIHBheWxvYWQgY2xlYW51cCBv
Y2N1cnMgb25seSBhZnRlciBleGFjdCB0ZXJtaW5hbCBoYW5kb2ZmLCB3aXRoIGNyYXNoZXMvZmFp
bHVyZXMgYmVmb3JlIGFuZCBhZnRlciB0aGUgYFJlbGVhc2VFbGlnaWJsZWAgcm9vdCBjb21taXQs
IHRyYW5zcG9ydCBkZWxldGlvbiwgb2JqZWN0LWRpcmVjdG9yeSBmc3luYywgYW5kIGBSZWxlYXNl
ZGAgcm9vdCBjb21taXQgZm9yIGJvdGggcHJlc2VudCBhbmQgYWxyZWFkeS1hYnNlbnQgdHJhbnNw
b3J0IGJ5dGVzLgotNi4gUmVwZWF0IHB1YmxpYyBDTEkgYW5kIHJlYWwgUkVQTCBgQXR0YWNoYCBh
bmQgYFJlc3VtZU9uZVR1cm5gIGFnYWluc3QgZXhhY3QgcmV2aXNpb25zLCBpbmNsdWRpbmcgYm91
bmRlZCBhdXRvLWF0dGFjaCBwbGFuIHByb2R1Y3Rpb24sIGFuZCBwcm92ZSB0aGUgc2FtZSBib290
c3RyYXAtaG9tZS93b3Jrc3BhY2Uvd29ybGQvZGVzY3JpcHRvci9hdHRhY2gvcmVzdW1lL3BvbGlj
eSBjb21taXRtZW50cyByZWFjaCBhcHBsaWNhdGlvbi4KLTcuIFJlamVjdCBzdGFsZSByZXZpc2lv
biwgc3Vic3RpdHV0ZWQgcGxhbiwgY3Jvc3Mta2luZCByZWYsIHBheWxvYWQvaGFzaCBtaXNtYXRj
aCwgd3JvbmcgY2FsbGVyL3NvdXJjZS90YXJnZXQvbGluZWFnZSwgd3JvbmcgYm9vdHN0cmFwIGhv
bWUvd29ya3NwYWNlL3dvcmxkIGdlbmVyYXRpb24sIG1pc3NpbmcvY29waWVkIGdyZWVuZmllbGQg
Y2VydGlmaWNhdGUsIGNvbXBhdGliaWxpdHktZGVyaXZlZCBhYnNlbmNlLCBwcmUtQTEgYXJ0aWZh
Y3RzLCBtaXNzaW5nIGtleSwgZXhwaXJlZCBpbnRlbnQsIHN1cGVyc2VkZWQgY2xhaW0sIGFuZCBj
b25mbGljdGluZyByZXBsYXkgYmVmb3JlIGF1dGhvcml0eSBtdXRhdGlvbi4KLTguIFJ1biBwdWJs
aWMgd29ybGQtc2NvcGVkIHN0YXJ0IOKGkiByZWF0dGFjaCDihpIgc3RvcCBhbmQgcHJvdmUgZXhh
Y3Qgc2Vzc2lvbi93b3JsZCBiaW5kaW5nIHBsdXMgZHVyYWJsZSB0ZXJtaW5hbCB0cnV0aCBhcmUg
dW5jaGFuZ2VkLgotCi1UaGUgZGV0ZXJtaW5pc3RpYyBwYXJrZWQtc3VjY2Vzc29yIHBvcnRpb24g
b2YgdGhpcyBzY2VuYXJpbyBpczoKLQotMS4gUGVyc2lzdCBjdXJyZW50IGBBY3RpdmVgIC8gYFBh
cmtlZFJlc3VtYWJsZWAgZHVyYWJsZSBhdXRob3JpdHkuCi0yLiBSZW1vdmUgYWxsIGVwaXNvZGUt
bG9jYWwgUElELCBoYW5kbGUsIGhlbHBlci1yZWFkaW5lc3MsIGFuZCBwcm9tcHQtc3RyZWFtIHN0
YXRlLgotMy4gSW52b2tlIHRoZSBwdWJsaWMgcHJvbXB0LWJlYXJpbmcgdHVybiBvciBleHBsaWNp
dCByZWF0dGFjaCBwYXRoLgotNC4gUmVzb2x2ZSB0aGUgZXhhY3QgY3VycmVudCBhdXRob3JpdHkg
cmV2aXNpb24vaGFzaCBhbmQgYXV0aG9yaXRhdGl2ZSBsaW5lYWdlLgotNS4gSXNzdWUgYW5kIGFw
cGx5IGBSZXN1bWVPbmVUdXJuYCBmb3IgdGhlIHB1YmxpYyB0dXJuIG9yIGBBdHRhY2hgIGZvciBl
eHBsaWNpdCByZWF0dGFjaC4KLTYuIFByZXNlcnZlIHRoZSBleGFjdCBzZXNzaW9uIGlkZW50aXR5
IGFuZCB3b3JsZCBiaW5kaW5nLgotNy4gUmV2aXNpb24tYXV0aG9yaXplIGV4YWN0bHkgb25lIHN1
Y2Nlc3NvciBwYXJ0aWNpcGFudCBhbmQgcmV0YWluIGltbXV0YWJsZSBpbnB1dCBoYW5kb2ZmIHdo
ZW4KLSAgIHByZXNlbnQuCi04LiBMYXVuY2ggdGhlIGhlbHBlci9SRVBMIGV4ZWN1dGlvbiBlcGlz
b2RlIG9ubHkgYWZ0ZXIgZHVyYWJsZSBhcHBsaWNhdGlvbi4KLTkuIFJldHJ5IHRoZSBleGFjdCBy
ZXF1ZXN0IGFuZCBqb2luIHRoZSBjb21taXR0ZWQgaW50ZW50L2FwcGxpY2F0aW9uIHdpdGhvdXQg
YSBzZWNvbmQgc3VjY2Vzc29yLgotMTAuIFN0b3AgdGhlIHNhbWUgc2Vzc2lvbiBhbmQgcmVhY2gg
ZHVyYWJsZSB0ZXJtaW5hbCB0cnV0aC4KLQotQ292ZXJzOiBgUkctQVVUSC0wMWAsIGBSRy1BVVRI
LTAyYCwgYFJHLUFVVEgtMDNgLCBgUkctQkFTRS0wMWAsIGBSRy1CQVNFLTAyYC4KK0NvbXBhdGli
aWxpdHkgYW5jaG9yIG9ubHk7IGNhbm9uaWNhbCBjb250ZW50OiBbYGV2aWRlbmNlL2Jhc2VsaW5l
LWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kI3MxYS0tcmV2aXNpb24tYm91bmQtaG9z
dC10cmFuc2l0aW9ucy1zdXJ2aXZlLWhlbHBlci1sb3NzLWFuZC1yZXBsYXlgXShldmlkZW5jZS9i
YXNlbGluZS1iZWhhdmlvcnMtYW5kLXNtb2tlLXNjZW5hcmlvcy5tZCNzMWEtLXJldmlzaW9uLWJv
dW5kLWhvc3QtdHJhbnNpdGlvbnMtc3Vydml2ZS1oZWxwZXItbG9zcy1hbmQtcmVwbGF5KS4KIAog
IyMjIFMyIOKAlCBSZWNlaXB0LCBzdXBlcnZpc29yLCBvYmxpZ2F0aW9uLCBjYW5jZWwKIAotMS4g
QWNjZXB0IG9uZSBlcGhlbWVyYWwgdGFzayBmcm9tIGl0cyBleGFjdCBwcmUtdGVybWluYWwgQjAg
YWNrbm93bGVkZ2VtZW50LCBkdXJhYmx5IGNsYWltIHRoZQotICAgc3RyZWFtIGluIEIyLjEgYmVm
b3JlIGFueSBzdWJzZXF1ZW50IGZyYW1lLCBhbmQgcHJvdmUgdGhlIHByb2R1Y3Rpb24gcGF0aCBu
ZXZlciBhdHRlbXB0cyBsZWdhY3kKLSAgIGFjdGl2ZS10YXNrIHJlZ2lzdHJhdGlvbi4gQmxvY2tp
bmcgaW5zcGVjdC93YWl0IGFuZCBuYXJyb3dseSBhZGFwdGVkIGNhbmNlbCBjb25zdW1lIGV4YWN0
Ci0gICByZWNlaXB0L3N1cGVydmlzb3IgdHJ1dGgsIGFuZCBmb3JlZ3JvdW5kIGRyb3AgZG9lcyBu
b3QgZXJhc2UgdGhlIGFjY2VwdGVkIG9yIHN1cGVydmlzZWQgd29yay4KLTIuIENvbnRpbnVlIGFu
IGV4YWN0IHJldGFpbmVkIHdvcmtlciwgcGVyc2lzdCBpdHMgQjEgYWNjZXB0ZWQgYWN0aXZlLXJ1
biBpZGVudGl0eSwgaGFuZCB0aGUgc2FtZSBCMAotICAgc3RyZWFtIHRvIEIyLjEgd2l0aCBubyBv
YnNlcnZhdGlvbiBnYXAsIGFuZCBwcm92ZSB0aGUgZm9yZWdyb3VuZCBtYXkgcmVtYWluIGEgYmxv
Y2tpbmcgd2FpdGVyCi0gICB3aXRob3V0IHJldGFpbmluZyBvYnNlcnZhdGlvbiBvd25lcnNoaXAu
Ci0zLiBOb3JtYWxpemUgb25lIHN1cHBvcnRlZCBwcm92aWRlciBhdHRlbnRpb24gc2hhcGUgYmVm
b3JlIGBBZ2VudEV2ZW50YCBjb25zdHJ1Y3Rpb24sIGVtaXQgb25lCi0gICBCMy4xIGV2ZW50IHdp
dGggZXhhY3QgdGFyZ2V0L3J1bi90aHJlYWQvY2xhc3MvYXR0ZW50aW9uL2NhdXNhdGlvbiBpZGVu
dGl0eSwgYW5kIHByb3ZlIEIyLjEKLSAgIGpvdXJuYWxzIGl0IGR1cmFibHkgYW5kIEMxIG1hdGVy
aWFsaXplcyBvbmUgb2JsaWdhdGlvbiBiZWZvcmUgdGhlIHRlcm1pbmFsIGV2ZW50LiBQcm92ZSBh
bgotICAgYW1iaWd1b3VzIG9yIG1hbGZvcm1lZCBhdHRlbnRpb24tZHJpdmluZyBwcm92aWRlciBz
aGFwZSBmYWlscyBiZWZvcmUgZW1pc3Npb24gcmF0aGVyIHRoYW4KLSAgIGJlY29taW5nIHByb2dy
ZXNzIG9yIG5vLWF0dGVudGlvbiwgYW5kIHByb3ZlIGhvc3QgSlNPTi1wb2ludGVyIHBhcnNpbmcg
Y2Fubm90IHJlcGFpciBpdC4gRm9yCi0gICBldmVyeSBwb3N0LWFjayByZXRhaW5lZCBgRXZlbnRg
IGZyYW1lLCBwcm92ZSBlaXRoZXIgdGhlIHR5cGVkIG1lbWJlciByZWFjaGVzIHRoZSBqb3VybmFs
L2N1dCBvcgotICAgdGhlIHN0cmVhbSBhbmQgQ29tcGxldGUgY3V0IGZhaWwgY2xvc2VkOyBubyBl
dmVudCBpcyBzaWxlbnRseSBkcm9wcGVkIG9yIGxlZnQgdW50eXBlZC4KLTQuIERyb3AgY2FsbGVy
cyBhbmQgcmVzdGFydCB0aGUgc3VwZXJ2aXNvciBkdXJpbmcgYm90aCBhY2NlcHRlZCBmYW1pbGll
cywgdGhlbiByZXBsYXkgZHVwbGljYXRlCi0gICBmcmFtZXMvZXZlbnRzIGFuZCBwcm92ZSB0aGUg
Y2xhaW0sIGpvdXJuYWwsIHJlY2VpcHQsIG9ibGlnYXRpb24sIGxlZGdlciByZXZpc2lvbiwgd2F0
ZXJtYXJrLCBhbmQKLSAgIHRlcm1pbmFsIHN0YXRlIHJlbWFpbiBpZGVtcG90ZW50LiBNaXNzaW5n
IGV4YWN0IHRlcm1pbmFsIHRydXRoIHJlbWFpbnMgaW50ZXJydXB0ZWQvbm9udGVybWluYWwuCi01
LiBRdWVyeSBDMSBiZWZvcmUgYW5kIGFmdGVyIHRoZSBleGFjdCB0ZXJtaW5hbCBjdXQ7IHByb3Zl
IFBlbmRpbmcgZmlyc3QsIHRoZW4gQ29tcGxldGUgZm9yIGJvdGgKLSAgIGVtcHR5L25vLWF0dGVu
dGlvbiBhbmQgbm9uLWVtcHR5L2F0dGVudGlvbiBkaXNwb3NpdGlvbnMgd2l0aG91dCBjb25zdWx0
aW5nIHByb2plY3Rpb25zLiBUaGUKLSAgIENvbXBsZXRlIHNuYXBzaG90J3Mgb3JkZXJlZCBldmVu
dCB2ZWN0b3IgbXVzdCBqb2luIGV4aGF1c3RpdmVseSB0byBldmVyeSByZXRhaW5lZCBCMi4xIGBF
dmVudGAKLSAgIHJlZiBpbiBzY29wZSwgaW5jbHVkaW5nIHR5cGVkIG5vbi1hdHRlbnRpb24gZXZl
bnRzLgotNi4gQWZ0ZXIgQjIuMi9CMy4yLCByZWNlaXZlIHRoZSBhY2NlcHRlZCByZWNlaXB0IGJl
Zm9yZSB0ZXJtaW5hbCBleGl0IGFuZCBjYW5jZWwgYnkKLSAgIGBhY3RpdmVfcnVuX2lkYCBpbiB0
aGUgc2FtZSBob3N0IHR1cm4uCi0KLUNvdmVyczogYFJHLUVWRU5ULTAxYCwgYFJHLVJFQ0VJUFQt
MDJgLCBgUkctUkVDRUlQVC0wM2AsIGBSRy1TVVAtMDFgLCBgUkctU1VQLTAyYCwKLWBSRy1NU0ct
MDFgLCBgUkctQ0FOQ0VMLTAxYCwgYFJHLU9CTC0wMWAsIGBSRy1PQkwtMDJgLCBgUkctT0JTLTAx
YC4KK0NvbXBhdGliaWxpdHkgYW5jaG9yIG9ubHk7IGNhbm9uaWNhbCBjb250ZW50OiBbYGV2aWRl
bmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kI3MyLS1yZWNlaXB0
LXN1cGVydmlzb3Itb2JsaWdhdGlvbi1jYW5jZWxgXShldmlkZW5jZS9iYXNlbGluZS1iZWhhdmlv
cnMtYW5kLXNtb2tlLXNjZW5hcmlvcy5tZCNzMi0tcmVjZWlwdC1zdXBlcnZpc29yLW9ibGlnYXRp
b24tY2FuY2VsKS4KIAogIyMjIFMzIOKAlCBXb3JsZC1VQUEgbWVkaWF0aW9uIHVuZGVyIG5hcnJv
d2VkIHBvbGljeQogCi0xLiBTcGF3bi9jb250aW51ZSB3b3JsZCBDb2RleCB3aXRoIHdyaXRlIG5h
cnJvd2VkIHRvIGBzcmMvcGFyc2VyLnJzYC4KLTIuIFByb3ZlIHNoZWxsIGFuZCBwcm92aWRlci1u
YXRpdmUgZWRpdCBjYW4gbW9kaWZ5IHRoYXQgZmlsZS4KLTMuIFByb3ZlIHNoZWxsLCBkaXJlY3Qg
ZWRpdCwgYW5kIHdyaXRlLWNhcGFibGUgdG9vbCBjYW5ub3QgbW9kaWZ5IGBSRUFETUUubWRgIG9y
IGVzY2FwZSByb290LgotNC4gUmVwZWF0IHdpdGggYnJva2VyIHVuYXZhaWxhYmxlIGFuZCByZXF1
aXJlIGZhaWwtY2xvc2VkIGJlaGF2aW9yLgotNS4gSm9pbiBlYWNoIG9wZXJhdGlvbiB0byB0aGUg
cmVjZWlwdCdzIGltbXV0YWJsZSBwb2xpY3kgaGFzaC4KLQotQ292ZXJzOiBgUkctVUFBLTAxYCwg
YFJHLVVBQS0wMmAsIGBSRy1VQUEtMDNgLCBgUkctUE9MSUNZLTAxYCwgYFJHLVBPTElDWS0wMmAu
CitDb21wYXRpYmlsaXR5IGFuY2hvciBvbmx5OyBjYW5vbmljYWwgY29udGVudDogW2BldmlkZW5j
ZS9iYXNlbGluZS1iZWhhdmlvcnMtYW5kLXNtb2tlLXNjZW5hcmlvcy5tZCNzMy0td29ybGQtdWFh
LW1lZGlhdGlvbi11bmRlci1uYXJyb3dlZC1wb2xpY3lgXShldmlkZW5jZS9iYXNlbGluZS1iZWhh
dmlvcnMtYW5kLXNtb2tlLXNjZW5hcmlvcy5tZCNzMy0td29ybGQtdWFhLW1lZGlhdGlvbi11bmRl
ci1uYXJyb3dlZC1wb2xpY3kpLgogCiAjIyMgUzQg4oCUIEhvc3QgdmlzaWJpbGl0eSBhbmQgZmFp
bHVyZSB0cnV0aAogCi0xLiBSdW4gZXF1aXZhbGVudCB3cml0ZXMgdW5kZXIgaG9zdC12aXNpYmxl
IGFuZCBmdWxsLWlzb2xhdGlvbiBwb2xpY2llcy4KLTIuIFZlcmlmeSBleGFjdCB2aXNpYmlsaXR5
L3JlY29uY2lsaWF0aW9uIHNlbWFudGljcy4KLTMuIEZvcmNlIGEgbm9uLXplcm8gVUFBIHR1cm4g
YWZ0ZXIgYSBwZXJtaXR0ZWQgd3JpdGUuCi00LiBQcm92ZSBkdXJhYmxlIGZhaWxlZCBhY3RpdmUt
cnVuIHN0YXRlLCByZXRhaW5lZCBhdXRob3JpdHkgYmVoYXZpb3IsIGRpYWdub3N0aWNzLCBhbmQg
ZmlsZSB2aXNpYmlsaXR5IG91dGNvbWUgaW5kZXBlbmRlbnRseS4KLQotQ292ZXJzOiBgUkctU1lO
Qy0wMWAsIGBSRy1XT1JLRVItRVhJVC0wMWAsIGBSRy1PQlMtMDFgLgorQ29tcGF0aWJpbGl0eSBh
bmNob3Igb25seTsgY2Fub25pY2FsIGNvbnRlbnQ6IFtgZXZpZGVuY2UvYmFzZWxpbmUtYmVoYXZp
b3JzLWFuZC1zbW9rZS1zY2VuYXJpb3MubWQjczQtLWhvc3QtdmlzaWJpbGl0eS1hbmQtZmFpbHVy
ZS10cnV0aGBdKGV2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9z
Lm1kI3M0LS1ob3N0LXZpc2liaWxpdHktYW5kLWZhaWx1cmUtdHJ1dGgpLgogCiAjIyMgUzUg4oCU
IEV4aXN0aW5nIGdhdGV3YXkgY2FycmllciBwcmVzZXJ2YXRpb24gYW5kIGRpcmVjdCBDb2RleCBh
ZG9wdGlvbgogCi0xLiBSdW4gdGhlIGV4aXN0aW5nIGxhdW5jaGVyL2NvbnN1bWVyIHJlZ3Jlc3Np
b24gYmFzZWxpbmU6IHZhbGlkYXRlZCBgR2F0ZXdheUF1dGhCdW5kbGVWMWAsIHJhdy1zZWNyZXQg
ZW52IHNjcnViYmluZywgb25lLXRpbWUgZ2F0ZXdheSBjb25zdW1wdGlvbiwgYW5kIGZyZXNoIGJ1
bmRsZSBkZWxpdmVyeSBvbiByZXN0YXJ0LgotMi4gUmVzb2x2ZSBhIGNyZWRlbnRpYWwgc291cmNl
IG9uIHRoZSBob3N0IGFuZCBsYXVuY2ggdGhlIGV4YWN0IG1hbmFnZWQgaW4td29ybGQgZ2F0ZXdh
eSB3aXRob3V0IGNvcHlpbmcgdGhlIHNlY3JldCBwYXlsb2FkIGludG8gYSB3b3JsZC12aXNpYmxl
IHJ1bnRpbWUgaG9tZS4KLTMuIFN0YXJ0IGRpcmVjdCB3b3JsZCBDb2RleCB3aXRoIGEgcGVyLXdv
cmtlciBTdWJzdHJhdGUtb3duZWQgcHJvamVjdGlvbiB0aGF0IHBvaW50cyBwcm92aWRlciB0cmFm
ZmljIGF0IHRoYXQgZ2F0ZXdheTsgZG8gbm90IHNlZWQgYSBob3N0IGF1dGggZmlsZSBpbiBjb250
cmFjdC1jb3JyZWN0IG1vZGUuCi00LiBKb2luIHRoZSBlbnZlbG9wZSwgYWNjZXB0ZWQgcG9saWN5
IHNuYXBzaG90LCBwcm9qZWN0aW9uIGlkZW50aXR5LCBleGFjdCBnYXRld2F5IHJlY2VpdmVyLCBh
bmQgbm9uLXNlY3JldCBoYW5kb2ZmIGV2aWRlbmNlIGZvciB0aGUgc2FtZSB3b3JsZCBnZW5lcmF0
aW9uLgotNS4gUHJvdmUgdGhlIFVBQSBjaGlsZCBhbmQgZGVzY2VuZGFudHMgY2Fubm90IGluaGVy
aXQgdGhlIHNlY3JldCBGRCBvciByYXcgc2VjcmV0IGFuZCB0aGF0IG5vIHNlY3JldC1iZWFyaW5n
IHJ1bnRpbWUtbmF0aXZlIGZpbGUgaXMgY3JlYXRlZC4KLTYuIEV4ZXJjaXNlIGR1cGxpY2F0ZSBj
b25zdW1lLCB3cm9uZyBnYXRld2F5L3dvcmxkIGdlbmVyYXRpb24sIGV4cGlyeSwgdW5hdmFpbGFi
bGUgaGFuZG9mZiwgYW5kIHVuYXZhaWxhYmxlIGdhdGV3YXkgYWRvcHRpb247IHJlcXVpcmUgZmFp
bC1jbG9zZWQgYmVoYXZpb3IuCi03LiBFeGVyY2lzZSB0aGUgZXhwbGljaXRseSBuYW1lZCBjb21w
YXRpYmlsaXR5IGNvcHkgbW9kZSBzZXBhcmF0ZWx5IGFuZCBwcm92ZSBpdCBpcyBsb2dnZWQsIGhh
cyByZXRpcmVtZW50IG1ldGFkYXRhLCBhbmQgY2Fubm90IHNhdGlzZnkgY29udHJhY3QtcHJvbW90
aW9uIGV2aWRlbmNlLgotOC4gSW5zcGVjdCBsb2dzL3RyYWNlcy9tYW5pZmVzdHMgZm9yIHNlY3Jl
dCBwYXlsb2Fkcywgc2VjcmV0LWJlYXJpbmcgcGF0aHMsIGFuZCByZXVzYWJsZSBzZWNyZXQtZGVy
aXZlZCBoYXNoZXM7IG5vbmUgbWF5IGFwcGVhci4KLQotQ292ZXJzOiBgUkctQ09ORklHLTAyYCwg
YFJHLUNPTkZJRy0wM2AsIGBSRy1DT05GSUctMDRgLCBgUkctVUFBLTAyYCwgYFJHLVVBQS0wM2As
IGBSRy1PQlMtMDFgLgorQ29tcGF0aWJpbGl0eSBhbmNob3Igb25seTsgY2Fub25pY2FsIGNvbnRl
bnQ6IFtgZXZpZGVuY2UvYmFzZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9rZS1zY2VuYXJpb3MubWQj
czUtLWV4aXN0aW5nLWdhdGV3YXktY2Fycmllci1wcmVzZXJ2YXRpb24tYW5kLWRpcmVjdC1jb2Rl
eC1hZG9wdGlvbmBdKGV2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFy
aW9zLm1kI3M1LS1leGlzdGluZy1nYXRld2F5LWNhcnJpZXItcHJlc2VydmF0aW9uLWFuZC1kaXJl
Y3QtY29kZXgtYWRvcHRpb24pLgogCiAjIyBDbG9zZW91dCBydWxlCiAKZGlmZiAtLWdpdCBhL2xs
bS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9ldmlkZW5jZS9SRUFETUUubWQgYi9sbG0tbGFz
dC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZXZpZGVuY2UvUkVBRE1FLm1kCmluZGV4IDMwYmMxYzM4
NzMwOTcyMmU3NTc4MmE0MGFjYTg0NDFhZTZiMjQ3ODYuLmM1ZjU3NDJiYTM3MTAyMjlhZWJkNmY1
YWY1MmRlODk3M2Q0N2IyM2IgMTAwNjQ0Ci0tLSBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZh
Y3Rvci9ldmlkZW5jZS9SRUFETUUubWQKKysrIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFj
dG9yL2V2aWRlbmNlL1JFQURNRS5tZApAQCAtMTgsMyArMTgsNCBAQAogfC0tLXwtLS18LS0tfAog
fCBCYXRjaCAxIOKAlCBjYW5vbmljYWwgaXNzdWUgbGVkZ2VyIHwgW2BjYW5vbmljYWwtaXNzdWUt
bGVkZ2VyLm1kI2Nhbm9uaWNhbC1pc3N1ZS1sZWRnZXJgXShjYW5vbmljYWwtaXNzdWUtbGVkZ2Vy
Lm1kI2Nhbm9uaWNhbC1pc3N1ZS1sZWRnZXIpIHwgRnVsbCBleHRyYWN0ZWQgYCMjIENhbm9uaWNh
bCBpc3N1ZSBsZWRnZXJgIHRhYmxlIHBsdXMgdGhlIHByZXNlcnZlZCBgIyMjIEExLjJhLVdCIGdh
dGUgYXNzaWdubWVudGAgYm9keTsgZXhhY3QgaXNzdWUgSURzLCBjaHJvbm9sb2d5LCBsaW1pdGF0
aW9ucywgb2JsaWdhdGlvbnMsIGxpdGVyYWxzLCBoYXNoZXMsIGNvbW1hbmRzLCBhbmQgbmVnYXRp
dmUgcmVxdWlyZW1lbnRzIHJlbWFpbiB1bmNoYW5nZWQuIHwKIHwgQmF0Y2ggMiDigJQgY2xvc2Vv
dXQgcnVsZSArIHJldmlldy1wcm9jZXNzIGNhbGlicmF0aW9uIHwgW2BjbG9zZW91dC1hbmQtcmV2
aWV3LWNhbGlicmF0aW9uLm1kYF0oY2xvc2VvdXQtYW5kLXJldmlldy1jYWxpYnJhdGlvbi5tZCkg
fCBFeGFjdCBleHRyYWN0ZWQgW2AjIyBDbG9zZW91dCBydWxlYF0oY2xvc2VvdXQtYW5kLXJldmll
dy1jYWxpYnJhdGlvbi5tZCNjbG9zZW91dC1ydWxlKSBhbmQgW2AjIyBSZXZpZXctcHJvY2VzcyBj
YWxpYnJhdGlvbmBdKGNsb3Nlb3V0LWFuZC1yZXZpZXctY2FsaWJyYXRpb24ubWQjcmV2aWV3LXBy
b2Nlc3MtY2FsaWJyYXRpb24pIGJvZGllcyB3aXRoIHNlcGFyYXRlIG5vbmNvbnRpZ3VvdXMgc291
cmNlIHByb3ZlbmFuY2U7IG9yZGVyZWQgYnVsbGV0cywgbGl0ZXJhbHMsIElEcywgbmVnYXRpdmUg
cmVxdWlyZW1lbnRzLCBleGNlcHRpb25zLCByZXZpZXctZ292ZXJuYW5jZSByZWZlcmVuY2Utb25s
eSBib3VuZGFyaWVzLCBhbmQgdGhlIG5vLXJlbWVkaWF0aW9uL25vLXByb29mLXJlc3VsdC1jaGFu
Z2Ugc3RhdGVtZW50IHJlbWFpbiB1bmNoYW5nZWQuIHwKK3wgQmF0Y2ggMyDigJQgYmFzZWxpbmUg
YmVoYXZpb3JzICsgY3Jvc3MtZ2F0ZSBzbW9rZSBzY2VuYXJpb3MgfCBbYGJhc2VsaW5lLWJlaGF2
aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kYF0oYmFzZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9r
ZS1zY2VuYXJpb3MubWQpIHwgRXhhY3QgZXh0cmFjdGVkIFtgIyMgQmFzZWxpbmUgYmVoYXZpb3Jz
IHRoYXQgYWxsIHRyYWNrcyBwcmVzZXJ2ZWBdKGJhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Ut
c2NlbmFyaW9zLm1kI2Jhc2VsaW5lLWJlaGF2aW9ycy10aGF0LWFsbC10cmFja3MtcHJlc2VydmUp
IGFuZCBbYCMjIENyb3NzLWdhdGUgc21va2Ugc2NlbmFyaW9zYF0oYmFzZWxpbmUtYmVoYXZpb3Jz
LWFuZC1zbW9rZS1zY2VuYXJpb3MubWQjY3Jvc3MtZ2F0ZS1zbW9rZS1zY2VuYXJpb3MpIGJvZGll
cyB3aXRoIHNlcGFyYXRlIGV4YWN0IHNvdXJjZSBwcm92ZW5hbmNlIHByZXNlcnZlZCBiZXR3ZWVu
IGRpc3RpbmN0IGJvdW5kYXJ5IG1hcmtlcnM7IEdhdGUgSURzLCBzdGF0dXNlcywgdGFibGUgb3Jk
ZXIvY2VsbHMsIG9yZGVyZWQgc2NlbmFyaW8gc3RlcHMsIGRldGVybWluaXN0aWMgcGFya2VkLXN1
Y2Nlc3NvciBzZXF1ZW5jZSwgQ292ZXJzIGxpbmVzLCBhbmQgZmFpbC1jbG9zZWQvZXhjZXB0aW9u
IGJvdW5kYXJpZXMgcmVtYWluIHVuY2hhbmdlZC4gfApkaWZmIC0tZ2l0IGEvbGxtLWxhc3QtbWls
ZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZCBiL2xsbS1sYXN0LW1pbGUvcnVudGlt
ZS1yZWZhY3Rvci9pbmRleC9SRUFETUUubWQKaW5kZXggNzc1NDk1M2QxOWRkYzVmYjk2NTUxM2Ew
MGI3MzkwYjVhZGZhODAxOC4uNDExOWQ4MWQ5ODdkOGI5YWU4MjRiZjY5YTg2YWEyMDI2MzA2YWZk
NiAxMDA2NDQKLS0tIGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURN
RS5tZAorKysgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvaW5kZXgvUkVBRE1FLm1k
CkBAIC0zNCw2ICszNCw3IEBACiB8IGBzaGFyZWQtc2xpY2UtbWFwYCB8IHNsaWNlIGluZGV4IHwg
W2BzbGljZXMvUkVBRE1FLm1kYF0oLi4vc2xpY2VzL1JFQURNRS5tZCkgfCBgY2Fub25pY2FsIHNo
YXJlZCBzZXF1ZW5jaW5nL2RlcGVuZGVuY3kgYW5kIHNsaWNlLWNsb3Nlb3V0IG93bmVyIHBsdXMg
bm9uLWF1dGhvcml0YXRpdmUgRDkgVHJhY2sgQeKAk0UgbmF2aWdhdGlvbiBhbmQgQTEvQTEuNCBw
cm9qZWN0aW9uIGluZGV4YCB8IFN1cGVyc2VkZXMgb25seSByb290LWNhbm9uaWNhbCBvd25lcnNo
aXAgb2YgdGhlIGV4dHJhY3RlZCBzaGFyZWQgc2VxdWVuY2luZy9jbG9zZW91dCBzcGFucyBwbHVz
IHRoZSBleHRyYWN0ZWQgVHJhY2sgQeKAk0UgYW5kIEExL0ExLjQgcHJvamVjdGlvbiBzcGFuczsg
Y29udHJvbGxpbmcgc2NoZWR1bGUgYXV0aG9yaXR5IHJlbWFpbnMgd2l0aCBkZWNpc2lvbnMsIHBh
Y2tldHMsIGdhdGVzLCBhbmQgYGN1cnJlbnQubWRgLiB8IFtgMDNgXSguLi8wMy1waGFzZS1zbGlj
ZS1tYXAubWQjc2VxdWVuY2luZy1ydWxlcyksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAu
bWQjdHJhY2stYS0tYXV0aG9yaXR5LWFuZC1zdXJmYWNlLW5ldXRyYWxpdHkpLCBbYDAzYF0oLi4v
MDMtcGhhc2Utc2xpY2UtbWFwLm1kI2ExLWJvdW5kZWQtcGFja2V0LWRlY29tcG9zaXRpb24pLCBb
YDAzYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3RyYWNrLWItLXdvcmxkLWRpc3BhdGNoLXJl
Y2VpcHRzLXN1cGVydmlzaW9uLWFuZC1jYW5jZWwpLCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2Ut
bWFwLm1kI3RyYWNrLWMtLW9ibGlnYXRpb25zLWluYm94LWF1dG8tYXR0YWNoLWFuZC1yb3V0ZXIt
YXR0YWNoKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCN0cmFjay1kLS11YWEtZXhl
Y3V0aW9uLWVudmVsb3BlLWFuZC1zaWRlLWVmZmVjdC1tZWRpYXRpb24pLCBbYDAzYF0oLi4vMDMt
cGhhc2Utc2xpY2UtbWFwLm1kI3RyYWNrLWUtLWRpc3BhdGNoLXNjb3BlZC1wb2xpY3ktbmFycm93
aW5nLWFuZC1jb25maWctcHJvamVjdGlvbiksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAu
bWQjc2xpY2UtY2xvc2VvdXQtbWluaW11bSkgfAogfCBDYW5vbmljYWwgaXNzdWUgbGVkZ2VyIHwg
ZXZpZGVuY2UvcmVncmVzc2lvbiB8IFtgZXZpZGVuY2UvY2Fub25pY2FsLWlzc3VlLWxlZGdlci5t
ZGBdKC4uL2V2aWRlbmNlL2Nhbm9uaWNhbC1pc3N1ZS1sZWRnZXIubWQpIHwgYGNhbm9uaWNhbCBl
eHRyYWN0ZWQgb3duZXIgZm9yIHRoZSBmdWxsIGlzc3VlL3JlZ3Jlc3Npb24tb2JsaWdhdGlvbiBs
ZWRnZXIgdGFibGUgcGx1cyB0aGUgcHJlc2VydmVkIEExLjJhLVdCIGdhdGUtYXNzaWdubWVudCBi
b2R5YCB8IFN1cGVyc2VkZXMgb25seSByb290LWNhbm9uaWNhbCBvd25lcnNoaXAgb2YgdGhlIGV4
dHJhY3RlZCBgIyMgQ2Fub25pY2FsIGlzc3VlIGxlZGdlcmAgYm9keTsgdGhlIGxlZ2FjeSBgIyMg
Q2Fub25pY2FsIGlzc3VlIGxlZGdlcmAgYW5kIGAjIyMgQTEuMmEtV0IgZ2F0ZSBhc3NpZ25tZW50
YCBoZWFkaW5ncyByZW1haW4gY29tcGF0aWJpbGl0eSBhbmNob3JzIHVudGlsIEQxMi4gfCBbYDA1
YF0oLi4vMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjY2Fub25pY2FsLWlzc3VlLWxlZGdl
ciksIFtgMDVgXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNhMTJhLXdiLWdhdGUt
YXNzaWdubWVudCkgfAogfCBDbG9zZW91dCBydWxlICsgUmV2aWV3LXByb2Nlc3MgY2FsaWJyYXRp
b24gfCBldmlkZW5jZS9yZWdyZXNzaW9uIHwgW2BldmlkZW5jZS9jbG9zZW91dC1hbmQtcmV2aWV3
LWNhbGlicmF0aW9uLm1kYF0oLi4vZXZpZGVuY2UvY2xvc2VvdXQtYW5kLXJldmlldy1jYWxpYnJh
dGlvbi5tZCkgfCBgY2Fub25pY2FsIGV4dHJhY3RlZCBjb21wb3VuZCBvd25lciBmb3IgdGhlIGV4
YWN0IG5vbmNvbnRpZ3VvdXMgQ2xvc2VvdXQgcnVsZSBhbmQgUmV2aWV3LXByb2Nlc3MgY2FsaWJy
YXRpb24gZXZpZGVuY2UvcmVncmVzc2lvbiBib2RpZXNgIHwgU3VwZXJzZWRlcyBvbmx5IHJvb3Qt
Y2Fub25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIGAjIyBDbG9zZW91dCBydWxlYCBh
bmQgYCMjIFJldmlldy1wcm9jZXNzIGNhbGlicmF0aW9uYCBib2RpZXM7IHRoZSByb290IGhlYWRp
bmdzIHJlbWFpbiBjb21wYXRpYmlsaXR5IGFuY2hvcnMsIGFuZCB0aGUgZGV2ZWxvcG1lbnQtcmV2
aWV3IGNvbnRyYWN0IHJlbWFpbnMgcmVmZXJlbmNlLW9ubHkgd2l0aG91dCByZW9wZW5pbmcgRDMu
IHwgW2AwNWBdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2Nsb3Nlb3V0LXJ1bGUp
LCBbYDA1YF0oLi4vMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjcmV2aWV3LXByb2Nlc3Mt
Y2FsaWJyYXRpb24pIHwKK3wgQmFzZWxpbmUgYmVoYXZpb3JzICsgQ3Jvc3MtZ2F0ZSBzbW9rZSBz
Y2VuYXJpb3MgfCBldmlkZW5jZS9yZWdyZXNzaW9uIHwgW2BldmlkZW5jZS9iYXNlbGluZS1iZWhh
dmlvcnMtYW5kLXNtb2tlLXNjZW5hcmlvcy5tZGBdKC4uL2V2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2
aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kKSB8IGBjYW5vbmljYWwgZXh0cmFjdGVkIGNvbXBv
dW5kIG93bmVyIGZvciB0aGUgZXhhY3QgYmFzZWxpbmUtYmVoYXZpb3IgdGFibGUgYW5kIGNyb3Nz
LWdhdGUgc21va2Utc2NlbmFyaW8gYm9kaWVzYCB8IFN1cGVyc2VkZXMgb25seSByb290LWNhbm9u
aWNhbCBvd25lcnNoaXAgb2YgdGhlIGV4dHJhY3RlZCBgIyMgQmFzZWxpbmUgYmVoYXZpb3JzIHRo
YXQgYWxsIHRyYWNrcyBwcmVzZXJ2ZWAgYW5kIGAjIyBDcm9zcy1nYXRlIHNtb2tlIHNjZW5hcmlv
c2AgYm9kaWVzOyB0aGUgbGVnYWN5IHJvb3QgaGVhZGluZ3MgYW5kIHNjZW5hcmlvIHN1YmhlYWRp
bmdzIHJlbWFpbiBjb21wYXRpYmlsaXR5IGFuY2hvcnMgdW50aWwgRDEyLCBhbmQgYHJldmlldy1j
b250cm9sL2AgcmVtYWlucyB0aGUgY2Fub25pY2FsIHJlY2VpcHQgc3RvcmUuIHwgW2AwNWBdKC4u
LzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2Jhc2VsaW5lLWJlaGF2aW9ycy10aGF0LWFs
bC10cmFja3MtcHJlc2VydmUpLCBbYDA1YF0oLi4vMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIu
bWQjY3Jvc3MtZ2F0ZS1zbW9rZS1zY2VuYXJpb3MpIHwKIHwgYEExLjFkLTVSMi00YCB8IHBhY2tl
dCBpbmRleCB8IFtgYTEuMWQtNXIyLTQvUkVBRE1FLm1kYF0oLi4vYTEuMWQtNXIyLTQvUkVBRE1F
Lm1kKSB8IGBjbG9zZWQgb25seSBhcyB0aGUgYm91bmRlZCBSMiBwcm9wYWdhdGlvbiBqb2luYCB8
IEFzc2VtYmxlcyB0aGUgZml2ZSBleHRyYWN0ZWQgY2Fub25pY2FsIFIyLTQgY29tcG9uZW50IG93
bmVyczsgZXhpc3RpbmcgYHJldmlldy1jb250cm9sL2AgcmVjb3JkcyBzdGF5IGF0IHRoZWlyIGN1
cnJlbnQgcGF0aHMuIHwgW2AwMWBdKC4uLzAxLXRhcmdldC1hcmNoaXRlY3R1cmUubWQjcjItNC1i
b3VuZGVkLWNsb3Nlb3V0LWFyY2hpdGVjdHVyZS1kaXNwb3NpdGlvbiksIFtgMDJgXSguLi8wMi1z
ZWFtLWNyb3Nzd2Fsay5tZCNyMi00LXRlcm1pbmFsLWNyb3Nzd2Fsay1kaXNwb3NpdGlvbiksIFtg
MDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjYTExZC01cjItNC0tcjItaW50ZWdyYXRpb24t
YW5kLWNsb3Nlb3V0KSwgW2AwNGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjYTExZC01
cjItNC10ZXJtaW5hbC1nYXRlLWRpc3Bvc2l0aW9uKSwgW2AwNWBdKC4uLzA1LWRlYnVnLXJlZ3Jl
c3Npb24tbGVkZ2VyLm1kI2ExMWQtNXIyLTQtdGVybWluYWwtZXZpZGVuY2UtbGVkZ2VyKSB8CiB8
IGBBMS4xZC01UjMtZmFtaWx5YCB8IHBhY2tldCBpbmRleCB8IFtgYTEuMWQtNXIzL1JFQURNRS5t
ZGBdKC4uL2ExLjFkLTVyMy9SRUFETUUubWQpIHwgYGFyY2hpdmVkIGZvciBhY3RpdmUgc2NoZWR1
bGluZzsgbm9uLWF1dGhvcml0YXRpdmUgbmF2aWdhdGlvbiBmb3IgdGhlIGV4dHJhY3RlZCBSMyBp
bXBsZW1lbnRhdGlvbi1mYW1pbHkgb3duZXJzYCB8IEFzc2VtYmxlcyB0aGUgYXJjaGl2ZWQgcGxh
bm5pbmcvY3VycmVudC1zdGF0ZSBwcm9qZWN0aW9ucywgbGlmZWN5Y2xlIGFyY2hpdGVjdHVyZSwg
Y3Jvc3N3YWxrLCBpbXBsZW1lbnRhdGlvbiBpbmRleCwgY29udHJhY3RzLCBwcm9vZiBsZWRnZXIs
IGFuZCB0aGVpciBzdGF0dXMgYXBwZW5kaWNlcyB3aXRob3V0IGRpc3BhdGNoaW5nIFIzLCBuYXRp
dmUgZXZpZGVuY2UsIG9yIHJlY292ZXJ5IGltcGxlbWVudGF0aW9uLiB8IFtgMDBgXSguLi8wMC1S
RUFETUUubWQjYTExZC01cjMtcGxhbi1hdXRob3JpdGF0aXZlLXBsYW5uaW5nLXN0YXR1cy1hcmNo
aXZlZC1mb3ItYWN0aXZlLXNjaGVkdWxpbmcpLCBbYDAxYF0oLi4vMDEtdGFyZ2V0LWFyY2hpdGVj
dHVyZS5tZCNhMTFkLTVyMy1saWZlY3ljbGUtb3duZXJzaGlwLWFyY2hpdGVjdHVyZSksIFtgMDJg
XSguLi8wMi1zZWFtLWNyb3Nzd2Fsay5tZCNhMTFkLTVyMy1jYW5vbmljYWwtb3duZXJzaGlwLWFu
ZC1zb3VyY2UtY2xvc3VyZS1jcm9zc3dhbGspLCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFw
Lm1kI2ExMWQtNXIzLWF1dGhvcml0YXRpdmUtaW1wbGVtZW50YXRpb24taW5kZXgpLCBbYDA0YF0o
Li4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCNhMTFkLTVyMy1saWZlY3ljbGUtY29udHJhY3Rz
LWFuZC1nYXRlcyksIFtgMDVgXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNhMTFk
LTVyMy1wbGFubmVkLXByb29mLWFuZC1yZWdyZXNzaW9uLWxlZGdlci1hcmNoaXZlZC1mb3ItYWN0
aXZlLXNjaGVkdWxpbmcpIHwKIHwgYHIzLW1hYy1ldmlkZW5jZS1yZWNvdmVyeS1mYW1pbHlgIHwg
cGxhbm5pbmcgaW5kZXggfCBbYHIzLW1hYy1ldmlkZW5jZS1yZWNvdmVyeS9SRUFETUUubWRgXSgu
Li9yMy1tYWMtZXZpZGVuY2UtcmVjb3ZlcnkvUkVBRE1FLm1kKSB8IGBwbGFubmluZy1vbmx5IGF1
dGhvcml0eSBwcmVzZXJ2ZWQ7IG5vbi1hdXRob3JpdGF0aXZlIG5hdmlnYXRpb24gZm9yIGV4dHJh
Y3RlZCByZWNvdmVyeSBzdGF0dXMgYW5kIGNvcnJlY3Rpb24gcHJvamVjdGlvbnNgIHwgQXNzZW1i
bGVzIHRoZSBleGlzdGluZyBgUExBTi5tZGAvYFNQRUMubWRgL2BUQVNLUy5tZGAgcGFja2V0IHRv
Z2V0aGVyIHdpdGggdGhlIGV4dHJhY3RlZCAwMC8wMi8wMy8wNC8wNSByZWNvdmVyeSBzdGF0dXMv
Y29ycmVjdGlvbiBwcm9qZWN0aW9ucyB3aXRob3V0IGF1dGhvcml6aW5nIHJlY292ZXJ5IGltcGxl
bWVudGF0aW9uLCBuYXRpdmUgZXZpZGVuY2UsIG9yIE1BQyBjbG9zZW91dC4gfCBbYDAwYF0oLi4v
MDAtUkVBRE1FLm1kI2ExMWQtNXIzLW1hYy1hdHRlbXB0LTQtcmVtZWRpYXRpb24tc3RhdHVzLTIw
MjYtMDgtMDYpLCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjYTExZC01cjMtbWFjLWF0
dGVtcHQtNC1yZW1lZGlhdGlvbi1zdGF0dXMtMjAyNi0wOC0wNiksIFtgMDNgXSguLi8wMy1waGFz
ZS1zbGljZS1tYXAubWQjYXV4LXIzLW1hYy1ldmlkZW5jZS1yZWNvdmVyeS1wbGFuLXBhY2tldC1z
dGF0dXMtMjAyNi0wOC0wNyksIFtgMDRgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kI2F1
eC1yMy1tYWMtZXZpZGVuY2UtcmVjb3ZlcnktcGxhbi1yZWNvdmVyeS1kZWNpc2lvbi0yMDI2LTA4
LTA3KSwgW2AwNWBdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2F1eC1yMy1tYWMt
ZXZpZGVuY2UtcmVjb3ZlcnktcGxhbi1yZWdyZXNzaW9uLXN0YXR1cy0yMDI2LTA4LTA3KSB8CmRp
ZmYgLS1naXQgYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJh
Y3Rpb24tbGVkZ2VyLm1kIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlv
bi9leHRyYWN0aW9uLWxlZGdlci5tZAppbmRleCBiOTE3NGNkMWE4YWZiZjg4NGJlZDkxOGNmMjhl
YmQxNzhjMjI0MTMxLi41ZjVkNWY1MDQ2OGRhOTNkZmJkMDZjOGY5Njc3NzFhM2JiYzExNTQ2IDEw
MDY0NAotLS0gYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJh
Y3Rpb24tbGVkZ2VyLm1kCisrKyBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdy
YXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQKQEAgLTE5OSwzICsxOTksNSBAQCBUaGlzIGxlZGdl
ciByZWNvcmRzIGNvbnRlbnQtcHJlc2VydmluZyBhdXRob3JpdHkgdHJhbnNmZXJzIHdoaWxlIHJl
dGFpbmluZyByZXF1aQogfCBEMTEgfCBbYDA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kYF0o
Li4vMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQpIHwgQ2Fub25pY2FsIGlzc3VlIGxlZGdl
ciArIEExLjJhLVdCIGdhdGUgYXNzaWdubWVudCB8IFtgY2Fub25pY2FsLWlzc3VlLWxlZGdlcmBd
KC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2Nhbm9uaWNhbC1pc3N1ZS1sZWRnZXIp
OyBbYGExMmEtd2ItZ2F0ZS1hc3NpZ25tZW50YF0oLi4vMDUtZGVidWctcmVncmVzc2lvbi1sZWRn
ZXIubWQjYTEyYS13Yi1nYXRlLWFzc2lnbm1lbnQpIHwgbGluZXMgMTbigJM5NyAoNDE0OTIgYnl0
ZXMpIHwgYDE5MWE2MmVhMGM3NzA2NDk0MTk0NzI4MGQ4N2M1MzkxNjNhYjY5Yjc3MmE4MDE4ZjZi
MmY0ZTNkM2U1YWFlM2VgIHwgW2AuLi9ldmlkZW5jZS9jYW5vbmljYWwtaXNzdWUtbGVkZ2VyLm1k
YF0oLi4vZXZpZGVuY2UvY2Fub25pY2FsLWlzc3VlLWxlZGdlci5tZCkgfCBldmlkZW5jZS9yZWdy
ZXNzaW9uIHwgY2Fub25pY2FsIGRlc3RpbmF0aW9uOyBzb3VyY2UgY29tcGF0aWJpbGl0eSBhbmNo
b3JzIGZvciBib3RoIHByZXNlcnZlZCByb290IGhlYWRpbmdzIHwgbm9uZTsgZXhhY3QgNDE0OTIt
Ynl0ZSBzb3VyY2UgYm9keSBwcmVzZXJ2ZWQgYmV0d2VlbiBvd25lciBib3VuZGFyeSBtYXJrZXJz
IHwgcmVzdG9yZSB0aGUgZXhhY3QgNDE0OTItYnl0ZSBzb3VyY2UgYm9keSBhdCBgMDUtZGVidWct
cmVncmVzc2lvbi1sZWRnZXIubWQjY2Fub25pY2FsLWlzc3VlLWxlZGdlcmAgYW5kIGAjYTEyYS13
Yi1nYXRlLWFzc2lnbm1lbnRgIGluIG9yaWdpbmFsIG9yZGVyOyByZW1vdmUgYGV2aWRlbmNlL2Nh
bm9uaWNhbC1pc3N1ZS1sZWRnZXIubWRgOyByZW1vdmUgdGhlIHNvbGUgQmF0Y2gtMSByb3cgZnJv
bSBgZXZpZGVuY2UvUkVBRE1FLm1kYCBhbmQgZGVsZXRlIGBldmlkZW5jZS9SRUFETUUubWRgIGlm
IG5vIG5hdmlnYXRpb24gcm93cyByZW1haW47IHJlbW92ZSB0aGUgZXhhY3QgYENhbm9uaWNhbCBp
c3N1ZSBsZWRnZXJgIHJvdyBmcm9tIGBpbmRleC9SRUFETUUubWRgOyByZW1vdmUgdGhpcyBEMTEg
bGVkZ2VyIGVudHJ5OyBsZWF2ZSBEM+KAk0QxMCBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwg
YW5kIGByZXZpZXctY29udHJvbC9gIHVuY2hhbmdlZCB8CiB8IEQxMSB8IFtgMDUtZGVidWctcmVn
cmVzc2lvbi1sZWRnZXIubWRgXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCkgfCBD
bG9zZW91dCBydWxlIHwgW2BjbG9zZW91dC1ydWxlYF0oLi4vMDUtZGVidWctcmVncmVzc2lvbi1s
ZWRnZXIubWQjY2xvc2VvdXQtcnVsZSkgfCBsaW5lcyAxOTPigJMyMDEgKDM0NyBieXRlcykgfCBg
ZjgxMTQyYTJiZjZkMGNiZDZiMzFhMjcwMTEyNjBkYTI1MTA4Y2M1MThiNTgyZGQxYmU4MTI2OGY5
NDYzNTQ2MGAgfCBbYC4uL2V2aWRlbmNlL2Nsb3Nlb3V0LWFuZC1yZXZpZXctY2FsaWJyYXRpb24u
bWRgXSguLi9ldmlkZW5jZS9jbG9zZW91dC1hbmQtcmV2aWV3LWNhbGlicmF0aW9uLm1kKSB8IGV2
aWRlbmNlL3JlZ3Jlc3Npb24gfCBjYW5vbmljYWwgY29tcG91bmQgZGVzdGluYXRpb247IHNvdXJj
ZSBjb21wYXRpYmlsaXR5IGFuY2hvcjsgc2VwYXJhdGUgbm9uY29udGlndW91cyBwcm92ZW5hbmNl
IHByZXNlcnZlZCB8IG5vbmU7IGV4YWN0IDM0Ny1ieXRlIHNvdXJjZSBib2R5IHByZXNlcnZlZCBi
ZXR3ZWVuIHRoZSBgY2xvc2VvdXQtcnVsZWAgb3duZXIgYm91bmRhcnkgbWFya2VycyBpbiB0aGUg
Y29tcG91bmQgZXZpZGVuY2Ugb3duZXIgfCByZXN0b3JlIHRoZSBleGFjdCAzNDctYnl0ZSBzb3Vy
Y2UgYm9keSBhdCBgMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjY2xvc2VvdXQtcnVsZWAg
YW5kIHRoZSBleGFjdCAxMzgyLWJ5dGUgc291cmNlIGJvZHkgYXQgYDA1LWRlYnVnLXJlZ3Jlc3Np
b24tbGVkZ2VyLm1kI3Jldmlldy1wcm9jZXNzLWNhbGlicmF0aW9uYCBpbiBvcmlnaW5hbCBvcmRl
cjsgcmVtb3ZlIGBldmlkZW5jZS9jbG9zZW91dC1hbmQtcmV2aWV3LWNhbGlicmF0aW9uLm1kYDsg
cmVtb3ZlIHRoZSBzb2xlIEJhdGNoLTIgcm93IGZyb20gYGV2aWRlbmNlL1JFQURNRS5tZGA7IHJl
bW92ZSB0aGUgZXhhY3QgYENsb3Nlb3V0IHJ1bGUgKyBSZXZpZXctcHJvY2VzcyBjYWxpYnJhdGlv
bmAgcm93IGZyb20gYGluZGV4L1JFQURNRS5tZGA7IHJlbW92ZSBib3RoIEQxMSBsZWRnZXIgZW50
cmllcyBmb3IgdGhpcyBiYXRjaDsgbGVhdmUgQmF0Y2ggMSBwbHVzIEQz4oCTRDEwIG93bmVycywg
YGluZGV4L2N1cnJlbnQubWRgLCBgcmV2aWV3LWNvbnRyb2wvYCwgYW5kIGV2ZXJ5IHBhdGggb3V0
c2lkZSB0aGUgZml2ZS1wYXRoIGZlbmNlIHVuY2hhbmdlZCB8CiB8IEQxMSB8IFtgMDUtZGVidWct
cmVncmVzc2lvbi1sZWRnZXIubWRgXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCkg
fCBSZXZpZXctcHJvY2VzcyBjYWxpYnJhdGlvbiB8IFtgcmV2aWV3LXByb2Nlc3MtY2FsaWJyYXRp
b25gXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNyZXZpZXctcHJvY2Vzcy1jYWxp
YnJhdGlvbikgfCBsaW5lcyAzMzLigJMzNTMgKDEzODIgYnl0ZXMpIHwgYDI4NDI4NGFjNTA3YTli
ZGIzMjYzZjJjMThiYzYxNWMwNWRmM2ZkMjNlZDMzYTdhNjc0MmY4NWUyOWZiZTE3NWVgIHwgW2Au
Li9ldmlkZW5jZS9jbG9zZW91dC1hbmQtcmV2aWV3LWNhbGlicmF0aW9uLm1kYF0oLi4vZXZpZGVu
Y2UvY2xvc2VvdXQtYW5kLXJldmlldy1jYWxpYnJhdGlvbi5tZCkgfCBldmlkZW5jZS9yZWdyZXNz
aW9uIHwgY2Fub25pY2FsIGNvbXBvdW5kIGRlc3RpbmF0aW9uOyBzb3VyY2UgY29tcGF0aWJpbGl0
eSBhbmNob3I7IHNlcGFyYXRlIG5vbmNvbnRpZ3VvdXMgcHJvdmVuYW5jZSBwcmVzZXJ2ZWQgfCBu
b25lOyBleGFjdCAxMzgyLWJ5dGUgc291cmNlIGJvZHkgcHJlc2VydmVkIGJldHdlZW4gdGhlIGBy
ZXZpZXctcHJvY2Vzcy1jYWxpYnJhdGlvbmAgb3duZXIgYm91bmRhcnkgbWFya2VycyBpbiB0aGUg
Y29tcG91bmQgZXZpZGVuY2Ugb3duZXI7IHRoZSBkZXZlbG9wbWVudC1yZXZpZXcgY29udHJhY3Qg
cmVsYXRpb25zaGlwIHJlbWFpbnMgcmVmZXJlbmNlLW9ubHkgYW5kIGRvZXMgbm90IHJlb3BlbiBE
MyB8IHJlc3RvcmUgdGhlIGV4YWN0IDM0Ny1ieXRlIHNvdXJjZSBib2R5IGF0IGAwNS1kZWJ1Zy1y
ZWdyZXNzaW9uLWxlZGdlci5tZCNjbG9zZW91dC1ydWxlYCBhbmQgdGhlIGV4YWN0IDEzODItYnl0
ZSBzb3VyY2UgYm9keSBhdCBgMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjcmV2aWV3LXBy
b2Nlc3MtY2FsaWJyYXRpb25gIGluIG9yaWdpbmFsIG9yZGVyOyByZW1vdmUgYGV2aWRlbmNlL2Ns
b3Nlb3V0LWFuZC1yZXZpZXctY2FsaWJyYXRpb24ubWRgOyByZW1vdmUgdGhlIHNvbGUgQmF0Y2gt
MiByb3cgZnJvbSBgZXZpZGVuY2UvUkVBRE1FLm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgQ2xvc2Vv
dXQgcnVsZSArIFJldmlldy1wcm9jZXNzIGNhbGlicmF0aW9uYCByb3cgZnJvbSBgaW5kZXgvUkVB
RE1FLm1kYDsgcmVtb3ZlIGJvdGggRDExIGxlZGdlciBlbnRyaWVzIGZvciB0aGlzIGJhdGNoOyBs
ZWF2ZSBCYXRjaCAxIHBsdXMgRDPigJNEMTAgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGBy
ZXZpZXctY29udHJvbC9gLCBhbmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmaXZlLXBhdGggZmVu
Y2UgdW5jaGFuZ2VkIHwKK3wgRDExIHwgW2AwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZGBd
KC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kKSB8IEJhc2VsaW5lIGJlaGF2aW9ycyB0
aGF0IGFsbCB0cmFja3MgcHJlc2VydmUgfCBbYGJhc2VsaW5lLWJlaGF2aW9ycy10aGF0LWFsbC10
cmFja3MtcHJlc2VydmVgXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNiYXNlbGlu
ZS1iZWhhdmlvcnMtdGhhdC1hbGwtdHJhY2tzLXByZXNlcnZlKSB8IGxpbmVzIDgw4oCTOTIgKDM2
NDQgYnl0ZXMpIHwgYDVmOWVjOGQ5ODMyNDAxNTk4NmM4NDYyMDhkYjFjNTY3YjQ5YzkwOWJhYTcx
NDQyOGFjM2E2NDQ3MDlmZTkxMzJgIHwgW2AuLi9ldmlkZW5jZS9iYXNlbGluZS1iZWhhdmlvcnMt
YW5kLXNtb2tlLXNjZW5hcmlvcy5tZGBdKC4uL2V2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1h
bmQtc21va2Utc2NlbmFyaW9zLm1kKSB8IGV2aWRlbmNlL3JlZ3Jlc3Npb24gfCBjYW5vbmljYWwg
Y29tcG91bmQgZGVzdGluYXRpb247IHNvdXJjZSBjb21wYXRpYmlsaXR5IGFuY2hvcjsgc2VwYXJh
dGUgZXhhY3QtYm9keSBwcm92ZW5hbmNlIHByZXNlcnZlZCB8IHBhaXJlZCBmYW1pbHkgYXRvbWlj
aXR5IHdpdGggYENyb3NzLWdhdGUgc21va2Ugc2NlbmFyaW9zYDsgZXhhY3QgMzY0NC1ieXRlIHNv
dXJjZSBib2R5IHByZXNlcnZlZCBiZXR3ZWVuIHRoZSBgYmFzZWxpbmUtYmVoYXZpb3JzYCBvd25l
ciBib3VuZGFyeSBtYXJrZXJzIGFuZCBub3QgcmVwcmVzZW50ZWQgYXMgb25lIGNvbnRpZ3VvdXMg
ZXh0cmFjdGlvbiB3aXRoIHRoZSBsYXRlciBzbW9rZS1zY2VuYXJpbyBib2R5IHwgcmVzdG9yZSB0
aGUgZXhhY3QgMzY0NC1ieXRlIHNvdXJjZSBib2R5IGF0IGAwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxl
ZGdlci5tZCNiYXNlbGluZS1iZWhhdmlvcnMtdGhhdC1hbGwtdHJhY2tzLXByZXNlcnZlYCBhbmQg
dGhlIGV4YWN0IDExNzE0LWJ5dGUgc291cmNlIGJvZHkgYXQgYDA1LWRlYnVnLXJlZ3Jlc3Npb24t
bGVkZ2VyLm1kI2Nyb3NzLWdhdGUtc21va2Utc2NlbmFyaW9zYCBpbiBvcmlnaW5hbCBvcmRlcjsg
cmVtb3ZlIGBldmlkZW5jZS9iYXNlbGluZS1iZWhhdmlvcnMtYW5kLXNtb2tlLXNjZW5hcmlvcy5t
ZGA7IHJlbW92ZSB0aGUgc29sZSBCYXRjaC0zIHJvdyBmcm9tIGBldmlkZW5jZS9SRUFETUUubWRg
OyByZW1vdmUgdGhlIGV4YWN0IGBCYXNlbGluZSBiZWhhdmlvcnMgKyBDcm9zcy1nYXRlIHNtb2tl
IHNjZW5hcmlvc2Agcm93IGZyb20gYGluZGV4L1JFQURNRS5tZGA7IHJlbW92ZSBib3RoIEQxMSBs
ZWRnZXIgZW50cmllcyBmb3IgdGhpcyBiYXRjaDsgbGVhdmUgQmF0Y2ggMSwgQmF0Y2ggMiwgRDPi
gJNEMTAgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9gLCBhbmQg
ZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmaXZlLXBhdGggZmVuY2UgdW5jaGFuZ2VkIHwKK3wgRDEx
IHwgW2AwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZGBdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Np
b24tbGVkZ2VyLm1kKSB8IENyb3NzLWdhdGUgc21va2Ugc2NlbmFyaW9zIHwgW2Bjcm9zcy1nYXRl
LXNtb2tlLXNjZW5hcmlvc2BdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2Nyb3Nz
LWdhdGUtc21va2Utc2NlbmFyaW9zKSB8IGxpbmVzIDkz4oCTMTkyICgxMTcxNCBieXRlcykgfCBg
NTg2ZTU0MDJiOTJiNmU0ZTk4YWI3Y2JkZDA0ZTY3MTIxZmYyMDljY2YyM2M3ZjFmZjExNWMxODhi
ZWRiY2E4NWAgfCBbYC4uL2V2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2Nl
bmFyaW9zLm1kYF0oLi4vZXZpZGVuY2UvYmFzZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9rZS1zY2Vu
YXJpb3MubWQpIHwgZXZpZGVuY2UvcmVncmVzc2lvbiB8IGNhbm9uaWNhbCBjb21wb3VuZCBkZXN0
aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9yOyBzZXBhcmF0ZSBleGFjdC1ib2R5
IHByb3ZlbmFuY2UgcHJlc2VydmVkIHwgcGFpcmVkIGZhbWlseSBhdG9taWNpdHkgd2l0aCBgQmFz
ZWxpbmUgYmVoYXZpb3JzIHRoYXQgYWxsIHRyYWNrcyBwcmVzZXJ2ZWA7IGV4YWN0IDExNzE0LWJ5
dGUgc291cmNlIGJvZHkgcHJlc2VydmVkIGJldHdlZW4gdGhlIGBjcm9zcy1nYXRlLXNtb2tlLXNj
ZW5hcmlvc2Agb3duZXIgYm91bmRhcnkgbWFya2VycyB3aXRoIG9yZGVyZWQgc2NlbmFyaW8gc3Vi
aGVhZGluZ3MsIGRldGVybWluaXN0aWMgcGFya2VkLXN1Y2Nlc3NvciBzZXF1ZW5jZSwgYW5kIENv
dmVycyBsaW5lcyBpbnRlcm5hbCB0byB0aGUgcHJlc2VydmVkIGJvZHkgfCByZXN0b3JlIHRoZSBl
eGFjdCAzNjQ0LWJ5dGUgc291cmNlIGJvZHkgYXQgYDA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2Vy
Lm1kI2Jhc2VsaW5lLWJlaGF2aW9ycy10aGF0LWFsbC10cmFja3MtcHJlc2VydmVgIGFuZCB0aGUg
ZXhhY3QgMTE3MTQtYnl0ZSBzb3VyY2UgYm9keSBhdCBgMDUtZGVidWctcmVncmVzc2lvbi1sZWRn
ZXIubWQjY3Jvc3MtZ2F0ZS1zbW9rZS1zY2VuYXJpb3NgIGluIG9yaWdpbmFsIG9yZGVyOyByZW1v
dmUgYGV2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kYDsg
cmVtb3ZlIHRoZSBzb2xlIEJhdGNoLTMgcm93IGZyb20gYGV2aWRlbmNlL1JFQURNRS5tZGA7IHJl
bW92ZSB0aGUgZXhhY3QgYEJhc2VsaW5lIGJlaGF2aW9ycyArIENyb3NzLWdhdGUgc21va2Ugc2Nl
bmFyaW9zYCByb3cgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsgcmVtb3ZlIGJvdGggRDExIGxlZGdl
ciBlbnRyaWVzIGZvciB0aGlzIGJhdGNoOyBsZWF2ZSBCYXRjaCAxLCBCYXRjaCAyLCBEM+KAk0Qx
MCBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYHJldmlldy1jb250cm9sL2AsIGFuZCBldmVy
eSBwYXRoIG91dHNpZGUgdGhlIGZpdmUtcGF0aCBmZW5jZSB1bmNoYW5nZWQgfApkaWZmIC0tZ2l0
IGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2V2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2
aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFj
dG9yL2V2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kCm5l
dyBmaWxlIG1vZGUgMTAwNjQ0CmluZGV4IDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAw
MDAwMDAwMDAuLmUwNDBlMGM1YzI2NjlkNTBmZjdiYTBmOTFjMWU4YzAxY2MwOTc2ZmEKLS0tIC9k
ZXYvbnVsbAorKysgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZXZpZGVuY2UvYmFz
ZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9rZS1zY2VuYXJpb3MubWQKQEAgLTAsMCArMSwxMzUgQEAK
KyoqS2luZDoqKiBldmlkZW5jZS9yZWdyZXNzaW9uCisqKlN0YXR1czoqKiBjYW5vbmljYWwKKyoq
Q2Fub25pY2FsIGZvcjoqKiBleGFjdCBleHRyYWN0ZWQgYEJhc2VsaW5lIGJlaGF2aW9ycyB0aGF0
IGFsbCB0cmFja3MgcHJlc2VydmVgIGFuZCBgQ3Jvc3MtZ2F0ZSBzbW9rZSBzY2VuYXJpb3NgIGJv
ZGllcworKipBdXRob3JpdHkgc2NvcGU6KiogZG9jdW1lbnRhdGlvbiBkZWNvbXBvc2l0aW9uIG9u
bHk7IGV4YWN0IGV4dHJhY3RlZCBldmlkZW5jZS9yZWdyZXNzaW9uIHNvdXJjZSBib2RpZXMgb25s
eQorKipCYXNlbGluZSBiZWhhdmlvcnMgc291cmNlIHByb3ZlbmFuY2U6KiogZXh0cmFjdGVkIGJ5
dGUtZm9yLWJ5dGUgZnJvbSBbYC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2Jhc2Vs
aW5lLWJlaGF2aW9ycy10aGF0LWFsbC10cmFja3MtcHJlc2VydmVgXSguLi8wNS1kZWJ1Zy1yZWdy
ZXNzaW9uLWxlZGdlci5tZCNiYXNlbGluZS1iZWhhdmlvcnMtdGhhdC1hbGwtdHJhY2tzLXByZXNl
cnZlKSwgYmFzZWxpbmUgbGluZXMgODDigJM5MiBpbmNsdXNpdmUgKGAzNjQ0YCBieXRlcyk7IHRo
aXMgZXhhY3Qgc291cmNlIGJvZHkgaXMgcHJlc2VydmVkIGluZGVwZW5kZW50bHkgYmV0d2VlbiBp
dHMgbWF0Y2hpbmcgYm91bmRhcnkgbWFya2VycyBiZWxvdworKipCYXNlbGluZSBiZWhhdmlvcnMg
c291cmNlIFNIQS0yNTY6KiogYDVmOWVjOGQ5ODMyNDAxNTk4NmM4NDYyMDhkYjFjNTY3YjQ5Yzkw
OWJhYTcxNDQyOGFjM2E2NDQ3MDlmZTkxMzJgCisqKkNyb3NzLWdhdGUgc21va2Ugc2NlbmFyaW9z
IHNvdXJjZSBwcm92ZW5hbmNlOioqIGV4dHJhY3RlZCBieXRlLWZvci1ieXRlIGZyb20gW2AuLi8w
NS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNjcm9zcy1nYXRlLXNtb2tlLXNjZW5hcmlvc2Bd
KC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2Nyb3NzLWdhdGUtc21va2Utc2NlbmFy
aW9zKSwgYmFzZWxpbmUgbGluZXMgOTPigJMxOTIgaW5jbHVzaXZlIChgMTE3MTRgIGJ5dGVzKTsg
dGhpcyBleGFjdCBzb3VyY2UgYm9keSBpcyBwcmVzZXJ2ZWQgaW5kZXBlbmRlbnRseSBiZXR3ZWVu
IGl0cyBtYXRjaGluZyBib3VuZGFyeSBtYXJrZXJzIGJlbG93CisqKkNyb3NzLWdhdGUgc21va2Ug
c2NlbmFyaW9zIHNvdXJjZSBTSEEtMjU2OioqIGA1ODZlNTQwMmI5MmI2ZTRlOThhYjdjYmRkMDRl
NjcxMjFmZjIwOWNjZjIzYzdmMWZmMTE1YzE4OGJlZGJjYTg1YAorKipTb3VyY2UgcmVsYXRpb25z
aGlwOioqIHRoZSBwcmVzZXJ2ZWQgYm9kaWVzIGFyZSBkaXN0aW5jdCBleHRyYWN0aW9ucyB3aXRo
IHNlcGFyYXRlIHByb3ZlbmFuY2UgYW5kIGFyZSBub3QgcmVwcmVzZW50ZWQgYXMgb25lIGNvbnRp
Z3VvdXMgc291cmNlIHNwYW4KKyoqU3VwZXJzZWRlczoqKiBjYW5vbmljYWwgb3duZXJzaGlwIG9m
IHRoZSBleHRyYWN0ZWQgc291cmNlIGJvZGllczsgdGhlIHNvdXJjZSBoZWFkaW5ncyByZW1haW4g
Y29tcGF0aWJpbGl0eSBhbmNob3JzCisqKlN1cGVyc2VkZWQgYnk6Kiogbm9uZQorKipSZWNlaXB0
IGJvdW5kYXJ5OioqIFtgLi4vcmV2aWV3LWNvbnRyb2wvUkVBRE1FLm1kYF0oLi4vcmV2aWV3LWNv
bnRyb2wvUkVBRE1FLm1kKSByZW1haW5zIHRoZSBjYW5vbmljYWwgcmVjZWlwdCwgcmV2aWV3LXJl
Y29yZCwgYW5kIGNsb3Nlb3V0IGFydGlmYWN0IHN0b3JlCisqKkF1dGhvcml0eSBib3VuZGFyeToq
KiB0aGlzIGV2aWRlbmNlL3JlZ3Jlc3Npb24gb3duZXIgZG9lcyBub3Qgc2F0aXNmeSBhIGdhdGUs
IGNsYWltIGEgY3VycmVudCBwYXNzLCBhdXRob3JpemUgaW1wbGVtZW50YXRpb24sIHByb21vdGlv
biwgcmVtZWRpYXRpb24sIGRpc3BhdGNoLCBvciBzdWNjZXNzb3IgYXV0aG9yaXR5CisqKlByb2pl
Y3Rpb24gY29uc3VtZXJzOioqIFtgUkVBRE1FLm1kYF0oUkVBRE1FLm1kKSwgW2AuLi9pbmRleC9S
RUFETUUubWRgXSguLi9pbmRleC9SRUFETUUubWQpCisKKyMgQmFzZWxpbmUgYmVoYXZpb3JzIGFu
ZCBzbW9rZSBzY2VuYXJpb3MKKworPCEtLSBleGFjdC1leHRyYWN0ZWQtYm9keTpiYXNlbGluZS1i
ZWhhdmlvcnM6c3RhcnQgLS0+CisjIyBCYXNlbGluZSBiZWhhdmlvcnMgdGhhdCBhbGwgdHJhY2tz
IHByZXNlcnZlCisKK1RoZXNlIHJvd3MgZGVmaW5lIHBlcm1hbmVudCBiZWhhdmlvciBjb250cmFj
dHMsIG5vdCBjdXJyZW50IHBhc3MgY2xhaW1zLiBBbnkgcm93IGV4cGxpY2l0bHkgbWFya2VkCitv
cGVuIHJlbWFpbnMgYSBibG9ja2luZyByZWdyZXNzaW9uIGdhdGUgdW50aWwgaXRzIG5hbWVkIG93
bmVyIGFuZCByZWFsLXBhdGggcHJvb2YgY2xvc2UgaXQuCisKK3wgR2F0ZSBJRCB8IEJhc2VsaW5l
IHwgUmVxdWlyZWQgcHJvb2YgfAorfC0tLXwtLS18LS0tfAorfCAqKlJHLUJBU0UtMDEqKiB8IFB1
YmxpYyB3b3JsZC1zY29wZWQgc3RhcnQg4oaSIHR1cm4vcmVhdHRhY2gg4oaSIHN0b3A7ICoqb3Bl
biwgYmxvY2tpbmcsIGFuZCBub3Qgd2FpdmVkKiogfCBBMS4yIG11c3Qgc3VwcGx5IGV4YWN0IHBh
cmtlZC1zdWNjZXNzb3IgYEF0dGFjaGAvYFJlc3VtZU9uZVR1cm5gIGFwcGxpY2F0aW9uLiBUaGUg
aGVsZCBBMS4zLVAwIGFuZCBBMS4zIHJlY29yZHMgcmVtYWluIHByZXNlcnZlZCBidXQgY2Fubm90
IGJlIGRpc3BhdGNoZWQuIEExLjMtUDEgbXVzdCBhdG9taWNhbGx5IGxhbmQgdGhlIG5ldyBhdXRo
b3JpdHktbWFuYWdlZCBwdWJsaWMgU3RhcnQvdHVybi9yZWF0dGFjaCB0cmFuc3BvcnQgcGF0aCwg
ZXhhY3Qgc3RhcnR1cC9wb3N0LXR1cm4gYWN0b3ItZXZlbnQgcmVzb2x1dGlvbiBvbiB0aGUgcmVh
bCBwdWJsaWMgQ0xJL2hlbHBlci9SRVBMIHBhdGgsIGFuZCBvbmx5IHRoZSBzdHJpY3RseSBtZWNo
YW5pY2FsIHByb2plY3Rpb24gb2YgYWxyZWFkeSBIU0EtYXV0aG9yaXplZCBjb3JyZWxhdGlvbiB0
aHJvdWdoIHRoZSBleGFjdCByZXRhaW5lZC10dXJuIGBvcmNoZXN0cmF0b3Jfd29ybGRfZGlzcGF0
Y2gucnNgIGFjY2VwdGFuY2UtY29udGV4dCBjb25zdHJ1Y3Rpb24vcHJvamVjdGlvbiBzZWFtIGFu
ZCBpbnRvIHRoZSBleGlzdGluZyBCMSBhY2NlcHRhbmNlLWNvbnRleHQgZmllbGQsIHdpdGhvdXQg
bGVnYWN5IGF1dGhvcml0eSB3cml0ZXMsIHJlYWRpbmVzcy9wcm9jZXNzIGluZmVyZW5jZSwgYXV0
by1hdHRhY2ggZHJpZnQsIEIxIHNlbWFudGljIG93bmVyc2hpcCBjaGFuZ2VzLCBvciBDMS9CMy4x
IHNlbWFudGljIGNoYW5nZXMuIEV4YWN0IHNlc3Npb24gYW5kIHdvcmxkIGJpbmRpbmcgc3Vydml2
ZTsgc3RvcCByZWFjaGVzIGR1cmFibGUgdGVybWluYWwgdHJ1dGggZXZlbiB3aGVuIHRyYW5zcG9y
dCBwb3N0dXJlIGNoYW5nZXMuIFN0YWxlIGxpZmVjeWNsZS93b3JsZC1iaW5kaW5nIG92ZXJ3cml0
ZSByZW1haW5zIHJlamVjdGVkLiBQcm92ZSB1bnNhZmUgb3RoZXItcHJpbmNpcGFsIGF1dGhvcml0
eSBmYWlscyBjbG9zZWQgc2VwYXJhdGVseSwgdGhlbiBydW4gdGhlIHBvc2l0aXZlIHNtb2tlIGFn
YWluc3QgYW4gb3duZXItb25seSBwcml2YXRlIGJvb3RzdHJhcCBob21lIHdpdGggZXhhY3QgdHlw
ZS9gMDcwMGAsIGRlc2NyaXB0b3IgaWRlbnRpdHkvcmVwbGFjZW1lbnQgc2FmZXR5LCBxdWFsaWZp
ZWQgYWNjZXNzL2RlZmF1bHQgYE5vRGF0YWAgdW5kZXIgYXV0aG9yaXRhdGl2ZSBtb2RlIGJpdHMs
IGFuZCBubyBlZmZlY3RpdmUgb3RoZXItcHJpbmNpcGFsIGF1dGhvcml0eTsgYE5vRGF0YWAgaXMg
bm90IHBoeXNpY2FsLWFic2VuY2UgcHJvb2YuIFRoaXMgZ2F0ZSBpcyBuZWl0aGVyIHBlcm1hbmVu
dGx5IGV4cGVjdGVkIHRvIGZhaWwgbm9yIHN1Y2Nlc3NmdWwgdW50aWwgdGhhdCByZWFsLXBhdGgg
d2FsbCBpcyBncmVlbi4gfAorfCAqKlJHLUJBU0UtMDIqKiB8IFJFUEwgZmlyc3QtZGlzcGF0Y2gg
YHJ1bl93b3JsZF90YXNrYCBiaW5kaW5nIHJlcGFpciB8IENvcnJlY3QgYmluZGluZyBzdWNjZWVk
czsgc3RhbGUvbWlzbWF0Y2hlZCB3b3JsZCBnZW5lcmF0aW9uIGZhaWxzIGNsb3NlZDsgbm8gZ2Vu
ZXJpYyBiaW5kaW5nIHN5bnRoZXNpcyBvbiB1bnJlbGF0ZWQgc3VyZmFjZXMuIEEgaG9zdCBydW50
aW1lIG1heSBjYXJyeSBleGFjdCBwYXJlbnQtc2Vzc2lvbiB3b3JsZCBiaW5kaW5nIHdoaWxlIGl0
cyBkZXNjcmlwdG9yIGFuZCBwYXJ0aWNpcGFudCBtYW5pZmVzdCByZW1haW4gaG9zdC1zY29wZWQu
IHwKK3wgKipSRy1CQVNFLTAzKiogfCBQYXJrZWQgaG9zdCBvcmRpbmFyeS1jb21tYW5kIGFuZCBj
b250aW51aXR5IHBhcml0eSB8IFVucHJlZml4ZWQgYGxzYC9gcHdkYCByZW1haW4gdXNhYmxlOyBw
b2xpY3ktcmVxdWlyZWQgYGNkIC4uL2AgY2FnZSBkZW5pYWwgcmVtYWluczsgbGF0ZXIgdGFyZ2V0
ZWQgaG9zdCB0dXJuIHJldXNlcyBzZXNzaW9uL1VBQSBjb250aW51aXR5OyBwdWJsaWMgQ0xJIHBh
cml0eSBzdGF5cyBncmVlbi4gfAorfCAqKlJHLUJBU0UtMDQqKiB8IFJldGFpbmVkIHNwYXduL2Zv
cmsvZXhhY3QgY29udGludWUvZXhhY3Qgc3RvcCBwbHVzIGFtYmlndWl0eSBjbG9zZSB8IEV4YWN0
IHNvdXJjZSBhbmQgY2hpbGQgaGFuZGxlcyByb3V0ZSBjb3JyZWN0bHk7IGJhY2tlbmQtb25seSBm
b2xsb3ctdXAgd2l0aCBtdWx0aXBsZSByZXRhaW5lZCB3b3JrZXJzIGZhaWxzIGNsb3NlZDsgc291
cmNlIGRldGFjaGVkIHN0b3AgYW5kIGNoaWxkIGxpdmUtdHJhbnNwb3J0IHN0b3AgcmVtYWluIHZh
bGlkLiB8Cit8ICoqUkctRElGRi0wMSoqIHwgTW9ub3RvbmljIGJyb2FkLXN1aXRlIGRpZmZlcmVu
dGlhbDsgYSBoaXN0b3JpY2FsIGZhaWx1cmUgYmVjb21pbmcgYSBwYXNzIGlzIG5laXRoZXIgYXV0
b21hdGljIHN1Y2Nlc3Mgbm9yIGF1dG9tYXRpYyByZWdyZXNzaW9uIHwgQXBwbHkgdGhlIGV4YWN0
LW5hbWUgYW5kIG5vcm1hbGl6ZWQtc2lnbmF0dXJlIHRyYW5zaXRpb24gZ2F0ZSBpbiBgMDRgLiBQ
cmVzZXJ2ZSBjb21wbGV0ZSBpbnZlbnRvcmllcyBhbmQgYXJ0aWZhY3QgaGFzaGVzOyBwcm92ZSBl
dmVyeSBoaXN0b3JpY2FsIGZhaWx1cmUtdG8tcGFzcyB0cmFuc2l0aW9uIHRocm91Z2ggdGhlIGV4
YWN0IGJhc2VsaW5lIGFuZCBjdXJyZW50IGZ1bGwgcHJvZHVjdGlvbiBkaXNwYXRjaGVyOyBpbnNw
ZWN0IHRlc3QgYW5kIGFzc2VydGlvbiBkaWZmczsgbWFwIHRoZSBjYXVzYWwgY2hhbmdlIHRvIHRo
ZSBvd25pbmcgc2xpY2UgYW5kIHN5bWJvbDsgYW5kIHByb3ZlIG5vIGRpcmVjdCByZXNvbHZlciwg
dHJhbnNwb3J0LCByZWNlaXB0LCBzdXBlcnZpc29yLCBoZWxwZXIsIGJ5cGFzcywgd2Vha2VuZWQg
ZW5mb3JjZW1lbnQsIHJlbmFtZWQvcmVwbGFjZWQvaWdub3JlZCB0ZXN0LCB3ZWFrZW5lZCBhc3Nl
cnRpb24sIHJlbW92ZWQgYmVoYXZpb3IsIG9yIHVucmVsYXRlZCBjYXBhYmlsaXR5IGxvc3MuIFRo
ZSBleGFjdCBmb3VydGVlbi10ZXN0IEIxL0IyLjEgaW52ZW50b3J5IGFib3ZlIHJldGFpbnMgZXZl
cnkgaGlzdG9yaWNhbCBuYW1lIGFuZCBvcmlnaW5hbCBwcm9kdWN0aW9uLWxldmVsIGFzc2VydGlv
bi4gQSBsb3dlci1sZXZlbCBzdWJzdGl0dXRlIGlzIGBSZWdyZXNzaW9uTWFza2VkYDsgYW55IG90
aGVyd2lzZSB1bmNlcnRhaW4gdHJhbnNpdGlvbiBpcyBgQmFzZWxpbmVSZWdyZXNzaW9uQW1iaWd1
b3VzYC4gRWl0aGVyIGNsYXNzaWZpY2F0aW9uIGtlZXBzIHRoZSBvd25pbmcgY2xvc2VvdXQgb3Bl
biwgYW5kIG5laXRoZXIgY2FuIGNvdW50IGFzIGBGYWlsVG9QYXNzYCBwcm9vZi4gSW5kZXBlbmRl
bnQgcmV2aWV3IGlzIG1hbmRhdG9yeS4gfAorCis8IS0tIGV4YWN0LWV4dHJhY3RlZC1ib2R5OmJh
c2VsaW5lLWJlaGF2aW9yczplbmQgLS0+CisKKzwhLS0gZXhhY3QtZXh0cmFjdGVkLWJvZHk6Y3Jv
c3MtZ2F0ZS1zbW9rZS1zY2VuYXJpb3M6c3RhcnQgLS0+CisjIyBDcm9zcy1nYXRlIHNtb2tlIHNj
ZW5hcmlvcworCisjIyMgUzEg4oCUIEF1dGhvcml0eSBzdXJ2aXZlcyBlcGlzb2RlIGxvc3MKKwor
MS4gU3RhcnQgYSB3b3JsZC1zY29wZWQgaG9zdCBzZXNzaW9uIGFuZCByZXRhaW4gYSB3b3JrZXIu
CisyLiBLaWxsIHRoZSBjdXJyZW50IGhlbHBlci9lcGlzb2RlIGFuZCByZW1vdmUgaXRzIHByaXZh
dGUgc29ja2V0LgorMy4gUmVhZCBkdXJhYmxlIGF1dGhvcml0eSwgd29ya2VyIG1hbmlmZXN0LCBh
bmQgYW55IGFjdGl2ZSByZWNlaXB0LgorNC4gUmVhdHRhY2ggb3IgcGVyZm9ybSBzYW5jdGlvbmVk
IGR1cmFibGUgY2xvc2VvdXQuCis1LiBQcm92ZSBubyBzdGFsZSBlcGlzb2RlIHdyaXRlIHJlZ3Jl
c3NlZCB0aGUgYXV0aG9yaXR5IHJldmlzaW9uLgorCitDb3ZlcnM6IGBSRy1BVVRILTAxYCwgYFJH
LUFVVEgtMDJgLCBgUkctQ0xPU0UtMDFgLCBgUkctQkFTRS0wMWAuCisKKyMjIyBTMUEg4oCUIFJl
dmlzaW9uLWJvdW5kIGhvc3QgdHJhbnNpdGlvbnMgc3Vydml2ZSBoZWxwZXIgbG9zcyBhbmQgcmVw
bGF5CisKKzEuIEJlZm9yZSBhbnkgcHJvZHVjdGlvbiBTdGFydCwgY2xhc3NpZnkgZ2VudWluZWx5
IGFic2VudCwgaW50ZXJydXB0ZWQtaW5pdGlhbGl6YXRpb24sIHZhbGlkLWV4aXN0aW5nLCB1bnN1
cHBvcnRlZC1wcmUtQTEsIGFuZCBjb3JydXB0L3Vuc3VwcG9ydGVkIHN0b3Jlcy4gUmVqZWN0IG1h
bGZvcm1lZCwgcGFydGlhbCwgdW5yZWFkYWJsZSwgc3ltbGlua2VkLCBwZXJtaXNzaW9uLWludmFs
aWQsIGFuZCB1bnN1cHBvcnRlZCByb290cyB3aXRob3V0IGdlbmVyYXRpbmcgYSByZXBsYWNlbWVu
dCBzdG9yZSBJRC4gUGVyc2lzdCB0aGUgc2FtZSBub3JtYWxpemVkIHBoeXNpY2FsIGJvb3RzdHJh
cCBob21lIGluIHRoZSBpbml0IG1hcmtlciwgZ3JlZW5maWVsZCBjZXJ0aWZpY2F0ZSwgYW5kIHJv
b3Q7IHBhc3MgaXRzIG9wZW5lZCBoYW5kbGUgdGhyb3VnaCB0aGUgYm91bmRlZCBTdGF0ZVN0b3Jl
L2NvbmZpZy9wb2xpY3kvaW52ZW50b3J5IGVudHJ5IHBvaW50czsgY2hhbmdlL3JlbW92ZSBhbWJp
ZW50IGhvbWUgdmFsdWVzIGJldHdlZW4gZWFjaCBjYWxsIGFuZCBwcm92ZSBubyByZXJlYWQ7IHJl
amVjdCBhIGNvcGllZC9yZWJvdW5kIGF1dGhvcml0eSB0cmVlOyBhbmQgY3Jhc2gvcmVzdGFydCBh
ZnRlciBtYXJrZXIsIGtleSwgcm9vdCwgYW5kIG1hcmtlci1jbGVhbnVwIHN0ZXBzIHdpdGhvdXQg
Y2hhbmdpbmcgc3RvcmUvaG9tZSBpZGVudGl0eS4KKzIuIENvbXBsZXRlbHkgYW5kIHNhZmVseSBl
bnVtZXJhdGUgYm90aCBwcmUtQTEgc2Vzc2lvbi9wYXJ0aWNpcGFudCBhdXRob3JpdHkgY29sbGVj
dGlvbnMgYmVmb3JlIGNlcnRpZmljYXRlIHB1YmxpY2F0aW9uIGFuZCBldmVyeSBzZW1hbnRpYyB0
cmFuc2FjdGlvbi4gUHJvdmUgY29tcG9uZW50LWJ5LWNvbXBvbmVudCBuby1mb2xsb3cgdHJhdmVy
c2FsIGZyb20gdGhlIHRydXN0ZWQgaG9tZSByZWplY3RzIGFuY2VzdG9yIHN5bWxpbmtzLCB1bnNh
ZmUgb3duZXIvbW9kZS9BQ0wsIGNyb3NzLWRldmljZSByZWJpbmRpbmcsIGFuZCBzY2FuLXRvLXB1
YmxpY2F0aW9uIGlkZW50aXR5IHJlcGxhY2VtZW50LiBQcm92ZSB2YWxpZGF0ZWQgZW1wdHkgY29s
bGVjdGlvbnMgcGVybWl0IGdyZWVuZmllbGQgY2VydGlmaWNhdGlvbiwgd2hpbGUgYW55IGFydGlm
YWN0IGlzIGBVbnN1cHBvcnRlZExlZ2FjeVN0YXRlYCB3aXRob3V0IHBhcnNpbmcvY29udmVyc2lv
biBhbmQgdW5yZWFkYWJsZSwgcGVybWlzc2lvbi1pbnZhbGlkLCBzeW1saW5rZWQsIG9yIHBhcnRp
YWxseSBlbnVtZXJhYmxlIGNvbGxlY3Rpb25zIGZhaWwgY2xvc2VkLiBFdmVyeSBwcmUtQTEgcmVh
ZC1kZWNpZGUtd3JpdGUtcmVtb3ZlLXJlbmFtZSB0cmFuc2FjdGlvbiByZXRhaW5zIHRoZSBvcGVu
ZWQgdHJ1c3RlZCBwaHlzaWNhbCByb290LCBkZXNjZW5kYW50IGhhbmRsZXMsIGV4YWN0IGlkZW50
aXR5LCBhY3RpdmF0aW9uIG9ic2VydmF0aW9uLCBhbmQgcm9vdCBsb2NrIHRocm91Z2ggZmluYWwg
ZmlsZS9kaXJlY3RvcnkgYGZzeW5jYDsgaXQgbmV2ZXIgcmVyZWFkcyBgU1VCU1RSQVRFX0hPTUVg
LCB1c2VzIGFtYmllbnQgQ1dELCByZWNvbnN0cnVjdHMgYW4gYWJzb2x1dGUgZGVzY2VuZGFudCBw
YXRoLCBvciByZWxlYXNlcyB0aGUgbG9jayBlYXJseS4gSW4gYSByZWFsIHN1YnByb2Nlc3MsIHJl
bmFtZS9yZXBsYWNlL3JlYmluZCB0aGUgbGV4aWNhbCByb290IGFmdGVyIGxvY2sgYWNxdWlzaXRp
b24gYW5kIHByb3ZlIG9yaWdpbmFsL3JlcGxhY2VtZW50IGlkZW50aXRpZXMgZXhwbGljaXRseTog
dGhlIHJlcGxhY2VtZW50IHJlY2VpdmVzIHplcm8gd3JpdGVzLCB0ZW1wcywgcmVtb3ZhbHMsIG9y
IGBmc3luY2AtZGVwZW5kZW50IHB1YmxpY2F0aW9uLCBhbmQgYSBsb2NrIG9uIHRoZSBmb3JtZXIg
cm9vdCBpcyBuZXZlciByZXByZXNlbnRlZCBhcyBwcm90ZWN0aW9uIGZvciB0aGUgcmVwbGFjZW1l
bnQuIFNlcmlhbGl6ZSBldmVyeSBpbi1yZXBvc2l0b3J5IHByZS1BMSB3cml0ZXIgb24gdGhlIHNh
bWUgcm9vdCBsb2NrIGJlZm9yZSBhY3RpdmF0aW9uOyBhZnRlciBpbml0LW1hcmtlciBvciByb290
IHB1YmxpY2F0aW9uIGl0IHJlamVjdHMgYmVmb3JlIG11dGF0aW9uLiBObyBtaXNzaW5nIHJvdyBv
ciBjb21wYXRpYmlsaXR5IHByb2plY3Rpb24gc2F0aXNmaWVzIGFic2VuY2UuCiszLiBFeGVyY2lz
ZSBldmVyeSBBMSBvYmplY3Qga2luZCBhZ2FpbnN0IGl0cyBleGFjdCBWMSBzY2hlbWEsIGZpbGUg
Ynl0ZXMsIHR5cGVkIHBhdGgsIGFuZCBjb21taXRtZW50IHZhcmlhbnQvZG9tYWluLiBSZWplY3Qg
Y3Jvc3Mta2luZC9zY2hlbWEvZG9tYWluIHN1YnN0aXR1dGlvbjsgcmVxdWlyZSBgcnVuX3ByZXNl
bnQ9MHgwMWAgYW5kIHRoZSBleGFjdCBub24tZW1wdHkgcGFyZW50LWludGVudCBydW4gSUQgZm9y
IGV2ZXJ5IHNlbnNpdGl2ZSBkb21haW4sIHdpdGggZ29sZGVuIG5lZ2F0aXZlIGZpeHR1cmVzIGZv
ciBhYnNlbnQvZW1wdHkvZGlmZmVyZW50IHJ1bnMuIENyYXNoIGJlZm9yZS9hZnRlciBlYWNoIHRl
bXAtZmlsZSBmc3luYyBhbmQgcmVuYW1lOiByZWNvZ25pemVkIG9wZXJhdGlvbi1ib3VuZCB0ZW1w
cyBhcmUgcmVtb3ZlZCBhbmQgbmV2ZXIgcHJvbW90ZWQsIHdoaWxlIHVuc2FmZS91bnJlY29nbml6
ZWQgdGVtcHMgZmFpbCBjbG9zZWQuIEZvciBwZW5kaW5nIGluaXRpYWxpemF0aW9uLCB2YWxpZGF0
ZSBhIHJldXNlZCBrZXkgb25seSBhZ2FpbnN0IHRoZSBpbml0IG1hcmtlciBhbmQgcmUtZnN5bmMg
dGhlIGZpbGUgcGx1cyBga2V5cy9gIGJlZm9yZSByb290IHB1YmxpY2F0aW9uLiBDcmFzaCBiZXR3
ZWVuIGZpcnN0LXVzZSBraW5kL3ZlcnNpb24gZGlyZWN0b3J5IGNyZWF0aW9uIGFuZCBldmVyeSBk
aXJlY3RvcnkvcGFyZW50IGZzeW5jOyBvbmx5IHNhZmUgZW1wdHkgY2xvc2VkIGRpcmVjdG9yaWVz
IHN1cnZpdmUgd2l0aG91dCBvYmplY3QgYXV0aG9yaXR5LiBDcmFzaCBhZnRlciBvYmplY3QgcmVu
YW1lIGJ1dCBiZWZvcmUgcm9vdCBjb21taXQ7IHJlc3RhcnQgbXVzdCBjbGFzc2lmeSB0aGUgc2Fm
ZSB1bmluZGV4ZWQgZmlsZSBhcyBhIHJldGFpbmVkIG5vbi1hdXRob3JpdGF0aXZlIG9ycGhhbiwg
YW5kIG9ubHkgYSByZXRyeSB3aXRoIHRoZSBjb21wbGV0ZSByZWYsIGV4YWN0IHBhcmVudCBpbnRl
bnQvcnVuL3N0b3JlIGNvbnRleHQsIGFuZCB2ZXJpZmllZCBieXRlcyBtYXkgYWRvcHQgaXQuIEV4
YWN0IHJldHJ5IG11c3QgdHJlYXQgZXZlcnkgcmVzZXJ2YXRpb24sIHRvbWJzdG9uZSwgdHJhbnNp
dGlvbiBpbnRlbnQsIGlzc3Vlci1yZXF1ZXN0IGluZGV4LCBhcHBsaWNhdGlvbi1qb3VybmFsLCBh
bmQgb2JqZWN0LWluZGV4IGVudHJ5IGFzIHNlbWFudGljIGF1dGhvcml0eSBvY2N1cGFuY3kgYW5k
IG1heSBqb2luIG9ubHkgY29tcGxldGUgbWF0Y2hpbmcgY29tbWl0dGVkIHN0YXRlLiBSb3RhdGUg
YW5kIHJldGlyZSBITUFDIGtleXMgYWNyb3NzIGNyYXNoZXMgYmVmb3JlL2FmdGVyIGtleSBwdWJs
aWNhdGlvbiwgcm9vdCBzZWxlY3Rpb24sIHJldGlyZW1lbnQgcm9vdCBjb21taXQsIGFuZCBrZXkg
ZGVsZXRpb247IGFmdGVyIHJlY29uY2lsaWF0aW9uIGFuZCBpbW1lZGlhdGVseSBiZWZvcmUgcm9v
dCBwdWJsaWNhdGlvbiwgdmFsaWRhdGUgdGhlIGNvbXBsZXRlIGNhbmRpZGF0ZSBhZ2FpbnN0IHRo
ZSBleGFjdCBsb2NrZWQgcm9vdC9yZXZpc2lvbiwgaG9tZS9zdG9yZSBpZGVudGl0eSwga2V5IHJl
Z2lzdHJ5L2ZpbGVzLCBhY3RpdmUvdmVyaWZpY2F0aW9uL3JldGlyZWQgY29uc3RyYWludHMsIHJl
YWNoYWJsZSBvYmplY3RzLCBzZW1hbnRpYyBtYXBzL2luZGV4ZXMvam91cm5hbCwgZW1wdHkgcHJl
LUExIGNvbGxlY3Rpb25zLCBhbmQgdW5jaGFuZ2VkIHBoeXNpY2FsIHJvb3QuIE9sZCByZWZlcmVu
Y2VkIGtleXMgcmVtYWluIHZlcmlmaWNhdGlvbi1vbmx5LCBrZXkgbG9zcyBvciBwb3N0LXJlY29u
Y2lsaWF0aW9uIGludmFsaWRpdHkgZmFpbHMgYmVmb3JlIHB1YmxpY2F0aW9uIHdpdGggemVybyBz
ZW1hbnRpYyBtdXRhdGlvbiwgYW5kIGNyYXNoL3Jlc3RhcnQgcGx1cyBleGFjdCByZXRyeSBjb252
ZXJnZS4KKzQuIElzc3VlIGEgcHVibGljIEExLjIgYFN0YXJ0YCBpbnRlbnQgYWdhaW5zdCBncmVl
bmZpZWxkLWNlcnRpZmljYXRlLXByb3ZlbiBgRXhwZWN0ZWRBYnNlbnRgLCB0aGVuIHRlcm1pbmF0
ZSBkZWxpdmVyeSBvbmNlIHdoaWxlIGBJc3N1ZWRgLCBvbmNlIGFmdGVyIHBsYW4gbG9hZC9yZW1v
dmFsIGJ1dCBiZWZvcmUgY2xhaW0sIG9uY2Ugd2hpbGUgYENsYWltZWRgLCBvbmNlIGFmdGVyIGlu
aXRpYWwgYXV0aG9yaXR5IGJpcnRoLCBhbmQgb25jZSBhZnRlciBpbnB1dCBhY2NlcHRhbmNlIGJ1
dCBiZWZvcmUgc3VjY2VzcyBpcyBvYnNlcnZlZC4KKzUuIFJlc3RhcnQgYW5kIHJldHJ5IHRoZSBl
eGFjdCBpbnRlbnQ7IHByb3ZlIG1pc3NpbmcgcGxhbiB0cmFuc3BvcnQgaXMgcmVwcm9kdWNlZCBm
cm9tIHRoZSByZXRhaW5lZCBwYXlsb2FkLCBwcmUtYXBwbGljYXRpb24gY2FzZXMgc2FmZWx5IHJl
c3VtZS9yZWNvbmNpbGUsIGFuZCBjb21taXR0ZWQvYWNjZXB0ZWQgY2FzZXMgam9pbiBvbmUgaW1t
dXRhYmxlIHJlc3VsdCB3aXRob3V0IGFub3RoZXIgbmFtZXNwYWNlLCBhdXRob3JpdHkgcmV2aXNp
b24sIHBhcnRpY2lwYW50IGFsbG9jYXRpb24sIG9yIGlucHV0IGRlbGl2ZXJ5LiBQcm92ZSBwYXls
b2FkIGNsZWFudXAgb2NjdXJzIG9ubHkgYWZ0ZXIgZXhhY3QgdGVybWluYWwgaGFuZG9mZiwgd2l0
aCBjcmFzaGVzL2ZhaWx1cmVzIGJlZm9yZSBhbmQgYWZ0ZXIgdGhlIGBSZWxlYXNlRWxpZ2libGVg
IHJvb3QgY29tbWl0LCB0cmFuc3BvcnQgZGVsZXRpb24sIG9iamVjdC1kaXJlY3RvcnkgZnN5bmMs
IGFuZCBgUmVsZWFzZWRgIHJvb3QgY29tbWl0IGZvciBib3RoIHByZXNlbnQgYW5kIGFscmVhZHkt
YWJzZW50IHRyYW5zcG9ydCBieXRlcy4KKzYuIFJlcGVhdCBwdWJsaWMgQ0xJIGFuZCByZWFsIFJF
UEwgYEF0dGFjaGAgYW5kIGBSZXN1bWVPbmVUdXJuYCBhZ2FpbnN0IGV4YWN0IHJldmlzaW9ucywg
aW5jbHVkaW5nIGJvdW5kZWQgYXV0by1hdHRhY2ggcGxhbiBwcm9kdWN0aW9uLCBhbmQgcHJvdmUg
dGhlIHNhbWUgYm9vdHN0cmFwLWhvbWUvd29ya3NwYWNlL3dvcmxkL2Rlc2NyaXB0b3IvYXR0YWNo
L3Jlc3VtZS9wb2xpY3kgY29tbWl0bWVudHMgcmVhY2ggYXBwbGljYXRpb24uCis3LiBSZWplY3Qg
c3RhbGUgcmV2aXNpb24sIHN1YnN0aXR1dGVkIHBsYW4sIGNyb3NzLWtpbmQgcmVmLCBwYXlsb2Fk
L2hhc2ggbWlzbWF0Y2gsIHdyb25nIGNhbGxlci9zb3VyY2UvdGFyZ2V0L2xpbmVhZ2UsIHdyb25n
IGJvb3RzdHJhcCBob21lL3dvcmtzcGFjZS93b3JsZCBnZW5lcmF0aW9uLCBtaXNzaW5nL2NvcGll
ZCBncmVlbmZpZWxkIGNlcnRpZmljYXRlLCBjb21wYXRpYmlsaXR5LWRlcml2ZWQgYWJzZW5jZSwg
cHJlLUExIGFydGlmYWN0cywgbWlzc2luZyBrZXksIGV4cGlyZWQgaW50ZW50LCBzdXBlcnNlZGVk
IGNsYWltLCBhbmQgY29uZmxpY3RpbmcgcmVwbGF5IGJlZm9yZSBhdXRob3JpdHkgbXV0YXRpb24u
Cis4LiBSdW4gcHVibGljIHdvcmxkLXNjb3BlZCBzdGFydCDihpIgcmVhdHRhY2gg4oaSIHN0b3Ag
YW5kIHByb3ZlIGV4YWN0IHNlc3Npb24vd29ybGQgYmluZGluZyBwbHVzIGR1cmFibGUgdGVybWlu
YWwgdHJ1dGggYXJlIHVuY2hhbmdlZC4KKworVGhlIGRldGVybWluaXN0aWMgcGFya2VkLXN1Y2Nl
c3NvciBwb3J0aW9uIG9mIHRoaXMgc2NlbmFyaW8gaXM6CisKKzEuIFBlcnNpc3QgY3VycmVudCBg
QWN0aXZlYCAvIGBQYXJrZWRSZXN1bWFibGVgIGR1cmFibGUgYXV0aG9yaXR5LgorMi4gUmVtb3Zl
IGFsbCBlcGlzb2RlLWxvY2FsIFBJRCwgaGFuZGxlLCBoZWxwZXItcmVhZGluZXNzLCBhbmQgcHJv
bXB0LXN0cmVhbSBzdGF0ZS4KKzMuIEludm9rZSB0aGUgcHVibGljIHByb21wdC1iZWFyaW5nIHR1
cm4gb3IgZXhwbGljaXQgcmVhdHRhY2ggcGF0aC4KKzQuIFJlc29sdmUgdGhlIGV4YWN0IGN1cnJl
bnQgYXV0aG9yaXR5IHJldmlzaW9uL2hhc2ggYW5kIGF1dGhvcml0YXRpdmUgbGluZWFnZS4KKzUu
IElzc3VlIGFuZCBhcHBseSBgUmVzdW1lT25lVHVybmAgZm9yIHRoZSBwdWJsaWMgdHVybiBvciBg
QXR0YWNoYCBmb3IgZXhwbGljaXQgcmVhdHRhY2guCis2LiBQcmVzZXJ2ZSB0aGUgZXhhY3Qgc2Vz
c2lvbiBpZGVudGl0eSBhbmQgd29ybGQgYmluZGluZy4KKzcuIFJldmlzaW9uLWF1dGhvcml6ZSBl
eGFjdGx5IG9uZSBzdWNjZXNzb3IgcGFydGljaXBhbnQgYW5kIHJldGFpbiBpbW11dGFibGUgaW5w
dXQgaGFuZG9mZiB3aGVuCisgICBwcmVzZW50LgorOC4gTGF1bmNoIHRoZSBoZWxwZXIvUkVQTCBl
eGVjdXRpb24gZXBpc29kZSBvbmx5IGFmdGVyIGR1cmFibGUgYXBwbGljYXRpb24uCis5LiBSZXRy
eSB0aGUgZXhhY3QgcmVxdWVzdCBhbmQgam9pbiB0aGUgY29tbWl0dGVkIGludGVudC9hcHBsaWNh
dGlvbiB3aXRob3V0IGEgc2Vjb25kIHN1Y2Nlc3Nvci4KKzEwLiBTdG9wIHRoZSBzYW1lIHNlc3Np
b24gYW5kIHJlYWNoIGR1cmFibGUgdGVybWluYWwgdHJ1dGguCisKK0NvdmVyczogYFJHLUFVVEgt
MDFgLCBgUkctQVVUSC0wMmAsIGBSRy1BVVRILTAzYCwgYFJHLUJBU0UtMDFgLCBgUkctQkFTRS0w
MmAuCisKKyMjIyBTMiDigJQgUmVjZWlwdCwgc3VwZXJ2aXNvciwgb2JsaWdhdGlvbiwgY2FuY2Vs
CisKKzEuIEFjY2VwdCBvbmUgZXBoZW1lcmFsIHRhc2sgZnJvbSBpdHMgZXhhY3QgcHJlLXRlcm1p
bmFsIEIwIGFja25vd2xlZGdlbWVudCwgZHVyYWJseSBjbGFpbSB0aGUKKyAgIHN0cmVhbSBpbiBC
Mi4xIGJlZm9yZSBhbnkgc3Vic2VxdWVudCBmcmFtZSwgYW5kIHByb3ZlIHRoZSBwcm9kdWN0aW9u
IHBhdGggbmV2ZXIgYXR0ZW1wdHMgbGVnYWN5CisgICBhY3RpdmUtdGFzayByZWdpc3RyYXRpb24u
IEJsb2NraW5nIGluc3BlY3Qvd2FpdCBhbmQgbmFycm93bHkgYWRhcHRlZCBjYW5jZWwgY29uc3Vt
ZSBleGFjdAorICAgcmVjZWlwdC9zdXBlcnZpc29yIHRydXRoLCBhbmQgZm9yZWdyb3VuZCBkcm9w
IGRvZXMgbm90IGVyYXNlIHRoZSBhY2NlcHRlZCBvciBzdXBlcnZpc2VkIHdvcmsuCisyLiBDb250
aW51ZSBhbiBleGFjdCByZXRhaW5lZCB3b3JrZXIsIHBlcnNpc3QgaXRzIEIxIGFjY2VwdGVkIGFj
dGl2ZS1ydW4gaWRlbnRpdHksIGhhbmQgdGhlIHNhbWUgQjAKKyAgIHN0cmVhbSB0byBCMi4xIHdp
dGggbm8gb2JzZXJ2YXRpb24gZ2FwLCBhbmQgcHJvdmUgdGhlIGZvcmVncm91bmQgbWF5IHJlbWFp
biBhIGJsb2NraW5nIHdhaXRlcgorICAgd2l0aG91dCByZXRhaW5pbmcgb2JzZXJ2YXRpb24gb3du
ZXJzaGlwLgorMy4gTm9ybWFsaXplIG9uZSBzdXBwb3J0ZWQgcHJvdmlkZXIgYXR0ZW50aW9uIHNo
YXBlIGJlZm9yZSBgQWdlbnRFdmVudGAgY29uc3RydWN0aW9uLCBlbWl0IG9uZQorICAgQjMuMSBl
dmVudCB3aXRoIGV4YWN0IHRhcmdldC9ydW4vdGhyZWFkL2NsYXNzL2F0dGVudGlvbi9jYXVzYXRp
b24gaWRlbnRpdHksIGFuZCBwcm92ZSBCMi4xCisgICBqb3VybmFscyBpdCBkdXJhYmx5IGFuZCBD
MSBtYXRlcmlhbGl6ZXMgb25lIG9ibGlnYXRpb24gYmVmb3JlIHRoZSB0ZXJtaW5hbCBldmVudC4g
UHJvdmUgYW4KKyAgIGFtYmlndW91cyBvciBtYWxmb3JtZWQgYXR0ZW50aW9uLWRyaXZpbmcgcHJv
dmlkZXIgc2hhcGUgZmFpbHMgYmVmb3JlIGVtaXNzaW9uIHJhdGhlciB0aGFuCisgICBiZWNvbWlu
ZyBwcm9ncmVzcyBvciBuby1hdHRlbnRpb24sIGFuZCBwcm92ZSBob3N0IEpTT04tcG9pbnRlciBw
YXJzaW5nIGNhbm5vdCByZXBhaXIgaXQuIEZvcgorICAgZXZlcnkgcG9zdC1hY2sgcmV0YWluZWQg
YEV2ZW50YCBmcmFtZSwgcHJvdmUgZWl0aGVyIHRoZSB0eXBlZCBtZW1iZXIgcmVhY2hlcyB0aGUg
am91cm5hbC9jdXQgb3IKKyAgIHRoZSBzdHJlYW0gYW5kIENvbXBsZXRlIGN1dCBmYWlsIGNsb3Nl
ZDsgbm8gZXZlbnQgaXMgc2lsZW50bHkgZHJvcHBlZCBvciBsZWZ0IHVudHlwZWQuCis0LiBEcm9w
IGNhbGxlcnMgYW5kIHJlc3RhcnQgdGhlIHN1cGVydmlzb3IgZHVyaW5nIGJvdGggYWNjZXB0ZWQg
ZmFtaWxpZXMsIHRoZW4gcmVwbGF5IGR1cGxpY2F0ZQorICAgZnJhbWVzL2V2ZW50cyBhbmQgcHJv
dmUgdGhlIGNsYWltLCBqb3VybmFsLCByZWNlaXB0LCBvYmxpZ2F0aW9uLCBsZWRnZXIgcmV2aXNp
b24sIHdhdGVybWFyaywgYW5kCisgICB0ZXJtaW5hbCBzdGF0ZSByZW1haW4gaWRlbXBvdGVudC4g
TWlzc2luZyBleGFjdCB0ZXJtaW5hbCB0cnV0aCByZW1haW5zIGludGVycnVwdGVkL25vbnRlcm1p
bmFsLgorNS4gUXVlcnkgQzEgYmVmb3JlIGFuZCBhZnRlciB0aGUgZXhhY3QgdGVybWluYWwgY3V0
OyBwcm92ZSBQZW5kaW5nIGZpcnN0LCB0aGVuIENvbXBsZXRlIGZvciBib3RoCisgICBlbXB0eS9u
by1hdHRlbnRpb24gYW5kIG5vbi1lbXB0eS9hdHRlbnRpb24gZGlzcG9zaXRpb25zIHdpdGhvdXQg
Y29uc3VsdGluZyBwcm9qZWN0aW9ucy4gVGhlCisgICBDb21wbGV0ZSBzbmFwc2hvdCdzIG9yZGVy
ZWQgZXZlbnQgdmVjdG9yIG11c3Qgam9pbiBleGhhdXN0aXZlbHkgdG8gZXZlcnkgcmV0YWluZWQg
QjIuMSBgRXZlbnRgCisgICByZWYgaW4gc2NvcGUsIGluY2x1ZGluZyB0eXBlZCBub24tYXR0ZW50
aW9uIGV2ZW50cy4KKzYuIEFmdGVyIEIyLjIvQjMuMiwgcmVjZWl2ZSB0aGUgYWNjZXB0ZWQgcmVj
ZWlwdCBiZWZvcmUgdGVybWluYWwgZXhpdCBhbmQgY2FuY2VsIGJ5CisgICBgYWN0aXZlX3J1bl9p
ZGAgaW4gdGhlIHNhbWUgaG9zdCB0dXJuLgorCitDb3ZlcnM6IGBSRy1FVkVOVC0wMWAsIGBSRy1S
RUNFSVBULTAyYCwgYFJHLVJFQ0VJUFQtMDNgLCBgUkctU1VQLTAxYCwgYFJHLVNVUC0wMmAsCitg
UkctTVNHLTAxYCwgYFJHLUNBTkNFTC0wMWAsIGBSRy1PQkwtMDFgLCBgUkctT0JMLTAyYCwgYFJH
LU9CUy0wMWAuCisKKyMjIyBTMyDigJQgV29ybGQtVUFBIG1lZGlhdGlvbiB1bmRlciBuYXJyb3dl
ZCBwb2xpY3kKKworMS4gU3Bhd24vY29udGludWUgd29ybGQgQ29kZXggd2l0aCB3cml0ZSBuYXJy
b3dlZCB0byBgc3JjL3BhcnNlci5yc2AuCisyLiBQcm92ZSBzaGVsbCBhbmQgcHJvdmlkZXItbmF0
aXZlIGVkaXQgY2FuIG1vZGlmeSB0aGF0IGZpbGUuCiszLiBQcm92ZSBzaGVsbCwgZGlyZWN0IGVk
aXQsIGFuZCB3cml0ZS1jYXBhYmxlIHRvb2wgY2Fubm90IG1vZGlmeSBgUkVBRE1FLm1kYCBvciBl
c2NhcGUgcm9vdC4KKzQuIFJlcGVhdCB3aXRoIGJyb2tlciB1bmF2YWlsYWJsZSBhbmQgcmVxdWly
ZSBmYWlsLWNsb3NlZCBiZWhhdmlvci4KKzUuIEpvaW4gZWFjaCBvcGVyYXRpb24gdG8gdGhlIHJl
Y2VpcHQncyBpbW11dGFibGUgcG9saWN5IGhhc2guCisKK0NvdmVyczogYFJHLVVBQS0wMWAsIGBS
Ry1VQUEtMDJgLCBgUkctVUFBLTAzYCwgYFJHLVBPTElDWS0wMWAsIGBSRy1QT0xJQ1ktMDJgLgor
CisjIyMgUzQg4oCUIEhvc3QgdmlzaWJpbGl0eSBhbmQgZmFpbHVyZSB0cnV0aAorCisxLiBSdW4g
ZXF1aXZhbGVudCB3cml0ZXMgdW5kZXIgaG9zdC12aXNpYmxlIGFuZCBmdWxsLWlzb2xhdGlvbiBw
b2xpY2llcy4KKzIuIFZlcmlmeSBleGFjdCB2aXNpYmlsaXR5L3JlY29uY2lsaWF0aW9uIHNlbWFu
dGljcy4KKzMuIEZvcmNlIGEgbm9uLXplcm8gVUFBIHR1cm4gYWZ0ZXIgYSBwZXJtaXR0ZWQgd3Jp
dGUuCis0LiBQcm92ZSBkdXJhYmxlIGZhaWxlZCBhY3RpdmUtcnVuIHN0YXRlLCByZXRhaW5lZCBh
dXRob3JpdHkgYmVoYXZpb3IsIGRpYWdub3N0aWNzLCBhbmQgZmlsZSB2aXNpYmlsaXR5IG91dGNv
bWUgaW5kZXBlbmRlbnRseS4KKworQ292ZXJzOiBgUkctU1lOQy0wMWAsIGBSRy1XT1JLRVItRVhJ
VC0wMWAsIGBSRy1PQlMtMDFgLgorCisjIyMgUzUg4oCUIEV4aXN0aW5nIGdhdGV3YXkgY2Fycmll
ciBwcmVzZXJ2YXRpb24gYW5kIGRpcmVjdCBDb2RleCBhZG9wdGlvbgorCisxLiBSdW4gdGhlIGV4
aXN0aW5nIGxhdW5jaGVyL2NvbnN1bWVyIHJlZ3Jlc3Npb24gYmFzZWxpbmU6IHZhbGlkYXRlZCBg
R2F0ZXdheUF1dGhCdW5kbGVWMWAsIHJhdy1zZWNyZXQgZW52IHNjcnViYmluZywgb25lLXRpbWUg
Z2F0ZXdheSBjb25zdW1wdGlvbiwgYW5kIGZyZXNoIGJ1bmRsZSBkZWxpdmVyeSBvbiByZXN0YXJ0
LgorMi4gUmVzb2x2ZSBhIGNyZWRlbnRpYWwgc291cmNlIG9uIHRoZSBob3N0IGFuZCBsYXVuY2gg
dGhlIGV4YWN0IG1hbmFnZWQgaW4td29ybGQgZ2F0ZXdheSB3aXRob3V0IGNvcHlpbmcgdGhlIHNl
Y3JldCBwYXlsb2FkIGludG8gYSB3b3JsZC12aXNpYmxlIHJ1bnRpbWUgaG9tZS4KKzMuIFN0YXJ0
IGRpcmVjdCB3b3JsZCBDb2RleCB3aXRoIGEgcGVyLXdvcmtlciBTdWJzdHJhdGUtb3duZWQgcHJv
amVjdGlvbiB0aGF0IHBvaW50cyBwcm92aWRlciB0cmFmZmljIGF0IHRoYXQgZ2F0ZXdheTsgZG8g
bm90IHNlZWQgYSBob3N0IGF1dGggZmlsZSBpbiBjb250cmFjdC1jb3JyZWN0IG1vZGUuCis0LiBK
b2luIHRoZSBlbnZlbG9wZSwgYWNjZXB0ZWQgcG9saWN5IHNuYXBzaG90LCBwcm9qZWN0aW9uIGlk
ZW50aXR5LCBleGFjdCBnYXRld2F5IHJlY2VpdmVyLCBhbmQgbm9uLXNlY3JldCBoYW5kb2ZmIGV2
aWRlbmNlIGZvciB0aGUgc2FtZSB3b3JsZCBnZW5lcmF0aW9uLgorNS4gUHJvdmUgdGhlIFVBQSBj
aGlsZCBhbmQgZGVzY2VuZGFudHMgY2Fubm90IGluaGVyaXQgdGhlIHNlY3JldCBGRCBvciByYXcg
c2VjcmV0IGFuZCB0aGF0IG5vIHNlY3JldC1iZWFyaW5nIHJ1bnRpbWUtbmF0aXZlIGZpbGUgaXMg
Y3JlYXRlZC4KKzYuIEV4ZXJjaXNlIGR1cGxpY2F0ZSBjb25zdW1lLCB3cm9uZyBnYXRld2F5L3dv
cmxkIGdlbmVyYXRpb24sIGV4cGlyeSwgdW5hdmFpbGFibGUgaGFuZG9mZiwgYW5kIHVuYXZhaWxh
YmxlIGdhdGV3YXkgYWRvcHRpb247IHJlcXVpcmUgZmFpbC1jbG9zZWQgYmVoYXZpb3IuCis3LiBF
eGVyY2lzZSB0aGUgZXhwbGljaXRseSBuYW1lZCBjb21wYXRpYmlsaXR5IGNvcHkgbW9kZSBzZXBh
cmF0ZWx5IGFuZCBwcm92ZSBpdCBpcyBsb2dnZWQsIGhhcyByZXRpcmVtZW50IG1ldGFkYXRhLCBh
bmQgY2Fubm90IHNhdGlzZnkgY29udHJhY3QtcHJvbW90aW9uIGV2aWRlbmNlLgorOC4gSW5zcGVj
dCBsb2dzL3RyYWNlcy9tYW5pZmVzdHMgZm9yIHNlY3JldCBwYXlsb2Fkcywgc2VjcmV0LWJlYXJp
bmcgcGF0aHMsIGFuZCByZXVzYWJsZSBzZWNyZXQtZGVyaXZlZCBoYXNoZXM7IG5vbmUgbWF5IGFw
cGVhci4KKworQ292ZXJzOiBgUkctQ09ORklHLTAyYCwgYFJHLUNPTkZJRy0wM2AsIGBSRy1DT05G
SUctMDRgLCBgUkctVUFBLTAyYCwgYFJHLVVBQS0wM2AsIGBSRy1PQlMtMDFgLgorCis8IS0tIGV4
YWN0LWV4dHJhY3RlZC1ib2R5OmNyb3NzLWdhdGUtc21va2Utc2NlbmFyaW9zOmVuZCAtLT4KYGBg
CgpWYWxpZGF0aW9uIGxvZyAoU0hBLTI1NiBjZTQyMmY2OWY5NTRkODQ4ZWMzMmZiOTg2ZmQyOWYz
OTljZjM0MDdiNTUxY2Y1OGNiZDVkZWJmNDQ2MGExMTA2KToKYGBgdGV4dApBUlRJRkFDVF9BX09L
IGJ5dGVzPTM2NDQgc2hhMjU2PTVmOWVjOGQ5ODMyNDAxNTk4NmM4NDYyMDhkYjFjNTY3YjQ5Yzkw
OWJhYTcxNDQyOGFjM2E2NDQ3MDlmZTkxMzIgcGF0aD0vcHJpdmF0ZS90bXAvZDExLWJhc2VsaW5l
LWJlaGF2aW9ycy1saXZlLXNvdXJjZS1saW5lcy04MC05Mi5tZApBUlRJRkFDVF9CX09LIGJ5dGVz
PTExNzE0IHNoYTI1Nj01ODZlNTQwMmI5MmI2ZTRlOThhYjdjYmRkMDRlNjcxMjFmZjIwOWNjZjIz
YzdmMWZmMTE1YzE4OGJlZGJjYTg1IHBhdGg9L3ByaXZhdGUvdG1wL2QxMS1jcm9zcy1nYXRlLXNt
b2tlLXNjZW5hcmlvcy1saXZlLXNvdXJjZS1saW5lcy05My0xOTIubWQKUFJFX0VESVRfU1BBTl9N
QVRDSF9PSyBiYXNlbGluZS1saW5lcz04MC05MiBieXRlcz0zNjQ0IHNoYTI1Nj01ZjllYzhkOTgz
MjQwMTU5ODZjODQ2MjA4ZGIxYzU2N2I0OWM5MDliYWE3MTQ0MjhhYzNhNjQ0NzA5ZmU5MTMyClBS
RV9FRElUX1NQQU5fTUFUQ0hfT0sgYmFzZWxpbmUtbGluZXM9OTMtMTkyIGJ5dGVzPTExNzE0IHNo
YTI1Nj01ODZlNTQwMmI5MmI2ZTRlOThhYjdjYmRkMDRlNjcxMjFmZjIwOWNjZjIzYzdmMWZmMTE1
YzE4OGJlZGJjYTg1CkNNRCBnaXQgc3RhdHVzIC0tc2hvcnQKU1RBVFVTICBNIGxsbS1sYXN0LW1p
bGUvcnVudGltZS1yZWZhY3Rvci8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZApTVEFUVVMg
IE0gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2V2aWRlbmNlL1JFQURNRS5tZApTVEFU
VVMgIE0gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZApTVEFU
VVMgIE0gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9u
LWxlZGdlci5tZApTVEFUVVMgPz8gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2V2aWRl
bmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kClBBVEhfRkVOQ0Vf
T0sgNSBjaGFuZ2VkIHBhdGhzIGV4YWN0bHkgbWF0Y2ggYWxsb3dlZCBmZW5jZTsgbm8gc3RhZ2Vk
IGZpbGVzCkNNRCBnaXQgZGlmZiAtLWNoZWNrIC0tIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZh
Y3Rvci8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCBsbG0tbGFzdC1taWxlL3J1bnRpbWUt
cmVmYWN0b3IvZXZpZGVuY2UvUkVBRE1FLm1kIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rv
ci9pbmRleC9SRUFETUUubWQgbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlv
bi9leHRyYWN0aW9uLWxlZGdlci5tZApHSVRfRElGRl9DSEVDS19PSwpDTUQgZ2l0IGRpZmYgLS1u
by1pbmRleCAtLWNoZWNrIC9kZXYvbnVsbCBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3Iv
ZXZpZGVuY2UvYmFzZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9rZS1zY2VuYXJpb3MubWQKTkVXX0ZJ
TEVfRElGRl9DSEVDS19PSyBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZXZpZGVuY2Uv
YmFzZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9rZS1zY2VuYXJpb3MubWQKT1dORVJfQk9EWV9FUVVB
TF9PSyBuYW1lPWJhc2VsaW5lLWJlaGF2aW9ycyBieXRlcz0zNjQ0IHNoYTI1Nj01ZjllYzhkOTgz
MjQwMTU5ODZjODQ2MjA4ZGIxYzU2N2I0OWM5MDliYWE3MTQ0MjhhYzNhNjQ0NzA5ZmU5MTMyIGNt
cD1leGFjdApPV05FUl9CT0RZX0VRVUFMX09LIG5hbWU9Y3Jvc3MtZ2F0ZS1zbW9rZS1zY2VuYXJp
b3MgYnl0ZXM9MTE3MTQgc2hhMjU2PTU4NmU1NDAyYjkyYjZlNGU5OGFiN2NiZGQwNGU2NzEyMWZm
MjA5Y2NmMjNjN2YxZmYxMTVjMTg4YmVkYmNhODUgY21wPWV4YWN0ClJPT1RfUkVDT05TVFJVQ1RJ
T05fT0sgY3VycmVudC1yb290IGVxdWFscyBiYXNlbGluZSBwcmVmaXggKyBCYXRjaDMgY29tcGF0
aWJpbGl0eSBwcm9qZWN0aW9ucyArIGJhc2VsaW5lIHN1ZmZpeApST09UX1BSRUZJWF9TVUZGSVhf
U1RBQkxFX09LIGxpbmVzMS03OSBhbmQgbGluZXMxOTMtRU9GIGFyZSBieXRlLXN0YWJsZSB2ZXJz
dXMgSEVBRApST09UX1NUVUJfT0sgbGVnYWN5IGhlYWRpbmdzIHByZXNlcnZlZCBvbmNlIGVhY2gg
aW4gb3JpZ2luYWwgb3JkZXIgd2l0aCBvd25lci1hbmNob3IgY29tcGF0aWJpbGl0eSBwb2ludGVy
cwpPV05FUl9BTkNIT1JTX09LIGJhc2VsaW5lLWJlaGF2aW9ycy10aGF0LWFsbC10cmFja3MtcHJl
c2VydmU9MSBjcm9zcy1nYXRlLXNtb2tlLXNjZW5hcmlvcz0xIHMxLS1hdXRob3JpdHktc3Vydml2
ZXMtZXBpc29kZS1sb3NzPTEgczFhLS1yZXZpc2lvbi1ib3VuZC1ob3N0LXRyYW5zaXRpb25zLXN1
cnZpdmUtaGVscGVyLWxvc3MtYW5kLXJlcGxheT0xIHMyLS1yZWNlaXB0LXN1cGVydmlzb3Itb2Js
aWdhdGlvbi1jYW5jZWw9MSBzMy0td29ybGQtdWFhLW1lZGlhdGlvbi11bmRlci1uYXJyb3dlZC1w
b2xpY3k9MSBzNC0taG9zdC12aXNpYmlsaXR5LWFuZC1mYWlsdXJlLXRydXRoPTEgczUtLWV4aXN0
aW5nLWdhdGV3YXktY2Fycmllci1wcmVzZXJ2YXRpb24tYW5kLWRpcmVjdC1jb2RleC1hZG9wdGlv
bj0xCkJBU0VMSU5FX1RBQkxFX09LIGdhdGVfaWRzX29yZGVyPVsnUkctQkFTRS0wMScsICdSRy1C
QVNFLTAyJywgJ1JHLUJBU0UtMDMnLCAnUkctQkFTRS0wNCcsICdSRy1ESUZGLTAxJ10gcm93X2Nv
dW50PTUgb3Blbl9ibG9ja2luZ19ub3Rfd2FpdmVkPXByZXNlbnQKU01PS0VfU0NFTkFSSU9TX09L
IHNjZW5hcmlvc19vcmRlcj1bJ1MxJywgJ1MxQScsICdTMicsICdTMycsICdTNCcsICdTNSddIHN0
ZXBfY291bnRzPXsnUzEnOiA1LCAnUzFBJzogMTgsICdTMic6IDYsICdTMyc6IDUsICdTNCc6IDQs
ICdTNSc6IDh9IGRldGVybWluaXN0aWNfcGFya2VkX3N1Y2Nlc3Nvcl9zdGVwcz0xMApDT1ZFUlNf
T0sgY291bnQ9NiBTMSxTMUEsUzIsUzMsUzQsUzUgcHJlc2VydmVkIGV4YWN0bHkgd2l0aCBtdWx0
aWxpbmUgUzIgY29udGludWF0aW9uIGludGFjdApCT0RZX1BSRVNFUlZBVElPTl9PSyBleGFjdCBi
eXRlIGVxdWFsaXR5IHByZXNlcnZlcyB0YWJsZSBjZWxscy9vcmRlciwgR2F0ZSBJRHMsIHN0YXR1
c2VzLCBvcmRlcmVkIGFuZCBuZXN0ZWQgc2NlbmFyaW8gc2VxdWVuY2VzLCBDb3ZlcnMgbGluZXMs
IG5lZ2F0aXZlIHJlcXVpcmVtZW50cywgZmFpbC1jbG9zZWQgY2FzZXMsIGV4Y2VwdGlvbnMsIHBs
YXRmb3JtIGxpbWl0YXRpb25zLCBhbmQgZXZpZGVuY2UtdnMtc2F0aXNmYWN0aW9uIGJvdW5kYXJp
ZXMKUk9XX0NPVU5UX09LIGV2aWRlbmNlX25hdmlnYXRpb249MSBnbG9iYWxfaW5kZXg9MSBkMTFf
bGVkZ2VyPTIKT1dORVJfUkVWSUVXX1NUQUJJTElUWV9PSyBzbmFwc2hvdF9wYXRocz0zMjMKTElO
S19TQ0FOX1NVTU1BUlkgY2hlY2tlZD0xOTQxIGV4Y2x1ZGVkPTQgZmFpbHVyZXM9MApQQVRDSF9X
UklUVEVOIC9wcml2YXRlL3RtcC9kMTEtYmFzZWxpbmUtc21va2UtY2FuZGlkYXRlLnBhdGNoIHNo
YTI1Nj0zMzcwZWNlMWY4N2UzNzIzZDAwYmJmZjBiN2NlNmJkNmFmMzA0Y2Q3MDI2YTdkYmE5OWEy
OGQ3MGFmYjAyMDk0ClBBVENIX1NIQV9TSURFQ0FSX1dSSVRURU4gL3ByaXZhdGUvdG1wL2QxMS1i
YXNlbGluZS1zbW9rZS1jYW5kaWRhdGUucGF0Y2guc2hhMjU2ClBBVENIX0FQUExZX09LIGZvcndh
cmRfYW5kX3JldmVyc2VfYXBwbHlfbWF0Y2hfY2xlYW5fYmFzZWxpbmUKRklMRV9TSEEgbGxtLWxh
c3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kIDE1
OWY1MzFkYzA1YWVmMWFmYzRkNWEyNmViNDEyYzEzMzBkYjc1ZWY0ZGIxYTkwNTAzNjkwZjEzOWM1
MzhiZjYKRklMRV9TSEEgbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2V2aWRlbmNlL2Jh
c2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kIDAyM2RlZjY0ODgyZTczODI2
NGIwNzJmMjZmYjQzNDM5N2Y1M2VhOGExMjJhOGQ1ZmQ0NWExMDRiMDZiMzc2OTgKRklMRV9TSEEg
bGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2V2aWRlbmNlL1JFQURNRS5tZCAyZWQ5OWU0
NjBmZjg0YmI3NDJjNWQ5OWI4MmRmMjViM2I2MWQzZTE2OGVhNDdhNWY2YzBkY2ZlNzI0ZTUyNmYy
CkZJTEVfU0hBIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9SRUFETUUubWQg
NzE1N2I1YWUxYTM2YzU1Mzk3ODhlOGYzOWUzMjM3NTU4NTU2NWEyNTFkYTYzODZhYWYxZTYwYjUx
YzI2MzZlMgpGSUxFX1NIQSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9u
L2V4dHJhY3Rpb24tbGVkZ2VyLm1kIGYxZGRmNDNiNTQ0MDBhMjlkNjljYThkMzk4MjhlZjUzYzRh
NTAyZGRlZTdkMjBkNWYyYjI0ZmQ4ODViYzBjNDYKYGBgCg==
```

</details>

<details>
<summary>Initial-range review answer (rendered copy)</summary>

<pre>
P2 — RG-DIFF-01 loses a required source-position referent in the new canonical owner

File/hunk: llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md, new-file hunk, ## Baseline behaviors that all tracks preserve, RG-DIFF-01.

Supported-input reachability: A maintainer following the new canonical-owner navigation reaches RG-DIFF-01, whose required-proof cell still says: “The exact fourteen-test B1/B2.1 inventory above…”. In the complete destination file, nothing above that row identifies or links that fourteen-test inventory: the destination contains metadata, the new title, the baseline heading/introduction, and the preceding RG-BASE rows. The provenance also establishes that this preserved body begins at source line 80, so the positional referent necessarily lies outside the extracted body.&#32;


Violated invariant: The candidate requires the permanent baseline table to preserve its semantic state and required-proof meaning, while the new file becomes the single canonical owner for that family.&#32;


 The review threshold explicitly treats patch-caused loss of preserved baseline semantics, or another defect that can make maintainers materially rely on wrong D11 state, as blocking.&#32;


Material consequence: The clause is not merely descriptive: it identifies the exact fourteen-test inventory whose historical names and production-level assertions must be preserved for RG-DIFF-01. In the canonical destination, “above” no longer resolves that evidence set. A maintainer applying the differential gate therefore lacks an unambiguous canonical identification of which fourteen tests the requirement governs, potentially affecting whether a failure-to-pass transition is accepted or keeps closeout open.

Patch causality: This candidate transfers canonical ownership of the row into a standalone file while intentionally extracting only baseline source lines 80–92. It does not carry or link the source context referenced by “above.” The reported byte-equality checks correctly establish textual preservation, but cannot establish preservation of this source-relative semantic reference.&#32;


Required remediation: Preserve the exact marked 3644-byte body unchanged, but add destination-local context outside that preservation marker that explicitly identifies and links the canonical owner/location of the exact fourteen-test B1/B2.1 inventory, so the preserved “inventory above” reference remains truthfully resolvable.

I found no additional qualifying blocking issue in the compatibility-heading projections, Batch 3 evidence navigation, global-index row, distinct source provenance, paired atomic rollback entries, authority boundaries, scenario ordering, or supported relative links. The supplied validation also supports the claimed five-path fence, exact-body hashes, legacy-anchor uniqueness, scenario/Covers preservation, and zero supported-link failures.&#32;


VERDICT: REQUEST_CHANGES
</pre>

</details>

<details>
<summary>Initial-range review answer exact bytes (base64 UTF-8)</summary>

```text
UDIg4oCUIFJHLURJRkYtMDEgbG9zZXMgYSByZXF1aXJlZCBzb3VyY2UtcG9zaXRpb24gcmVmZXJl
bnQgaW4gdGhlIG5ldyBjYW5vbmljYWwgb3duZXIKCkZpbGUvaHVuazogbGxtLWxhc3QtbWlsZS9y
dW50aW1lLXJlZmFjdG9yL2V2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2Nl
bmFyaW9zLm1kLCBuZXctZmlsZSBodW5rLCAjIyBCYXNlbGluZSBiZWhhdmlvcnMgdGhhdCBhbGwg
dHJhY2tzIHByZXNlcnZlLCBSRy1ESUZGLTAxLgoKU3VwcG9ydGVkLWlucHV0IHJlYWNoYWJpbGl0
eTogQSBtYWludGFpbmVyIGZvbGxvd2luZyB0aGUgbmV3IGNhbm9uaWNhbC1vd25lciBuYXZpZ2F0
aW9uIHJlYWNoZXMgUkctRElGRi0wMSwgd2hvc2UgcmVxdWlyZWQtcHJvb2YgY2VsbCBzdGlsbCBz
YXlzOiDigJxUaGUgZXhhY3QgZm91cnRlZW4tdGVzdCBCMS9CMi4xIGludmVudG9yeSBhYm92ZeKA
puKAnS4gSW4gdGhlIGNvbXBsZXRlIGRlc3RpbmF0aW9uIGZpbGUsIG5vdGhpbmcgYWJvdmUgdGhh
dCByb3cgaWRlbnRpZmllcyBvciBsaW5rcyB0aGF0IGZvdXJ0ZWVuLXRlc3QgaW52ZW50b3J5OiB0
aGUgZGVzdGluYXRpb24gY29udGFpbnMgbWV0YWRhdGEsIHRoZSBuZXcgdGl0bGUsIHRoZSBiYXNl
bGluZSBoZWFkaW5nL2ludHJvZHVjdGlvbiwgYW5kIHRoZSBwcmVjZWRpbmcgUkctQkFTRSByb3dz
LiBUaGUgcHJvdmVuYW5jZSBhbHNvIGVzdGFibGlzaGVzIHRoYXQgdGhpcyBwcmVzZXJ2ZWQgYm9k
eSBiZWdpbnMgYXQgc291cmNlIGxpbmUgODAsIHNvIHRoZSBwb3NpdGlvbmFsIHJlZmVyZW50IG5l
Y2Vzc2FyaWx5IGxpZXMgb3V0c2lkZSB0aGUgZXh0cmFjdGVkIGJvZHkuIAoKClZpb2xhdGVkIGlu
dmFyaWFudDogVGhlIGNhbmRpZGF0ZSByZXF1aXJlcyB0aGUgcGVybWFuZW50IGJhc2VsaW5lIHRh
YmxlIHRvIHByZXNlcnZlIGl0cyBzZW1hbnRpYyBzdGF0ZSBhbmQgcmVxdWlyZWQtcHJvb2YgbWVh
bmluZywgd2hpbGUgdGhlIG5ldyBmaWxlIGJlY29tZXMgdGhlIHNpbmdsZSBjYW5vbmljYWwgb3du
ZXIgZm9yIHRoYXQgZmFtaWx5LiAKCgogVGhlIHJldmlldyB0aHJlc2hvbGQgZXhwbGljaXRseSB0
cmVhdHMgcGF0Y2gtY2F1c2VkIGxvc3Mgb2YgcHJlc2VydmVkIGJhc2VsaW5lIHNlbWFudGljcywg
b3IgYW5vdGhlciBkZWZlY3QgdGhhdCBjYW4gbWFrZSBtYWludGFpbmVycyBtYXRlcmlhbGx5IHJl
bHkgb24gd3JvbmcgRDExIHN0YXRlLCBhcyBibG9ja2luZy4gCgoKTWF0ZXJpYWwgY29uc2VxdWVu
Y2U6IFRoZSBjbGF1c2UgaXMgbm90IG1lcmVseSBkZXNjcmlwdGl2ZTogaXQgaWRlbnRpZmllcyB0
aGUgZXhhY3QgZm91cnRlZW4tdGVzdCBpbnZlbnRvcnkgd2hvc2UgaGlzdG9yaWNhbCBuYW1lcyBh
bmQgcHJvZHVjdGlvbi1sZXZlbCBhc3NlcnRpb25zIG11c3QgYmUgcHJlc2VydmVkIGZvciBSRy1E
SUZGLTAxLiBJbiB0aGUgY2Fub25pY2FsIGRlc3RpbmF0aW9uLCDigJxhYm92ZeKAnSBubyBsb25n
ZXIgcmVzb2x2ZXMgdGhhdCBldmlkZW5jZSBzZXQuIEEgbWFpbnRhaW5lciBhcHBseWluZyB0aGUg
ZGlmZmVyZW50aWFsIGdhdGUgdGhlcmVmb3JlIGxhY2tzIGFuIHVuYW1iaWd1b3VzIGNhbm9uaWNh
bCBpZGVudGlmaWNhdGlvbiBvZiB3aGljaCBmb3VydGVlbiB0ZXN0cyB0aGUgcmVxdWlyZW1lbnQg
Z292ZXJucywgcG90ZW50aWFsbHkgYWZmZWN0aW5nIHdoZXRoZXIgYSBmYWlsdXJlLXRvLXBhc3Mg
dHJhbnNpdGlvbiBpcyBhY2NlcHRlZCBvciBrZWVwcyBjbG9zZW91dCBvcGVuLgoKUGF0Y2ggY2F1
c2FsaXR5OiBUaGlzIGNhbmRpZGF0ZSB0cmFuc2ZlcnMgY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0
aGUgcm93IGludG8gYSBzdGFuZGFsb25lIGZpbGUgd2hpbGUgaW50ZW50aW9uYWxseSBleHRyYWN0
aW5nIG9ubHkgYmFzZWxpbmUgc291cmNlIGxpbmVzIDgw4oCTOTIuIEl0IGRvZXMgbm90IGNhcnJ5
IG9yIGxpbmsgdGhlIHNvdXJjZSBjb250ZXh0IHJlZmVyZW5jZWQgYnkg4oCcYWJvdmUu4oCdIFRo
ZSByZXBvcnRlZCBieXRlLWVxdWFsaXR5IGNoZWNrcyBjb3JyZWN0bHkgZXN0YWJsaXNoIHRleHR1
YWwgcHJlc2VydmF0aW9uLCBidXQgY2Fubm90IGVzdGFibGlzaCBwcmVzZXJ2YXRpb24gb2YgdGhp
cyBzb3VyY2UtcmVsYXRpdmUgc2VtYW50aWMgcmVmZXJlbmNlLiAKCgpSZXF1aXJlZCByZW1lZGlh
dGlvbjogUHJlc2VydmUgdGhlIGV4YWN0IG1hcmtlZCAzNjQ0LWJ5dGUgYm9keSB1bmNoYW5nZWQs
IGJ1dCBhZGQgZGVzdGluYXRpb24tbG9jYWwgY29udGV4dCBvdXRzaWRlIHRoYXQgcHJlc2VydmF0
aW9uIG1hcmtlciB0aGF0IGV4cGxpY2l0bHkgaWRlbnRpZmllcyBhbmQgbGlua3MgdGhlIGNhbm9u
aWNhbCBvd25lci9sb2NhdGlvbiBvZiB0aGUgZXhhY3QgZm91cnRlZW4tdGVzdCBCMS9CMi4xIGlu
dmVudG9yeSwgc28gdGhlIHByZXNlcnZlZCDigJxpbnZlbnRvcnkgYWJvdmXigJ0gcmVmZXJlbmNl
IHJlbWFpbnMgdHJ1dGhmdWxseSByZXNvbHZhYmxlLgoKSSBmb3VuZCBubyBhZGRpdGlvbmFsIHF1
YWxpZnlpbmcgYmxvY2tpbmcgaXNzdWUgaW4gdGhlIGNvbXBhdGliaWxpdHktaGVhZGluZyBwcm9q
ZWN0aW9ucywgQmF0Y2ggMyBldmlkZW5jZSBuYXZpZ2F0aW9uLCBnbG9iYWwtaW5kZXggcm93LCBk
aXN0aW5jdCBzb3VyY2UgcHJvdmVuYW5jZSwgcGFpcmVkIGF0b21pYyByb2xsYmFjayBlbnRyaWVz
LCBhdXRob3JpdHkgYm91bmRhcmllcywgc2NlbmFyaW8gb3JkZXJpbmcsIG9yIHN1cHBvcnRlZCBy
ZWxhdGl2ZSBsaW5rcy4gVGhlIHN1cHBsaWVkIHZhbGlkYXRpb24gYWxzbyBzdXBwb3J0cyB0aGUg
Y2xhaW1lZCBmaXZlLXBhdGggZmVuY2UsIGV4YWN0LWJvZHkgaGFzaGVzLCBsZWdhY3ktYW5jaG9y
IHVuaXF1ZW5lc3MsIHNjZW5hcmlvL0NvdmVycyBwcmVzZXJ2YXRpb24sIGFuZCB6ZXJvIHN1cHBv
cnRlZC1saW5rIGZhaWx1cmVzLiAKCgpWRVJESUNUOiBSRVFVRVNUX0NIQU5HRVMK
```

</details>

<details>
<summary>Remediation-follow-up review prompt (rendered copy)</summary>

<pre>
Review mode: remediation-follow-up
Review context: independent fresh conversation; no reliance on prior chat history beyond the content-addressed facts below
Review boundary: baseline commit c164ec0c7acb24abcb44211afa26a2630dfa86cf / tree 3170c00c22c65b693299137de03404e57be3108b. Previously reviewed complete initial candidate patch SHA-256 3370ece1f87e3723d00bbff0b7ce6bd6af304cd7026a7dba99a28d70afb02094. Accepted-finding remediation delta SHA-256 54e5796515f1d5c77d335f9493cfdcd47f727c135c926c9261652b6bf360341b. Ordered byte stream formed by initial-patch bytes followed immediately by remediation-delta bytes SHA-256 e4ff934ea42b5495b8fa543d44197fb9a3f3e6d15506af6e11b4ff29c30317ee. Updated complete candidate patch from the clean baseline SHA-256 ff7a759e8b559a84aacf3e4338e7195c6899c8afc26af652a70d631464726c28.
Review target: only whether the accepted P2 finding is fixed and whether this one-file remediation delta introduced a qualifying regression. Do not reopen findings already reviewed as clean and do not broaden to adjacent concerns.

Supported inputs and behavior: repository-local UTF-8 Markdown with GitHub-style anchors, relative links/fragments, tables, ordered lists, inline code, and HTML-comment exact-body markers. Relative links resolve from the containing file. Exact preserved body bytes are those strictly between named marker pairs.
Supported platforms and dialects: documentation-only repository-local Markdown/GitHub-style anchors. Exclude runtime/platform execution, alternative dialects, remote URL availability, and D12 cutover.

Accepted prior finding (the only remediation objective):
P2 — `RG-DIFF-01` in the new canonical owner retained the exact phrase `The exact fourteen-test B1/B2.1 inventory above...`, but the standalone extraction began at the baseline heading, so nothing above the row identified or linked the source-position referent. A maintainer could not unambiguously locate the exact fourteen historical names/routes whose production assertions govern failure-to-pass classification. Required remediation: keep the exact 3644-byte preserved body unchanged, and add destination-local context outside the marker, above the preserved table, identifying and linking the already-canonical packet-local owner/location of that exact inventory.

Remediation implementation:
- Edited only `llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md` relative to the reviewed candidate.
- Added a 540-byte context note above the first exact-body marker.
- The note points to `../b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result`, the already-canonical packet-local section containing the exact fourteen historical B1/B2.1 names/production routes.
- It states that this is referent clarification only, does not duplicate/re-extract packet-local evidence, and changes no evidence, gate, implementation, or authority status.
- Current remediated owner SHA-256: 90c9fe4e20fed14a5d2c3a22cb697e56e1a29191d905172e0104e19a02b2c0d2.

In-scope invariants for this follow-up:
- The link makes the source-relative `inventory above` referent reachable and unambiguous in the standalone owner.
- The already-canonical packet-local owner remains unchanged and is not duplicated/re-extracted.
- Both exact preserved bodies and all markers remain byte-identical: baseline body 3644 bytes / SHA-256 5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132; smoke body 11714 bytes / SHA-256 586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85.
- The delta changes no other candidate file, legacy heading/anchor, table/scenario order, evidence/gate status, or authority boundary.
- The new relative link/fragment resolves under supported Markdown behavior and no supported local-link or whitespace regression is introduced.

Required material consequence: a blocker must show the accepted source-relative referent remains materially unresolved, or the delta introduced a reachable P1/P2-equivalent regression such as changing exact preserved bytes/semantics, breaking supported links/anchors, duplicating/inflating authority, or changing another path/claim. Style preference or unrelated/pre-existing behavior is not material.
Blocking threshold: reachable through supported behavior, violates the accepted remediation objective or a listed invariant, meets the material threshold, and is caused by this delta.
Deferred or out-of-scope concerns: every finding previously reviewed clean; wording/style outside the delta; global formatting; D3/review-control contents; D5/D6 owner changes; D11 Batch 1/2; tracker/closeout; D12; runtime/platform/implementation; proof satisfaction, promotion, remediation authority, dispatch, successor work; four predeclared opaque links; unrelated repository defects.

Validation evidence:
- exact dirty candidate fence remains five paths; the delta versus reviewed candidate touches only the owner file; nothing staged
- four non-owner candidate hashes unchanged
- packet-local target owner hash-stable and its heading anchor resolves
- new destination-local link resolves
- both marker-delimited bodies retain exact bytes/counts/hashes
- eight legacy root/owner anchors, baseline table order/state, scenario/Covers/order semantics unchanged
- git diff --check and new-file no-index whitespace check pass
- runtime-refactor scan checked 1942 local links/fragments, excluded only four known opaque links, failures zero
- remediation delta and updated full patch both forward/reverse apply in their proper states

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Report a blocker only if the accepted finding remains unfixed or this delta introduces a qualifying regression. For any blocker give exact file/hunk, reachability, invariant, material consequence, and delta causality. Otherwise explicitly state the accepted finding is fixed and no qualifying delta regression remains. End with exactly one verdict line: `VERDICT: APPROVED` or `VERDICT: REQUEST_CHANGES`.

Remediation delta (SHA-256 54e5796515f1d5c77d335f9493cfdcd47f727c135c926c9261652b6bf360341b):
```diff
--- a/llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md
+++ b/llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md
@@ -15,6 +15,8 @@
&#32;
 # Baseline behaviors and smoke scenarios
&#32;
+Context note: the exact fourteen historical B1/B2.1 names and production routes referenced by RG-DIFF-01&#x27;s source-relative phrase `inventory above` remain canonically owned in [`../b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result`](../b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result). This destination-local link clarifies that referent only; it does not duplicate or re-extract packet-local evidence and does not change any evidence, gate, implementation, or authority status.
+
 &lt;!-- exact-extracted-body:baseline-behaviors:start --&gt;
 ## Baseline behaviors that all tracks preserve
&#32;
```

Remediation validation log (SHA-256 eb7d77ccf76385fd3c981452760e3e5212fab8a03dd6d8b9ebd01ce609bc76b1):
```text
PRECHECK_HEAD c164ec0c7acb24abcb44211afa26a2630dfa86cf
PRECHECK_TREE 3170c00c22c65b693299137de03404e57be3108b
PRECHECK_BRANCH (detached)
STATUS_LINES_START
STATUS  M llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
STATUS  M llm-last-mile/runtime-refactor/evidence/README.md
STATUS  M llm-last-mile/runtime-refactor/index/README.md
STATUS  M llm-last-mile/runtime-refactor/migration/extraction-ledger.md
STATUS ?? llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md
STATUS_LINES_END
PATH_FENCE_OK 5 changed paths exactly match allowed fence; no staged files
FILE_SHA llm-last-mile/runtime-refactor/05-debug-regression-ledger.md 159f531dc05aef1afc4d5a26eb412c1330db75ef4db1a90503690f139c538bf6
FILE_SHA llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md 90c9fe4e20fed14a5d2c3a22cb697e56e1a29191d905172e0104e19a02b2c0d2
FILE_SHA llm-last-mile/runtime-refactor/evidence/README.md 2ed99e460ff84bb742c5d99b82df25b3b61d3e168ea47a5f6c0dcfe724e526f2
FILE_SHA llm-last-mile/runtime-refactor/index/README.md 7157b5ae1a36c5539788e8f39e32375585565a251da6386aaf1e60b51c2636e2
FILE_SHA llm-last-mile/runtime-refactor/migration/extraction-ledger.md f1ddf43b54400a29d69ca8d39828ef53c4a502ddee7d20d5f2b24fd885bc0c46
REVIEWED_CANDIDATE_SHA 023def64882e738264b072f26fb434397f53ea8a122a8d5fd45a104b06b37698 path=/private/tmp/d11-baseline-smoke-reviewed-candidate-owner.md
REMEDIATION_DELTA_SCOPE_OK non-owner candidate hashes remain exact and only the owner differs from the reviewed candidate baseline
OWNER_CONTEXT_NOTE_OK line=18 bytes_added=540 link=../b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result
B1_REFERENT_LINK_OK target=../b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result anchor_present=1 packet_local_owner_sha256=6387b0a8548c542c9c985f69e5f6845eb1379c59534421009de7763a5ce3dbed
ARTIFACT_A_OK bytes=3644 sha256=5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132 path=/private/tmp/d11-baseline-behaviors-live-source-lines-80-92.md
ARTIFACT_B_OK bytes=11714 sha256=586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85 path=/private/tmp/d11-cross-gate-smoke-scenarios-live-source-lines-93-192.md
OWNER_BODY_EQUAL_OK name=baseline-behaviors bytes=3644 sha256=5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132 cmp=exact
OWNER_BODY_EQUAL_OK name=cross-gate-smoke-scenarios bytes=11714 sha256=586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85 cmp=exact
BODY_PRESERVATION_OK exact body byte equality preserves the baseline table, RG-DIFF-01 sentence, scenario ordering, step ordering, Covers lines, and all evidence-vs-status semantics
ROOT_STUB_OK legacy headings preserved once each in original order with owner-anchor compatibility pointers
OWNER_ANCHORS_OK baseline-behaviors-that-all-tracks-preserve=1 cross-gate-smoke-scenarios=1 s1--authority-survives-episode-loss=1 s1a--revision-bound-host-transitions-survive-helper-loss-and-replay=1 s2--receipt-supervisor-obligation-cancel=1 s3--world-uaa-mediation-under-narrowed-policy=1 s4--host-visibility-and-failure-truth=1 s5--existing-gateway-carrier-preservation-and-direct-codex-adoption=1
BASELINE_TABLE_OK gate_ids_order=[&#x27;RG-BASE-01&#x27;, &#x27;RG-BASE-02&#x27;, &#x27;RG-BASE-03&#x27;, &#x27;RG-BASE-04&#x27;, &#x27;RG-DIFF-01&#x27;] row_count=5 open_blocking_not_waived=present
SMOKE_SCENARIOS_OK scenarios_order=[&#x27;S1&#x27;, &#x27;S1A&#x27;, &#x27;S2&#x27;, &#x27;S3&#x27;, &#x27;S4&#x27;, &#x27;S5&#x27;] step_counts={&#x27;S1&#x27;: 5, &#x27;S1A&#x27;: 18, &#x27;S2&#x27;: 6, &#x27;S3&#x27;: 5, &#x27;S4&#x27;: 4, &#x27;S5&#x27;: 8} deterministic_parked_successor_steps=10
SCENARIO_SEMANTICS_OK exact preserved bodies keep the same ordered nested scenario sequence and the same causal/negative requirements
ROW_COUNT_OK evidence_navigation=1 global_index=1 d11_ledger=2
OWNER_REVIEW_STABILITY_OK stable_owner=llm-last-mile/runtime-refactor/b1-b2-1/evidence-regression.md sha256=6387b0a8548c542c9c985f69e5f6845eb1379c59534421009de7763a5ce3dbed git_diff_clean=yes
GIT_DIFF_CHECK_OK
NEW_FILE_DIFF_CHECK_OK llm-last-mile/runtime-refactor/evidence/baseline-behaviors-and-smoke-scenarios.md
LINK_SCAN_SUMMARY checked=1942 excluded=4 failures=0
KNOWN_OPAQUE_OK 00-README.md-&gt;../../substrate-runtime-refactor-directive-revised.md; 05-debug-regression-ledger.md-&gt;../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md; 05-debug-regression-ledger.md-&gt;../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md; 05-debug-regression-ledger.md-&gt;../../RUN_WORLD_TASK_DEBUG_CANONICAL.md
DELTA_PATCH_WRITTEN /private/tmp/d11-baseline-smoke-remediation-delta.patch sha256=54e5796515f1d5c77d335f9493cfdcd47f727c135c926c9261652b6bf360341b
DELTA_PATCH_SHA_SIDECAR_WRITTEN /private/tmp/d11-baseline-smoke-remediation-delta.patch.sha256
DELTA_PATCH_APPLY_OK forward_and_reverse_apply_match_reviewed_candidate
FULL_PATCH_WRITTEN /private/tmp/d11-baseline-smoke-remediated-candidate.patch sha256=ff7a759e8b559a84aacf3e4338e7195c6899c8afc26af652a70d631464726c28
FULL_PATCH_SHA_SIDECAR_WRITTEN /private/tmp/d11-baseline-smoke-remediated-candidate.patch.sha256
FULL_PATCH_APPLY_OK forward_and_reverse_apply_match_clean_baseline
```
</pre>

</details>

<details>
<summary>Remediation-follow-up review prompt exact bytes (base64 UTF-8)</summary>

```text
UmV2aWV3IG1vZGU6IHJlbWVkaWF0aW9uLWZvbGxvdy11cApSZXZpZXcgY29udGV4dDogaW5kZXBl
bmRlbnQgZnJlc2ggY29udmVyc2F0aW9uOyBubyByZWxpYW5jZSBvbiBwcmlvciBjaGF0IGhpc3Rv
cnkgYmV5b25kIHRoZSBjb250ZW50LWFkZHJlc3NlZCBmYWN0cyBiZWxvdwpSZXZpZXcgYm91bmRh
cnk6IGJhc2VsaW5lIGNvbW1pdCBjMTY0ZWMwYzdhY2IyNGFiY2I0NDIxMWFmYTI2YTI2MzBkZmE4
NmNmIC8gdHJlZSAzMTcwYzAwYzIyYzY1YjY5MzI5OTEzN2RlMDM0MDRlNTdiZTMxMDhiLiBQcmV2
aW91c2x5IHJldmlld2VkIGNvbXBsZXRlIGluaXRpYWwgY2FuZGlkYXRlIHBhdGNoIFNIQS0yNTYg
MzM3MGVjZTFmODdlMzcyM2QwMGJiZmYwYjdjZTZiZDZhZjMwNGNkNzAyNmE3ZGJhOTlhMjhkNzBh
ZmIwMjA5NC4gQWNjZXB0ZWQtZmluZGluZyByZW1lZGlhdGlvbiBkZWx0YSBTSEEtMjU2IDU0ZTU3
OTY1MTVmMWQ1Yzc3ZDMzNWY5NDkzY2ZkY2Q0N2Y3MjdjMTM1YzkyNmM5MjYxNjUyYjZiZjM2MDM0
MWIuIE9yZGVyZWQgYnl0ZSBzdHJlYW0gZm9ybWVkIGJ5IGluaXRpYWwtcGF0Y2ggYnl0ZXMgZm9s
bG93ZWQgaW1tZWRpYXRlbHkgYnkgcmVtZWRpYXRpb24tZGVsdGEgYnl0ZXMgU0hBLTI1NiBlNGZm
OTM0ZWE0MmI1NDk1YjhmYTU0M2Q0NDE5N2ZiOWEzZjNlNmQxNTUwNmFmNmUxMWI0ZmYyOWMzMDMx
N2VlLiBVcGRhdGVkIGNvbXBsZXRlIGNhbmRpZGF0ZSBwYXRjaCBmcm9tIHRoZSBjbGVhbiBiYXNl
bGluZSBTSEEtMjU2IGZmN2E3NTllOGI1NTlhODRhYWNmM2U0MzM4ZTcxOTVjNjg5OWM4YWZjMjZh
ZjY1MmE3MGQ2MzE0NjQ3MjZjMjguClJldmlldyB0YXJnZXQ6IG9ubHkgd2hldGhlciB0aGUgYWNj
ZXB0ZWQgUDIgZmluZGluZyBpcyBmaXhlZCBhbmQgd2hldGhlciB0aGlzIG9uZS1maWxlIHJlbWVk
aWF0aW9uIGRlbHRhIGludHJvZHVjZWQgYSBxdWFsaWZ5aW5nIHJlZ3Jlc3Npb24uIERvIG5vdCBy
ZW9wZW4gZmluZGluZ3MgYWxyZWFkeSByZXZpZXdlZCBhcyBjbGVhbiBhbmQgZG8gbm90IGJyb2Fk
ZW4gdG8gYWRqYWNlbnQgY29uY2VybnMuCgpTdXBwb3J0ZWQgaW5wdXRzIGFuZCBiZWhhdmlvcjog
cmVwb3NpdG9yeS1sb2NhbCBVVEYtOCBNYXJrZG93biB3aXRoIEdpdEh1Yi1zdHlsZSBhbmNob3Jz
LCByZWxhdGl2ZSBsaW5rcy9mcmFnbWVudHMsIHRhYmxlcywgb3JkZXJlZCBsaXN0cywgaW5saW5l
IGNvZGUsIGFuZCBIVE1MLWNvbW1lbnQgZXhhY3QtYm9keSBtYXJrZXJzLiBSZWxhdGl2ZSBsaW5r
cyByZXNvbHZlIGZyb20gdGhlIGNvbnRhaW5pbmcgZmlsZS4gRXhhY3QgcHJlc2VydmVkIGJvZHkg
Ynl0ZXMgYXJlIHRob3NlIHN0cmljdGx5IGJldHdlZW4gbmFtZWQgbWFya2VyIHBhaXJzLgpTdXBw
b3J0ZWQgcGxhdGZvcm1zIGFuZCBkaWFsZWN0czogZG9jdW1lbnRhdGlvbi1vbmx5IHJlcG9zaXRv
cnktbG9jYWwgTWFya2Rvd24vR2l0SHViLXN0eWxlIGFuY2hvcnMuIEV4Y2x1ZGUgcnVudGltZS9w
bGF0Zm9ybSBleGVjdXRpb24sIGFsdGVybmF0aXZlIGRpYWxlY3RzLCByZW1vdGUgVVJMIGF2YWls
YWJpbGl0eSwgYW5kIEQxMiBjdXRvdmVyLgoKQWNjZXB0ZWQgcHJpb3IgZmluZGluZyAodGhlIG9u
bHkgcmVtZWRpYXRpb24gb2JqZWN0aXZlKToKUDIg4oCUIGBSRy1ESUZGLTAxYCBpbiB0aGUgbmV3
IGNhbm9uaWNhbCBvd25lciByZXRhaW5lZCB0aGUgZXhhY3QgcGhyYXNlIGBUaGUgZXhhY3QgZm91
cnRlZW4tdGVzdCBCMS9CMi4xIGludmVudG9yeSBhYm92ZS4uLmAsIGJ1dCB0aGUgc3RhbmRhbG9u
ZSBleHRyYWN0aW9uIGJlZ2FuIGF0IHRoZSBiYXNlbGluZSBoZWFkaW5nLCBzbyBub3RoaW5nIGFi
b3ZlIHRoZSByb3cgaWRlbnRpZmllZCBvciBsaW5rZWQgdGhlIHNvdXJjZS1wb3NpdGlvbiByZWZl
cmVudC4gQSBtYWludGFpbmVyIGNvdWxkIG5vdCB1bmFtYmlndW91c2x5IGxvY2F0ZSB0aGUgZXhh
Y3QgZm91cnRlZW4gaGlzdG9yaWNhbCBuYW1lcy9yb3V0ZXMgd2hvc2UgcHJvZHVjdGlvbiBhc3Nl
cnRpb25zIGdvdmVybiBmYWlsdXJlLXRvLXBhc3MgY2xhc3NpZmljYXRpb24uIFJlcXVpcmVkIHJl
bWVkaWF0aW9uOiBrZWVwIHRoZSBleGFjdCAzNjQ0LWJ5dGUgcHJlc2VydmVkIGJvZHkgdW5jaGFu
Z2VkLCBhbmQgYWRkIGRlc3RpbmF0aW9uLWxvY2FsIGNvbnRleHQgb3V0c2lkZSB0aGUgbWFya2Vy
LCBhYm92ZSB0aGUgcHJlc2VydmVkIHRhYmxlLCBpZGVudGlmeWluZyBhbmQgbGlua2luZyB0aGUg
YWxyZWFkeS1jYW5vbmljYWwgcGFja2V0LWxvY2FsIG93bmVyL2xvY2F0aW9uIG9mIHRoYXQgZXhh
Y3QgaW52ZW50b3J5LgoKUmVtZWRpYXRpb24gaW1wbGVtZW50YXRpb246Ci0gRWRpdGVkIG9ubHkg
YGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9ldmlkZW5jZS9iYXNlbGluZS1iZWhhdmlv
cnMtYW5kLXNtb2tlLXNjZW5hcmlvcy5tZGAgcmVsYXRpdmUgdG8gdGhlIHJldmlld2VkIGNhbmRp
ZGF0ZS4KLSBBZGRlZCBhIDU0MC1ieXRlIGNvbnRleHQgbm90ZSBhYm92ZSB0aGUgZmlyc3QgZXhh
Y3QtYm9keSBtYXJrZXIuCi0gVGhlIG5vdGUgcG9pbnRzIHRvIGAuLi9iMS1iMi0xL2V2aWRlbmNl
LXJlZ3Jlc3Npb24ubWQjYjFiMjEtY29yZS1yZWNvdmVyeS1hbmQtYjFiMjEtMC1yZWNvcmRlZC1y
ZXN1bHRgLCB0aGUgYWxyZWFkeS1jYW5vbmljYWwgcGFja2V0LWxvY2FsIHNlY3Rpb24gY29udGFp
bmluZyB0aGUgZXhhY3QgZm91cnRlZW4gaGlzdG9yaWNhbCBCMS9CMi4xIG5hbWVzL3Byb2R1Y3Rp
b24gcm91dGVzLgotIEl0IHN0YXRlcyB0aGF0IHRoaXMgaXMgcmVmZXJlbnQgY2xhcmlmaWNhdGlv
biBvbmx5LCBkb2VzIG5vdCBkdXBsaWNhdGUvcmUtZXh0cmFjdCBwYWNrZXQtbG9jYWwgZXZpZGVu
Y2UsIGFuZCBjaGFuZ2VzIG5vIGV2aWRlbmNlLCBnYXRlLCBpbXBsZW1lbnRhdGlvbiwgb3IgYXV0
aG9yaXR5IHN0YXR1cy4KLSBDdXJyZW50IHJlbWVkaWF0ZWQgb3duZXIgU0hBLTI1NjogOTBjOWZl
NGUyMGZlZDE0YTVkMmMzYTIyY2I2OTdlNTZlMWEyOTE5MWQ5MDUxNzJlMDEwNGUxOWEwMmIyYzBk
Mi4KCkluLXNjb3BlIGludmFyaWFudHMgZm9yIHRoaXMgZm9sbG93LXVwOgotIFRoZSBsaW5rIG1h
a2VzIHRoZSBzb3VyY2UtcmVsYXRpdmUgYGludmVudG9yeSBhYm92ZWAgcmVmZXJlbnQgcmVhY2hh
YmxlIGFuZCB1bmFtYmlndW91cyBpbiB0aGUgc3RhbmRhbG9uZSBvd25lci4KLSBUaGUgYWxyZWFk
eS1jYW5vbmljYWwgcGFja2V0LWxvY2FsIG93bmVyIHJlbWFpbnMgdW5jaGFuZ2VkIGFuZCBpcyBu
b3QgZHVwbGljYXRlZC9yZS1leHRyYWN0ZWQuCi0gQm90aCBleGFjdCBwcmVzZXJ2ZWQgYm9kaWVz
IGFuZCBhbGwgbWFya2VycyByZW1haW4gYnl0ZS1pZGVudGljYWw6IGJhc2VsaW5lIGJvZHkgMzY0
NCBieXRlcyAvIFNIQS0yNTYgNWY5ZWM4ZDk4MzI0MDE1OTg2Yzg0NjIwOGRiMWM1NjdiNDljOTA5
YmFhNzE0NDI4YWMzYTY0NDcwOWZlOTEzMjsgc21va2UgYm9keSAxMTcxNCBieXRlcyAvIFNIQS0y
NTYgNTg2ZTU0MDJiOTJiNmU0ZTk4YWI3Y2JkZDA0ZTY3MTIxZmYyMDljY2YyM2M3ZjFmZjExNWMx
ODhiZWRiY2E4NS4KLSBUaGUgZGVsdGEgY2hhbmdlcyBubyBvdGhlciBjYW5kaWRhdGUgZmlsZSwg
bGVnYWN5IGhlYWRpbmcvYW5jaG9yLCB0YWJsZS9zY2VuYXJpbyBvcmRlciwgZXZpZGVuY2UvZ2F0
ZSBzdGF0dXMsIG9yIGF1dGhvcml0eSBib3VuZGFyeS4KLSBUaGUgbmV3IHJlbGF0aXZlIGxpbmsv
ZnJhZ21lbnQgcmVzb2x2ZXMgdW5kZXIgc3VwcG9ydGVkIE1hcmtkb3duIGJlaGF2aW9yIGFuZCBu
byBzdXBwb3J0ZWQgbG9jYWwtbGluayBvciB3aGl0ZXNwYWNlIHJlZ3Jlc3Npb24gaXMgaW50cm9k
dWNlZC4KClJlcXVpcmVkIG1hdGVyaWFsIGNvbnNlcXVlbmNlOiBhIGJsb2NrZXIgbXVzdCBzaG93
IHRoZSBhY2NlcHRlZCBzb3VyY2UtcmVsYXRpdmUgcmVmZXJlbnQgcmVtYWlucyBtYXRlcmlhbGx5
IHVucmVzb2x2ZWQsIG9yIHRoZSBkZWx0YSBpbnRyb2R1Y2VkIGEgcmVhY2hhYmxlIFAxL1AyLWVx
dWl2YWxlbnQgcmVncmVzc2lvbiBzdWNoIGFzIGNoYW5naW5nIGV4YWN0IHByZXNlcnZlZCBieXRl
cy9zZW1hbnRpY3MsIGJyZWFraW5nIHN1cHBvcnRlZCBsaW5rcy9hbmNob3JzLCBkdXBsaWNhdGlu
Zy9pbmZsYXRpbmcgYXV0aG9yaXR5LCBvciBjaGFuZ2luZyBhbm90aGVyIHBhdGgvY2xhaW0uIFN0
eWxlIHByZWZlcmVuY2Ugb3IgdW5yZWxhdGVkL3ByZS1leGlzdGluZyBiZWhhdmlvciBpcyBub3Qg
bWF0ZXJpYWwuCkJsb2NraW5nIHRocmVzaG9sZDogcmVhY2hhYmxlIHRocm91Z2ggc3VwcG9ydGVk
IGJlaGF2aW9yLCB2aW9sYXRlcyB0aGUgYWNjZXB0ZWQgcmVtZWRpYXRpb24gb2JqZWN0aXZlIG9y
IGEgbGlzdGVkIGludmFyaWFudCwgbWVldHMgdGhlIG1hdGVyaWFsIHRocmVzaG9sZCwgYW5kIGlz
IGNhdXNlZCBieSB0aGlzIGRlbHRhLgpEZWZlcnJlZCBvciBvdXQtb2Ytc2NvcGUgY29uY2VybnM6
IGV2ZXJ5IGZpbmRpbmcgcHJldmlvdXNseSByZXZpZXdlZCBjbGVhbjsgd29yZGluZy9zdHlsZSBv
dXRzaWRlIHRoZSBkZWx0YTsgZ2xvYmFsIGZvcm1hdHRpbmc7IEQzL3Jldmlldy1jb250cm9sIGNv
bnRlbnRzOyBENS9ENiBvd25lciBjaGFuZ2VzOyBEMTEgQmF0Y2ggMS8yOyB0cmFja2VyL2Nsb3Nl
b3V0OyBEMTI7IHJ1bnRpbWUvcGxhdGZvcm0vaW1wbGVtZW50YXRpb247IHByb29mIHNhdGlzZmFj
dGlvbiwgcHJvbW90aW9uLCByZW1lZGlhdGlvbiBhdXRob3JpdHksIGRpc3BhdGNoLCBzdWNjZXNz
b3Igd29yazsgZm91ciBwcmVkZWNsYXJlZCBvcGFxdWUgbGlua3M7IHVucmVsYXRlZCByZXBvc2l0
b3J5IGRlZmVjdHMuCgpWYWxpZGF0aW9uIGV2aWRlbmNlOgotIGV4YWN0IGRpcnR5IGNhbmRpZGF0
ZSBmZW5jZSByZW1haW5zIGZpdmUgcGF0aHM7IHRoZSBkZWx0YSB2ZXJzdXMgcmV2aWV3ZWQgY2Fu
ZGlkYXRlIHRvdWNoZXMgb25seSB0aGUgb3duZXIgZmlsZTsgbm90aGluZyBzdGFnZWQKLSBmb3Vy
IG5vbi1vd25lciBjYW5kaWRhdGUgaGFzaGVzIHVuY2hhbmdlZAotIHBhY2tldC1sb2NhbCB0YXJn
ZXQgb3duZXIgaGFzaC1zdGFibGUgYW5kIGl0cyBoZWFkaW5nIGFuY2hvciByZXNvbHZlcwotIG5l
dyBkZXN0aW5hdGlvbi1sb2NhbCBsaW5rIHJlc29sdmVzCi0gYm90aCBtYXJrZXItZGVsaW1pdGVk
IGJvZGllcyByZXRhaW4gZXhhY3QgYnl0ZXMvY291bnRzL2hhc2hlcwotIGVpZ2h0IGxlZ2FjeSBy
b290L293bmVyIGFuY2hvcnMsIGJhc2VsaW5lIHRhYmxlIG9yZGVyL3N0YXRlLCBzY2VuYXJpby9D
b3ZlcnMvb3JkZXIgc2VtYW50aWNzIHVuY2hhbmdlZAotIGdpdCBkaWZmIC0tY2hlY2sgYW5kIG5l
dy1maWxlIG5vLWluZGV4IHdoaXRlc3BhY2UgY2hlY2sgcGFzcwotIHJ1bnRpbWUtcmVmYWN0b3Ig
c2NhbiBjaGVja2VkIDE5NDIgbG9jYWwgbGlua3MvZnJhZ21lbnRzLCBleGNsdWRlZCBvbmx5IGZv
dXIga25vd24gb3BhcXVlIGxpbmtzLCBmYWlsdXJlcyB6ZXJvCi0gcmVtZWRpYXRpb24gZGVsdGEg
YW5kIHVwZGF0ZWQgZnVsbCBwYXRjaCBib3RoIGZvcndhcmQvcmV2ZXJzZSBhcHBseSBpbiB0aGVp
ciBwcm9wZXIgc3RhdGVzCgpEbyBub3QgcmVkdWNlIHRoZSB0YXNrIG9yIHJlcGxhY2UgaXQgd2l0
aCBhbiBlYXNpZXIgYWx0ZXJuYXRpdmUuIFByZXNlcnZlIHRoZSByZXF1ZXN0ZWQgc2NvcGUgYW5k
IHByb2plY3QgY29udmVudGlvbnMuCgpSZXBvcnQgYSBibG9ja2VyIG9ubHkgaWYgdGhlIGFjY2Vw
dGVkIGZpbmRpbmcgcmVtYWlucyB1bmZpeGVkIG9yIHRoaXMgZGVsdGEgaW50cm9kdWNlcyBhIHF1
YWxpZnlpbmcgcmVncmVzc2lvbi4gRm9yIGFueSBibG9ja2VyIGdpdmUgZXhhY3QgZmlsZS9odW5r
LCByZWFjaGFiaWxpdHksIGludmFyaWFudCwgbWF0ZXJpYWwgY29uc2VxdWVuY2UsIGFuZCBkZWx0
YSBjYXVzYWxpdHkuIE90aGVyd2lzZSBleHBsaWNpdGx5IHN0YXRlIHRoZSBhY2NlcHRlZCBmaW5k
aW5nIGlzIGZpeGVkIGFuZCBubyBxdWFsaWZ5aW5nIGRlbHRhIHJlZ3Jlc3Npb24gcmVtYWlucy4g
RW5kIHdpdGggZXhhY3RseSBvbmUgdmVyZGljdCBsaW5lOiBgVkVSRElDVDogQVBQUk9WRURgIG9y
IGBWRVJESUNUOiBSRVFVRVNUX0NIQU5HRVNgLgoKUmVtZWRpYXRpb24gZGVsdGEgKFNIQS0yNTYg
NTRlNTc5NjUxNWYxZDVjNzdkMzM1Zjk0OTNjZmRjZDQ3ZjcyN2MxMzVjOTI2YzkyNjE2NTJiNmJm
MzYwMzQxYik6CmBgYGRpZmYKLS0tIGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2V2
aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kCisrKyBiL2xs
bS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9ldmlkZW5jZS9iYXNlbGluZS1iZWhhdmlvcnMt
YW5kLXNtb2tlLXNjZW5hcmlvcy5tZApAQCAtMTUsNiArMTUsOCBAQAogCiAjIEJhc2VsaW5lIGJl
aGF2aW9ycyBhbmQgc21va2Ugc2NlbmFyaW9zCiAKK0NvbnRleHQgbm90ZTogdGhlIGV4YWN0IGZv
dXJ0ZWVuIGhpc3RvcmljYWwgQjEvQjIuMSBuYW1lcyBhbmQgcHJvZHVjdGlvbiByb3V0ZXMgcmVm
ZXJlbmNlZCBieSBSRy1ESUZGLTAxJ3Mgc291cmNlLXJlbGF0aXZlIHBocmFzZSBgaW52ZW50b3J5
IGFib3ZlYCByZW1haW4gY2Fub25pY2FsbHkgb3duZWQgaW4gW2AuLi9iMS1iMi0xL2V2aWRlbmNl
LXJlZ3Jlc3Npb24ubWQjYjFiMjEtY29yZS1yZWNvdmVyeS1hbmQtYjFiMjEtMC1yZWNvcmRlZC1y
ZXN1bHRgXSguLi9iMS1iMi0xL2V2aWRlbmNlLXJlZ3Jlc3Npb24ubWQjYjFiMjEtY29yZS1yZWNv
dmVyeS1hbmQtYjFiMjEtMC1yZWNvcmRlZC1yZXN1bHQpLiBUaGlzIGRlc3RpbmF0aW9uLWxvY2Fs
IGxpbmsgY2xhcmlmaWVzIHRoYXQgcmVmZXJlbnQgb25seTsgaXQgZG9lcyBub3QgZHVwbGljYXRl
IG9yIHJlLWV4dHJhY3QgcGFja2V0LWxvY2FsIGV2aWRlbmNlIGFuZCBkb2VzIG5vdCBjaGFuZ2Ug
YW55IGV2aWRlbmNlLCBnYXRlLCBpbXBsZW1lbnRhdGlvbiwgb3IgYXV0aG9yaXR5IHN0YXR1cy4K
KwogPCEtLSBleGFjdC1leHRyYWN0ZWQtYm9keTpiYXNlbGluZS1iZWhhdmlvcnM6c3RhcnQgLS0+
CiAjIyBCYXNlbGluZSBiZWhhdmlvcnMgdGhhdCBhbGwgdHJhY2tzIHByZXNlcnZlCiAKYGBgCgpS
ZW1lZGlhdGlvbiB2YWxpZGF0aW9uIGxvZyAoU0hBLTI1NiBlYjdkNzdjY2Y3NjM4NWZkM2M5ODE0
NTI3NjBlM2U1MjEyZmFiOGEwM2RkNmQ4YjllYmQwMWNlNjA5YmM3NmIxKToKYGBgdGV4dApQUkVD
SEVDS19IRUFEIGMxNjRlYzBjN2FjYjI0YWJjYjQ0MjExYWZhMjZhMjYzMGRmYTg2Y2YKUFJFQ0hF
Q0tfVFJFRSAzMTcwYzAwYzIyYzY1YjY5MzI5OTEzN2RlMDM0MDRlNTdiZTMxMDhiClBSRUNIRUNL
X0JSQU5DSCAoZGV0YWNoZWQpClNUQVRVU19MSU5FU19TVEFSVApTVEFUVVMgIE0gbGxtLWxhc3Qt
bWlsZS9ydW50aW1lLXJlZmFjdG9yLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kClNUQVRV
UyAgTSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZXZpZGVuY2UvUkVBRE1FLm1kClNU
QVRVUyAgTSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvaW5kZXgvUkVBRE1FLm1kClNU
QVRVUyAgTSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJhY3Rp
b24tbGVkZ2VyLm1kClNUQVRVUyA/PyBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZXZp
ZGVuY2UvYmFzZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9rZS1zY2VuYXJpb3MubWQKU1RBVFVTX0xJ
TkVTX0VORApQQVRIX0ZFTkNFX09LIDUgY2hhbmdlZCBwYXRocyBleGFjdGx5IG1hdGNoIGFsbG93
ZWQgZmVuY2U7IG5vIHN0YWdlZCBmaWxlcwpGSUxFX1NIQSBsbG0tbGFzdC1taWxlL3J1bnRpbWUt
cmVmYWN0b3IvMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQgMTU5ZjUzMWRjMDVhZWYxYWZj
NGQ1YTI2ZWI0MTJjMTMzMGRiNzVlZjRkYjFhOTA1MDM2OTBmMTM5YzUzOGJmNgpGSUxFX1NIQSBs
bG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZXZpZGVuY2UvYmFzZWxpbmUtYmVoYXZpb3Jz
LWFuZC1zbW9rZS1zY2VuYXJpb3MubWQgOTBjOWZlNGUyMGZlZDE0YTVkMmMzYTIyY2I2OTdlNTZl
MWEyOTE5MWQ5MDUxNzJlMDEwNGUxOWEwMmIyYzBkMgpGSUxFX1NIQSBsbG0tbGFzdC1taWxlL3J1
bnRpbWUtcmVmYWN0b3IvZXZpZGVuY2UvUkVBRE1FLm1kIDJlZDk5ZTQ2MGZmODRiYjc0MmM1ZDk5
YjgyZGYyNWIzYjYxZDNlMTY4ZWE0N2E1ZjZjMGRjZmU3MjRlNTI2ZjIKRklMRV9TSEEgbGxtLWxh
c3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZCA3MTU3YjVhZTFhMzZjNTUz
OTc4OGU4ZjM5ZTMyMzc1NTg1NTY1YTI1MWRhNjM4NmFhZjFlNjBiNTFjMjYzNmUyCkZJTEVfU0hB
IGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRn
ZXIubWQgZjFkZGY0M2I1NDQwMGEyOWQ2OWNhOGQzOTgyOGVmNTNjNGE1MDJkZGVlN2QyMGQ1ZjJi
MjRmZDg4NWJjMGM0NgpSRVZJRVdFRF9DQU5ESURBVEVfU0hBIDAyM2RlZjY0ODgyZTczODI2NGIw
NzJmMjZmYjQzNDM5N2Y1M2VhOGExMjJhOGQ1ZmQ0NWExMDRiMDZiMzc2OTggcGF0aD0vcHJpdmF0
ZS90bXAvZDExLWJhc2VsaW5lLXNtb2tlLXJldmlld2VkLWNhbmRpZGF0ZS1vd25lci5tZApSRU1F
RElBVElPTl9ERUxUQV9TQ09QRV9PSyBub24tb3duZXIgY2FuZGlkYXRlIGhhc2hlcyByZW1haW4g
ZXhhY3QgYW5kIG9ubHkgdGhlIG93bmVyIGRpZmZlcnMgZnJvbSB0aGUgcmV2aWV3ZWQgY2FuZGlk
YXRlIGJhc2VsaW5lCk9XTkVSX0NPTlRFWFRfTk9URV9PSyBsaW5lPTE4IGJ5dGVzX2FkZGVkPTU0
MCBsaW5rPS4uL2IxLWIyLTEvZXZpZGVuY2UtcmVncmVzc2lvbi5tZCNiMWIyMS1jb3JlLXJlY292
ZXJ5LWFuZC1iMWIyMS0wLXJlY29yZGVkLXJlc3VsdApCMV9SRUZFUkVOVF9MSU5LX09LIHRhcmdl
dD0uLi9iMS1iMi0xL2V2aWRlbmNlLXJlZ3Jlc3Npb24ubWQjYjFiMjEtY29yZS1yZWNvdmVyeS1h
bmQtYjFiMjEtMC1yZWNvcmRlZC1yZXN1bHQgYW5jaG9yX3ByZXNlbnQ9MSBwYWNrZXRfbG9jYWxf
b3duZXJfc2hhMjU2PTYzODdiMGE4NTQ4YzU0MmM5Yzk4NWY2OWU1ZjY4NDVlYjEzNzljNTk1MzQ0
MjEwMDlkZTc3NjNhNWNlM2RiZWQKQVJUSUZBQ1RfQV9PSyBieXRlcz0zNjQ0IHNoYTI1Nj01Zjll
YzhkOTgzMjQwMTU5ODZjODQ2MjA4ZGIxYzU2N2I0OWM5MDliYWE3MTQ0MjhhYzNhNjQ0NzA5ZmU5
MTMyIHBhdGg9L3ByaXZhdGUvdG1wL2QxMS1iYXNlbGluZS1iZWhhdmlvcnMtbGl2ZS1zb3VyY2Ut
bGluZXMtODAtOTIubWQKQVJUSUZBQ1RfQl9PSyBieXRlcz0xMTcxNCBzaGEyNTY9NTg2ZTU0MDJi
OTJiNmU0ZTk4YWI3Y2JkZDA0ZTY3MTIxZmYyMDljY2YyM2M3ZjFmZjExNWMxODhiZWRiY2E4NSBw
YXRoPS9wcml2YXRlL3RtcC9kMTEtY3Jvc3MtZ2F0ZS1zbW9rZS1zY2VuYXJpb3MtbGl2ZS1zb3Vy
Y2UtbGluZXMtOTMtMTkyLm1kCk9XTkVSX0JPRFlfRVFVQUxfT0sgbmFtZT1iYXNlbGluZS1iZWhh
dmlvcnMgYnl0ZXM9MzY0NCBzaGEyNTY9NWY5ZWM4ZDk4MzI0MDE1OTg2Yzg0NjIwOGRiMWM1Njdi
NDljOTA5YmFhNzE0NDI4YWMzYTY0NDcwOWZlOTEzMiBjbXA9ZXhhY3QKT1dORVJfQk9EWV9FUVVB
TF9PSyBuYW1lPWNyb3NzLWdhdGUtc21va2Utc2NlbmFyaW9zIGJ5dGVzPTExNzE0IHNoYTI1Nj01
ODZlNTQwMmI5MmI2ZTRlOThhYjdjYmRkMDRlNjcxMjFmZjIwOWNjZjIzYzdmMWZmMTE1YzE4OGJl
ZGJjYTg1IGNtcD1leGFjdApCT0RZX1BSRVNFUlZBVElPTl9PSyBleGFjdCBib2R5IGJ5dGUgZXF1
YWxpdHkgcHJlc2VydmVzIHRoZSBiYXNlbGluZSB0YWJsZSwgUkctRElGRi0wMSBzZW50ZW5jZSwg
c2NlbmFyaW8gb3JkZXJpbmcsIHN0ZXAgb3JkZXJpbmcsIENvdmVycyBsaW5lcywgYW5kIGFsbCBl
dmlkZW5jZS12cy1zdGF0dXMgc2VtYW50aWNzClJPT1RfU1RVQl9PSyBsZWdhY3kgaGVhZGluZ3Mg
cHJlc2VydmVkIG9uY2UgZWFjaCBpbiBvcmlnaW5hbCBvcmRlciB3aXRoIG93bmVyLWFuY2hvciBj
b21wYXRpYmlsaXR5IHBvaW50ZXJzCk9XTkVSX0FOQ0hPUlNfT0sgYmFzZWxpbmUtYmVoYXZpb3Jz
LXRoYXQtYWxsLXRyYWNrcy1wcmVzZXJ2ZT0xIGNyb3NzLWdhdGUtc21va2Utc2NlbmFyaW9zPTEg
czEtLWF1dGhvcml0eS1zdXJ2aXZlcy1lcGlzb2RlLWxvc3M9MSBzMWEtLXJldmlzaW9uLWJvdW5k
LWhvc3QtdHJhbnNpdGlvbnMtc3Vydml2ZS1oZWxwZXItbG9zcy1hbmQtcmVwbGF5PTEgczItLXJl
Y2VpcHQtc3VwZXJ2aXNvci1vYmxpZ2F0aW9uLWNhbmNlbD0xIHMzLS13b3JsZC11YWEtbWVkaWF0
aW9uLXVuZGVyLW5hcnJvd2VkLXBvbGljeT0xIHM0LS1ob3N0LXZpc2liaWxpdHktYW5kLWZhaWx1
cmUtdHJ1dGg9MSBzNS0tZXhpc3RpbmctZ2F0ZXdheS1jYXJyaWVyLXByZXNlcnZhdGlvbi1hbmQt
ZGlyZWN0LWNvZGV4LWFkb3B0aW9uPTEKQkFTRUxJTkVfVEFCTEVfT0sgZ2F0ZV9pZHNfb3JkZXI9
WydSRy1CQVNFLTAxJywgJ1JHLUJBU0UtMDInLCAnUkctQkFTRS0wMycsICdSRy1CQVNFLTA0Jywg
J1JHLURJRkYtMDEnXSByb3dfY291bnQ9NSBvcGVuX2Jsb2NraW5nX25vdF93YWl2ZWQ9cHJlc2Vu
dApTTU9LRV9TQ0VOQVJJT1NfT0sgc2NlbmFyaW9zX29yZGVyPVsnUzEnLCAnUzFBJywgJ1MyJywg
J1MzJywgJ1M0JywgJ1M1J10gc3RlcF9jb3VudHM9eydTMSc6IDUsICdTMUEnOiAxOCwgJ1MyJzog
NiwgJ1MzJzogNSwgJ1M0JzogNCwgJ1M1JzogOH0gZGV0ZXJtaW5pc3RpY19wYXJrZWRfc3VjY2Vz
c29yX3N0ZXBzPTEwClNDRU5BUklPX1NFTUFOVElDU19PSyBleGFjdCBwcmVzZXJ2ZWQgYm9kaWVz
IGtlZXAgdGhlIHNhbWUgb3JkZXJlZCBuZXN0ZWQgc2NlbmFyaW8gc2VxdWVuY2UgYW5kIHRoZSBz
YW1lIGNhdXNhbC9uZWdhdGl2ZSByZXF1aXJlbWVudHMKUk9XX0NPVU5UX09LIGV2aWRlbmNlX25h
dmlnYXRpb249MSBnbG9iYWxfaW5kZXg9MSBkMTFfbGVkZ2VyPTIKT1dORVJfUkVWSUVXX1NUQUJJ
TElUWV9PSyBzdGFibGVfb3duZXI9bGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2IxLWIy
LTEvZXZpZGVuY2UtcmVncmVzc2lvbi5tZCBzaGEyNTY9NjM4N2IwYTg1NDhjNTQyYzljOTg1ZjY5
ZTVmNjg0NWViMTM3OWM1OTUzNDQyMTAwOWRlNzc2M2E1Y2UzZGJlZCBnaXRfZGlmZl9jbGVhbj15
ZXMKR0lUX0RJRkZfQ0hFQ0tfT0sKTkVXX0ZJTEVfRElGRl9DSEVDS19PSyBsbG0tbGFzdC1taWxl
L3J1bnRpbWUtcmVmYWN0b3IvZXZpZGVuY2UvYmFzZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9rZS1z
Y2VuYXJpb3MubWQKTElOS19TQ0FOX1NVTU1BUlkgY2hlY2tlZD0xOTQyIGV4Y2x1ZGVkPTQgZmFp
bHVyZXM9MApLTk9XTl9PUEFRVUVfT0sgMDAtUkVBRE1FLm1kLT4uLi8uLi9zdWJzdHJhdGUtcnVu
dGltZS1yZWZhY3Rvci1kaXJlY3RpdmUtcmV2aXNlZC5tZDsgMDUtZGVidWctcmVncmVzc2lvbi1s
ZWRnZXIubWQtPi4uLy4uL0NPREVYX1dPUkxEX0RJU1BBVENIX0dBUF9XUklURVVQLm1kOyAwNS1k
ZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZC0+Li4vLi4vQ09OVElOVUVfV09STERfV09SS0VSX0JM
T0NLSU5HX0RFVklBVElPTl9ERUJVRy5tZDsgMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQt
Pi4uLy4uL1JVTl9XT1JMRF9UQVNLX0RFQlVHX0NBTk9OSUNBTC5tZApERUxUQV9QQVRDSF9XUklU
VEVOIC9wcml2YXRlL3RtcC9kMTEtYmFzZWxpbmUtc21va2UtcmVtZWRpYXRpb24tZGVsdGEucGF0
Y2ggc2hhMjU2PTU0ZTU3OTY1MTVmMWQ1Yzc3ZDMzNWY5NDkzY2ZkY2Q0N2Y3MjdjMTM1YzkyNmM5
MjYxNjUyYjZiZjM2MDM0MWIKREVMVEFfUEFUQ0hfU0hBX1NJREVDQVJfV1JJVFRFTiAvcHJpdmF0
ZS90bXAvZDExLWJhc2VsaW5lLXNtb2tlLXJlbWVkaWF0aW9uLWRlbHRhLnBhdGNoLnNoYTI1NgpE
RUxUQV9QQVRDSF9BUFBMWV9PSyBmb3J3YXJkX2FuZF9yZXZlcnNlX2FwcGx5X21hdGNoX3Jldmll
d2VkX2NhbmRpZGF0ZQpGVUxMX1BBVENIX1dSSVRURU4gL3ByaXZhdGUvdG1wL2QxMS1iYXNlbGlu
ZS1zbW9rZS1yZW1lZGlhdGVkLWNhbmRpZGF0ZS5wYXRjaCBzaGEyNTY9ZmY3YTc1OWU4YjU1OWE4
NGFhY2YzZTQzMzhlNzE5NWM2ODk5YzhhZmMyNmFmNjUyYTcwZDYzMTQ2NDcyNmMyOApGVUxMX1BB
VENIX1NIQV9TSURFQ0FSX1dSSVRURU4gL3ByaXZhdGUvdG1wL2QxMS1iYXNlbGluZS1zbW9rZS1y
ZW1lZGlhdGVkLWNhbmRpZGF0ZS5wYXRjaC5zaGEyNTYKRlVMTF9QQVRDSF9BUFBMWV9PSyBmb3J3
YXJkX2FuZF9yZXZlcnNlX2FwcGx5X21hdGNoX2NsZWFuX2Jhc2VsaW5lCmBgYAo=
```

</details>

<details>
<summary>Remediation-follow-up review answer (rendered copy)</summary>

<pre>
The accepted P2 finding is fixed. The remediation does exactly what the prior finding required: it adds destination-local context above—and outside—the exact-body marker, explicitly identifies the RG-DIFF-01 source-relative inventory above referent as the exact fourteen historical B1/B2.1 names/production routes, and links directly to their already-canonical packet-local owner.&#32;


 The added note is sufficiently specific and unambiguous; the fact that its link text is the path itself is only a presentation choice, not a material defect.&#32;


No qualifying delta regression remains. The supplied validation establishes that only the intended owner differs from the reviewed candidate, the new target anchor resolves while the packet-local owner remains unchanged, both preserved bodies retain their exact byte counts and hashes, existing anchors/table/scenario semantics remain stable, whitespace checks pass, and the supported local-link scan reports zero failures.&#32;


 These results satisfy the stated invariants against altered preserved semantics, broken links/anchors, duplicated authority, or unintended path/claim changes.&#32;


No blocker found.

VERDICT: APPROVED
</pre>

</details>

<details>
<summary>Remediation-follow-up review answer exact bytes (base64 UTF-8)</summary>

```text
VGhlIGFjY2VwdGVkIFAyIGZpbmRpbmcgaXMgZml4ZWQuIFRoZSByZW1lZGlhdGlvbiBkb2VzIGV4
YWN0bHkgd2hhdCB0aGUgcHJpb3IgZmluZGluZyByZXF1aXJlZDogaXQgYWRkcyBkZXN0aW5hdGlv
bi1sb2NhbCBjb250ZXh0IGFib3Zl4oCUYW5kIG91dHNpZGXigJR0aGUgZXhhY3QtYm9keSBtYXJr
ZXIsIGV4cGxpY2l0bHkgaWRlbnRpZmllcyB0aGUgUkctRElGRi0wMSBzb3VyY2UtcmVsYXRpdmUg
aW52ZW50b3J5IGFib3ZlIHJlZmVyZW50IGFzIHRoZSBleGFjdCBmb3VydGVlbiBoaXN0b3JpY2Fs
IEIxL0IyLjEgbmFtZXMvcHJvZHVjdGlvbiByb3V0ZXMsIGFuZCBsaW5rcyBkaXJlY3RseSB0byB0
aGVpciBhbHJlYWR5LWNhbm9uaWNhbCBwYWNrZXQtbG9jYWwgb3duZXIuIAoKCiBUaGUgYWRkZWQg
bm90ZSBpcyBzdWZmaWNpZW50bHkgc3BlY2lmaWMgYW5kIHVuYW1iaWd1b3VzOyB0aGUgZmFjdCB0
aGF0IGl0cyBsaW5rIHRleHQgaXMgdGhlIHBhdGggaXRzZWxmIGlzIG9ubHkgYSBwcmVzZW50YXRp
b24gY2hvaWNlLCBub3QgYSBtYXRlcmlhbCBkZWZlY3QuIAoKCk5vIHF1YWxpZnlpbmcgZGVsdGEg
cmVncmVzc2lvbiByZW1haW5zLiBUaGUgc3VwcGxpZWQgdmFsaWRhdGlvbiBlc3RhYmxpc2hlcyB0
aGF0IG9ubHkgdGhlIGludGVuZGVkIG93bmVyIGRpZmZlcnMgZnJvbSB0aGUgcmV2aWV3ZWQgY2Fu
ZGlkYXRlLCB0aGUgbmV3IHRhcmdldCBhbmNob3IgcmVzb2x2ZXMgd2hpbGUgdGhlIHBhY2tldC1s
b2NhbCBvd25lciByZW1haW5zIHVuY2hhbmdlZCwgYm90aCBwcmVzZXJ2ZWQgYm9kaWVzIHJldGFp
biB0aGVpciBleGFjdCBieXRlIGNvdW50cyBhbmQgaGFzaGVzLCBleGlzdGluZyBhbmNob3JzL3Rh
YmxlL3NjZW5hcmlvIHNlbWFudGljcyByZW1haW4gc3RhYmxlLCB3aGl0ZXNwYWNlIGNoZWNrcyBw
YXNzLCBhbmQgdGhlIHN1cHBvcnRlZCBsb2NhbC1saW5rIHNjYW4gcmVwb3J0cyB6ZXJvIGZhaWx1
cmVzLiAKCgogVGhlc2UgcmVzdWx0cyBzYXRpc2Z5IHRoZSBzdGF0ZWQgaW52YXJpYW50cyBhZ2Fp
bnN0IGFsdGVyZWQgcHJlc2VydmVkIHNlbWFudGljcywgYnJva2VuIGxpbmtzL2FuY2hvcnMsIGR1
cGxpY2F0ZWQgYXV0aG9yaXR5LCBvciB1bmludGVuZGVkIHBhdGgvY2xhaW0gY2hhbmdlcy4gCgoK
Tm8gYmxvY2tlciBmb3VuZC4KClZFUkRJQ1Q6IEFQUFJPVkVECg==
```

</details>
