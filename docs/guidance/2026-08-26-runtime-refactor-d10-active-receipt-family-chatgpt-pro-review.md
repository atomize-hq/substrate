# ChatGPT Pro advisory review: runtime-refactor D10 active receipt contract family

- Date: 2026-08-26
- Bound baseline commit/tree: `740dcdc1441bccecae8944ad7229025c0f22da24` / `9c62caa64abbe04fffa0d00737dc45de5b7a1b67`
- Candidate implementation subagent (`gpt-5.4`, Extra High): `/root/d10_receipt_family_landing` (display `Herschel`)
- Closeout subagent: `/root/d10_receipt_family_closeout` (display name not exposed to this subagent runtime and therefore not asserted here)
- Root/orchestrator review owner: `/root`
- Independent review chat: https://chatgpt.com/c/6a8f4a0c-3cb0-83ea-8b80-765bb4b1709a
- Review mode: `initial-range`
- Review context: independent fresh conversation
- ChatGPT account surface: `Pro`
- Visible reasoning-effort control: `Extra High`
- Exact model label: not exposed by the current visible ChatGPT UI
- Candidate patch: `sha256:4332a5098e977b5edda996708b0ef26143d24ebce6deeb9bbe178fe8f9d89748` at `/private/tmp/d10-receipt-family-candidate.patch`
- Review prompt: `sha256:9754c76f300b00d82ca2d2095bdf09a231539592d0b7fa05e4b6802f8a090402` at `/private/tmp/d10-receipt-family-chatgpt-pro-initial-review-prompt.txt`
- Review answer: `sha256:20b184365811cc30d73471a92c31798dba649a0e708a92ee40857ea691bf6d5c` at `/private/tmp/d10-receipt-family-chatgpt-pro-initial-review-answer.txt`
- Imported validation log: `sha256:13610fc322a366e496687a51b6f6bd9bbe1307a956ac3438bb3df1ba94855bff` at `/private/tmp/d10-receipt-family-validation.log`
- Exact embedded source body (`ActiveEphemeralTaskReceiptV1`): baseline lines `91–138`, `1557 bytes`, `sha256:c997752654f041de820210f03c86706551b44389994df5e8e0341aa41a34fa3b`
- Exact embedded source body (`ActiveRetainedTurnReceiptV1`): baseline lines `139–183`, `1443 bytes`, `sha256:2cc996111d82d296f89bdd88f80df5c0b559bdb305aec546881b749ace93f67e`
- Review verdict: `APPROVED`
- Review finding summary: `No qualifying findings.`
- Remediation rounds: `0`
- Review adjudication posture: root agent owns review/adjudication; subagents do not self-approve
- Commit posture: the candidate remains uncommitted and unstaged during closeout; root will validate, run GitNexus, and commit the seven-path batch atomically; not yet committed
- Push posture: `not pushed`
- Successor authority posture: no successor dispatch authority is granted

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Candidate scope and outcome

This independently reviewed substantive D10 receipt family batch extracts `ActiveEphemeralTaskReceiptV1` and `ActiveRetainedTurnReceiptV1` into `llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md` and `llm-last-mile/runtime-refactor/contracts/active-retained-turn-receipt-v1.md`, replaces only the two root spans in `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` with shallow compatibility pointers, adds the two truthful index rows, and adds the two matching D10 extraction-ledger rows. `Receipt acceptance source` remains unchanged. The active receipt contract family is complete locally and approved after initial-range review with zero remediation and is landed pending one atomic commit. D10 remains explicitly in progress and incomplete; the exact next batched substantive D10 unit is `RetainedWorkerManifestV1` + `DispatchPolicyNarrowingPatchV1`; remaining later D10 units remain queued; D11 remains blocked until D10 is fully complete and separately authorized; D12 remains blocked by D11 and is separately unauthorized; no successor dispatch authority is granted.

## Prospective seven-path landing manifest

Prospective landing manifest for this independently reviewed batch:

- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md`
- `llm-last-mile/runtime-refactor/contracts/active-retained-turn-receipt-v1.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `docs/guidance/2026-08-26-runtime-refactor-d10-active-receipt-family-chatgpt-pro-review.md`

## Candidate path digests and imported review artifacts

Candidate path digests:

- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`: `sha256:47f3da5500b3da843f1e87acecf81f5b8733d0790d3e318b4013485cfef5bde0`
- `llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md`: `sha256:5e106d88d1826641fe1b26a838717c43baf3171f67ae4dff2181f82e7d6f6887`
- `llm-last-mile/runtime-refactor/contracts/active-retained-turn-receipt-v1.md`: `sha256:3e4b187aa6f7a8d57fdeeab563e6866d4a962fec685c2b9d6285209dd91b22d0`
- `llm-last-mile/runtime-refactor/index/README.md`: `sha256:d91a882a8a17a823d19422607a2eecb049489df89c08b88be06158b0eea510c1`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`: `sha256:ee38b69b091192bc27a35fc5f1e4b316572562013250e374b786d28ed7a22653`

Imported review artifacts:

- Candidate patch: `sha256:4332a5098e977b5edda996708b0ef26143d24ebce6deeb9bbe178fe8f9d89748`
- Review prompt: `sha256:9754c76f300b00d82ca2d2095bdf09a231539592d0b7fa05e4b6802f8a090402`
- Review answer: `sha256:20b184365811cc30d73471a92c31798dba649a0e708a92ee40857ea691bf6d5c`
- Imported validation log: `sha256:13610fc322a366e496687a51b6f6bd9bbe1307a956ac3438bb3df1ba94855bff`
- Exact embedded source body (`ActiveEphemeralTaskReceiptV1`): `1557 bytes`, `sha256:c997752654f041de820210f03c86706551b44389994df5e8e0341aa41a34fa3b`
- Exact embedded source body (`ActiveRetainedTurnReceiptV1`): `1443 bytes`, `sha256:2cc996111d82d296f89bdd88f80df5c0b559bdb305aec546881b749ace93f67e`

## Exact source-body proof

- Baseline `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` lines `91–138` are exactly `1557` bytes with SHA-256 `c997752654f041de820210f03c86706551b44389994df5e8e0341aa41a34fa3b`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` in `llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md` are also exactly `1557` bytes with SHA-256 `c997752654f041de820210f03c86706551b44389994df5e8e0341aa41a34fa3b`.
- Baseline `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` lines `139–183` are exactly `1443` bytes with SHA-256 `2cc996111d82d296f89bdd88f80df5c0b559bdb305aec546881b749ace93f67e`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` in `llm-last-mile/runtime-refactor/contracts/active-retained-turn-receipt-v1.md` are also exactly `1443` bytes with SHA-256 `2cc996111d82d296f89bdd88f80df5c0b559bdb305aec546881b749ace93f67e`.
- The baseline source spans and the two owner-body byte ranges are byte-identical, including the final blank separator before each end marker.
- The two ledger entries preserve the receipt family as one atomic rollback unit: each row requires restoring both root bodies, removing both canonical owners, removing both index rows, and removing both batch ledger entries.
- `Receipt acceptance source` remains the root-compatible owner and is unchanged by this batch.

## Validation evidence

- The supplied review answer states `No qualifying findings.` and ends with `VERDICT: APPROVED`.
- The imported validation log confirms unique root anchors for `3-activeephemeraltaskreceiptv1` and `4-activeretainedturnreceiptv1`, unique owner anchors for both canonical files, exact marker-body equality for both owners, an unchanged `Receipt acceptance source` compatibility owner, a five-path candidate fence, and clean candidate-patch forward/reverse application.
- The candidate patch changes exactly the five imported candidate paths and no others.
- Candidate file hashes, candidate-patch SHA-256, prompt SHA-256, answer SHA-256, validation-log SHA-256, and both exact source-body SHA-256 digests were all reverified during this closeout.
- The rendered prompt and rendered answer below are whitespace-clean: every trailing literal space from the imported UTF-8 prompt is rendered as `&#32;` so this repository-local Markdown artifact stays clean under `git diff --check` while the base64 blocks preserve the exact bytes.
- The rendered prompt and rendered answer were both revalidated against the imported files, and the embedded base64 blocks decode exactly back to the imported UTF-8 bytes without silently correcting any captured whitespace or fence content.
- Final changed-path fence including untracked files is exactly these seven paths:
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md`
- `llm-last-mile/runtime-refactor/contracts/active-retained-turn-receipt-v1.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `docs/guidance/2026-08-26-runtime-refactor-d10-active-receipt-family-chatgpt-pro-review.md`
- `git diff --check` passed for the final closeout state.
- `git diff --no-index --check /dev/null docs/guidance/2026-08-26-runtime-refactor-d10-active-receipt-family-chatgpt-pro-review.md` reported no whitespace diagnostics.
- Tracker truth validation passed for only the active receipt family approved, D10 still incomplete, the exact next batched unit `RetainedWorkerManifestV1` + `DispatchPolicyNarrowingPatchV1`, D11 blocked, D12 separately unauthorized, and no successor dispatch authority.
- Prior D3/D5–D10 owner/review-artifact stability passed via the exact seven-path fence.
- Repository Markdown link/anchor validation passed with `LINKS_CHECKED 1833 FAILURES 0`.
- Complete combined candidate+closeout patch was generated at `/private/tmp/d10-receipt-family-closeout.patch` with a SHA-256 sidecar, and forward/reverse apply against disposable clean baseline exports both passed.
- No commit, stage, or push is performed during this closeout.

## Atomic rollback instructions

1. If the prospective seven-path batch has been committed, reverse that single batch commit atomically (for example, `git revert --no-commit <batch-commit>`) and verify that only the seven paths above are touched.
2. If the batch is still uncommitted, restore the tracked paths from the parent baseline and remove the three untracked files: `git restore --source=HEAD -- llm-last-mile/runtime-refactor/04-contracts-and-gates.md llm-last-mile/runtime-refactor/index/README.md llm-last-mile/runtime-refactor/migration/extraction-ledger.md docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md` then `rm -f llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md llm-last-mile/runtime-refactor/contracts/active-retained-turn-receipt-v1.md docs/guidance/2026-08-26-runtime-refactor-d10-active-receipt-family-chatgpt-pro-review.md`.
3. After either rollback path, confirm together that both `04-contracts-and-gates.md` receipt bodies are restored, both canonical owner files are removed, both `index/README.md` rows are removed, both `migration/extraction-ledger.md` rows are removed, the tracker summary/queue/closeout-row additions are reverted, and the review record is removed.
4. Leave `HostExecutionEpisodeV1`, deferred retained-spawn admission recovery, `Receipt acceptance source`, prior D3/D5–D10 owners and review artifacts, `index/current.md`, `review-control/`, remaining later D10 units, and every path outside this seven-path batch unchanged.
5. Never strand one member of the receipt family: both owners, both root compatibility pointers, both index rows, both ledger rows, the tracker update, and this review record roll back together or not at all.

## Final verdict

`ActiveEphemeralTaskReceiptV1` + `ActiveRetainedTurnReceiptV1` is complete locally and **APPROVED** after initial-range review with zero remediation. D10 remains explicitly in progress and incomplete; the exact next batched substantive D10 unit is `RetainedWorkerManifestV1` + `DispatchPolicyNarrowingPatchV1`; remaining later D10 units remain queued; D11 remains blocked until D10 is fully complete and separately authorized; D12 remains blocked by D11 and is separately unauthorized; no successor dispatch authority is granted.

## Preserved review prompt

<details>
<summary>Initial-range review prompt (rendered copy)</summary>

``````text
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: exact baseline commit 740dcdc1441bccecae8944ad7229025c0f22da24, tree 9c62caa64abbe04fffa0d00737dc45de5b7a1b67, plus the complete content-addressed candidate patch below. Candidate patch SHA-256: 4332a5098e977b5edda996708b0ef26143d24ebce6deeb9bbe178fe8f9d89748.
Review target: the complete bounded D10 documentation extraction batch for the inseparable active receipt contract family. Review every hunk and both new files; no unprovided repository source may be inferred.

Manifest (exactly five changed paths, including untracked new files):
- llm-last-mile/runtime-refactor/04-contracts-and-gates.md — SHA-256 47f3da5500b3da843f1e87acecf81f5b8733d0790d3e318b4013485cfef5bde0
- llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md — new — SHA-256 5e106d88d1826641fe1b26a838717c43baf3171f67ae4dff2181f82e7d6f6887
- llm-last-mile/runtime-refactor/contracts/active-retained-turn-receipt-v1.md — new — SHA-256 3e4b187aa6f7a8d57fdeeab563e6866d4a962fec685c2b9d6285209dd91b22d0
- llm-last-mile/runtime-refactor/index/README.md — SHA-256 d91a882a8a17a823d19422607a2eecb049489df89c08b88be06158b0eea510c1
- llm-last-mile/runtime-refactor/migration/extraction-ledger.md — SHA-256 ee38b69b091192bc27a35fc5f1e4b316572562013250e374b786d28ed7a22653

Supported inputs and behavior: GitHub-flavored Markdown documentation with ATX headings and generated heading anchors, relative Markdown links with optional fragments, fenced code blocks, Markdown tables whose row and cell order is semantic, inline code/literals, and HTML comments used as extraction-body hash markers. The legacy root remains a compatibility projection until a separately authorized D12 cutover. Canonical owners are normative; root compatibility headings and anchors must continue to resolve.
Supported platforms and dialects: repository Markdown rendered with GitHub-style heading slug behavior and validated by the project's Markdown link/anchor checker. No runtime, Rust, script, platform, parser, dispatch, or implementation behavior is in scope.

In-scope invariants:
1. Exactly one canonical owner exists for each of ActiveEphemeralTaskReceiptV1 and ActiveRetainedTurnReceiptV1.
2. Each extracted canonical body is byte-for-byte identical to its frozen live-root source span, including ordered fields, table-cell order, fenced schemas/code, literals, hashes, commands, validation sequence, negative requirements, exclusions, exception states, version suffixes, semantic status, and authority boundaries.
3. Frozen source artifact facts: ActiveEphemeralTaskReceiptV1 body is 1557 bytes with SHA-256 c997752654f041de820210f03c86706551b44389994df5e8e0341aa41a34fa3b; ActiveRetainedTurnReceiptV1 body is 1443 bytes with SHA-256 2cc996111d82d296f89bdd88f80df5c0b559bdb305aec546881b749ace93f67e.
4. The root must retain the exact legacy headings/anchors `3-activeephemeraltaskreceiptv1` and `4-activeretainedturnreceiptv1`, replacing their duplicated bodies only with truthful links to the canonical owners.
5. Navigation must contain exactly one row for each new owner, with correct relative links and no broken links or anchors.
6. Extraction-ledger provenance must contain exactly one truthful D10 row per owner, including exact source span/hash and complete atomic rollback instructions that restore both root bodies and remove both owners/navigation/ledger rows together.
7. The two contracts are one atomic version family; rollback must not strand only one receipt owner.
8. Existing receipt-acceptance-source compatibility ownership and all prior D3/D5-D10 owners/review artifacts must remain stable.
9. The patch must not claim implementation, dispatch, gate satisfaction, successor authority, D10 completion, D11 authority, or D12 cutover.
10. The exact five-path fence is mandatory; no formatting normalization or unrelated edits are allowed.

