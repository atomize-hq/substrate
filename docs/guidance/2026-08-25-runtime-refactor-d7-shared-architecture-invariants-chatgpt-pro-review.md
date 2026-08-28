# ChatGPT Pro advisory review: runtime-refactor D7 shared target architecture and invariants

- Date: 2026-08-25
- Bound baseline commit/tree: `8f8561a87cd20e1bd1d639d66a271595860d6584` / `4a5ab3980d38e5d4ea8b9a1c2349bb78dc4da3f4`
- Landing subagent thread (`gpt-5.4`, Extra High): `01a039c1-e7a8-78c1-a29c-6e026dcdc899`
- Initial independent review chat: https://chatgpt.com/c/6a8dc85c-e2ac-83e9-8be2-d8762144be8f
- Remediation subagent thread (`gpt-5.4`, Extra High): `01a039dd-599a-7eb3-bc22-203e88b7e778`
- Remediation-follow-up review chat: https://chatgpt.com/c/6a8dcbbd-65a4-83ea-87da-ee9b90d1884b
- ChatGPT account surface: `Pro`
- Visible reasoning-effort control: `Extra High`
- Exact model label: not exposed by the fresh ChatGPT UI
- Initially reviewed patch: `sha256:3baf3c2ae4df2b81a188dab990e5871d7b509fe78c2c6059df2a56e36b277130`
- Initial review ZIP: `sha256:2a2c846e7ee08b402fe6334f545b2ff18c254c6c485872a046f344dc5d84d8d0`
- Initial review prompt: `sha256:fa899a55529767c48af2d5fb33c58778f0025b7ae3093bc5a371331b16d71bf7`
- Remediation delta: `sha256:8bf4b291fa3ea7e2482edad38939155f80883d93d5493bf4d16c3b8b86c1b67b`
- Ordered initial-patch-plus-remediation-delta digest: `sha256:b6f8af377e45102b3958d56db63de19324c313e8b50175351d9ec96a2a1ea5d2`
- Remediation review ZIP: `sha256:33ea5ad681126bb747b8a1f4877b593310c552f79e57a75e664937be3b52cdd8`
- Remediation review prompt: `sha256:96d6f63b40cd954b2d3d92134bc4b246d423b0de43817e0cea81bf3aaa0063e4`

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## D7 source-span inventory and landing boundary

Live inventory found the only uniquely canonical D7 shared-architecture prose in `01-target-architecture.md` on the clean D6-complete baseline:

| D7 unit | Baseline line range |
|---|---:|
| Executive decision | 3–56 |
| Authority map | 62–83 |
| Invariant 1 | 87–132 |
| Invariant 2 | 134–147 |
| Invariant 3 | 149–151 |
| Invariant 4 | 153–165 |
| Invariant 5 | 167–169 |
| Invariant 6 | 171–173 |
| Invariant 7 | 175–177 |
| Invariant 8 | 179–186 |
| Invariant 9 | 188–190 |
| Invariant 10 | 192–201 |
| Invariant 11 | 203–205 |
| Invariant 12 | 207–209 |
| Invariant 13 | 211–236 |
| Invariant 14 | 238–380 |
| Stable review question | 487–493 |

No uniquely canonical D7-local span remained in `00-README.md`, `02-seam-crosswalk.md`, `03-phase-slice-map.md`, `04-contracts-and-gates.md`, or `05-debug-regression-ledger.md`. The baseline `Remaining authenticated runtime projections` section at lines 382–482 is packet/history material, not a D7 shared invariant, and remains byte-exact and root-canonical after remediation because no exact D5/D6 canonical owner exists.

The 17 D7 units form one joined shared-architecture family: the executive decision introduces the authority surfaces, the authority map assigns their ownership boundaries, invariants 1–14 constrain those same boundaries, and the stable review question tests the complete joined authority proof. Each independently interpretable invariant has its own canonical file while the exchange remains one atomic family landing.

## D7 landing files

The landing subagent changed these 23 source-document paths:

