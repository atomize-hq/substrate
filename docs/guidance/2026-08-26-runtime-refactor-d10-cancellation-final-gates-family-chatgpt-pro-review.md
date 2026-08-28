# ChatGPT Pro advisory review: runtime-refactor D10 cancellation/final-gates family

- Date: 2026-08-26
- Bound baseline commit/tree: `2daccda74991c8b74530beaeb0b188fe7663d118` / `e0338ef0cd7367e483dab681bc1ed6b159f20f48`
- Candidate implementation subagent: `/root/d10_cancellation_final_gates_landing` (`gpt-5.4`, Extra High; display name not exposed)
- Closeout subagent: `/root/d10_cancellation_final_gates_closeout` (`gpt-5.4`, Extra High; display name not exposed)
- Root reviewer/adjudicator: `/root`
- Independent review chat: https://chatgpt.com/c/6a8f62f0-42fc-83e9-9590-b40b51222a61
- Review mode: `initial-range`
- Review context: fresh ChatGPT conversation
- ChatGPT account surface: `Pro`
- Visible reasoning-effort control: `Extra High`
- Exact model label: not exposed by the current visible ChatGPT UI
- Candidate patch: `sha256:bc3cf88f5f6cbc1f5531618aa3be1ddbbfe5c5e41326a8e9f092298b2725e801` at `/private/tmp/d10-cancellation-final-gates-family-candidate.patch`
- Review prompt: `sha256:589b048f72ac7201792ffb5207218d014f2b19524ec0a1426ccb723f55cb8f90` at `/private/tmp/d10-cancellation-final-gates-family-chatgpt-pro-initial-review-prompt.txt` (`36812 bytes`)
- Review answer: `sha256:1547bbd981aa063b044783bccefc32a001c022593f150375afca7f923fb2e4a2` at `/private/tmp/d10-cancellation-final-gates-family-chatgpt-pro-initial-review-answer.txt` (`1109 bytes`)
- Imported validation log: `sha256:4053b30fff99121ca9d9e23d637e9eea2c57e35ebb467a1e725d3012a39ef39b` at `/private/tmp/d10-cancellation-final-gates-family-validation.log`
- Root exhaustive coverage audit: `sha256:8100ecf7eb8f18768be2d50b1fe36a023ac8404522484cd96e3b3d1133793ab6` at `/private/tmp/d10-exhaustive-coverage-audit.txt`
- Exact embedded source body (Cancel outcome categories): baseline lines `119–143`, `1179 bytes`, `sha256:e8f373c9e7cbf16d286ff8f9a9146306644e430e3d630b7078ec7501b72ed4b3`
- Exact embedded source body (Final-receipt immutable `PolicySnapshotV3` acceptance rules): baseline lines `152–180`, `1675 bytes`, `sha256:672c9ba3db1e792a7afb145a4081450deafe6a1123262ae7c104c6f1b4b0b24a`
- Exact embedded source body (Dispatch narrowing monotonicity rules): baseline lines `181–214`, `1930 bytes`, `sha256:b12ef2b8c8bcffad5139e196df49ba2f59094ddf024f59d2d2a645eb9be2210a`
- Exact embedded source body (Contract promotion gates): baseline lines `215–231`, `1106 bytes`, `sha256:0fa77777b7f8ae85bc4e06082454b243677d89d070ff4c510bd9e148b61b2d43`
- Review verdict: `APPROVED`
- Review finding summary: `No qualifying findings exist.`
- Remediation rounds: `0`
- Review adjudication posture: root orchestrated review/adjudication; subagents did not self-approve
- Commit posture: candidate and closeout are local/unstaged, landed-pending-commit; root will validate, stage, GitNexus-check, and commit atomically
- Push posture: `not pushed`
- Successor authority posture: no successor dispatch authority is granted

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Candidate scope and outcome

This independently reviewed final substantive D10 family extracts `Cancel outcome categories` to `llm-last-mile/runtime-refactor/contracts/cancel-outcome-categories.md`, `Final-receipt immutable PolicySnapshotV3 acceptance rules` to `llm-last-mile/runtime-refactor/gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`, `Dispatch narrowing monotonicity rules` to `llm-last-mile/runtime-refactor/gates/dispatch-narrowing-monotonicity.md`, and `Contract promotion gates` to `llm-last-mile/runtime-refactor/gates/contract-promotion.md`; replaces only the four root spans in `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` with shallow compatibility pointers; adds four truthful navigation rows in `llm-last-mile/runtime-refactor/index/README.md`; and adds four matching D10 extraction-ledger rows in `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`. The family preserves the exact 1179-byte, 1675-byte, 1930-byte, and 1106-byte source bodies between owner boundary markers and forms one four-owner atomic rollback unit. The family is complete locally and approved after initial-range review with zero remediation and is landed pending one atomic commit. The root exhaustive audit proves every roadmap category `1–14` is accounted for and no genuinely blocked roadmap target remains, so D10 becomes fully complete when this candidate+closeout batch is atomically committed. Documentation extraction is not runtime implementation, evidence, gate satisfaction, promotion, or dispatch authority. D11 remains BLOCKED and separately unauthorized even after D10 completion. D12 remains blocked by D11 and is separately unauthorized. No successor dispatch authority is granted.

## Prospective nine-path landing manifest

Prospective landing manifest for this independently reviewed final family:

- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/contracts/cancel-outcome-categories.md`
- `llm-last-mile/runtime-refactor/gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`
- `llm-last-mile/runtime-refactor/gates/dispatch-narrowing-monotonicity.md`
- `llm-last-mile/runtime-refactor/gates/contract-promotion.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `docs/guidance/2026-08-26-runtime-refactor-d10-cancellation-final-gates-family-chatgpt-pro-review.md`

## Candidate path digests and imported review artifacts

Candidate path digests (reverified unchanged before and after closeout):

- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`: `sha256:2ff334d374a317cb5bf967225eaf17b832a8109e4e83e42415b28b6e80155165`
- `llm-last-mile/runtime-refactor/contracts/cancel-outcome-categories.md`: `sha256:436abc2077e0b0e51935c8b828bf552b27c5a26ff79420cda888d76f5597955a`
- `llm-last-mile/runtime-refactor/gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`: `sha256:bb9d51065d08844fe38761f245f22abad6c6cee5cd34b4280b344d4ec87a30f6`
- `llm-last-mile/runtime-refactor/gates/dispatch-narrowing-monotonicity.md`: `sha256:dc68bbf5c70e763f5f53c9cd122af5255135c02f5af0aa860e993efe9cd82370`
- `llm-last-mile/runtime-refactor/gates/contract-promotion.md`: `sha256:378de9e21eb2c0162d74dfef9da2b3a80d9c2af5478d80957e9a0df38d4b87f7`
- `llm-last-mile/runtime-refactor/index/README.md`: `sha256:aa60f0f606fa6b333c879598c22b1cf6475baacb353d8353e6ed9d9870c22c28`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`: `sha256:2c1115be700fb0a7d221e152574c5f9b139624eb79b6ff7ed906e270d39bdcf8`

Imported review artifacts:

- Candidate patch: `sha256:bc3cf88f5f6cbc1f5531618aa3be1ddbbfe5c5e41326a8e9f092298b2725e801`
- Review prompt: `sha256:589b048f72ac7201792ffb5207218d014f2b19524ec0a1426ccb723f55cb8f90` (`36812 bytes`)
- Review answer: `sha256:1547bbd981aa063b044783bccefc32a001c022593f150375afca7f923fb2e4a2` (`1109 bytes`)
- Imported validation log: `sha256:4053b30fff99121ca9d9e23d637e9eea2c57e35ebb467a1e725d3012a39ef39b`
- Root exhaustive coverage audit: `sha256:8100ecf7eb8f18768be2d50b1fe36a023ac8404522484cd96e3b3d1133793ab6`
- Exact embedded source body (Cancel outcome categories): `1179 bytes`, `sha256:e8f373c9e7cbf16d286ff8f9a9146306644e430e3d630b7078ec7501b72ed4b3`
- Exact embedded source body (Final-receipt immutable `PolicySnapshotV3` acceptance rules): `1675 bytes`, `sha256:672c9ba3db1e792a7afb145a4081450deafe6a1123262ae7c104c6f1b4b0b24a`
- Exact embedded source body (Dispatch narrowing monotonicity rules): `1930 bytes`, `sha256:b12ef2b8c8bcffad5139e196df49ba2f59094ddf024f59d2d2a645eb9be2210a`
- Exact embedded source body (Contract promotion gates): `1106 bytes`, `sha256:0fa77777b7f8ae85bc4e06082454b243677d89d070ff4c510bd9e148b61b2d43`

## Exact source-body proof

- Baseline `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` lines `119–143` are exactly `1179` bytes with SHA-256 `e8f373c9e7cbf16d286ff8f9a9146306644e430e3d630b7078ec7501b72ed4b3`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` in `llm-last-mile/runtime-refactor/contracts/cancel-outcome-categories.md` are also exactly `1179` bytes with SHA-256 `e8f373c9e7cbf16d286ff8f9a9146306644e430e3d630b7078ec7501b72ed4b3`.
- Baseline `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` lines `152–180` are exactly `1675` bytes with SHA-256 `672c9ba3db1e792a7afb145a4081450deafe6a1123262ae7c104c6f1b4b0b24a`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` in `llm-last-mile/runtime-refactor/gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md` are also exactly `1675` bytes with SHA-256 `672c9ba3db1e792a7afb145a4081450deafe6a1123262ae7c104c6f1b4b0b24a`.
- Baseline `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` lines `181–214` are exactly `1930` bytes with SHA-256 `b12ef2b8c8bcffad5139e196df49ba2f59094ddf024f59d2d2a645eb9be2210a`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` in `llm-last-mile/runtime-refactor/gates/dispatch-narrowing-monotonicity.md` are also exactly `1930` bytes with SHA-256 `b12ef2b8c8bcffad5139e196df49ba2f59094ddf024f59d2d2a645eb9be2210a`.
- Baseline `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` lines `215–231` are exactly `1106` bytes with SHA-256 `0fa77777b7f8ae85bc4e06082454b243677d89d070ff4c510bd9e148b61b2d43`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` in `llm-last-mile/runtime-refactor/gates/contract-promotion.md` are also exactly `1106` bytes with SHA-256 `0fa77777b7f8ae85bc4e06082454b243677d89d070ff4c510bd9e148b61b2d43`.
- The four baseline source spans and the four owner-body byte ranges are byte-identical, including the final blank separator before each end marker.
- The four ledger entries preserve one atomic rollback unit that restores all four root spans, removes all four canonical owner files, removes all four index rows, and removes all four D10 ledger rows together.

## Exhaustive D10 coverage audit

- Imported audit source: `/private/tmp/d10-exhaustive-coverage-audit.txt` (`sha256:8100ecf7eb8f18768be2d50b1fe36a023ac8404522484cd96e3b3d1133793ab6`).
- Mechanical coverage facts recorded by the audit: `69` root canonical-pointer lines, `55` root compatibility-anchor-only lines, `0` residual nonheading/nonpointer substantive root lines after the final candidate, `0` residual fenced-code/table/list syntax lines after the final candidate, and `12` D10 extraction-ledger rows after this candidate.
- Audit classification: categories `1–5` are already satisfied or compatibility-only by D3/D5/D6 owners; category `6` transition intent is already satisfied and `HostExecutionEpisodeV1` is extracted by D10; category `7` carrier is already satisfied and deferred retained-spawn admission recovery is extracted by D10; category `8` acceptance/receipt foundations are already satisfied and the D10 active receipt owners account for the remaining substantive receipt spans; category `9` manifest/policy owners plus the final `PolicySnapshotV3` acceptance and dispatch-narrowing gates are now accounted for; category `10` execution-envelope and secret-handoff owners are already extracted by D10; category `11` cancellation is now owned here while D6 supervision/reconciliation remain already satisfied; category `12` differential baseline remains already satisfied while these dispatch-narrowing monotonicity and contract-promotion gates close the remaining substantive spans; category `13` packet-local gates are already satisfied or compatibility-only by D5/D6 owners; and category `14` authority-required gates are already owned under `gates/`.
- The audit confirms root legacy headings remain, the candidate validation proves unique required final four root/owner anchors, supervisor `11`, differential `11A`, and all line-232-onward packet compatibility material remain byte-stable, existing D3/D5–D10 owners and review artifacts are unchanged outside the exact candidate fence, and no genuinely blocked roadmap target remains.
- Completion conclusion from the audit: if and only if this final four-span candidate receives a clean independent review and is atomically committed with truthful closeout, every substantive live D10 span is accounted for as extracted, already satisfied, or compatibility-only. This closeout records that clean independent review. Documentation extraction still grants no D11, D12, implementation, promotion, evidence, or successor authority.

## Validation evidence

- The supplied review answer states `No qualifying findings exist.` and ends with `VERDICT: APPROVED`.
- The imported validation log confirms the exact seven-path candidate fence, unique final four root/owner anchors, exact marker-body equality for all four owners, exactly one index row and one ledger row per final owner, line-232-onward packet compatibility byte stability, zero Markdown link/anchor failures, and clean candidate-patch forward/reverse application.
- The candidate patch changes exactly the seven imported candidate paths and no others.
- Candidate file hashes, candidate-patch SHA-256, prompt SHA-256, answer SHA-256, validation-log SHA-256, exhaustive-audit SHA-256, and all four exact source-body SHA-256 digests were all reverified during this closeout.
- The rendered prompt and rendered answer below are whitespace-clean: every trailing literal space or tab from the imported UTF-8 prompt or answer is rendered as `&#32;` or `&#09;` so this repository-local Markdown artifact stays clean under `git diff --check` while the base64 blocks preserve the exact bytes.
- The rendered prompt and rendered answer were both revalidated against the imported files, and the embedded base64 blocks decode exactly back to the imported UTF-8 bytes without silently correcting any captured whitespace, diff fences, or wording.
- The final changed-path fence for candidate plus closeout is exactly nine paths: the seven imported candidate paths, `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`, and this review record.
- Prior D3/D5–D9 owners and review artifacts remain unchanged; the only `docs/guidance/` changes are the tracker and this new review record.
- Complete combined candidate+closeout patch was generated at `/private/tmp/d10-cancellation-final-gates-family-closeout.patch` with a SHA-256 sidecar, and forward/reverse apply against disposable clean baseline exports both passed.
- No commit, stage, or push is performed during this closeout.

## Atomic rollback instructions

1. If the prospective nine-path batch has been committed, reverse that single batch commit atomically (for example, `git revert --no-commit <batch-commit>`) and verify that only the nine paths above are touched.
2. If the batch is still uncommitted, restore the tracked paths from the parent baseline and remove the five untracked files: `git restore --source=HEAD -- llm-last-mile/runtime-refactor/04-contracts-and-gates.md llm-last-mile/runtime-refactor/index/README.md llm-last-mile/runtime-refactor/migration/extraction-ledger.md docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md` then `rm -f llm-last-mile/runtime-refactor/contracts/cancel-outcome-categories.md llm-last-mile/runtime-refactor/gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md llm-last-mile/runtime-refactor/gates/dispatch-narrowing-monotonicity.md llm-last-mile/runtime-refactor/gates/contract-promotion.md docs/guidance/2026-08-26-runtime-refactor-d10-cancellation-final-gates-family-chatgpt-pro-review.md`.
3. After either rollback path, confirm together that all four `04-contracts-and-gates.md` bodies are restored, all four canonical owner files are removed, all four `index/README.md` rows are removed, all four `migration/extraction-ledger.md` rows are removed, the tracker summary/queue/closeout-row additions are reverted, and the review record is removed.
4. Never strand one family member: all four owners, all four root compatibility pointers, all four index rows, all four ledger rows, the tracker update, and this review record roll back together or not at all.
5. Documentation rollback must not be rewritten as runtime implementation, evidence, gate satisfaction, promotion, or dispatch authority; D11 remains blocked and separately unauthorized, and D12 remains blocked and separately unauthorized.

## Final verdict

`Cancel outcome categories` + `Final-receipt immutable PolicySnapshotV3 acceptance rules` + `Dispatch narrowing monotonicity rules` + `Contract promotion gates` are complete locally and **APPROVED** after initial-range review with zero remediation and are landed pending one atomic commit. The root exhaustive audit proves all roadmap categories `1–14` are accounted for, with `69` root canonical-pointer lines, `55` compatibility-anchor-only lines, zero residual nonheading/nonpointer substantive root lines, zero residual fenced code/table/list syntax, `12` D10 extraction-ledger rows after this candidate, and no genuinely blocked roadmap target. Therefore D10 becomes fully complete when this candidate+closeout batch is atomically committed. This documentation extraction is not runtime implementation, evidence, gate satisfaction, promotion, or dispatch authority. D11 remains BLOCKED and separately unauthorized even after D10 completion. D12 remains blocked and separately unauthorized. No successor dispatch authority is granted.

## Preserved review prompt

<details>
<summary>Initial-range review prompt (rendered copy)</summary>

``````text
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: exact baseline commit 2daccda74991c8b74530beaeb0b188fe7663d118, tree e0338ef0cd7367e483dab681bc1ed6b159f20f48, plus the complete content-addressed candidate patch below. Candidate patch SHA-256: bc3cf88f5f6cbc1f5531618aa3be1ddbbfe5c5e41326a8e9f092298b2725e801.
Review target: complete bounded final D10 candidate batch containing one cancellation contract and three final gates. Review every hunk and all four new owner files; do not infer unprovided source.

Manifest (exactly seven changed paths, including four untracked files):
- llm-last-mile/runtime-refactor/04-contracts-and-gates.md — SHA-256 2ff334d374a317cb5bf967225eaf17b832a8109e4e83e42415b28b6e80155165
- llm-last-mile/runtime-refactor/contracts/cancel-outcome-categories.md — new — SHA-256 436abc2077e0b0e51935c8b828bf552b27c5a26ff79420cda888d76f5597955a
- llm-last-mile/runtime-refactor/gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md — new — SHA-256 bb9d51065d08844fe38761f245f22abad6c6cee5cd34b4280b344d4ec87a30f6
- llm-last-mile/runtime-refactor/gates/dispatch-narrowing-monotonicity.md — new — SHA-256 dc68bbf5c70e763f5f53c9cd122af5255135c02f5af0aa860e993efe9cd82370
- llm-last-mile/runtime-refactor/gates/contract-promotion.md — new — SHA-256 378de9e21eb2c0162d74dfef9da2b3a80d9c2af5478d80957e9a0df38d4b87f7
- llm-last-mile/runtime-refactor/index/README.md — SHA-256 aa60f0f606fa6b333c879598c22b1cf6475baacb353d8353e6ed9d9870c22c28
- llm-last-mile/runtime-refactor/migration/extraction-ledger.md — SHA-256 2c1115be700fb0a7d221e152574c5f9b139624eb79b6ff7ed906e270d39bdcf8

Supported inputs and behavior: GitHub-flavored Markdown with ATX headings/generated anchors, relative links/fragments, fenced Rust/text blocks, ordered lists, Markdown tables whose row/cell order is semantic, inline code/literals, and HTML exact-body markers. Root headings remain compatibility projections until separately authorized D12. Canonical contract/gate owners are normative.
Supported platforms and dialects: repository Markdown with GitHub-style heading slug behavior and the project link/anchor validator. No runtime, Rust, script, platform, policy engine, filesystem resolution, or credential behavior is implemented by this patch.

