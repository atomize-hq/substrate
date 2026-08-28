# ChatGPT Pro advisory review: runtime-refactor D8 seams — persistence/compatibility and dispatch/episode transport families

- Date: 2026-08-25
- Bound baseline commit/tree: `ae06868602e305400da6207732de2a2ffc79cc5c` / `695c85685bee955babaded08d56cf2ed2508e9c5`
- Landing subagent thread (`gpt-5.4`, Extra High): `01a03a34-c3f0-73f0-a0c1-1cc8a7bcee6c`
- Initial independent review chat: https://chatgpt.com/c/6a8de656-7454-83ea-a626-a6720e2d9b35
- ChatGPT account surface: `Pro`
- Visible reasoning-effort control: `Extra High`
- Exact model label: not exposed by the fresh ChatGPT UI
- Review mode: `initial-range`
- Reviewed complete patch: `sha256:37c52f8e5b13cd84cdc12a1432a6f23f8799c34d920b0df02aac40510ba66da3`
- Initial review ZIP: `sha256:f6bbdf49ed3fed5585f7ba6153768f52e3f51019709bb143dafb3ba973387341`
- Initial review prompt: `sha256:51b9949d1bb3e50beb9560d22b25f4120c82ddfec71b9e0f1fc1f4b41965ec5a`
- Initial review answer: `sha256:7c343ab1f66aab9aa49c48e775e11e0daf14ff8d601af48c41190e92f6dabc8f`
- Review ZIP path at review time: `/tmp/d8-persistence-dispatch-review.XXXXXX.zip`

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## D8 live source-span inventory and grouping decision

The clean post-host/session D8 baseline retained five separable live seam rows in `02-seam-crosswalk.md`:

| D8 unit | Baseline location | Landing disposition |
|---|---:|---|
| StateStore | line 39 | moved to `seams/persistence-and-compatibility.md#statestore` |
| CompatibilityReadModel | line 40 | moved to `seams/persistence-and-compatibility.md#compatibilityreadmodel` |
| InternalToolboxTransport | line 41 | moved to `seams/dispatch-and-episode-transport.md#internaltoolboxtransport` |
| RuntimeToolInvocationAdapter | line 42 | moved to `seams/dispatch-and-episode-transport.md#runtimetoolinvocationadapter` |
| WorldDispatchControl | line 48 | moved to `seams/dispatch-and-episode-transport.md#worlddispatchcontrol` |
| A0 authority-leak inventory | root `02` | explicitly retained at its existing owner |
| Host/session authority family | already under `seams/host-session-authority.md` | existing approved D8 owner preserved byte-for-byte |
| Later policy seams | root `02` lines 49–51 | left root-canonical for later D8 family landings |

This grouped landing therefore covered only the next two logical D8 seam families in root order: persistence/compatibility (`StateStore`, `CompatibilityReadModel`) and dispatch/episode transport (`InternalToolboxTransport`, `RuntimeToolInvocationAdapter`, `WorldDispatchControl`). `SteeringPolicyEngine`, `EffectivePolicyResolver`, `DispatchPolicyNarrowingPatch`, and all later D8 families remain separate blocked later work.

No uniquely D8-family-local canonical span was identified in `00-README.md`, `01-target-architecture.md`, `03-phase-slice-map.md`, `04-contracts-and-gates.md`, or `05-debug-regression-ledger.md`; those files remained unchanged by this grouped landing.

## Landing files

The reviewed D8 candidate changed exactly seven source-document paths:

- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `llm-last-mile/runtime-refactor/seams/README.md`
- `llm-last-mile/runtime-refactor/seams/dispatch-and-episode-transport.md`
- `llm-last-mile/runtime-refactor/seams/persistence-and-compatibility.md`

The final local closeout adds only this review record as the eighth changed path.

## Initial local validation

- `git diff --check`
- `git diff --no-index --check /dev/null` for both new `seams/*.md` files
- `python3 /tmp/validate_d8_grouped_batch.py`
  - `D8_GROUPED_VALIDATION_OK`
  - `changed_paths=7`
  - `new_d8_rows=5`
  - `total_d8_ledger_rows=8`
  - `root_projection_rows=5`
  - `semantic_cells_preserved=35`
  - `seam_name_links=5`
  - reproduced all five baseline source-row hashes and destination bodies
  - verified the reviewed seven-path fence, later-family exclusions, and byte-stability of the committed host/session canonical file and review artifact
- `python3 /tmp/check_d8_grouped_links.py`
  - `D8_GROUPED_LINKS_OK checks=60`