- `llm-last-mile/runtime-refactor/01-target-architecture.md`
- `llm-last-mile/runtime-refactor/a1-2-earlier-histories/README.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `llm-last-mile/runtime-refactor/architecture/README.md`
- `llm-last-mile/runtime-refactor/architecture/authority-map.md`
- `llm-last-mile/runtime-refactor/architecture/executive-target.md`
- `llm-last-mile/runtime-refactor/architecture/review-question.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/README.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/01-durable-session-truth-is-process-independent.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/02-private-transports-are-fast-paths.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/03-routing-is-exact-and-fail-closed.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/04-long-lived-work-accepts-before-it-completes.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/05-cancel-targets-active-work.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/06-obligations-are-event-derived-canonical-truth.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/07-auto-attach-restores-ownership-only.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/08-world-placement-and-policy-mediation-are-separate-proofs.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/09-external-sandbox-assigns-responsibility.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/10-dispatch-policy-only-narrows.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/11-existing-world-fs-enforcement-is-the-execution-path.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/12-runtime-native-configuration-is-projection.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/13-credentials-are-launch-time-gateway-handoff-not-projected-files.md`
- `llm-last-mile/runtime-refactor/architecture/invariants/14-substrate-home-is-private-per-user-authority-state.md`

It created canonical owners under `architecture/`, retained every legacy root heading and anchor as a compatibility forwarder, added the `shared-target-architecture` navigation row, added 17 D7 extraction-ledger rows, and made the strictly necessary compatibility-truth update in `a1-2-earlier-histories/README.md`. It did not modify D5/D6 canonical family owners, `review-control/`, implementation code, or D8+ surfaces.

## Initial local validation

- `git diff --check`
- `git diff --no-index --check /dev/null <new-file>` for all 19 new `architecture/**/*.md` files
- `python3 /tmp/validate_d7_shared_architecture.py`
  - `D7_VALIDATION_OK`
  - `changed_paths=23`
  - `d7_rows=17`
  - reproduced all 17 baseline source-span SHA-256 values
  - reproduced all 17 destination bodies, with only the two disclosed repository-relative packet-link rebases
  - verified invariant numbering, root forwarders, navigation, changed-path fence, and excluded-family/review-control path stability
- `git apply --reverse --check /tmp/d7-shared-architecture-invariants.patch`

## Initial independent review

The initial review returned one accepted `Important` finding and no other qualifying finding. The reviewed candidate had accidentally deleted the non-D7 `Remaining authenticated runtime projections` heading/body at baseline lines 382–482 while replacing the adjacent D7 invariant-14 and review-question spans. That deletion removed a legacy root anchor and R2-2E/F0a/F0b packet/history material that did not belong in the extracted shared invariant family.

Initial review result: **`CHANGES_REQUIRED`**.

## Bounded local remediation

A fresh remediation subagent reproduced the deletion and searched all live D5/D6 canonical owners. No existing owner contained the exact complete missing block. To avoid a false forwarder, new D6 extraction work, or packet-history misclassification under D7, remediation restored the exact baseline 102-line heading/body in `01-target-architecture.md` before `Historical pre-F architecture checkpoint`.

The remediation changed one file only:

- `llm-last-mile/runtime-refactor/01-target-architecture.md`

It added 102 lines and deleted none. The 17 D7 extraction rows, all new architecture owners, invariant numbering, navigation, and D5/D6 family content remained unchanged.

## Remediation validation

- `git diff --check`
- all 19 new architecture-file whitespace checks remained clean
- `python3 /tmp/validate_d7_shared_architecture.py`
  - retained `D7_VALIDATION_OK`, `changed_paths=23`, and `d7_rows=17`
  - additionally proved the restored non-D7 root block byte-exact against baseline
- `git apply --reverse --check /tmp/d7-shared-architecture-remediation.patch`
- content-addressed review-bundle manifest verification
- ordered initial-patch-plus-remediation-delta digest reproduction

## Fresh remediation-follow-up review

The fresh review returned:

- **`No qualifying Critical or Important findings.`**
- **`APPROVED`**

It explicitly confirmed that the accepted finding was fully resolved, the exact root heading/anchor and history were restored, the remediation was limited to one file and 102 inserted lines, the 17 D7 extraction units remained untouched, no false canonical forwarder was created, and all supplied artifact identities reproduced.

## Final verdict

D7 shared target architecture and invariants are complete locally and approved after remediation-follow-up review. D8 seam families and inventories remain untouched and blocked pending later explicit authorization. No commit or push was performed.

## Preserved review prompts and answers

<details>
<summary>Initial review prompt</summary>

```text
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: exact clean baseline 8f8561a87cd20e1bd1d639d66a271595860d6584 plus attached complete patch sha256:3baf3c2ae4df2b81a188dab990e5871d7b509fe78c2c6059df2a56e36b277130
Review target: complete bounded D7 shared target-architecture and invariant-family landing represented by the attached patch. Read the entire patch, including all deleted root bodies, all new files, compatibility forwarders, navigation update, compatibility-truth correction, and 17 extraction-ledger rows. Do not infer unavailable repository content beyond the explicit baseline facts below.

