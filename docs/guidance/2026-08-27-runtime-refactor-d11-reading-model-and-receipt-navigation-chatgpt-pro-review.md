# ChatGPT Pro advisory review: runtime-refactor D11 reading model and receipt navigation

- Date: 2026-08-27
- Bound baseline commit/tree: `3fba14c8a8acb90bc3de40a75035e2a0f5b5fd76` / `0bcf7ec8b51ada720acaa321934cbfb368cc08c4`
- Landing subagent (`gpt-5.4`, Extra High; display name Carson): `/root/d11_batch4_reading_receipts`
- Remediation subagent (`gpt-5.4`, Extra High; display name Harvey): `/root/d11_batch4_rollback_remediation`
- Closeout subagent (`gpt-5.4`, Extra High; display name not exposed): `/root/d11_batch4_closeout`
- Post-landing truth-finalization subagent (`gpt-5.4`, Extra High; display name not exposed): `/root/d11_batch4_committed_truth_finalize`
- Root/orchestrator review owner: `/root`
- Initial independent review chat: https://chatgpt.com/c/6a8fb3db-ff38-83ea-ad6f-253f969c9a9c
- Initial review mode: `initial-range`
- Initial review context: fresh ChatGPT conversation
- Fresh remediation-follow-up review chat: https://chatgpt.com/c/6a8fb841-5948-83ea-8afe-c973cb190e43
- Remediation-follow-up review mode: `remediation-follow-up`
- Remediation-follow-up review context: fresh ChatGPT conversation
- ChatGPT account surface: `Pro`
- Visible reasoning-effort control: `Extra High`
- Exact model label: not exposed by the current visible ChatGPT UI
- Initially reviewed complete candidate patch: `sha256:2e1e9687f6f0e5991bbde8ac6a2cbf32180ce2604810f31dfc37e75fa1c7fe4e` (`20502 bytes`) at `/private/tmp/d11-reading-receipts-candidate.patch`
- Initial review prompt: `sha256:167c91b1b28dddbd5d0aadfba484e6ce9d544ce6aed9ab297771e61dc8461b92` (`33412 bytes`) at `/private/tmp/d11-reading-receipts-chatgpt-pro-initial-review-prompt.txt`
- Initial review answer: `sha256:3147949ed8eebbfc0065693c2fcdebda28e447556c13e814a53dd497dce8e9ba` (`1841 bytes`) at `/private/tmp/d11-reading-receipts-chatgpt-pro-initial-review-answer.txt`
- Initial validation log: `sha256:66a0b7703bc4c36837d79ae15d91236e7c834f019941b2057db5f845a7c1bf38` (`1898 bytes`) at `/private/tmp/d11-reading-receipts-validation.log`
- Remediation delta: `sha256:17f7b3e2779a77ea4d910d01bc17944ec0aa4a49f4522aa01dd10c8dd4cee723` (`7677 bytes`) at `/private/tmp/d11-reading-receipts-remediation-delta.patch`
- Ordered initial-patch-plus-remediation-delta digest: `sha256:7611ffa2dee56e9b0447320dc2a508ff555fef34790d7f6ad18e3cc0aaa42d96` (`28179 bytes`)
- Updated full remediated candidate patch: `sha256:b3198c157c82ab9ad89ff16691f607427d785ccf9823170b53bfbf49796fd786` (`20774 bytes`) at `/private/tmp/d11-reading-receipts-remediated-candidate.patch`
- Remediation validation log: `sha256:c03fd810295427690ddadd8c041434d999c8cd88751de4858dc37b71d3c697b3` (`2564 bytes`) at `/private/tmp/d11-reading-receipts-remediation-validation.log`
- Remediation-follow-up review prompt: `sha256:1083782e97cd7002605ea85273540f2fb36eae019522231b4de4ebcc60d8b7da` (`17505 bytes`) at `/private/tmp/d11-reading-receipts-chatgpt-pro-remediation-follow-up-prompt.txt`
- Remediation-follow-up review answer: `sha256:96b77793d3b934ec4d67f78eb2112e309fe6c15538bbba020abca311352d3e7c` (`2149 bytes`) at `/private/tmp/d11-reading-receipts-chatgpt-pro-remediation-follow-up-answer.txt`
- Pre-Batch4 exhaustive coverage audit: `sha256:0e07beed0193b03ccdb1b2a4954f9f99e9e97c52ef5d6961ecb4136e15395042` (`5578 bytes`) at `/private/tmp/d11-exhaustive-coverage-audit.txt`
- Post-candidate exhaustive coverage audit: `sha256:e3a3955731d99f5ffc1332eedc95055dcdcf85e2d6ebc5f5b85c74a3b03c89f5` (`3739 bytes`) at `/private/tmp/d11-exhaustive-coverage-audit-post-candidate.txt`
- Exact live source span artifact: lines `3–15`, `823 bytes`, `sha256:80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002` at `/private/tmp/d11-reading-model-live-source-lines-3-15.md`
- Review verdict: `APPROVED`
- Review finding summary: `Initial review returned one accepted P2 on the Batch 4 rollback omitting the separately changed pre-Batch4 evidence/README authority-boundary sentence; remediation changed only the sole D11 Batch 4 rollback cell to restore that exact sentence; follow-up review approved with no qualifying delta regression.`
- Remediation rounds: `1`
- Review adjudication posture: root agent owns review/adjudication; subagents do not self-approve
- Commit posture: Batch 1 is committed complete at detached HEAD `d02168ada079b79c3a12709058f4db893bddabdb`; Batch 2 is committed complete at detached HEAD `c164ec0c7acb24abcb44211afa26a2630dfa86cf`; Batch 3 is committed complete at detached HEAD `3fba14c8a8acb90bc3de40a75035e2a0f5b5fd76`; at closeout time, Batch 4 candidate plus this closeout remained uncommitted and unstaged, but the seven-path Batch 4 artifact has since landed atomically in this commit, and D11 is now fully complete at the resulting committed HEAD
- Push posture: `not pushed`
- D12 posture: `separately unauthorized; blocked by authorization even after D11 completeness; no D12 work has started or been dispatched`
- Successor authority posture: no successor dispatch authority is granted

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Candidate scope and outcome

This independently reviewed final D11 evidence/navigation family extracts the exact `## How to read this ledger` body into `llm-last-mile/runtime-refactor/evidence/README.md`, replaces only that root body with a truthful compatibility pointer, adds the navigation-only receipt index `llm-last-mile/runtime-refactor/evidence/receipts/README.md`, adds the single truthful `shared-evidence-navigation` global-index row, and adds the matching D11 extraction-ledger row while leaving D11 Batch 1–3 owners, prior D3–D10 owners/review artifacts, the existing D6 current-state owner/projection, `index/current.md`, and `review-control/` unchanged. The batch preserves the exact 823-byte reading-model body, keeps the three opaque reading-model links byte-identical in the canonical owner, preserves the root-00 opaque link outside the fence, keeps the receipt index navigation-only, and keeps the current cross-lane projection navigation-only under its existing D6 owner. The initial review accepted one P2: the rollback recipe did not explicitly restore the separately changed pre-Batch4 rendered `evidence/README.md` authority-boundary sentence, so a literal rollback could leave a false canonical-body claim. The sole remediation changed only the single D11 Batch 4 rollback cell to restore that exact sentence; the follow-up review approved with no qualifying delta regression. Batch 4 is complete locally and independently approved after one remediation round and, since the closeout-time uncommitted state recorded above, was landed atomically in this commit. Batch 1 remains committed complete at detached HEAD `d02168ada079b79c3a12709058f4db893bddabdb`, Batch 2 at detached HEAD `c164ec0c7acb24abcb44211afa26a2630dfa86cf`, and Batch 3 at detached HEAD `3fba14c8a8acb90bc3de40a75035e2a0f5b5fd76`. Exhaustive coverage proves every substantive live D11 span is accounted for, so D11 is now fully complete at the resulting committed HEAD. This documentation decomposition is not proof/evidence satisfaction, gate promotion, implementation, remediation, dispatch, or successor authority. D12 remains separately unauthorized and blocked by authorization even after D11 completeness; no D12 work has started or been dispatched; no push or successor dispatch authority is granted.

## Atomic seven-path landing manifest

Atomic seven-path landing manifest for this independently reviewed final D11 batch as landed in this atomic commit:

- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
- `llm-last-mile/runtime-refactor/evidence/README.md`
- `llm-last-mile/runtime-refactor/evidence/receipts/README.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `docs/guidance/2026-08-27-runtime-refactor-d11-reading-model-and-receipt-navigation-chatgpt-pro-review.md`

## Candidate path digests and imported review artifacts

Candidate path digests:

- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`: `sha256:fa4381423c7eb9b3d1d8855f6ffd2a51a727dc941cf1811204cf619fb2925508`
- `llm-last-mile/runtime-refactor/evidence/README.md`: `sha256:bc55fe07b2b4941da1b899046d3efedff08f3fff5419e223d85d9af59391e79a`
- `llm-last-mile/runtime-refactor/evidence/receipts/README.md`: `sha256:1de062b755df040af799e501669aefac646241beebf9caad410ca65bf140324d`
- `llm-last-mile/runtime-refactor/index/README.md`: `sha256:5c3b9376ab54c4ed6c2ecce86e9f2bcb546d0c60bd3bd735c7f34f96e2ae5e2f`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`: `sha256:a1bcf3e7eab175e8e7a6db147432705df217609c4680dc9e96f0777f1fa9d714`

Imported review artifacts:

- Initial candidate patch: `sha256:2e1e9687f6f0e5991bbde8ac6a2cbf32180ce2604810f31dfc37e75fa1c7fe4e` (`20502 bytes`)
- Initial review prompt: `sha256:167c91b1b28dddbd5d0aadfba484e6ce9d544ce6aed9ab297771e61dc8461b92` (`33412 bytes`)
- Initial review answer: `sha256:3147949ed8eebbfc0065693c2fcdebda28e447556c13e814a53dd497dce8e9ba` (`1841 bytes`)
- Initial validation log: `sha256:66a0b7703bc4c36837d79ae15d91236e7c834f019941b2057db5f845a7c1bf38` (`1898 bytes`)
- Remediation delta: `sha256:17f7b3e2779a77ea4d910d01bc17944ec0aa4a49f4522aa01dd10c8dd4cee723` (`7677 bytes`)
- Ordered initial-patch-plus-remediation-delta byte stream: `sha256:7611ffa2dee56e9b0447320dc2a508ff555fef34790d7f6ad18e3cc0aaa42d96` (`28179 bytes`)
- Updated full remediated candidate patch: `sha256:b3198c157c82ab9ad89ff16691f607427d785ccf9823170b53bfbf49796fd786` (`20774 bytes`)
- Remediation validation log: `sha256:c03fd810295427690ddadd8c041434d999c8cd88751de4858dc37b71d3c697b3` (`2564 bytes`)
- Remediation-follow-up review prompt: `sha256:1083782e97cd7002605ea85273540f2fb36eae019522231b4de4ebcc60d8b7da` (`17505 bytes`)
- Remediation-follow-up review answer: `sha256:96b77793d3b934ec4d67f78eb2112e309fe6c15538bbba020abca311352d3e7c` (`2149 bytes`)
- Pre-Batch4 exhaustive coverage audit: `sha256:0e07beed0193b03ccdb1b2a4954f9f99e9e97c52ef5d6961ecb4136e15395042` (`5578 bytes`)
- Post-candidate exhaustive coverage audit: `sha256:e3a3955731d99f5ffc1332eedc95055dcdcf85e2d6ebc5f5b85c74a3b03c89f5` (`3739 bytes`)
- Exact source-span artifact: lines `3–15`, `823 bytes`, `sha256:80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002`

## Exact source-body proof

- Baseline `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md` lines `3–15` are exactly `823` bytes with SHA-256 `80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:how-to-read-this-ledger:start -->` and `<!-- exact-extracted-body:how-to-read-this-ledger:end -->` in `llm-last-mile/runtime-refactor/evidence/README.md` are also exactly `823` bytes with SHA-256 `80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002`.
- Validation confirmed the root ledger now has exactly `83` headings below H1, exactly `83` canonical/compatibility pointer lines, and zero residual substantive nonheading/nonpointer lines; the preserved root heading `## How to read this ledger` remains exactly once as a compatibility anchor.
- The three reading-model opaque Markdown links remain byte-identical exactly once each in `evidence/README.md`, and the root-00 opaque link remains byte-stable outside this batch.
- `ROW_COUNT_OK` preserved Batch 1–3 evidence-navigation rows unchanged and added exactly two Batch 4 navigation rows: one navigation-only receipt/review-control row and one navigation-only current-state compatibility row.
- `RECEIPT_BOUNDARY_OK` confirmed `evidence/receipts/README.md` links only to `../../review-control/README.md`, copies no receipt or review record, and transfers no proof/gate/packet/implementation/dispatch/promotion/review authority.
- `CURRENT_STATE_OWNER_STABLE` confirmed the existing D6 current-state owner `llm-last-mile/runtime-refactor/macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling` stayed byte-identical at `sha256:2a362c70fdbf80a2a67643c9bcf6e4419653cbc9f652cc12e17b72b985a6a678` while Batch 4 added navigation only.
- `INDEX_LEDGER_COUNT_OK` preserved exactly one `shared-evidence-navigation` global-index row and exactly one D11 Batch 4 extraction-ledger row.

## Initial independent review

- The exact initial review prompt preserved below scoped supported behavior to repository-local GitHub-Flavored Markdown, literal navigation/rollback semantics, root compatibility anchors, receipt/current-state navigation boundaries, and exhaustive accounting of all ten D11 roadmap target categories; it excluded D12 cutover, runtime/platform/Rust/script behavior, proof or gate satisfaction, review-control edits, broader repository content, and opaque-link target resolution.
- That prompt required the exact five-path candidate fence, exact 823-byte source-body preservation, root `83/83/0` audit counts, truthful receipt/current-state navigation boundaries, exactly two Batch 4 navigation rows, the single `shared-evidence-navigation` row, the single D11 Batch 4 ledger row, truthful atomic rollback, and no widened authority or D12 claim.
- Its blocking threshold allowed only P1/P2 findings that are reachable through supported behavior, violate an in-scope invariant, have the required material consequence, and are caused or materially worsened by the candidate patch.
- The initial review returned one accepted P2 and **`REQUEST_CHANGES`**.
- Accepted finding: in `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`, the new D11 `How to read this ledger` rollback recipe restored pre-Batch4 metadata and removed the marker body plus Batch 4 navigation rows, but it did not explicitly restore the separately changed pre-Batch4 rendered `evidence/README.md` authority-boundary sentence. A supported reader following the rollback literally could therefore remove the canonical body and still leave behind the patch-caused claim that a body between named markers remained canonical there.
- Material consequence accepted by review: the documented rollback would leave a false ownership/status statement in the canonical destination file after the extracted body was removed, so the rollback was not truthfully atomic.
- The initial review found no additional qualifying P1/P2 blocker in the root compatibility projection, receipt boundary, current-state navigation boundary, global-index row, extraction-ledger provenance, exhaustive ten-category D11 accounting, or supported relative links.

## Bounded remediation

A fresh remediation subagent changed only `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`. It edited only the sole D11 `How to read this ledger` rollback cell, restoring the exact pre-Batch4 rendered authority-boundary sentence `> **Authority boundary:** This file is navigation only. It does not authorize implementation, close a regression gate, satisfy a proof obligation, promote evidence into authority, or replace the linked canonical owners.` as an explicit rollback requirement. The exact 823-byte source body, the root compatibility projection, `evidence/receipts/README.md`, `index/README.md`, `05-debug-regression-ledger.md`, and the rest of `evidence/README.md` remained byte-identical during remediation.

## Remediation validation

- The remediation validation log reverified detached baseline `HEAD`/tree, the exact five-path candidate fence, and zero staged files.
- `git diff --check` remained clean, and `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/evidence/receipts/README.md` remained clean.
- Candidate comparison proved only `llm-last-mile/runtime-refactor/migration/extraction-ledger.md` differs from the initially reviewed candidate; the other four candidate paths remained byte-identical.
- `ROW_ONLY_REMEDIATION_OK` confirmed one replacement line at extraction-ledger line `204`, with no diff outside the single rollback cell relative to the reviewed candidate artifact.
- `BODY_HASH_STABLE` reverified the exact 823-byte source body remained unchanged at `sha256:80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002`.
- Root audit counts remained `83` headings below H1, `83` pointer lines, and `0` residual substantive nonheading/nonpointer lines.
- Markdown link/anchor validation stayed clean with `checked=1953 skipped_opaque=4 failures=0`; the four opaque skips remained exactly the root-00 pointer plus the three reading-model links.
- `CURRENT_STATE_OWNER_STABLE` reverified `llm-last-mile/runtime-refactor/macos-dev-parity/evidence-regression.md` remained unchanged at `sha256:2a362c70fdbf80a2a67643c9bcf6e4419653cbc9f652cc12e17b72b985a6a678`.
- Prior D3/D5–D10 owners, D11 Batch 1–3 owners, and `review-control/` remained stable because the exact dirty fence stayed limited to the same five candidate paths and no staged changes existed.
- The remediation delta forward/reverse apply succeeded against a filesystem copy of the reviewed candidate and returned to byte-identical candidate state; the full remediated candidate patch forward/reverse apply also succeeded against a clean baseline export and returned to a byte-identical baseline tree.

## Fresh remediation-follow-up review

The fresh independent remediation-follow-up review returned:

- **`No qualifying P1/P2 findings.`**
- **`APPROVED`**

It confirmed that the accepted P2 is fully fixed: a supported reader following the remediated rollback literally must restore the exact pre-Batch4 authority-boundary sentence before removing the marker body and Batch 4 navigation rows, so the reachable false canonical-body claim is eliminated. It also confirmed that the one-line remediation delta introduces no qualifying regression in rollback semantics, provenance, source/hash preservation, authority/status boundaries, path fencing, or D12/no-successor state.

## Exhaustive D11 coverage classification

The pre-Batch4 and post-candidate exhaustive audits preserved below in digested form classify every D11 roadmap target truthfully:

1. `canonical issue ledger` — committed complete by D11 Batch 1 at `evidence/canonical-issue-ledger.md`; root compatibility anchors remain unchanged.
2. `packet-specific evidence` — already satisfied by prior D5/D6 packet-family canonical owners; no re-extraction and no drift.
3. `failed-attempt records` — already satisfied by prior packet-family evidence owners; no re-extraction and no drift.
4. `bounded-stop records` — already satisfied by prior packet-family evidence owners; no re-extraction and no drift.
5. `platform-evidence limitations` — already satisfied by prior D5/D6 owners plus macOS parity owners; no promotion or limitation change.
6. `baseline behavior` — committed complete by D11 Batch 3 at `evidence/baseline-behaviors-and-smoke-scenarios.md`; unchanged compatibility anchors remain in root `05`.
7. `regression obligations` — still satisfied by D11 Batch 1, D11 Batch 3, and packet-local D5/D6 owners; no residual unowned obligation span remains.
8. `smoke scenarios` — committed complete by D11 Batch 3 at `evidence/baseline-behaviors-and-smoke-scenarios.md`; all scenario anchors remain compatible and resolving.
9. `closeout evidence` — committed complete by D11 Batch 2 at `evidence/closeout-and-review-calibration.md` plus packet-local D5/D6 owners; no residual unowned closeout span remains.
10. `current-state projection` — remains satisfied by the existing D6 owner `macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling`; Batch 4 adds navigation only and transfers no ownership or status.

The audits also record that the sole actionable residual live root span before Batch 4 was `## How to read this ledger` (lines `3–15`, `823` bytes, `sha256:80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002`), that Batch 4 extracts exactly that body while keeping receipt/current-state additions navigation-only, and that no genuinely blocked D11 target remains. They further record that D11 was not complete by the candidate alone because independent review, accepted-remediation handling, truthful closeout, and atomic landing still remained at candidate time; with those closeout conditions recorded here and the seven-path Batch 4 artifact now landed atomically in this commit, D11 is now fully complete at the resulting committed HEAD. D12 remains separately unauthorized and blocked by authorization even after D11 completeness, and no D12 work has started or been dispatched.

## Post-review closeout validation

- The rendered prompt/answer copies below are whitespace-clean: any trailing literal spaces or tabs from the imported UTF-8 prompt or answer are rendered as `&#32;` or `&#09;` so this repository-local Markdown artifact stays clean under `git diff --check` while the adjacent base64 blocks preserve the exact bytes.
- At closeout time, the final changed-path fence including untracked files was exactly these seven paths:
  - `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
  - `llm-last-mile/runtime-refactor/evidence/README.md`
  - `llm-last-mile/runtime-refactor/evidence/receipts/README.md`
  - `llm-last-mile/runtime-refactor/index/README.md`
  - `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
  - `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
  - `docs/guidance/2026-08-27-runtime-refactor-d11-reading-model-and-receipt-navigation-chatgpt-pro-review.md`
- Candidate file hashes remained unchanged before and after closeout:
  - `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`: `sha256:fa4381423c7eb9b3d1d8855f6ffd2a51a727dc941cf1811204cf619fb2925508`
  - `llm-last-mile/runtime-refactor/evidence/README.md`: `sha256:bc55fe07b2b4941da1b899046d3efedff08f3fff5419e223d85d9af59391e79a`
  - `llm-last-mile/runtime-refactor/evidence/receipts/README.md`: `sha256:1de062b755df040af799e501669aefac646241beebf9caad410ca65bf140324d`
  - `llm-last-mile/runtime-refactor/index/README.md`: `sha256:5c3b9376ab54c4ed6c2ecce86e9f2bcb546d0c60bd3bd735c7f34f96e2ae5e2f`
  - `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`: `sha256:a1bcf3e7eab175e8e7a6db147432705df217609c4680dc9e96f0777f1fa9d714`
- `git diff --check` passed for the final closeout state.
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/evidence/receipts/README.md` and `git diff --no-index --check /dev/null docs/guidance/2026-08-27-runtime-refactor-d11-reading-model-and-receipt-navigation-chatgpt-pro-review.md` reported no whitespace diagnostics.
- The rendered initial prompt, initial answer, remediation-follow-up prompt, and remediation-follow-up answer below all round-tripped exactly back to the imported UTF-8 artifacts, and the embedded base64 blocks decode exactly back to the imported bytes without silently correcting whitespace, diff fences, or wording.
- The exact 823-byte source-body hash remained unchanged; the root ledger still reports `83` headings below H1, `83` pointer lines, and `0` residual substantive nonheading/nonpointer lines; all legacy root headings remain resolving; the three extracted opaque links and the root-00 opaque link remained byte-stable; Batch 1–3 navigation rows remained unchanged; exactly two Batch 4 navigation rows remained present; exactly one `shared-evidence-navigation` index row remained present; exactly one D11 Batch 4 extraction-ledger row remained present; the current-state D6 owner remained stable; and the complete rollback still explicitly restores the pre-Batch4 authority-boundary sentence.
- Runtime-refactor Markdown link/anchor validation passed with only the four declared opaque skips (`checked=1953 skipped_opaque=4 failures=0`), and tracker/review-record repository-local link checks also passed. The current-state owner remained byte-identical at `sha256:2a362c70fdbf80a2a67643c9bcf6e4419653cbc9f652cc12e17b72b985a6a678`.
- Prior D3/D5–D10 owners, D11 Batch 1–3 owners, and `review-control/` artifacts remained stable outside the exact seven-path fence because the final dirty fence remained limited to the five reviewed candidate paths plus the two closeout paths and no staged changes existed.
- The pre-Batch4 and post-candidate exhaustive audits are both recorded truthfully here, all ten roadmap classifications remain accounted for with no blocked D11 target, and the final state does not imply runtime proof, gate satisfaction, implementation, promotion, remediation authority, dispatch authority, platform work, or successor authority.
- The complete combined remediated candidate plus closeout patch forward/reverse applies cleanly against a clean baseline export and returns byte-identically.
- No files were staged, committed, or pushed during this closeout.

## Atomic rollback instructions

1. To reverse the landed seven-path batch, revert that single atomic batch commit (for example, `git revert --no-commit <batch-commit>`) and verify that only the seven paths above are touched.
2. If you need the equivalent filesystem rollback without reverting the landed commit, restore the five pre-landing tracked paths from parent `3fba14c8a8acb90bc3de40a75035e2a0f5b5fd76` and remove the two files introduced by the seven-path landing: `git restore --source=3fba14c8a8acb90bc3de40a75035e2a0f5b5fd76 -- llm-last-mile/runtime-refactor/05-debug-regression-ledger.md llm-last-mile/runtime-refactor/evidence/README.md llm-last-mile/runtime-refactor/index/README.md llm-last-mile/runtime-refactor/migration/extraction-ledger.md docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md` then `rm -f llm-last-mile/runtime-refactor/evidence/receipts/README.md docs/guidance/2026-08-27-runtime-refactor-d11-reading-model-and-receipt-navigation-chatgpt-pro-review.md`.
3. After either rollback path, confirm together that the exact `823`-byte `How to read this ledger` body is restored to `05-debug-regression-ledger.md` in original location; the pre-Batch4 `evidence/README.md` metadata and the exact rendered authority-boundary sentence are restored; the exact marker body plus the exact receipt/current-state navigation rows are removed; `evidence/receipts/README.md` is removed; the exact `shared-evidence-navigation` global-index row is removed; the exact D11 Batch 4 extraction-ledger row is removed; and the tracker summary/queue/closeout-row additions are reverted.
4. Leave D11 Batches 1–3, prior D3–D10 owners and review artifacts, the existing D6 current-state owner/projection, `index/current.md`, `review-control/`, the root-00 opaque link, and every path outside this seven-path batch unchanged.
5. Documentation rollback must not be rewritten as proof/evidence satisfaction, gate promotion, implementation, remediation authority, dispatch, platform work, or successor authority. Rolling back this atomic seven-path commit makes D11 incomplete again until the full seven-path landing is restored, and D12 remains separately unauthorized and blocked by authorization even after D11 completeness.

## Final verdict

Final D11 Batch 4 reading model/current-state projection/receipt navigation plus exhaustive closeout is complete locally and **APPROVED** after one remediation round and was landed atomically in this commit. Batch 1 canonical issue ledger + A1.2a-WB gate assignment remains committed complete at detached HEAD `d02168ada079b79c3a12709058f4db893bddabdb`, Batch 2 shared `Closeout rule` + residual `Review-process calibration` remains committed complete at detached HEAD `c164ec0c7acb24abcb44211afa26a2630dfa86cf`, and Batch 3 shared `Baseline behaviors` + `Cross-gate smoke scenarios` remains committed complete at detached HEAD `3fba14c8a8acb90bc3de40a75035e2a0f5b5fd76`. Exhaustive coverage proves every substantive live D11 span is accounted for, so D11 is now fully complete at the resulting committed HEAD. This documentation decomposition does not complete evidence, satisfy a gate, authorize implementation, promotion, remediation, dispatch, or successor work, and it does not replace review-control receipts. D12 remains separately unauthorized and blocked by authorization even after D11 completeness; no D12 work has started or been dispatched. No push or successor dispatch authority is granted.

## Preserved review prompts and answers

<details>
<summary>Initial-range review prompt (rendered copy)</summary>

<pre>
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: exact clean detached baseline commit 3fba14c8a8acb90bc3de40a75035e2a0f5b5fd76 with tree 0bcf7ec8b51ada720acaa321934cbfb368cc08c4, plus the complete content-addressed candidate patch below. Candidate patch SHA-256: 2e1e9687f6f0e5991bbde8ac6a2cbf32180ce2604810f31dfc37e75fa1c7fe4e. This patch includes the untracked new file.
Review target: the complete bounded D11 Batch 4 documentation candidate only: extract the remaining reading-model body exactly; add navigation-only receipt and current-state projections; finish shared evidence navigation; preserve all root compatibility anchors; and prove exhaustive D11 source-span accounting without starting D12.

Manifest — exactly these five paths are changed:
1. llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
2. llm-last-mile/runtime-refactor/evidence/README.md
3. llm-last-mile/runtime-refactor/evidence/receipts/README.md (new)
4. llm-last-mile/runtime-refactor/index/README.md
5. llm-last-mile/runtime-refactor/migration/extraction-ledger.md

Supported inputs and behavior: GitHub-Flavored Markdown repository documents. Relative Markdown links and fragments must resolve under ordinary repository Markdown behavior. The four specifically declared historical links are intentionally opaque lexical references: their byte spelling and placement, not filesystem resolution, is the supported invariant. HTML comments delimit an exact extracted-body region and are not rendered prose. Root legacy headings and anchors intentionally remain as compatibility projections until a separately authorized D12 cutover. The candidate is documentation extraction/navigation only; it does not implement runtime behavior, satisfy gates, promote evidence, or authorize dispatch.
Supported platforms and dialects: repository Markdown rendering and link/anchor resolution only; no runtime platform, Rust, script, product behavior, or browser-dialect behavior is in scope. Assume GitHub-style heading slug behavior used by the repository&#x27;s existing validator. The opaque historical references are excluded from resolution but included in lexical preservation checks.