In-scope invariants:
1. Exactly one owner each: Cancel outcome categories is a contract owner; final-receipt acceptance, dispatch narrowing monotonicity, and contract promotion are gate owners. All four land/rollback atomically.
2. Body exactness: Cancel outcome lines 119–143 are 1179 bytes SHA e8f373c9e7cbf16d286ff8f9a9146306644e430e3d630b7078ec7501b72ed4b3; final-receipt lines 152–180 are 1675 bytes SHA 672c9ba3db1e792a7afb145a4081450deafe6a1123262ae7c104c6f1b4b0b24a; narrowing lines 181–214 are 1930 bytes SHA b12ef2b8c8bcffad5139e196df49ba2f59094ddf024f59d2d2a645eb9be2210a; promotion lines 215–231 are 1106 bytes SHA 0fa77777b7f8ae85bc4e06082454b243677d89d070ff4c510bd9e148b61b2d43.
3. Cancel owner preserves CancelWorldWorkOutcomeV1 name/suffix, exact variant/field order, rules 1–6, pending-closeout nonterminal meaning, no-active versus owner-unreachable distinction, AlreadyTerminal monotonicity, and stop/cancel non-alias.
4. Final-receipt gate preserves B1 pre-E2 not-final/not-model-facing boundary, E2/B2.2/B2.1 ownership/ordering, all 11 acceptance conditions, and all post-acceptance immutability/future-only/revocation/fail-closed bullets.
5. Narrowing gate preserves resolver formula order, table row/cell order and true/false transitions, allow/deny semantics, path examples, rules 1–8, absolute/escape/glob/NUL/symlink/root exclusions, no-file-existence comparison, enforcement-time recheck, host_visible fail-closed exception, allow_capability_narrowing rejected-not-ignored rule, no forbidden capability enabling, and broker/world-service enforcement boundary.
6. Promotion gate preserves `not considered landed until tests prove` and ordered tests 1–10, including CLI/REPL/crash/no-reapply, smoke/e2e join, credential/gateway proof, compatibility evidence non-promotion, both accepted-work families, durable supervisor, foreground-drop truth, and B3.1-after-joint-closeout ordering.
7. Exact root legacy anchors remain unique/resolving: `10-cancel-outcome-categories`, `12-final-receipt-immutable-policysnapshotv3-acceptance-rules`, `13-dispatch-narrowing-monotonicity-rules`, `14-contract-promotion-gates`.
8. Supervisor idempotency 11 and differential-baseline 11A remain unchanged prior-owner projections. Line 232 `Remaining R2-2 same-process carrier closure` and all later packet-local/authority-required compatibility material remain byte-stable and are not re-extracted.
9. Exactly one correct index row and one truthful D10 ledger row per owner. Every row includes exact provenance and a complete four-owner atomic rollback restoring all four root bodies/removing four owners/four index rows/four ledger rows together.
10. Prior D3/D5-D10 owners and review artifacts are stable. Exact seven-path fence only; no formatting normalization or unrelated edit.
11. Candidate does not claim implementation, evidence/gate satisfaction, promotion, successor authority, D10 completion, D11 authority/decomposition, or D12 cutover.
12. Root-owned exhaustive classification evidence is conditional, not a patch claim: after this candidate, root has 69 canonical pointer lines, 55 compatibility-anchor-only lines, zero residual nonheading/nonpointer substantive lines, zero residual code/table/list syntax, 12 D10 ledger rows, and no blocked roadmap target; audit SHA-256 8100ecf7eb8f18768be2d50b1fe36a023ac8404522484cd96e3b3d1133793ab6. Only after clean review and atomic closeout may D10 be declared complete; D11 remains separately unauthorized.

Required material consequence: block only a reachable, patch-causal defect that materially loses/alters/reorders contract or gate semantics; weakens negative/fail-closed/immutability/non-promotion requirements; misidentifies ownership/status; breaks required link/anchor/provenance/atomic rollback; re-extracts prior-owned line-232+ material; breaches the seven-path fence; or prevents clean application/reversal. Style preferences, optional wording, broader redesign, unsupported Markdown behavior, and unchanged baseline defects are nonmaterial.
Blocking threshold: only reachable P1/P2 findings violating an invariant with material consequence. Body/hash mismatch, enum/field/rule/table/order loss, false gate satisfaction/landing, weakened rejection/fail-closed constraint, wrong owner, broken required anchor/link, incomplete four-owner rollback, false authority/status, or scope breach qualifies. Label other observations deferred and do not change verdict.
Accepted prior findings: none.
Deferred/out of scope: runtime implementation or test adequacy beyond preserved gate text; D3 review governance; prior D5/D6/D9 owners; D11 evidence; D12 cutover; Rust/scripts/platform behavior; roadmap/ZIP; review-control/slices/tasks; global formatting; unrelated pre-existing issues; implementation/successor dispatch; independent re-audit of unchanged repository sources not supplied here.
Project sources: only this prompt, manifest, invariants, validation/audit facts, and complete patch. Excluded source is unavailable and must not be inferred.

Validation evidence:
- exact seven-path fence/no staged files: PASS
- `git diff --check`: PASS
- no-index checks all four owners: ordinary difference exit, zero diagnostics
- Markdown link/anchor validation: 1,889 checked, 0 failures
- four marker bodies exact bytes/hashes; root pointers/owner anchors unique; exactly one index/ledger row each: PASS
- enum/field/rule/table/formula/literal/negative/exception/authority/order preservation: PASS
- sections 11/11A and line-232 onward byte-stable; prior owners/reviews stable: PASS
- patch forward exact seven-file apply and reverse clean restoration: PASS
- candidate validation log SHA-256 4053b30fff99121ca9d9e23d637e9eea2c57e35ebb467a1e725d3012a39ef39b

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return findings first by severity. Each blocker must name exact file/hunk, supported-input reachability, violated invariant, material consequence, and patch causality. State explicitly when no qualifying findings exist. End exactly:
VERDICT: APPROVED
or
VERDICT: CHANGES REQUIRED

Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