- `git apply --reverse --check /tmp/d8-grouped-seam-families.patch`
- forward `git apply --check` against a clean archive of the detached baseline
- seven-path postimage equality check passed
- index clean and nothing staged at review time

The reviewed patch includes all seven source-document paths, including both new untracked family files.

## Initial independent review

The fresh independent review returned:

- **`No qualifying findings.`**
- **`APPROVED`**

It confirmed that the seven-path fence is exact; all five source-row hashes match the manifest; both canonical destination files preserve their extracted rows byte-exact; all 35 non-name semantic cells remain byte-exact in the root compatibility projections; only the five seam-name cells become canonical links; A0 and the later policy seams remain untouched/root-canonical; the extraction ledger adds exactly five D8 row-level entries; the tracker remained truthful about local-only completion and `not committed` / `not pushed`; and no introduced authority promotion, conflicting ownership, broken required projection/navigation path, false review advancement, or excluded-family scope widening was found.

## No remediation follow-up required

The initial-range review approved the grouped batch without qualifying findings. No local remediation, no reviewed delta artifact, and no remediation-follow-up review were needed.

## Post-review closeout validation

- `git diff --check` passed.
- `git diff --no-index --check /dev/null` produced no whitespace output for both new seam family files and this review record.
- `python3 /tmp/validate_d8_grouped_batch.py` rerun on the final closeout exits nonzero only because the script still asserts the original seven-path reviewed candidate; the final closeout intentionally adds this review record as an eighth changed path.
- `python3 /tmp/check_d8_grouped_links.py` rerun returned `D8_GROUPED_LINKS_OK checks=60`.
- Exact final changed-path fence was confirmed as the original seven reviewed landing paths plus `docs/guidance/2026-08-25-runtime-refactor-d8-persistence-dispatch-seams-chatgpt-pro-review.md`.
- `llm-last-mile/runtime-refactor/seams/host-session-authority.md` remained byte-stable at `sha256:cd6c169618ba2adc6563e54360f293bc1d42bef8717223e9d4db0bcba78554dc`.
- `docs/guidance/2026-08-25-runtime-refactor-d8-seams-host-session-authority-chatgpt-pro-review.md` remained byte-stable at `sha256:b262bd475735f30ee68abc32d4069c7417be89f91dedf25125e14fd2c7ebd6f3`.
- No files were staged.
- Complete post-review closeout patch was written to `/tmp/d8-persistence-dispatch-closeout.patch`; its final SHA-256 is reported in this session closeout.
- `git apply --reverse --check /tmp/d8-persistence-dispatch-closeout.patch` passed.

## Final verdict

This D8 persistence/compatibility plus dispatch/episode transport grouped batch is complete locally and approved after initial-range review. D8 remains partially complete: the later separable seam families remain unlanded and blocked pending later explicit authorization, and D9 remains blocked by D8. No D8 commit or push was performed.

## Preserved review prompts and answers

<details>
<summary>Initial review prompt</summary>