In-scope invariants:
- The exact 823-byte baseline body from root `## How to read this ledger`, SHA-256 80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002, appears byte-for-byte between the named markers in evidence/README.md, while the root heading survives exactly once as a compatibility anchor pointing to that owner.
- The root ledger has exactly 83 headings below H1, exactly 83 canonical/compatibility pointer lines, and zero residual substantive nonheading/nonpointer lines after this candidate; no substantive live D11 span is silently discarded.
- All legacy root headings/anchors remain resolving until D12. Root prefix/suffix reconstruction and the untouched current-state suffix remain byte-stable.
- The three reading-model opaque Markdown link bullets remain byte-identical exactly once in the extracted owner; the fourth opaque link in root 00 remains stable and outside this patch.
- evidence/README.md becomes canonical only for the exact marked reading-model body. Its navigation rows remain non-authoritative. Its existing Batch 1–3 rows are unchanged, and exactly two Batch 4 navigation rows are added: receipt/review-control navigation and a compatibility pointer to the already-canonical D6 current-state owner.
- evidence/receipts/README.md is navigation only, links only to ../../review-control/README.md, copies/reorganizes/normalizes no receipt or review record, and transfers no proof, gate, packet, implementation, dispatch, promotion, or review authority.
- The root current cross-lane projection remains compatibility-only and byte-stable; canonical ownership/status remain at macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling.
- Exactly one shared-evidence-navigation row is added to the global index and exactly one D11 Batch 4 row is added to the extraction ledger.
- The ledger provenance is truthful and its atomic rollback restores the exact root body and pre-Batch4 metadata/navigation, removes the new receipt index and exact new index/ledger rows, preserves D11 Batches 1–3, D3–D10 owners, the existing D6 current-state owner/projection, index/current.md, review-control/, opaque root-00 link, and every path outside the five-path fence.
- All 10 D11 roadmap target categories are accounted for as newly extracted, already satisfied by D11 Batches 1–3, already satisfied by prior D5/D6 owners, or compatibility-only; none is genuinely blocked. This candidate itself does not claim landed completion before review, closeout, and commit.
- Prior D3/D5–D10 canonical owners and review-control artifacts remain stable.
- D12 remains not started, not dispatched, and separately unauthorized.

Required material consequence: a blocking finding must identify a reachable patch-causal defect that loses or changes authoritative source semantics, breaks a supported link/anchor or compatibility projection, creates duplicate or false ownership, misstates authority/status/provenance/rollback, leaves a substantive D11 span unaccounted for, or widens into excluded D12/runtime/platform/review-governance work.
Blocking threshold: P1/P2 only. The finding must be reachable through the supported Markdown behavior, material under the consequence definition, caused or materially worsened by this patch, and within the exact D11 Batch 4 fence. Pre-existing behavior, stylistic preferences, speculative future concerns, and unsupported opaque-link resolution are non-blocking.
Accepted prior findings: none; this is an initial-range review.
Deferred or out-of-scope concerns: D12 root cutover; runtime implementation; Rust/scripts/platform behavior; proof or gate satisfaction; authority/dispatch/promotion; successor work; edits to D3 review-control governance or review-control/; re-extraction or modification of D5/D6 packet-family owners; changes to D11 Batches 1–3; global Markdown normalization; roadmap changes; unrelated docs; the reviewed ZIP; opaque-reference target existence; and pre-existing defects not introduced or worsened by this patch.
Project sources: no broader repository bundle is provided. Review only the complete patch, explicit baseline identifiers, manifest, invariants, validation evidence, and exhaustive coverage audit below. Excluded source is unavailable and must not be inferred. The patch is complete for the five-path candidate and includes its new untracked file.

Validation evidence:
STATUS
 M llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
 M llm-last-mile/runtime-refactor/evidence/README.md
 M llm-last-mile/runtime-refactor/index/README.md
 M llm-last-mile/runtime-refactor/migration/extraction-ledger.md
?? llm-last-mile/runtime-refactor/evidence/receipts/README.md

Dirty fence validated: exact five paths dirty; nothing staged.

git diff --check: clean

git diff --no-index --check /dev/null receipts README: clean

Root prefix/suffix byte stability and exact reconstruction validated.

Evidence README marker body bytes/hash match exact 823-byte source artifact.

Root audit counts validated: headings=83, pointer_lines=83, residual=0

Root heading anchors unique; legacy how-to-read anchor preserved.

Opaque links validated: three reading-model link bullets preserved exactly once in owner; 00 opaque link byte-stable.

Evidence README metadata updated truthfully; Batch1-3 rows preserved; exactly one receipt row and one current-state row added.

Receipts README restricted to a single review-control README link with correct boundary statements.

macOS parity owner hash stable: 2a362c70fdbf80a2a67643c9bcf6e4419653cbc9f652cc12e17b72b985a6a678

Exactly one shared-evidence-navigation index row and one D11 Batch4 ledger row present.

Hash stability validated for 256 prior owner/review-control artifacts outside the five-path fence.

Markdown link/fragment scan passed across runtime-refactor: resolved=1953, skipped_opaque=4, failures=0

Patch written: /private/tmp/d11-reading-receipts-candidate.patch sha256=2e1e9687f6f0e5991bbde8ac6a2cbf32180ce2604810f31dfc37e75fa1c7fe4e

Forward and reverse patch apply succeeded against clean baseline exports and returned to byte-identical baseline.

Post-candidate audit written: /private/tmp/d11-exhaustive-coverage-audit-post-candidate.txt sha256=e3a3955731d99f5ffc1332eedc95055dcdcf85e2d6ebc5f5b85c74a3b03c89f5

Exhaustive post-candidate coverage audit (SHA-256 e3a3955731d99f5ffc1332eedc95055dcdcf85e2d6ebc5f5b85c74a3b03c89f5):
D11 exhaustive live coverage audit after final Batch 4 candidate
Date: 2026-08-27
Worktree: /Users/spensermcconnell/.codex/worktrees/bae3/substrate
Detached baseline HEAD: 3fba14c8a8acb90bc3de40a75035e2a0f5b5fd76
Baseline tree: 0bcf7ec8b51ada720acaa321934cbfb368cc08c4
Candidate status: exact five-path docs-only fence; nothing staged
Root: llm-last-mile/runtime-refactor/05-debug-regression-ledger.md

Mechanical root audit
- Root headings below H1: 83
- Canonical/compatibility pointer lines: 83
- Residual substantive nonheading/nonpointer lines: 0
- Legacy `## How to read this ledger` root anchor remains present exactly once and now points to `evidence/README.md#how-to-read-this-ledger` as a compatibility-only projection.
- Current cross-lane root projection remains compatibility-only and byte-stable in the preserved root suffix; canonical ownership stays at `macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling`.

Roadmap target classification
1. canonical issue ledger — still satisfied by D11 Batch 1 at `evidence/canonical-issue-ledger.md`; unchanged compatibility anchors remain in root 05.
2. packet-specific evidence — still satisfied by prior D5/D6 packet-family canonical owners; no re-extraction and no drift.
3. failed-attempt records — still satisfied by prior packet-family evidence owners; no re-extraction and no drift.
4. bounded-stop records — still satisfied by prior packet-family evidence owners; no re-extraction and no drift.
5. platform-evidence limitations — still satisfied by prior D5/D6 owners plus macOS parity owners; no promotion or limitation change.
6. baseline behavior — still satisfied by D11 Batch 3 at `evidence/baseline-behaviors-and-smoke-scenarios.md`; unchanged compatibility anchors remain in root 05.
7. regression obligations — still satisfied by D11 Batch 1, D11 Batch 3, and packet-local D5/D6 owners; no residual unowned obligation span remains.
8. smoke scenarios — still satisfied by D11 Batch 3 at `evidence/baseline-behaviors-and-smoke-scenarios.md`; all scenario anchors remain compatible and resolving.
9. closeout evidence — still satisfied by D11 Batch 2 at `evidence/closeout-and-review-calibration.md` plus packet-local D5/D6 owners; no residual unowned closeout span remains.
10. current-state projection — remains satisfied by the existing D6 owner `macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling`; Batch 4 adds navigation only and transfers no ownership or status.

Batch 4 completion classification
- Actionable reading-model span extracted exactly: lines 3–15, 823 bytes, sha256 80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002, now preserved between `exact-extracted-body:how-to-read-this-ledger:start/end` in `evidence/README.md`.
- Receipt boundary satisfied as navigation only: `evidence/receipts/README.md` links only to `../../review-control/README.md`, copies no receipt, and claims no proof/authority transfer.
- Shared evidence navigation completed: `shared-evidence-navigation` now exists exactly once in `index/README.md` and covers the exact reading-model canonical scope plus non-authoritative navigation for D11 owners, the already-canonical current-state projection, and the receipt index.
- No blocked D11 roadmap target remains.
- D11 is not landed or complete by this candidate alone: independent review, any accepted remediation, re-review, truthful closeout, and atomic landing remain outside this local candidate.
- D12 remains not started, not dispatched, and separately unauthorized; this candidate grants no successor, implementation, proof/gate, remediation, dispatch, promotion, or platform authority.

Complete candidate patch (SHA-256 2e1e9687f6f0e5991bbde8ac6a2cbf32180ce2604810f31dfc37e75fa1c7fe4e):
```diff
diff --git a/llm-last-mile/runtime-refactor/05-debug-regression-ledger.md b/llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
index 11a9046ff..0f763da2b 100644
--- a/llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
+++ b/llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
@@ -2,16 +2,7 @@
&#32;
 ## How to read this ledger
&#32;
-- **Resolved baseline** means a specific observed failure has a trustworthy fix or live proof that must not regress. It does **not** promote the surrounding architecture seam.
-- **Partially resolved** means a narrow behavior works while the target ownership model remains wrong or unproven.
-- **Unresolved** means the target behavior lacks an implementation and proof gate.
-- Historical diagnoses are retained only when they define a permanent negative or regression test.
-
-Primary source memos:
-
-- [`../../RUN_WORLD_TASK_DEBUG_CANONICAL.md`](../../RUN_WORLD_TASK_DEBUG_CANONICAL.md)
-- [`../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md`](../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md)
-- [`../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md`](../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md)
+Compatibility anchor only; canonical content: [`evidence/README.md#how-to-read-this-ledger`](evidence/README.md#how-to-read-this-ledger).
&#32;
 ## Canonical issue ledger
&#32;
diff --git a/llm-last-mile/runtime-refactor/evidence/README.md b/llm-last-mile/runtime-refactor/evidence/README.md
index c5f5742ba..43bb0afe8 100644
--- a/llm-last-mile/runtime-refactor/evidence/README.md
+++ b/llm-last-mile/runtime-refactor/evidence/README.md
@@ -1,21 +1,40 @@
 **Kind:** evidence index
 **Stable ID:** `shared-evidence-navigation`