COMPLETE CONTENT-ADDRESSED PATCH (SHA-256 bc3cf88f5f6cbc1f5531618aa3be1ddbbfe5c5e41326a8e9f092298b2725e801):
```diff
diff --git a/llm-last-mile/runtime-refactor/04-contracts-and-gates.md b/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
index 16cab4f1a66895b7610b4e5ee759f4040165d1fe..7410f0b2ca6e7470c3d7bf1ddf9c9564cc68c205 100644
--- a/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
+++ b/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
@@ -121,22 +121 @@ Canonical content: [`contracts/launch-time-secret-handoff-v1.md#9-launchtimesecr
-```rust
-enum CancelWorldWorkOutcomeV1 {
-    CancelledViaLiveTransport { active_run_id, terminal_ref },
-    CancelAcceptedPendingCloseout { active_run_id, cancel_request_id },
-    AlreadyTerminal { active_run_id, terminal_ref },
-    NoActiveCancelableWork { worker_or_task_ref },
-    OwnerUnreachable { active_run_id, durable_state, retryability },
-    InvalidTarget { reason },
-    WorldBindingMismatch { expected, actual },
-    AmbiguousTarget { candidates },
-    PolicyDenied { denial_code, explanation },
-}
-```
-
-Rules:
-
-1. Exact identity and world binding resolve before transport use.
-2. `NoActiveCancelableWork` means valid routing context but no accepted non-terminal cancelable receipt.
-3. `OwnerUnreachable` means a valid active receipt exists but the live cancellation route is unavailable and durable policy cannot yet declare closeout.
-4. `CancelAcceptedPendingCloseout` is not terminal success; inspect remains able to observe eventual closeout.
-5. Repeated cancel after terminal returns `AlreadyTerminal`; it never regresses the receipt.
-6. Stop targets worker lifecycle. Cancel targets one active task/turn. They are not aliases.
+Canonical content: [`contracts/cancel-outcome-categories.md#10-cancel-outcome-categories`](contracts/cancel-outcome-categories.md#10-cancel-outcome-categories).
@@ -154,26 +133 @@ Canonical content: [`b1-b2-1/contracts-and-gates.md#11a-differential-baseline-tr
-B1's pre-E2 acceptance anchor records the exact current policy identity used by the runtime but is
-not a final receipt and is not model-facing. E2 owns the immutable active-run snapshot and retained
-worker cap below; B2.2 may expose a receipt only after those commitments and the B2.1 observation
-claim are durable and linked to the B1 record.
-
-A final active task/turn receipt may be exposed only when all are true:
-
-1. exact session, caller, backend, and world binding are resolved;
-2. steering policy allows the verb/mode/target;
-3. current parent policy is resolved at a known revision;
-4. retained worker cap is loaded and hash-verified when applicable;
-5. optional narrowing is validated as monotonic;
-6. the resulting `PolicySnapshotV3` canonicalizes and passes existing schema/enforcement validation;
-7. snapshot bytes/ref/hash/revision are durable;
-8. the execution envelope/world-service request carries the same verified snapshot;
-9. runtime acceptance evidence joins the acknowledgement to the exact work identity;
-10. the observation claim is durable and resumable; and
-11. the receipt references the snapshot, acceptance evidence, and observation claim before the foreground caller is told the work was accepted.
-
-After acceptance:
-
-- the receipt's policy ref/hash is immutable;
-- parent broadening or narrowing does not rewrite the active receipt;
-- parent changes apply to future task acceptance, continue, fork, or worker turns;
-- emergency revocation is an explicit audited cancel/revoke path, never silent snapshot mutation; and
-- snapshot mismatch at broker/world-service fails closed.
+Canonical content: [`gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules`](gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules).
@@ -183,31 +137 @@ After acceptance:
-The resolver computes:
-
-```text
-ephemeral = current_parent AND dispatch_patch
-worker_cap = current_parent_at_spawn AND spawn_patch
-turn = current_parent_now AND worker_cap AND turn_patch
-fork_cap = current_parent_now AND source_worker_cap AND fork_patch
-```
-
-V1 field rules:
-
-| Field | Allowed narrowing | Rejected broadening |
-|---|---|---|
-| `world_fs.host_visible` | `true -> false` | `false -> true` |
-| `world_fs.fail_closed.routing` | `false -> true` | `true -> false` |
-| `world_fs.caged_required` | `false -> true` | `true -> false` |
-| `world_fs.write.enabled` | `true -> false` | `false -> true` |
-| `world_fs.deny_enforcement` | same or stronger rank | weaker rank or removal |
-| discover/read/write `allow_list` | each requested path is contained by at least one parent path | root/directory/file widening |
-| discover/read/write `deny_list` | add denials or retain parent denials | remove parent denial |
-
-Path-containment rules:
-
-1. Normalize relative to the authoritative world/project root.
-2. `.` contains `src` and `src/parser.rs`; `src` contains `src/parser.rs`; a file contains only itself.
-3. Reject absolute host paths, `..` escape, unsupported glob semantics, NUL, symlink escape, and paths outside the root.
-4. Compare canonical policy paths without requiring the target file to already exist; runtime resolution must re-check symlink/ancestor escape at enforcement time.
-5. `host_visible=true` with deny-list or unprovable isolation semantics fails closed unless the patch legally narrows to supported full isolation.
-6. A patch supplied while `agents.world_dispatch.allow_capability_narrowing=false` is rejected, not ignored.
-7. Narrowing may not enable a dispatch action, backend, mode, capability, network route, or side-effect channel forbidden by the parent.
-8. Adapter config may receive policy hints, but only broker/world-service enforcement counts.
+Canonical content: [`gates/dispatch-narrowing-monotonicity.md#13-dispatch-narrowing-monotonicity-rules`](gates/dispatch-narrowing-monotonicity.md#13-dispatch-narrowing-monotonicity-rules).
@@ -217,14 +141 @@ Path-containment rules:
-A contract is not considered landed until tests prove:
-
-1. serialization and validation;
-2. atomic persistence and revision conflict handling;
-3. the real ingress/dispatch/runtime path uses it;
-4. restart/replay behavior where durable;
-5. fail-closed negative cases;
-6. every revision-bound host transition joins intent issuance, claim, authority application, and exact result on the real CLI and REPL path, including crash reconciliation and no-reapply exact retry;
-7. at least one smoke/e2e path joins session, binding, policy, receipt, runtime event, and terminal/obligation truth;
-8. credential-requiring world UAA proof joins the envelope to a consumed one-time in-world gateway handoff without copied secret files or inherited descriptors; and
-9. no compatibility copy or `CompatibilityUnproven` evidence is used for contract promotion; and
-10. B1/B2.1 production proof shows both accepted work families enter the durable supervisor without
-    a legacy-writer attempt, caller/foreground drop does not erase truth, and B3.1 begins only after
-    the joint closeout.
+Canonical content: [`gates/contract-promotion.md#14-contract-promotion-gates`](gates/contract-promotion.md#14-contract-promotion-gates).
diff --git a/llm-last-mile/runtime-refactor/index/README.md b/llm-last-mile/runtime-refactor/index/README.md
index a558fc56c573ceeb90dd4d4c873211fcdfe23ef1..89f330dd5874fea64bae99fdd563f41e3876a74a 100644
--- a/llm-last-mile/runtime-refactor/index/README.md
+++ b/llm-last-mile/runtime-refactor/index/README.md
@@ -27,0 +28,4 @@
+| Cancel outcome categories | contract | [`contracts/cancel-outcome-categories.md`](../contracts/cancel-outcome-categories.md) | `canonical extracted contract owner for the complete CancelWorldWorkOutcomeV1 enum and rules 1–6 covering identity-before-transport resolution, pending closeout, no-active versus owner-unreachable, terminal monotonicity, and stop-versus-cancel non-aliasing` | Supersedes only root-canonical ownership of the extracted `Cancel outcome categories` span; `LaunchTimeSecretHandoffV1`, `Supervisor idempotency and restart rules`, `Differential-baseline transition gate`, and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#10-cancel-outcome-categories) |
+| Final-receipt immutable `PolicySnapshotV3` acceptance rules | gate | [`gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`](../gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md) | `canonical extracted gate owner for the B1 pre-E2 acceptance-anchor boundary, ordered conditions 1–11, immutable receipt-exposure rule, and post-acceptance future-only, revocation, and fail-closed constraints` | Supersedes only root-canonical ownership of the extracted `Final-receipt immutable PolicySnapshotV3 acceptance rules` span; `Supervisor idempotency and restart rules`, `Differential-baseline transition gate`, `Dispatch narrowing monotonicity rules`, and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules) |
+| Dispatch narrowing monotonicity rules | gate | [`gates/dispatch-narrowing-monotonicity.md`](../gates/dispatch-narrowing-monotonicity.md) | `canonical extracted gate owner for the resolver formulas, V1 field table, path-containment rules 1–8, host_visible fail-closed exception, rejected-not-ignored narrowing, and broker/world-service enforcement boundary` | Supersedes only root-canonical ownership of the extracted `Dispatch narrowing monotonicity rules` span; `Final-receipt immutable PolicySnapshotV3 acceptance rules`, `Contract promotion gates`, and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules) |
+| Contract promotion gates | gate | [`gates/contract-promotion.md`](../gates/contract-promotion.md) | `canonical extracted gate owner for the tests-prove-before-landed promotion rule and ordered proof gates 1–10 covering CLI/REPL, restart, smoke, credential handoff, compatibility exclusion, and B1/B2.1-to-B3.1 sequencing` | Supersedes only root-canonical ownership of the extracted `Contract promotion gates` span; `Dispatch narrowing monotonicity rules`, `Remaining R2-2 same-process carrier closure`, and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#14-contract-promotion-gates) |
diff --git a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
index 5a7daaf2806de810957bb1c55aeff88473d71aaf..8190bad96188703b8c04d383f753f38054a68685 100644
--- a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
+++ b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
@@ -194,0 +195,4 @@ This ledger records content-preserving authority transfers while retaining requi
+| D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 10. Cancel outcome categories | [`10-cancel-outcome-categories`](../04-contracts-and-gates.md#10-cancel-outcome-categories) | lines 119–143 (1179 bytes) | `e8f373c9e7cbf16d286ff8f9a9146306644e430e3d630b7078ec7501b72ed4b3` | [`../contracts/cancel-outcome-categories.md`](../contracts/cancel-outcome-categories.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1179-byte source span at `04-contracts-and-gates.md#10-cancel-outcome-categories`, the exact 1675-byte source span at `04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules`, the exact 1930-byte source span at `04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules`, and the exact 1106-byte source span at `04-contracts-and-gates.md#14-contract-promotion-gates`; remove `contracts/cancel-outcome-categories.md`, `gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`, `gates/dispatch-narrowing-monotonicity.md`, and `gates/contract-promotion.md`; remove the exact `Cancel outcome categories`, `Final-receipt immutable PolicySnapshotV3 acceptance rules`, `Dispatch narrowing monotonicity rules`, and `Contract promotion gates` rows from `index/README.md`; remove all four D10 ledger entries for this batch; leave `LaunchTimeSecretHandoffV1`, `Supervisor idempotency and restart rules`, `Differential-baseline transition gate`, `Remaining R2-2 same-process carrier closure`, all D3/D5–D10 owners, `index/current.md`, `review-control/`, and every path outside the seven-path fence unchanged |
+| D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 12. Final-receipt immutable `PolicySnapshotV3` acceptance rules | [`12-final-receipt-immutable-policysnapshotv3-acceptance-rules`](../04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules) | lines 152–180 (1675 bytes) | `672c9ba3db1e792a7afb145a4081450deafe6a1123262ae7c104c6f1b4b0b24a` | [`../gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`](../gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md) | gate | canonical destination; source compatibility anchor | none | restore the exact 1179-byte source span at `04-contracts-and-gates.md#10-cancel-outcome-categories`, the exact 1675-byte source span at `04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules`, the exact 1930-byte source span at `04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules`, and the exact 1106-byte source span at `04-contracts-and-gates.md#14-contract-promotion-gates`; remove `contracts/cancel-outcome-categories.md`, `gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`, `gates/dispatch-narrowing-monotonicity.md`, and `gates/contract-promotion.md`; remove the exact `Cancel outcome categories`, `Final-receipt immutable PolicySnapshotV3 acceptance rules`, `Dispatch narrowing monotonicity rules`, and `Contract promotion gates` rows from `index/README.md`; remove all four D10 ledger entries for this batch; leave `LaunchTimeSecretHandoffV1`, `Supervisor idempotency and restart rules`, `Differential-baseline transition gate`, `Remaining R2-2 same-process carrier closure`, all D3/D5–D10 owners, `index/current.md`, `review-control/`, and every path outside the seven-path fence unchanged |
+| D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 13. Dispatch narrowing monotonicity rules | [`13-dispatch-narrowing-monotonicity-rules`](../04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules) | lines 181–214 (1930 bytes) | `b12ef2b8c8bcffad5139e196df49ba2f59094ddf024f59d2d2a645eb9be2210a` | [`../gates/dispatch-narrowing-monotonicity.md`](../gates/dispatch-narrowing-monotonicity.md) | gate | canonical destination; source compatibility anchor | none | restore the exact 1179-byte source span at `04-contracts-and-gates.md#10-cancel-outcome-categories`, the exact 1675-byte source span at `04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules`, the exact 1930-byte source span at `04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules`, and the exact 1106-byte source span at `04-contracts-and-gates.md#14-contract-promotion-gates`; remove `contracts/cancel-outcome-categories.md`, `gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`, `gates/dispatch-narrowing-monotonicity.md`, and `gates/contract-promotion.md`; remove the exact `Cancel outcome categories`, `Final-receipt immutable PolicySnapshotV3 acceptance rules`, `Dispatch narrowing monotonicity rules`, and `Contract promotion gates` rows from `index/README.md`; remove all four D10 ledger entries for this batch; leave `LaunchTimeSecretHandoffV1`, `Supervisor idempotency and restart rules`, `Differential-baseline transition gate`, `Remaining R2-2 same-process carrier closure`, all D3/D5–D10 owners, `index/current.md`, `review-control/`, and every path outside the seven-path fence unchanged |
+| D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 14. Contract promotion gates | [`14-contract-promotion-gates`](../04-contracts-and-gates.md#14-contract-promotion-gates) | lines 215–231 (1106 bytes) | `0fa77777b7f8ae85bc4e06082454b243677d89d070ff4c510bd9e148b61b2d43` | [`../gates/contract-promotion.md`](../gates/contract-promotion.md) | gate | canonical destination; source compatibility anchor | none | restore the exact 1179-byte source span at `04-contracts-and-gates.md#10-cancel-outcome-categories`, the exact 1675-byte source span at `04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules`, the exact 1930-byte source span at `04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules`, and the exact 1106-byte source span at `04-contracts-and-gates.md#14-contract-promotion-gates`; remove `contracts/cancel-outcome-categories.md`, `gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`, `gates/dispatch-narrowing-monotonicity.md`, and `gates/contract-promotion.md`; remove the exact `Cancel outcome categories`, `Final-receipt immutable PolicySnapshotV3 acceptance rules`, `Dispatch narrowing monotonicity rules`, and `Contract promotion gates` rows from `index/README.md`; remove all four D10 ledger entries for this batch; leave `LaunchTimeSecretHandoffV1`, `Supervisor idempotency and restart rules`, `Differential-baseline transition gate`, `Remaining R2-2 same-process carrier closure`, all D3/D5–D10 owners, `index/current.md`, `review-control/`, and every path outside the seven-path fence unchanged |
diff --git a/llm-last-mile/runtime-refactor/contracts/cancel-outcome-categories.md b/llm-last-mile/runtime-refactor/contracts/cancel-outcome-categories.md
new file mode 100644
index 0000000000000000000000000000000000000000..cdebd197b375a6d99a89d0676db422548602dad2
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/contracts/cancel-outcome-categories.md
@@ -0,0 +1,33 @@
+**Kind:** contract
+**Status:** canonical
+**Canonical for:** complete extracted `CancelWorldWorkOutcomeV1` enum and rules 1–6 covering exact identity and world-binding resolution, `NoActiveCancelableWork` versus `OwnerUnreachable`, pending closeout, terminal monotonicity, and stop-versus-cancel non-aliasing
+**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#10-cancel-outcome-categories`](../04-contracts-and-gates.md#10-cancel-outcome-categories), baseline lines 119–143; the exact 1179-byte source body is preserved between the boundary markers below
+**Baseline span SHA-256:** `e8f373c9e7cbf16d286ff8f9a9146306644e430e3d630b7078ec7501b72ed4b3`
+
+<!-- exact-extracted-body:start -->
+## 10. Cancel outcome categories
+
+```rust
+enum CancelWorldWorkOutcomeV1 {
+    CancelledViaLiveTransport { active_run_id, terminal_ref },
+    CancelAcceptedPendingCloseout { active_run_id, cancel_request_id },
+    AlreadyTerminal { active_run_id, terminal_ref },
+    NoActiveCancelableWork { worker_or_task_ref },
+    OwnerUnreachable { active_run_id, durable_state, retryability },
+    InvalidTarget { reason },
+    WorldBindingMismatch { expected, actual },
+    AmbiguousTarget { candidates },
+    PolicyDenied { denial_code, explanation },
+}
+```
+
+Rules:
+
+1. Exact identity and world binding resolve before transport use.
+2. `NoActiveCancelableWork` means valid routing context but no accepted non-terminal cancelable receipt.
+3. `OwnerUnreachable` means a valid active receipt exists but the live cancellation route is unavailable and durable policy cannot yet declare closeout.
+4. `CancelAcceptedPendingCloseout` is not terminal success; inspect remains able to observe eventual closeout.
+5. Repeated cancel after terminal returns `AlreadyTerminal`; it never regresses the receipt.
+6. Stop targets worker lifecycle. Cancel targets one active task/turn. They are not aliases.
+
+<!-- exact-extracted-body:end -->
diff --git a/llm-last-mile/runtime-refactor/gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md b/llm-last-mile/runtime-refactor/gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md
new file mode 100644
index 0000000000000000000000000000000000000000..33404557c718c3feebef0a543abe6e4406762095
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md
@@ -0,0 +1,37 @@
+**Kind:** gate
+**Status:** canonical
+**Canonical for:** complete extracted B1 pre-E2 versus E2/B2.2 acceptance boundary, ordered conditions 1–11, immutable `PolicySnapshotV3` receipt exposure rule, and the post-acceptance immutable, future-only, revocation, and fail-closed requirements
+**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules`](../04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules), baseline lines 152–180; the exact 1675-byte source body is preserved between the boundary markers below
+**Baseline span SHA-256:** `672c9ba3db1e792a7afb145a4081450deafe6a1123262ae7c104c6f1b4b0b24a`
+
+<!-- exact-extracted-body:start -->
+## 12. Final-receipt immutable `PolicySnapshotV3` acceptance rules
+
+B1's pre-E2 acceptance anchor records the exact current policy identity used by the runtime but is
+not a final receipt and is not model-facing. E2 owns the immutable active-run snapshot and retained
+worker cap below; B2.2 may expose a receipt only after those commitments and the B2.1 observation
+claim are durable and linked to the B1 record.
+
+A final active task/turn receipt may be exposed only when all are true:
+
+1. exact session, caller, backend, and world binding are resolved;
+2. steering policy allows the verb/mode/target;
+3. current parent policy is resolved at a known revision;
+4. retained worker cap is loaded and hash-verified when applicable;
+5. optional narrowing is validated as monotonic;
+6. the resulting `PolicySnapshotV3` canonicalizes and passes existing schema/enforcement validation;
+7. snapshot bytes/ref/hash/revision are durable;
+8. the execution envelope/world-service request carries the same verified snapshot;
+9. runtime acceptance evidence joins the acknowledgement to the exact work identity;
+10. the observation claim is durable and resumable; and
+11. the receipt references the snapshot, acceptance evidence, and observation claim before the foreground caller is told the work was accepted.
+
+After acceptance:
+
+- the receipt's policy ref/hash is immutable;
+- parent broadening or narrowing does not rewrite the active receipt;
+- parent changes apply to future task acceptance, continue, fork, or worker turns;
+- emergency revocation is an explicit audited cancel/revoke path, never silent snapshot mutation; and
+- snapshot mismatch at broker/world-service fails closed.
+
+<!-- exact-extracted-body:end -->
diff --git a/llm-last-mile/runtime-refactor/gates/dispatch-narrowing-monotonicity.md b/llm-last-mile/runtime-refactor/gates/dispatch-narrowing-monotonicity.md
new file mode 100644
index 0000000000000000000000000000000000000000..fd2b74554b46afdbed559f62a32577c617b85ee8
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/gates/dispatch-narrowing-monotonicity.md
@@ -0,0 +1,42 @@
+**Kind:** gate
+**Status:** canonical
+**Canonical for:** complete extracted resolver formulas, V1 field rules table, path-containment rules 1–8, host-visible fail-closed exception, rejected-not-ignored narrowing rule, and broker versus world-service enforcement boundary
+**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules`](../04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules), baseline lines 181–214; the exact 1930-byte source body is preserved between the boundary markers below
+**Baseline span SHA-256:** `b12ef2b8c8bcffad5139e196df49ba2f59094ddf024f59d2d2a645eb9be2210a`
+
+<!-- exact-extracted-body:start -->
+## 13. Dispatch narrowing monotonicity rules
+
+The resolver computes:
+
+```text
+ephemeral = current_parent AND dispatch_patch
+worker_cap = current_parent_at_spawn AND spawn_patch
+turn = current_parent_now AND worker_cap AND turn_patch
+fork_cap = current_parent_now AND source_worker_cap AND fork_patch
+```
+
+V1 field rules:
+
+| Field | Allowed narrowing | Rejected broadening |
+|---|---|---|
+| `world_fs.host_visible` | `true -> false` | `false -> true` |
+| `world_fs.fail_closed.routing` | `false -> true` | `true -> false` |
+| `world_fs.caged_required` | `false -> true` | `true -> false` |
+| `world_fs.write.enabled` | `true -> false` | `false -> true` |
+| `world_fs.deny_enforcement` | same or stronger rank | weaker rank or removal |
+| discover/read/write `allow_list` | each requested path is contained by at least one parent path | root/directory/file widening |
+| discover/read/write `deny_list` | add denials or retain parent denials | remove parent denial |
+
+Path-containment rules:
+
+1. Normalize relative to the authoritative world/project root.
+2. `.` contains `src` and `src/parser.rs`; `src` contains `src/parser.rs`; a file contains only itself.
+3. Reject absolute host paths, `..` escape, unsupported glob semantics, NUL, symlink escape, and paths outside the root.
+4. Compare canonical policy paths without requiring the target file to already exist; runtime resolution must re-check symlink/ancestor escape at enforcement time.
+5. `host_visible=true` with deny-list or unprovable isolation semantics fails closed unless the patch legally narrows to supported full isolation.
+6. A patch supplied while `agents.world_dispatch.allow_capability_narrowing=false` is rejected, not ignored.
+7. Narrowing may not enable a dispatch action, backend, mode, capability, network route, or side-effect channel forbidden by the parent.
+8. Adapter config may receive policy hints, but only broker/world-service enforcement counts.
+
+<!-- exact-extracted-body:end -->
diff --git a/llm-last-mile/runtime-refactor/gates/contract-promotion.md b/llm-last-mile/runtime-refactor/gates/contract-promotion.md
new file mode 100644
index 0000000000000000000000000000000000000000..b7c74545b5a54bc6536f32fb921bd7a2c5dd6e25
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/gates/contract-promotion.md
@@ -0,0 +1,25 @@
+**Kind:** gate
+**Status:** canonical
+**Canonical for:** complete extracted tests-prove-before-landed promotion gate and ordered proof requirements 1–10 covering serialization, persistence, real-path usage, restart and replay, smoke and e2e joins, one-time gateway handoff proof, compatibility exclusion, and B1/B2.1-to-B3.1 sequencing
+**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#14-contract-promotion-gates`](../04-contracts-and-gates.md#14-contract-promotion-gates), baseline lines 215–231; the exact 1106-byte source body is preserved between the boundary markers below
+**Baseline span SHA-256:** `0fa77777b7f8ae85bc4e06082454b243677d89d070ff4c510bd9e148b61b2d43`
+
+<!-- exact-extracted-body:start -->
+## 14. Contract promotion gates
+
+A contract is not considered landed until tests prove:
+
+1. serialization and validation;
+2. atomic persistence and revision conflict handling;
+3. the real ingress/dispatch/runtime path uses it;
+4. restart/replay behavior where durable;
+5. fail-closed negative cases;
+6. every revision-bound host transition joins intent issuance, claim, authority application, and exact result on the real CLI and REPL path, including crash reconciliation and no-reapply exact retry;
+7. at least one smoke/e2e path joins session, binding, policy, receipt, runtime event, and terminal/obligation truth;
+8. credential-requiring world UAA proof joins the envelope to a consumed one-time in-world gateway handoff without copied secret files or inherited descriptors; and
+9. no compatibility copy or `CompatibilityUnproven` evidence is used for contract promotion; and
+10. B1/B2.1 production proof shows both accepted work families enter the durable supervisor without
+    a legacy-writer attempt, caller/foreground drop does not erase truth, and B3.1 begins only after
+    the joint closeout.
+
+<!-- exact-extracted-body:end -->
```
``````

</details>

<details>
<summary>Initial-range review prompt exact bytes (base64 UTF-8)</summary>

```text
UmV2aWV3IG1vZGU6IGluaXRpYWwtcmFuZ2UKUmV2aWV3IGNvbnRleHQ6IGluZGVwZW5kZW50IGZy
ZXNoIGNvbnZlcnNhdGlvbgpSZXZpZXcgYm91bmRhcnk6IGV4YWN0IGJhc2VsaW5lIGNvbW1pdCAy
ZGFjY2RhNzQ5OTFjOGI3NDUzMGJlYWViMGIxODhmZTc2NjNkMTE4LCB0cmVlIGUwMzM4ZWYwY2Q3
MzY3ZTQ4M2RhYjY4MWJjMWVkNmIxNTlmMjBmNDgsIHBsdXMgdGhlIGNvbXBsZXRlIGNvbnRlbnQt
YWRkcmVzc2VkIGNhbmRpZGF0ZSBwYXRjaCBiZWxvdy4gQ2FuZGlkYXRlIHBhdGNoIFNIQS0yNTY6
IGJjM2NmODhmNWY2Y2JjMWY1NTMxNjE4YWEzYmUxZGRiYmZlNWM1ZTQxMzI2YThlOWYwOTIyOThi
MjcyNWU4MDEuClJldmlldyB0YXJnZXQ6IGNvbXBsZXRlIGJvdW5kZWQgZmluYWwgRDEwIGNhbmRp
ZGF0ZSBiYXRjaCBjb250YWluaW5nIG9uZSBjYW5jZWxsYXRpb24gY29udHJhY3QgYW5kIHRocmVl
IGZpbmFsIGdhdGVzLiBSZXZpZXcgZXZlcnkgaHVuayBhbmQgYWxsIGZvdXIgbmV3IG93bmVyIGZp
bGVzOyBkbyBub3QgaW5mZXIgdW5wcm92aWRlZCBzb3VyY2UuCgpNYW5pZmVzdCAoZXhhY3RseSBz
ZXZlbiBjaGFuZ2VkIHBhdGhzLCBpbmNsdWRpbmcgZm91ciB1bnRyYWNrZWQgZmlsZXMpOgotIGxs
bS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIOKA
lCBTSEEtMjU2IDJmZjMzNGQzNzRhMzE3Y2I1YmY5NjcyMjVlYWYxN2I4MzJhODEwOWU0ZTgzZTQy
NDE1YjI4YjZlODAxNTUxNjUKLSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvY29udHJh
Y3RzL2NhbmNlbC1vdXRjb21lLWNhdGVnb3JpZXMubWQg4oCUIG5ldyDigJQgU0hBLTI1NiA0MzZh
YmMyMDc3ZTBiMGU1MTkzNWM4YjgyOGJmNTUyYjI3YzVhMjZmZjc5NDIwY2RhODg4ZDc2ZjU1OTc5
NTVhCi0gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2dhdGVzL2ZpbmFsLXJlY2VpcHQt
aW1tdXRhYmxlLXBvbGljeS1zbmFwc2hvdC12My1hY2NlcHRhbmNlLm1kIOKAlCBuZXcg4oCUIFNI
QS0yNTYgYmI5ZDUxMDY1ZDA4ODQ0ZmUzODc2MWYyNDVmMjJhYmFkNmM2Y2VlNWNkMzRiNDI4MGIz
NDRkNGVjODdhMzBmNgotIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9nYXRlcy9kaXNw
YXRjaC1uYXJyb3dpbmctbW9ub3RvbmljaXR5Lm1kIOKAlCBuZXcg4oCUIFNIQS0yNTYgZGM2OGJi
ZjVjNzBlNzYzZjVmNTNjOWNkMTIyYWY1MjU1MTM1YzAyZjVhZjBhYTg2MGU5OTNlZmU5Y2Q4MjM3
MAotIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9nYXRlcy9jb250cmFjdC1wcm9tb3Rp
b24ubWQg4oCUIG5ldyDigJQgU0hBLTI1NiAzNzhkZTllMjFlYjJjMDE2MmQ3NGRmZWY5ZGEyYjNh
ODBkOWMyYWY1NDc4ZDgwOTU3ZTlhMGRmMzhkNGI4N2Y3Ci0gbGxtLWxhc3QtbWlsZS9ydW50aW1l
LXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZCDigJQgU0hBLTI1NiBhYTYwZjBmNjA2ZmE2YjMzM2M4
Nzk1OThjMjJiMWNmNjQ3NWJhYWNiMzUzZDgzNTNlNmVkOWQ5ODcwYzIyYzI4Ci0gbGxtLWxhc3Qt
bWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZCDigJQg
U0hBLTI1NiAyYzExMTViZTcwMGZiMGE3ZDIyMWUxNTI1NzRjNWY5YjEzOTYyNGViNzliNmZmN2Vk
OTA2ZTI3MGQzOWJkY2Y4CgpTdXBwb3J0ZWQgaW5wdXRzIGFuZCBiZWhhdmlvcjogR2l0SHViLWZs
YXZvcmVkIE1hcmtkb3duIHdpdGggQVRYIGhlYWRpbmdzL2dlbmVyYXRlZCBhbmNob3JzLCByZWxh
dGl2ZSBsaW5rcy9mcmFnbWVudHMsIGZlbmNlZCBSdXN0L3RleHQgYmxvY2tzLCBvcmRlcmVkIGxp
c3RzLCBNYXJrZG93biB0YWJsZXMgd2hvc2Ugcm93L2NlbGwgb3JkZXIgaXMgc2VtYW50aWMsIGlu
bGluZSBjb2RlL2xpdGVyYWxzLCBhbmQgSFRNTCBleGFjdC1ib2R5IG1hcmtlcnMuIFJvb3QgaGVh
ZGluZ3MgcmVtYWluIGNvbXBhdGliaWxpdHkgcHJvamVjdGlvbnMgdW50aWwgc2VwYXJhdGVseSBh
dXRob3JpemVkIEQxMi4gQ2Fub25pY2FsIGNvbnRyYWN0L2dhdGUgb3duZXJzIGFyZSBub3JtYXRp
dmUuClN1cHBvcnRlZCBwbGF0Zm9ybXMgYW5kIGRpYWxlY3RzOiByZXBvc2l0b3J5IE1hcmtkb3du
IHdpdGggR2l0SHViLXN0eWxlIGhlYWRpbmcgc2x1ZyBiZWhhdmlvciBhbmQgdGhlIHByb2plY3Qg
bGluay9hbmNob3IgdmFsaWRhdG9yLiBObyBydW50aW1lLCBSdXN0LCBzY3JpcHQsIHBsYXRmb3Jt
LCBwb2xpY3kgZW5naW5lLCBmaWxlc3lzdGVtIHJlc29sdXRpb24sIG9yIGNyZWRlbnRpYWwgYmVo
YXZpb3IgaXMgaW1wbGVtZW50ZWQgYnkgdGhpcyBwYXRjaC4KCkluLXNjb3BlIGludmFyaWFudHM6
CjEuIEV4YWN0bHkgb25lIG93bmVyIGVhY2g6IENhbmNlbCBvdXRjb21lIGNhdGVnb3JpZXMgaXMg
YSBjb250cmFjdCBvd25lcjsgZmluYWwtcmVjZWlwdCBhY2NlcHRhbmNlLCBkaXNwYXRjaCBuYXJy
b3dpbmcgbW9ub3RvbmljaXR5LCBhbmQgY29udHJhY3QgcHJvbW90aW9uIGFyZSBnYXRlIG93bmVy
cy4gQWxsIGZvdXIgbGFuZC9yb2xsYmFjayBhdG9taWNhbGx5LgoyLiBCb2R5IGV4YWN0bmVzczog
Q2FuY2VsIG91dGNvbWUgbGluZXMgMTE54oCTMTQzIGFyZSAxMTc5IGJ5dGVzIFNIQSBlOGYzNzNj
OWU3Y2JmMTZkMjg2ZmY4ZjlhOTE0NjMwNjY0NGU0MzBlM2Q2MzBiNzA3OGVjNzUwMWI3MmVkNGIz
OyBmaW5hbC1yZWNlaXB0IGxpbmVzIDE1MuKAkzE4MCBhcmUgMTY3NSBieXRlcyBTSEEgNjcyYzli
YTNkYjFlNzkyYTdhZmIxNDVhNDA4MTQ1MGRlYWZlNmExMTIzMjYyYWU3YzEwNGM2ZjFiNGIwYjI0
YTsgbmFycm93aW5nIGxpbmVzIDE4MeKAkzIxNCBhcmUgMTkzMCBieXRlcyBTSEEgYjEyZWYyYjhj
OGJjZmZhZDUxMzllMTk2ZGY0OWJhMmY1OTA5NGRkZjAyNGY1OWQyZDJhNjQ1ZWI5YmUyMjEwYTsg
cHJvbW90aW9uIGxpbmVzIDIxNeKAkzIzMSBhcmUgMTEwNiBieXRlcyBTSEEgMGZhNzc3NzdiN2Y4
YWU4NWJjNGUwNjA4MjQ1NGIyNDM2NzdkODlkMDcwZmY0YzUxMGJkOWUxNDhiNjFiMmQ0My4KMy4g
Q2FuY2VsIG93bmVyIHByZXNlcnZlcyBDYW5jZWxXb3JsZFdvcmtPdXRjb21lVjEgbmFtZS9zdWZm
aXgsIGV4YWN0IHZhcmlhbnQvZmllbGQgb3JkZXIsIHJ1bGVzIDHigJM2LCBwZW5kaW5nLWNsb3Nl
b3V0IG5vbnRlcm1pbmFsIG1lYW5pbmcsIG5vLWFjdGl2ZSB2ZXJzdXMgb3duZXItdW5yZWFjaGFi
bGUgZGlzdGluY3Rpb24sIEFscmVhZHlUZXJtaW5hbCBtb25vdG9uaWNpdHksIGFuZCBzdG9wL2Nh
bmNlbCBub24tYWxpYXMuCjQuIEZpbmFsLXJlY2VpcHQgZ2F0ZSBwcmVzZXJ2ZXMgQjEgcHJlLUUy
IG5vdC1maW5hbC9ub3QtbW9kZWwtZmFjaW5nIGJvdW5kYXJ5LCBFMi9CMi4yL0IyLjEgb3duZXJz
aGlwL29yZGVyaW5nLCBhbGwgMTEgYWNjZXB0YW5jZSBjb25kaXRpb25zLCBhbmQgYWxsIHBvc3Qt
YWNjZXB0YW5jZSBpbW11dGFiaWxpdHkvZnV0dXJlLW9ubHkvcmV2b2NhdGlvbi9mYWlsLWNsb3Nl
ZCBidWxsZXRzLgo1LiBOYXJyb3dpbmcgZ2F0ZSBwcmVzZXJ2ZXMgcmVzb2x2ZXIgZm9ybXVsYSBv
cmRlciwgdGFibGUgcm93L2NlbGwgb3JkZXIgYW5kIHRydWUvZmFsc2UgdHJhbnNpdGlvbnMsIGFs
bG93L2Rlbnkgc2VtYW50aWNzLCBwYXRoIGV4YW1wbGVzLCBydWxlcyAx4oCTOCwgYWJzb2x1dGUv
ZXNjYXBlL2dsb2IvTlVML3N5bWxpbmsvcm9vdCBleGNsdXNpb25zLCBuby1maWxlLWV4aXN0ZW5j
ZSBjb21wYXJpc29uLCBlbmZvcmNlbWVudC10aW1lIHJlY2hlY2ssIGhvc3RfdmlzaWJsZSBmYWls
LWNsb3NlZCBleGNlcHRpb24sIGFsbG93X2NhcGFiaWxpdHlfbmFycm93aW5nIHJlamVjdGVkLW5v
dC1pZ25vcmVkIHJ1bGUsIG5vIGZvcmJpZGRlbiBjYXBhYmlsaXR5IGVuYWJsaW5nLCBhbmQgYnJv
a2VyL3dvcmxkLXNlcnZpY2UgZW5mb3JjZW1lbnQgYm91bmRhcnkuCjYuIFByb21vdGlvbiBnYXRl
IHByZXNlcnZlcyBgbm90IGNvbnNpZGVyZWQgbGFuZGVkIHVudGlsIHRlc3RzIHByb3ZlYCBhbmQg
b3JkZXJlZCB0ZXN0cyAx4oCTMTAsIGluY2x1ZGluZyBDTEkvUkVQTC9jcmFzaC9uby1yZWFwcGx5
LCBzbW9rZS9lMmUgam9pbiwgY3JlZGVudGlhbC9nYXRld2F5IHByb29mLCBjb21wYXRpYmlsaXR5
IGV2aWRlbmNlIG5vbi1wcm9tb3Rpb24sIGJvdGggYWNjZXB0ZWQtd29yayBmYW1pbGllcywgZHVy
YWJsZSBzdXBlcnZpc29yLCBmb3JlZ3JvdW5kLWRyb3AgdHJ1dGgsIGFuZCBCMy4xLWFmdGVyLWpv
aW50LWNsb3Nlb3V0IG9yZGVyaW5nLgo3LiBFeGFjdCByb290IGxlZ2FjeSBhbmNob3JzIHJlbWFp
biB1bmlxdWUvcmVzb2x2aW5nOiBgMTAtY2FuY2VsLW91dGNvbWUtY2F0ZWdvcmllc2AsIGAxMi1m
aW5hbC1yZWNlaXB0LWltbXV0YWJsZS1wb2xpY3lzbmFwc2hvdHYzLWFjY2VwdGFuY2UtcnVsZXNg
LCBgMTMtZGlzcGF0Y2gtbmFycm93aW5nLW1vbm90b25pY2l0eS1ydWxlc2AsIGAxNC1jb250cmFj
dC1wcm9tb3Rpb24tZ2F0ZXNgLgo4LiBTdXBlcnZpc29yIGlkZW1wb3RlbmN5IDExIGFuZCBkaWZm
ZXJlbnRpYWwtYmFzZWxpbmUgMTFBIHJlbWFpbiB1bmNoYW5nZWQgcHJpb3Itb3duZXIgcHJvamVj
dGlvbnMuIExpbmUgMjMyIGBSZW1haW5pbmcgUjItMiBzYW1lLXByb2Nlc3MgY2FycmllciBjbG9z
dXJlYCBhbmQgYWxsIGxhdGVyIHBhY2tldC1sb2NhbC9hdXRob3JpdHktcmVxdWlyZWQgY29tcGF0
aWJpbGl0eSBtYXRlcmlhbCByZW1haW4gYnl0ZS1zdGFibGUgYW5kIGFyZSBub3QgcmUtZXh0cmFj
dGVkLgo5LiBFeGFjdGx5IG9uZSBjb3JyZWN0IGluZGV4IHJvdyBhbmQgb25lIHRydXRoZnVsIEQx
MCBsZWRnZXIgcm93IHBlciBvd25lci4gRXZlcnkgcm93IGluY2x1ZGVzIGV4YWN0IHByb3ZlbmFu
Y2UgYW5kIGEgY29tcGxldGUgZm91ci1vd25lciBhdG9taWMgcm9sbGJhY2sgcmVzdG9yaW5nIGFs
bCBmb3VyIHJvb3QgYm9kaWVzL3JlbW92aW5nIGZvdXIgb3duZXJzL2ZvdXIgaW5kZXggcm93cy9m
b3VyIGxlZGdlciByb3dzIHRvZ2V0aGVyLgoxMC4gUHJpb3IgRDMvRDUtRDEwIG93bmVycyBhbmQg
cmV2aWV3IGFydGlmYWN0cyBhcmUgc3RhYmxlLiBFeGFjdCBzZXZlbi1wYXRoIGZlbmNlIG9ubHk7
IG5vIGZvcm1hdHRpbmcgbm9ybWFsaXphdGlvbiBvciB1bnJlbGF0ZWQgZWRpdC4KMTEuIENhbmRp
ZGF0ZSBkb2VzIG5vdCBjbGFpbSBpbXBsZW1lbnRhdGlvbiwgZXZpZGVuY2UvZ2F0ZSBzYXRpc2Zh
Y3Rpb24sIHByb21vdGlvbiwgc3VjY2Vzc29yIGF1dGhvcml0eSwgRDEwIGNvbXBsZXRpb24sIEQx
MSBhdXRob3JpdHkvZGVjb21wb3NpdGlvbiwgb3IgRDEyIGN1dG92ZXIuCjEyLiBSb290LW93bmVk
IGV4aGF1c3RpdmUgY2xhc3NpZmljYXRpb24gZXZpZGVuY2UgaXMgY29uZGl0aW9uYWwsIG5vdCBh
IHBhdGNoIGNsYWltOiBhZnRlciB0aGlzIGNhbmRpZGF0ZSwgcm9vdCBoYXMgNjkgY2Fub25pY2Fs
IHBvaW50ZXIgbGluZXMsIDU1IGNvbXBhdGliaWxpdHktYW5jaG9yLW9ubHkgbGluZXMsIHplcm8g
cmVzaWR1YWwgbm9uaGVhZGluZy9ub25wb2ludGVyIHN1YnN0YW50aXZlIGxpbmVzLCB6ZXJvIHJl
c2lkdWFsIGNvZGUvdGFibGUvbGlzdCBzeW50YXgsIDEyIEQxMCBsZWRnZXIgcm93cywgYW5kIG5v
IGJsb2NrZWQgcm9hZG1hcCB0YXJnZXQ7IGF1ZGl0IFNIQS0yNTYgODEwMGVjZjdlYjhmMTg3Njhi
ZTJkNTBiMWZlMzZhMDIzYWM4NDA0NTIyNDg0Y2Q5NmUzYjNkMTEzMzc5M2FiNi4gT25seSBhZnRl
ciBjbGVhbiByZXZpZXcgYW5kIGF0b21pYyBjbG9zZW91dCBtYXkgRDEwIGJlIGRlY2xhcmVkIGNv
bXBsZXRlOyBEMTEgcmVtYWlucyBzZXBhcmF0ZWx5IHVuYXV0aG9yaXplZC4KClJlcXVpcmVkIG1h
dGVyaWFsIGNvbnNlcXVlbmNlOiBibG9jayBvbmx5IGEgcmVhY2hhYmxlLCBwYXRjaC1jYXVzYWwg
ZGVmZWN0IHRoYXQgbWF0ZXJpYWxseSBsb3Nlcy9hbHRlcnMvcmVvcmRlcnMgY29udHJhY3Qgb3Ig
Z2F0ZSBzZW1hbnRpY3M7IHdlYWtlbnMgbmVnYXRpdmUvZmFpbC1jbG9zZWQvaW1tdXRhYmlsaXR5
L25vbi1wcm9tb3Rpb24gcmVxdWlyZW1lbnRzOyBtaXNpZGVudGlmaWVzIG93bmVyc2hpcC9zdGF0
dXM7IGJyZWFrcyByZXF1aXJlZCBsaW5rL2FuY2hvci9wcm92ZW5hbmNlL2F0b21pYyByb2xsYmFj
azsgcmUtZXh0cmFjdHMgcHJpb3Itb3duZWQgbGluZS0yMzIrIG1hdGVyaWFsOyBicmVhY2hlcyB0
aGUgc2V2ZW4tcGF0aCBmZW5jZTsgb3IgcHJldmVudHMgY2xlYW4gYXBwbGljYXRpb24vcmV2ZXJz
YWwuIFN0eWxlIHByZWZlcmVuY2VzLCBvcHRpb25hbCB3b3JkaW5nLCBicm9hZGVyIHJlZGVzaWdu
LCB1bnN1cHBvcnRlZCBNYXJrZG93biBiZWhhdmlvciwgYW5kIHVuY2hhbmdlZCBiYXNlbGluZSBk
ZWZlY3RzIGFyZSBub25tYXRlcmlhbC4KQmxvY2tpbmcgdGhyZXNob2xkOiBvbmx5IHJlYWNoYWJs
ZSBQMS9QMiBmaW5kaW5ncyB2aW9sYXRpbmcgYW4gaW52YXJpYW50IHdpdGggbWF0ZXJpYWwgY29u
c2VxdWVuY2UuIEJvZHkvaGFzaCBtaXNtYXRjaCwgZW51bS9maWVsZC9ydWxlL3RhYmxlL29yZGVy
IGxvc3MsIGZhbHNlIGdhdGUgc2F0aXNmYWN0aW9uL2xhbmRpbmcsIHdlYWtlbmVkIHJlamVjdGlv
bi9mYWlsLWNsb3NlZCBjb25zdHJhaW50LCB3cm9uZyBvd25lciwgYnJva2VuIHJlcXVpcmVkIGFu
Y2hvci9saW5rLCBpbmNvbXBsZXRlIGZvdXItb3duZXIgcm9sbGJhY2ssIGZhbHNlIGF1dGhvcml0
eS9zdGF0dXMsIG9yIHNjb3BlIGJyZWFjaCBxdWFsaWZpZXMuIExhYmVsIG90aGVyIG9ic2VydmF0
aW9ucyBkZWZlcnJlZCBhbmQgZG8gbm90IGNoYW5nZSB2ZXJkaWN0LgpBY2NlcHRlZCBwcmlvciBm
aW5kaW5nczogbm9uZS4KRGVmZXJyZWQvb3V0IG9mIHNjb3BlOiBydW50aW1lIGltcGxlbWVudGF0
aW9uIG9yIHRlc3QgYWRlcXVhY3kgYmV5b25kIHByZXNlcnZlZCBnYXRlIHRleHQ7IEQzIHJldmll
dyBnb3Zlcm5hbmNlOyBwcmlvciBENS9ENi9EOSBvd25lcnM7IEQxMSBldmlkZW5jZTsgRDEyIGN1
dG92ZXI7IFJ1c3Qvc2NyaXB0cy9wbGF0Zm9ybSBiZWhhdmlvcjsgcm9hZG1hcC9aSVA7IHJldmll
dy1jb250cm9sL3NsaWNlcy90YXNrczsgZ2xvYmFsIGZvcm1hdHRpbmc7IHVucmVsYXRlZCBwcmUt
ZXhpc3RpbmcgaXNzdWVzOyBpbXBsZW1lbnRhdGlvbi9zdWNjZXNzb3IgZGlzcGF0Y2g7IGluZGVw
ZW5kZW50IHJlLWF1ZGl0IG9mIHVuY2hhbmdlZCByZXBvc2l0b3J5IHNvdXJjZXMgbm90IHN1cHBs
aWVkIGhlcmUuClByb2plY3Qgc291cmNlczogb25seSB0aGlzIHByb21wdCwgbWFuaWZlc3QsIGlu
dmFyaWFudHMsIHZhbGlkYXRpb24vYXVkaXQgZmFjdHMsIGFuZCBjb21wbGV0ZSBwYXRjaC4gRXhj
bHVkZWQgc291cmNlIGlzIHVuYXZhaWxhYmxlIGFuZCBtdXN0IG5vdCBiZSBpbmZlcnJlZC4KClZh
bGlkYXRpb24gZXZpZGVuY2U6Ci0gZXhhY3Qgc2V2ZW4tcGF0aCBmZW5jZS9ubyBzdGFnZWQgZmls
ZXM6IFBBU1MKLSBgZ2l0IGRpZmYgLS1jaGVja2A6IFBBU1MKLSBuby1pbmRleCBjaGVja3MgYWxs
IGZvdXIgb3duZXJzOiBvcmRpbmFyeSBkaWZmZXJlbmNlIGV4aXQsIHplcm8gZGlhZ25vc3RpY3MK
LSBNYXJrZG93biBsaW5rL2FuY2hvciB2YWxpZGF0aW9uOiAxLDg4OSBjaGVja2VkLCAwIGZhaWx1
cmVzCi0gZm91ciBtYXJrZXIgYm9kaWVzIGV4YWN0IGJ5dGVzL2hhc2hlczsgcm9vdCBwb2ludGVy
cy9vd25lciBhbmNob3JzIHVuaXF1ZTsgZXhhY3RseSBvbmUgaW5kZXgvbGVkZ2VyIHJvdyBlYWNo
OiBQQVNTCi0gZW51bS9maWVsZC9ydWxlL3RhYmxlL2Zvcm11bGEvbGl0ZXJhbC9uZWdhdGl2ZS9l
eGNlcHRpb24vYXV0aG9yaXR5L29yZGVyIHByZXNlcnZhdGlvbjogUEFTUwotIHNlY3Rpb25zIDEx
LzExQSBhbmQgbGluZS0yMzIgb253YXJkIGJ5dGUtc3RhYmxlOyBwcmlvciBvd25lcnMvcmV2aWV3
cyBzdGFibGU6IFBBU1MKLSBwYXRjaCBmb3J3YXJkIGV4YWN0IHNldmVuLWZpbGUgYXBwbHkgYW5k
IHJldmVyc2UgY2xlYW4gcmVzdG9yYXRpb246IFBBU1MKLSBjYW5kaWRhdGUgdmFsaWRhdGlvbiBs
b2cgU0hBLTI1NiA0MDUzYjMwZmZmOTkxMjFjYTlkOWUyM2Q2MzdlOWVlYTJjNTdlMzVlYmI0Njdh
MWU3MjVkMzAxMmEzOWVmMzliCgpEbyBub3QgcmVkdWNlIHRoZSB0YXNrIG9yIHJlcGxhY2UgaXQg
d2l0aCBhbiBlYXNpZXIgYWx0ZXJuYXRpdmUuIFByZXNlcnZlIHRoZSByZXF1ZXN0ZWQgc2NvcGUg
YW5kIHByb2plY3QgY29udmVudGlvbnMuCgpSZXR1cm4gZmluZGluZ3MgZmlyc3QgYnkgc2V2ZXJp
dHkuIEVhY2ggYmxvY2tlciBtdXN0IG5hbWUgZXhhY3QgZmlsZS9odW5rLCBzdXBwb3J0ZWQtaW5w
dXQgcmVhY2hhYmlsaXR5LCB2aW9sYXRlZCBpbnZhcmlhbnQsIG1hdGVyaWFsIGNvbnNlcXVlbmNl
LCBhbmQgcGF0Y2ggY2F1c2FsaXR5LiBTdGF0ZSBleHBsaWNpdGx5IHdoZW4gbm8gcXVhbGlmeWlu
ZyBmaW5kaW5ncyBleGlzdC4gRW5kIGV4YWN0bHk6ClZFUkRJQ1Q6IEFQUFJPVkVECm9yClZFUkRJ
Q1Q6IENIQU5HRVMgUkVRVUlSRUQKCkFkdmlzb3J5IG9ubHk7IHZlcmlmeSBhZ2FpbnN0IGxvY2Fs
IHByb2plY3QgdHJ1dGggYW5kIGF1dGhvcml0YXRpdmUgZG9jczsgZG8gbm90IHJlZHVjZSBzY29w
ZSB3aXRob3V0IHVzZXIgYXBwcm92YWwuCgpDT01QTEVURSBDT05URU5ULUFERFJFU1NFRCBQQVRD
SCAoU0hBLTI1NiBiYzNjZjg4ZjVmNmNiYzFmNTUzMTYxOGFhM2JlMWRkYmJmZTVjNWU0MTMyNmE4
ZTlmMDkyMjk4YjI3MjVlODAxKToKYGBgZGlmZgpkaWZmIC0tZ2l0IGEvbGxtLWxhc3QtbWlsZS9y
dW50aW1lLXJlZmFjdG9yLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQgYi9sbG0tbGFzdC1taWxl
L3J1bnRpbWUtcmVmYWN0b3IvMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZAppbmRleCAxNmNhYjRm
MWE2Njg5NWI3NjEwYjRlNWVlNzU5ZjQwNDAxNjVkMWZlLi43NDEwZjBiMmNhNmU3NDcwYzNkN2Jm
MWRkZjljOTU2NGNjNjhjMjA1IDEwMDY0NAotLS0gYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVm
YWN0b3IvMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZAorKysgYi9sbG0tbGFzdC1taWxlL3J1bnRp
bWUtcmVmYWN0b3IvMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZApAQCAtMTIxLDIyICsxMjEgQEAg
Q2Fub25pY2FsIGNvbnRlbnQ6IFtgY29udHJhY3RzL2xhdW5jaC10aW1lLXNlY3JldC1oYW5kb2Zm
LXYxLm1kIzktbGF1bmNodGltZXNlY3IKLWBgYHJ1c3QKLWVudW0gQ2FuY2VsV29ybGRXb3JrT3V0
Y29tZVYxIHsKLSAgICBDYW5jZWxsZWRWaWFMaXZlVHJhbnNwb3J0IHsgYWN0aXZlX3J1bl9pZCwg
dGVybWluYWxfcmVmIH0sCi0gICAgQ2FuY2VsQWNjZXB0ZWRQZW5kaW5nQ2xvc2VvdXQgeyBhY3Rp
dmVfcnVuX2lkLCBjYW5jZWxfcmVxdWVzdF9pZCB9LAotICAgIEFscmVhZHlUZXJtaW5hbCB7IGFj
dGl2ZV9ydW5faWQsIHRlcm1pbmFsX3JlZiB9LAotICAgIE5vQWN0aXZlQ2FuY2VsYWJsZVdvcmsg
eyB3b3JrZXJfb3JfdGFza19yZWYgfSwKLSAgICBPd25lclVucmVhY2hhYmxlIHsgYWN0aXZlX3J1
bl9pZCwgZHVyYWJsZV9zdGF0ZSwgcmV0cnlhYmlsaXR5IH0sCi0gICAgSW52YWxpZFRhcmdldCB7
IHJlYXNvbiB9LAotICAgIFdvcmxkQmluZGluZ01pc21hdGNoIHsgZXhwZWN0ZWQsIGFjdHVhbCB9
LAotICAgIEFtYmlndW91c1RhcmdldCB7IGNhbmRpZGF0ZXMgfSwKLSAgICBQb2xpY3lEZW5pZWQg
eyBkZW5pYWxfY29kZSwgZXhwbGFuYXRpb24gfSwKLX0KLWBgYAotCi1SdWxlczoKLQotMS4gRXhh
Y3QgaWRlbnRpdHkgYW5kIHdvcmxkIGJpbmRpbmcgcmVzb2x2ZSBiZWZvcmUgdHJhbnNwb3J0IHVz
ZS4KLTIuIGBOb0FjdGl2ZUNhbmNlbGFibGVXb3JrYCBtZWFucyB2YWxpZCByb3V0aW5nIGNvbnRl
eHQgYnV0IG5vIGFjY2VwdGVkIG5vbi10ZXJtaW5hbCBjYW5jZWxhYmxlIHJlY2VpcHQuCi0zLiBg
T3duZXJVbnJlYWNoYWJsZWAgbWVhbnMgYSB2YWxpZCBhY3RpdmUgcmVjZWlwdCBleGlzdHMgYnV0
IHRoZSBsaXZlIGNhbmNlbGxhdGlvbiByb3V0ZSBpcyB1bmF2YWlsYWJsZSBhbmQgZHVyYWJsZSBw
b2xpY3kgY2Fubm90IHlldCBkZWNsYXJlIGNsb3Nlb3V0LgotNC4gYENhbmNlbEFjY2VwdGVkUGVu
ZGluZ0Nsb3Nlb3V0YCBpcyBub3QgdGVybWluYWwgc3VjY2VzczsgaW5zcGVjdCByZW1haW5zIGFi
bGUgdG8gb2JzZXJ2ZSBldmVudHVhbCBjbG9zZW91dC4KLTUuIFJlcGVhdGVkIGNhbmNlbCBhZnRl
ciB0ZXJtaW5hbCByZXR1cm5zIGBBbHJlYWR5VGVybWluYWxgOyBpdCBuZXZlciByZWdyZXNzZXMg
dGhlIHJlY2VpcHQuCi02LiBTdG9wIHRhcmdldHMgd29ya2VyIGxpZmVjeWNsZS4gQ2FuY2VsIHRh
cmdldHMgb25lIGFjdGl2ZSB0YXNrL3R1cm4uIFRoZXkgYXJlIG5vdCBhbGlhc2VzLgorQ2Fub25p
Y2FsIGNvbnRlbnQ6IFtgY29udHJhY3RzL2NhbmNlbC1vdXRjb21lLWNhdGVnb3JpZXMubWQjMTAt
Y2FuY2VsLW91dGNvbWUtY2F0ZWdvcmllc2BdKGNvbnRyYWN0cy9jYW5jZWwtb3V0Y29tZS1jYXRl
Z29yaWVzLm1kIzEwLWNhbmNlbC1vdXRjb21lLWNhdGVnb3JpZXMpLgpAQCAtMTU0LDI2ICsxMzMg
QEAgQ2Fub25pY2FsIGNvbnRlbnQ6IFtgYjEtYjItMS9jb250cmFjdHMtYW5kLWdhdGVzLm1kIzEx
YS1kaWZmZXJlbnRpYWwtYmFzZWxpbmUtdHIKLUIxJ3MgcHJlLUUyIGFjY2VwdGFuY2UgYW5jaG9y
IHJlY29yZHMgdGhlIGV4YWN0IGN1cnJlbnQgcG9saWN5IGlkZW50aXR5IHVzZWQgYnkgdGhlIHJ1
bnRpbWUgYnV0IGlzCi1ub3QgYSBmaW5hbCByZWNlaXB0IGFuZCBpcyBub3QgbW9kZWwtZmFjaW5n
LiBFMiBvd25zIHRoZSBpbW11dGFibGUgYWN0aXZlLXJ1biBzbmFwc2hvdCBhbmQgcmV0YWluZWQK
LXdvcmtlciBjYXAgYmVsb3c7IEIyLjIgbWF5IGV4cG9zZSBhIHJlY2VpcHQgb25seSBhZnRlciB0
aG9zZSBjb21taXRtZW50cyBhbmQgdGhlIEIyLjEgb2JzZXJ2YXRpb24KLWNsYWltIGFyZSBkdXJh
YmxlIGFuZCBsaW5rZWQgdG8gdGhlIEIxIHJlY29yZC4KLQotQSBmaW5hbCBhY3RpdmUgdGFzay90
dXJuIHJlY2VpcHQgbWF5IGJlIGV4cG9zZWQgb25seSB3aGVuIGFsbCBhcmUgdHJ1ZToKLQotMS4g
ZXhhY3Qgc2Vzc2lvbiwgY2FsbGVyLCBiYWNrZW5kLCBhbmQgd29ybGQgYmluZGluZyBhcmUgcmVz
b2x2ZWQ7Ci0yLiBzdGVlcmluZyBwb2xpY3kgYWxsb3dzIHRoZSB2ZXJiL21vZGUvdGFyZ2V0Owot
My4gY3VycmVudCBwYXJlbnQgcG9saWN5IGlzIHJlc29sdmVkIGF0IGEga25vd24gcmV2aXNpb247
Ci00LiByZXRhaW5lZCB3b3JrZXIgY2FwIGlzIGxvYWRlZCBhbmQgaGFzaC12ZXJpZmllZCB3aGVu
IGFwcGxpY2FibGU7Ci01LiBvcHRpb25hbCBuYXJyb3dpbmcgaXMgdmFsaWRhdGVkIGFzIG1vbm90
b25pYzsKLTYuIHRoZSByZXN1bHRpbmcgYFBvbGljeVNuYXBzaG90VjNgIGNhbm9uaWNhbGl6ZXMg
YW5kIHBhc3NlcyBleGlzdGluZyBzY2hlbWEvZW5mb3JjZW1lbnQgdmFsaWRhdGlvbjsKLTcuIHNu
YXBzaG90IGJ5dGVzL3JlZi9oYXNoL3JldmlzaW9uIGFyZSBkdXJhYmxlOwotOC4gdGhlIGV4ZWN1
dGlvbiBlbnZlbG9wZS93b3JsZC1zZXJ2aWNlIHJlcXVlc3QgY2FycmllcyB0aGUgc2FtZSB2ZXJp
ZmllZCBzbmFwc2hvdDsKLTkuIHJ1bnRpbWUgYWNjZXB0YW5jZSBldmlkZW5jZSBqb2lucyB0aGUg
YWNrbm93bGVkZ2VtZW50IHRvIHRoZSBleGFjdCB3b3JrIGlkZW50aXR5OwotMTAuIHRoZSBvYnNl
cnZhdGlvbiBjbGFpbSBpcyBkdXJhYmxlIGFuZCByZXN1bWFibGU7IGFuZAotMTEuIHRoZSByZWNl
aXB0IHJlZmVyZW5jZXMgdGhlIHNuYXBzaG90LCBhY2NlcHRhbmNlIGV2aWRlbmNlLCBhbmQgb2Jz
ZXJ2YXRpb24gY2xhaW0gYmVmb3JlIHRoZSBmb3JlZ3JvdW5kIGNhbGxlciBpcyB0b2xkIHRoZSB3
b3JrIHdhcyBhY2NlcHRlZC4KLQotQWZ0ZXIgYWNjZXB0YW5jZToKLQotLSB0aGUgcmVjZWlwdCdz
IHBvbGljeSByZWYvaGFzaCBpcyBpbW11dGFibGU7Ci0tIHBhcmVudCBicm9hZGVuaW5nIG9yIG5h
cnJvd2luZyBkb2VzIG5vdCByZXdyaXRlIHRoZSBhY3RpdmUgcmVjZWlwdDsKLS0gcGFyZW50IGNo
YW5nZXMgYXBwbHkgdG8gZnV0dXJlIHRhc2sgYWNjZXB0YW5jZSwgY29udGludWUsIGZvcmssIG9y
IHdvcmtlciB0dXJuczsKLS0gZW1lcmdlbmN5IHJldm9jYXRpb24gaXMgYW4gZXhwbGljaXQgYXVk
aXRlZCBjYW5jZWwvcmV2b2tlIHBhdGgsIG5ldmVyIHNpbGVudCBzbmFwc2hvdCBtdXRhdGlvbjsg
YW5kCi0tIHNuYXBzaG90IG1pc21hdGNoIGF0IGJyb2tlci93b3JsZC1zZXJ2aWNlIGZhaWxzIGNs
b3NlZC4KK0Nhbm9uaWNhbCBjb250ZW50OiBbYGdhdGVzL2ZpbmFsLXJlY2VpcHQtaW1tdXRhYmxl
LXBvbGljeS1zbmFwc2hvdC12My1hY2NlcHRhbmNlLm1kIzEyLWZpbmFsLXJlY2VpcHQtaW1tdXRh
YmxlLXBvbGljeXNuYXBzaG90djMtYWNjZXB0YW5jZS1ydWxlc2BdKGdhdGVzL2ZpbmFsLXJlY2Vp
cHQtaW1tdXRhYmxlLXBvbGljeS1zbmFwc2hvdC12My1hY2NlcHRhbmNlLm1kIzEyLWZpbmFsLXJl
Y2VpcHQtaW1tdXRhYmxlLXBvbGljeXNuYXBzaG90djMtYWNjZXB0YW5jZS1ydWxlcykuCkBAIC0x
ODMsMzEgKzEzNyBAQCBBZnRlciBhY2NlcHRhbmNlOgotVGhlIHJlc29sdmVyIGNvbXB1dGVzOgot
Ci1gYGB0ZXh0Ci1lcGhlbWVyYWwgPSBjdXJyZW50X3BhcmVudCBBTkQgZGlzcGF0Y2hfcGF0Y2gK
LXdvcmtlcl9jYXAgPSBjdXJyZW50X3BhcmVudF9hdF9zcGF3biBBTkQgc3Bhd25fcGF0Y2gKLXR1
cm4gPSBjdXJyZW50X3BhcmVudF9ub3cgQU5EIHdvcmtlcl9jYXAgQU5EIHR1cm5fcGF0Y2gKLWZv
cmtfY2FwID0gY3VycmVudF9wYXJlbnRfbm93IEFORCBzb3VyY2Vfd29ya2VyX2NhcCBBTkQgZm9y
a19wYXRjaAotYGBgCi0KLVYxIGZpZWxkIHJ1bGVzOgotCi18IEZpZWxkIHwgQWxsb3dlZCBuYXJy
b3dpbmcgfCBSZWplY3RlZCBicm9hZGVuaW5nIHwKLXwtLS18LS0tfC0tLXwKLXwgYHdvcmxkX2Zz
Lmhvc3RfdmlzaWJsZWAgfCBgdHJ1ZSAtPiBmYWxzZWAgfCBgZmFsc2UgLT4gdHJ1ZWAgfAotfCBg
d29ybGRfZnMuZmFpbF9jbG9zZWQucm91dGluZ2AgfCBgZmFsc2UgLT4gdHJ1ZWAgfCBgdHJ1ZSAt
PiBmYWxzZWAgfAotfCBgd29ybGRfZnMuY2FnZWRfcmVxdWlyZWRgIHwgYGZhbHNlIC0+IHRydWVg
IHwgYHRydWUgLT4gZmFsc2VgIHwKLXwgYHdvcmxkX2ZzLndyaXRlLmVuYWJsZWRgIHwgYHRydWUg
LT4gZmFsc2VgIHwgYGZhbHNlIC0+IHRydWVgIHwKLXwgYHdvcmxkX2ZzLmRlbnlfZW5mb3JjZW1l
bnRgIHwgc2FtZSBvciBzdHJvbmdlciByYW5rIHwgd2Vha2VyIHJhbmsgb3IgcmVtb3ZhbCB8Ci18
IGRpc2NvdmVyL3JlYWQvd3JpdGUgYGFsbG93X2xpc3RgIHwgZWFjaCByZXF1ZXN0ZWQgcGF0aCBp
cyBjb250YWluZWQgYnkgYXQgbGVhc3Qgb25lIHBhcmVudCBwYXRoIHwgcm9vdC9kaXJlY3Rvcnkv
ZmlsZSB3aWRlbmluZyB8Ci18IGRpc2NvdmVyL3JlYWQvd3JpdGUgYGRlbnlfbGlzdGAgfCBhZGQg
ZGVuaWFscyBvciByZXRhaW4gcGFyZW50IGRlbmlhbHMgfCByZW1vdmUgcGFyZW50IGRlbmlhbCB8
Ci0KLVBhdGgtY29udGFpbm1lbnQgcnVsZXM6Ci0KLTEuIE5vcm1hbGl6ZSByZWxhdGl2ZSB0byB0
aGUgYXV0aG9yaXRhdGl2ZSB3b3JsZC9wcm9qZWN0IHJvb3QuCi0yLiBgLmAgY29udGFpbnMgYHNy
Y2AgYW5kIGBzcmMvcGFyc2VyLnJzYDsgYHNyY2AgY29udGFpbnMgYHNyYy9wYXJzZXIucnNgOyBh
IGZpbGUgY29udGFpbnMgb25seSBpdHNlbGYuCi0zLiBSZWplY3QgYWJzb2x1dGUgaG9zdCBwYXRo
cywgYC4uYCBlc2NhcGUsIHVuc3VwcG9ydGVkIGdsb2Igc2VtYW50aWNzLCBOVUwsIHN5bWxpbmsg
ZXNjYXBlLCBhbmQgcGF0aHMgb3V0c2lkZSB0aGUgcm9vdC4KLTQuIENvbXBhcmUgY2Fub25pY2Fs
IHBvbGljeSBwYXRocyB3aXRob3V0IHJlcXVpcmluZyB0aGUgdGFyZ2V0IGZpbGUgdG8gYWxyZWFk
eSBleGlzdDsgcnVudGltZSByZXNvbHV0aW9uIG11c3QgcmUtY2hlY2sgc3ltbGluay9hbmNlc3Rv
ciBlc2NhcGUgYXQgZW5mb3JjZW1lbnQgdGltZS4KLTUuIGBob3N0X3Zpc2libGU9dHJ1ZWAgd2l0
aCBkZW55LWxpc3Qgb3IgdW5wcm92YWJsZSBpc29sYXRpb24gc2VtYW50aWNzIGZhaWxzIGNsb3Nl
ZCB1bmxlc3MgdGhlIHBhdGNoIGxlZ2FsbHkgbmFycm93cyB0byBzdXBwb3J0ZWQgZnVsbCBpc29s
YXRpb24uCi02LiBBIHBhdGNoIHN1cHBsaWVkIHdoaWxlIGBhZ2VudHMud29ybGRfZGlzcGF0Y2gu
YWxsb3dfY2FwYWJpbGl0eV9uYXJyb3dpbmc9ZmFsc2VgIGlzIHJlamVjdGVkLCBub3QgaWdub3Jl
ZC4KLTcuIE5hcnJvd2luZyBtYXkgbm90IGVuYWJsZSBhIGRpc3BhdGNoIGFjdGlvbiwgYmFja2Vu
ZCwgbW9kZSwgY2FwYWJpbGl0eSwgbmV0d29yayByb3V0ZSwgb3Igc2lkZS1lZmZlY3QgY2hhbm5l
bCBmb3JiaWRkZW4gYnkgdGhlIHBhcmVudC4KLTguIEFkYXB0ZXIgY29uZmlnIG1heSByZWNlaXZl
IHBvbGljeSBoaW50cywgYnV0IG9ubHkgYnJva2VyL3dvcmxkLXNlcnZpY2UgZW5mb3JjZW1lbnQg
Y291bnRzLgorQ2Fub25pY2FsIGNvbnRlbnQ6IFtgZ2F0ZXMvZGlzcGF0Y2gtbmFycm93aW5nLW1v
bm90b25pY2l0eS5tZCMxMy1kaXNwYXRjaC1uYXJyb3dpbmctbW9ub3RvbmljaXR5LXJ1bGVzYF0o
Z2F0ZXMvZGlzcGF0Y2gtbmFycm93aW5nLW1vbm90b25pY2l0eS5tZCMxMy1kaXNwYXRjaC1uYXJy
b3dpbmctbW9ub3RvbmljaXR5LXJ1bGVzKS4KQEAgLTIxNywxNCArMTQxIEBAIFBhdGgtY29udGFp
bm1lbnQgcnVsZXM6Ci1BIGNvbnRyYWN0IGlzIG5vdCBjb25zaWRlcmVkIGxhbmRlZCB1bnRpbCB0
ZXN0cyBwcm92ZToKLQotMS4gc2VyaWFsaXphdGlvbiBhbmQgdmFsaWRhdGlvbjsKLTIuIGF0b21p
YyBwZXJzaXN0ZW5jZSBhbmQgcmV2aXNpb24gY29uZmxpY3QgaGFuZGxpbmc7Ci0zLiB0aGUgcmVh
bCBpbmdyZXNzL2Rpc3BhdGNoL3J1bnRpbWUgcGF0aCB1c2VzIGl0OwotNC4gcmVzdGFydC9yZXBs
YXkgYmVoYXZpb3Igd2hlcmUgZHVyYWJsZTsKLTUuIGZhaWwtY2xvc2VkIG5lZ2F0aXZlIGNhc2Vz
OwotNi4gZXZlcnkgcmV2aXNpb24tYm91bmQgaG9zdCB0cmFuc2l0aW9uIGpvaW5zIGludGVudCBp
c3N1YW5jZSwgY2xhaW0sIGF1dGhvcml0eSBhcHBsaWNhdGlvbiwgYW5kIGV4YWN0IHJlc3VsdCBv
biB0aGUgcmVhbCBDTEkgYW5kIFJFUEwgcGF0aCwgaW5jbHVkaW5nIGNyYXNoIHJlY29uY2lsaWF0
aW9uIGFuZCBuby1yZWFwcGx5IGV4YWN0IHJldHJ5OwotNy4gYXQgbGVhc3Qgb25lIHNtb2tlL2Uy
ZSBwYXRoIGpvaW5zIHNlc3Npb24sIGJpbmRpbmcsIHBvbGljeSwgcmVjZWlwdCwgcnVudGltZSBl
dmVudCwgYW5kIHRlcm1pbmFsL29ibGlnYXRpb24gdHJ1dGg7Ci04LiBjcmVkZW50aWFsLXJlcXVp
cmluZyB3b3JsZCBVQUEgcHJvb2Ygam9pbnMgdGhlIGVudmVsb3BlIHRvIGEgY29uc3VtZWQgb25l
LXRpbWUgaW4td29ybGQgZ2F0ZXdheSBoYW5kb2ZmIHdpdGhvdXQgY29waWVkIHNlY3JldCBmaWxl
cyBvciBpbmhlcml0ZWQgZGVzY3JpcHRvcnM7IGFuZAotOS4gbm8gY29tcGF0aWJpbGl0eSBjb3B5
IG9yIGBDb21wYXRpYmlsaXR5VW5wcm92ZW5gIGV2aWRlbmNlIGlzIHVzZWQgZm9yIGNvbnRyYWN0
IHByb21vdGlvbjsgYW5kCi0xMC4gQjEvQjIuMSBwcm9kdWN0aW9uIHByb29mIHNob3dzIGJvdGgg
YWNjZXB0ZWQgd29yayBmYW1pbGllcyBlbnRlciB0aGUgZHVyYWJsZSBzdXBlcnZpc29yIHdpdGhv
dXQKLSAgICBhIGxlZ2FjeS13cml0ZXIgYXR0ZW1wdCwgY2FsbGVyL2ZvcmVncm91bmQgZHJvcCBk
b2VzIG5vdCBlcmFzZSB0cnV0aCwgYW5kIEIzLjEgYmVnaW5zIG9ubHkgYWZ0ZXIKLSAgICB0aGUg
am9pbnQgY2xvc2VvdXQuCitDYW5vbmljYWwgY29udGVudDogW2BnYXRlcy9jb250cmFjdC1wcm9t
b3Rpb24ubWQjMTQtY29udHJhY3QtcHJvbW90aW9uLWdhdGVzYF0oZ2F0ZXMvY29udHJhY3QtcHJv
bW90aW9uLm1kIzE0LWNvbnRyYWN0LXByb21vdGlvbi1nYXRlcykuCmRpZmYgLS1naXQgYS9sbG0t
bGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvaW5kZXgvUkVBRE1FLm1kIGIvbGxtLWxhc3QtbWls
ZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZAppbmRleCBhNTU4ZmM1NmM1NzNjZWVi
OTBkZDRkNGM4NzMyMTFmY2RmZTIzZWYxLi44OWYzMzBkZDU4NzRmZWE2NGJhZTk5ZmRkNTYzZjQx
ZTM4NzZhNzRhIDEwMDY0NAotLS0gYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvaW5k
ZXgvUkVBRE1FLm1kCisrKyBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9S
RUFETUUubWQKQEAgLTI3LDAgKzI4LDQgQEAKK3wgQ2FuY2VsIG91dGNvbWUgY2F0ZWdvcmllcyB8
IGNvbnRyYWN0IHwgW2Bjb250cmFjdHMvY2FuY2VsLW91dGNvbWUtY2F0ZWdvcmllcy5tZGBdKC4u
L2NvbnRyYWN0cy9jYW5jZWwtb3V0Y29tZS1jYXRlZ29yaWVzLm1kKSB8IGBjYW5vbmljYWwgZXh0
cmFjdGVkIGNvbnRyYWN0IG93bmVyIGZvciB0aGUgY29tcGxldGUgQ2FuY2VsV29ybGRXb3JrT3V0
Y29tZVYxIGVudW0gYW5kIHJ1bGVzIDHigJM2IGNvdmVyaW5nIGlkZW50aXR5LWJlZm9yZS10cmFu
c3BvcnQgcmVzb2x1dGlvbiwgcGVuZGluZyBjbG9zZW91dCwgbm8tYWN0aXZlIHZlcnN1cyBvd25l
ci11bnJlYWNoYWJsZSwgdGVybWluYWwgbW9ub3RvbmljaXR5LCBhbmQgc3RvcC12ZXJzdXMtY2Fu
Y2VsIG5vbi1hbGlhc2luZ2AgfCBTdXBlcnNlZGVzIG9ubHkgcm9vdC1jYW5vbmljYWwgb3duZXJz
aGlwIG9mIHRoZSBleHRyYWN0ZWQgYENhbmNlbCBvdXRjb21lIGNhdGVnb3JpZXNgIHNwYW47IGBM
YXVuY2hUaW1lU2VjcmV0SGFuZG9mZlYxYCwgYFN1cGVydmlzb3IgaWRlbXBvdGVuY3kgYW5kIHJl
c3RhcnQgcnVsZXNgLCBgRGlmZmVyZW50aWFsLWJhc2VsaW5lIHRyYW5zaXRpb24gZ2F0ZWAsIGFu
ZCBuZWlnaGJvcmluZyBEMTAgb3duZXJzIHJlbWFpbiB1bmNoYW5nZWQuIHwgW2AwNGBdKC4uLzA0
LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMTAtY2FuY2VsLW91dGNvbWUtY2F0ZWdvcmllcykgfAor
fCBGaW5hbC1yZWNlaXB0IGltbXV0YWJsZSBgUG9saWN5U25hcHNob3RWM2AgYWNjZXB0YW5jZSBy
dWxlcyB8IGdhdGUgfCBbYGdhdGVzL2ZpbmFsLXJlY2VpcHQtaW1tdXRhYmxlLXBvbGljeS1zbmFw
c2hvdC12My1hY2NlcHRhbmNlLm1kYF0oLi4vZ2F0ZXMvZmluYWwtcmVjZWlwdC1pbW11dGFibGUt
cG9saWN5LXNuYXBzaG90LXYzLWFjY2VwdGFuY2UubWQpIHwgYGNhbm9uaWNhbCBleHRyYWN0ZWQg
Z2F0ZSBvd25lciBmb3IgdGhlIEIxIHByZS1FMiBhY2NlcHRhbmNlLWFuY2hvciBib3VuZGFyeSwg
b3JkZXJlZCBjb25kaXRpb25zIDHigJMxMSwgaW1tdXRhYmxlIHJlY2VpcHQtZXhwb3N1cmUgcnVs
ZSwgYW5kIHBvc3QtYWNjZXB0YW5jZSBmdXR1cmUtb25seSwgcmV2b2NhdGlvbiwgYW5kIGZhaWwt
Y2xvc2VkIGNvbnN0cmFpbnRzYCB8IFN1cGVyc2VkZXMgb25seSByb290LWNhbm9uaWNhbCBvd25l
cnNoaXAgb2YgdGhlIGV4dHJhY3RlZCBgRmluYWwtcmVjZWlwdCBpbW11dGFibGUgUG9saWN5U25h
cHNob3RWMyBhY2NlcHRhbmNlIHJ1bGVzYCBzcGFuOyBgU3VwZXJ2aXNvciBpZGVtcG90ZW5jeSBh
bmQgcmVzdGFydCBydWxlc2AsIGBEaWZmZXJlbnRpYWwtYmFzZWxpbmUgdHJhbnNpdGlvbiBnYXRl
YCwgYERpc3BhdGNoIG5hcnJvd2luZyBtb25vdG9uaWNpdHkgcnVsZXNgLCBhbmQgbmVpZ2hib3Jp
bmcgRDEwIG93bmVycyByZW1haW4gdW5jaGFuZ2VkLiB8IFtgMDRgXSguLi8wNC1jb250cmFjdHMt
YW5kLWdhdGVzLm1kIzEyLWZpbmFsLXJlY2VpcHQtaW1tdXRhYmxlLXBvbGljeXNuYXBzaG90djMt
YWNjZXB0YW5jZS1ydWxlcykgfAorfCBEaXNwYXRjaCBuYXJyb3dpbmcgbW9ub3RvbmljaXR5IHJ1
bGVzIHwgZ2F0ZSB8IFtgZ2F0ZXMvZGlzcGF0Y2gtbmFycm93aW5nLW1vbm90b25pY2l0eS5tZGBd
KC4uL2dhdGVzL2Rpc3BhdGNoLW5hcnJvd2luZy1tb25vdG9uaWNpdHkubWQpIHwgYGNhbm9uaWNh
bCBleHRyYWN0ZWQgZ2F0ZSBvd25lciBmb3IgdGhlIHJlc29sdmVyIGZvcm11bGFzLCBWMSBmaWVs
ZCB0YWJsZSwgcGF0aC1jb250YWlubWVudCBydWxlcyAx4oCTOCwgaG9zdF92aXNpYmxlIGZhaWwt
Y2xvc2VkIGV4Y2VwdGlvbiwgcmVqZWN0ZWQtbm90LWlnbm9yZWQgbmFycm93aW5nLCBhbmQgYnJv
a2VyL3dvcmxkLXNlcnZpY2UgZW5mb3JjZW1lbnQgYm91bmRhcnlgIHwgU3VwZXJzZWRlcyBvbmx5
IHJvb3QtY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIGBEaXNwYXRjaCBuYXJy
b3dpbmcgbW9ub3RvbmljaXR5IHJ1bGVzYCBzcGFuOyBgRmluYWwtcmVjZWlwdCBpbW11dGFibGUg
UG9saWN5U25hcHNob3RWMyBhY2NlcHRhbmNlIHJ1bGVzYCwgYENvbnRyYWN0IHByb21vdGlvbiBn
YXRlc2AsIGFuZCBuZWlnaGJvcmluZyBEMTAgb3duZXJzIHJlbWFpbiB1bmNoYW5nZWQuIHwgW2Aw
NGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMTMtZGlzcGF0Y2gtbmFycm93aW5nLW1v
bm90b25pY2l0eS1ydWxlcykgfAorfCBDb250cmFjdCBwcm9tb3Rpb24gZ2F0ZXMgfCBnYXRlIHwg
W2BnYXRlcy9jb250cmFjdC1wcm9tb3Rpb24ubWRgXSguLi9nYXRlcy9jb250cmFjdC1wcm9tb3Rp
b24ubWQpIHwgYGNhbm9uaWNhbCBleHRyYWN0ZWQgZ2F0ZSBvd25lciBmb3IgdGhlIHRlc3RzLXBy
b3ZlLWJlZm9yZS1sYW5kZWQgcHJvbW90aW9uIHJ1bGUgYW5kIG9yZGVyZWQgcHJvb2YgZ2F0ZXMg
MeKAkzEwIGNvdmVyaW5nIENMSS9SRVBMLCByZXN0YXJ0LCBzbW9rZSwgY3JlZGVudGlhbCBoYW5k
b2ZmLCBjb21wYXRpYmlsaXR5IGV4Y2x1c2lvbiwgYW5kIEIxL0IyLjEtdG8tQjMuMSBzZXF1ZW5j
aW5nYCB8IFN1cGVyc2VkZXMgb25seSByb290LWNhbm9uaWNhbCBvd25lcnNoaXAgb2YgdGhlIGV4
dHJhY3RlZCBgQ29udHJhY3QgcHJvbW90aW9uIGdhdGVzYCBzcGFuOyBgRGlzcGF0Y2ggbmFycm93
aW5nIG1vbm90b25pY2l0eSBydWxlc2AsIGBSZW1haW5pbmcgUjItMiBzYW1lLXByb2Nlc3MgY2Fy
cmllciBjbG9zdXJlYCwgYW5kIG5laWdoYm9yaW5nIEQxMCBvd25lcnMgcmVtYWluIHVuY2hhbmdl
ZC4gfCBbYDA0YF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMxNC1jb250cmFjdC1wcm9t
b3Rpb24tZ2F0ZXMpIHwKZGlmZiAtLWdpdCBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rv
ci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUt
cmVmYWN0b3IvbWlncmF0aW9uL2V4dHJhY3Rpb24tbGVkZ2VyLm1kCmluZGV4IDVhN2RhYWYyODA2
ZGU4MTA5NTdiYjFjNTVhZWZmODg0NzNkNzFhYWYuLjgxOTBiYWQ5NjE4ODcwM2I4YzA0ZDM4M2Y3
NTNmMzgwNTRhNjg2ODUgMTAwNjQ0Ci0tLSBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rv
ci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQKKysrIGIvbGxtLWxhc3QtbWlsZS9ydW50
aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZApAQCAtMTk0LDAgKzE5
NSw0IEBAIFRoaXMgbGVkZ2VyIHJlY29yZHMgY29udGVudC1wcmVzZXJ2aW5nIGF1dGhvcml0eSB0
cmFuc2ZlcnMgd2hpbGUgcmV0YWluaW5nIHJlcXVpCit8IEQxMCB8IFtgMDQtY29udHJhY3RzLWFu
ZC1nYXRlcy5tZGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQpIHwgMTAuIENhbmNlbCBv
dXRjb21lIGNhdGVnb3JpZXMgfCBbYDEwLWNhbmNlbC1vdXRjb21lLWNhdGVnb3JpZXNgXSguLi8w
NC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzEwLWNhbmNlbC1vdXRjb21lLWNhdGVnb3JpZXMpIHwg
bGluZXMgMTE54oCTMTQzICgxMTc5IGJ5dGVzKSB8IGBlOGYzNzNjOWU3Y2JmMTZkMjg2ZmY4Zjlh
OTE0NjMwNjY0NGU0MzBlM2Q2MzBiNzA3OGVjNzUwMWI3MmVkNGIzYCB8IFtgLi4vY29udHJhY3Rz
L2NhbmNlbC1vdXRjb21lLWNhdGVnb3JpZXMubWRgXSguLi9jb250cmFjdHMvY2FuY2VsLW91dGNv
bWUtY2F0ZWdvcmllcy5tZCkgfCBjb250cmFjdCB8IGNhbm9uaWNhbCBkZXN0aW5hdGlvbjsgc291
cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9yIHwgbm9uZSB8IHJlc3RvcmUgdGhlIGV4YWN0IDExNzkt
Ynl0ZSBzb3VyY2Ugc3BhbiBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMxMC1jYW5jZWwt
b3V0Y29tZS1jYXRlZ29yaWVzYCwgdGhlIGV4YWN0IDE2NzUtYnl0ZSBzb3VyY2Ugc3BhbiBhdCBg
MDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMxMi1maW5hbC1yZWNlaXB0LWltbXV0YWJsZS1wb2xp
Y3lzbmFwc2hvdHYzLWFjY2VwdGFuY2UtcnVsZXNgLCB0aGUgZXhhY3QgMTkzMC1ieXRlIHNvdXJj
ZSBzcGFuIGF0IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzEzLWRpc3BhdGNoLW5hcnJvd2lu
Zy1tb25vdG9uaWNpdHktcnVsZXNgLCBhbmQgdGhlIGV4YWN0IDExMDYtYnl0ZSBzb3VyY2Ugc3Bh
biBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMxNC1jb250cmFjdC1wcm9tb3Rpb24tZ2F0
ZXNgOyByZW1vdmUgYGNvbnRyYWN0cy9jYW5jZWwtb3V0Y29tZS1jYXRlZ29yaWVzLm1kYCwgYGdh
dGVzL2ZpbmFsLXJlY2VpcHQtaW1tdXRhYmxlLXBvbGljeS1zbmFwc2hvdC12My1hY2NlcHRhbmNl
Lm1kYCwgYGdhdGVzL2Rpc3BhdGNoLW5hcnJvd2luZy1tb25vdG9uaWNpdHkubWRgLCBhbmQgYGdh
dGVzL2NvbnRyYWN0LXByb21vdGlvbi5tZGA7IHJlbW92ZSB0aGUgZXhhY3QgYENhbmNlbCBvdXRj
b21lIGNhdGVnb3JpZXNgLCBgRmluYWwtcmVjZWlwdCBpbW11dGFibGUgUG9saWN5U25hcHNob3RW
MyBhY2NlcHRhbmNlIHJ1bGVzYCwgYERpc3BhdGNoIG5hcnJvd2luZyBtb25vdG9uaWNpdHkgcnVs
ZXNgLCBhbmQgYENvbnRyYWN0IHByb21vdGlvbiBnYXRlc2Agcm93cyBmcm9tIGBpbmRleC9SRUFE
TUUubWRgOyByZW1vdmUgYWxsIGZvdXIgRDEwIGxlZGdlciBlbnRyaWVzIGZvciB0aGlzIGJhdGNo
OyBsZWF2ZSBgTGF1bmNoVGltZVNlY3JldEhhbmRvZmZWMWAsIGBTdXBlcnZpc29yIGlkZW1wb3Rl
bmN5IGFuZCByZXN0YXJ0IHJ1bGVzYCwgYERpZmZlcmVudGlhbC1iYXNlbGluZSB0cmFuc2l0aW9u
IGdhdGVgLCBgUmVtYWluaW5nIFIyLTIgc2FtZS1wcm9jZXNzIGNhcnJpZXIgY2xvc3VyZWAsIGFs
bCBEMy9ENeKAk0QxMCBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYHJldmlldy1jb250cm9s
L2AsIGFuZCBldmVyeSBwYXRoIG91dHNpZGUgdGhlIHNldmVuLXBhdGggZmVuY2UgdW5jaGFuZ2Vk
IHwKK3wgRDEwIHwgW2AwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kYF0oLi4vMDQtY29udHJhY3Rz
LWFuZC1nYXRlcy5tZCkgfCAxMi4gRmluYWwtcmVjZWlwdCBpbW11dGFibGUgYFBvbGljeVNuYXBz
aG90VjNgIGFjY2VwdGFuY2UgcnVsZXMgfCBbYDEyLWZpbmFsLXJlY2VpcHQtaW1tdXRhYmxlLXBv
bGljeXNuYXBzaG90djMtYWNjZXB0YW5jZS1ydWxlc2BdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0
ZXMubWQjMTItZmluYWwtcmVjZWlwdC1pbW11dGFibGUtcG9saWN5c25hcHNob3R2My1hY2NlcHRh
bmNlLXJ1bGVzKSB8IGxpbmVzIDE1MuKAkzE4MCAoMTY3NSBieXRlcykgfCBgNjcyYzliYTNkYjFl
NzkyYTdhZmIxNDVhNDA4MTQ1MGRlYWZlNmExMTIzMjYyYWU3YzEwNGM2ZjFiNGIwYjI0YWAgfCBb
YC4uL2dhdGVzL2ZpbmFsLXJlY2VpcHQtaW1tdXRhYmxlLXBvbGljeS1zbmFwc2hvdC12My1hY2Nl
cHRhbmNlLm1kYF0oLi4vZ2F0ZXMvZmluYWwtcmVjZWlwdC1pbW11dGFibGUtcG9saWN5LXNuYXBz
aG90LXYzLWFjY2VwdGFuY2UubWQpIHwgZ2F0ZSB8IGNhbm9uaWNhbCBkZXN0aW5hdGlvbjsgc291
cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9yIHwgbm9uZSB8IHJlc3RvcmUgdGhlIGV4YWN0IDExNzkt
Ynl0ZSBzb3VyY2Ugc3BhbiBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMxMC1jYW5jZWwt
b3V0Y29tZS1jYXRlZ29yaWVzYCwgdGhlIGV4YWN0IDE2NzUtYnl0ZSBzb3VyY2Ugc3BhbiBhdCBg
MDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMxMi1maW5hbC1yZWNlaXB0LWltbXV0YWJsZS1wb2xp
Y3lzbmFwc2hvdHYzLWFjY2VwdGFuY2UtcnVsZXNgLCB0aGUgZXhhY3QgMTkzMC1ieXRlIHNvdXJj
ZSBzcGFuIGF0IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzEzLWRpc3BhdGNoLW5hcnJvd2lu
Zy1tb25vdG9uaWNpdHktcnVsZXNgLCBhbmQgdGhlIGV4YWN0IDExMDYtYnl0ZSBzb3VyY2Ugc3Bh
biBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMxNC1jb250cmFjdC1wcm9tb3Rpb24tZ2F0
ZXNgOyByZW1vdmUgYGNvbnRyYWN0cy9jYW5jZWwtb3V0Y29tZS1jYXRlZ29yaWVzLm1kYCwgYGdh
dGVzL2ZpbmFsLXJlY2VpcHQtaW1tdXRhYmxlLXBvbGljeS1zbmFwc2hvdC12My1hY2NlcHRhbmNl
Lm1kYCwgYGdhdGVzL2Rpc3BhdGNoLW5hcnJvd2luZy1tb25vdG9uaWNpdHkubWRgLCBhbmQgYGdh
dGVzL2NvbnRyYWN0LXByb21vdGlvbi5tZGA7IHJlbW92ZSB0aGUgZXhhY3QgYENhbmNlbCBvdXRj
b21lIGNhdGVnb3JpZXNgLCBgRmluYWwtcmVjZWlwdCBpbW11dGFibGUgUG9saWN5U25hcHNob3RW
MyBhY2NlcHRhbmNlIHJ1bGVzYCwgYERpc3BhdGNoIG5hcnJvd2luZyBtb25vdG9uaWNpdHkgcnVs
ZXNgLCBhbmQgYENvbnRyYWN0IHByb21vdGlvbiBnYXRlc2Agcm93cyBmcm9tIGBpbmRleC9SRUFE
TUUubWRgOyByZW1vdmUgYWxsIGZvdXIgRDEwIGxlZGdlciBlbnRyaWVzIGZvciB0aGlzIGJhdGNo
OyBsZWF2ZSBgTGF1bmNoVGltZVNlY3JldEhhbmRvZmZWMWAsIGBTdXBlcnZpc29yIGlkZW1wb3Rl
bmN5IGFuZCByZXN0YXJ0IHJ1bGVzYCwgYERpZmZlcmVudGlhbC1iYXNlbGluZSB0cmFuc2l0aW9u
IGdhdGVgLCBgUmVtYWluaW5nIFIyLTIgc2FtZS1wcm9jZXNzIGNhcnJpZXIgY2xvc3VyZWAsIGFs
bCBEMy9ENeKAk0QxMCBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYHJldmlldy1jb250cm9s
L2AsIGFuZCBldmVyeSBwYXRoIG91dHNpZGUgdGhlIHNldmVuLXBhdGggZmVuY2UgdW5jaGFuZ2Vk
IHwKK3wgRDEwIHwgW2AwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kYF0oLi4vMDQtY29udHJhY3Rz
LWFuZC1nYXRlcy5tZCkgfCAxMy4gRGlzcGF0Y2ggbmFycm93aW5nIG1vbm90b25pY2l0eSBydWxl
cyB8IFtgMTMtZGlzcGF0Y2gtbmFycm93aW5nLW1vbm90b25pY2l0eS1ydWxlc2BdKC4uLzA0LWNv
bnRyYWN0cy1hbmQtZ2F0ZXMubWQjMTMtZGlzcGF0Y2gtbmFycm93aW5nLW1vbm90b25pY2l0eS1y
dWxlcykgfCBsaW5lcyAxODHigJMyMTQgKDE5MzAgYnl0ZXMpIHwgYGIxMmVmMmI4YzhiY2ZmYWQ1
MTM5ZTE5NmRmNDliYTJmNTkwOTRkZGYwMjRmNTlkMmQyYTY0NWViOWJlMjIxMGFgIHwgW2AuLi9n
YXRlcy9kaXNwYXRjaC1uYXJyb3dpbmctbW9ub3RvbmljaXR5Lm1kYF0oLi4vZ2F0ZXMvZGlzcGF0
Y2gtbmFycm93aW5nLW1vbm90b25pY2l0eS5tZCkgfCBnYXRlIHwgY2Fub25pY2FsIGRlc3RpbmF0
aW9uOyBzb3VyY2UgY29tcGF0aWJpbGl0eSBhbmNob3IgfCBub25lIHwgcmVzdG9yZSB0aGUgZXhh
Y3QgMTE3OS1ieXRlIHNvdXJjZSBzcGFuIGF0IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzEw
LWNhbmNlbC1vdXRjb21lLWNhdGVnb3JpZXNgLCB0aGUgZXhhY3QgMTY3NS1ieXRlIHNvdXJjZSBz
cGFuIGF0IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzEyLWZpbmFsLXJlY2VpcHQtaW1tdXRh
YmxlLXBvbGljeXNuYXBzaG90djMtYWNjZXB0YW5jZS1ydWxlc2AsIHRoZSBleGFjdCAxOTMwLWJ5
dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMTMtZGlzcGF0Y2gt
bmFycm93aW5nLW1vbm90b25pY2l0eS1ydWxlc2AsIGFuZCB0aGUgZXhhY3QgMTEwNi1ieXRlIHNv
dXJjZSBzcGFuIGF0IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzE0LWNvbnRyYWN0LXByb21v
dGlvbi1nYXRlc2A7IHJlbW92ZSBgY29udHJhY3RzL2NhbmNlbC1vdXRjb21lLWNhdGVnb3JpZXMu
bWRgLCBgZ2F0ZXMvZmluYWwtcmVjZWlwdC1pbW11dGFibGUtcG9saWN5LXNuYXBzaG90LXYzLWFj
Y2VwdGFuY2UubWRgLCBgZ2F0ZXMvZGlzcGF0Y2gtbmFycm93aW5nLW1vbm90b25pY2l0eS5tZGAs
IGFuZCBgZ2F0ZXMvY29udHJhY3QtcHJvbW90aW9uLm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgQ2Fu
Y2VsIG91dGNvbWUgY2F0ZWdvcmllc2AsIGBGaW5hbC1yZWNlaXB0IGltbXV0YWJsZSBQb2xpY3lT
bmFwc2hvdFYzIGFjY2VwdGFuY2UgcnVsZXNgLCBgRGlzcGF0Y2ggbmFycm93aW5nIG1vbm90b25p
Y2l0eSBydWxlc2AsIGFuZCBgQ29udHJhY3QgcHJvbW90aW9uIGdhdGVzYCByb3dzIGZyb20gYGlu
ZGV4L1JFQURNRS5tZGA7IHJlbW92ZSBhbGwgZm91ciBEMTAgbGVkZ2VyIGVudHJpZXMgZm9yIHRo
aXMgYmF0Y2g7IGxlYXZlIGBMYXVuY2hUaW1lU2VjcmV0SGFuZG9mZlYxYCwgYFN1cGVydmlzb3Ig
aWRlbXBvdGVuY3kgYW5kIHJlc3RhcnQgcnVsZXNgLCBgRGlmZmVyZW50aWFsLWJhc2VsaW5lIHRy
YW5zaXRpb24gZ2F0ZWAsIGBSZW1haW5pbmcgUjItMiBzYW1lLXByb2Nlc3MgY2FycmllciBjbG9z
dXJlYCwgYWxsIEQzL0Q14oCTRDEwIG93bmVycywgYGluZGV4L2N1cnJlbnQubWRgLCBgcmV2aWV3
LWNvbnRyb2wvYCwgYW5kIGV2ZXJ5IHBhdGggb3V0c2lkZSB0aGUgc2V2ZW4tcGF0aCBmZW5jZSB1
bmNoYW5nZWQgfAorfCBEMTAgfCBbYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWRgXSguLi8wNC1j
b250cmFjdHMtYW5kLWdhdGVzLm1kKSB8IDE0LiBDb250cmFjdCBwcm9tb3Rpb24gZ2F0ZXMgfCBb
YDE0LWNvbnRyYWN0LXByb21vdGlvbi1nYXRlc2BdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMu
bWQjMTQtY29udHJhY3QtcHJvbW90aW9uLWdhdGVzKSB8IGxpbmVzIDIxNeKAkzIzMSAoMTEwNiBi
eXRlcykgfCBgMGZhNzc3NzdiN2Y4YWU4NWJjNGUwNjA4MjQ1NGIyNDM2NzdkODlkMDcwZmY0YzUx
MGJkOWUxNDhiNjFiMmQ0M2AgfCBbYC4uL2dhdGVzL2NvbnRyYWN0LXByb21vdGlvbi5tZGBdKC4u
L2dhdGVzL2NvbnRyYWN0LXByb21vdGlvbi5tZCkgfCBnYXRlIHwgY2Fub25pY2FsIGRlc3RpbmF0
aW9uOyBzb3VyY2UgY29tcGF0aWJpbGl0eSBhbmNob3IgfCBub25lIHwgcmVzdG9yZSB0aGUgZXhh
Y3QgMTE3OS1ieXRlIHNvdXJjZSBzcGFuIGF0IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzEw
LWNhbmNlbC1vdXRjb21lLWNhdGVnb3JpZXNgLCB0aGUgZXhhY3QgMTY3NS1ieXRlIHNvdXJjZSBz
cGFuIGF0IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzEyLWZpbmFsLXJlY2VpcHQtaW1tdXRh
YmxlLXBvbGljeXNuYXBzaG90djMtYWNjZXB0YW5jZS1ydWxlc2AsIHRoZSBleGFjdCAxOTMwLWJ5
dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMTMtZGlzcGF0Y2gt
bmFycm93aW5nLW1vbm90b25pY2l0eS1ydWxlc2AsIGFuZCB0aGUgZXhhY3QgMTEwNi1ieXRlIHNv
dXJjZSBzcGFuIGF0IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzE0LWNvbnRyYWN0LXByb21v
dGlvbi1nYXRlc2A7IHJlbW92ZSBgY29udHJhY3RzL2NhbmNlbC1vdXRjb21lLWNhdGVnb3JpZXMu
bWRgLCBgZ2F0ZXMvZmluYWwtcmVjZWlwdC1pbW11dGFibGUtcG9saWN5LXNuYXBzaG90LXYzLWFj
Y2VwdGFuY2UubWRgLCBgZ2F0ZXMvZGlzcGF0Y2gtbmFycm93aW5nLW1vbm90b25pY2l0eS5tZGAs
IGFuZCBgZ2F0ZXMvY29udHJhY3QtcHJvbW90aW9uLm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgQ2Fu
Y2VsIG91dGNvbWUgY2F0ZWdvcmllc2AsIGBGaW5hbC1yZWNlaXB0IGltbXV0YWJsZSBQb2xpY3lT
bmFwc2hvdFYzIGFjY2VwdGFuY2UgcnVsZXNgLCBgRGlzcGF0Y2ggbmFycm93aW5nIG1vbm90b25p
Y2l0eSBydWxlc2AsIGFuZCBgQ29udHJhY3QgcHJvbW90aW9uIGdhdGVzYCByb3dzIGZyb20gYGlu
ZGV4L1JFQURNRS5tZGA7IHJlbW92ZSBhbGwgZm91ciBEMTAgbGVkZ2VyIGVudHJpZXMgZm9yIHRo
aXMgYmF0Y2g7IGxlYXZlIGBMYXVuY2hUaW1lU2VjcmV0SGFuZG9mZlYxYCwgYFN1cGVydmlzb3Ig
aWRlbXBvdGVuY3kgYW5kIHJlc3RhcnQgcnVsZXNgLCBgRGlmZmVyZW50aWFsLWJhc2VsaW5lIHRy
YW5zaXRpb24gZ2F0ZWAsIGBSZW1haW5pbmcgUjItMiBzYW1lLXByb2Nlc3MgY2FycmllciBjbG9z
dXJlYCwgYWxsIEQzL0Q14oCTRDEwIG93bmVycywgYGluZGV4L2N1cnJlbnQubWRgLCBgcmV2aWV3
LWNvbnRyb2wvYCwgYW5kIGV2ZXJ5IHBhdGggb3V0c2lkZSB0aGUgc2V2ZW4tcGF0aCBmZW5jZSB1
bmNoYW5nZWQgfApkaWZmIC0tZ2l0IGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2Nv
bnRyYWN0cy9jYW5jZWwtb3V0Y29tZS1jYXRlZ29yaWVzLm1kIGIvbGxtLWxhc3QtbWlsZS9ydW50
aW1lLXJlZmFjdG9yL2NvbnRyYWN0cy9jYW5jZWwtb3V0Y29tZS1jYXRlZ29yaWVzLm1kCm5ldyBm
aWxlIG1vZGUgMTAwNjQ0CmluZGV4IDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAw
MDAwMDAuLmNkZWJkMTk3YjM3NWE2ZDk5YTg5ZDA2NzZkYjQyMjU0ODYwMmRhZDIKLS0tIC9kZXYv
bnVsbAorKysgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvY29udHJhY3RzL2NhbmNl
bC1vdXRjb21lLWNhdGVnb3JpZXMubWQKQEAgLTAsMCArMSwzMyBAQAorKipLaW5kOioqIGNvbnRy
YWN0CisqKlN0YXR1czoqKiBjYW5vbmljYWwKKyoqQ2Fub25pY2FsIGZvcjoqKiBjb21wbGV0ZSBl
eHRyYWN0ZWQgYENhbmNlbFdvcmxkV29ya091dGNvbWVWMWAgZW51bSBhbmQgcnVsZXMgMeKAkzYg
Y292ZXJpbmcgZXhhY3QgaWRlbnRpdHkgYW5kIHdvcmxkLWJpbmRpbmcgcmVzb2x1dGlvbiwgYE5v
QWN0aXZlQ2FuY2VsYWJsZVdvcmtgIHZlcnN1cyBgT3duZXJVbnJlYWNoYWJsZWAsIHBlbmRpbmcg
Y2xvc2VvdXQsIHRlcm1pbmFsIG1vbm90b25pY2l0eSwgYW5kIHN0b3AtdmVyc3VzLWNhbmNlbCBu
b24tYWxpYXNpbmcKKyoqU291cmNlIHByb3ZlbmFuY2U6KiogZXh0cmFjdGVkIGJ5dGUtZm9yLWJ5
dGUgZnJvbSBbYC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMTAtY2FuY2VsLW91dGNvbWUt
Y2F0ZWdvcmllc2BdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMTAtY2FuY2VsLW91dGNv
bWUtY2F0ZWdvcmllcyksIGJhc2VsaW5lIGxpbmVzIDExOeKAkzE0MzsgdGhlIGV4YWN0IDExNzkt
Ynl0ZSBzb3VyY2UgYm9keSBpcyBwcmVzZXJ2ZWQgYmV0d2VlbiB0aGUgYm91bmRhcnkgbWFya2Vy
cyBiZWxvdworKipCYXNlbGluZSBzcGFuIFNIQS0yNTY6KiogYGU4ZjM3M2M5ZTdjYmYxNmQyODZm
ZjhmOWE5MTQ2MzA2NjQ0ZTQzMGUzZDYzMGI3MDc4ZWM3NTAxYjcyZWQ0YjNgCisKKzwhLS0gZXhh
Y3QtZXh0cmFjdGVkLWJvZHk6c3RhcnQgLS0+CisjIyAxMC4gQ2FuY2VsIG91dGNvbWUgY2F0ZWdv
cmllcworCitgYGBydXN0CitlbnVtIENhbmNlbFdvcmxkV29ya091dGNvbWVWMSB7CisgICAgQ2Fu
Y2VsbGVkVmlhTGl2ZVRyYW5zcG9ydCB7IGFjdGl2ZV9ydW5faWQsIHRlcm1pbmFsX3JlZiB9LAor
ICAgIENhbmNlbEFjY2VwdGVkUGVuZGluZ0Nsb3Nlb3V0IHsgYWN0aXZlX3J1bl9pZCwgY2FuY2Vs
X3JlcXVlc3RfaWQgfSwKKyAgICBBbHJlYWR5VGVybWluYWwgeyBhY3RpdmVfcnVuX2lkLCB0ZXJt
aW5hbF9yZWYgfSwKKyAgICBOb0FjdGl2ZUNhbmNlbGFibGVXb3JrIHsgd29ya2VyX29yX3Rhc2tf
cmVmIH0sCisgICAgT3duZXJVbnJlYWNoYWJsZSB7IGFjdGl2ZV9ydW5faWQsIGR1cmFibGVfc3Rh
dGUsIHJldHJ5YWJpbGl0eSB9LAorICAgIEludmFsaWRUYXJnZXQgeyByZWFzb24gfSwKKyAgICBX
b3JsZEJpbmRpbmdNaXNtYXRjaCB7IGV4cGVjdGVkLCBhY3R1YWwgfSwKKyAgICBBbWJpZ3VvdXNU
YXJnZXQgeyBjYW5kaWRhdGVzIH0sCisgICAgUG9saWN5RGVuaWVkIHsgZGVuaWFsX2NvZGUsIGV4
cGxhbmF0aW9uIH0sCit9CitgYGAKKworUnVsZXM6CisKKzEuIEV4YWN0IGlkZW50aXR5IGFuZCB3
b3JsZCBiaW5kaW5nIHJlc29sdmUgYmVmb3JlIHRyYW5zcG9ydCB1c2UuCisyLiBgTm9BY3RpdmVD
YW5jZWxhYmxlV29ya2AgbWVhbnMgdmFsaWQgcm91dGluZyBjb250ZXh0IGJ1dCBubyBhY2NlcHRl
ZCBub24tdGVybWluYWwgY2FuY2VsYWJsZSByZWNlaXB0LgorMy4gYE93bmVyVW5yZWFjaGFibGVg
IG1lYW5zIGEgdmFsaWQgYWN0aXZlIHJlY2VpcHQgZXhpc3RzIGJ1dCB0aGUgbGl2ZSBjYW5jZWxs
YXRpb24gcm91dGUgaXMgdW5hdmFpbGFibGUgYW5kIGR1cmFibGUgcG9saWN5IGNhbm5vdCB5ZXQg
ZGVjbGFyZSBjbG9zZW91dC4KKzQuIGBDYW5jZWxBY2NlcHRlZFBlbmRpbmdDbG9zZW91dGAgaXMg
bm90IHRlcm1pbmFsIHN1Y2Nlc3M7IGluc3BlY3QgcmVtYWlucyBhYmxlIHRvIG9ic2VydmUgZXZl
bnR1YWwgY2xvc2VvdXQuCis1LiBSZXBlYXRlZCBjYW5jZWwgYWZ0ZXIgdGVybWluYWwgcmV0dXJu
cyBgQWxyZWFkeVRlcm1pbmFsYDsgaXQgbmV2ZXIgcmVncmVzc2VzIHRoZSByZWNlaXB0LgorNi4g
U3RvcCB0YXJnZXRzIHdvcmtlciBsaWZlY3ljbGUuIENhbmNlbCB0YXJnZXRzIG9uZSBhY3RpdmUg
dGFzay90dXJuLiBUaGV5IGFyZSBub3QgYWxpYXNlcy4KKworPCEtLSBleGFjdC1leHRyYWN0ZWQt
Ym9keTplbmQgLS0+CmRpZmYgLS1naXQgYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3Iv
Z2F0ZXMvZmluYWwtcmVjZWlwdC1pbW11dGFibGUtcG9saWN5LXNuYXBzaG90LXYzLWFjY2VwdGFu
Y2UubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvZ2F0ZXMvZmluYWwtcmVjZWlw
dC1pbW11dGFibGUtcG9saWN5LXNuYXBzaG90LXYzLWFjY2VwdGFuY2UubWQKbmV3IGZpbGUgbW9k
ZSAxMDA2NDQKaW5kZXggMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMC4u
MzM0MDQ1NTdjNzE4YzNmZWViZWYwYTU0M2FiZTZlNDQwNjc2MjA5NQotLS0gL2Rldi9udWxsCisr
KyBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9nYXRlcy9maW5hbC1yZWNlaXB0LWlt
bXV0YWJsZS1wb2xpY3ktc25hcHNob3QtdjMtYWNjZXB0YW5jZS5tZApAQCAtMCwwICsxLDM3IEBA
CisqKktpbmQ6KiogZ2F0ZQorKipTdGF0dXM6KiogY2Fub25pY2FsCisqKkNhbm9uaWNhbCBmb3I6
KiogY29tcGxldGUgZXh0cmFjdGVkIEIxIHByZS1FMiB2ZXJzdXMgRTIvQjIuMiBhY2NlcHRhbmNl
IGJvdW5kYXJ5LCBvcmRlcmVkIGNvbmRpdGlvbnMgMeKAkzExLCBpbW11dGFibGUgYFBvbGljeVNu
YXBzaG90VjNgIHJlY2VpcHQgZXhwb3N1cmUgcnVsZSwgYW5kIHRoZSBwb3N0LWFjY2VwdGFuY2Ug
aW1tdXRhYmxlLCBmdXR1cmUtb25seSwgcmV2b2NhdGlvbiwgYW5kIGZhaWwtY2xvc2VkIHJlcXVp
cmVtZW50cworKipTb3VyY2UgcHJvdmVuYW5jZToqKiBleHRyYWN0ZWQgYnl0ZS1mb3ItYnl0ZSBm
cm9tIFtgLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMxMi1maW5hbC1yZWNlaXB0LWltbXV0
YWJsZS1wb2xpY3lzbmFwc2hvdHYzLWFjY2VwdGFuY2UtcnVsZXNgXSguLi8wNC1jb250cmFjdHMt
YW5kLWdhdGVzLm1kIzEyLWZpbmFsLXJlY2VpcHQtaW1tdXRhYmxlLXBvbGljeXNuYXBzaG90djMt
YWNjZXB0YW5jZS1ydWxlcyksIGJhc2VsaW5lIGxpbmVzIDE1MuKAkzE4MDsgdGhlIGV4YWN0IDE2
NzUtYnl0ZSBzb3VyY2UgYm9keSBpcyBwcmVzZXJ2ZWQgYmV0d2VlbiB0aGUgYm91bmRhcnkgbWFy
a2VycyBiZWxvdworKipCYXNlbGluZSBzcGFuIFNIQS0yNTY6KiogYDY3MmM5YmEzZGIxZTc5MmE3
YWZiMTQ1YTQwODE0NTBkZWFmZTZhMTEyMzI2MmFlN2MxMDRjNmYxYjRiMGIyNGFgCisKKzwhLS0g
ZXhhY3QtZXh0cmFjdGVkLWJvZHk6c3RhcnQgLS0+CisjIyAxMi4gRmluYWwtcmVjZWlwdCBpbW11
dGFibGUgYFBvbGljeVNuYXBzaG90VjNgIGFjY2VwdGFuY2UgcnVsZXMKKworQjEncyBwcmUtRTIg
YWNjZXB0YW5jZSBhbmNob3IgcmVjb3JkcyB0aGUgZXhhY3QgY3VycmVudCBwb2xpY3kgaWRlbnRp
dHkgdXNlZCBieSB0aGUgcnVudGltZSBidXQgaXMKK25vdCBhIGZpbmFsIHJlY2VpcHQgYW5kIGlz
IG5vdCBtb2RlbC1mYWNpbmcuIEUyIG93bnMgdGhlIGltbXV0YWJsZSBhY3RpdmUtcnVuIHNuYXBz
aG90IGFuZCByZXRhaW5lZAord29ya2VyIGNhcCBiZWxvdzsgQjIuMiBtYXkgZXhwb3NlIGEgcmVj
ZWlwdCBvbmx5IGFmdGVyIHRob3NlIGNvbW1pdG1lbnRzIGFuZCB0aGUgQjIuMSBvYnNlcnZhdGlv
bgorY2xhaW0gYXJlIGR1cmFibGUgYW5kIGxpbmtlZCB0byB0aGUgQjEgcmVjb3JkLgorCitBIGZp
bmFsIGFjdGl2ZSB0YXNrL3R1cm4gcmVjZWlwdCBtYXkgYmUgZXhwb3NlZCBvbmx5IHdoZW4gYWxs
IGFyZSB0cnVlOgorCisxLiBleGFjdCBzZXNzaW9uLCBjYWxsZXIsIGJhY2tlbmQsIGFuZCB3b3Js
ZCBiaW5kaW5nIGFyZSByZXNvbHZlZDsKKzIuIHN0ZWVyaW5nIHBvbGljeSBhbGxvd3MgdGhlIHZl
cmIvbW9kZS90YXJnZXQ7CiszLiBjdXJyZW50IHBhcmVudCBwb2xpY3kgaXMgcmVzb2x2ZWQgYXQg
YSBrbm93biByZXZpc2lvbjsKKzQuIHJldGFpbmVkIHdvcmtlciBjYXAgaXMgbG9hZGVkIGFuZCBo
YXNoLXZlcmlmaWVkIHdoZW4gYXBwbGljYWJsZTsKKzUuIG9wdGlvbmFsIG5hcnJvd2luZyBpcyB2
YWxpZGF0ZWQgYXMgbW9ub3RvbmljOworNi4gdGhlIHJlc3VsdGluZyBgUG9saWN5U25hcHNob3RW
M2AgY2Fub25pY2FsaXplcyBhbmQgcGFzc2VzIGV4aXN0aW5nIHNjaGVtYS9lbmZvcmNlbWVudCB2
YWxpZGF0aW9uOworNy4gc25hcHNob3QgYnl0ZXMvcmVmL2hhc2gvcmV2aXNpb24gYXJlIGR1cmFi
bGU7Cis4LiB0aGUgZXhlY3V0aW9uIGVudmVsb3BlL3dvcmxkLXNlcnZpY2UgcmVxdWVzdCBjYXJy
aWVzIHRoZSBzYW1lIHZlcmlmaWVkIHNuYXBzaG90OworOS4gcnVudGltZSBhY2NlcHRhbmNlIGV2
aWRlbmNlIGpvaW5zIHRoZSBhY2tub3dsZWRnZW1lbnQgdG8gdGhlIGV4YWN0IHdvcmsgaWRlbnRp
dHk7CisxMC4gdGhlIG9ic2VydmF0aW9uIGNsYWltIGlzIGR1cmFibGUgYW5kIHJlc3VtYWJsZTsg
YW5kCisxMS4gdGhlIHJlY2VpcHQgcmVmZXJlbmNlcyB0aGUgc25hcHNob3QsIGFjY2VwdGFuY2Ug
ZXZpZGVuY2UsIGFuZCBvYnNlcnZhdGlvbiBjbGFpbSBiZWZvcmUgdGhlIGZvcmVncm91bmQgY2Fs
bGVyIGlzIHRvbGQgdGhlIHdvcmsgd2FzIGFjY2VwdGVkLgorCitBZnRlciBhY2NlcHRhbmNlOgor
CistIHRoZSByZWNlaXB0J3MgcG9saWN5IHJlZi9oYXNoIGlzIGltbXV0YWJsZTsKKy0gcGFyZW50
IGJyb2FkZW5pbmcgb3IgbmFycm93aW5nIGRvZXMgbm90IHJld3JpdGUgdGhlIGFjdGl2ZSByZWNl
aXB0OworLSBwYXJlbnQgY2hhbmdlcyBhcHBseSB0byBmdXR1cmUgdGFzayBhY2NlcHRhbmNlLCBj
b250aW51ZSwgZm9yaywgb3Igd29ya2VyIHR1cm5zOworLSBlbWVyZ2VuY3kgcmV2b2NhdGlvbiBp
cyBhbiBleHBsaWNpdCBhdWRpdGVkIGNhbmNlbC9yZXZva2UgcGF0aCwgbmV2ZXIgc2lsZW50IHNu
YXBzaG90IG11dGF0aW9uOyBhbmQKKy0gc25hcHNob3QgbWlzbWF0Y2ggYXQgYnJva2VyL3dvcmxk
LXNlcnZpY2UgZmFpbHMgY2xvc2VkLgorCis8IS0tIGV4YWN0LWV4dHJhY3RlZC1ib2R5OmVuZCAt
LT4KZGlmZiAtLWdpdCBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9nYXRlcy9kaXNw
YXRjaC1uYXJyb3dpbmctbW9ub3RvbmljaXR5Lm1kIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJl
ZmFjdG9yL2dhdGVzL2Rpc3BhdGNoLW5hcnJvd2luZy1tb25vdG9uaWNpdHkubWQKbmV3IGZpbGUg
bW9kZSAxMDA2NDQKaW5kZXggMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAw
MC4uZmQyYjc0NTU0YjQ2YWZkYmVkNTU5ZjYyYTMyNTc3YzYxN2I4NWVlOAotLS0gL2Rldi9udWxs
CisrKyBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9nYXRlcy9kaXNwYXRjaC1uYXJy
b3dpbmctbW9ub3RvbmljaXR5Lm1kCkBAIC0wLDAgKzEsNDIgQEAKKyoqS2luZDoqKiBnYXRlCisq
KlN0YXR1czoqKiBjYW5vbmljYWwKKyoqQ2Fub25pY2FsIGZvcjoqKiBjb21wbGV0ZSBleHRyYWN0
ZWQgcmVzb2x2ZXIgZm9ybXVsYXMsIFYxIGZpZWxkIHJ1bGVzIHRhYmxlLCBwYXRoLWNvbnRhaW5t
ZW50IHJ1bGVzIDHigJM4LCBob3N0LXZpc2libGUgZmFpbC1jbG9zZWQgZXhjZXB0aW9uLCByZWpl
Y3RlZC1ub3QtaWdub3JlZCBuYXJyb3dpbmcgcnVsZSwgYW5kIGJyb2tlciB2ZXJzdXMgd29ybGQt
c2VydmljZSBlbmZvcmNlbWVudCBib3VuZGFyeQorKipTb3VyY2UgcHJvdmVuYW5jZToqKiBleHRy
YWN0ZWQgYnl0ZS1mb3ItYnl0ZSBmcm9tIFtgLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMx
My1kaXNwYXRjaC1uYXJyb3dpbmctbW9ub3RvbmljaXR5LXJ1bGVzYF0oLi4vMDQtY29udHJhY3Rz
LWFuZC1nYXRlcy5tZCMxMy1kaXNwYXRjaC1uYXJyb3dpbmctbW9ub3RvbmljaXR5LXJ1bGVzKSwg
YmFzZWxpbmUgbGluZXMgMTgx4oCTMjE0OyB0aGUgZXhhY3QgMTkzMC1ieXRlIHNvdXJjZSBib2R5
IGlzIHByZXNlcnZlZCBiZXR3ZWVuIHRoZSBib3VuZGFyeSBtYXJrZXJzIGJlbG93CisqKkJhc2Vs
aW5lIHNwYW4gU0hBLTI1NjoqKiBgYjEyZWYyYjhjOGJjZmZhZDUxMzllMTk2ZGY0OWJhMmY1OTA5
NGRkZjAyNGY1OWQyZDJhNjQ1ZWI5YmUyMjEwYWAKKworPCEtLSBleGFjdC1leHRyYWN0ZWQtYm9k
eTpzdGFydCAtLT4KKyMjIDEzLiBEaXNwYXRjaCBuYXJyb3dpbmcgbW9ub3RvbmljaXR5IHJ1bGVz
CisKK1RoZSByZXNvbHZlciBjb21wdXRlczoKKworYGBgdGV4dAorZXBoZW1lcmFsID0gY3VycmVu
dF9wYXJlbnQgQU5EIGRpc3BhdGNoX3BhdGNoCit3b3JrZXJfY2FwID0gY3VycmVudF9wYXJlbnRf
YXRfc3Bhd24gQU5EIHNwYXduX3BhdGNoCit0dXJuID0gY3VycmVudF9wYXJlbnRfbm93IEFORCB3
b3JrZXJfY2FwIEFORCB0dXJuX3BhdGNoCitmb3JrX2NhcCA9IGN1cnJlbnRfcGFyZW50X25vdyBB
TkQgc291cmNlX3dvcmtlcl9jYXAgQU5EIGZvcmtfcGF0Y2gKK2BgYAorCitWMSBmaWVsZCBydWxl
czoKKworfCBGaWVsZCB8IEFsbG93ZWQgbmFycm93aW5nIHwgUmVqZWN0ZWQgYnJvYWRlbmluZyB8
Cit8LS0tfC0tLXwtLS18Cit8IGB3b3JsZF9mcy5ob3N0X3Zpc2libGVgIHwgYHRydWUgLT4gZmFs
c2VgIHwgYGZhbHNlIC0+IHRydWVgIHwKK3wgYHdvcmxkX2ZzLmZhaWxfY2xvc2VkLnJvdXRpbmdg
IHwgYGZhbHNlIC0+IHRydWVgIHwgYHRydWUgLT4gZmFsc2VgIHwKK3wgYHdvcmxkX2ZzLmNhZ2Vk
X3JlcXVpcmVkYCB8IGBmYWxzZSAtPiB0cnVlYCB8IGB0cnVlIC0+IGZhbHNlYCB8Cit8IGB3b3Js
ZF9mcy53cml0ZS5lbmFibGVkYCB8IGB0cnVlIC0+IGZhbHNlYCB8IGBmYWxzZSAtPiB0cnVlYCB8
Cit8IGB3b3JsZF9mcy5kZW55X2VuZm9yY2VtZW50YCB8IHNhbWUgb3Igc3Ryb25nZXIgcmFuayB8
IHdlYWtlciByYW5rIG9yIHJlbW92YWwgfAorfCBkaXNjb3Zlci9yZWFkL3dyaXRlIGBhbGxvd19s
aXN0YCB8IGVhY2ggcmVxdWVzdGVkIHBhdGggaXMgY29udGFpbmVkIGJ5IGF0IGxlYXN0IG9uZSBw
YXJlbnQgcGF0aCB8IHJvb3QvZGlyZWN0b3J5L2ZpbGUgd2lkZW5pbmcgfAorfCBkaXNjb3Zlci9y
ZWFkL3dyaXRlIGBkZW55X2xpc3RgIHwgYWRkIGRlbmlhbHMgb3IgcmV0YWluIHBhcmVudCBkZW5p
YWxzIHwgcmVtb3ZlIHBhcmVudCBkZW5pYWwgfAorCitQYXRoLWNvbnRhaW5tZW50IHJ1bGVzOgor
CisxLiBOb3JtYWxpemUgcmVsYXRpdmUgdG8gdGhlIGF1dGhvcml0YXRpdmUgd29ybGQvcHJvamVj
dCByb290LgorMi4gYC5gIGNvbnRhaW5zIGBzcmNgIGFuZCBgc3JjL3BhcnNlci5yc2A7IGBzcmNg
IGNvbnRhaW5zIGBzcmMvcGFyc2VyLnJzYDsgYSBmaWxlIGNvbnRhaW5zIG9ubHkgaXRzZWxmLgor
My4gUmVqZWN0IGFic29sdXRlIGhvc3QgcGF0aHMsIGAuLmAgZXNjYXBlLCB1bnN1cHBvcnRlZCBn
bG9iIHNlbWFudGljcywgTlVMLCBzeW1saW5rIGVzY2FwZSwgYW5kIHBhdGhzIG91dHNpZGUgdGhl
IHJvb3QuCis0LiBDb21wYXJlIGNhbm9uaWNhbCBwb2xpY3kgcGF0aHMgd2l0aG91dCByZXF1aXJp
bmcgdGhlIHRhcmdldCBmaWxlIHRvIGFscmVhZHkgZXhpc3Q7IHJ1bnRpbWUgcmVzb2x1dGlvbiBt
dXN0IHJlLWNoZWNrIHN5bWxpbmsvYW5jZXN0b3IgZXNjYXBlIGF0IGVuZm9yY2VtZW50IHRpbWUu
Cis1LiBgaG9zdF92aXNpYmxlPXRydWVgIHdpdGggZGVueS1saXN0IG9yIHVucHJvdmFibGUgaXNv
bGF0aW9uIHNlbWFudGljcyBmYWlscyBjbG9zZWQgdW5sZXNzIHRoZSBwYXRjaCBsZWdhbGx5IG5h
cnJvd3MgdG8gc3VwcG9ydGVkIGZ1bGwgaXNvbGF0aW9uLgorNi4gQSBwYXRjaCBzdXBwbGllZCB3
aGlsZSBgYWdlbnRzLndvcmxkX2Rpc3BhdGNoLmFsbG93X2NhcGFiaWxpdHlfbmFycm93aW5nPWZh
bHNlYCBpcyByZWplY3RlZCwgbm90IGlnbm9yZWQuCis3LiBOYXJyb3dpbmcgbWF5IG5vdCBlbmFi
bGUgYSBkaXNwYXRjaCBhY3Rpb24sIGJhY2tlbmQsIG1vZGUsIGNhcGFiaWxpdHksIG5ldHdvcmsg
cm91dGUsIG9yIHNpZGUtZWZmZWN0IGNoYW5uZWwgZm9yYmlkZGVuIGJ5IHRoZSBwYXJlbnQuCis4
LiBBZGFwdGVyIGNvbmZpZyBtYXkgcmVjZWl2ZSBwb2xpY3kgaGludHMsIGJ1dCBvbmx5IGJyb2tl
ci93b3JsZC1zZXJ2aWNlIGVuZm9yY2VtZW50IGNvdW50cy4KKworPCEtLSBleGFjdC1leHRyYWN0
ZWQtYm9keTplbmQgLS0+CmRpZmYgLS1naXQgYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0
b3IvZ2F0ZXMvY29udHJhY3QtcHJvbW90aW9uLm1kIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJl
ZmFjdG9yL2dhdGVzL2NvbnRyYWN0LXByb21vdGlvbi5tZApuZXcgZmlsZSBtb2RlIDEwMDY0NApp
bmRleCAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwLi5iN2M3NDU0NWI1
YTU0YmM2NTM2ZjMyZmI5MjFiZDdhMmM1ZGQ2ZTI1Ci0tLSAvZGV2L251bGwKKysrIGIvbGxtLWxh
c3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2dhdGVzL2NvbnRyYWN0LXByb21vdGlvbi5tZApAQCAt
MCwwICsxLDI1IEBACisqKktpbmQ6KiogZ2F0ZQorKipTdGF0dXM6KiogY2Fub25pY2FsCisqKkNh
bm9uaWNhbCBmb3I6KiogY29tcGxldGUgZXh0cmFjdGVkIHRlc3RzLXByb3ZlLWJlZm9yZS1sYW5k
ZWQgcHJvbW90aW9uIGdhdGUgYW5kIG9yZGVyZWQgcHJvb2YgcmVxdWlyZW1lbnRzIDHigJMxMCBj
b3ZlcmluZyBzZXJpYWxpemF0aW9uLCBwZXJzaXN0ZW5jZSwgcmVhbC1wYXRoIHVzYWdlLCByZXN0
YXJ0IGFuZCByZXBsYXksIHNtb2tlIGFuZCBlMmUgam9pbnMsIG9uZS10aW1lIGdhdGV3YXkgaGFu
ZG9mZiBwcm9vZiwgY29tcGF0aWJpbGl0eSBleGNsdXNpb24sIGFuZCBCMS9CMi4xLXRvLUIzLjEg
c2VxdWVuY2luZworKipTb3VyY2UgcHJvdmVuYW5jZToqKiBleHRyYWN0ZWQgYnl0ZS1mb3ItYnl0
ZSBmcm9tIFtgLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMxNC1jb250cmFjdC1wcm9tb3Rp
b24tZ2F0ZXNgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzE0LWNvbnRyYWN0LXByb21v
dGlvbi1nYXRlcyksIGJhc2VsaW5lIGxpbmVzIDIxNeKAkzIzMTsgdGhlIGV4YWN0IDExMDYtYnl0
ZSBzb3VyY2UgYm9keSBpcyBwcmVzZXJ2ZWQgYmV0d2VlbiB0aGUgYm91bmRhcnkgbWFya2VycyBi
ZWxvdworKipCYXNlbGluZSBzcGFuIFNIQS0yNTY6KiogYDBmYTc3Nzc3YjdmOGFlODViYzRlMDYw
ODI0NTRiMjQzNjc3ZDg5ZDA3MGZmNGM1MTBiZDllMTQ4YjYxYjJkNDNgCisKKzwhLS0gZXhhY3Qt
ZXh0cmFjdGVkLWJvZHk6c3RhcnQgLS0+CisjIyAxNC4gQ29udHJhY3QgcHJvbW90aW9uIGdhdGVz
CisKK0EgY29udHJhY3QgaXMgbm90IGNvbnNpZGVyZWQgbGFuZGVkIHVudGlsIHRlc3RzIHByb3Zl
OgorCisxLiBzZXJpYWxpemF0aW9uIGFuZCB2YWxpZGF0aW9uOworMi4gYXRvbWljIHBlcnNpc3Rl
bmNlIGFuZCByZXZpc2lvbiBjb25mbGljdCBoYW5kbGluZzsKKzMuIHRoZSByZWFsIGluZ3Jlc3Mv
ZGlzcGF0Y2gvcnVudGltZSBwYXRoIHVzZXMgaXQ7Cis0LiByZXN0YXJ0L3JlcGxheSBiZWhhdmlv
ciB3aGVyZSBkdXJhYmxlOworNS4gZmFpbC1jbG9zZWQgbmVnYXRpdmUgY2FzZXM7Cis2LiBldmVy
eSByZXZpc2lvbi1ib3VuZCBob3N0IHRyYW5zaXRpb24gam9pbnMgaW50ZW50IGlzc3VhbmNlLCBj
bGFpbSwgYXV0aG9yaXR5IGFwcGxpY2F0aW9uLCBhbmQgZXhhY3QgcmVzdWx0IG9uIHRoZSByZWFs
IENMSSBhbmQgUkVQTCBwYXRoLCBpbmNsdWRpbmcgY3Jhc2ggcmVjb25jaWxpYXRpb24gYW5kIG5v
LXJlYXBwbHkgZXhhY3QgcmV0cnk7Cis3LiBhdCBsZWFzdCBvbmUgc21va2UvZTJlIHBhdGggam9p
bnMgc2Vzc2lvbiwgYmluZGluZywgcG9saWN5LCByZWNlaXB0LCBydW50aW1lIGV2ZW50LCBhbmQg
dGVybWluYWwvb2JsaWdhdGlvbiB0cnV0aDsKKzguIGNyZWRlbnRpYWwtcmVxdWlyaW5nIHdvcmxk
IFVBQSBwcm9vZiBqb2lucyB0aGUgZW52ZWxvcGUgdG8gYSBjb25zdW1lZCBvbmUtdGltZSBpbi13
b3JsZCBnYXRld2F5IGhhbmRvZmYgd2l0aG91dCBjb3BpZWQgc2VjcmV0IGZpbGVzIG9yIGluaGVy
aXRlZCBkZXNjcmlwdG9yczsgYW5kCis5LiBubyBjb21wYXRpYmlsaXR5IGNvcHkgb3IgYENvbXBh
dGliaWxpdHlVbnByb3ZlbmAgZXZpZGVuY2UgaXMgdXNlZCBmb3IgY29udHJhY3QgcHJvbW90aW9u
OyBhbmQKKzEwLiBCMS9CMi4xIHByb2R1Y3Rpb24gcHJvb2Ygc2hvd3MgYm90aCBhY2NlcHRlZCB3
b3JrIGZhbWlsaWVzIGVudGVyIHRoZSBkdXJhYmxlIHN1cGVydmlzb3Igd2l0aG91dAorICAgIGEg
bGVnYWN5LXdyaXRlciBhdHRlbXB0LCBjYWxsZXIvZm9yZWdyb3VuZCBkcm9wIGRvZXMgbm90IGVy
YXNlIHRydXRoLCBhbmQgQjMuMSBiZWdpbnMgb25seSBhZnRlcgorICAgIHRoZSBqb2ludCBjbG9z
ZW91dC4KKworPCEtLSBleGFjdC1leHRyYWN0ZWQtYm9keTplbmQgLS0+CmBgYAo=
```

</details>

## Preserved review answer

<details>
<summary>Initial-range review answer (rendered copy)</summary>

``````text
No qualifying findings exist.

I reviewed the complete bounded seven-path candidate against the stated P1/P2 threshold and invariants. The four extracted bodies preserve the cancellation enum/rules, final-receipt conditions and immutability constraints, narrowing formulas/table/rules, and promotion gates in the supplied order and wording.

The root changes reduce those four bodies to compatibility pointers while retaining the required legacy headings/anchors. The index contains one owner row per extracted section, and each of the four ledger entries carries the complete four-owner atomic rollback covering restoration of all source bodies and removal of all four owners, index rows, and ledger rows.

I also independently confirmed from the supplied patch that its SHA-256 matches the declared candidate digest, all four new-file SHA-256 values match the manifest, and all four marker-body byte counts and SHA-256 values match invariant 2. These checks are consistent with the packet's supplied link/anchor, seven-path-fence, byte-stability, and forward/reverse validation evidence.

VERDICT: APPROVED
``````

</details>

<details>
<summary>Initial-range review answer exact bytes (base64 UTF-8)</summary>

```text
Tm8gcXVhbGlmeWluZyBmaW5kaW5ncyBleGlzdC4KCkkgcmV2aWV3ZWQgdGhlIGNvbXBsZXRlIGJv
dW5kZWQgc2V2ZW4tcGF0aCBjYW5kaWRhdGUgYWdhaW5zdCB0aGUgc3RhdGVkIFAxL1AyIHRocmVz
aG9sZCBhbmQgaW52YXJpYW50cy4gVGhlIGZvdXIgZXh0cmFjdGVkIGJvZGllcyBwcmVzZXJ2ZSB0
aGUgY2FuY2VsbGF0aW9uIGVudW0vcnVsZXMsIGZpbmFsLXJlY2VpcHQgY29uZGl0aW9ucyBhbmQg
aW1tdXRhYmlsaXR5IGNvbnN0cmFpbnRzLCBuYXJyb3dpbmcgZm9ybXVsYXMvdGFibGUvcnVsZXMs
IGFuZCBwcm9tb3Rpb24gZ2F0ZXMgaW4gdGhlIHN1cHBsaWVkIG9yZGVyIGFuZCB3b3JkaW5nLgoK
VGhlIHJvb3QgY2hhbmdlcyByZWR1Y2UgdGhvc2UgZm91ciBib2RpZXMgdG8gY29tcGF0aWJpbGl0
eSBwb2ludGVycyB3aGlsZSByZXRhaW5pbmcgdGhlIHJlcXVpcmVkIGxlZ2FjeSBoZWFkaW5ncy9h
bmNob3JzLiBUaGUgaW5kZXggY29udGFpbnMgb25lIG93bmVyIHJvdyBwZXIgZXh0cmFjdGVkIHNl
Y3Rpb24sIGFuZCBlYWNoIG9mIHRoZSBmb3VyIGxlZGdlciBlbnRyaWVzIGNhcnJpZXMgdGhlIGNv
bXBsZXRlIGZvdXItb3duZXIgYXRvbWljIHJvbGxiYWNrIGNvdmVyaW5nIHJlc3RvcmF0aW9uIG9m
IGFsbCBzb3VyY2UgYm9kaWVzIGFuZCByZW1vdmFsIG9mIGFsbCBmb3VyIG93bmVycywgaW5kZXgg
cm93cywgYW5kIGxlZGdlciByb3dzLgoKSSBhbHNvIGluZGVwZW5kZW50bHkgY29uZmlybWVkIGZy
b20gdGhlIHN1cHBsaWVkIHBhdGNoIHRoYXQgaXRzIFNIQS0yNTYgbWF0Y2hlcyB0aGUgZGVjbGFy
ZWQgY2FuZGlkYXRlIGRpZ2VzdCwgYWxsIGZvdXIgbmV3LWZpbGUgU0hBLTI1NiB2YWx1ZXMgbWF0
Y2ggdGhlIG1hbmlmZXN0LCBhbmQgYWxsIGZvdXIgbWFya2VyLWJvZHkgYnl0ZSBjb3VudHMgYW5k
IFNIQS0yNTYgdmFsdWVzIG1hdGNoIGludmFyaWFudCAyLiBUaGVzZSBjaGVja3MgYXJlIGNvbnNp
c3RlbnQgd2l0aCB0aGUgcGFja2V0J3Mgc3VwcGxpZWQgbGluay9hbmNob3IsIHNldmVuLXBhdGgt
ZmVuY2UsIGJ5dGUtc3RhYmlsaXR5LCBhbmQgZm9yd2FyZC9yZXZlcnNlIHZhbGlkYXRpb24gZXZp
ZGVuY2UuCgpWRVJESUNUOiBBUFBST1ZFRAo=
```

</details>