Supported inputs and behavior: Markdown documentation and repo-relative Markdown links/anchors only. This is a documentation decomposition: canonical content is moved without changing runtime behavior or granting implementation authority. The patch is the authoritative review artifact.
Supported platforms and dialects: architecture text spans Linux, macOS, Windows, runtime families, and shared policy/session/world concepts exactly as preserved in the patch. No platform implementation, native proof execution, code change, contract/gate decomposition, seam decomposition, task decomposition, evidence decomposition, or D12 cutover is in scope.

In-scope invariants:
1. D7 may extract only stable shared architecture: executive target decision, authority map, numbered invariants or inseparable invariant families, shared ownership boundaries, and stable review question.
2. Preserve invariant numbering and extracted source bodies exactly, except required repo-relative link rebasing after relocation.
3. Preserve every legacy root heading/anchor with a truthful canonical forwarder; do not perform the final D12 root compatibility-index cutover.
4. Preserve packet/history separation from completed D5/D6 work. No packet-family-local material may become uniquely canonical in shared architecture; packet-local forwarders must continue to point to their existing canonical owners.
5. Preserve semantic status, exclusions, predecessor/successor relations, and authority boundaries: A1.3-P1 remains the sole active global implementation packet; AUTHORITY_REQUIRED:MACOS_DEV_PARITY remains lane-local; Phase 1 grants no Phase 2 or native-operation authority; Attempt 4 quarantine and Linux-first/macOS separation remain exact.
6. Existing focused family directories and review-control evidence remain unchanged except the single attached A1.2 compatibility-truth sentence update needed because 01 is no longer root-canonical.
7. D8 seams, D9 slices/tasks, D10 contracts/gates, D11 evidence, D12 cutover, implementation, runtime proof, platform mutation, and unrelated cleanup remain excluded.
8. The extraction ledger must truthfully enumerate every moved D7 source span, its baseline line range/hash, canonical destination, semantic role, projection state, intentional transform, and rollback unit.
9. The landing must be one truthful inseparable shared-architecture/invariant-family exchange rather than an artificial mega-family. Evaluate whether the authority map, 14 numbered invariants, executive target, and review question have enough shared joins to justify this one exchange.
10. All changed/new links and anchors in the attached patch must resolve under ordinary GitHub-style Markdown anchor behavior, including nested packet-family forwarders.

Required material consequence: A blocking finding must identify a concrete content loss or semantic change, broken or misleading canonical/compatibility link or anchor, false authority/status claim, packet/history misclassification, extraction-ledger defect that makes rollback or provenance untruthful, D8+ scope widening, or a demonstrated violation of the one-family landing rule.
Blocking threshold: Critical or Important only when the defect is introduced by this patch, directly reachable in the changed documentation surface, and materially violates an in-scope invariant. Suggestions, stylistic preferences, optional finer decomposition, pre-existing root wording, and concerns requiring excluded source are non-blocking and must be labeled accordingly.
Accepted prior findings: none.
Deferred or out-of-scope concerns: D8-D12 decomposition design; actual runtime implementation correctness; code/API behavior; macOS native proof, Lima mutation, Keychain work, protected lifecycle revival, Linux implementation, Windows execution; edits to existing D5/D6 family owners or review-control evidence; general prose cleanup or minimization.
Project sources: the attached file d7-shared-architecture-invariants.patch is the complete content-addressed review packet. Read the patch before evaluating. Excluded source is unavailable and must not be inferred.
Validation evidence:
- baseline HEAD and clean tree verified before landing;
- git diff --check passed;
- every new architecture Markdown file passed git diff --no-index --check /dev/null;
- python3 /tmp/validate_d7_shared_architecture.py returned D7_VALIDATION_OK, changed_paths=23, d7_rows=17;
- validation reproduced 17 baseline source-span SHA-256 values, exact destination bodies, the two required packet-link rebases, invariant numbering/index completeness, compatibility forwarders, changed-path fence, and excluded D5/D6/review-control path stability;
- git apply --reverse --check d7-shared-architecture-invariants.patch passed.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return findings first by severity. For each finding include file/section, reachability through the attached patch, violated invariant, material consequence, and patch causality. State explicitly when there are no qualifying findings. End with exactly one result line: APPROVED; CHANGES_REQUIRED; or HUMAN_DECISION_REQUIRED.
```

</details>

<details>
<summary>Initial review answer</summary>

```text
Important