Required material consequence: a finding blocks only if the complete patch makes a supported repository reader materially lose, alter, duplicate, or misidentify contract semantics or canonical ownership; breaks a supported link/legacy anchor/provenance/atomic rollback; falsifies status or authority; violates the exact path fence; or prevents clean application/reversal. Pure style preferences, optional prose, broader redesigns, theoretical unsupported Markdown behavior, or unchanged baseline defects are not material.
Blocking threshold: report only reachable, patch-causal P1 or P2 defects that violate an in-scope invariant and meet the material-consequence threshold. A complete body/hash mismatch, wrong owner or target, broken required anchor/link, incomplete rollback, false authority/status, or scope breach qualifies. Non-blocking observations must be clearly labeled deferred and must not change the verdict.
Accepted prior findings: none.
Deferred or out-of-scope concerns: D3 review-governance; packet-local D5/D6 content already canonically owned; D9 slices/tasks; D11 evidence decomposition; D12 root cutover; Rust/scripts/runtime/platform behavior; roadmap or reviewed ZIP changes; global Markdown formatting; unrelated pre-existing issues; style-only improvements; implementation or successor dispatch.
Project sources: only this prompt, manifest, invariants, validation evidence, and the complete patch below are available. Excluded source is unavailable and must not be inferred.

Validation evidence:
- exact five-path fence including untracked files: PASS
- no staged files: PASS
- `git diff --check`: PASS
- `git diff --no-index --check /dev/null <new-file>` for both new owners: zero diagnostics (the command's ordinary difference exit status is not a whitespace failure)
- Markdown link/anchor validation: 1,833 links checked, 0 failures
- both root legacy anchors and both owner anchors: unique and resolving
- extracted marker-body equality: 1557/1557 bytes and c997... hash equal; 1443/1443 bytes and 2cc9... hash equal
- root canonical pointers, exactly one index row each, and exactly one ledger row each: PASS
- receipt-acceptance-source compatibility pointer: unchanged
- prior D10 owners and all paths outside the manifest: stable
- patch forward-apply and exact five-file content match from clean baseline: PASS
- patch reverse-apply and exact clean-baseline restoration: PASS
- candidate validation log SHA-256: 13610fc322a366e496687a51b6f6bd9bbe1307a956ac3438bb3df1ba94855bff

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return findings first by severity. For every blocking finding, state the exact file/hunk, supported-input reachability, violated invariant, material consequence, and patch causality. State explicitly when there are no qualifying findings. End with exactly one of:
VERDICT: APPROVED
VERDICT: CHANGES REQUIRED

Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

COMPLETE CONTENT-ADDRESSED PATCH (SHA-256 4332a5098e977b5edda996708b0ef26143d24ebce6deeb9bbe178fe8f9d89748):
```diff
diff --git a/llm-last-mile/runtime-refactor/04-contracts-and-gates.md b/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
index 7b80bac59..a9e7c32f0 100644
--- a/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
+++ b/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
@@ -90,96 +90,11 @@ Canonical content: [`b1-b2-1/contracts-and-gates.md#b1-frozen-proposal-persisten
&#32;
 ## 3. `ActiveEphemeralTaskReceiptV1`
&#32;
-```rust
-struct ActiveEphemeralTaskReceiptV1 {
-    schema_version: u32,
-    acceptance_record_id: String,
-    task_run_id: String,
-    request_id: String,
-    orchestration_session_id: String,
-    caller_participant_id: String,
-    target_backend_id: String,
-    world_id: String,
-    world_generation: u64,
-    policy_snapshot_ref: PolicySnapshotRefV1,
-    policy_snapshot_hash: String,
-    policy_revision: String,
-    narrowing_reason: Option<String>,
-    runtime_acceptance: RuntimeAcceptanceEvidenceV1,
-    observation_claim: SupervisorObservationClaimV1,
-    accepted_at: Timestamp,
-    state_revision: u64,
-    state: ActiveTaskStateV1,
-    cancel_supported: bool,
-    terminal: Option<WorldWorkTerminalV1>,
-}
-```
-
-States:
-
-```text
-Accepted -> Running -> AttentionPending -> Running
-Accepted|Running|AttentionPending -> Terminal|Failed|Cancelled|Invalidated
-```
-
-`Terminal`, `Failed`, `Cancelled`, and `Invalidated` are terminal and monotonic. `Parked` is not valid for an ephemeral task.
-
-`WorldWorkTerminalV1` for an ephemeral task carries one explicit result class:
-
-```text
-Completed
-Failed
-Cancelled
-NeedsRetainedFollowup
-Invalidated
-```
-
-`NeedsRetainedFollowup` is a terminal ephemeral result, not a retained worker state. It promises no durable participant identity, creates no `continue_world_worker` route, and does not silently create a retained worker or durable conversational obligation. The host must make a new, explicit, policy-checked `spawn_world_worker` decision if ongoing work is warranted.
+Canonical content: [`contracts/active-ephemeral-task-receipt-v1.md#3-activeephemeraltaskreceiptv1`](contracts/active-ephemeral-task-receipt-v1.md#3-activeephemeraltaskreceiptv1).
&#32;
 ## 4. `ActiveRetainedTurnReceiptV1`
&#32;
-```rust
-struct ActiveRetainedTurnReceiptV1 {
-    schema_version: u32,
-    acceptance_record_id: String,
-    active_run_id: String,
-    request_id: String,
-    orchestration_session_id: String,
-    orchestrator_participant_id: String,
-    target_participant_id: String,
-    target_backend_id: String,
-    world_id: String,
-    world_generation: u64,
-    message_id: String,
-    thread_id: Option<String>,
-    worker_policy_cap_hash: String,
-    turn_policy_snapshot_ref: PolicySnapshotRefV1,
-    turn_policy_snapshot_hash: String,
-    turn_policy_revision: String,
-    narrowing_reason: Option<String>,
-    runtime_acceptance: RuntimeAcceptanceEvidenceV1,
-    observation_claim: SupervisorObservationClaimV1,
-    accepted_at: Timestamp,
-    state_revision: u64,
-    state: ActiveRetainedTurnStateV1,
-    cancel_supported: bool,
-    terminal: Option<WorldWorkTerminalV1>,
-}
-```
-
-States:
-
-```text
-Accepted -> Running
-Running -> AttentionPending -> Running
-Accepted|Running|AttentionPending -> Parked|Terminal|Failed|Cancelled|Stopped
-```
-
-Rules:
-
-1. One retained worker may have at most one active cancelable turn unless a later version explicitly models concurrency.
-2. `Parked` closes the active turn but preserves the retained worker manifest and resume handle.
-3. Host posture is not copied from turn state. Obligations may cause host `AwaitingAttention`; worker `AttentionPending` remains worker truth.
+Canonical content: [`contracts/active-retained-turn-receipt-v1.md#4-activeretainedturnreceiptv1`](contracts/active-retained-turn-receipt-v1.md#4-activeretainedturnreceiptv1).
&#32;
 ## 5. Receipt acceptance source
&#32;
diff --git a/llm-last-mile/runtime-refactor/index/README.md b/llm-last-mile/runtime-refactor/index/README.md
index 9cf2a1f07..00c862d7f 100644
--- a/llm-last-mile/runtime-refactor/index/README.md
+++ b/llm-last-mile/runtime-refactor/index/README.md
@@ -19,6 +19,8 @@
 | `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` | gate | [`gates/authority-required-runtime-refactor-reentry.md`](../gates/authority-required-runtime-refactor-reentry.md) | closed documentation/control-plane selection gate | Closed as selection history; A1.3-P1 is the separate active packet. | [`04`](../04-contracts-and-gates.md#authority_requiredruntime_refactor_reentry-contract-2026-08-20-closed-selection-record) |
 | `HostExecutionEpisodeV1` | contract | [`contracts/host-execution-episode-v1.md`](../contracts/host-execution-episode-v1.md) | `canonical extracted contract owner for the complete schema, episode-kind literals, transport-status literals, and rules 1–6` | Supersedes only root-canonical ownership of the extracted `HostExecutionEpisodeV1` span; the committed pre-span compatibility anchors and neighboring `2A`/`2B` owners remain unchanged. | [`04`](../04-contracts-and-gates.md#2-hostexecutionepisodev1) |
 | `Deferred retained-spawn admission recovery contract` | contract | [`contracts/deferred-retained-spawn-admission-recovery-contract.md`](../contracts/deferred-retained-spawn-admission-recovery-contract.md) | `canonical extracted owner for the complete deferred retained-spawn admission recovery contract, paired V1 recovery schemas, compatibility boundaries, route exclusions, allowlists, and recorded-result limits` | Supersedes only root-canonical ownership of the extracted `Deferred retained-spawn admission recovery contract` span; the two prerequisite compatibility anchors, `HostExecutionEpisodeV1`, and the following `B3.2a-WA ExactBoundWorldOwnershipAdoptionV1` owner remain unchanged. | [`04`](../04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract) |
+| `ActiveEphemeralTaskReceiptV1` | contract | [`contracts/active-ephemeral-task-receipt-v1.md`](../contracts/active-ephemeral-task-receipt-v1.md) | `canonical extracted contract owner for the complete schema, state transitions, terminal monotonicity, result-class literals, and explicit NeedsRetainedFollowup constraints` | Supersedes only root-canonical ownership of the extracted `ActiveEphemeralTaskReceiptV1` span; `Receipt acceptance source` and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1) |
+| `ActiveRetainedTurnReceiptV1` | contract | [`contracts/active-retained-turn-receipt-v1.md`](../contracts/active-retained-turn-receipt-v1.md) | `canonical extracted contract owner for the complete schema, state transitions, and rules 1–3 covering the single active cancelable turn, Parked continuity, and the host/worker posture boundary` | Supersedes only root-canonical ownership of the extracted `ActiveRetainedTurnReceiptV1` span; `Receipt acceptance source` and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#4-activeretainedturnreceiptv1) |
 | `shared-target-architecture` | architecture index | [`architecture/README.md`](../architecture/README.md) | `non-authoritative navigation for the extracted executive target decision, authority map, numbered invariants, and stable review question` | Supersedes only root-canonical ownership of the extracted shared architecture spans; packet-family-local D5/D6 forwarders remain at their existing owners. | [`01`](../01-target-architecture.md#executive-decision), [`01`](../01-target-architecture.md#authority-map), [`01`](../01-target-architecture.md#non-negotiable-invariants), [`01`](../01-target-architecture.md#review-question) |
 | `shared-seam-crosswalk` | seam index | [`seams/README.md`](../seams/README.md) | `canonical shared seam-crosswalk rules plus extracted seam-family navigation for host/session, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation` | Supersedes only the extracted root reading-rule/classification spans and the extracted host/session authority, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation family rows; A0 and the existing D6 `HostSessionAuthority`, `WorldWorkerMessagingProtocol`, and `ObligationLedger` compatibility rows remain at their current owners. | [`02`](../02-seam-crosswalk.md#reading-rule), [`02`](../02-seam-crosswalk.md#a-host-authority-and-ingress), [`02`](../02-seam-crosswalk.md#b-dispatch-policy-receipts-and-retained-runtime), [`02`](../02-seam-crosswalk.md#c-obligations-and-host-re-engagement), [`02`](../02-seam-crosswalk.md#d-uaa-realization-projection-and-side-effect-mediation), [`02`](../02-seam-crosswalk.md#classification-consequences) |
 | `shared-slice-map` | slice index | [`slices/README.md`](../slices/README.md) | `canonical shared sequencing/dependency and slice-closeout owner plus non-authoritative D9 Track A–E navigation and A1/A1.4 projection index` | Supersedes only root-canonical ownership of the extracted shared sequencing/closeout spans plus the extracted Track A–E and A1/A1.4 projection spans; controlling schedule authority remains with decisions, packets, gates, and `current.md`. | [`03`](../03-phase-slice-map.md#sequencing-rules), [`03`](../03-phase-slice-map.md#track-a--authority-and-surface-neutrality), [`03`](../03-phase-slice-map.md#a1-bounded-packet-decomposition), [`03`](../03-phase-slice-map.md#track-b--world-dispatch-receipts-supervision-and-cancel), [`03`](../03-phase-slice-map.md#track-c--obligations-inbox-auto-attach-and-router-attach), [`03`](../03-phase-slice-map.md#track-d--uaa-execution-envelope-and-side-effect-mediation), [`03`](../03-phase-slice-map.md#track-e--dispatch-scoped-policy-narrowing-and-config-projection), [`03`](../03-phase-slice-map.md#slice-closeout-minimum) |
diff --git a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
index 619c1826c..16fd610fc 100644
--- a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
+++ b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
@@ -186,3 +186,5 @@ This ledger records content-preserving authority transfers while retaining requi
 | D9 | [`03-phase-slice-map.md`](../03-phase-slice-map.md) | A1.4 — bounded auto-attach producer adoption and regression closure | [`a1-bounded-packet-decomposition`](../03-phase-slice-map.md#a1-bounded-packet-decomposition) | line 145 | `58d7982b23a7b2f5f41f3e0b5170996e0f3b331dfbbcbb5875d02e801bd5d408` | [`../slices/tasks/a1-4-auto-attach-producer-adoption.md`](../slices/tasks/a1-4-auto-attach-producer-adoption.md) | task row | canonical destination; source compatibility table row | none | restore the exact extracted D9 source bodies/rows in `03-phase-slice-map.md`; remove `slices/` and all D9-added files beneath it; remove the `shared-slice-map` index row from `index/README.md`; remove all matching D9 ledger entries; leave D5–D8 canonical owners, `index/current.md`, and `review-control/` unchanged |
 | D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 2. `HostExecutionEpisodeV1` | [`2-hostexecutionepisodev1`](../04-contracts-and-gates.md#2-hostexecutionepisodev1) | lines 51–100 | `28e86e2e7e29b458280875f921ca327a469621ac96bd7e592a0a176027aa0117` | [`../contracts/host-execution-episode-v1.md`](../contracts/host-execution-episode-v1.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1645-byte source span at `04-contracts-and-gates.md#2-hostexecutionepisodev1`; remove `contracts/host-execution-episode-v1.md`; remove the exact `HostExecutionEpisodeV1` row from `index/README.md`; remove this D10 ledger entry; leave the committed pre-span compatibility anchors, D3/D5–D9 owners, `index/current.md`, `review-control/`, all other D10 units, and every path outside the four-path fence unchanged |
 | D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | Deferred retained-spawn admission recovery contract | [`deferred-retained-spawn-admission-recovery-contract`](../04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract) | lines 71–213 | `d903299bdd345316fd0943d28abc300a1cef001ce0744746f72f53607eb64c12` | [`../contracts/deferred-retained-spawn-admission-recovery-contract.md`](../contracts/deferred-retained-spawn-admission-recovery-contract.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 9896-byte source span at `04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract`; remove `contracts/deferred-retained-spawn-admission-recovery-contract.md`; remove the exact `Deferred retained-spawn admission recovery contract` row from `index/README.md`; remove this D10 ledger entry; leave `HostExecutionEpisodeV1`, the two prerequisite compatibility anchors, all D3/D5–D9 owners, `index/current.md`, `review-control/`, later D10 units, and every path outside the four-path fence unchanged |
+| D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 3. `ActiveEphemeralTaskReceiptV1` | [`3-activeephemeraltaskreceiptv1`](../04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1) | lines 91–138 | `c997752654f041de820210f03c86706551b44389994df5e8e0341aa41a34fa3b` | [`../contracts/active-ephemeral-task-receipt-v1.md`](../contracts/active-ephemeral-task-receipt-v1.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1557-byte source span at `04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1` and the exact 1443-byte source span at `04-contracts-and-gates.md#4-activeretainedturnreceiptv1`; remove `contracts/active-ephemeral-task-receipt-v1.md` and `contracts/active-retained-turn-receipt-v1.md`; remove the exact `ActiveEphemeralTaskReceiptV1` and `ActiveRetainedTurnReceiptV1` rows from `index/README.md`; remove both D10 ledger entries for this batch; leave `HostExecutionEpisodeV1`, deferred retained-spawn admission recovery, `Receipt acceptance source`, all D3/D5–D9 owners, `index/current.md`, `review-control/`, remaining D10 units, and every path outside the five-path fence unchanged |
+| D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 4. `ActiveRetainedTurnReceiptV1` | [`4-activeretainedturnreceiptv1`](../04-contracts-and-gates.md#4-activeretainedturnreceiptv1) | lines 139–183 | `2cc996111d82d296f89bdd88f80df5c0b559bdb305aec546881b749ace93f67e` | [`../contracts/active-retained-turn-receipt-v1.md`](../contracts/active-retained-turn-receipt-v1.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1557-byte source span at `04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1` and the exact 1443-byte source span at `04-contracts-and-gates.md#4-activeretainedturnreceiptv1`; remove `contracts/active-ephemeral-task-receipt-v1.md` and `contracts/active-retained-turn-receipt-v1.md`; remove the exact `ActiveEphemeralTaskReceiptV1` and `ActiveRetainedTurnReceiptV1` rows from `index/README.md`; remove both D10 ledger entries for this batch; leave `HostExecutionEpisodeV1`, deferred retained-spawn admission recovery, `Receipt acceptance source`, all D3/D5–D9 owners, `index/current.md`, `review-control/`, remaining D10 units, and every path outside the five-path fence unchanged |
diff --git a/llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md b/llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md
new file mode 100644
index 000000000..af6da98aa
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md
@@ -0,0 +1,56 @@
+**Kind:** contract
+**Status:** canonical
+**Canonical for:** complete extracted `ActiveEphemeralTaskReceiptV1` schema, state transitions, terminal monotonicity, explicit result-class literal order, and `NeedsRetainedFollowup` negative requirements
+**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1`](../04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1), baseline lines 91–138; the exact 1557-byte source body is preserved between the boundary markers below
+**Baseline span SHA-256:** `c997752654f041de820210f03c86706551b44389994df5e8e0341aa41a34fa3b`
+
+<!-- exact-extracted-body:start -->
+## 3. `ActiveEphemeralTaskReceiptV1`
+
+```rust
+struct ActiveEphemeralTaskReceiptV1 {
+    schema_version: u32,
+    acceptance_record_id: String,
+    task_run_id: String,
+    request_id: String,
+    orchestration_session_id: String,
+    caller_participant_id: String,
+    target_backend_id: String,
+    world_id: String,
+    world_generation: u64,
+    policy_snapshot_ref: PolicySnapshotRefV1,
+    policy_snapshot_hash: String,
+    policy_revision: String,
+    narrowing_reason: Option<String>,
+    runtime_acceptance: RuntimeAcceptanceEvidenceV1,
+    observation_claim: SupervisorObservationClaimV1,
+    accepted_at: Timestamp,
+    state_revision: u64,
+    state: ActiveTaskStateV1,
+    cancel_supported: bool,
+    terminal: Option<WorldWorkTerminalV1>,
+}
+```
+
+States:
+
+```text
+Accepted -> Running -> AttentionPending -> Running
+Accepted|Running|AttentionPending -> Terminal|Failed|Cancelled|Invalidated
+```
+
+`Terminal`, `Failed`, `Cancelled`, and `Invalidated` are terminal and monotonic. `Parked` is not valid for an ephemeral task.
+
+`WorldWorkTerminalV1` for an ephemeral task carries one explicit result class:
+
+```text
+Completed
+Failed
+Cancelled
+NeedsRetainedFollowup
+Invalidated
+```
+
+`NeedsRetainedFollowup` is a terminal ephemeral result, not a retained worker state. It promises no durable participant identity, creates no `continue_world_worker` route, and does not silently create a retained worker or durable conversational obligation. The host must make a new, explicit, policy-checked `spawn_world_worker` decision if ongoing work is warranted.
+
+<!-- exact-extracted-body:end -->
diff --git a/llm-last-mile/runtime-refactor/contracts/active-retained-turn-receipt-v1.md b/llm-last-mile/runtime-refactor/contracts/active-retained-turn-receipt-v1.md
new file mode 100644
index 000000000..c4d218fab
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/contracts/active-retained-turn-receipt-v1.md
@@ -0,0 +1,53 @@
+**Kind:** contract
+**Status:** canonical
+**Canonical for:** complete extracted `ActiveRetainedTurnReceiptV1` schema, state transitions, and rules 1–3 covering the single active cancelable turn, `Parked` continuity, and the host/worker posture boundary
+**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#4-activeretainedturnreceiptv1`](../04-contracts-and-gates.md#4-activeretainedturnreceiptv1), baseline lines 139–183; the exact 1443-byte source body is preserved between the boundary markers below
+**Baseline span SHA-256:** `2cc996111d82d296f89bdd88f80df5c0b559bdb305aec546881b749ace93f67e`
+
+<!-- exact-extracted-body:start -->
+## 4. `ActiveRetainedTurnReceiptV1`
+
+```rust
+struct ActiveRetainedTurnReceiptV1 {
+    schema_version: u32,
+    acceptance_record_id: String,
+    active_run_id: String,
+    request_id: String,
+    orchestration_session_id: String,
+    orchestrator_participant_id: String,
+    target_participant_id: String,
+    target_backend_id: String,
+    world_id: String,
+    world_generation: u64,
+    message_id: String,
+    thread_id: Option<String>,
+    worker_policy_cap_hash: String,
+    turn_policy_snapshot_ref: PolicySnapshotRefV1,
+    turn_policy_snapshot_hash: String,
+    turn_policy_revision: String,
+    narrowing_reason: Option<String>,
+    runtime_acceptance: RuntimeAcceptanceEvidenceV1,
+    observation_claim: SupervisorObservationClaimV1,
+    accepted_at: Timestamp,
+    state_revision: u64,
+    state: ActiveRetainedTurnStateV1,
+    cancel_supported: bool,
+    terminal: Option<WorldWorkTerminalV1>,
+}
+```
+
+States:
+
+```text
+Accepted -> Running
+Running -> AttentionPending -> Running
+Accepted|Running|AttentionPending -> Parked|Terminal|Failed|Cancelled|Stopped
+```
+
+Rules:
+
+1. One retained worker may have at most one active cancelable turn unless a later version explicitly models concurrency.
+2. `Parked` closes the active turn but preserves the retained worker manifest and resume handle.
+3. Host posture is not copied from turn state. Obligations may cause host `AwaitingAttention`; worker `AttentionPending` remains worker truth.
+
+<!-- exact-extracted-body:end -->
```
``````

</details>

<details>
<summary>Initial-range review prompt exact bytes (base64 UTF-8)</summary>

```text
UmV2aWV3IG1vZGU6IGluaXRpYWwtcmFuZ2UKUmV2aWV3IGNvbnRleHQ6IGluZGVwZW5kZW50IGZyZXNoIGNvbnZlcnNhdGlvbgpSZXZpZXcgYm91bmRhcnk6IGV4YWN0IGJhc2VsaW5lIGNvbW1pdCA3NDBkY2RjMTQ0MWJjY2VjYWU4OTQ0YWQ3MjI5MDI1YzBmMjJkYTI0LCB0cmVlIDljNjJjYWE2NGFiYmUwNGZmZmEwZDAwNzM3ZGM0NWRlNWI3YTFiNjcsIHBsdXMgdGhlIGNvbXBsZXRlIGNvbnRlbnQtYWRkcmVzc2VkIGNhbmRpZGF0ZSBwYXRjaCBiZWxvdy4gQ2FuZGlkYXRlIHBhdGNoIFNIQS0yNTY6IDQzMzJhNTA5OGU5NzdiNWVkZGE5OTY3MDhiMGVmMjYxNDNkMjRlYmNlNmRlZWI5YmJlMTc4ZmU4ZjlkODk3NDguClJldmlldyB0YXJnZXQ6IHRoZSBjb21wbGV0ZSBib3VuZGVkIEQxMCBkb2N1bWVudGF0aW9uIGV4dHJhY3Rpb24gYmF0Y2ggZm9yIHRoZSBpbnNlcGFyYWJsZSBhY3RpdmUgcmVjZWlwdCBjb250cmFjdCBmYW1pbHkuIFJldmlldyBldmVyeSBodW5rIGFuZCBib3RoIG5ldyBmaWxlczsgbm8gdW5wcm92aWRlZCByZXBvc2l0b3J5IHNvdXJjZSBtYXkgYmUgaW5mZXJyZWQuCgpNYW5pZmVzdCAoZXhhY3RseSBmaXZlIGNoYW5nZWQgcGF0aHMsIGluY2x1ZGluZyB1bnRyYWNrZWQgbmV3IGZpbGVzKToKLSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCDigJQgU0hBLTI1NiA0N2YzZGE1NTAwYjNkYTg0M2YxZTg3YWNlY2Y4MWY1Yjg3MzNkMDc5MGQzZTMxOGI0MDEzNDg1Y2ZlZjViZGUwCi0gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2NvbnRyYWN0cy9hY3RpdmUtZXBoZW1lcmFsLXRhc2stcmVjZWlwdC12MS5tZCDigJQgbmV3IOKAlCBTSEEtMjU2IDVlMTA2ZDg4ZDE4MjY2NDFmZTFiMjZhODM4NzE3YzQzYmFmMzE3MWY2N2FlNGRmZjIxODFmODJlN2Q2ZjY4ODcKLSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvY29udHJhY3RzL2FjdGl2ZS1yZXRhaW5lZC10dXJuLXJlY2VpcHQtdjEubWQg4oCUIG5ldyDigJQgU0hBLTI1NiAzZTRiMTg3YWE2ZjdhOGQ1N2ZkZWVhYjU2M2U2ODY2ZDRhOTYyZmVjNjg1YzJiOWQ2Mjg1MjA5ZGQ5MWIyMmQwCi0gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZCDigJQgU0hBLTI1NiBkOTFhODgyYThhMTdhODIzZDE5NDIyNjA3YTJlZWNiMDQ5NDg5ZGY4OWMwOGI4OGJlMDYxNThiMGVlYTUxMGMxCi0gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZCDigJQgU0hBLTI1NiBlZTM4YjY5YjA5MTE5MmJjMjdhMzVmYzVmMWU0YjMxNjU3MjU2MjAxMzI1MGUzNzRiNzg2ZDI4ZWQ3YTIyNjUzCgpTdXBwb3J0ZWQgaW5wdXRzIGFuZCBiZWhhdmlvcjogR2l0SHViLWZsYXZvcmVkIE1hcmtkb3duIGRvY3VtZW50YXRpb24gd2l0aCBBVFggaGVhZGluZ3MgYW5kIGdlbmVyYXRlZCBoZWFkaW5nIGFuY2hvcnMsIHJlbGF0aXZlIE1hcmtkb3duIGxpbmtzIHdpdGggb3B0aW9uYWwgZnJhZ21lbnRzLCBmZW5jZWQgY29kZSBibG9ja3MsIE1hcmtkb3duIHRhYmxlcyB3aG9zZSByb3cgYW5kIGNlbGwgb3JkZXIgaXMgc2VtYW50aWMsIGlubGluZSBjb2RlL2xpdGVyYWxzLCBhbmQgSFRNTCBjb21tZW50cyB1c2VkIGFzIGV4dHJhY3Rpb24tYm9keSBoYXNoIG1hcmtlcnMuIFRoZSBsZWdhY3kgcm9vdCByZW1haW5zIGEgY29tcGF0aWJpbGl0eSBwcm9qZWN0aW9uIHVudGlsIGEgc2VwYXJhdGVseSBhdXRob3JpemVkIEQxMiBjdXRvdmVyLiBDYW5vbmljYWwgb3duZXJzIGFyZSBub3JtYXRpdmU7IHJvb3QgY29tcGF0aWJpbGl0eSBoZWFkaW5ncyBhbmQgYW5jaG9ycyBtdXN0IGNvbnRpbnVlIHRvIHJlc29sdmUuClN1cHBvcnRlZCBwbGF0Zm9ybXMgYW5kIGRpYWxlY3RzOiByZXBvc2l0b3J5IE1hcmtkb3duIHJlbmRlcmVkIHdpdGggR2l0SHViLXN0eWxlIGhlYWRpbmcgc2x1ZyBiZWhhdmlvciBhbmQgdmFsaWRhdGVkIGJ5IHRoZSBwcm9qZWN0J3MgTWFya2Rvd24gbGluay9hbmNob3IgY2hlY2tlci4gTm8gcnVudGltZSwgUnVzdCwgc2NyaXB0LCBwbGF0Zm9ybSwgcGFyc2VyLCBkaXNwYXRjaCwgb3IgaW1wbGVtZW50YXRpb24gYmVoYXZpb3IgaXMgaW4gc2NvcGUuCgpJbi1zY29wZSBpbnZhcmlhbnRzOgoxLiBFeGFjdGx5IG9uZSBjYW5vbmljYWwgb3duZXIgZXhpc3RzIGZvciBlYWNoIG9mIEFjdGl2ZUVwaGVtZXJhbFRhc2tSZWNlaXB0VjEgYW5kIEFjdGl2ZVJldGFpbmVkVHVyblJlY2VpcHRWMS4KMi4gRWFjaCBleHRyYWN0ZWQgY2Fub25pY2FsIGJvZHkgaXMgYnl0ZS1mb3ItYnl0ZSBpZGVudGljYWwgdG8gaXRzIGZyb3plbiBsaXZlLXJvb3Qgc291cmNlIHNwYW4sIGluY2x1ZGluZyBvcmRlcmVkIGZpZWxkcywgdGFibGUtY2VsbCBvcmRlciwgZmVuY2VkIHNjaGVtYXMvY29kZSwgbGl0ZXJhbHMsIGhhc2hlcywgY29tbWFuZHMsIHZhbGlkYXRpb24gc2VxdWVuY2UsIG5lZ2F0aXZlIHJlcXVpcmVtZW50cywgZXhjbHVzaW9ucywgZXhjZXB0aW9uIHN0YXRlcywgdmVyc2lvbiBzdWZmaXhlcywgc2VtYW50aWMgc3RhdHVzLCBhbmQgYXV0aG9yaXR5IGJvdW5kYXJpZXMuCjMuIEZyb3plbiBzb3VyY2UgYXJ0aWZhY3QgZmFjdHM6IEFjdGl2ZUVwaGVtZXJhbFRhc2tSZWNlaXB0VjEgYm9keSBpcyAxNTU3IGJ5dGVzIHdpdGggU0hBLTI1NiBjOTk3NzUyNjU0ZjA0MWRlODIwMjEwZjAzYzg2NzA2NTUxYjQ0Mzg5OTk0ZGY1ZThlMDM0MWFhNDFhMzRmYTNiOyBBY3RpdmVSZXRhaW5lZFR1cm5SZWNlaXB0VjEgYm9keSBpcyAxNDQzIGJ5dGVzIHdpdGggU0hBLTI1NiAyY2M5OTYxMTFkODJkMjk2Zjg5YmRkODhmODBkZjVjMGI1NTliZGIzMDVhZWM1NDY4ODFiNzQ5YWNlOTNmNjdlLgo0LiBUaGUgcm9vdCBtdXN0IHJldGFpbiB0aGUgZXhhY3QgbGVnYWN5IGhlYWRpbmdzL2FuY2hvcnMgYDMtYWN0aXZlZXBoZW1lcmFsdGFza3JlY2VpcHR2MWAgYW5kIGA0LWFjdGl2ZXJldGFpbmVkdHVybnJlY2VpcHR2MWAsIHJlcGxhY2luZyB0aGVpciBkdXBsaWNhdGVkIGJvZGllcyBvbmx5IHdpdGggdHJ1dGhmdWwgbGlua3MgdG8gdGhlIGNhbm9uaWNhbCBvd25lcnMuCjUuIE5hdmlnYXRpb24gbXVzdCBjb250YWluIGV4YWN0bHkgb25lIHJvdyBmb3IgZWFjaCBuZXcgb3duZXIsIHdpdGggY29ycmVjdCByZWxhdGl2ZSBsaW5rcyBhbmQgbm8gYnJva2VuIGxpbmtzIG9yIGFuY2hvcnMuCjYuIEV4dHJhY3Rpb24tbGVkZ2VyIHByb3ZlbmFuY2UgbXVzdCBjb250YWluIGV4YWN0bHkgb25lIHRydXRoZnVsIEQxMCByb3cgcGVyIG93bmVyLCBpbmNsdWRpbmcgZXhhY3Qgc291cmNlIHNwYW4vaGFzaCBhbmQgY29tcGxldGUgYXRvbWljIHJvbGxiYWNrIGluc3RydWN0aW9ucyB0aGF0IHJlc3RvcmUgYm90aCByb290IGJvZGllcyBhbmQgcmVtb3ZlIGJvdGggb3duZXJzL25hdmlnYXRpb24vbGVkZ2VyIHJvd3MgdG9nZXRoZXIuCjcuIFRoZSB0d28gY29udHJhY3RzIGFyZSBvbmUgYXRvbWljIHZlcnNpb24gZmFtaWx5OyByb2xsYmFjayBtdXN0IG5vdCBzdHJhbmQgb25seSBvbmUgcmVjZWlwdCBvd25lci4KOC4gRXhpc3RpbmcgcmVjZWlwdC1hY2NlcHRhbmNlLXNvdXJjZSBjb21wYXRpYmlsaXR5IG93bmVyc2hpcCBhbmQgYWxsIHByaW9yIEQzL0Q1LUQxMCBvd25lcnMvcmV2aWV3IGFydGlmYWN0cyBtdXN0IHJlbWFpbiBzdGFibGUuCjkuIFRoZSBwYXRjaCBtdXN0IG5vdCBjbGFpbSBpbXBsZW1lbnRhdGlvbiwgZGlzcGF0Y2gsIGdhdGUgc2F0aXNmYWN0aW9uLCBzdWNjZXNzb3IgYXV0aG9yaXR5LCBEMTAgY29tcGxldGlvbiwgRDExIGF1dGhvcml0eSwgb3IgRDEyIGN1dG92ZXIuCjEwLiBUaGUgZXhhY3QgZml2ZS1wYXRoIGZlbmNlIGlzIG1hbmRhdG9yeTsgbm8gZm9ybWF0dGluZyBub3JtYWxpemF0aW9uIG9yIHVucmVsYXRlZCBlZGl0cyBhcmUgYWxsb3dlZC4KClJlcXVpcmVkIG1hdGVyaWFsIGNvbnNlcXVlbmNlOiBhIGZpbmRpbmcgYmxvY2tzIG9ubHkgaWYgdGhlIGNvbXBsZXRlIHBhdGNoIG1ha2VzIGEgc3VwcG9ydGVkIHJlcG9zaXRvcnkgcmVhZGVyIG1hdGVyaWFsbHkgbG9zZSwgYWx0ZXIsIGR1cGxpY2F0ZSwgb3IgbWlzaWRlbnRpZnkgY29udHJhY3Qgc2VtYW50aWNzIG9yIGNhbm9uaWNhbCBvd25lcnNoaXA7IGJyZWFrcyBhIHN1cHBvcnRlZCBsaW5rL2xlZ2FjeSBhbmNob3IvcHJvdmVuYW5jZS9hdG9taWMgcm9sbGJhY2s7IGZhbHNpZmllcyBzdGF0dXMgb3IgYXV0aG9yaXR5OyB2aW9sYXRlcyB0aGUgZXhhY3QgcGF0aCBmZW5jZTsgb3IgcHJldmVudHMgY2xlYW4gYXBwbGljYXRpb24vcmV2ZXJzYWwuIFB1cmUgc3R5bGUgcHJlZmVyZW5jZXMsIG9wdGlvbmFsIHByb3NlLCBicm9hZGVyIHJlZGVzaWducywgdGhlb3JldGljYWwgdW5zdXBwb3J0ZWQgTWFya2Rvd24gYmVoYXZpb3IsIG9yIHVuY2hhbmdlZCBiYXNlbGluZSBkZWZlY3RzIGFyZSBub3QgbWF0ZXJpYWwuCkJsb2NraW5nIHRocmVzaG9sZDogcmVwb3J0IG9ubHkgcmVhY2hhYmxlLCBwYXRjaC1jYXVzYWwgUDEgb3IgUDIgZGVmZWN0cyB0aGF0IHZpb2xhdGUgYW4gaW4tc2NvcGUgaW52YXJpYW50IGFuZCBtZWV0IHRoZSBtYXRlcmlhbC1jb25zZXF1ZW5jZSB0aHJlc2hvbGQuIEEgY29tcGxldGUgYm9keS9oYXNoIG1pc21hdGNoLCB3cm9uZyBvd25lciBvciB0YXJnZXQsIGJyb2tlbiByZXF1aXJlZCBhbmNob3IvbGluaywgaW5jb21wbGV0ZSByb2xsYmFjaywgZmFsc2UgYXV0aG9yaXR5L3N0YXR1cywgb3Igc2NvcGUgYnJlYWNoIHF1YWxpZmllcy4gTm9uLWJsb2NraW5nIG9ic2VydmF0aW9ucyBtdXN0IGJlIGNsZWFybHkgbGFiZWxlZCBkZWZlcnJlZCBhbmQgbXVzdCBub3QgY2hhbmdlIHRoZSB2ZXJkaWN0LgpBY2NlcHRlZCBwcmlvciBmaW5kaW5nczogbm9uZS4KRGVmZXJyZWQgb3Igb3V0LW9mLXNjb3BlIGNvbmNlcm5zOiBEMyByZXZpZXctZ292ZXJuYW5jZTsgcGFja2V0LWxvY2FsIEQ1L0Q2IGNvbnRlbnQgYWxyZWFkeSBjYW5vbmljYWxseSBvd25lZDsgRDkgc2xpY2VzL3Rhc2tzOyBEMTEgZXZpZGVuY2UgZGVjb21wb3NpdGlvbjsgRDEyIHJvb3QgY3V0b3ZlcjsgUnVzdC9zY3JpcHRzL3J1bnRpbWUvcGxhdGZvcm0gYmVoYXZpb3I7IHJvYWRtYXAgb3IgcmV2aWV3ZWQgWklQIGNoYW5nZXM7IGdsb2JhbCBNYXJrZG93biBmb3JtYXR0aW5nOyB1bnJlbGF0ZWQgcHJlLWV4aXN0aW5nIGlzc3Vlczsgc3R5bGUtb25seSBpbXByb3ZlbWVudHM7IGltcGxlbWVudGF0aW9uIG9yIHN1Y2Nlc3NvciBkaXNwYXRjaC4KUHJvamVjdCBzb3VyY2VzOiBvbmx5IHRoaXMgcHJvbXB0LCBtYW5pZmVzdCwgaW52YXJpYW50cywgdmFsaWRhdGlvbiBldmlkZW5jZSwgYW5kIHRoZSBjb21wbGV0ZSBwYXRjaCBiZWxvdyBhcmUgYXZhaWxhYmxlLiBFeGNsdWRlZCBzb3VyY2UgaXMgdW5hdmFpbGFibGUgYW5kIG11c3Qgbm90IGJlIGluZmVycmVkLgoKVmFsaWRhdGlvbiBldmlkZW5jZToKLSBleGFjdCBmaXZlLXBhdGggZmVuY2UgaW5jbHVkaW5nIHVudHJhY2tlZCBmaWxlczogUEFTUwotIG5vIHN0YWdlZCBmaWxlczogUEFTUwotIGBnaXQgZGlmZiAtLWNoZWNrYDogUEFTUwotIGBnaXQgZGlmZiAtLW5vLWluZGV4IC0tY2hlY2sgL2Rldi9udWxsIDxuZXctZmlsZT5gIGZvciBib3RoIG5ldyBvd25lcnM6IHplcm8gZGlhZ25vc3RpY3MgKHRoZSBjb21tYW5kJ3Mgb3JkaW5hcnkgZGlmZmVyZW5jZSBleGl0IHN0YXR1cyBpcyBub3QgYSB3aGl0ZXNwYWNlIGZhaWx1cmUpCi0gTWFya2Rvd24gbGluay9hbmNob3IgdmFsaWRhdGlvbjogMSw4MzMgbGlua3MgY2hlY2tlZCwgMCBmYWlsdXJlcwotIGJvdGggcm9vdCBsZWdhY3kgYW5jaG9ycyBhbmQgYm90aCBvd25lciBhbmNob3JzOiB1bmlxdWUgYW5kIHJlc29sdmluZwotIGV4dHJhY3RlZCBtYXJrZXItYm9keSBlcXVhbGl0eTogMTU1Ny8xNTU3IGJ5dGVzIGFuZCBjOTk3Li4uIGhhc2ggZXF1YWw7IDE0NDMvMTQ0MyBieXRlcyBhbmQgMmNjOS4uLiBoYXNoIGVxdWFsCi0gcm9vdCBjYW5vbmljYWwgcG9pbnRlcnMsIGV4YWN0bHkgb25lIGluZGV4IHJvdyBlYWNoLCBhbmQgZXhhY3RseSBvbmUgbGVkZ2VyIHJvdyBlYWNoOiBQQVNTCi0gcmVjZWlwdC1hY2NlcHRhbmNlLXNvdXJjZSBjb21wYXRpYmlsaXR5IHBvaW50ZXI6IHVuY2hhbmdlZAotIHByaW9yIEQxMCBvd25lcnMgYW5kIGFsbCBwYXRocyBvdXRzaWRlIHRoZSBtYW5pZmVzdDogc3RhYmxlCi0gcGF0Y2ggZm9yd2FyZC1hcHBseSBhbmQgZXhhY3QgZml2ZS1maWxlIGNvbnRlbnQgbWF0Y2ggZnJvbSBjbGVhbiBiYXNlbGluZTogUEFTUwotIHBhdGNoIHJldmVyc2UtYXBwbHkgYW5kIGV4YWN0IGNsZWFuLWJhc2VsaW5lIHJlc3RvcmF0aW9uOiBQQVNTCi0gY2FuZGlkYXRlIHZhbGlkYXRpb24gbG9nIFNIQS0yNTY6IDEzNjEwZmMzMjJhMzY2ZTQ5NjY4N2E1MWI2ZjZiZDliYmUxMzA3YTk1NmFjMzQzOGJiM2RmMWJhOTQ4NTViZmYKCkRvIG5vdCByZWR1Y2UgdGhlIHRhc2sgb3IgcmVwbGFjZSBpdCB3aXRoIGFuIGVhc2llciBhbHRlcm5hdGl2ZS4gUHJlc2VydmUgdGhlIHJlcXVlc3RlZCBzY29wZSBhbmQgcHJvamVjdCBjb252ZW50aW9ucy4KClJldHVybiBmaW5kaW5ncyBmaXJzdCBieSBzZXZlcml0eS4gRm9yIGV2ZXJ5IGJsb2NraW5nIGZpbmRpbmcsIHN0YXRlIHRoZSBleGFjdCBmaWxlL2h1bmssIHN1cHBvcnRlZC1pbnB1dCByZWFjaGFiaWxpdHksIHZpb2xhdGVkIGludmFyaWFudCwgbWF0ZXJpYWwgY29uc2VxdWVuY2UsIGFuZCBwYXRjaCBjYXVzYWxpdHkuIFN0YXRlIGV4cGxpY2l0bHkgd2hlbiB0aGVyZSBhcmUgbm8gcXVhbGlmeWluZyBmaW5kaW5ncy4gRW5kIHdpdGggZXhhY3RseSBvbmUgb2Y6ClZFUkRJQ1Q6IEFQUFJPVkVEClZFUkRJQ1Q6IENIQU5HRVMgUkVRVUlSRUQKCkFkdmlzb3J5IG9ubHk7IHZlcmlmeSBhZ2FpbnN0IGxvY2FsIHByb2plY3QgdHJ1dGggYW5kIGF1dGhvcml0YXRpdmUgZG9jczsgZG8gbm90IHJlZHVjZSBzY29wZSB3aXRob3V0IHVzZXIgYXBwcm92YWwuCgpDT01QTEVURSBDT05URU5ULUFERFJFU1NFRCBQQVRDSCAoU0hBLTI1NiA0MzMyYTUwOThlOTc3YjVlZGRhOTk2NzA4YjBlZjI2MTQzZDI0ZWJjZTZkZWViOWJiZTE3OGZlOGY5ZDg5NzQ4KToKYGBgZGlmZgpkaWZmIC0tZ2l0IGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZAppbmRleCA3YjgwYmFjNTkuLmE5ZTdjMzJmMCAxMDA2NDQKLS0tIGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQKKysrIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQKQEAgLTkwLDk2ICs5MCwxMSBAQCBDYW5vbmljYWwgY29udGVudDogW2BiMS1iMi0xL2NvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjYjEtZnJvemVuLXByb3Bvc2FsLXBlcnNpc3RlbgogCiAjIyAzLiBgQWN0aXZlRXBoZW1lcmFsVGFza1JlY2VpcHRWMWAKIAotYGBgcnVzdAotc3RydWN0IEFjdGl2ZUVwaGVtZXJhbFRhc2tSZWNlaXB0VjEgewotICAgIHNjaGVtYV92ZXJzaW9uOiB1MzIsCi0gICAgYWNjZXB0YW5jZV9yZWNvcmRfaWQ6IFN0cmluZywKLSAgICB0YXNrX3J1bl9pZDogU3RyaW5nLAotICAgIHJlcXVlc3RfaWQ6IFN0cmluZywKLSAgICBvcmNoZXN0cmF0aW9uX3Nlc3Npb25faWQ6IFN0cmluZywKLSAgICBjYWxsZXJfcGFydGljaXBhbnRfaWQ6IFN0cmluZywKLSAgICB0YXJnZXRfYmFja2VuZF9pZDogU3RyaW5nLAotICAgIHdvcmxkX2lkOiBTdHJpbmcsCi0gICAgd29ybGRfZ2VuZXJhdGlvbjogdTY0LAotICAgIHBvbGljeV9zbmFwc2hvdF9yZWY6IFBvbGljeVNuYXBzaG90UmVmVjEsCi0gICAgcG9saWN5X3NuYXBzaG90X2hhc2g6IFN0cmluZywKLSAgICBwb2xpY3lfcmV2aXNpb246IFN0cmluZywKLSAgICBuYXJyb3dpbmdfcmVhc29uOiBPcHRpb248U3RyaW5nPiwKLSAgICBydW50aW1lX2FjY2VwdGFuY2U6IFJ1bnRpbWVBY2NlcHRhbmNlRXZpZGVuY2VWMSwKLSAgICBvYnNlcnZhdGlvbl9jbGFpbTogU3VwZXJ2aXNvck9ic2VydmF0aW9uQ2xhaW1WMSwKLSAgICBhY2NlcHRlZF9hdDogVGltZXN0YW1wLAotICAgIHN0YXRlX3JldmlzaW9uOiB1NjQsCi0gICAgc3RhdGU6IEFjdGl2ZVRhc2tTdGF0ZVYxLAotICAgIGNhbmNlbF9zdXBwb3J0ZWQ6IGJvb2wsCi0gICAgdGVybWluYWw6IE9wdGlvbjxXb3JsZFdvcmtUZXJtaW5hbFYxPiwKLX0KLWBgYAotCi1TdGF0ZXM6Ci0KLWBgYHRleHQKLUFjY2VwdGVkIC0+IFJ1bm5pbmcgLT4gQXR0ZW50aW9uUGVuZGluZyAtPiBSdW5uaW5nCi1BY2NlcHRlZHxSdW5uaW5nfEF0dGVudGlvblBlbmRpbmcgLT4gVGVybWluYWx8RmFpbGVkfENhbmNlbGxlZHxJbnZhbGlkYXRlZAotYGBgCi0KLWBUZXJtaW5hbGAsIGBGYWlsZWRgLCBgQ2FuY2VsbGVkYCwgYW5kIGBJbnZhbGlkYXRlZGAgYXJlIHRlcm1pbmFsIGFuZCBtb25vdG9uaWMuIGBQYXJrZWRgIGlzIG5vdCB2YWxpZCBmb3IgYW4gZXBoZW1lcmFsIHRhc2suCi0KLWBXb3JsZFdvcmtUZXJtaW5hbFYxYCBmb3IgYW4gZXBoZW1lcmFsIHRhc2sgY2FycmllcyBvbmUgZXhwbGljaXQgcmVzdWx0IGNsYXNzOgotCi1gYGB0ZXh0Ci1Db21wbGV0ZWQKLUZhaWxlZAotQ2FuY2VsbGVkCi1OZWVkc1JldGFpbmVkRm9sbG93dXAKLUludmFsaWRhdGVkCi1gYGAKLQotYE5lZWRzUmV0YWluZWRGb2xsb3d1cGAgaXMgYSB0ZXJtaW5hbCBlcGhlbWVyYWwgcmVzdWx0LCBub3QgYSByZXRhaW5lZCB3b3JrZXIgc3RhdGUuIEl0IHByb21pc2VzIG5vIGR1cmFibGUgcGFydGljaXBhbnQgaWRlbnRpdHksIGNyZWF0ZXMgbm8gYGNvbnRpbnVlX3dvcmxkX3dvcmtlcmAgcm91dGUsIGFuZCBkb2VzIG5vdCBzaWxlbnRseSBjcmVhdGUgYSByZXRhaW5lZCB3b3JrZXIgb3IgZHVyYWJsZSBjb252ZXJzYXRpb25hbCBvYmxpZ2F0aW9uLiBUaGUgaG9zdCBtdXN0IG1ha2UgYSBuZXcsIGV4cGxpY2l0LCBwb2xpY3ktY2hlY2tlZCBgc3Bhd25fd29ybGRfd29ya2VyYCBkZWNpc2lvbiBpZiBvbmdvaW5nIHdvcmsgaXMgd2FycmFudGVkLgorQ2Fub25pY2FsIGNvbnRlbnQ6IFtgY29udHJhY3RzL2FjdGl2ZS1lcGhlbWVyYWwtdGFzay1yZWNlaXB0LXYxLm1kIzMtYWN0aXZlZXBoZW1lcmFsdGFza3JlY2VpcHR2MWBdKGNvbnRyYWN0cy9hY3RpdmUtZXBoZW1lcmFsLXRhc2stcmVjZWlwdC12MS5tZCMzLWFjdGl2ZWVwaGVtZXJhbHRhc2tyZWNlaXB0djEpLgogCiAjIyA0LiBgQWN0aXZlUmV0YWluZWRUdXJuUmVjZWlwdFYxYAogCi1gYGBydXN0Ci1zdHJ1Y3QgQWN0aXZlUmV0YWluZWRUdXJuUmVjZWlwdFYxIHsKLSAgICBzY2hlbWFfdmVyc2lvbjogdTMyLAotICAgIGFjY2VwdGFuY2VfcmVjb3JkX2lkOiBTdHJpbmcsCi0gICAgYWN0aXZlX3J1bl9pZDogU3RyaW5nLAotICAgIHJlcXVlc3RfaWQ6IFN0cmluZywKLSAgICBvcmNoZXN0cmF0aW9uX3Nlc3Npb25faWQ6IFN0cmluZywKLSAgICBvcmNoZXN0cmF0b3JfcGFydGljaXBhbnRfaWQ6IFN0cmluZywKLSAgICB0YXJnZXRfcGFydGljaXBhbnRfaWQ6IFN0cmluZywKLSAgICB0YXJnZXRfYmFja2VuZF9pZDogU3RyaW5nLAotICAgIHdvcmxkX2lkOiBTdHJpbmcsCi0gICAgd29ybGRfZ2VuZXJhdGlvbjogdTY0LAotICAgIG1lc3NhZ2VfaWQ6IFN0cmluZywKLSAgICB0aHJlYWRfaWQ6IE9wdGlvbjxTdHJpbmc+LAotICAgIHdvcmtlcl9wb2xpY3lfY2FwX2hhc2g6IFN0cmluZywKLSAgICB0dXJuX3BvbGljeV9zbmFwc2hvdF9yZWY6IFBvbGljeVNuYXBzaG90UmVmVjEsCi0gICAgdHVybl9wb2xpY3lfc25hcHNob3RfaGFzaDogU3RyaW5nLAotICAgIHR1cm5fcG9saWN5X3JldmlzaW9uOiBTdHJpbmcsCi0gICAgbmFycm93aW5nX3JlYXNvbjogT3B0aW9uPFN0cmluZz4sCi0gICAgcnVudGltZV9hY2NlcHRhbmNlOiBSdW50aW1lQWNjZXB0YW5jZUV2aWRlbmNlVjEsCi0gICAgb2JzZXJ2YXRpb25fY2xhaW06IFN1cGVydmlzb3JPYnNlcnZhdGlvbkNsYWltVjEsCi0gICAgYWNjZXB0ZWRfYXQ6IFRpbWVzdGFtcCwKLSAgICBzdGF0ZV9yZXZpc2lvbjogdTY0LAotICAgIHN0YXRlOiBBY3RpdmVSZXRhaW5lZFR1cm5TdGF0ZVYxLAotICAgIGNhbmNlbF9zdXBwb3J0ZWQ6IGJvb2wsCi0gICAgdGVybWluYWw6IE9wdGlvbjxXb3JsZFdvcmtUZXJtaW5hbFYxPiwKLX0KLWBgYAotCi1TdGF0ZXM6Ci0KLWBgYHRleHQKLUFjY2VwdGVkIC0+IFJ1bm5pbmcKLVJ1bm5pbmcgLT4gQXR0ZW50aW9uUGVuZGluZyAtPiBSdW5uaW5nCi1BY2NlcHRlZHxSdW5uaW5nfEF0dGVudGlvblBlbmRpbmcgLT4gUGFya2VkfFRlcm1pbmFsfEZhaWxlZHxDYW5jZWxsZWR8U3RvcHBlZAotYGBgCi0KLVJ1bGVzOgotCi0xLiBPbmUgcmV0YWluZWQgd29ya2VyIG1heSBoYXZlIGF0IG1vc3Qgb25lIGFjdGl2ZSBjYW5jZWxhYmxlIHR1cm4gdW5sZXNzIGEgbGF0ZXIgdmVyc2lvbiBleHBsaWNpdGx5IG1vZGVscyBjb25jdXJyZW5jeS4KLTIuIGBQYXJrZWRgIGNsb3NlcyB0aGUgYWN0aXZlIHR1cm4gYnV0IHByZXNlcnZlcyB0aGUgcmV0YWluZWQgd29ya2VyIG1hbmlmZXN0IGFuZCByZXN1bWUgaGFuZGxlLgotMy4gSG9zdCBwb3N0dXJlIGlzIG5vdCBjb3BpZWQgZnJvbSB0dXJuIHN0YXRlLiBPYmxpZ2F0aW9ucyBtYXkgY2F1c2UgaG9zdCBgQXdhaXRpbmdBdHRlbnRpb25gOyB3b3JrZXIgYEF0dGVudGlvblBlbmRpbmdgIHJlbWFpbnMgd29ya2VyIHRydXRoLgorQ2Fub25pY2FsIGNvbnRlbnQ6IFtgY29udHJhY3RzL2FjdGl2ZS1yZXRhaW5lZC10dXJuLXJlY2VpcHQtdjEubWQjNC1hY3RpdmVyZXRhaW5lZHR1cm5yZWNlaXB0djFgXShjb250cmFjdHMvYWN0aXZlLXJldGFpbmVkLXR1cm4tcmVjZWlwdC12MS5tZCM0LWFjdGl2ZXJldGFpbmVkdHVybnJlY2VpcHR2MSkuCiAKICMjIDUuIFJlY2VpcHQgYWNjZXB0YW5jZSBzb3VyY2UKIApkaWZmIC0tZ2l0IGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZCBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9SRUFETUUubWQKaW5kZXggOWNmMmExZjA3Li4wMGM4NjJkN2YgMTAwNjQ0Ci0tLSBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9SRUFETUUubWQKKysrIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZApAQCAtMTksNiArMTksOCBAQAogfCBgQVVUSE9SSVRZX1JFUVVJUkVEOlJVTlRJTUVfUkVGQUNUT1JfUkVFTlRSWWAgfCBnYXRlIHwgW2BnYXRlcy9hdXRob3JpdHktcmVxdWlyZWQtcnVudGltZS1yZWZhY3Rvci1yZWVudHJ5Lm1kYF0oLi4vZ2F0ZXMvYXV0aG9yaXR5LXJlcXVpcmVkLXJ1bnRpbWUtcmVmYWN0b3ItcmVlbnRyeS5tZCkgfCBjbG9zZWQgZG9jdW1lbnRhdGlvbi9jb250cm9sLXBsYW5lIHNlbGVjdGlvbiBnYXRlIHwgQ2xvc2VkIGFzIHNlbGVjdGlvbiBoaXN0b3J5OyBBMS4zLVAxIGlzIHRoZSBzZXBhcmF0ZSBhY3RpdmUgcGFja2V0LiB8IFtgMDRgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kI2F1dGhvcml0eV9yZXF1aXJlZHJ1bnRpbWVfcmVmYWN0b3JfcmVlbnRyeS1jb250cmFjdC0yMDI2LTA4LTIwLWNsb3NlZC1zZWxlY3Rpb24tcmVjb3JkKSB8CiB8IGBIb3N0RXhlY3V0aW9uRXBpc29kZVYxYCB8IGNvbnRyYWN0IHwgW2Bjb250cmFjdHMvaG9zdC1leGVjdXRpb24tZXBpc29kZS12MS5tZGBdKC4uL2NvbnRyYWN0cy9ob3N0LWV4ZWN1dGlvbi1lcGlzb2RlLXYxLm1kKSB8IGBjYW5vbmljYWwgZXh0cmFjdGVkIGNvbnRyYWN0IG93bmVyIGZvciB0aGUgY29tcGxldGUgc2NoZW1hLCBlcGlzb2RlLWtpbmQgbGl0ZXJhbHMsIHRyYW5zcG9ydC1zdGF0dXMgbGl0ZXJhbHMsIGFuZCBydWxlcyAx4oCTNmAgfCBTdXBlcnNlZGVzIG9ubHkgcm9vdC1jYW5vbmljYWwgb3duZXJzaGlwIG9mIHRoZSBleHRyYWN0ZWQgYEhvc3RFeGVjdXRpb25FcGlzb2RlVjFgIHNwYW47IHRoZSBjb21taXR0ZWQgcHJlLXNwYW4gY29tcGF0aWJpbGl0eSBhbmNob3JzIGFuZCBuZWlnaGJvcmluZyBgMkFgL2AyQmAgb3duZXJzIHJlbWFpbiB1bmNoYW5nZWQuIHwgW2AwNGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMi1ob3N0ZXhlY3V0aW9uZXBpc29kZXYxKSB8CiB8IGBEZWZlcnJlZCByZXRhaW5lZC1zcGF3biBhZG1pc3Npb24gcmVjb3ZlcnkgY29udHJhY3RgIHwgY29udHJhY3QgfCBbYGNvbnRyYWN0cy9kZWZlcnJlZC1yZXRhaW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29udHJhY3QubWRgXSguLi9jb250cmFjdHMvZGVmZXJyZWQtcmV0YWluZWQtc3Bhd24tYWRtaXNzaW9uLXJlY292ZXJ5LWNvbnRyYWN0Lm1kKSB8IGBjYW5vbmljYWwgZXh0cmFjdGVkIG93bmVyIGZvciB0aGUgY29tcGxldGUgZGVmZXJyZWQgcmV0YWluZWQtc3Bhd24gYWRtaXNzaW9uIHJlY292ZXJ5IGNvbnRyYWN0LCBwYWlyZWQgVjEgcmVjb3Zlcnkgc2NoZW1hcywgY29tcGF0aWJpbGl0eSBib3VuZGFyaWVzLCByb3V0ZSBleGNsdXNpb25zLCBhbGxvd2xpc3RzLCBhbmQgcmVjb3JkZWQtcmVzdWx0IGxpbWl0c2AgfCBTdXBlcnNlZGVzIG9ubHkgcm9vdC1jYW5vbmljYWwgb3duZXJzaGlwIG9mIHRoZSBleHRyYWN0ZWQgYERlZmVycmVkIHJldGFpbmVkLXNwYXduIGFkbWlzc2lvbiByZWNvdmVyeSBjb250cmFjdGAgc3BhbjsgdGhlIHR3byBwcmVyZXF1aXNpdGUgY29tcGF0aWJpbGl0eSBhbmNob3JzLCBgSG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWAsIGFuZCB0aGUgZm9sbG93aW5nIGBCMy4yYS1XQSBFeGFjdEJvdW5kV29ybGRPd25lcnNoaXBBZG9wdGlvblYxYCBvd25lciByZW1haW4gdW5jaGFuZ2VkLiB8IFtgMDRgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kI2RlZmVycmVkLXJldGFpbmVkLXNwYXduLWFkbWlzc2lvbi1yZWNvdmVyeS1jb250cmFjdCkgfAorfCBgQWN0aXZlRXBoZW1lcmFsVGFza1JlY2VpcHRWMWAgfCBjb250cmFjdCB8IFtgY29udHJhY3RzL2FjdGl2ZS1lcGhlbWVyYWwtdGFzay1yZWNlaXB0LXYxLm1kYF0oLi4vY29udHJhY3RzL2FjdGl2ZS1lcGhlbWVyYWwtdGFzay1yZWNlaXB0LXYxLm1kKSB8IGBjYW5vbmljYWwgZXh0cmFjdGVkIGNvbnRyYWN0IG93bmVyIGZvciB0aGUgY29tcGxldGUgc2NoZW1hLCBzdGF0ZSB0cmFuc2l0aW9ucywgdGVybWluYWwgbW9ub3RvbmljaXR5LCByZXN1bHQtY2xhc3MgbGl0ZXJhbHMsIGFuZCBleHBsaWNpdCBOZWVkc1JldGFpbmVkRm9sbG93dXAgY29uc3RyYWludHNgIHwgU3VwZXJzZWRlcyBvbmx5IHJvb3QtY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIGBBY3RpdmVFcGhlbWVyYWxUYXNrUmVjZWlwdFYxYCBzcGFuOyBgUmVjZWlwdCBhY2NlcHRhbmNlIHNvdXJjZWAgYW5kIG5laWdoYm9yaW5nIEQxMCBvd25lcnMgcmVtYWluIHVuY2hhbmdlZC4gfCBbYDA0YF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMzLWFjdGl2ZWVwaGVtZXJhbHRhc2tyZWNlaXB0djEpIHwKK3wgYEFjdGl2ZVJldGFpbmVkVHVyblJlY2VpcHRWMWAgfCBjb250cmFjdCB8IFtgY29udHJhY3RzL2FjdGl2ZS1yZXRhaW5lZC10dXJuLXJlY2VpcHQtdjEubWRgXSguLi9jb250cmFjdHMvYWN0aXZlLXJldGFpbmVkLXR1cm4tcmVjZWlwdC12MS5tZCkgfCBgY2Fub25pY2FsIGV4dHJhY3RlZCBjb250cmFjdCBvd25lciBmb3IgdGhlIGNvbXBsZXRlIHNjaGVtYSwgc3RhdGUgdHJhbnNpdGlvbnMsIGFuZCBydWxlcyAx4oCTMyBjb3ZlcmluZyB0aGUgc2luZ2xlIGFjdGl2ZSBjYW5jZWxhYmxlIHR1cm4sIFBhcmtlZCBjb250aW51aXR5LCBhbmQgdGhlIGhvc3Qvd29ya2VyIHBvc3R1cmUgYm91bmRhcnlgIHwgU3VwZXJzZWRlcyBvbmx5IHJvb3QtY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIGBBY3RpdmVSZXRhaW5lZFR1cm5SZWNlaXB0VjFgIHNwYW47IGBSZWNlaXB0IGFjY2VwdGFuY2Ugc291cmNlYCBhbmQgbmVpZ2hib3JpbmcgRDEwIG93bmVycyByZW1haW4gdW5jaGFuZ2VkLiB8IFtgMDRgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzQtYWN0aXZlcmV0YWluZWR0dXJucmVjZWlwdHYxKSB8CiB8IGBzaGFyZWQtdGFyZ2V0LWFyY2hpdGVjdHVyZWAgfCBhcmNoaXRlY3R1cmUgaW5kZXggfCBbYGFyY2hpdGVjdHVyZS9SRUFETUUubWRgXSguLi9hcmNoaXRlY3R1cmUvUkVBRE1FLm1kKSB8IGBub24tYXV0aG9yaXRhdGl2ZSBuYXZpZ2F0aW9uIGZvciB0aGUgZXh0cmFjdGVkIGV4ZWN1dGl2ZSB0YXJnZXQgZGVjaXNpb24sIGF1dGhvcml0eSBtYXAsIG51bWJlcmVkIGludmFyaWFudHMsIGFuZCBzdGFibGUgcmV2aWV3IHF1ZXN0aW9uYCB8IFN1cGVyc2VkZXMgb25seSByb290LWNhbm9uaWNhbCBvd25lcnNoaXAgb2YgdGhlIGV4dHJhY3RlZCBzaGFyZWQgYXJjaGl0ZWN0dXJlIHNwYW5zOyBwYWNrZXQtZmFtaWx5LWxvY2FsIEQ1L0Q2IGZvcndhcmRlcnMgcmVtYWluIGF0IHRoZWlyIGV4aXN0aW5nIG93bmVycy4gfCBbYDAxYF0oLi4vMDEtdGFyZ2V0LWFyY2hpdGVjdHVyZS5tZCNleGVjdXRpdmUtZGVjaXNpb24pLCBbYDAxYF0oLi4vMDEtdGFyZ2V0LWFyY2hpdGVjdHVyZS5tZCNhdXRob3JpdHktbWFwKSwgW2AwMWBdKC4uLzAxLXRhcmdldC1hcmNoaXRlY3R1cmUubWQjbm9uLW5lZ290aWFibGUtaW52YXJpYW50cyksIFtgMDFgXSguLi8wMS10YXJnZXQtYXJjaGl0ZWN0dXJlLm1kI3Jldmlldy1xdWVzdGlvbikgfAogfCBgc2hhcmVkLXNlYW0tY3Jvc3N3YWxrYCB8IHNlYW0gaW5kZXggfCBbYHNlYW1zL1JFQURNRS5tZGBdKC4uL3NlYW1zL1JFQURNRS5tZCkgfCBgY2Fub25pY2FsIHNoYXJlZCBzZWFtLWNyb3Nzd2FsayBydWxlcyBwbHVzIGV4dHJhY3RlZCBzZWFtLWZhbWlseSBuYXZpZ2F0aW9uIGZvciBob3N0L3Nlc3Npb24sIHBlcnNpc3RlbmNlL2NvbXBhdGliaWxpdHksIGRpc3BhdGNoL2VwaXNvZGUgdHJhbnNwb3J0LCBwb2xpY3kvbmFycm93aW5nLCBydW50aW1lLWV2ZW50L3JlY2VpcHQvc3VwZXJ2aXNpb24vcmV0YWluZWQtcnVudGltZSwgb2JsaWdhdGlvbnMvaG9zdCByZS1lbmdhZ2VtZW50LCBjb25maWd1cmF0aW9uL2dhdGV3YXkgYWRvcHRpb24sIGFuZCBVQUEvcHJvdmlkZXIgcmVhbGl6YXRpb24vc2lkZS1lZmZlY3QgbWVkaWF0aW9uYCB8IFN1cGVyc2VkZXMgb25seSB0aGUgZXh0cmFjdGVkIHJvb3QgcmVhZGluZy1ydWxlL2NsYXNzaWZpY2F0aW9uIHNwYW5zIGFuZCB0aGUgZXh0cmFjdGVkIGhvc3Qvc2Vzc2lvbiBhdXRob3JpdHksIHBlcnNpc3RlbmNlL2NvbXBhdGliaWxpdHksIGRpc3BhdGNoL2VwaXNvZGUgdHJhbnNwb3J0LCBwb2xpY3kvbmFycm93aW5nLCBydW50aW1lLWV2ZW50L3JlY2VpcHQvc3VwZXJ2aXNpb24vcmV0YWluZWQtcnVudGltZSwgb2JsaWdhdGlvbnMvaG9zdCByZS1lbmdhZ2VtZW50LCBjb25maWd1cmF0aW9uL2dhdGV3YXkgYWRvcHRpb24sIGFuZCBVQUEvcHJvdmlkZXIgcmVhbGl6YXRpb24vc2lkZS1lZmZlY3QgbWVkaWF0aW9uIGZhbWlseSByb3dzOyBBMCBhbmQgdGhlIGV4aXN0aW5nIEQ2IGBIb3N0U2Vzc2lvbkF1dGhvcml0eWAsIGBXb3JsZFdvcmtlck1lc3NhZ2luZ1Byb3RvY29sYCwgYW5kIGBPYmxpZ2F0aW9uTGVkZ2VyYCBjb21wYXRpYmlsaXR5IHJvd3MgcmVtYWluIGF0IHRoZWlyIGN1cnJlbnQgb3duZXJzLiB8IFtgMDJgXSguLi8wMi1zZWFtLWNyb3Nzd2Fsay5tZCNyZWFkaW5nLXJ1bGUpLCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjYS1ob3N0LWF1dGhvcml0eS1hbmQtaW5ncmVzcyksIFtgMDJgXSguLi8wMi1zZWFtLWNyb3Nzd2Fsay5tZCNiLWRpc3BhdGNoLXBvbGljeS1yZWNlaXB0cy1hbmQtcmV0YWluZWQtcnVudGltZSksIFtgMDJgXSguLi8wMi1zZWFtLWNyb3Nzd2Fsay5tZCNjLW9ibGlnYXRpb25zLWFuZC1ob3N0LXJlLWVuZ2FnZW1lbnQpLCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjZC11YWEtcmVhbGl6YXRpb24tcHJvamVjdGlvbi1hbmQtc2lkZS1lZmZlY3QtbWVkaWF0aW9uKSwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1kI2NsYXNzaWZpY2F0aW9uLWNvbnNlcXVlbmNlcykgfAogfCBgc2hhcmVkLXNsaWNlLW1hcGAgfCBzbGljZSBpbmRleCB8IFtgc2xpY2VzL1JFQURNRS5tZGBdKC4uL3NsaWNlcy9SRUFETUUubWQpIHwgYGNhbm9uaWNhbCBzaGFyZWQgc2VxdWVuY2luZy9kZXBlbmRlbmN5IGFuZCBzbGljZS1jbG9zZW91dCBvd25lciBwbHVzIG5vbi1hdXRob3JpdGF0aXZlIEQ5IFRyYWNrIEHigJNFIG5hdmlnYXRpb24gYW5kIEExL0ExLjQgcHJvamVjdGlvbiBpbmRleGAgfCBTdXBlcnNlZGVzIG9ubHkgcm9vdC1jYW5vbmljYWwgb3duZXJzaGlwIG9mIHRoZSBleHRyYWN0ZWQgc2hhcmVkIHNlcXVlbmNpbmcvY2xvc2VvdXQgc3BhbnMgcGx1cyB0aGUgZXh0cmFjdGVkIFRyYWNrIEHigJNFIGFuZCBBMS9BMS40IHByb2plY3Rpb24gc3BhbnM7IGNvbnRyb2xsaW5nIHNjaGVkdWxlIGF1dGhvcml0eSByZW1haW5zIHdpdGggZGVjaXNpb25zLCBwYWNrZXRzLCBnYXRlcywgYW5kIGBjdXJyZW50Lm1kYC4gfCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3NlcXVlbmNpbmctcnVsZXMpLCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3RyYWNrLWEtLWF1dGhvcml0eS1hbmQtc3VyZmFjZS1uZXV0cmFsaXR5KSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCNhMS1ib3VuZGVkLXBhY2tldC1kZWNvbXBvc2l0aW9uKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCN0cmFjay1iLS13b3JsZC1kaXNwYXRjaC1yZWNlaXB0cy1zdXBlcnZpc2lvbi1hbmQtY2FuY2VsKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCN0cmFjay1jLS1vYmxpZ2F0aW9ucy1pbmJveC1hdXRvLWF0dGFjaC1hbmQtcm91dGVyLWF0dGFjaCksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjdHJhY2stZC0tdWFhLWV4ZWN1dGlvbi1lbnZlbG9wZS1hbmQtc2lkZS1lZmZlY3QtbWVkaWF0aW9uKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCN0cmFjay1lLS1kaXNwYXRjaC1zY29wZWQtcG9saWN5LW5hcnJvd2luZy1hbmQtY29uZmlnLXByb2plY3Rpb24pLCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3NsaWNlLWNsb3Nlb3V0LW1pbmltdW0pIHwKZGlmZiAtLWdpdCBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJhY3Rpb24tbGVkZ2VyLm1kCmluZGV4IDYxOWMxODI2Yy4uMTZmZDYxMGZjIDEwMDY0NAotLS0gYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJhY3Rpb24tbGVkZ2VyLm1kCisrKyBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQKQEAgLTE4NiwzICsxODYsNSBAQCBUaGlzIGxlZGdlciByZWNvcmRzIGNvbnRlbnQtcHJlc2VydmluZyBhdXRob3JpdHkgdHJhbnNmZXJzIHdoaWxlIHJldGFpbmluZyByZXF1aQogfCBEOSB8IFtgMDMtcGhhc2Utc2xpY2UtbWFwLm1kYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kKSB8IEExLjQg4oCUIGJvdW5kZWQgYXV0by1hdHRhY2ggcHJvZHVjZXIgYWRvcHRpb24gYW5kIHJlZ3Jlc3Npb24gY2xvc3VyZSB8IFtgYTEtYm91bmRlZC1wYWNrZXQtZGVjb21wb3NpdGlvbmBdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCNhMS1ib3VuZGVkLXBhY2tldC1kZWNvbXBvc2l0aW9uKSB8IGxpbmUgMTQ1IHwgYDU4ZDc5ODJiMjNhN2IyZjVmNDFmM2UwYjUxNzA5OTZlMGYzYjMzMWRmYmJjYmI1ODc1ZDAyZTgwMWJkNWQ0MDhgIHwgW2AuLi9zbGljZXMvdGFza3MvYTEtNC1hdXRvLWF0dGFjaC1wcm9kdWNlci1hZG9wdGlvbi5tZGBdKC4uL3NsaWNlcy90YXNrcy9hMS00LWF1dG8tYXR0YWNoLXByb2R1Y2VyLWFkb3B0aW9uLm1kKSB8IHRhc2sgcm93IHwgY2Fub25pY2FsIGRlc3RpbmF0aW9uOyBzb3VyY2UgY29tcGF0aWJpbGl0eSB0YWJsZSByb3cgfCBub25lIHwgcmVzdG9yZSB0aGUgZXhhY3QgZXh0cmFjdGVkIEQ5IHNvdXJjZSBib2RpZXMvcm93cyBpbiBgMDMtcGhhc2Utc2xpY2UtbWFwLm1kYDsgcmVtb3ZlIGBzbGljZXMvYCBhbmQgYWxsIEQ5LWFkZGVkIGZpbGVzIGJlbmVhdGggaXQ7IHJlbW92ZSB0aGUgYHNoYXJlZC1zbGljZS1tYXBgIGluZGV4IHJvdyBmcm9tIGBpbmRleC9SRUFETUUubWRgOyByZW1vdmUgYWxsIG1hdGNoaW5nIEQ5IGxlZGdlciBlbnRyaWVzOyBsZWF2ZSBENeKAk0Q4IGNhbm9uaWNhbCBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYW5kIGByZXZpZXctY29udHJvbC9gIHVuY2hhbmdlZCB8CiB8IEQxMCB8IFtgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQpIHwgMi4gYEhvc3RFeGVjdXRpb25FcGlzb2RlVjFgIHwgW2AyLWhvc3RleGVjdXRpb25lcGlzb2RldjFgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzItaG9zdGV4ZWN1dGlvbmVwaXNvZGV2MSkgfCBsaW5lcyA1MeKAkzEwMCB8IGAyOGU4NmUyZTdlMjliNDU4MjgwODc1ZjkyMWNhMzI3YTQ2OTYyMWFjOTZiZDdlNTkyYTBhMTc2MDI3YWEwMTE3YCB8IFtgLi4vY29udHJhY3RzL2hvc3QtZXhlY3V0aW9uLWVwaXNvZGUtdjEubWRgXSguLi9jb250cmFjdHMvaG9zdC1leGVjdXRpb24tZXBpc29kZS12MS5tZCkgfCBjb250cmFjdCB8IGNhbm9uaWNhbCBkZXN0aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9yIHwgbm9uZSB8IHJlc3RvcmUgdGhlIGV4YWN0IDE2NDUtYnl0ZSBzb3VyY2Ugc3BhbiBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMyLWhvc3RleGVjdXRpb25lcGlzb2RldjFgOyByZW1vdmUgYGNvbnRyYWN0cy9ob3N0LWV4ZWN1dGlvbi1lcGlzb2RlLXYxLm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgSG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWAgcm93IGZyb20gYGluZGV4L1JFQURNRS5tZGA7IHJlbW92ZSB0aGlzIEQxMCBsZWRnZXIgZW50cnk7IGxlYXZlIHRoZSBjb21taXR0ZWQgcHJlLXNwYW4gY29tcGF0aWJpbGl0eSBhbmNob3JzLCBEMy9ENeKAk0Q5IG93bmVycywgYGluZGV4L2N1cnJlbnQubWRgLCBgcmV2aWV3LWNvbnRyb2wvYCwgYWxsIG90aGVyIEQxMCB1bml0cywgYW5kIGV2ZXJ5IHBhdGggb3V0c2lkZSB0aGUgZm91ci1wYXRoIGZlbmNlIHVuY2hhbmdlZCB8CiB8IEQxMCB8IFtgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQpIHwgRGVmZXJyZWQgcmV0YWluZWQtc3Bhd24gYWRtaXNzaW9uIHJlY292ZXJ5IGNvbnRyYWN0IHwgW2BkZWZlcnJlZC1yZXRhaW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29udHJhY3RgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kI2RlZmVycmVkLXJldGFpbmVkLXNwYXduLWFkbWlzc2lvbi1yZWNvdmVyeS1jb250cmFjdCkgfCBsaW5lcyA3MeKAkzIxMyB8IGBkOTAzMjk5YmRkMzQ1MzE2ZmQwOTQzZDI4YWJjMzAwYTFjZWYwMDFjZTA3NDQ3NDZmNzJmNTM2MDdlYjY0YzEyYCB8IFtgLi4vY29udHJhY3RzL2RlZmVycmVkLXJldGFpbmVkLXNwYXduLWFkbWlzc2lvbi1yZWNvdmVyeS1jb250cmFjdC5tZGBdKC4uL2NvbnRyYWN0cy9kZWZlcnJlZC1yZXRhaW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29udHJhY3QubWQpIHwgY29udHJhY3QgfCBjYW5vbmljYWwgZGVzdGluYXRpb247IHNvdXJjZSBjb21wYXRpYmlsaXR5IGFuY2hvciB8IG5vbmUgfCByZXN0b3JlIHRoZSBleGFjdCA5ODk2LWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjZGVmZXJyZWQtcmV0YWluZWQtc3Bhd24tYWRtaXNzaW9uLXJlY292ZXJ5LWNvbnRyYWN0YDsgcmVtb3ZlIGBjb250cmFjdHMvZGVmZXJyZWQtcmV0YWluZWQtc3Bhd24tYWRtaXNzaW9uLXJlY292ZXJ5LWNvbnRyYWN0Lm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgRGVmZXJyZWQgcmV0YWluZWQtc3Bhd24gYWRtaXNzaW9uIHJlY292ZXJ5IGNvbnRyYWN0YCByb3cgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsgcmVtb3ZlIHRoaXMgRDEwIGxlZGdlciBlbnRyeTsgbGVhdmUgYEhvc3RFeGVjdXRpb25FcGlzb2RlVjFgLCB0aGUgdHdvIHByZXJlcXVpc2l0ZSBjb21wYXRpYmlsaXR5IGFuY2hvcnMsIGFsbCBEMy9ENeKAk0Q5IG93bmVycywgYGluZGV4L2N1cnJlbnQubWRgLCBgcmV2aWV3LWNvbnRyb2wvYCwgbGF0ZXIgRDEwIHVuaXRzLCBhbmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmb3VyLXBhdGggZmVuY2UgdW5jaGFuZ2VkIHwKK3wgRDEwIHwgW2AwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kYF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCkgfCAzLiBgQWN0aXZlRXBoZW1lcmFsVGFza1JlY2VpcHRWMWAgfCBbYDMtYWN0aXZlZXBoZW1lcmFsdGFza3JlY2VpcHR2MWBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMy1hY3RpdmVlcGhlbWVyYWx0YXNrcmVjZWlwdHYxKSB8IGxpbmVzIDkx4oCTMTM4IHwgYGM5OTc3NTI2NTRmMDQxZGU4MjAyMTBmMDNjODY3MDY1NTFiNDQzODk5OTRkZjVlOGUwMzQxYWE0MWEzNGZhM2JgIHwgW2AuLi9jb250cmFjdHMvYWN0aXZlLWVwaGVtZXJhbC10YXNrLXJlY2VpcHQtdjEubWRgXSguLi9jb250cmFjdHMvYWN0aXZlLWVwaGVtZXJhbC10YXNrLXJlY2VpcHQtdjEubWQpIHwgY29udHJhY3QgfCBjYW5vbmljYWwgZGVzdGluYXRpb247IHNvdXJjZSBjb21wYXRpYmlsaXR5IGFuY2hvciB8IG5vbmUgfCByZXN0b3JlIHRoZSBleGFjdCAxNTU3LWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMy1hY3RpdmVlcGhlbWVyYWx0YXNrcmVjZWlwdHYxYCBhbmQgdGhlIGV4YWN0IDE0NDMtYnl0ZSBzb3VyY2Ugc3BhbiBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM0LWFjdGl2ZXJldGFpbmVkdHVybnJlY2VpcHR2MWA7IHJlbW92ZSBgY29udHJhY3RzL2FjdGl2ZS1lcGhlbWVyYWwtdGFzay1yZWNlaXB0LXYxLm1kYCBhbmQgYGNvbnRyYWN0cy9hY3RpdmUtcmV0YWluZWQtdHVybi1yZWNlaXB0LXYxLm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgQWN0aXZlRXBoZW1lcmFsVGFza1JlY2VpcHRWMWAgYW5kIGBBY3RpdmVSZXRhaW5lZFR1cm5SZWNlaXB0VjFgIHJvd3MgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsgcmVtb3ZlIGJvdGggRDEwIGxlZGdlciBlbnRyaWVzIGZvciB0aGlzIGJhdGNoOyBsZWF2ZSBgSG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWAsIGRlZmVycmVkIHJldGFpbmVkLXNwYXduIGFkbWlzc2lvbiByZWNvdmVyeSwgYFJlY2VpcHQgYWNjZXB0YW5jZSBzb3VyY2VgLCBhbGwgRDMvRDXigJNEOSBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYHJldmlldy1jb250cm9sL2AsIHJlbWFpbmluZyBEMTAgdW5pdHMsIGFuZCBldmVyeSBwYXRoIG91dHNpZGUgdGhlIGZpdmUtcGF0aCBmZW5jZSB1bmNoYW5nZWQgfAorfCBEMTAgfCBbYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWRgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kKSB8IDQuIGBBY3RpdmVSZXRhaW5lZFR1cm5SZWNlaXB0VjFgIHwgW2A0LWFjdGl2ZXJldGFpbmVkdHVybnJlY2VpcHR2MWBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjNC1hY3RpdmVyZXRhaW5lZHR1cm5yZWNlaXB0djEpIHwgbGluZXMgMTM54oCTMTgzIHwgYDJjYzk5NjExMWQ4MmQyOTZmODliZGQ4OGY4MGRmNWMwYjU1OWJkYjMwNWFlYzU0Njg4MWI3NDlhY2U5M2Y2N2VgIHwgW2AuLi9jb250cmFjdHMvYWN0aXZlLXJldGFpbmVkLXR1cm4tcmVjZWlwdC12MS5tZGBdKC4uL2NvbnRyYWN0cy9hY3RpdmUtcmV0YWluZWQtdHVybi1yZWNlaXB0LXYxLm1kKSB8IGNvbnRyYWN0IHwgY2Fub25pY2FsIGRlc3RpbmF0aW9uOyBzb3VyY2UgY29tcGF0aWJpbGl0eSBhbmNob3IgfCBub25lIHwgcmVzdG9yZSB0aGUgZXhhY3QgMTU1Ny1ieXRlIHNvdXJjZSBzcGFuIGF0IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzMtYWN0aXZlZXBoZW1lcmFsdGFza3JlY2VpcHR2MWAgYW5kIHRoZSBleGFjdCAxNDQzLWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjNC1hY3RpdmVyZXRhaW5lZHR1cm5yZWNlaXB0djFgOyByZW1vdmUgYGNvbnRyYWN0cy9hY3RpdmUtZXBoZW1lcmFsLXRhc2stcmVjZWlwdC12MS5tZGAgYW5kIGBjb250cmFjdHMvYWN0aXZlLXJldGFpbmVkLXR1cm4tcmVjZWlwdC12MS5tZGA7IHJlbW92ZSB0aGUgZXhhY3QgYEFjdGl2ZUVwaGVtZXJhbFRhc2tSZWNlaXB0VjFgIGFuZCBgQWN0aXZlUmV0YWluZWRUdXJuUmVjZWlwdFYxYCByb3dzIGZyb20gYGluZGV4L1JFQURNRS5tZGA7IHJlbW92ZSBib3RoIEQxMCBsZWRnZXIgZW50cmllcyBmb3IgdGhpcyBiYXRjaDsgbGVhdmUgYEhvc3RFeGVjdXRpb25FcGlzb2RlVjFgLCBkZWZlcnJlZCByZXRhaW5lZC1zcGF3biBhZG1pc3Npb24gcmVjb3ZlcnksIGBSZWNlaXB0IGFjY2VwdGFuY2Ugc291cmNlYCwgYWxsIEQzL0Q14oCTRDkgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9gLCByZW1haW5pbmcgRDEwIHVuaXRzLCBhbmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmaXZlLXBhdGggZmVuY2UgdW5jaGFuZ2VkIHwKZGlmZiAtLWdpdCBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9jb250cmFjdHMvYWN0aXZlLWVwaGVtZXJhbC10YXNrLXJlY2VpcHQtdjEubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvY29udHJhY3RzL2FjdGl2ZS1lcGhlbWVyYWwtdGFzay1yZWNlaXB0LXYxLm1kCm5ldyBmaWxlIG1vZGUgMTAwNjQ0CmluZGV4IDAwMDAwMDAwMC4uYWY2ZGE5OGFhCi0tLSAvZGV2L251bGwKKysrIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2NvbnRyYWN0cy9hY3RpdmUtZXBoZW1lcmFsLXRhc2stcmVjZWlwdC12MS5tZApAQCAtMCwwICsxLDU2IEBACisqKktpbmQ6KiogY29udHJhY3QKKyoqU3RhdHVzOioqIGNhbm9uaWNhbAorKipDYW5vbmljYWwgZm9yOioqIGNvbXBsZXRlIGV4dHJhY3RlZCBgQWN0aXZlRXBoZW1lcmFsVGFza1JlY2VpcHRWMWAgc2NoZW1hLCBzdGF0ZSB0cmFuc2l0aW9ucywgdGVybWluYWwgbW9ub3RvbmljaXR5LCBleHBsaWNpdCByZXN1bHQtY2xhc3MgbGl0ZXJhbCBvcmRlciwgYW5kIGBOZWVkc1JldGFpbmVkRm9sbG93dXBgIG5lZ2F0aXZlIHJlcXVpcmVtZW50cworKipTb3VyY2UgcHJvdmVuYW5jZToqKiBleHRyYWN0ZWQgYnl0ZS1mb3ItYnl0ZSBmcm9tIFtgLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMzLWFjdGl2ZWVwaGVtZXJhbHRhc2tyZWNlaXB0djFgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzMtYWN0aXZlZXBoZW1lcmFsdGFza3JlY2VpcHR2MSksIGJhc2VsaW5lIGxpbmVzIDkx4oCTMTM4OyB0aGUgZXhhY3QgMTU1Ny1ieXRlIHNvdXJjZSBib2R5IGlzIHByZXNlcnZlZCBiZXR3ZWVuIHRoZSBib3VuZGFyeSBtYXJrZXJzIGJlbG93CisqKkJhc2VsaW5lIHNwYW4gU0hBLTI1NjoqKiBgYzk5Nzc1MjY1NGYwNDFkZTgyMDIxMGYwM2M4NjcwNjU1MWI0NDM4OTk5NGRmNWU4ZTAzNDFhYTQxYTM0ZmEzYmAKKworPCEtLSBleGFjdC1leHRyYWN0ZWQtYm9keTpzdGFydCAtLT4KKyMjIDMuIGBBY3RpdmVFcGhlbWVyYWxUYXNrUmVjZWlwdFYxYAorCitgYGBydXN0CitzdHJ1Y3QgQWN0aXZlRXBoZW1lcmFsVGFza1JlY2VpcHRWMSB7CisgICAgc2NoZW1hX3ZlcnNpb246IHUzMiwKKyAgICBhY2NlcHRhbmNlX3JlY29yZF9pZDogU3RyaW5nLAorICAgIHRhc2tfcnVuX2lkOiBTdHJpbmcsCisgICAgcmVxdWVzdF9pZDogU3RyaW5nLAorICAgIG9yY2hlc3RyYXRpb25fc2Vzc2lvbl9pZDogU3RyaW5nLAorICAgIGNhbGxlcl9wYXJ0aWNpcGFudF9pZDogU3RyaW5nLAorICAgIHRhcmdldF9iYWNrZW5kX2lkOiBTdHJpbmcsCisgICAgd29ybGRfaWQ6IFN0cmluZywKKyAgICB3b3JsZF9nZW5lcmF0aW9uOiB1NjQsCisgICAgcG9saWN5X3NuYXBzaG90X3JlZjogUG9saWN5U25hcHNob3RSZWZWMSwKKyAgICBwb2xpY3lfc25hcHNob3RfaGFzaDogU3RyaW5nLAorICAgIHBvbGljeV9yZXZpc2lvbjogU3RyaW5nLAorICAgIG5hcnJvd2luZ19yZWFzb246IE9wdGlvbjxTdHJpbmc+LAorICAgIHJ1bnRpbWVfYWNjZXB0YW5jZTogUnVudGltZUFjY2VwdGFuY2VFdmlkZW5jZVYxLAorICAgIG9ic2VydmF0aW9uX2NsYWltOiBTdXBlcnZpc29yT2JzZXJ2YXRpb25DbGFpbVYxLAorICAgIGFjY2VwdGVkX2F0OiBUaW1lc3RhbXAsCisgICAgc3RhdGVfcmV2aXNpb246IHU2NCwKKyAgICBzdGF0ZTogQWN0aXZlVGFza1N0YXRlVjEsCisgICAgY2FuY2VsX3N1cHBvcnRlZDogYm9vbCwKKyAgICB0ZXJtaW5hbDogT3B0aW9uPFdvcmxkV29ya1Rlcm1pbmFsVjE+LAorfQorYGBgCisKK1N0YXRlczoKKworYGBgdGV4dAorQWNjZXB0ZWQgLT4gUnVubmluZyAtPiBBdHRlbnRpb25QZW5kaW5nIC0+IFJ1bm5pbmcKK0FjY2VwdGVkfFJ1bm5pbmd8QXR0ZW50aW9uUGVuZGluZyAtPiBUZXJtaW5hbHxGYWlsZWR8Q2FuY2VsbGVkfEludmFsaWRhdGVkCitgYGAKKworYFRlcm1pbmFsYCwgYEZhaWxlZGAsIGBDYW5jZWxsZWRgLCBhbmQgYEludmFsaWRhdGVkYCBhcmUgdGVybWluYWwgYW5kIG1vbm90b25pYy4gYFBhcmtlZGAgaXMgbm90IHZhbGlkIGZvciBhbiBlcGhlbWVyYWwgdGFzay4KKworYFdvcmxkV29ya1Rlcm1pbmFsVjFgIGZvciBhbiBlcGhlbWVyYWwgdGFzayBjYXJyaWVzIG9uZSBleHBsaWNpdCByZXN1bHQgY2xhc3M6CisKK2BgYHRleHQKK0NvbXBsZXRlZAorRmFpbGVkCitDYW5jZWxsZWQKK05lZWRzUmV0YWluZWRGb2xsb3d1cAorSW52YWxpZGF0ZWQKK2BgYAorCitgTmVlZHNSZXRhaW5lZEZvbGxvd3VwYCBpcyBhIHRlcm1pbmFsIGVwaGVtZXJhbCByZXN1bHQsIG5vdCBhIHJldGFpbmVkIHdvcmtlciBzdGF0ZS4gSXQgcHJvbWlzZXMgbm8gZHVyYWJsZSBwYXJ0aWNpcGFudCBpZGVudGl0eSwgY3JlYXRlcyBubyBgY29udGludWVfd29ybGRfd29ya2VyYCByb3V0ZSwgYW5kIGRvZXMgbm90IHNpbGVudGx5IGNyZWF0ZSBhIHJldGFpbmVkIHdvcmtlciBvciBkdXJhYmxlIGNvbnZlcnNhdGlvbmFsIG9ibGlnYXRpb24uIFRoZSBob3N0IG11c3QgbWFrZSBhIG5ldywgZXhwbGljaXQsIHBvbGljeS1jaGVja2VkIGBzcGF3bl93b3JsZF93b3JrZXJgIGRlY2lzaW9uIGlmIG9uZ29pbmcgd29yayBpcyB3YXJyYW50ZWQuCisKKzwhLS0gZXhhY3QtZXh0cmFjdGVkLWJvZHk6ZW5kIC0tPgpkaWZmIC0tZ2l0IGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2NvbnRyYWN0cy9hY3RpdmUtcmV0YWluZWQtdHVybi1yZWNlaXB0LXYxLm1kIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2NvbnRyYWN0cy9hY3RpdmUtcmV0YWluZWQtdHVybi1yZWNlaXB0LXYxLm1kCm5ldyBmaWxlIG1vZGUgMTAwNjQ0CmluZGV4IDAwMDAwMDAwMC4uYzRkMjE4ZmFiCi0tLSAvZGV2L251bGwKKysrIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2NvbnRyYWN0cy9hY3RpdmUtcmV0YWluZWQtdHVybi1yZWNlaXB0LXYxLm1kCkBAIC0wLDAgKzEsNTMgQEAKKyoqS2luZDoqKiBjb250cmFjdAorKipTdGF0dXM6KiogY2Fub25pY2FsCisqKkNhbm9uaWNhbCBmb3I6KiogY29tcGxldGUgZXh0cmFjdGVkIGBBY3RpdmVSZXRhaW5lZFR1cm5SZWNlaXB0VjFgIHNjaGVtYSwgc3RhdGUgdHJhbnNpdGlvbnMsIGFuZCBydWxlcyAx4oCTMyBjb3ZlcmluZyB0aGUgc2luZ2xlIGFjdGl2ZSBjYW5jZWxhYmxlIHR1cm4sIGBQYXJrZWRgIGNvbnRpbnVpdHksIGFuZCB0aGUgaG9zdC93b3JrZXIgcG9zdHVyZSBib3VuZGFyeQorKipTb3VyY2UgcHJvdmVuYW5jZToqKiBleHRyYWN0ZWQgYnl0ZS1mb3ItYnl0ZSBmcm9tIFtgLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM0LWFjdGl2ZXJldGFpbmVkdHVybnJlY2VpcHR2MWBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjNC1hY3RpdmVyZXRhaW5lZHR1cm5yZWNlaXB0djEpLCBiYXNlbGluZSBsaW5lcyAxMznigJMxODM7IHRoZSBleGFjdCAxNDQzLWJ5dGUgc291cmNlIGJvZHkgaXMgcHJlc2VydmVkIGJldHdlZW4gdGhlIGJvdW5kYXJ5IG1hcmtlcnMgYmVsb3cKKyoqQmFzZWxpbmUgc3BhbiBTSEEtMjU2OioqIGAyY2M5OTYxMTFkODJkMjk2Zjg5YmRkODhmODBkZjVjMGI1NTliZGIzMDVhZWM1NDY4ODFiNzQ5YWNlOTNmNjdlYAorCis8IS0tIGV4YWN0LWV4dHJhY3RlZC1ib2R5OnN0YXJ0IC0tPgorIyMgNC4gYEFjdGl2ZVJldGFpbmVkVHVyblJlY2VpcHRWMWAKKworYGBgcnVzdAorc3RydWN0IEFjdGl2ZVJldGFpbmVkVHVyblJlY2VpcHRWMSB7CisgICAgc2NoZW1hX3ZlcnNpb246IHUzMiwKKyAgICBhY2NlcHRhbmNlX3JlY29yZF9pZDogU3RyaW5nLAorICAgIGFjdGl2ZV9ydW5faWQ6IFN0cmluZywKKyAgICByZXF1ZXN0X2lkOiBTdHJpbmcsCisgICAgb3JjaGVzdHJhdGlvbl9zZXNzaW9uX2lkOiBTdHJpbmcsCisgICAgb3JjaGVzdHJhdG9yX3BhcnRpY2lwYW50X2lkOiBTdHJpbmcsCisgICAgdGFyZ2V0X3BhcnRpY2lwYW50X2lkOiBTdHJpbmcsCisgICAgdGFyZ2V0X2JhY2tlbmRfaWQ6IFN0cmluZywKKyAgICB3b3JsZF9pZDogU3RyaW5nLAorICAgIHdvcmxkX2dlbmVyYXRpb246IHU2NCwKKyAgICBtZXNzYWdlX2lkOiBTdHJpbmcsCisgICAgdGhyZWFkX2lkOiBPcHRpb248U3RyaW5nPiwKKyAgICB3b3JrZXJfcG9saWN5X2NhcF9oYXNoOiBTdHJpbmcsCisgICAgdHVybl9wb2xpY3lfc25hcHNob3RfcmVmOiBQb2xpY3lTbmFwc2hvdFJlZlYxLAorICAgIHR1cm5fcG9saWN5X3NuYXBzaG90X2hhc2g6IFN0cmluZywKKyAgICB0dXJuX3BvbGljeV9yZXZpc2lvbjogU3RyaW5nLAorICAgIG5hcnJvd2luZ19yZWFzb246IE9wdGlvbjxTdHJpbmc+LAorICAgIHJ1bnRpbWVfYWNjZXB0YW5jZTogUnVudGltZUFjY2VwdGFuY2VFdmlkZW5jZVYxLAorICAgIG9ic2VydmF0aW9uX2NsYWltOiBTdXBlcnZpc29yT2JzZXJ2YXRpb25DbGFpbVYxLAorICAgIGFjY2VwdGVkX2F0OiBUaW1lc3RhbXAsCisgICAgc3RhdGVfcmV2aXNpb246IHU2NCwKKyAgICBzdGF0ZTogQWN0aXZlUmV0YWluZWRUdXJuU3RhdGVWMSwKKyAgICBjYW5jZWxfc3VwcG9ydGVkOiBib29sLAorICAgIHRlcm1pbmFsOiBPcHRpb248V29ybGRXb3JrVGVybWluYWxWMT4sCit9CitgYGAKKworU3RhdGVzOgorCitgYGB0ZXh0CitBY2NlcHRlZCAtPiBSdW5uaW5nCitSdW5uaW5nIC0+IEF0dGVudGlvblBlbmRpbmcgLT4gUnVubmluZworQWNjZXB0ZWR8UnVubmluZ3xBdHRlbnRpb25QZW5kaW5nIC0+IFBhcmtlZHxUZXJtaW5hbHxGYWlsZWR8Q2FuY2VsbGVkfFN0b3BwZWQKK2BgYAorCitSdWxlczoKKworMS4gT25lIHJldGFpbmVkIHdvcmtlciBtYXkgaGF2ZSBhdCBtb3N0IG9uZSBhY3RpdmUgY2FuY2VsYWJsZSB0dXJuIHVubGVzcyBhIGxhdGVyIHZlcnNpb24gZXhwbGljaXRseSBtb2RlbHMgY29uY3VycmVuY3kuCisyLiBgUGFya2VkYCBjbG9zZXMgdGhlIGFjdGl2ZSB0dXJuIGJ1dCBwcmVzZXJ2ZXMgdGhlIHJldGFpbmVkIHdvcmtlciBtYW5pZmVzdCBhbmQgcmVzdW1lIGhhbmRsZS4KKzMuIEhvc3QgcG9zdHVyZSBpcyBub3QgY29waWVkIGZyb20gdHVybiBzdGF0ZS4gT2JsaWdhdGlvbnMgbWF5IGNhdXNlIGhvc3QgYEF3YWl0aW5nQXR0ZW50aW9uYDsgd29ya2VyIGBBdHRlbnRpb25QZW5kaW5nYCByZW1haW5zIHdvcmtlciB0cnV0aC4KKworPCEtLSBleGFjdC1leHRyYWN0ZWQtYm9keTplbmQgLS0+CmBgYAo=
```

</details>

## Preserved review answer

<details>
<summary>Initial-range review answer (rendered copy)</summary>

``````text
No qualifying findings.

I found no reachable, patch-causal P1 or P2 defect under the stated blocking threshold. The patch preserves the required legacy root anchors, creates exactly one canonical owner for each receipt contract, and adds exactly one navigation row per owner.

I independently verified the content-addressed portions of the supplied artifact:

the complete patch SHA-256 matches 4332a5098e977b5edda996708b0ef26143d24ebce6deeb9bbe178fe8f9d89748;

both new-file SHA-256 values match the manifest;

the marked ActiveEphemeralTaskReceiptV1 body is exactly 1,557 bytes and matches c9977526...a3b;

the marked ActiveRetainedTurnReceiptV1 body is exactly 1,443 bytes and matches 2cc99611...f67e.

The two ledger entries preserve the family as an atomic rollback unit: each requires restoring both root bodies, removing both canonical owners, removing both index rows, and removing both batch ledger entries.

The supplied validation evidence additionally reports the five-path fence, link/anchor checks, forward application, and exact reverse restoration as passing.

VERDICT: APPROVED
``````

</details>

<details>
<summary>Initial-range review answer exact bytes (base64 UTF-8)</summary>

```text
Tm8gcXVhbGlmeWluZyBmaW5kaW5ncy4KCkkgZm91bmQgbm8gcmVhY2hhYmxlLCBwYXRjaC1jYXVzYWwgUDEgb3IgUDIgZGVmZWN0IHVuZGVyIHRoZSBzdGF0ZWQgYmxvY2tpbmcgdGhyZXNob2xkLiBUaGUgcGF0Y2ggcHJlc2VydmVzIHRoZSByZXF1aXJlZCBsZWdhY3kgcm9vdCBhbmNob3JzLCBjcmVhdGVzIGV4YWN0bHkgb25lIGNhbm9uaWNhbCBvd25lciBmb3IgZWFjaCByZWNlaXB0IGNvbnRyYWN0LCBhbmQgYWRkcyBleGFjdGx5IG9uZSBuYXZpZ2F0aW9uIHJvdyBwZXIgb3duZXIuCgpJIGluZGVwZW5kZW50bHkgdmVyaWZpZWQgdGhlIGNvbnRlbnQtYWRkcmVzc2VkIHBvcnRpb25zIG9mIHRoZSBzdXBwbGllZCBhcnRpZmFjdDoKCnRoZSBjb21wbGV0ZSBwYXRjaCBTSEEtMjU2IG1hdGNoZXMgNDMzMmE1MDk4ZTk3N2I1ZWRkYTk5NjcwOGIwZWYyNjE0M2QyNGViY2U2ZGVlYjliYmUxNzhmZThmOWQ4OTc0ODsKCmJvdGggbmV3LWZpbGUgU0hBLTI1NiB2YWx1ZXMgbWF0Y2ggdGhlIG1hbmlmZXN0OwoKdGhlIG1hcmtlZCBBY3RpdmVFcGhlbWVyYWxUYXNrUmVjZWlwdFYxIGJvZHkgaXMgZXhhY3RseSAxLDU1NyBieXRlcyBhbmQgbWF0Y2hlcyBjOTk3NzUyNi4uLmEzYjsKCnRoZSBtYXJrZWQgQWN0aXZlUmV0YWluZWRUdXJuUmVjZWlwdFYxIGJvZHkgaXMgZXhhY3RseSAxLDQ0MyBieXRlcyBhbmQgbWF0Y2hlcyAyY2M5OTYxMS4uLmY2N2UuCgpUaGUgdHdvIGxlZGdlciBlbnRyaWVzIHByZXNlcnZlIHRoZSBmYW1pbHkgYXMgYW4gYXRvbWljIHJvbGxiYWNrIHVuaXQ6IGVhY2ggcmVxdWlyZXMgcmVzdG9yaW5nIGJvdGggcm9vdCBib2RpZXMsIHJlbW92aW5nIGJvdGggY2Fub25pY2FsIG93bmVycywgcmVtb3ZpbmcgYm90aCBpbmRleCByb3dzLCBhbmQgcmVtb3ZpbmcgYm90aCBiYXRjaCBsZWRnZXIgZW50cmllcy4KClRoZSBzdXBwbGllZCB2YWxpZGF0aW9uIGV2aWRlbmNlIGFkZGl0aW9uYWxseSByZXBvcnRzIHRoZSBmaXZlLXBhdGggZmVuY2UsIGxpbmsvYW5jaG9yIGNoZWNrcywgZm9yd2FyZCBhcHBsaWNhdGlvbiwgYW5kIGV4YWN0IHJldmVyc2UgcmVzdG9yYXRpb24gYXMgcGFzc2luZy4KClZFUkRJQ1Q6IEFQUFJPVkVECg==
```

</details>