```text
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: exact detached baseline ae06868602e305400da6207732de2a2ffc79cc5c plus complete candidate patch sha256:37c52f8e5b13cd84cdc12a1432a6f23f8799c34d920b0df02aac40510ba66da3
Review target: the complete bounded seven-path candidate for only these two logical D8 seam families: (1) persistence and compatibility projection (`StateStore`, `CompatibilityReadModel`) and (2) dispatch and episode transport (`InternalToolboxTransport`, `RuntimeToolInvocationAdapter`, `WorldDispatchControl`). Read MANIFEST.txt first, then the complete patch.

Supported inputs and behavior: Markdown control-plane decomposition records. The candidate moves exact canonical seam-row bodies out of `llm-last-mile/runtime-refactor/02-seam-crosswalk.md` into two family owner files under `seams/`, while retaining each root table row as a compatibility projection. The root projection must preserve the seven non-name semantic cells byte-exact and may change only the seam-name cell into a relative link to the new canonical row. It also updates only the shared seam index, global runtime-refactor index row, extraction ledger, and execution tracker needed for this landing.
Supported platforms and dialects: repository Markdown and relative Markdown links only. This is documentation decomposition, not Rust behavior or platform implementation. Linux implementation, macOS implementation/native proof, Windows, D9+, and any code or scripts are excluded.

In-scope invariants:
1. The patch is complete and bounded to exactly the seven paths listed in MANIFEST.txt and only the two named D8 seam families.
2. The five extracted destination rows preserve their baseline canonical source bodies exactly, including all seven non-name semantic cells and sibling relationships; only their seam-name cells may be linked in root `02`.
3. Root `02-seam-crosswalk.md` retains truthful, complete eight-cell compatibility projections for all five moved rows, with the seven non-name semantic cells byte-exact and the seam-name cell linked to the exact canonical anchor.
4. `A0 — authority leaks to remove` remains uniquely canonical in root `02`; no A0 extraction or authority change occurs.
5. The already-approved D8 host/session family owner and its review artifact remain byte-stable; previously landed D5/D6/D7 owners and review-control evidence are not reopened.
6. `SteeringPolicyEngine`, `EffectivePolicyResolver`, `DispatchPolicyNarrowingPatch`, all later D8 seam rows, and D9+ remain root-canonical and blocked for later work.
7. The landing grants no implementation or successor authority, promotes no seam, and preserves sole active global packet `A1.3-P1`, lane-local gate `AUTHORITY_REQUIRED:MACOS_DEV_PARITY`, Attempt 4 quarantine, and Linux-first/macOS-parity separation.
8. `seams/README.md`, `index/README.md`, and `migration/extraction-ledger.md` describe only the two new families, preserve reverse active chronology, and point to valid canonical files/anchors.
9. The extraction ledger contains exactly five new D8 row-level entries with truthful source, destination, projection, validation, and rollback descriptions.
10. The tracker truthfully says this batch is complete locally but pending independent review; it does not claim approval or commit/push and keeps remaining D8 and D9+ blocked.
11. Every added or changed relative Markdown link in the seven-path patch resolves to an existing file and, when an anchor is present, an existing heading.

Required material consequence: a blocking finding must identify a reachable documentation/control-plane defect introduced by this patch that materially loses or changes canonical seam semantics, creates conflicting ownership or authority, makes a required compatibility projection untruthful, breaks a required navigation/rollback path, falsely advances review/commit status, or widens scope into excluded families.
Blocking threshold: report Critical or Important findings only when the defect is introduced by this patch, is reachable in the supported Markdown consumption model, violates an in-scope invariant, and has the required material consequence. Minor editorial preferences, optional rewording, pre-existing issues, and deferred later-family work are non-blocking.
Accepted prior findings: none for this batch. The prior D8 host/session review found and remediated loss of root semantic cells; this candidate is independently reviewable and must preserve all seven non-name cells for each moved row.
Deferred or out-of-scope concerns: any D8 family not named in this target; any D9+ slice; code implementation; Linux/macOS/Windows runtime behavior; native proof; Lima, Keychain, protected lifecycle, shared-script integration; redesign of already-landed D5/D6/D7 owners; broad copyediting; new architecture policy.
Project sources: the attached ZIP is the complete bounded review packet. Excluded repository source is unavailable and must not be inferred. Read MANIFEST.txt before `d8-grouped-seam-families.patch`.
Validation evidence: `git diff --check` passed; no-index whitespace checks passed for both new files; exact validator reported `D8_GROUPED_VALIDATION_OK`, `changed_paths=7`, `new_d8_rows=5`, `total_d8_ledger_rows=8`, `root_projection_rows=5`, `semantic_cells_preserved=35`, `seam_name_links=5`; link validator reported `D8_GROUPED_LINKS_OK checks=60`; complete patch reverse-apply check passed; clean-baseline forward application passed; seven-path postimage equality passed; committed host/session canonical file and review artifact are byte-stable; index is clean and nothing is staged.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return findings first by severity. For each qualifying finding include the exact file/section, reachable supported-document path, violated invariant, material consequence, and patch causality. Then state one exact verdict line: `APPROVED` if there are no qualifying findings, or `REMEDIATION REQUIRED` if there is at least one. State explicitly when there are no qualifying findings. Do not invent findings outside the attached bounded patch.
```

</details>

<details>
<summary>Initial review answer</summary>

```text
No qualifying findings.

The bounded candidate satisfies the stated D8 invariants. In particular, the seven-path fence is exact; all five source-row hashes match the manifest; both canonical destination files preserve their extracted rows byte-exact; all 35 non-name semantic cells remain byte-exact in the root compatibility projections; and only the five seam-name cells become canonical links. A0 and the later policy seams remain untouched/root-canonical.

The extraction ledger adds exactly five row-level D8 entries, navigation is limited to the two new families, and the tracker records the batch as locally complete but **pending independent review**, explicitly `not committed` and `not pushed`. I found no introduced authority promotion, conflicting ownership, broken required projection/navigation path, false review advancement, or excluded-family scope widening.

APPROVED
```

</details>