llm-last-mile/runtime-refactor/01-target-architecture.md — #### Remaining authenticated runtime projections

Reachability through the attached patch: The baseline section begins at line 381 and continues through line 482. The patch deletes the heading and its R2-2E/F0a/F0b history, then the post-D7 root proceeds from the invariant-14 Installation prefix authority and platform realization forwarder directly to ### Historical pre-F architecture checkpoint. There is no added Remaining authenticated runtime projections heading or compatibility forwarder anywhere in the complete patch.

Violated invariant: Invariant 3, requiring every legacy root heading/anchor to remain with a truthful canonical forwarder until D12. It also conflicts with Invariant 4's requirement that packet/history-local material continue through its existing canonical owners rather than simply losing its root compatibility route.

Material consequence: The legacy root anchor 01-target-architecture.md#remaining-authenticated-runtime-projections is removed before the authorized D12 cutover. Existing references to that root anchor therefore lose their compatibility destination, and the R2-2E/F0a/F0b packet-history material that D7 intentionally should not make shared-canonical is removed without the required forwarder to its existing D5/D6 owner.

Patch causality: This is introduced directly by the D7 patch: the heading and body are explicit deletions, while neither 01-target-architecture.md nor any added architecture file supplies the required compatibility heading/forwarder.

No other qualifying Critical or Important finding was identified. The 17 ledger source hashes reproduce correctly, the declared destination bodies preserve those source spans exactly apart from the two documented repository-relative link rebases, invariant numbering is preserved, and the shared architecture set otherwise forms a coherent single D7 exchange.

CHANGES_REQUIRED
```

</details>

<details>
<summary>Remediation-follow-up review prompt</summary>

```text
Review mode: remediation-follow-up
Review context: independent fresh conversation
Review boundary: exact clean baseline 8f8561a87cd20e1bd1d639d66a271595860d6584; previously reviewed D7 patch sha256:3baf3c2ae4df2b81a188dab990e5871d7b509fe78c2c6059df2a56e36b277130; current remediation delta sha256:8bf4b291fa3ea7e2482edad38939155f80883d93d5493bf4d16c3b8b86c1b67b; composite ordered artifact-pair digest sha256:b6f8af377e45102b3958d56db63de19324c313e8b50175351d9ec96a2a1ea5d2
Review target: verify the one accepted initial-review finding is fixed and inspect the one-file remediation delta against the previously reviewed complete D7 patch only for regressions caused by that delta. Read INITIAL-REVIEW-ANSWER.txt, then the previously reviewed complete d7-shared-architecture-invariants.patch, then d7-shared-architecture-remediation.patch. Apply/read them in that order to derive the current postimage.

Supported inputs and behavior: Markdown documentation and repo-relative Markdown links/anchors only. This is a documentation decomposition with no runtime behavior or implementation authority change.
Supported platforms and dialects: the preserved architecture prose spans Linux, macOS, Windows, runtime families, and shared policy/session/world concepts. No platform implementation, native proof execution, code change, D8-D12 decomposition, or general audit is in scope.

In-scope invariants:
1. Preserve all baseline root headings/anchors until D12.
2. Preserve every baseline source body outside the 17 intentionally extracted D7 spans; do not delete packet/history material while extracting shared architecture.
3. Do not invent a false canonical forwarder or misclassify packet-local R2-2E/F0a/F0b history as a shared invariant.
4. Preserve the already-reviewed 17 D7 extraction rows, source bodies, destinations, invariant numbering, compatibility links, and D5/D6 ownership boundaries.
5. The remediation must be limited to the accepted finding and must not widen into D8+, implementation, or edits to existing D5/D6 owners.