-**Canonical for:** no authority; extracted evidence-owner navigation only
-**Status:** non-authoritative navigation
-**Authority scope:** none; follow the linked canonical owners and packet-family records
-**Source span:** D11 Batch 1 assembly only
-**Supersedes:** none
+**Canonical for:** exact extracted `How to read this ledger` body; non-authoritative navigation for extracted evidence owners, the existing current-state projection, and receipt/review-control lookup
+**Status:** canonical extracted body with non-authoritative navigation
+**Authority scope:** exact extracted reading/interpretation source body only; follow the linked canonical owners and review-control records for everything else
+**Source span:** [`../05-debug-regression-ledger.md#how-to-read-this-ledger`](../05-debug-regression-ledger.md#how-to-read-this-ledger), D11 Batch 4 baseline lines 3–15 (823 bytes)
+**Baseline span SHA-256:** `80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002`
+**Supersedes:** canonical ownership of that exact source body only; navigation remains non-authoritative
 **Superseded by:** none
-**Projection consumers:** [`../index/README.md`](../index/README.md)
+**Projection consumers:** [`../index/README.md`](../index/README.md), [`../05-debug-regression-ledger.md#how-to-read-this-ledger`](../05-debug-regression-ledger.md#how-to-read-this-ledger)
&#32;
 # Runtime evidence index
&#32;
-&gt; **Authority boundary:** This file is navigation only. It does not authorize implementation, close a regression gate, satisfy a proof obligation, promote evidence into authority, or replace the linked canonical owners.
+&gt; **Authority boundary:** Only the exact extracted body between the named markers below is canonical here. All navigation in this file remains non-authoritative. It does not authorize implementation, close a regression gate, satisfy a proof obligation, promote evidence into authority, or replace the linked canonical owners.
 &gt;
 &gt; **Receipt boundary:** `review-control/` remains the canonical receipt, review-record, and closeout artifact store. This file does not duplicate, normalize, or replace those records.
&#32;
+&lt;!-- exact-extracted-body:how-to-read-this-ledger:start --&gt;
+## How to read this ledger
+
+- **Resolved baseline** means a specific observed failure has a trustworthy fix or live proof that must not regress. It does **not** promote the surrounding architecture seam.
+- **Partially resolved** means a narrow behavior works while the target ownership model remains wrong or unproven.
+- **Unresolved** means the target behavior lacks an implementation and proof gate.
+- Historical diagnoses are retained only when they define a permanent negative or regression test.
+
+Primary source memos:
+
+- [`../../RUN_WORLD_TASK_DEBUG_CANONICAL.md`](../../RUN_WORLD_TASK_DEBUG_CANONICAL.md)
+- [`../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md`](../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md)
+- [`../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md`](../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md)
+
+&lt;!-- exact-extracted-body:how-to-read-this-ledger:end --&gt;
+
 | Evidence component | Canonical owner | Current extracted scope |
 |---|---|---|
 | Batch 1 — canonical issue ledger | [`canonical-issue-ledger.md#canonical-issue-ledger`](canonical-issue-ledger.md#canonical-issue-ledger) | Full extracted `## Canonical issue ledger` table plus the preserved `### A1.2a-WB gate assignment` body; exact issue IDs, chronology, limitations, obligations, literals, hashes, commands, and negative requirements remain unchanged. |
 | Batch 2 — closeout rule + review-process calibration | [`closeout-and-review-calibration.md`](closeout-and-review-calibration.md) | Exact extracted [`## Closeout rule`](closeout-and-review-calibration.md#closeout-rule) and [`## Review-process calibration`](closeout-and-review-calibration.md#review-process-calibration) bodies with separate noncontiguous source provenance; ordered bullets, literals, IDs, negative requirements, exceptions, review-governance reference-only boundaries, and the no-remediation/no-proof-result-change statement remain unchanged. |
 | Batch 3 — baseline behaviors + cross-gate smoke scenarios | [`baseline-behaviors-and-smoke-scenarios.md`](baseline-behaviors-and-smoke-scenarios.md) | Exact extracted [`## Baseline behaviors that all tracks preserve`](baseline-behaviors-and-smoke-scenarios.md#baseline-behaviors-that-all-tracks-preserve) and [`## Cross-gate smoke scenarios`](baseline-behaviors-and-smoke-scenarios.md#cross-gate-smoke-scenarios) bodies with separate exact source provenance preserved between distinct boundary markers; Gate IDs, statuses, table order/cells, ordered scenario steps, deterministic parked-successor sequence, Covers lines, and fail-closed/exception boundaries remain unchanged. |
+| Batch 4 — receipt/review-control navigation | [`receipts/README.md`](receipts/README.md) | Navigation only; follow the receipt index to the canonical `review-control/README.md` store. No receipt or review record is copied, reorganized into new evidence, or normalized here, and process/review evidence does not replace product proof, gate satisfaction, or packet/implementation/dispatch/promotion authority. |
+| Batch 4 — current-state projection compatibility pointer | [`../macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling`](../macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling) | The root `05` current cross-lane projection remains compatibility-only; canonical ownership and status stay with the existing D6 macOS-parity evidence owner, and no ownership or status transfer occurs here. |
diff --git a/llm-last-mile/runtime-refactor/index/README.md b/llm-last-mile/runtime-refactor/index/README.md
index 4119d81d9..980330a25 100644
--- a/llm-last-mile/runtime-refactor/index/README.md
+++ b/llm-last-mile/runtime-refactor/index/README.md
@@ -32,6 +32,7 @@
 | `shared-target-architecture` | architecture index | [`architecture/README.md`](../architecture/README.md) | `non-authoritative navigation for the extracted executive target decision, authority map, numbered invariants, and stable review question` | Supersedes only root-canonical ownership of the extracted shared architecture spans; packet-family-local D5/D6 forwarders remain at their existing owners. | [`01`](../01-target-architecture.md#executive-decision), [`01`](../01-target-architecture.md#authority-map), [`01`](../01-target-architecture.md#non-negotiable-invariants), [`01`](../01-target-architecture.md#review-question) |
 | `shared-seam-crosswalk` | seam index | [`seams/README.md`](../seams/README.md) | `canonical shared seam-crosswalk rules plus extracted seam-family navigation for host/session, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation` | Supersedes only the extracted root reading-rule/classification spans and the extracted host/session authority, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation family rows; A0 and the existing D6 `HostSessionAuthority`, `WorldWorkerMessagingProtocol`, and `ObligationLedger` compatibility rows remain at their current owners. | [`02`](../02-seam-crosswalk.md#reading-rule), [`02`](../02-seam-crosswalk.md#a-host-authority-and-ingress), [`02`](../02-seam-crosswalk.md#b-dispatch-policy-receipts-and-retained-runtime), [`02`](../02-seam-crosswalk.md#c-obligations-and-host-re-engagement), [`02`](../02-seam-crosswalk.md#d-uaa-realization-projection-and-side-effect-mediation), [`02`](../02-seam-crosswalk.md#classification-consequences) |
 | `shared-slice-map` | slice index | [`slices/README.md`](../slices/README.md) | `canonical shared sequencing/dependency and slice-closeout owner plus non-authoritative D9 Track A–E navigation and A1/A1.4 projection index` | Supersedes only root-canonical ownership of the extracted shared sequencing/closeout spans plus the extracted Track A–E and A1/A1.4 projection spans; controlling schedule authority remains with decisions, packets, gates, and `current.md`. | [`03`](../03-phase-slice-map.md#sequencing-rules), [`03`](../03-phase-slice-map.md#track-a--authority-and-surface-neutrality), [`03`](../03-phase-slice-map.md#a1-bounded-packet-decomposition), [`03`](../03-phase-slice-map.md#track-b--world-dispatch-receipts-supervision-and-cancel), [`03`](../03-phase-slice-map.md#track-c--obligations-inbox-auto-attach-and-router-attach), [`03`](../03-phase-slice-map.md#track-d--uaa-execution-envelope-and-side-effect-mediation), [`03`](../03-phase-slice-map.md#track-e--dispatch-scoped-policy-narrowing-and-config-projection), [`03`](../03-phase-slice-map.md#slice-closeout-minimum) |
+| `shared-evidence-navigation` | evidence index | [`evidence/README.md`](../evidence/README.md) | `canonical exact extracted How to read this ledger body plus non-authoritative navigation for D11 evidence owners, the existing D6 current-state projection, and the receipt index` | Supersedes only root-canonical ownership of the extracted `## How to read this ledger` body; the receipt index and current-state pointer remain navigation-only and do not transfer authority or change status. | [`05`](../05-debug-regression-ledger.md#how-to-read-this-ledger) |
 | Canonical issue ledger | evidence/regression | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) | `canonical extracted owner for the full issue/regression-obligation ledger table plus the preserved A1.2a-WB gate-assignment body` | Supersedes only root-canonical ownership of the extracted `## Canonical issue ledger` body; the legacy `## Canonical issue ledger` and `### A1.2a-WB gate assignment` headings remain compatibility anchors until D12. | [`05`](../05-debug-regression-ledger.md#canonical-issue-ledger), [`05`](../05-debug-regression-ledger.md#a12a-wb-gate-assignment) |
 | Closeout rule + Review-process calibration | evidence/regression | [`evidence/closeout-and-review-calibration.md`](../evidence/closeout-and-review-calibration.md) | `canonical extracted compound owner for the exact noncontiguous Closeout rule and Review-process calibration evidence/regression bodies` | Supersedes only root-canonical ownership of the extracted `## Closeout rule` and `## Review-process calibration` bodies; the root headings remain compatibility anchors, and the development-review contract remains reference-only without reopening D3. | [`05`](../05-debug-regression-ledger.md#closeout-rule), [`05`](../05-debug-regression-ledger.md#review-process-calibration) |
 | Baseline behaviors + Cross-gate smoke scenarios | evidence/regression | [`evidence/baseline-behaviors-and-smoke-scenarios.md`](../evidence/baseline-behaviors-and-smoke-scenarios.md) | `canonical extracted compound owner for the exact baseline-behavior table and cross-gate smoke-scenario bodies` | Supersedes only root-canonical ownership of the extracted `## Baseline behaviors that all tracks preserve` and `## Cross-gate smoke scenarios` bodies; the legacy root headings and scenario subheadings remain compatibility anchors until D12, and `review-control/` remains the canonical receipt store. | [`05`](../05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve), [`05`](../05-debug-regression-ledger.md#cross-gate-smoke-scenarios) |
diff --git a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
index 5f5d5f504..7f8dac309 100644
--- a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
+++ b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
@@ -201,3 +201,4 @@ This ledger records content-preserving authority transfers while retaining requi
 | D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Review-process calibration | [`review-process-calibration`](../05-debug-regression-ledger.md#review-process-calibration) | lines 332–353 (1382 bytes) | `284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e` | [`../evidence/closeout-and-review-calibration.md`](../evidence/closeout-and-review-calibration.md) | evidence/regression | canonical compound destination; source compatibility anchor; separate noncontiguous provenance preserved | none; exact 1382-byte source body preserved between the `review-process-calibration` owner boundary markers in the compound evidence owner; the development-review contract relationship remains reference-only and does not reopen D3 | restore the exact 347-byte source body at `05-debug-regression-ledger.md#closeout-rule` and the exact 1382-byte source body at `05-debug-regression-ledger.md#review-process-calibration` in original order; remove `evidence/closeout-and-review-calibration.md`; remove the sole Batch-2 row from `evidence/README.md`; remove the exact `Closeout rule + Review-process calibration` row from `index/README.md`; remove both D11 ledger entries for this batch; leave Batch 1 plus D3–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
 | D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Baseline behaviors that all tracks preserve | [`baseline-behaviors-that-all-tracks-preserve`](../05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve) | lines 80–92 (3644 bytes) | `5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132` | [`../evidence/baseline-behaviors-and-smoke-scenarios.md`](../evidence/baseline-behaviors-and-smoke-scenarios.md) | evidence/regression | canonical compound destination; source compatibility anchor; separate exact-body provenance preserved | paired family atomicity with `Cross-gate smoke scenarios`; exact 3644-byte source body preserved between the `baseline-behaviors` owner boundary markers and not represented as one contiguous extraction with the later smoke-scenario body | restore the exact 3644-byte source body at `05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve` and the exact 11714-byte source body at `05-debug-regression-ledger.md#cross-gate-smoke-scenarios` in original order; remove `evidence/baseline-behaviors-and-smoke-scenarios.md`; remove the sole Batch-3 row from `evidence/README.md`; remove the exact `Baseline behaviors + Cross-gate smoke scenarios` row from `index/README.md`; remove both D11 ledger entries for this batch; leave Batch 1, Batch 2, D3–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
 | D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Cross-gate smoke scenarios | [`cross-gate-smoke-scenarios`](../05-debug-regression-ledger.md#cross-gate-smoke-scenarios) | lines 93–192 (11714 bytes) | `586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85` | [`../evidence/baseline-behaviors-and-smoke-scenarios.md`](../evidence/baseline-behaviors-and-smoke-scenarios.md) | evidence/regression | canonical compound destination; source compatibility anchor; separate exact-body provenance preserved | paired family atomicity with `Baseline behaviors that all tracks preserve`; exact 11714-byte source body preserved between the `cross-gate-smoke-scenarios` owner boundary markers with ordered scenario subheadings, deterministic parked-successor sequence, and Covers lines internal to the preserved body | restore the exact 3644-byte source body at `05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve` and the exact 11714-byte source body at `05-debug-regression-ledger.md#cross-gate-smoke-scenarios` in original order; remove `evidence/baseline-behaviors-and-smoke-scenarios.md`; remove the sole Batch-3 row from `evidence/README.md`; remove the exact `Baseline behaviors + Cross-gate smoke scenarios` row from `index/README.md`; remove both D11 ledger entries for this batch; leave Batch 1, Batch 2, D3–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
+| D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | How to read this ledger | [`how-to-read-this-ledger`](../05-debug-regression-ledger.md#how-to-read-this-ledger) | lines 3–15 (823 bytes) | `80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002` | [`../evidence/README.md`](../evidence/README.md) | evidence index | canonical destination for the exact reading/interpretation body only; receipt/current-state rows remain non-authoritative navigation | none; exact 823-byte source body preserved between the `exact-extracted-body:how-to-read-this-ledger:start/end` markers in `evidence/README.md`; metadata and Batch 4 navigation rows remain outside the preserved body, and the three opaque Markdown links remain byte-identical | restore the exact 823-byte root body at `05-debug-regression-ledger.md#how-to-read-this-ledger`; restore the pre-Batch4 `evidence/README.md` metadata and remove the exact marker body plus the exact receipt/current-state navigation rows; remove `evidence/receipts/README.md`; remove the exact `shared-evidence-navigation` row from `index/README.md`; remove this D11 ledger entry; preserve D11 Batches 1–3, D3–D10 owners, the existing D6 current-state owner/projection, `index/current.md`, `review-control/`, the opaque `00` link, and every path outside the five-path fence unchanged |
diff --git a/llm-last-mile/runtime-refactor/evidence/receipts/README.md b/llm-last-mile/runtime-refactor/evidence/receipts/README.md
new file mode 100644
index 000000000..6b17b84b8
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/evidence/receipts/README.md
@@ -0,0 +1,13 @@
+**Kind:** receipt index
+**Status:** non-authoritative navigation
+**Canonical for:** no authority; receipt and review-record navigation only
+
+# Receipt and review-control navigation
+
+&gt; **Authority boundary:** This file is navigation only. It copies, reorganizes, and normalizes no receipt or review record.
+&gt;
+&gt; **Evidence boundary:** Process and review evidence do not replace product proof, gate satisfaction, or packet/implementation/dispatch/promotion authority.
+
+Follow the canonical receipt and review-record store: [`../../review-control/README.md`](../../review-control/README.md).
+
+That store remains the exact location for receipt artifacts, review-cycle records, closeout review metadata, and interpretation guidance. This file introduces no new canonical receipt body and transfers no ownership.
```

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return findings first by severity with exact file/section, supported-input reachability, violated invariant, material consequence, and patch causality. State explicitly when there are no qualifying findings. End with exactly one verdict token on its own line: APPROVED or REQUEST_CHANGES.
</pre>

</details>

<details>
<summary>Initial-range review prompt exact bytes (base64 UTF-8)</summary>

```text
UmV2aWV3IG1vZGU6IGluaXRpYWwtcmFuZ2UKUmV2aWV3IGNvbnRleHQ6IGluZGVwZW5kZW50IGZy
ZXNoIGNvbnZlcnNhdGlvbgpSZXZpZXcgYm91bmRhcnk6IGV4YWN0IGNsZWFuIGRldGFjaGVkIGJh
c2VsaW5lIGNvbW1pdCAzZmJhMTRjOGE4YWNiOTBiYzNkZTQwYTc1MDM1ZTJhMGY1YjVmZDc2IHdp
dGggdHJlZSAwYmNmN2VjOGI1MWFkYTcyMGFjYWEzMjE5MzRjYmZiMzY4Y2MwOGM0LCBwbHVzIHRo
ZSBjb21wbGV0ZSBjb250ZW50LWFkZHJlc3NlZCBjYW5kaWRhdGUgcGF0Y2ggYmVsb3cuIENhbmRp
ZGF0ZSBwYXRjaCBTSEEtMjU2OiAyZTFlOTY4N2Y2ZjBlNTk5MWJiZGU4YWM2YTJjYmYzMjE4MGNl
MjYwNDgxMGYzMWRmYzM3ZTc1ZmExYzdmZTRlLiBUaGlzIHBhdGNoIGluY2x1ZGVzIHRoZSB1bnRy
YWNrZWQgbmV3IGZpbGUuClJldmlldyB0YXJnZXQ6IHRoZSBjb21wbGV0ZSBib3VuZGVkIEQxMSBC
YXRjaCA0IGRvY3VtZW50YXRpb24gY2FuZGlkYXRlIG9ubHk6IGV4dHJhY3QgdGhlIHJlbWFpbmlu
ZyByZWFkaW5nLW1vZGVsIGJvZHkgZXhhY3RseTsgYWRkIG5hdmlnYXRpb24tb25seSByZWNlaXB0
IGFuZCBjdXJyZW50LXN0YXRlIHByb2plY3Rpb25zOyBmaW5pc2ggc2hhcmVkIGV2aWRlbmNlIG5h
dmlnYXRpb247IHByZXNlcnZlIGFsbCByb290IGNvbXBhdGliaWxpdHkgYW5jaG9yczsgYW5kIHBy
b3ZlIGV4aGF1c3RpdmUgRDExIHNvdXJjZS1zcGFuIGFjY291bnRpbmcgd2l0aG91dCBzdGFydGlu
ZyBEMTIuCgpNYW5pZmVzdCDigJQgZXhhY3RseSB0aGVzZSBmaXZlIHBhdGhzIGFyZSBjaGFuZ2Vk
OgoxLiBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDUtZGVidWctcmVncmVzc2lvbi1s
ZWRnZXIubWQKMi4gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2V2aWRlbmNlL1JFQURN
RS5tZAozLiBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZXZpZGVuY2UvcmVjZWlwdHMv
UkVBRE1FLm1kIChuZXcpCjQuIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9S
RUFETUUubWQKNS4gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRy
YWN0aW9uLWxlZGdlci5tZAoKU3VwcG9ydGVkIGlucHV0cyBhbmQgYmVoYXZpb3I6IEdpdEh1Yi1G
bGF2b3JlZCBNYXJrZG93biByZXBvc2l0b3J5IGRvY3VtZW50cy4gUmVsYXRpdmUgTWFya2Rvd24g
bGlua3MgYW5kIGZyYWdtZW50cyBtdXN0IHJlc29sdmUgdW5kZXIgb3JkaW5hcnkgcmVwb3NpdG9y
eSBNYXJrZG93biBiZWhhdmlvci4gVGhlIGZvdXIgc3BlY2lmaWNhbGx5IGRlY2xhcmVkIGhpc3Rv
cmljYWwgbGlua3MgYXJlIGludGVudGlvbmFsbHkgb3BhcXVlIGxleGljYWwgcmVmZXJlbmNlczog
dGhlaXIgYnl0ZSBzcGVsbGluZyBhbmQgcGxhY2VtZW50LCBub3QgZmlsZXN5c3RlbSByZXNvbHV0
aW9uLCBpcyB0aGUgc3VwcG9ydGVkIGludmFyaWFudC4gSFRNTCBjb21tZW50cyBkZWxpbWl0IGFu
IGV4YWN0IGV4dHJhY3RlZC1ib2R5IHJlZ2lvbiBhbmQgYXJlIG5vdCByZW5kZXJlZCBwcm9zZS4g
Um9vdCBsZWdhY3kgaGVhZGluZ3MgYW5kIGFuY2hvcnMgaW50ZW50aW9uYWxseSByZW1haW4gYXMg
Y29tcGF0aWJpbGl0eSBwcm9qZWN0aW9ucyB1bnRpbCBhIHNlcGFyYXRlbHkgYXV0aG9yaXplZCBE
MTIgY3V0b3Zlci4gVGhlIGNhbmRpZGF0ZSBpcyBkb2N1bWVudGF0aW9uIGV4dHJhY3Rpb24vbmF2
aWdhdGlvbiBvbmx5OyBpdCBkb2VzIG5vdCBpbXBsZW1lbnQgcnVudGltZSBiZWhhdmlvciwgc2F0
aXNmeSBnYXRlcywgcHJvbW90ZSBldmlkZW5jZSwgb3IgYXV0aG9yaXplIGRpc3BhdGNoLgpTdXBw
b3J0ZWQgcGxhdGZvcm1zIGFuZCBkaWFsZWN0czogcmVwb3NpdG9yeSBNYXJrZG93biByZW5kZXJp
bmcgYW5kIGxpbmsvYW5jaG9yIHJlc29sdXRpb24gb25seTsgbm8gcnVudGltZSBwbGF0Zm9ybSwg
UnVzdCwgc2NyaXB0LCBwcm9kdWN0IGJlaGF2aW9yLCBvciBicm93c2VyLWRpYWxlY3QgYmVoYXZp
b3IgaXMgaW4gc2NvcGUuIEFzc3VtZSBHaXRIdWItc3R5bGUgaGVhZGluZyBzbHVnIGJlaGF2aW9y
IHVzZWQgYnkgdGhlIHJlcG9zaXRvcnkncyBleGlzdGluZyB2YWxpZGF0b3IuIFRoZSBvcGFxdWUg
aGlzdG9yaWNhbCByZWZlcmVuY2VzIGFyZSBleGNsdWRlZCBmcm9tIHJlc29sdXRpb24gYnV0IGlu
Y2x1ZGVkIGluIGxleGljYWwgcHJlc2VydmF0aW9uIGNoZWNrcy4KCkluLXNjb3BlIGludmFyaWFu
dHM6Ci0gVGhlIGV4YWN0IDgyMy1ieXRlIGJhc2VsaW5lIGJvZHkgZnJvbSByb290IGAjIyBIb3cg
dG8gcmVhZCB0aGlzIGxlZGdlcmAsIFNIQS0yNTYgODBmNGI3NTNkNmU1NzQyMDJiNTI3NTg5ZDMz
MDIxYzczNWE3NmUyYzY1NjNmYzVkY2U4OTBkZjUyNGM5MzAwMiwgYXBwZWFycyBieXRlLWZvci1i
eXRlIGJldHdlZW4gdGhlIG5hbWVkIG1hcmtlcnMgaW4gZXZpZGVuY2UvUkVBRE1FLm1kLCB3aGls
ZSB0aGUgcm9vdCBoZWFkaW5nIHN1cnZpdmVzIGV4YWN0bHkgb25jZSBhcyBhIGNvbXBhdGliaWxp
dHkgYW5jaG9yIHBvaW50aW5nIHRvIHRoYXQgb3duZXIuCi0gVGhlIHJvb3QgbGVkZ2VyIGhhcyBl
eGFjdGx5IDgzIGhlYWRpbmdzIGJlbG93IEgxLCBleGFjdGx5IDgzIGNhbm9uaWNhbC9jb21wYXRp
YmlsaXR5IHBvaW50ZXIgbGluZXMsIGFuZCB6ZXJvIHJlc2lkdWFsIHN1YnN0YW50aXZlIG5vbmhl
YWRpbmcvbm9ucG9pbnRlciBsaW5lcyBhZnRlciB0aGlzIGNhbmRpZGF0ZTsgbm8gc3Vic3RhbnRp
dmUgbGl2ZSBEMTEgc3BhbiBpcyBzaWxlbnRseSBkaXNjYXJkZWQuCi0gQWxsIGxlZ2FjeSByb290
IGhlYWRpbmdzL2FuY2hvcnMgcmVtYWluIHJlc29sdmluZyB1bnRpbCBEMTIuIFJvb3QgcHJlZml4
L3N1ZmZpeCByZWNvbnN0cnVjdGlvbiBhbmQgdGhlIHVudG91Y2hlZCBjdXJyZW50LXN0YXRlIHN1
ZmZpeCByZW1haW4gYnl0ZS1zdGFibGUuCi0gVGhlIHRocmVlIHJlYWRpbmctbW9kZWwgb3BhcXVl
IE1hcmtkb3duIGxpbmsgYnVsbGV0cyByZW1haW4gYnl0ZS1pZGVudGljYWwgZXhhY3RseSBvbmNl
IGluIHRoZSBleHRyYWN0ZWQgb3duZXI7IHRoZSBmb3VydGggb3BhcXVlIGxpbmsgaW4gcm9vdCAw
MCByZW1haW5zIHN0YWJsZSBhbmQgb3V0c2lkZSB0aGlzIHBhdGNoLgotIGV2aWRlbmNlL1JFQURN
RS5tZCBiZWNvbWVzIGNhbm9uaWNhbCBvbmx5IGZvciB0aGUgZXhhY3QgbWFya2VkIHJlYWRpbmct
bW9kZWwgYm9keS4gSXRzIG5hdmlnYXRpb24gcm93cyByZW1haW4gbm9uLWF1dGhvcml0YXRpdmUu
IEl0cyBleGlzdGluZyBCYXRjaCAx4oCTMyByb3dzIGFyZSB1bmNoYW5nZWQsIGFuZCBleGFjdGx5
IHR3byBCYXRjaCA0IG5hdmlnYXRpb24gcm93cyBhcmUgYWRkZWQ6IHJlY2VpcHQvcmV2aWV3LWNv
bnRyb2wgbmF2aWdhdGlvbiBhbmQgYSBjb21wYXRpYmlsaXR5IHBvaW50ZXIgdG8gdGhlIGFscmVh
ZHktY2Fub25pY2FsIEQ2IGN1cnJlbnQtc3RhdGUgb3duZXIuCi0gZXZpZGVuY2UvcmVjZWlwdHMv
UkVBRE1FLm1kIGlzIG5hdmlnYXRpb24gb25seSwgbGlua3Mgb25seSB0byAuLi8uLi9yZXZpZXct
Y29udHJvbC9SRUFETUUubWQsIGNvcGllcy9yZW9yZ2FuaXplcy9ub3JtYWxpemVzIG5vIHJlY2Vp
cHQgb3IgcmV2aWV3IHJlY29yZCwgYW5kIHRyYW5zZmVycyBubyBwcm9vZiwgZ2F0ZSwgcGFja2V0
LCBpbXBsZW1lbnRhdGlvbiwgZGlzcGF0Y2gsIHByb21vdGlvbiwgb3IgcmV2aWV3IGF1dGhvcml0
eS4KLSBUaGUgcm9vdCBjdXJyZW50IGNyb3NzLWxhbmUgcHJvamVjdGlvbiByZW1haW5zIGNvbXBh
dGliaWxpdHktb25seSBhbmQgYnl0ZS1zdGFibGU7IGNhbm9uaWNhbCBvd25lcnNoaXAvc3RhdHVz
IHJlbWFpbiBhdCBtYWNvcy1kZXYtcGFyaXR5L2V2aWRlbmNlLXJlZ3Jlc3Npb24ubWQjY3VycmVu
dC1jcm9zcy1sYW5lLXJlZ3Jlc3Npb24tbGVkZ2VyLTIwMjYtMDgtMjAtY29udHJvbGxpbmcuCi0g
RXhhY3RseSBvbmUgc2hhcmVkLWV2aWRlbmNlLW5hdmlnYXRpb24gcm93IGlzIGFkZGVkIHRvIHRo
ZSBnbG9iYWwgaW5kZXggYW5kIGV4YWN0bHkgb25lIEQxMSBCYXRjaCA0IHJvdyBpcyBhZGRlZCB0
byB0aGUgZXh0cmFjdGlvbiBsZWRnZXIuCi0gVGhlIGxlZGdlciBwcm92ZW5hbmNlIGlzIHRydXRo
ZnVsIGFuZCBpdHMgYXRvbWljIHJvbGxiYWNrIHJlc3RvcmVzIHRoZSBleGFjdCByb290IGJvZHkg
YW5kIHByZS1CYXRjaDQgbWV0YWRhdGEvbmF2aWdhdGlvbiwgcmVtb3ZlcyB0aGUgbmV3IHJlY2Vp
cHQgaW5kZXggYW5kIGV4YWN0IG5ldyBpbmRleC9sZWRnZXIgcm93cywgcHJlc2VydmVzIEQxMSBC
YXRjaGVzIDHigJMzLCBEM+KAk0QxMCBvd25lcnMsIHRoZSBleGlzdGluZyBENiBjdXJyZW50LXN0
YXRlIG93bmVyL3Byb2plY3Rpb24sIGluZGV4L2N1cnJlbnQubWQsIHJldmlldy1jb250cm9sLywg
b3BhcXVlIHJvb3QtMDAgbGluaywgYW5kIGV2ZXJ5IHBhdGggb3V0c2lkZSB0aGUgZml2ZS1wYXRo
IGZlbmNlLgotIEFsbCAxMCBEMTEgcm9hZG1hcCB0YXJnZXQgY2F0ZWdvcmllcyBhcmUgYWNjb3Vu
dGVkIGZvciBhcyBuZXdseSBleHRyYWN0ZWQsIGFscmVhZHkgc2F0aXNmaWVkIGJ5IEQxMSBCYXRj
aGVzIDHigJMzLCBhbHJlYWR5IHNhdGlzZmllZCBieSBwcmlvciBENS9ENiBvd25lcnMsIG9yIGNv
bXBhdGliaWxpdHktb25seTsgbm9uZSBpcyBnZW51aW5lbHkgYmxvY2tlZC4gVGhpcyBjYW5kaWRh
dGUgaXRzZWxmIGRvZXMgbm90IGNsYWltIGxhbmRlZCBjb21wbGV0aW9uIGJlZm9yZSByZXZpZXcs
IGNsb3Nlb3V0LCBhbmQgY29tbWl0LgotIFByaW9yIEQzL0Q14oCTRDEwIGNhbm9uaWNhbCBvd25l
cnMgYW5kIHJldmlldy1jb250cm9sIGFydGlmYWN0cyByZW1haW4gc3RhYmxlLgotIEQxMiByZW1h
aW5zIG5vdCBzdGFydGVkLCBub3QgZGlzcGF0Y2hlZCwgYW5kIHNlcGFyYXRlbHkgdW5hdXRob3Jp
emVkLgoKUmVxdWlyZWQgbWF0ZXJpYWwgY29uc2VxdWVuY2U6IGEgYmxvY2tpbmcgZmluZGluZyBt
dXN0IGlkZW50aWZ5IGEgcmVhY2hhYmxlIHBhdGNoLWNhdXNhbCBkZWZlY3QgdGhhdCBsb3NlcyBv
ciBjaGFuZ2VzIGF1dGhvcml0YXRpdmUgc291cmNlIHNlbWFudGljcywgYnJlYWtzIGEgc3VwcG9y
dGVkIGxpbmsvYW5jaG9yIG9yIGNvbXBhdGliaWxpdHkgcHJvamVjdGlvbiwgY3JlYXRlcyBkdXBs
aWNhdGUgb3IgZmFsc2Ugb3duZXJzaGlwLCBtaXNzdGF0ZXMgYXV0aG9yaXR5L3N0YXR1cy9wcm92
ZW5hbmNlL3JvbGxiYWNrLCBsZWF2ZXMgYSBzdWJzdGFudGl2ZSBEMTEgc3BhbiB1bmFjY291bnRl
ZCBmb3IsIG9yIHdpZGVucyBpbnRvIGV4Y2x1ZGVkIEQxMi9ydW50aW1lL3BsYXRmb3JtL3Jldmll
dy1nb3Zlcm5hbmNlIHdvcmsuCkJsb2NraW5nIHRocmVzaG9sZDogUDEvUDIgb25seS4gVGhlIGZp
bmRpbmcgbXVzdCBiZSByZWFjaGFibGUgdGhyb3VnaCB0aGUgc3VwcG9ydGVkIE1hcmtkb3duIGJl
aGF2aW9yLCBtYXRlcmlhbCB1bmRlciB0aGUgY29uc2VxdWVuY2UgZGVmaW5pdGlvbiwgY2F1c2Vk
IG9yIG1hdGVyaWFsbHkgd29yc2VuZWQgYnkgdGhpcyBwYXRjaCwgYW5kIHdpdGhpbiB0aGUgZXhh
Y3QgRDExIEJhdGNoIDQgZmVuY2UuIFByZS1leGlzdGluZyBiZWhhdmlvciwgc3R5bGlzdGljIHBy
ZWZlcmVuY2VzLCBzcGVjdWxhdGl2ZSBmdXR1cmUgY29uY2VybnMsIGFuZCB1bnN1cHBvcnRlZCBv
cGFxdWUtbGluayByZXNvbHV0aW9uIGFyZSBub24tYmxvY2tpbmcuCkFjY2VwdGVkIHByaW9yIGZp
bmRpbmdzOiBub25lOyB0aGlzIGlzIGFuIGluaXRpYWwtcmFuZ2UgcmV2aWV3LgpEZWZlcnJlZCBv
ciBvdXQtb2Ytc2NvcGUgY29uY2VybnM6IEQxMiByb290IGN1dG92ZXI7IHJ1bnRpbWUgaW1wbGVt
ZW50YXRpb247IFJ1c3Qvc2NyaXB0cy9wbGF0Zm9ybSBiZWhhdmlvcjsgcHJvb2Ygb3IgZ2F0ZSBz
YXRpc2ZhY3Rpb247IGF1dGhvcml0eS9kaXNwYXRjaC9wcm9tb3Rpb247IHN1Y2Nlc3NvciB3b3Jr
OyBlZGl0cyB0byBEMyByZXZpZXctY29udHJvbCBnb3Zlcm5hbmNlIG9yIHJldmlldy1jb250cm9s
LzsgcmUtZXh0cmFjdGlvbiBvciBtb2RpZmljYXRpb24gb2YgRDUvRDYgcGFja2V0LWZhbWlseSBv
d25lcnM7IGNoYW5nZXMgdG8gRDExIEJhdGNoZXMgMeKAkzM7IGdsb2JhbCBNYXJrZG93biBub3Jt
YWxpemF0aW9uOyByb2FkbWFwIGNoYW5nZXM7IHVucmVsYXRlZCBkb2NzOyB0aGUgcmV2aWV3ZWQg
WklQOyBvcGFxdWUtcmVmZXJlbmNlIHRhcmdldCBleGlzdGVuY2U7IGFuZCBwcmUtZXhpc3Rpbmcg
ZGVmZWN0cyBub3QgaW50cm9kdWNlZCBvciB3b3JzZW5lZCBieSB0aGlzIHBhdGNoLgpQcm9qZWN0
IHNvdXJjZXM6IG5vIGJyb2FkZXIgcmVwb3NpdG9yeSBidW5kbGUgaXMgcHJvdmlkZWQuIFJldmll
dyBvbmx5IHRoZSBjb21wbGV0ZSBwYXRjaCwgZXhwbGljaXQgYmFzZWxpbmUgaWRlbnRpZmllcnMs
IG1hbmlmZXN0LCBpbnZhcmlhbnRzLCB2YWxpZGF0aW9uIGV2aWRlbmNlLCBhbmQgZXhoYXVzdGl2
ZSBjb3ZlcmFnZSBhdWRpdCBiZWxvdy4gRXhjbHVkZWQgc291cmNlIGlzIHVuYXZhaWxhYmxlIGFu
ZCBtdXN0IG5vdCBiZSBpbmZlcnJlZC4gVGhlIHBhdGNoIGlzIGNvbXBsZXRlIGZvciB0aGUgZml2
ZS1wYXRoIGNhbmRpZGF0ZSBhbmQgaW5jbHVkZXMgaXRzIG5ldyB1bnRyYWNrZWQgZmlsZS4KClZh
bGlkYXRpb24gZXZpZGVuY2U6ClNUQVRVUwogTSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0
b3IvMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQKIE0gbGxtLWxhc3QtbWlsZS9ydW50aW1l
LXJlZmFjdG9yL2V2aWRlbmNlL1JFQURNRS5tZAogTSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVm
YWN0b3IvaW5kZXgvUkVBRE1FLm1kCiBNIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9t
aWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQKPz8gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJl
ZmFjdG9yL2V2aWRlbmNlL3JlY2VpcHRzL1JFQURNRS5tZAoKRGlydHkgZmVuY2UgdmFsaWRhdGVk
OiBleGFjdCBmaXZlIHBhdGhzIGRpcnR5OyBub3RoaW5nIHN0YWdlZC4KCmdpdCBkaWZmIC0tY2hl
Y2s6IGNsZWFuCgpnaXQgZGlmZiAtLW5vLWluZGV4IC0tY2hlY2sgL2Rldi9udWxsIHJlY2VpcHRz
IFJFQURNRTogY2xlYW4KClJvb3QgcHJlZml4L3N1ZmZpeCBieXRlIHN0YWJpbGl0eSBhbmQgZXhh
Y3QgcmVjb25zdHJ1Y3Rpb24gdmFsaWRhdGVkLgoKRXZpZGVuY2UgUkVBRE1FIG1hcmtlciBib2R5
IGJ5dGVzL2hhc2ggbWF0Y2ggZXhhY3QgODIzLWJ5dGUgc291cmNlIGFydGlmYWN0LgoKUm9vdCBh
dWRpdCBjb3VudHMgdmFsaWRhdGVkOiBoZWFkaW5ncz04MywgcG9pbnRlcl9saW5lcz04MywgcmVz
aWR1YWw9MAoKUm9vdCBoZWFkaW5nIGFuY2hvcnMgdW5pcXVlOyBsZWdhY3kgaG93LXRvLXJlYWQg
YW5jaG9yIHByZXNlcnZlZC4KCk9wYXF1ZSBsaW5rcyB2YWxpZGF0ZWQ6IHRocmVlIHJlYWRpbmct
bW9kZWwgbGluayBidWxsZXRzIHByZXNlcnZlZCBleGFjdGx5IG9uY2UgaW4gb3duZXI7IDAwIG9w
YXF1ZSBsaW5rIGJ5dGUtc3RhYmxlLgoKRXZpZGVuY2UgUkVBRE1FIG1ldGFkYXRhIHVwZGF0ZWQg
dHJ1dGhmdWxseTsgQmF0Y2gxLTMgcm93cyBwcmVzZXJ2ZWQ7IGV4YWN0bHkgb25lIHJlY2VpcHQg
cm93IGFuZCBvbmUgY3VycmVudC1zdGF0ZSByb3cgYWRkZWQuCgpSZWNlaXB0cyBSRUFETUUgcmVz
dHJpY3RlZCB0byBhIHNpbmdsZSByZXZpZXctY29udHJvbCBSRUFETUUgbGluayB3aXRoIGNvcnJl
Y3QgYm91bmRhcnkgc3RhdGVtZW50cy4KCm1hY09TIHBhcml0eSBvd25lciBoYXNoIHN0YWJsZTog
MmEzNjJjNzBmZGJmODBhMmE2NzY0M2M5YmNmNmU0NDE5NjUzY2JjOWY2NTJjYzEyZTE3YjcyYjk4
NWE2YTY3OAoKRXhhY3RseSBvbmUgc2hhcmVkLWV2aWRlbmNlLW5hdmlnYXRpb24gaW5kZXggcm93
IGFuZCBvbmUgRDExIEJhdGNoNCBsZWRnZXIgcm93IHByZXNlbnQuCgpIYXNoIHN0YWJpbGl0eSB2
YWxpZGF0ZWQgZm9yIDI1NiBwcmlvciBvd25lci9yZXZpZXctY29udHJvbCBhcnRpZmFjdHMgb3V0
c2lkZSB0aGUgZml2ZS1wYXRoIGZlbmNlLgoKTWFya2Rvd24gbGluay9mcmFnbWVudCBzY2FuIHBh
c3NlZCBhY3Jvc3MgcnVudGltZS1yZWZhY3RvcjogcmVzb2x2ZWQ9MTk1Mywgc2tpcHBlZF9vcGFx
dWU9NCwgZmFpbHVyZXM9MAoKUGF0Y2ggd3JpdHRlbjogL3ByaXZhdGUvdG1wL2QxMS1yZWFkaW5n
LXJlY2VpcHRzLWNhbmRpZGF0ZS5wYXRjaCBzaGEyNTY9MmUxZTk2ODdmNmYwZTU5OTFiYmRlOGFj
NmEyY2JmMzIxODBjZTI2MDQ4MTBmMzFkZmMzN2U3NWZhMWM3ZmU0ZQoKRm9yd2FyZCBhbmQgcmV2
ZXJzZSBwYXRjaCBhcHBseSBzdWNjZWVkZWQgYWdhaW5zdCBjbGVhbiBiYXNlbGluZSBleHBvcnRz
IGFuZCByZXR1cm5lZCB0byBieXRlLWlkZW50aWNhbCBiYXNlbGluZS4KClBvc3QtY2FuZGlkYXRl
IGF1ZGl0IHdyaXR0ZW46IC9wcml2YXRlL3RtcC9kMTEtZXhoYXVzdGl2ZS1jb3ZlcmFnZS1hdWRp
dC1wb3N0LWNhbmRpZGF0ZS50eHQgc2hhMjU2PWUzYTM5NTU3MzFkOTlmNWZmYzEzMzJlZWRjOTUw
NTVkY2RjZjg1ZTJkNmViYzVmNWI4NWM3NGEzYjAzYzg5ZjUKCkV4aGF1c3RpdmUgcG9zdC1jYW5k
aWRhdGUgY292ZXJhZ2UgYXVkaXQgKFNIQS0yNTYgZTNhMzk1NTczMWQ5OWY1ZmZjMTMzMmVlZGM5
NTA1NWRjZGNmODVlMmQ2ZWJjNWY1Yjg1Yzc0YTNiMDNjODlmNSk6CkQxMSBleGhhdXN0aXZlIGxp
dmUgY292ZXJhZ2UgYXVkaXQgYWZ0ZXIgZmluYWwgQmF0Y2ggNCBjYW5kaWRhdGUKRGF0ZTogMjAy
Ni0wOC0yNwpXb3JrdHJlZTogL1VzZXJzL3NwZW5zZXJtY2Nvbm5lbGwvLmNvZGV4L3dvcmt0cmVl
cy9iYWUzL3N1YnN0cmF0ZQpEZXRhY2hlZCBiYXNlbGluZSBIRUFEOiAzZmJhMTRjOGE4YWNiOTBi
YzNkZTQwYTc1MDM1ZTJhMGY1YjVmZDc2CkJhc2VsaW5lIHRyZWU6IDBiY2Y3ZWM4YjUxYWRhNzIw
YWNhYTMyMTkzNGNiZmIzNjhjYzA4YzQKQ2FuZGlkYXRlIHN0YXR1czogZXhhY3QgZml2ZS1wYXRo
IGRvY3Mtb25seSBmZW5jZTsgbm90aGluZyBzdGFnZWQKUm9vdDogbGxtLWxhc3QtbWlsZS9ydW50
aW1lLXJlZmFjdG9yLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kCgpNZWNoYW5pY2FsIHJv
b3QgYXVkaXQKLSBSb290IGhlYWRpbmdzIGJlbG93IEgxOiA4MwotIENhbm9uaWNhbC9jb21wYXRp
YmlsaXR5IHBvaW50ZXIgbGluZXM6IDgzCi0gUmVzaWR1YWwgc3Vic3RhbnRpdmUgbm9uaGVhZGlu
Zy9ub25wb2ludGVyIGxpbmVzOiAwCi0gTGVnYWN5IGAjIyBIb3cgdG8gcmVhZCB0aGlzIGxlZGdl
cmAgcm9vdCBhbmNob3IgcmVtYWlucyBwcmVzZW50IGV4YWN0bHkgb25jZSBhbmQgbm93IHBvaW50
cyB0byBgZXZpZGVuY2UvUkVBRE1FLm1kI2hvdy10by1yZWFkLXRoaXMtbGVkZ2VyYCBhcyBhIGNv
bXBhdGliaWxpdHktb25seSBwcm9qZWN0aW9uLgotIEN1cnJlbnQgY3Jvc3MtbGFuZSByb290IHBy
b2plY3Rpb24gcmVtYWlucyBjb21wYXRpYmlsaXR5LW9ubHkgYW5kIGJ5dGUtc3RhYmxlIGluIHRo
ZSBwcmVzZXJ2ZWQgcm9vdCBzdWZmaXg7IGNhbm9uaWNhbCBvd25lcnNoaXAgc3RheXMgYXQgYG1h
Y29zLWRldi1wYXJpdHkvZXZpZGVuY2UtcmVncmVzc2lvbi5tZCNjdXJyZW50LWNyb3NzLWxhbmUt
cmVncmVzc2lvbi1sZWRnZXItMjAyNi0wOC0yMC1jb250cm9sbGluZ2AuCgpSb2FkbWFwIHRhcmdl
dCBjbGFzc2lmaWNhdGlvbgoxLiBjYW5vbmljYWwgaXNzdWUgbGVkZ2VyIOKAlCBzdGlsbCBzYXRp
c2ZpZWQgYnkgRDExIEJhdGNoIDEgYXQgYGV2aWRlbmNlL2Nhbm9uaWNhbC1pc3N1ZS1sZWRnZXIu
bWRgOyB1bmNoYW5nZWQgY29tcGF0aWJpbGl0eSBhbmNob3JzIHJlbWFpbiBpbiByb290IDA1Lgoy
LiBwYWNrZXQtc3BlY2lmaWMgZXZpZGVuY2Ug4oCUIHN0aWxsIHNhdGlzZmllZCBieSBwcmlvciBE
NS9ENiBwYWNrZXQtZmFtaWx5IGNhbm9uaWNhbCBvd25lcnM7IG5vIHJlLWV4dHJhY3Rpb24gYW5k
IG5vIGRyaWZ0LgozLiBmYWlsZWQtYXR0ZW1wdCByZWNvcmRzIOKAlCBzdGlsbCBzYXRpc2ZpZWQg
YnkgcHJpb3IgcGFja2V0LWZhbWlseSBldmlkZW5jZSBvd25lcnM7IG5vIHJlLWV4dHJhY3Rpb24g
YW5kIG5vIGRyaWZ0Lgo0LiBib3VuZGVkLXN0b3AgcmVjb3JkcyDigJQgc3RpbGwgc2F0aXNmaWVk
IGJ5IHByaW9yIHBhY2tldC1mYW1pbHkgZXZpZGVuY2Ugb3duZXJzOyBubyByZS1leHRyYWN0aW9u
IGFuZCBubyBkcmlmdC4KNS4gcGxhdGZvcm0tZXZpZGVuY2UgbGltaXRhdGlvbnMg4oCUIHN0aWxs
IHNhdGlzZmllZCBieSBwcmlvciBENS9ENiBvd25lcnMgcGx1cyBtYWNPUyBwYXJpdHkgb3duZXJz
OyBubyBwcm9tb3Rpb24gb3IgbGltaXRhdGlvbiBjaGFuZ2UuCjYuIGJhc2VsaW5lIGJlaGF2aW9y
IOKAlCBzdGlsbCBzYXRpc2ZpZWQgYnkgRDExIEJhdGNoIDMgYXQgYGV2aWRlbmNlL2Jhc2VsaW5l
LWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kYDsgdW5jaGFuZ2VkIGNvbXBhdGliaWxp
dHkgYW5jaG9ycyByZW1haW4gaW4gcm9vdCAwNS4KNy4gcmVncmVzc2lvbiBvYmxpZ2F0aW9ucyDi
gJQgc3RpbGwgc2F0aXNmaWVkIGJ5IEQxMSBCYXRjaCAxLCBEMTEgQmF0Y2ggMywgYW5kIHBhY2tl
dC1sb2NhbCBENS9ENiBvd25lcnM7IG5vIHJlc2lkdWFsIHVub3duZWQgb2JsaWdhdGlvbiBzcGFu
IHJlbWFpbnMuCjguIHNtb2tlIHNjZW5hcmlvcyDigJQgc3RpbGwgc2F0aXNmaWVkIGJ5IEQxMSBC
YXRjaCAzIGF0IGBldmlkZW5jZS9iYXNlbGluZS1iZWhhdmlvcnMtYW5kLXNtb2tlLXNjZW5hcmlv
cy5tZGA7IGFsbCBzY2VuYXJpbyBhbmNob3JzIHJlbWFpbiBjb21wYXRpYmxlIGFuZCByZXNvbHZp
bmcuCjkuIGNsb3Nlb3V0IGV2aWRlbmNlIOKAlCBzdGlsbCBzYXRpc2ZpZWQgYnkgRDExIEJhdGNo
IDIgYXQgYGV2aWRlbmNlL2Nsb3Nlb3V0LWFuZC1yZXZpZXctY2FsaWJyYXRpb24ubWRgIHBsdXMg
cGFja2V0LWxvY2FsIEQ1L0Q2IG93bmVyczsgbm8gcmVzaWR1YWwgdW5vd25lZCBjbG9zZW91dCBz
cGFuIHJlbWFpbnMuCjEwLiBjdXJyZW50LXN0YXRlIHByb2plY3Rpb24g4oCUIHJlbWFpbnMgc2F0
aXNmaWVkIGJ5IHRoZSBleGlzdGluZyBENiBvd25lciBgbWFjb3MtZGV2LXBhcml0eS9ldmlkZW5j
ZS1yZWdyZXNzaW9uLm1kI2N1cnJlbnQtY3Jvc3MtbGFuZS1yZWdyZXNzaW9uLWxlZGdlci0yMDI2
LTA4LTIwLWNvbnRyb2xsaW5nYDsgQmF0Y2ggNCBhZGRzIG5hdmlnYXRpb24gb25seSBhbmQgdHJh
bnNmZXJzIG5vIG93bmVyc2hpcCBvciBzdGF0dXMuCgpCYXRjaCA0IGNvbXBsZXRpb24gY2xhc3Np
ZmljYXRpb24KLSBBY3Rpb25hYmxlIHJlYWRpbmctbW9kZWwgc3BhbiBleHRyYWN0ZWQgZXhhY3Rs
eTogbGluZXMgM+KAkzE1LCA4MjMgYnl0ZXMsIHNoYTI1NiA4MGY0Yjc1M2Q2ZTU3NDIwMmI1Mjc1
ODlkMzMwMjFjNzM1YTc2ZTJjNjU2M2ZjNWRjZTg5MGRmNTI0YzkzMDAyLCBub3cgcHJlc2VydmVk
IGJldHdlZW4gYGV4YWN0LWV4dHJhY3RlZC1ib2R5Omhvdy10by1yZWFkLXRoaXMtbGVkZ2VyOnN0
YXJ0L2VuZGAgaW4gYGV2aWRlbmNlL1JFQURNRS5tZGAuCi0gUmVjZWlwdCBib3VuZGFyeSBzYXRp
c2ZpZWQgYXMgbmF2aWdhdGlvbiBvbmx5OiBgZXZpZGVuY2UvcmVjZWlwdHMvUkVBRE1FLm1kYCBs
aW5rcyBvbmx5IHRvIGAuLi8uLi9yZXZpZXctY29udHJvbC9SRUFETUUubWRgLCBjb3BpZXMgbm8g
cmVjZWlwdCwgYW5kIGNsYWltcyBubyBwcm9vZi9hdXRob3JpdHkgdHJhbnNmZXIuCi0gU2hhcmVk
IGV2aWRlbmNlIG5hdmlnYXRpb24gY29tcGxldGVkOiBgc2hhcmVkLWV2aWRlbmNlLW5hdmlnYXRp
b25gIG5vdyBleGlzdHMgZXhhY3RseSBvbmNlIGluIGBpbmRleC9SRUFETUUubWRgIGFuZCBjb3Zl
cnMgdGhlIGV4YWN0IHJlYWRpbmctbW9kZWwgY2Fub25pY2FsIHNjb3BlIHBsdXMgbm9uLWF1dGhv
cml0YXRpdmUgbmF2aWdhdGlvbiBmb3IgRDExIG93bmVycywgdGhlIGFscmVhZHktY2Fub25pY2Fs
IGN1cnJlbnQtc3RhdGUgcHJvamVjdGlvbiwgYW5kIHRoZSByZWNlaXB0IGluZGV4LgotIE5vIGJs
b2NrZWQgRDExIHJvYWRtYXAgdGFyZ2V0IHJlbWFpbnMuCi0gRDExIGlzIG5vdCBsYW5kZWQgb3Ig
Y29tcGxldGUgYnkgdGhpcyBjYW5kaWRhdGUgYWxvbmU6IGluZGVwZW5kZW50IHJldmlldywgYW55
IGFjY2VwdGVkIHJlbWVkaWF0aW9uLCByZS1yZXZpZXcsIHRydXRoZnVsIGNsb3Nlb3V0LCBhbmQg
YXRvbWljIGxhbmRpbmcgcmVtYWluIG91dHNpZGUgdGhpcyBsb2NhbCBjYW5kaWRhdGUuCi0gRDEy
IHJlbWFpbnMgbm90IHN0YXJ0ZWQsIG5vdCBkaXNwYXRjaGVkLCBhbmQgc2VwYXJhdGVseSB1bmF1
dGhvcml6ZWQ7IHRoaXMgY2FuZGlkYXRlIGdyYW50cyBubyBzdWNjZXNzb3IsIGltcGxlbWVudGF0
aW9uLCBwcm9vZi9nYXRlLCByZW1lZGlhdGlvbiwgZGlzcGF0Y2gsIHByb21vdGlvbiwgb3IgcGxh
dGZvcm0gYXV0aG9yaXR5LgoKQ29tcGxldGUgY2FuZGlkYXRlIHBhdGNoIChTSEEtMjU2IDJlMWU5
Njg3ZjZmMGU1OTkxYmJkZThhYzZhMmNiZjMyMTgwY2UyNjA0ODEwZjMxZGZjMzdlNzVmYTFjN2Zl
NGUpOgpgYGBkaWZmCmRpZmYgLS1naXQgYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3Iv
MDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVm
YWN0b3IvMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQKaW5kZXggMTFhOTA0NmZmLi4wZjc2
M2RhMmIgMTAwNjQ0Ci0tLSBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci8wNS1kZWJ1
Zy1yZWdyZXNzaW9uLWxlZGdlci5tZAorKysgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0
b3IvMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQKQEAgLTIsMTYgKzIsNyBAQAogCiAjIyBI
b3cgdG8gcmVhZCB0aGlzIGxlZGdlcgogCi0tICoqUmVzb2x2ZWQgYmFzZWxpbmUqKiBtZWFucyBh
IHNwZWNpZmljIG9ic2VydmVkIGZhaWx1cmUgaGFzIGEgdHJ1c3R3b3J0aHkgZml4IG9yIGxpdmUg
cHJvb2YgdGhhdCBtdXN0IG5vdCByZWdyZXNzLiBJdCBkb2VzICoqbm90KiogcHJvbW90ZSB0aGUg
c3Vycm91bmRpbmcgYXJjaGl0ZWN0dXJlIHNlYW0uCi0tICoqUGFydGlhbGx5IHJlc29sdmVkKiog
bWVhbnMgYSBuYXJyb3cgYmVoYXZpb3Igd29ya3Mgd2hpbGUgdGhlIHRhcmdldCBvd25lcnNoaXAg
bW9kZWwgcmVtYWlucyB3cm9uZyBvciB1bnByb3Zlbi4KLS0gKipVbnJlc29sdmVkKiogbWVhbnMg
dGhlIHRhcmdldCBiZWhhdmlvciBsYWNrcyBhbiBpbXBsZW1lbnRhdGlvbiBhbmQgcHJvb2YgZ2F0
ZS4KLS0gSGlzdG9yaWNhbCBkaWFnbm9zZXMgYXJlIHJldGFpbmVkIG9ubHkgd2hlbiB0aGV5IGRl
ZmluZSBhIHBlcm1hbmVudCBuZWdhdGl2ZSBvciByZWdyZXNzaW9uIHRlc3QuCi0KLVByaW1hcnkg
c291cmNlIG1lbW9zOgotCi0tIFtgLi4vLi4vUlVOX1dPUkxEX1RBU0tfREVCVUdfQ0FOT05JQ0FM
Lm1kYF0oLi4vLi4vUlVOX1dPUkxEX1RBU0tfREVCVUdfQ0FOT05JQ0FMLm1kKQotLSBbYC4uLy4u
L0NPTlRJTlVFX1dPUkxEX1dPUktFUl9CTE9DS0lOR19ERVZJQVRJT05fREVCVUcubWRgXSguLi8u
Li9DT05USU5VRV9XT1JMRF9XT1JLRVJfQkxPQ0tJTkdfREVWSUFUSU9OX0RFQlVHLm1kKQotLSBb
YC4uLy4uL0NPREVYX1dPUkxEX0RJU1BBVENIX0dBUF9XUklURVVQLm1kYF0oLi4vLi4vQ09ERVhf
V09STERfRElTUEFUQ0hfR0FQX1dSSVRFVVAubWQpCitDb21wYXRpYmlsaXR5IGFuY2hvciBvbmx5
OyBjYW5vbmljYWwgY29udGVudDogW2BldmlkZW5jZS9SRUFETUUubWQjaG93LXRvLXJlYWQtdGhp
cy1sZWRnZXJgXShldmlkZW5jZS9SRUFETUUubWQjaG93LXRvLXJlYWQtdGhpcy1sZWRnZXIpLgog
CiAjIyBDYW5vbmljYWwgaXNzdWUgbGVkZ2VyCiAKZGlmZiAtLWdpdCBhL2xsbS1sYXN0LW1pbGUv
cnVudGltZS1yZWZhY3Rvci9ldmlkZW5jZS9SRUFETUUubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRp
bWUtcmVmYWN0b3IvZXZpZGVuY2UvUkVBRE1FLm1kCmluZGV4IGM1ZjU3NDJiYS4uNDNiYjBhZmU4
IDEwMDY0NAotLS0gYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZXZpZGVuY2UvUkVB
RE1FLm1kCisrKyBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9ldmlkZW5jZS9SRUFE
TUUubWQKQEAgLTEsMjEgKzEsNDAgQEAKICoqS2luZDoqKiBldmlkZW5jZSBpbmRleAogKipTdGFi
bGUgSUQ6KiogYHNoYXJlZC1ldmlkZW5jZS1uYXZpZ2F0aW9uYAotKipDYW5vbmljYWwgZm9yOioq
IG5vIGF1dGhvcml0eTsgZXh0cmFjdGVkIGV2aWRlbmNlLW93bmVyIG5hdmlnYXRpb24gb25seQot
KipTdGF0dXM6Kiogbm9uLWF1dGhvcml0YXRpdmUgbmF2aWdhdGlvbgotKipBdXRob3JpdHkgc2Nv
cGU6Kiogbm9uZTsgZm9sbG93IHRoZSBsaW5rZWQgY2Fub25pY2FsIG93bmVycyBhbmQgcGFja2V0
LWZhbWlseSByZWNvcmRzCi0qKlNvdXJjZSBzcGFuOioqIEQxMSBCYXRjaCAxIGFzc2VtYmx5IG9u
bHkKLSoqU3VwZXJzZWRlczoqKiBub25lCisqKkNhbm9uaWNhbCBmb3I6KiogZXhhY3QgZXh0cmFj
dGVkIGBIb3cgdG8gcmVhZCB0aGlzIGxlZGdlcmAgYm9keTsgbm9uLWF1dGhvcml0YXRpdmUgbmF2
aWdhdGlvbiBmb3IgZXh0cmFjdGVkIGV2aWRlbmNlIG93bmVycywgdGhlIGV4aXN0aW5nIGN1cnJl
bnQtc3RhdGUgcHJvamVjdGlvbiwgYW5kIHJlY2VpcHQvcmV2aWV3LWNvbnRyb2wgbG9va3VwCisq
KlN0YXR1czoqKiBjYW5vbmljYWwgZXh0cmFjdGVkIGJvZHkgd2l0aCBub24tYXV0aG9yaXRhdGl2
ZSBuYXZpZ2F0aW9uCisqKkF1dGhvcml0eSBzY29wZToqKiBleGFjdCBleHRyYWN0ZWQgcmVhZGlu
Zy9pbnRlcnByZXRhdGlvbiBzb3VyY2UgYm9keSBvbmx5OyBmb2xsb3cgdGhlIGxpbmtlZCBjYW5v
bmljYWwgb3duZXJzIGFuZCByZXZpZXctY29udHJvbCByZWNvcmRzIGZvciBldmVyeXRoaW5nIGVs
c2UKKyoqU291cmNlIHNwYW46KiogW2AuLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNo
b3ctdG8tcmVhZC10aGlzLWxlZGdlcmBdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1k
I2hvdy10by1yZWFkLXRoaXMtbGVkZ2VyKSwgRDExIEJhdGNoIDQgYmFzZWxpbmUgbGluZXMgM+KA
kzE1ICg4MjMgYnl0ZXMpCisqKkJhc2VsaW5lIHNwYW4gU0hBLTI1NjoqKiBgODBmNGI3NTNkNmU1
NzQyMDJiNTI3NTg5ZDMzMDIxYzczNWE3NmUyYzY1NjNmYzVkY2U4OTBkZjUyNGM5MzAwMmAKKyoq
U3VwZXJzZWRlczoqKiBjYW5vbmljYWwgb3duZXJzaGlwIG9mIHRoYXQgZXhhY3Qgc291cmNlIGJv
ZHkgb25seTsgbmF2aWdhdGlvbiByZW1haW5zIG5vbi1hdXRob3JpdGF0aXZlCiAqKlN1cGVyc2Vk
ZWQgYnk6Kiogbm9uZQotKipQcm9qZWN0aW9uIGNvbnN1bWVyczoqKiBbYC4uL2luZGV4L1JFQURN
RS5tZGBdKC4uL2luZGV4L1JFQURNRS5tZCkKKyoqUHJvamVjdGlvbiBjb25zdW1lcnM6KiogW2Au
Li9pbmRleC9SRUFETUUubWRgXSguLi9pbmRleC9SRUFETUUubWQpLCBbYC4uLzA1LWRlYnVnLXJl
Z3Jlc3Npb24tbGVkZ2VyLm1kI2hvdy10by1yZWFkLXRoaXMtbGVkZ2VyYF0oLi4vMDUtZGVidWct
cmVncmVzc2lvbi1sZWRnZXIubWQjaG93LXRvLXJlYWQtdGhpcy1sZWRnZXIpCiAKICMgUnVudGlt
ZSBldmlkZW5jZSBpbmRleAogCi0+ICoqQXV0aG9yaXR5IGJvdW5kYXJ5OioqIFRoaXMgZmlsZSBp
cyBuYXZpZ2F0aW9uIG9ubHkuIEl0IGRvZXMgbm90IGF1dGhvcml6ZSBpbXBsZW1lbnRhdGlvbiwg
Y2xvc2UgYSByZWdyZXNzaW9uIGdhdGUsIHNhdGlzZnkgYSBwcm9vZiBvYmxpZ2F0aW9uLCBwcm9t
b3RlIGV2aWRlbmNlIGludG8gYXV0aG9yaXR5LCBvciByZXBsYWNlIHRoZSBsaW5rZWQgY2Fub25p
Y2FsIG93bmVycy4KKz4gKipBdXRob3JpdHkgYm91bmRhcnk6KiogT25seSB0aGUgZXhhY3QgZXh0
cmFjdGVkIGJvZHkgYmV0d2VlbiB0aGUgbmFtZWQgbWFya2VycyBiZWxvdyBpcyBjYW5vbmljYWwg
aGVyZS4gQWxsIG5hdmlnYXRpb24gaW4gdGhpcyBmaWxlIHJlbWFpbnMgbm9uLWF1dGhvcml0YXRp
dmUuIEl0IGRvZXMgbm90IGF1dGhvcml6ZSBpbXBsZW1lbnRhdGlvbiwgY2xvc2UgYSByZWdyZXNz
aW9uIGdhdGUsIHNhdGlzZnkgYSBwcm9vZiBvYmxpZ2F0aW9uLCBwcm9tb3RlIGV2aWRlbmNlIGlu
dG8gYXV0aG9yaXR5LCBvciByZXBsYWNlIHRoZSBsaW5rZWQgY2Fub25pY2FsIG93bmVycy4KID4K
ID4gKipSZWNlaXB0IGJvdW5kYXJ5OioqIGByZXZpZXctY29udHJvbC9gIHJlbWFpbnMgdGhlIGNh
bm9uaWNhbCByZWNlaXB0LCByZXZpZXctcmVjb3JkLCBhbmQgY2xvc2VvdXQgYXJ0aWZhY3Qgc3Rv
cmUuIFRoaXMgZmlsZSBkb2VzIG5vdCBkdXBsaWNhdGUsIG5vcm1hbGl6ZSwgb3IgcmVwbGFjZSB0
aG9zZSByZWNvcmRzLgogCis8IS0tIGV4YWN0LWV4dHJhY3RlZC1ib2R5Omhvdy10by1yZWFkLXRo
aXMtbGVkZ2VyOnN0YXJ0IC0tPgorIyMgSG93IHRvIHJlYWQgdGhpcyBsZWRnZXIKKworLSAqKlJl
c29sdmVkIGJhc2VsaW5lKiogbWVhbnMgYSBzcGVjaWZpYyBvYnNlcnZlZCBmYWlsdXJlIGhhcyBh
IHRydXN0d29ydGh5IGZpeCBvciBsaXZlIHByb29mIHRoYXQgbXVzdCBub3QgcmVncmVzcy4gSXQg
ZG9lcyAqKm5vdCoqIHByb21vdGUgdGhlIHN1cnJvdW5kaW5nIGFyY2hpdGVjdHVyZSBzZWFtLgor
LSAqKlBhcnRpYWxseSByZXNvbHZlZCoqIG1lYW5zIGEgbmFycm93IGJlaGF2aW9yIHdvcmtzIHdo
aWxlIHRoZSB0YXJnZXQgb3duZXJzaGlwIG1vZGVsIHJlbWFpbnMgd3Jvbmcgb3IgdW5wcm92ZW4u
CistICoqVW5yZXNvbHZlZCoqIG1lYW5zIHRoZSB0YXJnZXQgYmVoYXZpb3IgbGFja3MgYW4gaW1w
bGVtZW50YXRpb24gYW5kIHByb29mIGdhdGUuCistIEhpc3RvcmljYWwgZGlhZ25vc2VzIGFyZSBy
ZXRhaW5lZCBvbmx5IHdoZW4gdGhleSBkZWZpbmUgYSBwZXJtYW5lbnQgbmVnYXRpdmUgb3IgcmVn
cmVzc2lvbiB0ZXN0LgorCitQcmltYXJ5IHNvdXJjZSBtZW1vczoKKworLSBbYC4uLy4uL1JVTl9X
T1JMRF9UQVNLX0RFQlVHX0NBTk9OSUNBTC5tZGBdKC4uLy4uL1JVTl9XT1JMRF9UQVNLX0RFQlVH
X0NBTk9OSUNBTC5tZCkKKy0gW2AuLi8uLi9DT05USU5VRV9XT1JMRF9XT1JLRVJfQkxPQ0tJTkdf
REVWSUFUSU9OX0RFQlVHLm1kYF0oLi4vLi4vQ09OVElOVUVfV09STERfV09SS0VSX0JMT0NLSU5H
X0RFVklBVElPTl9ERUJVRy5tZCkKKy0gW2AuLi8uLi9DT0RFWF9XT1JMRF9ESVNQQVRDSF9HQVBf
V1JJVEVVUC5tZGBdKC4uLy4uL0NPREVYX1dPUkxEX0RJU1BBVENIX0dBUF9XUklURVVQLm1kKQor
Cis8IS0tIGV4YWN0LWV4dHJhY3RlZC1ib2R5Omhvdy10by1yZWFkLXRoaXMtbGVkZ2VyOmVuZCAt
LT4KKwogfCBFdmlkZW5jZSBjb21wb25lbnQgfCBDYW5vbmljYWwgb3duZXIgfCBDdXJyZW50IGV4
dHJhY3RlZCBzY29wZSB8CiB8LS0tfC0tLXwtLS18CiB8IEJhdGNoIDEg4oCUIGNhbm9uaWNhbCBp
c3N1ZSBsZWRnZXIgfCBbYGNhbm9uaWNhbC1pc3N1ZS1sZWRnZXIubWQjY2Fub25pY2FsLWlzc3Vl
LWxlZGdlcmBdKGNhbm9uaWNhbC1pc3N1ZS1sZWRnZXIubWQjY2Fub25pY2FsLWlzc3VlLWxlZGdl
cikgfCBGdWxsIGV4dHJhY3RlZCBgIyMgQ2Fub25pY2FsIGlzc3VlIGxlZGdlcmAgdGFibGUgcGx1
cyB0aGUgcHJlc2VydmVkIGAjIyMgQTEuMmEtV0IgZ2F0ZSBhc3NpZ25tZW50YCBib2R5OyBleGFj
dCBpc3N1ZSBJRHMsIGNocm9ub2xvZ3ksIGxpbWl0YXRpb25zLCBvYmxpZ2F0aW9ucywgbGl0ZXJh
bHMsIGhhc2hlcywgY29tbWFuZHMsIGFuZCBuZWdhdGl2ZSByZXF1aXJlbWVudHMgcmVtYWluIHVu
Y2hhbmdlZC4gfAogfCBCYXRjaCAyIOKAlCBjbG9zZW91dCBydWxlICsgcmV2aWV3LXByb2Nlc3Mg
Y2FsaWJyYXRpb24gfCBbYGNsb3Nlb3V0LWFuZC1yZXZpZXctY2FsaWJyYXRpb24ubWRgXShjbG9z
ZW91dC1hbmQtcmV2aWV3LWNhbGlicmF0aW9uLm1kKSB8IEV4YWN0IGV4dHJhY3RlZCBbYCMjIENs
b3Nlb3V0IHJ1bGVgXShjbG9zZW91dC1hbmQtcmV2aWV3LWNhbGlicmF0aW9uLm1kI2Nsb3Nlb3V0
LXJ1bGUpIGFuZCBbYCMjIFJldmlldy1wcm9jZXNzIGNhbGlicmF0aW9uYF0oY2xvc2VvdXQtYW5k
LXJldmlldy1jYWxpYnJhdGlvbi5tZCNyZXZpZXctcHJvY2Vzcy1jYWxpYnJhdGlvbikgYm9kaWVz
IHdpdGggc2VwYXJhdGUgbm9uY29udGlndW91cyBzb3VyY2UgcHJvdmVuYW5jZTsgb3JkZXJlZCBi
dWxsZXRzLCBsaXRlcmFscywgSURzLCBuZWdhdGl2ZSByZXF1aXJlbWVudHMsIGV4Y2VwdGlvbnMs
IHJldmlldy1nb3Zlcm5hbmNlIHJlZmVyZW5jZS1vbmx5IGJvdW5kYXJpZXMsIGFuZCB0aGUgbm8t
cmVtZWRpYXRpb24vbm8tcHJvb2YtcmVzdWx0LWNoYW5nZSBzdGF0ZW1lbnQgcmVtYWluIHVuY2hh
bmdlZC4gfAogfCBCYXRjaCAzIOKAlCBiYXNlbGluZSBiZWhhdmlvcnMgKyBjcm9zcy1nYXRlIHNt
b2tlIHNjZW5hcmlvcyB8IFtgYmFzZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9rZS1zY2VuYXJpb3Mu
bWRgXShiYXNlbGluZS1iZWhhdmlvcnMtYW5kLXNtb2tlLXNjZW5hcmlvcy5tZCkgfCBFeGFjdCBl
eHRyYWN0ZWQgW2AjIyBCYXNlbGluZSBiZWhhdmlvcnMgdGhhdCBhbGwgdHJhY2tzIHByZXNlcnZl
YF0oYmFzZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9rZS1zY2VuYXJpb3MubWQjYmFzZWxpbmUtYmVo
YXZpb3JzLXRoYXQtYWxsLXRyYWNrcy1wcmVzZXJ2ZSkgYW5kIFtgIyMgQ3Jvc3MtZ2F0ZSBzbW9r
ZSBzY2VuYXJpb3NgXShiYXNlbGluZS1iZWhhdmlvcnMtYW5kLXNtb2tlLXNjZW5hcmlvcy5tZCNj
cm9zcy1nYXRlLXNtb2tlLXNjZW5hcmlvcykgYm9kaWVzIHdpdGggc2VwYXJhdGUgZXhhY3Qgc291
cmNlIHByb3ZlbmFuY2UgcHJlc2VydmVkIGJldHdlZW4gZGlzdGluY3QgYm91bmRhcnkgbWFya2Vy
czsgR2F0ZSBJRHMsIHN0YXR1c2VzLCB0YWJsZSBvcmRlci9jZWxscywgb3JkZXJlZCBzY2VuYXJp
byBzdGVwcywgZGV0ZXJtaW5pc3RpYyBwYXJrZWQtc3VjY2Vzc29yIHNlcXVlbmNlLCBDb3ZlcnMg
bGluZXMsIGFuZCBmYWlsLWNsb3NlZC9leGNlcHRpb24gYm91bmRhcmllcyByZW1haW4gdW5jaGFu
Z2VkLiB8Cit8IEJhdGNoIDQg4oCUIHJlY2VpcHQvcmV2aWV3LWNvbnRyb2wgbmF2aWdhdGlvbiB8
IFtgcmVjZWlwdHMvUkVBRE1FLm1kYF0ocmVjZWlwdHMvUkVBRE1FLm1kKSB8IE5hdmlnYXRpb24g
b25seTsgZm9sbG93IHRoZSByZWNlaXB0IGluZGV4IHRvIHRoZSBjYW5vbmljYWwgYHJldmlldy1j
b250cm9sL1JFQURNRS5tZGAgc3RvcmUuIE5vIHJlY2VpcHQgb3IgcmV2aWV3IHJlY29yZCBpcyBj
b3BpZWQsIHJlb3JnYW5pemVkIGludG8gbmV3IGV2aWRlbmNlLCBvciBub3JtYWxpemVkIGhlcmUs
IGFuZCBwcm9jZXNzL3JldmlldyBldmlkZW5jZSBkb2VzIG5vdCByZXBsYWNlIHByb2R1Y3QgcHJv
b2YsIGdhdGUgc2F0aXNmYWN0aW9uLCBvciBwYWNrZXQvaW1wbGVtZW50YXRpb24vZGlzcGF0Y2gv
cHJvbW90aW9uIGF1dGhvcml0eS4gfAorfCBCYXRjaCA0IOKAlCBjdXJyZW50LXN0YXRlIHByb2pl
Y3Rpb24gY29tcGF0aWJpbGl0eSBwb2ludGVyIHwgW2AuLi9tYWNvcy1kZXYtcGFyaXR5L2V2aWRl
bmNlLXJlZ3Jlc3Npb24ubWQjY3VycmVudC1jcm9zcy1sYW5lLXJlZ3Jlc3Npb24tbGVkZ2VyLTIw
MjYtMDgtMjAtY29udHJvbGxpbmdgXSguLi9tYWNvcy1kZXYtcGFyaXR5L2V2aWRlbmNlLXJlZ3Jl
c3Npb24ubWQjY3VycmVudC1jcm9zcy1sYW5lLXJlZ3Jlc3Npb24tbGVkZ2VyLTIwMjYtMDgtMjAt
Y29udHJvbGxpbmcpIHwgVGhlIHJvb3QgYDA1YCBjdXJyZW50IGNyb3NzLWxhbmUgcHJvamVjdGlv
biByZW1haW5zIGNvbXBhdGliaWxpdHktb25seTsgY2Fub25pY2FsIG93bmVyc2hpcCBhbmQgc3Rh
dHVzIHN0YXkgd2l0aCB0aGUgZXhpc3RpbmcgRDYgbWFjT1MtcGFyaXR5IGV2aWRlbmNlIG93bmVy
LCBhbmQgbm8gb3duZXJzaGlwIG9yIHN0YXR1cyB0cmFuc2ZlciBvY2N1cnMgaGVyZS4gfApkaWZm
IC0tZ2l0IGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZCBi
L2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9SRUFETUUubWQKaW5kZXggNDEx
OWQ4MWQ5Li45ODAzMzBhMjUgMTAwNjQ0Ci0tLSBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZh
Y3Rvci9pbmRleC9SRUFETUUubWQKKysrIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9y
L2luZGV4L1JFQURNRS5tZApAQCAtMzIsNiArMzIsNyBAQAogfCBgc2hhcmVkLXRhcmdldC1hcmNo
aXRlY3R1cmVgIHwgYXJjaGl0ZWN0dXJlIGluZGV4IHwgW2BhcmNoaXRlY3R1cmUvUkVBRE1FLm1k
YF0oLi4vYXJjaGl0ZWN0dXJlL1JFQURNRS5tZCkgfCBgbm9uLWF1dGhvcml0YXRpdmUgbmF2aWdh
dGlvbiBmb3IgdGhlIGV4dHJhY3RlZCBleGVjdXRpdmUgdGFyZ2V0IGRlY2lzaW9uLCBhdXRob3Jp
dHkgbWFwLCBudW1iZXJlZCBpbnZhcmlhbnRzLCBhbmQgc3RhYmxlIHJldmlldyBxdWVzdGlvbmAg
fCBTdXBlcnNlZGVzIG9ubHkgcm9vdC1jYW5vbmljYWwgb3duZXJzaGlwIG9mIHRoZSBleHRyYWN0
ZWQgc2hhcmVkIGFyY2hpdGVjdHVyZSBzcGFuczsgcGFja2V0LWZhbWlseS1sb2NhbCBENS9ENiBm
b3J3YXJkZXJzIHJlbWFpbiBhdCB0aGVpciBleGlzdGluZyBvd25lcnMuIHwgW2AwMWBdKC4uLzAx
LXRhcmdldC1hcmNoaXRlY3R1cmUubWQjZXhlY3V0aXZlLWRlY2lzaW9uKSwgW2AwMWBdKC4uLzAx
LXRhcmdldC1hcmNoaXRlY3R1cmUubWQjYXV0aG9yaXR5LW1hcCksIFtgMDFgXSguLi8wMS10YXJn
ZXQtYXJjaGl0ZWN0dXJlLm1kI25vbi1uZWdvdGlhYmxlLWludmFyaWFudHMpLCBbYDAxYF0oLi4v
MDEtdGFyZ2V0LWFyY2hpdGVjdHVyZS5tZCNyZXZpZXctcXVlc3Rpb24pIHwKIHwgYHNoYXJlZC1z
ZWFtLWNyb3Nzd2Fsa2AgfCBzZWFtIGluZGV4IHwgW2BzZWFtcy9SRUFETUUubWRgXSguLi9zZWFt
cy9SRUFETUUubWQpIHwgYGNhbm9uaWNhbCBzaGFyZWQgc2VhbS1jcm9zc3dhbGsgcnVsZXMgcGx1
cyBleHRyYWN0ZWQgc2VhbS1mYW1pbHkgbmF2aWdhdGlvbiBmb3IgaG9zdC9zZXNzaW9uLCBwZXJz
aXN0ZW5jZS9jb21wYXRpYmlsaXR5LCBkaXNwYXRjaC9lcGlzb2RlIHRyYW5zcG9ydCwgcG9saWN5
L25hcnJvd2luZywgcnVudGltZS1ldmVudC9yZWNlaXB0L3N1cGVydmlzaW9uL3JldGFpbmVkLXJ1
bnRpbWUsIG9ibGlnYXRpb25zL2hvc3QgcmUtZW5nYWdlbWVudCwgY29uZmlndXJhdGlvbi9nYXRl
d2F5IGFkb3B0aW9uLCBhbmQgVUFBL3Byb3ZpZGVyIHJlYWxpemF0aW9uL3NpZGUtZWZmZWN0IG1l
ZGlhdGlvbmAgfCBTdXBlcnNlZGVzIG9ubHkgdGhlIGV4dHJhY3RlZCByb290IHJlYWRpbmctcnVs
ZS9jbGFzc2lmaWNhdGlvbiBzcGFucyBhbmQgdGhlIGV4dHJhY3RlZCBob3N0L3Nlc3Npb24gYXV0
aG9yaXR5LCBwZXJzaXN0ZW5jZS9jb21wYXRpYmlsaXR5LCBkaXNwYXRjaC9lcGlzb2RlIHRyYW5z
cG9ydCwgcG9saWN5L25hcnJvd2luZywgcnVudGltZS1ldmVudC9yZWNlaXB0L3N1cGVydmlzaW9u
L3JldGFpbmVkLXJ1bnRpbWUsIG9ibGlnYXRpb25zL2hvc3QgcmUtZW5nYWdlbWVudCwgY29uZmln
dXJhdGlvbi9nYXRld2F5IGFkb3B0aW9uLCBhbmQgVUFBL3Byb3ZpZGVyIHJlYWxpemF0aW9uL3Np
ZGUtZWZmZWN0IG1lZGlhdGlvbiBmYW1pbHkgcm93czsgQTAgYW5kIHRoZSBleGlzdGluZyBENiBg
SG9zdFNlc3Npb25BdXRob3JpdHlgLCBgV29ybGRXb3JrZXJNZXNzYWdpbmdQcm90b2NvbGAsIGFu
ZCBgT2JsaWdhdGlvbkxlZGdlcmAgY29tcGF0aWJpbGl0eSByb3dzIHJlbWFpbiBhdCB0aGVpciBj
dXJyZW50IG93bmVycy4gfCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjcmVhZGluZy1y
dWxlKSwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1kI2EtaG9zdC1hdXRob3JpdHktYW5k
LWluZ3Jlc3MpLCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjYi1kaXNwYXRjaC1wb2xp
Y3ktcmVjZWlwdHMtYW5kLXJldGFpbmVkLXJ1bnRpbWUpLCBbYDAyYF0oLi4vMDItc2VhbS1jcm9z
c3dhbGsubWQjYy1vYmxpZ2F0aW9ucy1hbmQtaG9zdC1yZS1lbmdhZ2VtZW50KSwgW2AwMmBdKC4u
LzAyLXNlYW0tY3Jvc3N3YWxrLm1kI2QtdWFhLXJlYWxpemF0aW9uLXByb2plY3Rpb24tYW5kLXNp
ZGUtZWZmZWN0LW1lZGlhdGlvbiksIFtgMDJgXSguLi8wMi1zZWFtLWNyb3Nzd2Fsay5tZCNjbGFz
c2lmaWNhdGlvbi1jb25zZXF1ZW5jZXMpIHwKIHwgYHNoYXJlZC1zbGljZS1tYXBgIHwgc2xpY2Ug
aW5kZXggfCBbYHNsaWNlcy9SRUFETUUubWRgXSguLi9zbGljZXMvUkVBRE1FLm1kKSB8IGBjYW5v
bmljYWwgc2hhcmVkIHNlcXVlbmNpbmcvZGVwZW5kZW5jeSBhbmQgc2xpY2UtY2xvc2VvdXQgb3du
ZXIgcGx1cyBub24tYXV0aG9yaXRhdGl2ZSBEOSBUcmFjayBB4oCTRSBuYXZpZ2F0aW9uIGFuZCBB
MS9BMS40IHByb2plY3Rpb24gaW5kZXhgIHwgU3VwZXJzZWRlcyBvbmx5IHJvb3QtY2Fub25pY2Fs
IG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIHNoYXJlZCBzZXF1ZW5jaW5nL2Nsb3Nlb3V0IHNw
YW5zIHBsdXMgdGhlIGV4dHJhY3RlZCBUcmFjayBB4oCTRSBhbmQgQTEvQTEuNCBwcm9qZWN0aW9u
IHNwYW5zOyBjb250cm9sbGluZyBzY2hlZHVsZSBhdXRob3JpdHkgcmVtYWlucyB3aXRoIGRlY2lz
aW9ucywgcGFja2V0cywgZ2F0ZXMsIGFuZCBgY3VycmVudC5tZGAuIHwgW2AwM2BdKC4uLzAzLXBo
YXNlLXNsaWNlLW1hcC5tZCNzZXF1ZW5jaW5nLXJ1bGVzKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNs
aWNlLW1hcC5tZCN0cmFjay1hLS1hdXRob3JpdHktYW5kLXN1cmZhY2UtbmV1dHJhbGl0eSksIFtg
MDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjYTEtYm91bmRlZC1wYWNrZXQtZGVjb21wb3Np
dGlvbiksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjdHJhY2stYi0td29ybGQtZGlz
cGF0Y2gtcmVjZWlwdHMtc3VwZXJ2aXNpb24tYW5kLWNhbmNlbCksIFtgMDNgXSguLi8wMy1waGFz
ZS1zbGljZS1tYXAubWQjdHJhY2stYy0tb2JsaWdhdGlvbnMtaW5ib3gtYXV0by1hdHRhY2gtYW5k
LXJvdXRlci1hdHRhY2gpLCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3RyYWNrLWQt
LXVhYS1leGVjdXRpb24tZW52ZWxvcGUtYW5kLXNpZGUtZWZmZWN0LW1lZGlhdGlvbiksIFtgMDNg
XSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjdHJhY2stZS0tZGlzcGF0Y2gtc2NvcGVkLXBvbGlj
eS1uYXJyb3dpbmctYW5kLWNvbmZpZy1wcm9qZWN0aW9uKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNs
aWNlLW1hcC5tZCNzbGljZS1jbG9zZW91dC1taW5pbXVtKSB8Cit8IGBzaGFyZWQtZXZpZGVuY2Ut
bmF2aWdhdGlvbmAgfCBldmlkZW5jZSBpbmRleCB8IFtgZXZpZGVuY2UvUkVBRE1FLm1kYF0oLi4v
ZXZpZGVuY2UvUkVBRE1FLm1kKSB8IGBjYW5vbmljYWwgZXhhY3QgZXh0cmFjdGVkIEhvdyB0byBy
ZWFkIHRoaXMgbGVkZ2VyIGJvZHkgcGx1cyBub24tYXV0aG9yaXRhdGl2ZSBuYXZpZ2F0aW9uIGZv
ciBEMTEgZXZpZGVuY2Ugb3duZXJzLCB0aGUgZXhpc3RpbmcgRDYgY3VycmVudC1zdGF0ZSBwcm9q
ZWN0aW9uLCBhbmQgdGhlIHJlY2VpcHQgaW5kZXhgIHwgU3VwZXJzZWRlcyBvbmx5IHJvb3QtY2Fu
b25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIGAjIyBIb3cgdG8gcmVhZCB0aGlzIGxl
ZGdlcmAgYm9keTsgdGhlIHJlY2VpcHQgaW5kZXggYW5kIGN1cnJlbnQtc3RhdGUgcG9pbnRlciBy
ZW1haW4gbmF2aWdhdGlvbi1vbmx5IGFuZCBkbyBub3QgdHJhbnNmZXIgYXV0aG9yaXR5IG9yIGNo
YW5nZSBzdGF0dXMuIHwgW2AwNWBdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2hv
dy10by1yZWFkLXRoaXMtbGVkZ2VyKSB8CiB8IENhbm9uaWNhbCBpc3N1ZSBsZWRnZXIgfCBldmlk
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
Z25tZW50KSB8CiB8IENsb3Nlb3V0IHJ1bGUgKyBSZXZpZXctcHJvY2VzcyBjYWxpYnJhdGlvbiB8
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
YnJhdGlvbikgfAogfCBCYXNlbGluZSBiZWhhdmlvcnMgKyBDcm9zcy1nYXRlIHNtb2tlIHNjZW5h
cmlvcyB8IGV2aWRlbmNlL3JlZ3Jlc3Npb24gfCBbYGV2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9y
cy1hbmQtc21va2Utc2NlbmFyaW9zLm1kYF0oLi4vZXZpZGVuY2UvYmFzZWxpbmUtYmVoYXZpb3Jz
LWFuZC1zbW9rZS1zY2VuYXJpb3MubWQpIHwgYGNhbm9uaWNhbCBleHRyYWN0ZWQgY29tcG91bmQg
b3duZXIgZm9yIHRoZSBleGFjdCBiYXNlbGluZS1iZWhhdmlvciB0YWJsZSBhbmQgY3Jvc3MtZ2F0
ZSBzbW9rZS1zY2VuYXJpbyBib2RpZXNgIHwgU3VwZXJzZWRlcyBvbmx5IHJvb3QtY2Fub25pY2Fs
IG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIGAjIyBCYXNlbGluZSBiZWhhdmlvcnMgdGhhdCBh
bGwgdHJhY2tzIHByZXNlcnZlYCBhbmQgYCMjIENyb3NzLWdhdGUgc21va2Ugc2NlbmFyaW9zYCBi
b2RpZXM7IHRoZSBsZWdhY3kgcm9vdCBoZWFkaW5ncyBhbmQgc2NlbmFyaW8gc3ViaGVhZGluZ3Mg
cmVtYWluIGNvbXBhdGliaWxpdHkgYW5jaG9ycyB1bnRpbCBEMTIsIGFuZCBgcmV2aWV3LWNvbnRy
b2wvYCByZW1haW5zIHRoZSBjYW5vbmljYWwgcmVjZWlwdCBzdG9yZS4gfCBbYDA1YF0oLi4vMDUt
ZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjYmFzZWxpbmUtYmVoYXZpb3JzLXRoYXQtYWxsLXRy
YWNrcy1wcmVzZXJ2ZSksIFtgMDVgXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNj
cm9zcy1nYXRlLXNtb2tlLXNjZW5hcmlvcykgfApkaWZmIC0tZ2l0IGEvbGxtLWxhc3QtbWlsZS9y
dW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZCBiL2xsbS1sYXN0
LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQKaW5k
ZXggNWY1ZDVmNTA0Li43ZjhkYWMzMDkgMTAwNjQ0Ci0tLSBhL2xsbS1sYXN0LW1pbGUvcnVudGlt
ZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQKKysrIGIvbGxtLWxhc3Qt
bWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZApAQCAt
MjAxLDMgKzIwMSw0IEBAIFRoaXMgbGVkZ2VyIHJlY29yZHMgY29udGVudC1wcmVzZXJ2aW5nIGF1
dGhvcml0eSB0cmFuc2ZlcnMgd2hpbGUgcmV0YWluaW5nIHJlcXVpCiB8IEQxMSB8IFtgMDUtZGVi
dWctcmVncmVzc2lvbi1sZWRnZXIubWRgXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5t
ZCkgfCBSZXZpZXctcHJvY2VzcyBjYWxpYnJhdGlvbiB8IFtgcmV2aWV3LXByb2Nlc3MtY2FsaWJy
YXRpb25gXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNyZXZpZXctcHJvY2Vzcy1j
YWxpYnJhdGlvbikgfCBsaW5lcyAzMzLigJMzNTMgKDEzODIgYnl0ZXMpIHwgYDI4NDI4NGFjNTA3
YTliZGIzMjYzZjJjMThiYzYxNWMwNWRmM2ZkMjNlZDMzYTdhNjc0MmY4NWUyOWZiZTE3NWVgIHwg
W2AuLi9ldmlkZW5jZS9jbG9zZW91dC1hbmQtcmV2aWV3LWNhbGlicmF0aW9uLm1kYF0oLi4vZXZp
ZGVuY2UvY2xvc2VvdXQtYW5kLXJldmlldy1jYWxpYnJhdGlvbi5tZCkgfCBldmlkZW5jZS9yZWdy
ZXNzaW9uIHwgY2Fub25pY2FsIGNvbXBvdW5kIGRlc3RpbmF0aW9uOyBzb3VyY2UgY29tcGF0aWJp
bGl0eSBhbmNob3I7IHNlcGFyYXRlIG5vbmNvbnRpZ3VvdXMgcHJvdmVuYW5jZSBwcmVzZXJ2ZWQg
fCBub25lOyBleGFjdCAxMzgyLWJ5dGUgc291cmNlIGJvZHkgcHJlc2VydmVkIGJldHdlZW4gdGhl
IGByZXZpZXctcHJvY2Vzcy1jYWxpYnJhdGlvbmAgb3duZXIgYm91bmRhcnkgbWFya2VycyBpbiB0
aGUgY29tcG91bmQgZXZpZGVuY2Ugb3duZXI7IHRoZSBkZXZlbG9wbWVudC1yZXZpZXcgY29udHJh
Y3QgcmVsYXRpb25zaGlwIHJlbWFpbnMgcmVmZXJlbmNlLW9ubHkgYW5kIGRvZXMgbm90IHJlb3Bl
biBEMyB8IHJlc3RvcmUgdGhlIGV4YWN0IDM0Ny1ieXRlIHNvdXJjZSBib2R5IGF0IGAwNS1kZWJ1
Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNjbG9zZW91dC1ydWxlYCBhbmQgdGhlIGV4YWN0IDEzODIt
Ynl0ZSBzb3VyY2UgYm9keSBhdCBgMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjcmV2aWV3
LXByb2Nlc3MtY2FsaWJyYXRpb25gIGluIG9yaWdpbmFsIG9yZGVyOyByZW1vdmUgYGV2aWRlbmNl
L2Nsb3Nlb3V0LWFuZC1yZXZpZXctY2FsaWJyYXRpb24ubWRgOyByZW1vdmUgdGhlIHNvbGUgQmF0
Y2gtMiByb3cgZnJvbSBgZXZpZGVuY2UvUkVBRE1FLm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgQ2xv
c2VvdXQgcnVsZSArIFJldmlldy1wcm9jZXNzIGNhbGlicmF0aW9uYCByb3cgZnJvbSBgaW5kZXgv
UkVBRE1FLm1kYDsgcmVtb3ZlIGJvdGggRDExIGxlZGdlciBlbnRyaWVzIGZvciB0aGlzIGJhdGNo
OyBsZWF2ZSBCYXRjaCAxIHBsdXMgRDPigJNEMTAgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAs
IGByZXZpZXctY29udHJvbC9gLCBhbmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmaXZlLXBhdGgg
ZmVuY2UgdW5jaGFuZ2VkIHwKIHwgRDExIHwgW2AwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5t
ZGBdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kKSB8IEJhc2VsaW5lIGJlaGF2aW9y
cyB0aGF0IGFsbCB0cmFja3MgcHJlc2VydmUgfCBbYGJhc2VsaW5lLWJlaGF2aW9ycy10aGF0LWFs
bC10cmFja3MtcHJlc2VydmVgXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNiYXNl
bGluZS1iZWhhdmlvcnMtdGhhdC1hbGwtdHJhY2tzLXByZXNlcnZlKSB8IGxpbmVzIDgw4oCTOTIg
KDM2NDQgYnl0ZXMpIHwgYDVmOWVjOGQ5ODMyNDAxNTk4NmM4NDYyMDhkYjFjNTY3YjQ5YzkwOWJh
YTcxNDQyOGFjM2E2NDQ3MDlmZTkxMzJgIHwgW2AuLi9ldmlkZW5jZS9iYXNlbGluZS1iZWhhdmlv
cnMtYW5kLXNtb2tlLXNjZW5hcmlvcy5tZGBdKC4uL2V2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9y
cy1hbmQtc21va2Utc2NlbmFyaW9zLm1kKSB8IGV2aWRlbmNlL3JlZ3Jlc3Npb24gfCBjYW5vbmlj
YWwgY29tcG91bmQgZGVzdGluYXRpb247IHNvdXJjZSBjb21wYXRpYmlsaXR5IGFuY2hvcjsgc2Vw
YXJhdGUgZXhhY3QtYm9keSBwcm92ZW5hbmNlIHByZXNlcnZlZCB8IHBhaXJlZCBmYW1pbHkgYXRv
bWljaXR5IHdpdGggYENyb3NzLWdhdGUgc21va2Ugc2NlbmFyaW9zYDsgZXhhY3QgMzY0NC1ieXRl
IHNvdXJjZSBib2R5IHByZXNlcnZlZCBiZXR3ZWVuIHRoZSBgYmFzZWxpbmUtYmVoYXZpb3JzYCBv
d25lciBib3VuZGFyeSBtYXJrZXJzIGFuZCBub3QgcmVwcmVzZW50ZWQgYXMgb25lIGNvbnRpZ3Vv
dXMgZXh0cmFjdGlvbiB3aXRoIHRoZSBsYXRlciBzbW9rZS1zY2VuYXJpbyBib2R5IHwgcmVzdG9y
ZSB0aGUgZXhhY3QgMzY0NC1ieXRlIHNvdXJjZSBib2R5IGF0IGAwNS1kZWJ1Zy1yZWdyZXNzaW9u
LWxlZGdlci5tZCNiYXNlbGluZS1iZWhhdmlvcnMtdGhhdC1hbGwtdHJhY2tzLXByZXNlcnZlYCBh
bmQgdGhlIGV4YWN0IDExNzE0LWJ5dGUgc291cmNlIGJvZHkgYXQgYDA1LWRlYnVnLXJlZ3Jlc3Np
b24tbGVkZ2VyLm1kI2Nyb3NzLWdhdGUtc21va2Utc2NlbmFyaW9zYCBpbiBvcmlnaW5hbCBvcmRl
cjsgcmVtb3ZlIGBldmlkZW5jZS9iYXNlbGluZS1iZWhhdmlvcnMtYW5kLXNtb2tlLXNjZW5hcmlv
cy5tZGA7IHJlbW92ZSB0aGUgc29sZSBCYXRjaC0zIHJvdyBmcm9tIGBldmlkZW5jZS9SRUFETUUu
bWRgOyByZW1vdmUgdGhlIGV4YWN0IGBCYXNlbGluZSBiZWhhdmlvcnMgKyBDcm9zcy1nYXRlIHNt
b2tlIHNjZW5hcmlvc2Agcm93IGZyb20gYGluZGV4L1JFQURNRS5tZGA7IHJlbW92ZSBib3RoIEQx
MSBsZWRnZXIgZW50cmllcyBmb3IgdGhpcyBiYXRjaDsgbGVhdmUgQmF0Y2ggMSwgQmF0Y2ggMiwg
RDPigJNEMTAgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9gLCBh
bmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmaXZlLXBhdGggZmVuY2UgdW5jaGFuZ2VkIHwKIHwg
RDExIHwgW2AwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZGBdKC4uLzA1LWRlYnVnLXJlZ3Jl
c3Npb24tbGVkZ2VyLm1kKSB8IENyb3NzLWdhdGUgc21va2Ugc2NlbmFyaW9zIHwgW2Bjcm9zcy1n
YXRlLXNtb2tlLXNjZW5hcmlvc2BdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2Ny
b3NzLWdhdGUtc21va2Utc2NlbmFyaW9zKSB8IGxpbmVzIDkz4oCTMTkyICgxMTcxNCBieXRlcykg
fCBgNTg2ZTU0MDJiOTJiNmU0ZTk4YWI3Y2JkZDA0ZTY3MTIxZmYyMDljY2YyM2M3ZjFmZjExNWMx
ODhiZWRiY2E4NWAgfCBbYC4uL2V2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Ut
c2NlbmFyaW9zLm1kYF0oLi4vZXZpZGVuY2UvYmFzZWxpbmUtYmVoYXZpb3JzLWFuZC1zbW9rZS1z
Y2VuYXJpb3MubWQpIHwgZXZpZGVuY2UvcmVncmVzc2lvbiB8IGNhbm9uaWNhbCBjb21wb3VuZCBk
ZXN0aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9yOyBzZXBhcmF0ZSBleGFjdC1i
b2R5IHByb3ZlbmFuY2UgcHJlc2VydmVkIHwgcGFpcmVkIGZhbWlseSBhdG9taWNpdHkgd2l0aCBg
QmFzZWxpbmUgYmVoYXZpb3JzIHRoYXQgYWxsIHRyYWNrcyBwcmVzZXJ2ZWA7IGV4YWN0IDExNzE0
LWJ5dGUgc291cmNlIGJvZHkgcHJlc2VydmVkIGJldHdlZW4gdGhlIGBjcm9zcy1nYXRlLXNtb2tl
LXNjZW5hcmlvc2Agb3duZXIgYm91bmRhcnkgbWFya2VycyB3aXRoIG9yZGVyZWQgc2NlbmFyaW8g
c3ViaGVhZGluZ3MsIGRldGVybWluaXN0aWMgcGFya2VkLXN1Y2Nlc3NvciBzZXF1ZW5jZSwgYW5k
IENvdmVycyBsaW5lcyBpbnRlcm5hbCB0byB0aGUgcHJlc2VydmVkIGJvZHkgfCByZXN0b3JlIHRo
ZSBleGFjdCAzNjQ0LWJ5dGUgc291cmNlIGJvZHkgYXQgYDA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVk
Z2VyLm1kI2Jhc2VsaW5lLWJlaGF2aW9ycy10aGF0LWFsbC10cmFja3MtcHJlc2VydmVgIGFuZCB0
aGUgZXhhY3QgMTE3MTQtYnl0ZSBzb3VyY2UgYm9keSBhdCBgMDUtZGVidWctcmVncmVzc2lvbi1s
ZWRnZXIubWQjY3Jvc3MtZ2F0ZS1zbW9rZS1zY2VuYXJpb3NgIGluIG9yaWdpbmFsIG9yZGVyOyBy
ZW1vdmUgYGV2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1k
YDsgcmVtb3ZlIHRoZSBzb2xlIEJhdGNoLTMgcm93IGZyb20gYGV2aWRlbmNlL1JFQURNRS5tZGA7
IHJlbW92ZSB0aGUgZXhhY3QgYEJhc2VsaW5lIGJlaGF2aW9ycyArIENyb3NzLWdhdGUgc21va2Ug
c2NlbmFyaW9zYCByb3cgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsgcmVtb3ZlIGJvdGggRDExIGxl
ZGdlciBlbnRyaWVzIGZvciB0aGlzIGJhdGNoOyBsZWF2ZSBCYXRjaCAxLCBCYXRjaCAyLCBEM+KA
k0QxMCBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYHJldmlldy1jb250cm9sL2AsIGFuZCBl
dmVyeSBwYXRoIG91dHNpZGUgdGhlIGZpdmUtcGF0aCBmZW5jZSB1bmNoYW5nZWQgfAorfCBEMTEg
fCBbYDA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kYF0oLi4vMDUtZGVidWctcmVncmVzc2lv
bi1sZWRnZXIubWQpIHwgSG93IHRvIHJlYWQgdGhpcyBsZWRnZXIgfCBbYGhvdy10by1yZWFkLXRo
aXMtbGVkZ2VyYF0oLi4vMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjaG93LXRvLXJlYWQt
dGhpcy1sZWRnZXIpIHwgbGluZXMgM+KAkzE1ICg4MjMgYnl0ZXMpIHwgYDgwZjRiNzUzZDZlNTc0
MjAyYjUyNzU4OWQzMzAyMWM3MzVhNzZlMmM2NTYzZmM1ZGNlODkwZGY1MjRjOTMwMDJgIHwgW2Au
Li9ldmlkZW5jZS9SRUFETUUubWRgXSguLi9ldmlkZW5jZS9SRUFETUUubWQpIHwgZXZpZGVuY2Ug
aW5kZXggfCBjYW5vbmljYWwgZGVzdGluYXRpb24gZm9yIHRoZSBleGFjdCByZWFkaW5nL2ludGVy
cHJldGF0aW9uIGJvZHkgb25seTsgcmVjZWlwdC9jdXJyZW50LXN0YXRlIHJvd3MgcmVtYWluIG5v
bi1hdXRob3JpdGF0aXZlIG5hdmlnYXRpb24gfCBub25lOyBleGFjdCA4MjMtYnl0ZSBzb3VyY2Ug
Ym9keSBwcmVzZXJ2ZWQgYmV0d2VlbiB0aGUgYGV4YWN0LWV4dHJhY3RlZC1ib2R5Omhvdy10by1y
ZWFkLXRoaXMtbGVkZ2VyOnN0YXJ0L2VuZGAgbWFya2VycyBpbiBgZXZpZGVuY2UvUkVBRE1FLm1k
YDsgbWV0YWRhdGEgYW5kIEJhdGNoIDQgbmF2aWdhdGlvbiByb3dzIHJlbWFpbiBvdXRzaWRlIHRo
ZSBwcmVzZXJ2ZWQgYm9keSwgYW5kIHRoZSB0aHJlZSBvcGFxdWUgTWFya2Rvd24gbGlua3MgcmVt
YWluIGJ5dGUtaWRlbnRpY2FsIHwgcmVzdG9yZSB0aGUgZXhhY3QgODIzLWJ5dGUgcm9vdCBib2R5
IGF0IGAwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNob3ctdG8tcmVhZC10aGlzLWxlZGdl
cmA7IHJlc3RvcmUgdGhlIHByZS1CYXRjaDQgYGV2aWRlbmNlL1JFQURNRS5tZGAgbWV0YWRhdGEg
YW5kIHJlbW92ZSB0aGUgZXhhY3QgbWFya2VyIGJvZHkgcGx1cyB0aGUgZXhhY3QgcmVjZWlwdC9j
dXJyZW50LXN0YXRlIG5hdmlnYXRpb24gcm93czsgcmVtb3ZlIGBldmlkZW5jZS9yZWNlaXB0cy9S
RUFETUUubWRgOyByZW1vdmUgdGhlIGV4YWN0IGBzaGFyZWQtZXZpZGVuY2UtbmF2aWdhdGlvbmAg
cm93IGZyb20gYGluZGV4L1JFQURNRS5tZGA7IHJlbW92ZSB0aGlzIEQxMSBsZWRnZXIgZW50cnk7
IHByZXNlcnZlIEQxMSBCYXRjaGVzIDHigJMzLCBEM+KAk0QxMCBvd25lcnMsIHRoZSBleGlzdGlu
ZyBENiBjdXJyZW50LXN0YXRlIG93bmVyL3Byb2plY3Rpb24sIGBpbmRleC9jdXJyZW50Lm1kYCwg
YHJldmlldy1jb250cm9sL2AsIHRoZSBvcGFxdWUgYDAwYCBsaW5rLCBhbmQgZXZlcnkgcGF0aCBv
dXRzaWRlIHRoZSBmaXZlLXBhdGggZmVuY2UgdW5jaGFuZ2VkIHwKZGlmZiAtLWdpdCBhL2xsbS1s
YXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9ldmlkZW5jZS9yZWNlaXB0cy9SRUFETUUubWQgYi9s
bG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZXZpZGVuY2UvcmVjZWlwdHMvUkVBRE1FLm1k
Cm5ldyBmaWxlIG1vZGUgMTAwNjQ0CmluZGV4IDAwMDAwMDAwMC4uNmIxN2I4NGI4Ci0tLSAvZGV2
L251bGwKKysrIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2V2aWRlbmNlL3JlY2Vp
cHRzL1JFQURNRS5tZApAQCAtMCwwICsxLDEzIEBACisqKktpbmQ6KiogcmVjZWlwdCBpbmRleAor
KipTdGF0dXM6Kiogbm9uLWF1dGhvcml0YXRpdmUgbmF2aWdhdGlvbgorKipDYW5vbmljYWwgZm9y
OioqIG5vIGF1dGhvcml0eTsgcmVjZWlwdCBhbmQgcmV2aWV3LXJlY29yZCBuYXZpZ2F0aW9uIG9u
bHkKKworIyBSZWNlaXB0IGFuZCByZXZpZXctY29udHJvbCBuYXZpZ2F0aW9uCisKKz4gKipBdXRo
b3JpdHkgYm91bmRhcnk6KiogVGhpcyBmaWxlIGlzIG5hdmlnYXRpb24gb25seS4gSXQgY29waWVz
LCByZW9yZ2FuaXplcywgYW5kIG5vcm1hbGl6ZXMgbm8gcmVjZWlwdCBvciByZXZpZXcgcmVjb3Jk
LgorPgorPiAqKkV2aWRlbmNlIGJvdW5kYXJ5OioqIFByb2Nlc3MgYW5kIHJldmlldyBldmlkZW5j
ZSBkbyBub3QgcmVwbGFjZSBwcm9kdWN0IHByb29mLCBnYXRlIHNhdGlzZmFjdGlvbiwgb3IgcGFj
a2V0L2ltcGxlbWVudGF0aW9uL2Rpc3BhdGNoL3Byb21vdGlvbiBhdXRob3JpdHkuCisKK0ZvbGxv
dyB0aGUgY2Fub25pY2FsIHJlY2VpcHQgYW5kIHJldmlldy1yZWNvcmQgc3RvcmU6IFtgLi4vLi4v
cmV2aWV3LWNvbnRyb2wvUkVBRE1FLm1kYF0oLi4vLi4vcmV2aWV3LWNvbnRyb2wvUkVBRE1FLm1k
KS4KKworVGhhdCBzdG9yZSByZW1haW5zIHRoZSBleGFjdCBsb2NhdGlvbiBmb3IgcmVjZWlwdCBh
cnRpZmFjdHMsIHJldmlldy1jeWNsZSByZWNvcmRzLCBjbG9zZW91dCByZXZpZXcgbWV0YWRhdGEs
IGFuZCBpbnRlcnByZXRhdGlvbiBndWlkYW5jZS4gVGhpcyBmaWxlIGludHJvZHVjZXMgbm8gbmV3
IGNhbm9uaWNhbCByZWNlaXB0IGJvZHkgYW5kIHRyYW5zZmVycyBubyBvd25lcnNoaXAuCmBgYAoK
RG8gbm90IHJlZHVjZSB0aGUgdGFzayBvciByZXBsYWNlIGl0IHdpdGggYW4gZWFzaWVyIGFsdGVy
bmF0aXZlLiBQcmVzZXJ2ZSB0aGUgcmVxdWVzdGVkIHNjb3BlIGFuZCBwcm9qZWN0IGNvbnZlbnRp
b25zLgoKUmV0dXJuIGZpbmRpbmdzIGZpcnN0IGJ5IHNldmVyaXR5IHdpdGggZXhhY3QgZmlsZS9z
ZWN0aW9uLCBzdXBwb3J0ZWQtaW5wdXQgcmVhY2hhYmlsaXR5LCB2aW9sYXRlZCBpbnZhcmlhbnQs
IG1hdGVyaWFsIGNvbnNlcXVlbmNlLCBhbmQgcGF0Y2ggY2F1c2FsaXR5LiBTdGF0ZSBleHBsaWNp
dGx5IHdoZW4gdGhlcmUgYXJlIG5vIHF1YWxpZnlpbmcgZmluZGluZ3MuIEVuZCB3aXRoIGV4YWN0
bHkgb25lIHZlcmRpY3QgdG9rZW4gb24gaXRzIG93biBsaW5lOiBBUFBST1ZFRCBvciBSRVFVRVNU
X0NIQU5HRVMuCg==
```

</details>

<details>
<summary>Initial-range review answer (rendered copy)</summary>

<pre>
Worked for 1m 41s
Called tool
Called tool
Findings
P2 — Batch 4 rollback leaves a false canonical-ownership boundary behind

File/section: llm-last-mile/runtime-refactor/migration/extraction-ledger.md — D11 How to read this ledger rollback column, affecting evidence/README.md → Authority boundary.

Supported-input reachability: Follow the newly documented Batch 4 rollback literally. The patch changes the rendered authority-boundary sentence from “This file is navigation only” to a statement that the exact body “between the named markers below is canonical here.”

The rollback instruction restores the pre-Batch4 evidence/README.md metadata and removes the marker body and Batch 4 navigation rows, but never instructs restoration of that separately changed rendered authority-boundary sentence.

Violated invariant: The candidate requires truthful atomic rollback and treats false ownership/status/rollback as a qualifying material consequence. The blockquoted authority boundary is not part of the key/value metadata above the H1 and is not one of the navigation rows, so the stated rollback does not cover it.

Material consequence: After the prescribed rollback, the extracted markers/body would be gone, but evidence/README.md would still claim that a body “between the named markers below” is canonical there. That leaves a patch-caused false ownership/status statement and fails to restore the pre-Batch4 authority boundary.

Patch causality: Both sides are introduced by this candidate: Batch 4 changes the authority-boundary prose and simultaneously adds an incomplete rollback recipe. The fix is narrow: explicitly restore the pre-Batch4 authority-boundary sentence as part of the D11 Batch 4 rollback.

No other qualifying P1/P2 findings identified within the supplied five-path candidate.

REQUEST_CHANGES
</pre>

</details>

<details>
<summary>Initial-range review answer exact bytes (base64 UTF-8)</summary>

```text
V29ya2VkIGZvciAxbSA0MXMKQ2FsbGVkIHRvb2wKQ2FsbGVkIHRvb2wKRmluZGluZ3MKUDIg4oCU
IEJhdGNoIDQgcm9sbGJhY2sgbGVhdmVzIGEgZmFsc2UgY2Fub25pY2FsLW93bmVyc2hpcCBib3Vu
ZGFyeSBiZWhpbmQKCkZpbGUvc2VjdGlvbjogbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9y
L21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZCDigJQgRDExIEhvdyB0byByZWFkIHRoaXMg
bGVkZ2VyIHJvbGxiYWNrIGNvbHVtbiwgYWZmZWN0aW5nIGV2aWRlbmNlL1JFQURNRS5tZCDihpIg
QXV0aG9yaXR5IGJvdW5kYXJ5LgoKU3VwcG9ydGVkLWlucHV0IHJlYWNoYWJpbGl0eTogRm9sbG93
IHRoZSBuZXdseSBkb2N1bWVudGVkIEJhdGNoIDQgcm9sbGJhY2sgbGl0ZXJhbGx5LiBUaGUgcGF0
Y2ggY2hhbmdlcyB0aGUgcmVuZGVyZWQgYXV0aG9yaXR5LWJvdW5kYXJ5IHNlbnRlbmNlIGZyb20g
4oCcVGhpcyBmaWxlIGlzIG5hdmlnYXRpb24gb25seeKAnSB0byBhIHN0YXRlbWVudCB0aGF0IHRo
ZSBleGFjdCBib2R5IOKAnGJldHdlZW4gdGhlIG5hbWVkIG1hcmtlcnMgYmVsb3cgaXMgY2Fub25p
Y2FsIGhlcmUu4oCdCgpUaGUgcm9sbGJhY2sgaW5zdHJ1Y3Rpb24gcmVzdG9yZXMgdGhlIHByZS1C
YXRjaDQgZXZpZGVuY2UvUkVBRE1FLm1kIG1ldGFkYXRhIGFuZCByZW1vdmVzIHRoZSBtYXJrZXIg
Ym9keSBhbmQgQmF0Y2ggNCBuYXZpZ2F0aW9uIHJvd3MsIGJ1dCBuZXZlciBpbnN0cnVjdHMgcmVz
dG9yYXRpb24gb2YgdGhhdCBzZXBhcmF0ZWx5IGNoYW5nZWQgcmVuZGVyZWQgYXV0aG9yaXR5LWJv
dW5kYXJ5IHNlbnRlbmNlLgoKVmlvbGF0ZWQgaW52YXJpYW50OiBUaGUgY2FuZGlkYXRlIHJlcXVp
cmVzIHRydXRoZnVsIGF0b21pYyByb2xsYmFjayBhbmQgdHJlYXRzIGZhbHNlIG93bmVyc2hpcC9z
dGF0dXMvcm9sbGJhY2sgYXMgYSBxdWFsaWZ5aW5nIG1hdGVyaWFsIGNvbnNlcXVlbmNlLiBUaGUg
YmxvY2txdW90ZWQgYXV0aG9yaXR5IGJvdW5kYXJ5IGlzIG5vdCBwYXJ0IG9mIHRoZSBrZXkvdmFs
dWUgbWV0YWRhdGEgYWJvdmUgdGhlIEgxIGFuZCBpcyBub3Qgb25lIG9mIHRoZSBuYXZpZ2F0aW9u
IHJvd3MsIHNvIHRoZSBzdGF0ZWQgcm9sbGJhY2sgZG9lcyBub3QgY292ZXIgaXQuCgpNYXRlcmlh
bCBjb25zZXF1ZW5jZTogQWZ0ZXIgdGhlIHByZXNjcmliZWQgcm9sbGJhY2ssIHRoZSBleHRyYWN0
ZWQgbWFya2Vycy9ib2R5IHdvdWxkIGJlIGdvbmUsIGJ1dCBldmlkZW5jZS9SRUFETUUubWQgd291
bGQgc3RpbGwgY2xhaW0gdGhhdCBhIGJvZHkg4oCcYmV0d2VlbiB0aGUgbmFtZWQgbWFya2VycyBi
ZWxvd+KAnSBpcyBjYW5vbmljYWwgdGhlcmUuIFRoYXQgbGVhdmVzIGEgcGF0Y2gtY2F1c2VkIGZh
bHNlIG93bmVyc2hpcC9zdGF0dXMgc3RhdGVtZW50IGFuZCBmYWlscyB0byByZXN0b3JlIHRoZSBw
cmUtQmF0Y2g0IGF1dGhvcml0eSBib3VuZGFyeS4KClBhdGNoIGNhdXNhbGl0eTogQm90aCBzaWRl
cyBhcmUgaW50cm9kdWNlZCBieSB0aGlzIGNhbmRpZGF0ZTogQmF0Y2ggNCBjaGFuZ2VzIHRoZSBh
dXRob3JpdHktYm91bmRhcnkgcHJvc2UgYW5kIHNpbXVsdGFuZW91c2x5IGFkZHMgYW4gaW5jb21w
bGV0ZSByb2xsYmFjayByZWNpcGUuIFRoZSBmaXggaXMgbmFycm93OiBleHBsaWNpdGx5IHJlc3Rv
cmUgdGhlIHByZS1CYXRjaDQgYXV0aG9yaXR5LWJvdW5kYXJ5IHNlbnRlbmNlIGFzIHBhcnQgb2Yg
dGhlIEQxMSBCYXRjaCA0IHJvbGxiYWNrLgoKTm8gb3RoZXIgcXVhbGlmeWluZyBQMS9QMiBmaW5k
aW5ncyBpZGVudGlmaWVkIHdpdGhpbiB0aGUgc3VwcGxpZWQgZml2ZS1wYXRoIGNhbmRpZGF0ZS4K
ClJFUVVFU1RfQ0hBTkdFUwo=
```

</details>

<details>
<summary>Remediation-follow-up review prompt (rendered copy)</summary>

<pre>
Review mode: remediation-follow-up
Review context: independent fresh conversation; no continuity assumptions
Review boundary: previously reviewed D11 Batch 4 candidate patch SHA-256 2e1e9687f6f0e5991bbde8ac6a2cbf32180ce2604810f31dfc37e75fa1c7fe4e at exact baseline commit 3fba14c8a8acb90bc3de40a75035e2a0f5b5fd76 / tree 0bcf7ec8b51ada720acaa321934cbfb368cc08c4, plus the complete one-file remediation delta SHA-256 17f7b3e2779a77ea4d910d01bc17944ec0aa4a49f4522aa01dd10c8dd4cee723. Ordered byte-concatenation digest of initial patch followed immediately by delta: 7611ffa2dee56e9b0447320dc2a508ff555fef34790d7f6ad18e3cc0aaa42d96. Full remediated candidate patch SHA-256 b3198c157c82ab9ad89ff16691f607427d785ccf9823170b53bfbf49796fd786.
Review target: only whether the accepted rollback finding is fixed and whether this delta introduces a qualifying regression. Do not re-review the initial candidate outside the delta&#x27;s causal effects.

Supported inputs and behavior: GitHub-Flavored Markdown documentation. The changed extraction-ledger row is an operational atomic rollback recipe. A supported reader follows that recipe literally. The delta changes only the D11 Batch 4 rollback cell in migration/extraction-ledger.md.
Supported platforms and dialects: repository Markdown and literal rollback interpretation only. Runtime behavior, Rust, scripts, platforms, broader Markdown dialects, opaque-link resolution, and D12 are excluded.

In-scope invariants:
- The rollback recipe must explicitly restore the exact pre-Batch4 rendered evidence/README.md authority-boundary sentence in addition to restoring metadata and removing the marked body/navigation rows.
- Literal rollback must not leave a false canonical-body claim after the marker/body is removed.
- The delta must change only the sole D11 Batch 4 rollback cell and must not alter source provenance, hashes, ownership/status, candidate semantics, any other row/path, or D12/no-successor state.
- The pre-Batch4 authority-boundary sentence must be reproduced exactly: `&gt; **Authority boundary:** This file is navigation only. It does not authorize implementation, close a regression gate, satisfy a proof obligation, promote evidence into authority, or replace the linked canonical owners.`
- Existing rollback steps for exact root-body restoration, pre-Batch4 metadata, marker/body and navigation-row removal, receipts README removal, shared index row removal, ledger-row removal, and preservation of D11 Batches 1–3, D3–D10 owners, existing D6 current-state owner/projection, index/current.md, review-control/, opaque root-00 link, and every path outside the five-path fence must remain intact.

Required material consequence: a blocking finding must show that the accepted rollback defect remains reachable/materially unfixed, or that this one-line delta causes a new reachable material regression in supported rollback semantics, provenance, authority/status, source/hash preservation, or path/phase fencing.
Blocking threshold: P1/P2 only; reachable through the supported literal rollback behavior, material, and caused by the remediation delta or proof the accepted root cause remains unfixed.
Accepted prior findings: exactly one P2 — the Batch 4 rollback restored metadata and removed the exact marker body/navigation rows but failed to explicitly restore the separately changed rendered authority-boundary sentence, leaving a false canonical-body claim after literal rollback.
Deferred or out-of-scope concerns: re-review of unchanged initial-candidate content; stylistic preferences; pre-existing behavior; D12; runtime implementation; Rust/scripts/platform behavior; proof/gate/authority/dispatch/promotion; review-control edits; D3/D5/D6 owners; D11 Batches 1–3; opaque target existence; broad Markdown normalization; and any concern not caused by this delta or showing the accepted finding remains unfixed.
Project sources: no broader repository bundle. Review only the accepted finding, the complete delta below, its content-addressed relationship to the previously reviewed patch, and validation evidence. Excluded source is unavailable and must not be inferred.

Initial review&#x27;s accepted finding and verdict:
Worked for 1m 41s
Called tool
Called tool
Findings
P2 — Batch 4 rollback leaves a false canonical-ownership boundary behind

File/section: llm-last-mile/runtime-refactor/migration/extraction-ledger.md — D11 How to read this ledger rollback column, affecting evidence/README.md → Authority boundary.

Supported-input reachability: Follow the newly documented Batch 4 rollback literally. The patch changes the rendered authority-boundary sentence from “This file is navigation only” to a statement that the exact body “between the named markers below is canonical here.”

The rollback instruction restores the pre-Batch4 evidence/README.md metadata and removes the marker body and Batch 4 navigation rows, but never instructs restoration of that separately changed rendered authority-boundary sentence.

Violated invariant: The candidate requires truthful atomic rollback and treats false ownership/status/rollback as a qualifying material consequence. The blockquoted authority boundary is not part of the key/value metadata above the H1 and is not one of the navigation rows, so the stated rollback does not cover it.

Material consequence: After the prescribed rollback, the extracted markers/body would be gone, but evidence/README.md would still claim that a body “between the named markers below” is canonical there. That leaves a patch-caused false ownership/status statement and fails to restore the pre-Batch4 authority boundary.

Patch causality: Both sides are introduced by this candidate: Batch 4 changes the authority-boundary prose and simultaneously adds an incomplete rollback recipe. The fix is narrow: explicitly restore the pre-Batch4 authority-boundary sentence as part of the D11 Batch 4 rollback.

No other qualifying P1/P2 findings identified within the supplied five-path candidate.

REQUEST_CHANGES

Complete remediation delta (SHA-256 17f7b3e2779a77ea4d910d01bc17944ec0aa4a49f4522aa01dd10c8dd4cee723):
```diff
diff --git a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
index 7f8dac3..fb1cd9e 100644
--- a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
+++ b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
@@ -201,4 +201,4 @@ This ledger records content-preserving authority transfers while retaining requi
 | D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Review-process calibration | [`review-process-calibration`](../05-debug-regression-ledger.md#review-process-calibration) | lines 332–353 (1382 bytes) | `284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e` | [`../evidence/closeout-and-review-calibration.md`](../evidence/closeout-and-review-calibration.md) | evidence/regression | canonical compound destination; source compatibility anchor; separate noncontiguous provenance preserved | none; exact 1382-byte source body preserved between the `review-process-calibration` owner boundary markers in the compound evidence owner; the development-review contract relationship remains reference-only and does not reopen D3 | restore the exact 347-byte source body at `05-debug-regression-ledger.md#closeout-rule` and the exact 1382-byte source body at `05-debug-regression-ledger.md#review-process-calibration` in original order; remove `evidence/closeout-and-review-calibration.md`; remove the sole Batch-2 row from `evidence/README.md`; remove the exact `Closeout rule + Review-process calibration` row from `index/README.md`; remove both D11 ledger entries for this batch; leave Batch 1 plus D3–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
 | D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Baseline behaviors that all tracks preserve | [`baseline-behaviors-that-all-tracks-preserve`](../05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve) | lines 80–92 (3644 bytes) | `5f9ec8d98324015986c846208db1c567b49c909baa714428ac3a644709fe9132` | [`../evidence/baseline-behaviors-and-smoke-scenarios.md`](../evidence/baseline-behaviors-and-smoke-scenarios.md) | evidence/regression | canonical compound destination; source compatibility anchor; separate exact-body provenance preserved | paired family atomicity with `Cross-gate smoke scenarios`; exact 3644-byte source body preserved between the `baseline-behaviors` owner boundary markers and not represented as one contiguous extraction with the later smoke-scenario body | restore the exact 3644-byte source body at `05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve` and the exact 11714-byte source body at `05-debug-regression-ledger.md#cross-gate-smoke-scenarios` in original order; remove `evidence/baseline-behaviors-and-smoke-scenarios.md`; remove the sole Batch-3 row from `evidence/README.md`; remove the exact `Baseline behaviors + Cross-gate smoke scenarios` row from `index/README.md`; remove both D11 ledger entries for this batch; leave Batch 1, Batch 2, D3–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
 | D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | Cross-gate smoke scenarios | [`cross-gate-smoke-scenarios`](../05-debug-regression-ledger.md#cross-gate-smoke-scenarios) | lines 93–192 (11714 bytes) | `586e5402b92b6e4e98ab7cbdd04e67121ff209ccf23c7f1ff115c188bedbca85` | [`../evidence/baseline-behaviors-and-smoke-scenarios.md`](../evidence/baseline-behaviors-and-smoke-scenarios.md) | evidence/regression | canonical compound destination; source compatibility anchor; separate exact-body provenance preserved | paired family atomicity with `Baseline behaviors that all tracks preserve`; exact 11714-byte source body preserved between the `cross-gate-smoke-scenarios` owner boundary markers with ordered scenario subheadings, deterministic parked-successor sequence, and Covers lines internal to the preserved body | restore the exact 3644-byte source body at `05-debug-regression-ledger.md#baseline-behaviors-that-all-tracks-preserve` and the exact 11714-byte source body at `05-debug-regression-ledger.md#cross-gate-smoke-scenarios` in original order; remove `evidence/baseline-behaviors-and-smoke-scenarios.md`; remove the sole Batch-3 row from `evidence/README.md`; remove the exact `Baseline behaviors + Cross-gate smoke scenarios` row from `index/README.md`; remove both D11 ledger entries for this batch; leave Batch 1, Batch 2, D3–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
-| D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | How to read this ledger | [`how-to-read-this-ledger`](../05-debug-regression-ledger.md#how-to-read-this-ledger) | lines 3–15 (823 bytes) | `80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002` | [`../evidence/README.md`](../evidence/README.md) | evidence index | canonical destination for the exact reading/interpretation body only; receipt/current-state rows remain non-authoritative navigation | none; exact 823-byte source body preserved between the `exact-extracted-body:how-to-read-this-ledger:start/end` markers in `evidence/README.md`; metadata and Batch 4 navigation rows remain outside the preserved body, and the three opaque Markdown links remain byte-identical | restore the exact 823-byte root body at `05-debug-regression-ledger.md#how-to-read-this-ledger`; restore the pre-Batch4 `evidence/README.md` metadata and remove the exact marker body plus the exact receipt/current-state navigation rows; remove `evidence/receipts/README.md`; remove the exact `shared-evidence-navigation` row from `index/README.md`; remove this D11 ledger entry; preserve D11 Batches 1–3, D3–D10 owners, the existing D6 current-state owner/projection, `index/current.md`, `review-control/`, the opaque `00` link, and every path outside the five-path fence unchanged |
+| D11 | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | How to read this ledger | [`how-to-read-this-ledger`](../05-debug-regression-ledger.md#how-to-read-this-ledger) | lines 3–15 (823 bytes) | `80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002` | [`../evidence/README.md`](../evidence/README.md) | evidence index | canonical destination for the exact reading/interpretation body only; receipt/current-state rows remain non-authoritative navigation | none; exact 823-byte source body preserved between the `exact-extracted-body:how-to-read-this-ledger:start/end` markers in `evidence/README.md`; metadata and Batch 4 navigation rows remain outside the preserved body, and the three opaque Markdown links remain byte-identical | restore the exact 823-byte root body at `05-debug-regression-ledger.md#how-to-read-this-ledger`; restore the pre-Batch4 `evidence/README.md` metadata, including the exact authority-boundary sentence `&gt; **Authority boundary:** This file is navigation only. It does not authorize implementation, close a regression gate, satisfy a proof obligation, promote evidence into authority, or replace the linked canonical owners.`, and remove the exact marker body plus the exact receipt/current-state navigation rows; remove `evidence/receipts/README.md`; remove the exact `shared-evidence-navigation` row from `index/README.md`; remove this D11 ledger entry; preserve D11 Batches 1–3, D3–D10 owners, the existing D6 current-state owner/projection, `index/current.md`, `review-control/`, the opaque `00` link, and every path outside the five-path fence unchanged |
```
Validation evidence (SHA-256 c03fd810295427690ddadd8c041434d999c8cd88751de4858dc37b71d3c697b3):
Date: 2026-08-27
HEAD 3fba14c8a8acb90bc3de40a75035e2a0f5b5fd76
TREE 0bcf7ec8b51ada720acaa321934cbfb368cc08c4
STATUS
 M llm-last-mile/runtime-refactor/05-debug-regression-ledger.md
 M llm-last-mile/runtime-refactor/evidence/README.md
 M llm-last-mile/runtime-refactor/index/README.md
 M llm-last-mile/runtime-refactor/migration/extraction-ledger.md
?? llm-last-mile/runtime-refactor/evidence/receipts/README.md

Dirty fence validated: exact five paths dirty; nothing staged.
git diff --check: clean
git diff --no-index --check /dev/null receipts README: clean
Candidate comparison: only llm-last-mile/runtime-refactor/migration/extraction-ledger.md differs from the reviewed candidate; the other four candidate paths are byte-identical.
Row-only remediation validated: one replacement line at extraction-ledger line 204; no diff outside the single rollback cell relative to the reviewed candidate artifact.
Evidence README marker body matches the exact 823-byte source artifact; sha256=80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002 unchanged.
Root audit counts unchanged: headings_below_H1=83, pointer_lines=83, residual=0.
Markdown link/anchor validation passed: checked=1953, skipped_opaque=4, failures=0.
OPAQUE_SKIP 00-README.md:5 ../../substrate-runtime-refactor-directive-revised.md
OPAQUE_SKIP evidence/README.md:28 ../../RUN_WORLD_TASK_DEBUG_CANONICAL.md
OPAQUE_SKIP evidence/README.md:29 ../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md
OPAQUE_SKIP evidence/README.md:30 ../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md
Current-state owner stable: llm-last-mile/runtime-refactor/macos-dev-parity/evidence-regression.md sha256=2a362c70fdbf80a2a67643c9bcf6e4419653cbc9f652cc12e17b72b985a6a678.
Prior D3/D5-D10 owners, D11 Batch1-3 owners, and review-control remain stable because the exact dirty fence is limited to the same five candidate paths and no staged changes exist.
Remediation delta forward/reverse apply succeeded against a filesystem copy of the reviewed candidate and returned to byte-identical candidate state.
Full remediated candidate patch forward/reverse apply succeeded against a clean baseline export and returned to a byte-identical baseline tree.
candidate_patch_sha256=2e1e9687f6f0e5991bbde8ac6a2cbf32180ce2604810f31dfc37e75fa1c7fe4e
remediation_delta_sha256=17f7b3e2779a77ea4d910d01bc17944ec0aa4a49f4522aa01dd10c8dd4cee723
remediated_candidate_sha256=b3198c157c82ab9ad89ff16691f607427d785ccf9823170b53bfbf49796fd786
ordered_initial_plus_delta_sha256=7611ffa2dee56e9b0447320dc2a508ff555fef34790d7f6ad18e3cc0aaa42d96


Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Report a blocking finding only when it is reachable through supported behavior, violates an in-scope invariant or accepted remediation objective, meets the material-consequence threshold, and was introduced by the delta, is a regression caused by it, or proves the accepted root cause remains unfixed. You may inspect unchanged supporting context only to establish causality; unchanged pre-existing behavior is not an in-scope finding merely because you observed it.

After the accepted finding is fixed and no qualifying delta regression remains, stop. Label adjacent observations as deferred, not blocking. Return findings first by severity with exact file/section, supported-input reachability, violated invariant, material consequence, and delta causality. State explicitly when there are no qualifying findings. End with exactly one verdict token on its own line: APPROVED or REQUEST_CHANGES.
</pre>

</details>

<details>
<summary>Remediation-follow-up review prompt exact bytes (base64 UTF-8)</summary>

```text
UmV2aWV3IG1vZGU6IHJlbWVkaWF0aW9uLWZvbGxvdy11cApSZXZpZXcgY29udGV4dDogaW5kZXBl
bmRlbnQgZnJlc2ggY29udmVyc2F0aW9uOyBubyBjb250aW51aXR5IGFzc3VtcHRpb25zClJldmll
dyBib3VuZGFyeTogcHJldmlvdXNseSByZXZpZXdlZCBEMTEgQmF0Y2ggNCBjYW5kaWRhdGUgcGF0
Y2ggU0hBLTI1NiAyZTFlOTY4N2Y2ZjBlNTk5MWJiZGU4YWM2YTJjYmYzMjE4MGNlMjYwNDgxMGYz
MWRmYzM3ZTc1ZmExYzdmZTRlIGF0IGV4YWN0IGJhc2VsaW5lIGNvbW1pdCAzZmJhMTRjOGE4YWNi
OTBiYzNkZTQwYTc1MDM1ZTJhMGY1YjVmZDc2IC8gdHJlZSAwYmNmN2VjOGI1MWFkYTcyMGFjYWEz
MjE5MzRjYmZiMzY4Y2MwOGM0LCBwbHVzIHRoZSBjb21wbGV0ZSBvbmUtZmlsZSByZW1lZGlhdGlv
biBkZWx0YSBTSEEtMjU2IDE3ZjdiM2UyNzc5YTc3ZWE0ZDkxMGQwMWJjMTc5NDRlYzBhYTRhNDlm
NDUyMmFhMDFkZDEwYzhkZDRjZWU3MjMuIE9yZGVyZWQgYnl0ZS1jb25jYXRlbmF0aW9uIGRpZ2Vz
dCBvZiBpbml0aWFsIHBhdGNoIGZvbGxvd2VkIGltbWVkaWF0ZWx5IGJ5IGRlbHRhOiA3NjExZmZh
MmRlZTU2ZTliMDQ0NzMyMGRjMmE1MDhmZjU1NWZlZjM0NzkwZDdmNmFkMThlM2NjMGFhYTQyZDk2
LiBGdWxsIHJlbWVkaWF0ZWQgY2FuZGlkYXRlIHBhdGNoIFNIQS0yNTYgYjMxOThjMTU3YzgyYWI5
YWQ4OWZmMTY2OTFmNjA3NDI3ZDc4NWNjZjk4MjMxNzBiNTNiZmJmNDk3OTZmZDc4Ni4KUmV2aWV3
IHRhcmdldDogb25seSB3aGV0aGVyIHRoZSBhY2NlcHRlZCByb2xsYmFjayBmaW5kaW5nIGlzIGZp
eGVkIGFuZCB3aGV0aGVyIHRoaXMgZGVsdGEgaW50cm9kdWNlcyBhIHF1YWxpZnlpbmcgcmVncmVz
c2lvbi4gRG8gbm90IHJlLXJldmlldyB0aGUgaW5pdGlhbCBjYW5kaWRhdGUgb3V0c2lkZSB0aGUg
ZGVsdGEncyBjYXVzYWwgZWZmZWN0cy4KClN1cHBvcnRlZCBpbnB1dHMgYW5kIGJlaGF2aW9yOiBH
aXRIdWItRmxhdm9yZWQgTWFya2Rvd24gZG9jdW1lbnRhdGlvbi4gVGhlIGNoYW5nZWQgZXh0cmFj
dGlvbi1sZWRnZXIgcm93IGlzIGFuIG9wZXJhdGlvbmFsIGF0b21pYyByb2xsYmFjayByZWNpcGUu
IEEgc3VwcG9ydGVkIHJlYWRlciBmb2xsb3dzIHRoYXQgcmVjaXBlIGxpdGVyYWxseS4gVGhlIGRl
bHRhIGNoYW5nZXMgb25seSB0aGUgRDExIEJhdGNoIDQgcm9sbGJhY2sgY2VsbCBpbiBtaWdyYXRp
b24vZXh0cmFjdGlvbi1sZWRnZXIubWQuClN1cHBvcnRlZCBwbGF0Zm9ybXMgYW5kIGRpYWxlY3Rz
OiByZXBvc2l0b3J5IE1hcmtkb3duIGFuZCBsaXRlcmFsIHJvbGxiYWNrIGludGVycHJldGF0aW9u
IG9ubHkuIFJ1bnRpbWUgYmVoYXZpb3IsIFJ1c3QsIHNjcmlwdHMsIHBsYXRmb3JtcywgYnJvYWRl
ciBNYXJrZG93biBkaWFsZWN0cywgb3BhcXVlLWxpbmsgcmVzb2x1dGlvbiwgYW5kIEQxMiBhcmUg
ZXhjbHVkZWQuCgpJbi1zY29wZSBpbnZhcmlhbnRzOgotIFRoZSByb2xsYmFjayByZWNpcGUgbXVz
dCBleHBsaWNpdGx5IHJlc3RvcmUgdGhlIGV4YWN0IHByZS1CYXRjaDQgcmVuZGVyZWQgZXZpZGVu
Y2UvUkVBRE1FLm1kIGF1dGhvcml0eS1ib3VuZGFyeSBzZW50ZW5jZSBpbiBhZGRpdGlvbiB0byBy
ZXN0b3JpbmcgbWV0YWRhdGEgYW5kIHJlbW92aW5nIHRoZSBtYXJrZWQgYm9keS9uYXZpZ2F0aW9u
IHJvd3MuCi0gTGl0ZXJhbCByb2xsYmFjayBtdXN0IG5vdCBsZWF2ZSBhIGZhbHNlIGNhbm9uaWNh
bC1ib2R5IGNsYWltIGFmdGVyIHRoZSBtYXJrZXIvYm9keSBpcyByZW1vdmVkLgotIFRoZSBkZWx0
YSBtdXN0IGNoYW5nZSBvbmx5IHRoZSBzb2xlIEQxMSBCYXRjaCA0IHJvbGxiYWNrIGNlbGwgYW5k
IG11c3Qgbm90IGFsdGVyIHNvdXJjZSBwcm92ZW5hbmNlLCBoYXNoZXMsIG93bmVyc2hpcC9zdGF0
dXMsIGNhbmRpZGF0ZSBzZW1hbnRpY3MsIGFueSBvdGhlciByb3cvcGF0aCwgb3IgRDEyL25vLXN1
Y2Nlc3NvciBzdGF0ZS4KLSBUaGUgcHJlLUJhdGNoNCBhdXRob3JpdHktYm91bmRhcnkgc2VudGVu
Y2UgbXVzdCBiZSByZXByb2R1Y2VkIGV4YWN0bHk6IGA+ICoqQXV0aG9yaXR5IGJvdW5kYXJ5Oioq
IFRoaXMgZmlsZSBpcyBuYXZpZ2F0aW9uIG9ubHkuIEl0IGRvZXMgbm90IGF1dGhvcml6ZSBpbXBs
ZW1lbnRhdGlvbiwgY2xvc2UgYSByZWdyZXNzaW9uIGdhdGUsIHNhdGlzZnkgYSBwcm9vZiBvYmxp
Z2F0aW9uLCBwcm9tb3RlIGV2aWRlbmNlIGludG8gYXV0aG9yaXR5LCBvciByZXBsYWNlIHRoZSBs
aW5rZWQgY2Fub25pY2FsIG93bmVycy5gCi0gRXhpc3Rpbmcgcm9sbGJhY2sgc3RlcHMgZm9yIGV4
YWN0IHJvb3QtYm9keSByZXN0b3JhdGlvbiwgcHJlLUJhdGNoNCBtZXRhZGF0YSwgbWFya2VyL2Jv
ZHkgYW5kIG5hdmlnYXRpb24tcm93IHJlbW92YWwsIHJlY2VpcHRzIFJFQURNRSByZW1vdmFsLCBz
aGFyZWQgaW5kZXggcm93IHJlbW92YWwsIGxlZGdlci1yb3cgcmVtb3ZhbCwgYW5kIHByZXNlcnZh
dGlvbiBvZiBEMTEgQmF0Y2hlcyAx4oCTMywgRDPigJNEMTAgb3duZXJzLCBleGlzdGluZyBENiBj
dXJyZW50LXN0YXRlIG93bmVyL3Byb2plY3Rpb24sIGluZGV4L2N1cnJlbnQubWQsIHJldmlldy1j
b250cm9sLywgb3BhcXVlIHJvb3QtMDAgbGluaywgYW5kIGV2ZXJ5IHBhdGggb3V0c2lkZSB0aGUg
Zml2ZS1wYXRoIGZlbmNlIG11c3QgcmVtYWluIGludGFjdC4KClJlcXVpcmVkIG1hdGVyaWFsIGNv
bnNlcXVlbmNlOiBhIGJsb2NraW5nIGZpbmRpbmcgbXVzdCBzaG93IHRoYXQgdGhlIGFjY2VwdGVk
IHJvbGxiYWNrIGRlZmVjdCByZW1haW5zIHJlYWNoYWJsZS9tYXRlcmlhbGx5IHVuZml4ZWQsIG9y
IHRoYXQgdGhpcyBvbmUtbGluZSBkZWx0YSBjYXVzZXMgYSBuZXcgcmVhY2hhYmxlIG1hdGVyaWFs
IHJlZ3Jlc3Npb24gaW4gc3VwcG9ydGVkIHJvbGxiYWNrIHNlbWFudGljcywgcHJvdmVuYW5jZSwg
YXV0aG9yaXR5L3N0YXR1cywgc291cmNlL2hhc2ggcHJlc2VydmF0aW9uLCBvciBwYXRoL3BoYXNl
IGZlbmNpbmcuCkJsb2NraW5nIHRocmVzaG9sZDogUDEvUDIgb25seTsgcmVhY2hhYmxlIHRocm91
Z2ggdGhlIHN1cHBvcnRlZCBsaXRlcmFsIHJvbGxiYWNrIGJlaGF2aW9yLCBtYXRlcmlhbCwgYW5k
IGNhdXNlZCBieSB0aGUgcmVtZWRpYXRpb24gZGVsdGEgb3IgcHJvb2YgdGhlIGFjY2VwdGVkIHJv
b3QgY2F1c2UgcmVtYWlucyB1bmZpeGVkLgpBY2NlcHRlZCBwcmlvciBmaW5kaW5nczogZXhhY3Rs
eSBvbmUgUDIg4oCUIHRoZSBCYXRjaCA0IHJvbGxiYWNrIHJlc3RvcmVkIG1ldGFkYXRhIGFuZCBy
ZW1vdmVkIHRoZSBleGFjdCBtYXJrZXIgYm9keS9uYXZpZ2F0aW9uIHJvd3MgYnV0IGZhaWxlZCB0
byBleHBsaWNpdGx5IHJlc3RvcmUgdGhlIHNlcGFyYXRlbHkgY2hhbmdlZCByZW5kZXJlZCBhdXRo
b3JpdHktYm91bmRhcnkgc2VudGVuY2UsIGxlYXZpbmcgYSBmYWxzZSBjYW5vbmljYWwtYm9keSBj
bGFpbSBhZnRlciBsaXRlcmFsIHJvbGxiYWNrLgpEZWZlcnJlZCBvciBvdXQtb2Ytc2NvcGUgY29u
Y2VybnM6IHJlLXJldmlldyBvZiB1bmNoYW5nZWQgaW5pdGlhbC1jYW5kaWRhdGUgY29udGVudDsg
c3R5bGlzdGljIHByZWZlcmVuY2VzOyBwcmUtZXhpc3RpbmcgYmVoYXZpb3I7IEQxMjsgcnVudGlt
ZSBpbXBsZW1lbnRhdGlvbjsgUnVzdC9zY3JpcHRzL3BsYXRmb3JtIGJlaGF2aW9yOyBwcm9vZi9n
YXRlL2F1dGhvcml0eS9kaXNwYXRjaC9wcm9tb3Rpb247IHJldmlldy1jb250cm9sIGVkaXRzOyBE
My9ENS9ENiBvd25lcnM7IEQxMSBCYXRjaGVzIDHigJMzOyBvcGFxdWUgdGFyZ2V0IGV4aXN0ZW5j
ZTsgYnJvYWQgTWFya2Rvd24gbm9ybWFsaXphdGlvbjsgYW5kIGFueSBjb25jZXJuIG5vdCBjYXVz
ZWQgYnkgdGhpcyBkZWx0YSBvciBzaG93aW5nIHRoZSBhY2NlcHRlZCBmaW5kaW5nIHJlbWFpbnMg
dW5maXhlZC4KUHJvamVjdCBzb3VyY2VzOiBubyBicm9hZGVyIHJlcG9zaXRvcnkgYnVuZGxlLiBS
ZXZpZXcgb25seSB0aGUgYWNjZXB0ZWQgZmluZGluZywgdGhlIGNvbXBsZXRlIGRlbHRhIGJlbG93
LCBpdHMgY29udGVudC1hZGRyZXNzZWQgcmVsYXRpb25zaGlwIHRvIHRoZSBwcmV2aW91c2x5IHJl
dmlld2VkIHBhdGNoLCBhbmQgdmFsaWRhdGlvbiBldmlkZW5jZS4gRXhjbHVkZWQgc291cmNlIGlz
IHVuYXZhaWxhYmxlIGFuZCBtdXN0IG5vdCBiZSBpbmZlcnJlZC4KCkluaXRpYWwgcmV2aWV3J3Mg
YWNjZXB0ZWQgZmluZGluZyBhbmQgdmVyZGljdDoKV29ya2VkIGZvciAxbSA0MXMKQ2FsbGVkIHRv
b2wKQ2FsbGVkIHRvb2wKRmluZGluZ3MKUDIg4oCUIEJhdGNoIDQgcm9sbGJhY2sgbGVhdmVzIGEg
ZmFsc2UgY2Fub25pY2FsLW93bmVyc2hpcCBib3VuZGFyeSBiZWhpbmQKCkZpbGUvc2VjdGlvbjog
bGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdl
ci5tZCDigJQgRDExIEhvdyB0byByZWFkIHRoaXMgbGVkZ2VyIHJvbGxiYWNrIGNvbHVtbiwgYWZm
ZWN0aW5nIGV2aWRlbmNlL1JFQURNRS5tZCDihpIgQXV0aG9yaXR5IGJvdW5kYXJ5LgoKU3VwcG9y
dGVkLWlucHV0IHJlYWNoYWJpbGl0eTogRm9sbG93IHRoZSBuZXdseSBkb2N1bWVudGVkIEJhdGNo
IDQgcm9sbGJhY2sgbGl0ZXJhbGx5LiBUaGUgcGF0Y2ggY2hhbmdlcyB0aGUgcmVuZGVyZWQgYXV0
aG9yaXR5LWJvdW5kYXJ5IHNlbnRlbmNlIGZyb20g4oCcVGhpcyBmaWxlIGlzIG5hdmlnYXRpb24g
b25seeKAnSB0byBhIHN0YXRlbWVudCB0aGF0IHRoZSBleGFjdCBib2R5IOKAnGJldHdlZW4gdGhl
IG5hbWVkIG1hcmtlcnMgYmVsb3cgaXMgY2Fub25pY2FsIGhlcmUu4oCdCgpUaGUgcm9sbGJhY2sg
aW5zdHJ1Y3Rpb24gcmVzdG9yZXMgdGhlIHByZS1CYXRjaDQgZXZpZGVuY2UvUkVBRE1FLm1kIG1l
dGFkYXRhIGFuZCByZW1vdmVzIHRoZSBtYXJrZXIgYm9keSBhbmQgQmF0Y2ggNCBuYXZpZ2F0aW9u
IHJvd3MsIGJ1dCBuZXZlciBpbnN0cnVjdHMgcmVzdG9yYXRpb24gb2YgdGhhdCBzZXBhcmF0ZWx5
IGNoYW5nZWQgcmVuZGVyZWQgYXV0aG9yaXR5LWJvdW5kYXJ5IHNlbnRlbmNlLgoKVmlvbGF0ZWQg
aW52YXJpYW50OiBUaGUgY2FuZGlkYXRlIHJlcXVpcmVzIHRydXRoZnVsIGF0b21pYyByb2xsYmFj
ayBhbmQgdHJlYXRzIGZhbHNlIG93bmVyc2hpcC9zdGF0dXMvcm9sbGJhY2sgYXMgYSBxdWFsaWZ5
aW5nIG1hdGVyaWFsIGNvbnNlcXVlbmNlLiBUaGUgYmxvY2txdW90ZWQgYXV0aG9yaXR5IGJvdW5k
YXJ5IGlzIG5vdCBwYXJ0IG9mIHRoZSBrZXkvdmFsdWUgbWV0YWRhdGEgYWJvdmUgdGhlIEgxIGFu
ZCBpcyBub3Qgb25lIG9mIHRoZSBuYXZpZ2F0aW9uIHJvd3MsIHNvIHRoZSBzdGF0ZWQgcm9sbGJh
Y2sgZG9lcyBub3QgY292ZXIgaXQuCgpNYXRlcmlhbCBjb25zZXF1ZW5jZTogQWZ0ZXIgdGhlIHBy
ZXNjcmliZWQgcm9sbGJhY2ssIHRoZSBleHRyYWN0ZWQgbWFya2Vycy9ib2R5IHdvdWxkIGJlIGdv
bmUsIGJ1dCBldmlkZW5jZS9SRUFETUUubWQgd291bGQgc3RpbGwgY2xhaW0gdGhhdCBhIGJvZHkg
4oCcYmV0d2VlbiB0aGUgbmFtZWQgbWFya2VycyBiZWxvd+KAnSBpcyBjYW5vbmljYWwgdGhlcmUu
IFRoYXQgbGVhdmVzIGEgcGF0Y2gtY2F1c2VkIGZhbHNlIG93bmVyc2hpcC9zdGF0dXMgc3RhdGVt
ZW50IGFuZCBmYWlscyB0byByZXN0b3JlIHRoZSBwcmUtQmF0Y2g0IGF1dGhvcml0eSBib3VuZGFy
eS4KClBhdGNoIGNhdXNhbGl0eTogQm90aCBzaWRlcyBhcmUgaW50cm9kdWNlZCBieSB0aGlzIGNh
bmRpZGF0ZTogQmF0Y2ggNCBjaGFuZ2VzIHRoZSBhdXRob3JpdHktYm91bmRhcnkgcHJvc2UgYW5k
IHNpbXVsdGFuZW91c2x5IGFkZHMgYW4gaW5jb21wbGV0ZSByb2xsYmFjayByZWNpcGUuIFRoZSBm
aXggaXMgbmFycm93OiBleHBsaWNpdGx5IHJlc3RvcmUgdGhlIHByZS1CYXRjaDQgYXV0aG9yaXR5
LWJvdW5kYXJ5IHNlbnRlbmNlIGFzIHBhcnQgb2YgdGhlIEQxMSBCYXRjaCA0IHJvbGxiYWNrLgoK
Tm8gb3RoZXIgcXVhbGlmeWluZyBQMS9QMiBmaW5kaW5ncyBpZGVudGlmaWVkIHdpdGhpbiB0aGUg
c3VwcGxpZWQgZml2ZS1wYXRoIGNhbmRpZGF0ZS4KClJFUVVFU1RfQ0hBTkdFUwoKQ29tcGxldGUg
cmVtZWRpYXRpb24gZGVsdGEgKFNIQS0yNTYgMTdmN2IzZTI3NzlhNzdlYTRkOTEwZDAxYmMxNzk0
NGVjMGFhNGE0OWY0NTIyYWEwMWRkMTBjOGRkNGNlZTcyMyk6CmBgYGRpZmYKZGlmZiAtLWdpdCBh
L2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRn
ZXIubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJhY3Rp
b24tbGVkZ2VyLm1kCmluZGV4IDdmOGRhYzMuLmZiMWNkOWUgMTAwNjQ0Ci0tLSBhL2xsbS1sYXN0
LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQKKysr
IGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxl
ZGdlci5tZApAQCAtMjAxLDQgKzIwMSw0IEBAIFRoaXMgbGVkZ2VyIHJlY29yZHMgY29udGVudC1w
cmVzZXJ2aW5nIGF1dGhvcml0eSB0cmFuc2ZlcnMgd2hpbGUgcmV0YWluaW5nIHJlcXVpCiB8IEQx
MSB8IFtgMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWRgXSguLi8wNS1kZWJ1Zy1yZWdyZXNz
aW9uLWxlZGdlci5tZCkgfCBSZXZpZXctcHJvY2VzcyBjYWxpYnJhdGlvbiB8IFtgcmV2aWV3LXBy
b2Nlc3MtY2FsaWJyYXRpb25gXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNyZXZp
ZXctcHJvY2Vzcy1jYWxpYnJhdGlvbikgfCBsaW5lcyAzMzLigJMzNTMgKDEzODIgYnl0ZXMpIHwg
YDI4NDI4NGFjNTA3YTliZGIzMjYzZjJjMThiYzYxNWMwNWRmM2ZkMjNlZDMzYTdhNjc0MmY4NWUy
OWZiZTE3NWVgIHwgW2AuLi9ldmlkZW5jZS9jbG9zZW91dC1hbmQtcmV2aWV3LWNhbGlicmF0aW9u
Lm1kYF0oLi4vZXZpZGVuY2UvY2xvc2VvdXQtYW5kLXJldmlldy1jYWxpYnJhdGlvbi5tZCkgfCBl
dmlkZW5jZS9yZWdyZXNzaW9uIHwgY2Fub25pY2FsIGNvbXBvdW5kIGRlc3RpbmF0aW9uOyBzb3Vy
Y2UgY29tcGF0aWJpbGl0eSBhbmNob3I7IHNlcGFyYXRlIG5vbmNvbnRpZ3VvdXMgcHJvdmVuYW5j
ZSBwcmVzZXJ2ZWQgfCBub25lOyBleGFjdCAxMzgyLWJ5dGUgc291cmNlIGJvZHkgcHJlc2VydmVk
IGJldHdlZW4gdGhlIGByZXZpZXctcHJvY2Vzcy1jYWxpYnJhdGlvbmAgb3duZXIgYm91bmRhcnkg
bWFya2VycyBpbiB0aGUgY29tcG91bmQgZXZpZGVuY2Ugb3duZXI7IHRoZSBkZXZlbG9wbWVudC1y
ZXZpZXcgY29udHJhY3QgcmVsYXRpb25zaGlwIHJlbWFpbnMgcmVmZXJlbmNlLW9ubHkgYW5kIGRv
ZXMgbm90IHJlb3BlbiBEMyB8IHJlc3RvcmUgdGhlIGV4YWN0IDM0Ny1ieXRlIHNvdXJjZSBib2R5
IGF0IGAwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNjbG9zZW91dC1ydWxlYCBhbmQgdGhl
IGV4YWN0IDEzODItYnl0ZSBzb3VyY2UgYm9keSBhdCBgMDUtZGVidWctcmVncmVzc2lvbi1sZWRn
ZXIubWQjcmV2aWV3LXByb2Nlc3MtY2FsaWJyYXRpb25gIGluIG9yaWdpbmFsIG9yZGVyOyByZW1v
dmUgYGV2aWRlbmNlL2Nsb3Nlb3V0LWFuZC1yZXZpZXctY2FsaWJyYXRpb24ubWRgOyByZW1vdmUg
dGhlIHNvbGUgQmF0Y2gtMiByb3cgZnJvbSBgZXZpZGVuY2UvUkVBRE1FLm1kYDsgcmVtb3ZlIHRo
ZSBleGFjdCBgQ2xvc2VvdXQgcnVsZSArIFJldmlldy1wcm9jZXNzIGNhbGlicmF0aW9uYCByb3cg
ZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsgcmVtb3ZlIGJvdGggRDExIGxlZGdlciBlbnRyaWVzIGZv
ciB0aGlzIGJhdGNoOyBsZWF2ZSBCYXRjaCAxIHBsdXMgRDPigJNEMTAgb3duZXJzLCBgaW5kZXgv
Y3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9gLCBhbmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRo
ZSBmaXZlLXBhdGggZmVuY2UgdW5jaGFuZ2VkIHwKIHwgRDExIHwgW2AwNS1kZWJ1Zy1yZWdyZXNz
aW9uLWxlZGdlci5tZGBdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kKSB8IEJhc2Vs
aW5lIGJlaGF2aW9ycyB0aGF0IGFsbCB0cmFja3MgcHJlc2VydmUgfCBbYGJhc2VsaW5lLWJlaGF2
aW9ycy10aGF0LWFsbC10cmFja3MtcHJlc2VydmVgXSguLi8wNS1kZWJ1Zy1yZWdyZXNzaW9uLWxl
ZGdlci5tZCNiYXNlbGluZS1iZWhhdmlvcnMtdGhhdC1hbGwtdHJhY2tzLXByZXNlcnZlKSB8IGxp
bmVzIDgw4oCTOTIgKDM2NDQgYnl0ZXMpIHwgYDVmOWVjOGQ5ODMyNDAxNTk4NmM4NDYyMDhkYjFj
NTY3YjQ5YzkwOWJhYTcxNDQyOGFjM2E2NDQ3MDlmZTkxMzJgIHwgW2AuLi9ldmlkZW5jZS9iYXNl
bGluZS1iZWhhdmlvcnMtYW5kLXNtb2tlLXNjZW5hcmlvcy5tZGBdKC4uL2V2aWRlbmNlL2Jhc2Vs
aW5lLWJlaGF2aW9ycy1hbmQtc21va2Utc2NlbmFyaW9zLm1kKSB8IGV2aWRlbmNlL3JlZ3Jlc3Np
b24gfCBjYW5vbmljYWwgY29tcG91bmQgZGVzdGluYXRpb247IHNvdXJjZSBjb21wYXRpYmlsaXR5
IGFuY2hvcjsgc2VwYXJhdGUgZXhhY3QtYm9keSBwcm92ZW5hbmNlIHByZXNlcnZlZCB8IHBhaXJl
ZCBmYW1pbHkgYXRvbWljaXR5IHdpdGggYENyb3NzLWdhdGUgc21va2Ugc2NlbmFyaW9zYDsgZXhh
Y3QgMzY0NC1ieXRlIHNvdXJjZSBib2R5IHByZXNlcnZlZCBiZXR3ZWVuIHRoZSBgYmFzZWxpbmUt
YmVoYXZpb3JzYCBvd25lciBib3VuZGFyeSBtYXJrZXJzIGFuZCBub3QgcmVwcmVzZW50ZWQgYXMg
b25lIGNvbnRpZ3VvdXMgZXh0cmFjdGlvbiB3aXRoIHRoZSBsYXRlciBzbW9rZS1zY2VuYXJpbyBi
b2R5IHwgcmVzdG9yZSB0aGUgZXhhY3QgMzY0NC1ieXRlIHNvdXJjZSBib2R5IGF0IGAwNS1kZWJ1
Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNiYXNlbGluZS1iZWhhdmlvcnMtdGhhdC1hbGwtdHJhY2tz
LXByZXNlcnZlYCBhbmQgdGhlIGV4YWN0IDExNzE0LWJ5dGUgc291cmNlIGJvZHkgYXQgYDA1LWRl
YnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2Nyb3NzLWdhdGUtc21va2Utc2NlbmFyaW9zYCBpbiBv
cmlnaW5hbCBvcmRlcjsgcmVtb3ZlIGBldmlkZW5jZS9iYXNlbGluZS1iZWhhdmlvcnMtYW5kLXNt
b2tlLXNjZW5hcmlvcy5tZGA7IHJlbW92ZSB0aGUgc29sZSBCYXRjaC0zIHJvdyBmcm9tIGBldmlk
ZW5jZS9SRUFETUUubWRgOyByZW1vdmUgdGhlIGV4YWN0IGBCYXNlbGluZSBiZWhhdmlvcnMgKyBD
cm9zcy1nYXRlIHNtb2tlIHNjZW5hcmlvc2Agcm93IGZyb20gYGluZGV4L1JFQURNRS5tZGA7IHJl
bW92ZSBib3RoIEQxMSBsZWRnZXIgZW50cmllcyBmb3IgdGhpcyBiYXRjaDsgbGVhdmUgQmF0Y2gg
MSwgQmF0Y2ggMiwgRDPigJNEMTAgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGByZXZpZXct
Y29udHJvbC9gLCBhbmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmaXZlLXBhdGggZmVuY2UgdW5j
aGFuZ2VkIHwKIHwgRDExIHwgW2AwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZGBdKC4uLzA1
LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kKSB8IENyb3NzLWdhdGUgc21va2Ugc2NlbmFyaW9z
IHwgW2Bjcm9zcy1nYXRlLXNtb2tlLXNjZW5hcmlvc2BdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24t
bGVkZ2VyLm1kI2Nyb3NzLWdhdGUtc21va2Utc2NlbmFyaW9zKSB8IGxpbmVzIDkz4oCTMTkyICgx
MTcxNCBieXRlcykgfCBgNTg2ZTU0MDJiOTJiNmU0ZTk4YWI3Y2JkZDA0ZTY3MTIxZmYyMDljY2Yy
M2M3ZjFmZjExNWMxODhiZWRiY2E4NWAgfCBbYC4uL2V2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9y
cy1hbmQtc21va2Utc2NlbmFyaW9zLm1kYF0oLi4vZXZpZGVuY2UvYmFzZWxpbmUtYmVoYXZpb3Jz
LWFuZC1zbW9rZS1zY2VuYXJpb3MubWQpIHwgZXZpZGVuY2UvcmVncmVzc2lvbiB8IGNhbm9uaWNh
bCBjb21wb3VuZCBkZXN0aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9yOyBzZXBh
cmF0ZSBleGFjdC1ib2R5IHByb3ZlbmFuY2UgcHJlc2VydmVkIHwgcGFpcmVkIGZhbWlseSBhdG9t
aWNpdHkgd2l0aCBgQmFzZWxpbmUgYmVoYXZpb3JzIHRoYXQgYWxsIHRyYWNrcyBwcmVzZXJ2ZWA7
IGV4YWN0IDExNzE0LWJ5dGUgc291cmNlIGJvZHkgcHJlc2VydmVkIGJldHdlZW4gdGhlIGBjcm9z
cy1nYXRlLXNtb2tlLXNjZW5hcmlvc2Agb3duZXIgYm91bmRhcnkgbWFya2VycyB3aXRoIG9yZGVy
ZWQgc2NlbmFyaW8gc3ViaGVhZGluZ3MsIGRldGVybWluaXN0aWMgcGFya2VkLXN1Y2Nlc3NvciBz
ZXF1ZW5jZSwgYW5kIENvdmVycyBsaW5lcyBpbnRlcm5hbCB0byB0aGUgcHJlc2VydmVkIGJvZHkg
fCByZXN0b3JlIHRoZSBleGFjdCAzNjQ0LWJ5dGUgc291cmNlIGJvZHkgYXQgYDA1LWRlYnVnLXJl
Z3Jlc3Npb24tbGVkZ2VyLm1kI2Jhc2VsaW5lLWJlaGF2aW9ycy10aGF0LWFsbC10cmFja3MtcHJl
c2VydmVgIGFuZCB0aGUgZXhhY3QgMTE3MTQtYnl0ZSBzb3VyY2UgYm9keSBhdCBgMDUtZGVidWct
cmVncmVzc2lvbi1sZWRnZXIubWQjY3Jvc3MtZ2F0ZS1zbW9rZS1zY2VuYXJpb3NgIGluIG9yaWdp
bmFsIG9yZGVyOyByZW1vdmUgYGV2aWRlbmNlL2Jhc2VsaW5lLWJlaGF2aW9ycy1hbmQtc21va2Ut
c2NlbmFyaW9zLm1kYDsgcmVtb3ZlIHRoZSBzb2xlIEJhdGNoLTMgcm93IGZyb20gYGV2aWRlbmNl
L1JFQURNRS5tZGA7IHJlbW92ZSB0aGUgZXhhY3QgYEJhc2VsaW5lIGJlaGF2aW9ycyArIENyb3Nz
LWdhdGUgc21va2Ugc2NlbmFyaW9zYCByb3cgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsgcmVtb3Zl
IGJvdGggRDExIGxlZGdlciBlbnRyaWVzIGZvciB0aGlzIGJhdGNoOyBsZWF2ZSBCYXRjaCAxLCBC
YXRjaCAyLCBEM+KAk0QxMCBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYHJldmlldy1jb250
cm9sL2AsIGFuZCBldmVyeSBwYXRoIG91dHNpZGUgdGhlIGZpdmUtcGF0aCBmZW5jZSB1bmNoYW5n
ZWQgfAotfCBEMTEgfCBbYDA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kYF0oLi4vMDUtZGVi
dWctcmVncmVzc2lvbi1sZWRnZXIubWQpIHwgSG93IHRvIHJlYWQgdGhpcyBsZWRnZXIgfCBbYGhv
dy10by1yZWFkLXRoaXMtbGVkZ2VyYF0oLi4vMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQj
aG93LXRvLXJlYWQtdGhpcy1sZWRnZXIpIHwgbGluZXMgM+KAkzE1ICg4MjMgYnl0ZXMpIHwgYDgw
ZjRiNzUzZDZlNTc0MjAyYjUyNzU4OWQzMzAyMWM3MzVhNzZlMmM2NTYzZmM1ZGNlODkwZGY1MjRj
OTMwMDJgIHwgW2AuLi9ldmlkZW5jZS9SRUFETUUubWRgXSguLi9ldmlkZW5jZS9SRUFETUUubWQp
IHwgZXZpZGVuY2UgaW5kZXggfCBjYW5vbmljYWwgZGVzdGluYXRpb24gZm9yIHRoZSBleGFjdCBy
ZWFkaW5nL2ludGVycHJldGF0aW9uIGJvZHkgb25seTsgcmVjZWlwdC9jdXJyZW50LXN0YXRlIHJv
d3MgcmVtYWluIG5vbi1hdXRob3JpdGF0aXZlIG5hdmlnYXRpb24gfCBub25lOyBleGFjdCA4MjMt
Ynl0ZSBzb3VyY2UgYm9keSBwcmVzZXJ2ZWQgYmV0d2VlbiB0aGUgYGV4YWN0LWV4dHJhY3RlZC1i
b2R5Omhvdy10by1yZWFkLXRoaXMtbGVkZ2VyOnN0YXJ0L2VuZGAgbWFya2VycyBpbiBgZXZpZGVu
Y2UvUkVBRE1FLm1kYDsgbWV0YWRhdGEgYW5kIEJhdGNoIDQgbmF2aWdhdGlvbiByb3dzIHJlbWFp
biBvdXRzaWRlIHRoZSBwcmVzZXJ2ZWQgYm9keSwgYW5kIHRoZSB0aHJlZSBvcGFxdWUgTWFya2Rv
d24gbGlua3MgcmVtYWluIGJ5dGUtaWRlbnRpY2FsIHwgcmVzdG9yZSB0aGUgZXhhY3QgODIzLWJ5
dGUgcm9vdCBib2R5IGF0IGAwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZCNob3ctdG8tcmVh
ZC10aGlzLWxlZGdlcmA7IHJlc3RvcmUgdGhlIHByZS1CYXRjaDQgYGV2aWRlbmNlL1JFQURNRS5t
ZGAgbWV0YWRhdGEgYW5kIHJlbW92ZSB0aGUgZXhhY3QgbWFya2VyIGJvZHkgcGx1cyB0aGUgZXhh
Y3QgcmVjZWlwdC9jdXJyZW50LXN0YXRlIG5hdmlnYXRpb24gcm93czsgcmVtb3ZlIGBldmlkZW5j
ZS9yZWNlaXB0cy9SRUFETUUubWRgOyByZW1vdmUgdGhlIGV4YWN0IGBzaGFyZWQtZXZpZGVuY2Ut
bmF2aWdhdGlvbmAgcm93IGZyb20gYGluZGV4L1JFQURNRS5tZGA7IHJlbW92ZSB0aGlzIEQxMSBs
ZWRnZXIgZW50cnk7IHByZXNlcnZlIEQxMSBCYXRjaGVzIDHigJMzLCBEM+KAk0QxMCBvd25lcnMs
IHRoZSBleGlzdGluZyBENiBjdXJyZW50LXN0YXRlIG93bmVyL3Byb2plY3Rpb24sIGBpbmRleC9j
dXJyZW50Lm1kYCwgYHJldmlldy1jb250cm9sL2AsIHRoZSBvcGFxdWUgYDAwYCBsaW5rLCBhbmQg
ZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmaXZlLXBhdGggZmVuY2UgdW5jaGFuZ2VkIHwKK3wgRDEx
IHwgW2AwNS1kZWJ1Zy1yZWdyZXNzaW9uLWxlZGdlci5tZGBdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Np
b24tbGVkZ2VyLm1kKSB8IEhvdyB0byByZWFkIHRoaXMgbGVkZ2VyIHwgW2Bob3ctdG8tcmVhZC10
aGlzLWxlZGdlcmBdKC4uLzA1LWRlYnVnLXJlZ3Jlc3Npb24tbGVkZ2VyLm1kI2hvdy10by1yZWFk
LXRoaXMtbGVkZ2VyKSB8IGxpbmVzIDPigJMxNSAoODIzIGJ5dGVzKSB8IGA4MGY0Yjc1M2Q2ZTU3
NDIwMmI1Mjc1ODlkMzMwMjFjNzM1YTc2ZTJjNjU2M2ZjNWRjZTg5MGRmNTI0YzkzMDAyYCB8IFtg
Li4vZXZpZGVuY2UvUkVBRE1FLm1kYF0oLi4vZXZpZGVuY2UvUkVBRE1FLm1kKSB8IGV2aWRlbmNl
IGluZGV4IHwgY2Fub25pY2FsIGRlc3RpbmF0aW9uIGZvciB0aGUgZXhhY3QgcmVhZGluZy9pbnRl
cnByZXRhdGlvbiBib2R5IG9ubHk7IHJlY2VpcHQvY3VycmVudC1zdGF0ZSByb3dzIHJlbWFpbiBu
b24tYXV0aG9yaXRhdGl2ZSBuYXZpZ2F0aW9uIHwgbm9uZTsgZXhhY3QgODIzLWJ5dGUgc291cmNl
IGJvZHkgcHJlc2VydmVkIGJldHdlZW4gdGhlIGBleGFjdC1leHRyYWN0ZWQtYm9keTpob3ctdG8t
cmVhZC10aGlzLWxlZGdlcjpzdGFydC9lbmRgIG1hcmtlcnMgaW4gYGV2aWRlbmNlL1JFQURNRS5t
ZGA7IG1ldGFkYXRhIGFuZCBCYXRjaCA0IG5hdmlnYXRpb24gcm93cyByZW1haW4gb3V0c2lkZSB0
aGUgcHJlc2VydmVkIGJvZHksIGFuZCB0aGUgdGhyZWUgb3BhcXVlIE1hcmtkb3duIGxpbmtzIHJl
bWFpbiBieXRlLWlkZW50aWNhbCB8IHJlc3RvcmUgdGhlIGV4YWN0IDgyMy1ieXRlIHJvb3QgYm9k
eSBhdCBgMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjaG93LXRvLXJlYWQtdGhpcy1sZWRn
ZXJgOyByZXN0b3JlIHRoZSBwcmUtQmF0Y2g0IGBldmlkZW5jZS9SRUFETUUubWRgIG1ldGFkYXRh
LCBpbmNsdWRpbmcgdGhlIGV4YWN0IGF1dGhvcml0eS1ib3VuZGFyeSBzZW50ZW5jZSBgPiAqKkF1
dGhvcml0eSBib3VuZGFyeToqKiBUaGlzIGZpbGUgaXMgbmF2aWdhdGlvbiBvbmx5LiBJdCBkb2Vz
IG5vdCBhdXRob3JpemUgaW1wbGVtZW50YXRpb24sIGNsb3NlIGEgcmVncmVzc2lvbiBnYXRlLCBz
YXRpc2Z5IGEgcHJvb2Ygb2JsaWdhdGlvbiwgcHJvbW90ZSBldmlkZW5jZSBpbnRvIGF1dGhvcml0
eSwgb3IgcmVwbGFjZSB0aGUgbGlua2VkIGNhbm9uaWNhbCBvd25lcnMuYCwgYW5kIHJlbW92ZSB0
aGUgZXhhY3QgbWFya2VyIGJvZHkgcGx1cyB0aGUgZXhhY3QgcmVjZWlwdC9jdXJyZW50LXN0YXRl
IG5hdmlnYXRpb24gcm93czsgcmVtb3ZlIGBldmlkZW5jZS9yZWNlaXB0cy9SRUFETUUubWRgOyBy
ZW1vdmUgdGhlIGV4YWN0IGBzaGFyZWQtZXZpZGVuY2UtbmF2aWdhdGlvbmAgcm93IGZyb20gYGlu
ZGV4L1JFQURNRS5tZGA7IHJlbW92ZSB0aGlzIEQxMSBsZWRnZXIgZW50cnk7IHByZXNlcnZlIEQx
MSBCYXRjaGVzIDHigJMzLCBEM+KAk0QxMCBvd25lcnMsIHRoZSBleGlzdGluZyBENiBjdXJyZW50
LXN0YXRlIG93bmVyL3Byb2plY3Rpb24sIGBpbmRleC9jdXJyZW50Lm1kYCwgYHJldmlldy1jb250
cm9sL2AsIHRoZSBvcGFxdWUgYDAwYCBsaW5rLCBhbmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBm
aXZlLXBhdGggZmVuY2UgdW5jaGFuZ2VkIHwKYGBgClZhbGlkYXRpb24gZXZpZGVuY2UgKFNIQS0y
NTYgYzAzZmQ4MTAyOTU0Mjc2OTBkZGFkZDhjMDQxNDM0ZDk5OWM4Y2Q4ODc1MWRlNDg1OGRjMzdi
NzFkM2M2OTdiMyk6CkRhdGU6IDIwMjYtMDgtMjcKSEVBRCAzZmJhMTRjOGE4YWNiOTBiYzNkZTQw
YTc1MDM1ZTJhMGY1YjVmZDc2ClRSRUUgMGJjZjdlYzhiNTFhZGE3MjBhY2FhMzIxOTM0Y2JmYjM2
OGNjMDhjNApTVEFUVVMKIE0gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yLzA1LWRlYnVn
LXJlZ3Jlc3Npb24tbGVkZ2VyLm1kCiBNIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9l
dmlkZW5jZS9SRUFETUUubWQKIE0gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4
L1JFQURNRS5tZAogTSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4
dHJhY3Rpb24tbGVkZ2VyLm1kCj8/IGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9ldmlk
ZW5jZS9yZWNlaXB0cy9SRUFETUUubWQKCkRpcnR5IGZlbmNlIHZhbGlkYXRlZDogZXhhY3QgZml2
ZSBwYXRocyBkaXJ0eTsgbm90aGluZyBzdGFnZWQuCmdpdCBkaWZmIC0tY2hlY2s6IGNsZWFuCmdp
dCBkaWZmIC0tbm8taW5kZXggLS1jaGVjayAvZGV2L251bGwgcmVjZWlwdHMgUkVBRE1FOiBjbGVh
bgpDYW5kaWRhdGUgY29tcGFyaXNvbjogb25seSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0
b3IvbWlncmF0aW9uL2V4dHJhY3Rpb24tbGVkZ2VyLm1kIGRpZmZlcnMgZnJvbSB0aGUgcmV2aWV3
ZWQgY2FuZGlkYXRlOyB0aGUgb3RoZXIgZm91ciBjYW5kaWRhdGUgcGF0aHMgYXJlIGJ5dGUtaWRl
bnRpY2FsLgpSb3ctb25seSByZW1lZGlhdGlvbiB2YWxpZGF0ZWQ6IG9uZSByZXBsYWNlbWVudCBs
aW5lIGF0IGV4dHJhY3Rpb24tbGVkZ2VyIGxpbmUgMjA0OyBubyBkaWZmIG91dHNpZGUgdGhlIHNp
bmdsZSByb2xsYmFjayBjZWxsIHJlbGF0aXZlIHRvIHRoZSByZXZpZXdlZCBjYW5kaWRhdGUgYXJ0
aWZhY3QuCkV2aWRlbmNlIFJFQURNRSBtYXJrZXIgYm9keSBtYXRjaGVzIHRoZSBleGFjdCA4MjMt
Ynl0ZSBzb3VyY2UgYXJ0aWZhY3Q7IHNoYTI1Nj04MGY0Yjc1M2Q2ZTU3NDIwMmI1Mjc1ODlkMzMw
MjFjNzM1YTc2ZTJjNjU2M2ZjNWRjZTg5MGRmNTI0YzkzMDAyIHVuY2hhbmdlZC4KUm9vdCBhdWRp
dCBjb3VudHMgdW5jaGFuZ2VkOiBoZWFkaW5nc19iZWxvd19IMT04MywgcG9pbnRlcl9saW5lcz04
MywgcmVzaWR1YWw9MC4KTWFya2Rvd24gbGluay9hbmNob3IgdmFsaWRhdGlvbiBwYXNzZWQ6IGNo
ZWNrZWQ9MTk1Mywgc2tpcHBlZF9vcGFxdWU9NCwgZmFpbHVyZXM9MC4KT1BBUVVFX1NLSVAgMDAt
UkVBRE1FLm1kOjUgLi4vLi4vc3Vic3RyYXRlLXJ1bnRpbWUtcmVmYWN0b3ItZGlyZWN0aXZlLXJl
dmlzZWQubWQKT1BBUVVFX1NLSVAgZXZpZGVuY2UvUkVBRE1FLm1kOjI4IC4uLy4uL1JVTl9XT1JM
RF9UQVNLX0RFQlVHX0NBTk9OSUNBTC5tZApPUEFRVUVfU0tJUCBldmlkZW5jZS9SRUFETUUubWQ6
MjkgLi4vLi4vQ09OVElOVUVfV09STERfV09SS0VSX0JMT0NLSU5HX0RFVklBVElPTl9ERUJVRy5t
ZApPUEFRVUVfU0tJUCBldmlkZW5jZS9SRUFETUUubWQ6MzAgLi4vLi4vQ09ERVhfV09STERfRElT
UEFUQ0hfR0FQX1dSSVRFVVAubWQKQ3VycmVudC1zdGF0ZSBvd25lciBzdGFibGU6IGxsbS1sYXN0
LW1pbGUvcnVudGltZS1yZWZhY3Rvci9tYWNvcy1kZXYtcGFyaXR5L2V2aWRlbmNlLXJlZ3Jlc3Np
b24ubWQgc2hhMjU2PTJhMzYyYzcwZmRiZjgwYTJhNjc2NDNjOWJjZjZlNDQxOTY1M2NiYzlmNjUy
Y2MxMmUxN2I3MmI5ODVhNmE2NzguClByaW9yIEQzL0Q1LUQxMCBvd25lcnMsIEQxMSBCYXRjaDEt
MyBvd25lcnMsIGFuZCByZXZpZXctY29udHJvbCByZW1haW4gc3RhYmxlIGJlY2F1c2UgdGhlIGV4
YWN0IGRpcnR5IGZlbmNlIGlzIGxpbWl0ZWQgdG8gdGhlIHNhbWUgZml2ZSBjYW5kaWRhdGUgcGF0
aHMgYW5kIG5vIHN0YWdlZCBjaGFuZ2VzIGV4aXN0LgpSZW1lZGlhdGlvbiBkZWx0YSBmb3J3YXJk
L3JldmVyc2UgYXBwbHkgc3VjY2VlZGVkIGFnYWluc3QgYSBmaWxlc3lzdGVtIGNvcHkgb2YgdGhl
IHJldmlld2VkIGNhbmRpZGF0ZSBhbmQgcmV0dXJuZWQgdG8gYnl0ZS1pZGVudGljYWwgY2FuZGlk
YXRlIHN0YXRlLgpGdWxsIHJlbWVkaWF0ZWQgY2FuZGlkYXRlIHBhdGNoIGZvcndhcmQvcmV2ZXJz
ZSBhcHBseSBzdWNjZWVkZWQgYWdhaW5zdCBhIGNsZWFuIGJhc2VsaW5lIGV4cG9ydCBhbmQgcmV0
dXJuZWQgdG8gYSBieXRlLWlkZW50aWNhbCBiYXNlbGluZSB0cmVlLgpjYW5kaWRhdGVfcGF0Y2hf
c2hhMjU2PTJlMWU5Njg3ZjZmMGU1OTkxYmJkZThhYzZhMmNiZjMyMTgwY2UyNjA0ODEwZjMxZGZj
MzdlNzVmYTFjN2ZlNGUKcmVtZWRpYXRpb25fZGVsdGFfc2hhMjU2PTE3ZjdiM2UyNzc5YTc3ZWE0
ZDkxMGQwMWJjMTc5NDRlYzBhYTRhNDlmNDUyMmFhMDFkZDEwYzhkZDRjZWU3MjMKcmVtZWRpYXRl
ZF9jYW5kaWRhdGVfc2hhMjU2PWIzMTk4YzE1N2M4MmFiOWFkODlmZjE2NjkxZjYwNzQyN2Q3ODVj
Y2Y5ODIzMTcwYjUzYmZiZjQ5Nzk2ZmQ3ODYKb3JkZXJlZF9pbml0aWFsX3BsdXNfZGVsdGFfc2hh
MjU2PTc2MTFmZmEyZGVlNTZlOWIwNDQ3MzIwZGMyYTUwOGZmNTU1ZmVmMzQ3OTBkN2Y2YWQxOGUz
Y2MwYWFhNDJkOTYKCgpEbyBub3QgcmVkdWNlIHRoZSB0YXNrIG9yIHJlcGxhY2UgaXQgd2l0aCBh
biBlYXNpZXIgYWx0ZXJuYXRpdmUuIFByZXNlcnZlIHRoZSByZXF1ZXN0ZWQgc2NvcGUgYW5kIHBy
b2plY3QgY29udmVudGlvbnMuCgpSZXBvcnQgYSBibG9ja2luZyBmaW5kaW5nIG9ubHkgd2hlbiBp
dCBpcyByZWFjaGFibGUgdGhyb3VnaCBzdXBwb3J0ZWQgYmVoYXZpb3IsIHZpb2xhdGVzIGFuIGlu
LXNjb3BlIGludmFyaWFudCBvciBhY2NlcHRlZCByZW1lZGlhdGlvbiBvYmplY3RpdmUsIG1lZXRz
IHRoZSBtYXRlcmlhbC1jb25zZXF1ZW5jZSB0aHJlc2hvbGQsIGFuZCB3YXMgaW50cm9kdWNlZCBi
eSB0aGUgZGVsdGEsIGlzIGEgcmVncmVzc2lvbiBjYXVzZWQgYnkgaXQsIG9yIHByb3ZlcyB0aGUg
YWNjZXB0ZWQgcm9vdCBjYXVzZSByZW1haW5zIHVuZml4ZWQuIFlvdSBtYXkgaW5zcGVjdCB1bmNo
YW5nZWQgc3VwcG9ydGluZyBjb250ZXh0IG9ubHkgdG8gZXN0YWJsaXNoIGNhdXNhbGl0eTsgdW5j
aGFuZ2VkIHByZS1leGlzdGluZyBiZWhhdmlvciBpcyBub3QgYW4gaW4tc2NvcGUgZmluZGluZyBt
ZXJlbHkgYmVjYXVzZSB5b3Ugb2JzZXJ2ZWQgaXQuCgpBZnRlciB0aGUgYWNjZXB0ZWQgZmluZGlu
ZyBpcyBmaXhlZCBhbmQgbm8gcXVhbGlmeWluZyBkZWx0YSByZWdyZXNzaW9uIHJlbWFpbnMsIHN0
b3AuIExhYmVsIGFkamFjZW50IG9ic2VydmF0aW9ucyBhcyBkZWZlcnJlZCwgbm90IGJsb2NraW5n
LiBSZXR1cm4gZmluZGluZ3MgZmlyc3QgYnkgc2V2ZXJpdHkgd2l0aCBleGFjdCBmaWxlL3NlY3Rp
b24sIHN1cHBvcnRlZC1pbnB1dCByZWFjaGFiaWxpdHksIHZpb2xhdGVkIGludmFyaWFudCwgbWF0
ZXJpYWwgY29uc2VxdWVuY2UsIGFuZCBkZWx0YSBjYXVzYWxpdHkuIFN0YXRlIGV4cGxpY2l0bHkg
d2hlbiB0aGVyZSBhcmUgbm8gcXVhbGlmeWluZyBmaW5kaW5ncy4gRW5kIHdpdGggZXhhY3RseSBv
bmUgdmVyZGljdCB0b2tlbiBvbiBpdHMgb3duIGxpbmU6IEFQUFJPVkVEIG9yIFJFUVVFU1RfQ0hB
TkdFUy4K
```

</details>

<details>
<summary>Remediation-follow-up review answer (rendered copy)</summary>

<pre>
Findings

No qualifying P1/P2 findings.

The accepted P2 rollback defect is fully resolved. The remediation changes the D11 Batch 4 rollback recipe so that a supported reader following it literally must restore the pre-Batch4 evidence/README.md metadata including the exact authority-boundary sentence, before removing the extracted marker body and Batch 4 navigation rows. The inserted sentence matches the required pre-Batch4 text exactly.

File/section: llm-last-mile/runtime-refactor/migration/extraction-ledger.md — D11 How to read this ledger rollback cell.

Supported-input reachability: Literal execution of the remediated rollback now restores the authority-boundary sentence that the original recipe omitted. Consequently, after the marker/body is removed, evidence/README.md says that the file is navigation only rather than retaining the Batch 4 claim that a now-removed marked body is canonical there. This directly removes the reachable false-ownership state identified by the initial review.

In-scope invariants: The changed recipe explicitly restores the required exact sentence while retaining the existing restoration/removal/preservation operations. The supplied validation also establishes that only migration/extraction-ledger.md differs from the reviewed candidate, with one replacement line confined to the single rollback cell; the other four candidate paths remain byte-identical.

Material consequence: The previously reachable material inconsistency—removed canonical body accompanied by a surviving claim that the body remained canonical—is no longer reachable through the documented literal rollback. No new supported rollback, provenance, authority/status, source/hash, or path/phase-fencing regression is introduced by this delta.

Delta causality: The sole semantic addition is precisely the missing rollback instruction identified in the accepted finding. The supplied forward/reverse validation further reports successful application of both the remediation delta against the reviewed candidate and the full remediated candidate against the clean baseline, returning byte-identically in each case.

APPROVED
</pre>

</details>

<details>
<summary>Remediation-follow-up review answer exact bytes (base64 UTF-8)</summary>

```text
RmluZGluZ3MKCk5vIHF1YWxpZnlpbmcgUDEvUDIgZmluZGluZ3MuCgpUaGUgYWNjZXB0ZWQgUDIg
cm9sbGJhY2sgZGVmZWN0IGlzIGZ1bGx5IHJlc29sdmVkLiBUaGUgcmVtZWRpYXRpb24gY2hhbmdl
cyB0aGUgRDExIEJhdGNoIDQgcm9sbGJhY2sgcmVjaXBlIHNvIHRoYXQgYSBzdXBwb3J0ZWQgcmVh
ZGVyIGZvbGxvd2luZyBpdCBsaXRlcmFsbHkgbXVzdCByZXN0b3JlIHRoZSBwcmUtQmF0Y2g0IGV2
aWRlbmNlL1JFQURNRS5tZCBtZXRhZGF0YSBpbmNsdWRpbmcgdGhlIGV4YWN0IGF1dGhvcml0eS1i
b3VuZGFyeSBzZW50ZW5jZSwgYmVmb3JlIHJlbW92aW5nIHRoZSBleHRyYWN0ZWQgbWFya2VyIGJv
ZHkgYW5kIEJhdGNoIDQgbmF2aWdhdGlvbiByb3dzLiBUaGUgaW5zZXJ0ZWQgc2VudGVuY2UgbWF0
Y2hlcyB0aGUgcmVxdWlyZWQgcHJlLUJhdGNoNCB0ZXh0IGV4YWN0bHkuCgpGaWxlL3NlY3Rpb246
IGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRn
ZXIubWQg4oCUIEQxMSBIb3cgdG8gcmVhZCB0aGlzIGxlZGdlciByb2xsYmFjayBjZWxsLgoKU3Vw
cG9ydGVkLWlucHV0IHJlYWNoYWJpbGl0eTogTGl0ZXJhbCBleGVjdXRpb24gb2YgdGhlIHJlbWVk
aWF0ZWQgcm9sbGJhY2sgbm93IHJlc3RvcmVzIHRoZSBhdXRob3JpdHktYm91bmRhcnkgc2VudGVu
Y2UgdGhhdCB0aGUgb3JpZ2luYWwgcmVjaXBlIG9taXR0ZWQuIENvbnNlcXVlbnRseSwgYWZ0ZXIg
dGhlIG1hcmtlci9ib2R5IGlzIHJlbW92ZWQsIGV2aWRlbmNlL1JFQURNRS5tZCBzYXlzIHRoYXQg
dGhlIGZpbGUgaXMgbmF2aWdhdGlvbiBvbmx5IHJhdGhlciB0aGFuIHJldGFpbmluZyB0aGUgQmF0
Y2ggNCBjbGFpbSB0aGF0IGEgbm93LXJlbW92ZWQgbWFya2VkIGJvZHkgaXMgY2Fub25pY2FsIHRo
ZXJlLiBUaGlzIGRpcmVjdGx5IHJlbW92ZXMgdGhlIHJlYWNoYWJsZSBmYWxzZS1vd25lcnNoaXAg
c3RhdGUgaWRlbnRpZmllZCBieSB0aGUgaW5pdGlhbCByZXZpZXcuCgpJbi1zY29wZSBpbnZhcmlh
bnRzOiBUaGUgY2hhbmdlZCByZWNpcGUgZXhwbGljaXRseSByZXN0b3JlcyB0aGUgcmVxdWlyZWQg
ZXhhY3Qgc2VudGVuY2Ugd2hpbGUgcmV0YWluaW5nIHRoZSBleGlzdGluZyByZXN0b3JhdGlvbi9y
ZW1vdmFsL3ByZXNlcnZhdGlvbiBvcGVyYXRpb25zLiBUaGUgc3VwcGxpZWQgdmFsaWRhdGlvbiBh
bHNvIGVzdGFibGlzaGVzIHRoYXQgb25seSBtaWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQg
ZGlmZmVycyBmcm9tIHRoZSByZXZpZXdlZCBjYW5kaWRhdGUsIHdpdGggb25lIHJlcGxhY2VtZW50
IGxpbmUgY29uZmluZWQgdG8gdGhlIHNpbmdsZSByb2xsYmFjayBjZWxsOyB0aGUgb3RoZXIgZm91
ciBjYW5kaWRhdGUgcGF0aHMgcmVtYWluIGJ5dGUtaWRlbnRpY2FsLgoKTWF0ZXJpYWwgY29uc2Vx
dWVuY2U6IFRoZSBwcmV2aW91c2x5IHJlYWNoYWJsZSBtYXRlcmlhbCBpbmNvbnNpc3RlbmN54oCU
cmVtb3ZlZCBjYW5vbmljYWwgYm9keSBhY2NvbXBhbmllZCBieSBhIHN1cnZpdmluZyBjbGFpbSB0
aGF0IHRoZSBib2R5IHJlbWFpbmVkIGNhbm9uaWNhbOKAlGlzIG5vIGxvbmdlciByZWFjaGFibGUg
dGhyb3VnaCB0aGUgZG9jdW1lbnRlZCBsaXRlcmFsIHJvbGxiYWNrLiBObyBuZXcgc3VwcG9ydGVk
IHJvbGxiYWNrLCBwcm92ZW5hbmNlLCBhdXRob3JpdHkvc3RhdHVzLCBzb3VyY2UvaGFzaCwgb3Ig
cGF0aC9waGFzZS1mZW5jaW5nIHJlZ3Jlc3Npb24gaXMgaW50cm9kdWNlZCBieSB0aGlzIGRlbHRh
LgoKRGVsdGEgY2F1c2FsaXR5OiBUaGUgc29sZSBzZW1hbnRpYyBhZGRpdGlvbiBpcyBwcmVjaXNl
bHkgdGhlIG1pc3Npbmcgcm9sbGJhY2sgaW5zdHJ1Y3Rpb24gaWRlbnRpZmllZCBpbiB0aGUgYWNj
ZXB0ZWQgZmluZGluZy4gVGhlIHN1cHBsaWVkIGZvcndhcmQvcmV2ZXJzZSB2YWxpZGF0aW9uIGZ1
cnRoZXIgcmVwb3J0cyBzdWNjZXNzZnVsIGFwcGxpY2F0aW9uIG9mIGJvdGggdGhlIHJlbWVkaWF0
aW9uIGRlbHRhIGFnYWluc3QgdGhlIHJldmlld2VkIGNhbmRpZGF0ZSBhbmQgdGhlIGZ1bGwgcmVt
ZWRpYXRlZCBjYW5kaWRhdGUgYWdhaW5zdCB0aGUgY2xlYW4gYmFzZWxpbmUsIHJldHVybmluZyBi
eXRlLWlkZW50aWNhbGx5IGluIGVhY2ggY2FzZS4KCkFQUFJPVkVECg==
```

</details>