Required material consequence: a blocking finding must prove the accepted root cause remains unfixed or that this remediation delta introduces a concrete content loss/change, broken link/anchor, false authority/status claim, packet/history misclassification, or regression in the bounded D7 surface.
Blocking threshold: Critical or Important only when directly reachable, material, and caused by the remediation delta or proof that the accepted finding remains. Suggestions, optional extraction of the restored root-canonical packet-history block, pre-existing concerns, and D8-D12 work are non-blocking/deferred.

Accepted prior findings:
- The initial patch deleted baseline lines 381-482 headed `#### Remaining authenticated runtime projections` from `01-target-architecture.md`, removing the legacy root anchor and R2-2E/F0a/F0b history without a compatibility route.
- Initial review found no other qualifying issue and otherwise accepted the 17 span hashes, exact destination bodies, link rebases, invariant numbering, and coherent single D7 exchange.

Remediation applied:
- Live search found no exact existing D5/D6 canonical owner for the full deleted block.
- The delta therefore restores the exact 102-line baseline heading/body at the same root location before `### Historical pre-F architecture checkpoint`.
- The restored block remains root-canonical and explicitly outside the 17 extracted shared-architecture spans. No false forwarder or new D6 extraction was invented.
- No ledger row changes were required because this block is not one of the 17 D7 extracted spans.

Deferred or out-of-scope concerns: later extraction/placement of the restored packet-history block; D8-D12 design; runtime implementation correctness; platform proof/mutation; edits to existing D5/D6 family owners or review-control evidence; prose cleanup/minimization.
Project sources: the attached ZIP is the complete content-addressed remediation packet. Read its manifest and files in the order above. Excluded source is unavailable and must not be inferred.
Validation evidence:
- git diff --check passed;
- all 19 new architecture files still pass git diff --no-index --check /dev/null;
- python3 /tmp/validate_d7_shared_architecture.py returned D7_VALIDATION_OK, changed_paths=23, d7_rows=17 and additionally proves the restored baseline span is byte-exact in root 01;
- git apply --reverse --check passed for the remediation delta against the current postimage and for the previously reviewed complete patch against its reviewed postimage;
- HEAD remains 8f8561a87cd20e1bd1d639d66a271595860d6584 and no commit/push occurred.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

For remediation-follow-up, report a blocking finding only when it is reachable through the attached delta/current postimage, violates an in-scope invariant or accepted remediation objective, meets the material-consequence threshold, and was introduced by the delta, is a regression caused by it, or proves that the accepted root cause remains unfixed. Unchanged pre-existing behavior and future placement of the restored non-D7 block are not in-scope findings merely because observed.

Return findings first with file/section, reachability, violated invariant, material consequence, and delta causality. State explicitly when there are no qualifying findings. End with exactly one result line: APPROVED; CHANGES_REQUIRED; or HUMAN_DECISION_REQUIRED.
```

</details>

<details>
<summary>Remediation-follow-up review answer</summary>

```text
No qualifying Critical or Important findings.

The accepted finding is fully resolved. In llm-last-mile/runtime-refactor/01-target-architecture.md → #### Remaining authenticated runtime projections, the remediation is directly reachable through its sole hunk and restores the missing legacy heading/anchor and R2-2E/F0a/F0b history immediately before ### Historical pre-F architecture checkpoint.

The remediation is bounded correctly: it changes one file, adds 102 lines, deletes none, and introduces no new Markdown links or canonical forwarders. Its added bytes exactly reproduce the corresponding deleted baseline heading/body plus trailing separator from the previously reviewed D7 patch. The preimage blob 96557f634… also matches the D7 patch's postimage blob, confirming the intended ordered application.

No reviewed D7 extraction content is disturbed: the 17 ledger rows, shared-architecture destinations, numbered invariant ownership, compatibility forwarders, D5/D6 owners, and navigation changes are untouched. The restored material remains root-canonical packet/history content rather than being misclassified as a D7 shared invariant. The restored heading re-establishes 01-target-architecture.md#remaining-authenticated-runtime-projections, and the remediation itself contains no repository-relative links capable of introducing a new broken target.

The supplied artifact identities also reproduce: the two patch SHA-256 values match the manifest, and concatenating the complete D7 patch followed by the remediation patch yields the stated composite digest b6f8af377e45102b3958d56db63de19324c313e8b50175351d9ec96a2a1ea5d2.

APPROVED
```

</details>
