# ChatGPT Pro advisory review: runtime-refactor D10 manifest/dispatch-policy family

- Date: 2026-08-26
- Bound baseline commit/tree: `66dd1fe275145714c4157368af2250e0c0a60198` / `c8686d178fa0cfd796daea7f2347040f56376b95`
- Candidate implementation subagent (`gpt-5.4`, Extra High): `/root/d10_manifest_policy_landing` (display name not exposed to this subagent runtime and therefore not asserted here)
- Closeout subagent: `/root/d10_manifest_policy_closeout` (display name not exposed to this subagent runtime and therefore not asserted here)
- Root/orchestrator review owner: `/root`
- Independent review chat: https://chatgpt.com/c/6a8f5134-169c-83e9-904d-a5fa1c76f7b1
- Review mode: `initial-range`
- Review context: independent fresh conversation
- ChatGPT account surface: `Pro`
- Visible reasoning-effort control: `Extra High`
- Exact model label: not exposed by the current visible ChatGPT UI
- Candidate patch: `sha256:5e3878b743106c01c0870419ff697a466909ae4505e1f2d14dec5c14c53532a7` at `/private/tmp/d10-manifest-policy-family-candidate.patch`
- Review prompt: `sha256:54ec7f66139992fea868336700680211437d5ac21f53e83968321c97c25baa94` at `/private/tmp/d10-manifest-policy-family-chatgpt-pro-initial-review-prompt.txt`
- Review answer: `sha256:193b554d021508ed21f14bc05eefa1cbfb54ceed0a2f320c50d247d87faa1d30` at `/private/tmp/d10-manifest-policy-family-chatgpt-pro-initial-review-answer.txt`
- Imported validation log: `sha256:3e86fc05c750616c65536b72cabfb57c313a1633f6eea0a629d8d9c922dcd103` at `/private/tmp/d10-manifest-policy-family-validation.log`
- Exact embedded source body (`RetainedWorkerManifestV1`): baseline lines `103–140`, `1576 bytes`, `sha256:cd25ec1c19548b8442e01f753517c117cab9a350cb828de8214a351bdc09ee07`
- Exact embedded source body (`DispatchPolicyNarrowingPatchV1`): baseline lines `141–167`, `810 bytes`, `sha256:c0fd22a7c630c30e8b6fca976645d84dc7515b435af759c4ecc0bc8b8871da4b`
- Review verdict: `APPROVED`
- Review finding summary: `No qualifying P1 or P2 findings.`
- Remediation rounds: `0`
- Review adjudication posture: root agent owns review/adjudication; subagents do not self-approve
- Commit posture: the candidate remains uncommitted and unstaged during closeout; root will validate, run GitNexus, and commit the seven-path batch atomically; not yet committed
- Push posture: `not pushed`
- Successor authority posture: no successor dispatch authority is granted

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Candidate scope and outcome

This independently reviewed substantive D10 manifest/dispatch-policy family batch extracts `RetainedWorkerManifestV1` and `DispatchPolicyNarrowingPatchV1` into `llm-last-mile/runtime-refactor/contracts/retained-worker-manifest-v1.md` and `llm-last-mile/runtime-refactor/contracts/dispatch-policy-narrowing-patch-v1.md`, replaces only the two root spans in `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` with shallow compatibility pointers, adds the two truthful index rows, and adds the two matching D10 extraction-ledger rows. `Receipt acceptance source`, `HostExecutionEpisodeV1`, deferred retained-spawn admission recovery, and the active receipt family remain unchanged. The manifest/dispatch-policy family is complete locally and approved after initial-range review with zero remediation and is landed pending one atomic commit. D10 remains explicitly in progress and incomplete; the exact next batched substantive D10 unit is `WorldRuntimeAdapterExecutionEnvelopeV1` + `LaunchTimeSecretHandoffV1`; later D10 units remain queued; D11 remains blocked pending full D10 completion and separate authorization; D12 remains blocked by D11 and is separately unauthorized; no successor dispatch authority is granted.

## Prospective seven-path landing manifest

Prospective landing manifest for this independently reviewed batch:

- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/contracts/retained-worker-manifest-v1.md`
- `llm-last-mile/runtime-refactor/contracts/dispatch-policy-narrowing-patch-v1.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `docs/guidance/2026-08-26-runtime-refactor-d10-manifest-dispatch-policy-family-chatgpt-pro-review.md`

## Candidate path digests and imported review artifacts

Candidate path digests:

- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`: `sha256:3ee3e6830ecfb49fc06c2792f4e7543e64a2ff561e26e9e7a6353a545487499f`
- `llm-last-mile/runtime-refactor/contracts/retained-worker-manifest-v1.md`: `sha256:4fdf0277ee0f16dd372809e65bf6c368c36fa4a89ba4ef12a6f757ce1bbca00a`
- `llm-last-mile/runtime-refactor/contracts/dispatch-policy-narrowing-patch-v1.md`: `sha256:e600e2b6a7f6c47fb44446c8d21e032663995204f82b59c39902e589b705c0dd`
- `llm-last-mile/runtime-refactor/index/README.md`: `sha256:5eeaf3da3d0d07636db9045cae5612e29e38fd3af7a3b0913f7c489ed736675b`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`: `sha256:23daa72489a3a237bdc9cee9fc23ecfa6d03b6dcd8eace8979622f21482f4b99`

Imported review artifacts:

- Candidate patch: `sha256:5e3878b743106c01c0870419ff697a466909ae4505e1f2d14dec5c14c53532a7`
- Review prompt: `sha256:54ec7f66139992fea868336700680211437d5ac21f53e83968321c97c25baa94`
- Review answer: `sha256:193b554d021508ed21f14bc05eefa1cbfb54ceed0a2f320c50d247d87faa1d30`
- Imported validation log: `sha256:3e86fc05c750616c65536b72cabfb57c313a1633f6eea0a629d8d9c922dcd103`
- Exact embedded source body (`RetainedWorkerManifestV1`): `1576 bytes`, `sha256:cd25ec1c19548b8442e01f753517c117cab9a350cb828de8214a351bdc09ee07`
- Exact embedded source body (`DispatchPolicyNarrowingPatchV1`): `810 bytes`, `sha256:c0fd22a7c630c30e8b6fca976645d84dc7515b435af759c4ecc0bc8b8871da4b`

## Exact source-body proof

- Baseline `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` lines `103–140` are exactly `1576` bytes with SHA-256 `cd25ec1c19548b8442e01f753517c117cab9a350cb828de8214a351bdc09ee07`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` in `llm-last-mile/runtime-refactor/contracts/retained-worker-manifest-v1.md` are also exactly `1576` bytes with SHA-256 `cd25ec1c19548b8442e01f753517c117cab9a350cb828de8214a351bdc09ee07`.
- Baseline `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` lines `141–167` are exactly `810` bytes with SHA-256 `c0fd22a7c630c30e8b6fca976645d84dc7515b435af759c4ecc0bc8b8871da4b`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` in `llm-last-mile/runtime-refactor/contracts/dispatch-policy-narrowing-patch-v1.md` are also exactly `810` bytes with SHA-256 `c0fd22a7c630c30e8b6fca976645d84dc7515b435af759c4ecc0bc8b8871da4b`.
- The baseline source spans and the two owner-body byte ranges are byte-identical.
- The two ledger entries preserve the manifest/dispatch-policy family as one atomic rollback unit: each requires restoring both root bodies, removing both canonical owners, removing both index rows, removing both batch ledger entries, reverting the tracker additions, and removing this review artifact together.
- `Receipt acceptance source`, `HostExecutionEpisodeV1`, deferred retained-spawn admission recovery, and the active receipt family remain unchanged by this batch.

## Deferred review observation and adjudication

- The independent review answer included one explicitly deferred, non-blocking clarity observation about the broad `remain unchanged` / `all D3/D5–D10 owners unchanged` phrasing.
- Root adjudication accepted no remediation: the explicit per-owner supersession statements and exact paired rollback operations already remove any material ambiguity.
- The observation is style/clarity only, does not meet the material blocking threshold, and must not trigger remediation for this batch.

## Validation evidence

- The supplied review answer states `No qualifying P1 or P2 findings.` and ends with `VERDICT: APPROVED`.
- The imported validation log confirms unique root anchors for `6-retainedworkermanifestv1` and `7-dispatchpolicynarrowingpatchv1`, unique owner anchors for both canonical files, exact marker-body equality for both owners, unchanged neighboring D10 owners, a five-path candidate fence, and clean candidate-patch forward/reverse application.
- The candidate patch changes exactly the five imported candidate paths and no others.
- Candidate file hashes, candidate-patch SHA-256, prompt SHA-256, answer SHA-256, validation-log SHA-256, and both exact source-body SHA-256 digests were all reverified during this closeout.
- The rendered prompt and rendered answer below are whitespace-clean: every trailing literal space from the imported UTF-8 prompt or answer is rendered as `&#32;` so this repository-local Markdown artifact stays clean under `git diff --check` while the base64 blocks preserve the exact bytes.
- The rendered prompt and rendered answer were both revalidated against the imported files, and the embedded base64 blocks decode exactly back to the imported UTF-8 bytes without silently correcting captured whitespace or fence content.
- Final changed-path fence including untracked files is exactly these seven paths:
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/contracts/retained-worker-manifest-v1.md`
- `llm-last-mile/runtime-refactor/contracts/dispatch-policy-narrowing-patch-v1.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `docs/guidance/2026-08-26-runtime-refactor-d10-manifest-dispatch-policy-family-chatgpt-pro-review.md`
- `git diff --check` passed for the final closeout state.
- `git diff --no-index --check /dev/null` for both new owner files and the new review record reported zero whitespace diagnostics.
- Tracker truth validation passed for only the manifest/dispatch-policy family approved, D10 still incomplete, the exact next batched unit `WorldRuntimeAdapterExecutionEnvelopeV1` + `LaunchTimeSecretHandoffV1`, D11 blocked pending full D10 completion and separate authorization, D12 blocked by D11 and separately unauthorized, and no successor dispatch authority.
- Prior D3/D5–D10 owner/review-artifact stability passed via the exact seven-path fence.
- Fence-scoped Markdown link/anchor validation passed with `LINKS_CHECKED 812 FAILURES 0`.
- Complete combined candidate+closeout patch was generated at `/private/tmp/d10-manifest-policy-family-closeout.patch` with a SHA-256 sidecar, and forward/reverse apply against disposable clean baseline exports both passed.
- No commit, stage, or push is performed during this closeout.

## Atomic rollback instructions

1. If the prospective seven-path family has been committed inside an authorized later batch, reverse that eventual batch commit atomically and ensure the full manifest/dispatch-policy family rolls back together.
2. If the batch is still uncommitted, restore the tracked paths from the parent baseline and remove the three untracked files: `git restore --source=HEAD -- llm-last-mile/runtime-refactor/04-contracts-and-gates.md llm-last-mile/runtime-refactor/index/README.md llm-last-mile/runtime-refactor/migration/extraction-ledger.md docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md` then `rm -f llm-last-mile/runtime-refactor/contracts/retained-worker-manifest-v1.md llm-last-mile/runtime-refactor/contracts/dispatch-policy-narrowing-patch-v1.md docs/guidance/2026-08-26-runtime-refactor-d10-manifest-dispatch-policy-family-chatgpt-pro-review.md`.
3. After either rollback path, confirm together that both `04-contracts-and-gates.md` bodies are restored, both canonical owner files are removed, both `index/README.md` rows are removed, both `migration/extraction-ledger.md` rows are removed, the tracker summary/queue/closeout-row additions are reverted, and this review record is removed.
4. Leave `HostExecutionEpisodeV1`, deferred retained-spawn admission recovery, the active receipt family, `Receipt acceptance source`, prior D3/D5–D10 owners and review artifacts, `index/current.md`, `review-control/`, `WorldRuntimeAdapterExecutionEnvelopeV1`, `LaunchTimeSecretHandoffV1`, remaining later D10 units, and every path outside this seven-path batch unchanged.
5. Never strand one member of this two-contract family: both owners, both root compatibility pointers, both index rows, both ledger rows, the tracker update, and this review record roll back together or not at all.

## Final verdict

`RetainedWorkerManifestV1` + `DispatchPolicyNarrowingPatchV1` is complete locally and **APPROVED** after initial-range review with zero remediation. D10 remains explicitly in progress and incomplete; the exact next batched substantive D10 unit is `WorldRuntimeAdapterExecutionEnvelopeV1` + `LaunchTimeSecretHandoffV1`; later D10 units remain queued; D11 remains blocked pending full D10 completion and separate authorization; D12 remains blocked by D11 and is separately unauthorized; no successor dispatch authority is granted.

## Preserved review prompt

<details>
<summary>Initial-range review prompt (rendered copy)</summary>

``````text
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: exact baseline commit 66dd1fe275145714c4157368af2250e0c0a60198, tree c8686d178fa0cfd796daea7f2347040f56376b95, plus the complete content-addressed candidate patch below. Candidate patch SHA-256: 5e3878b743106c01c0870419ff697a466909ae4505e1f2d14dec5c14c53532a7.
Review target: the complete bounded D10 documentation extraction batch for the inseparable manifest/dispatch-policy family. Review every hunk and both new files; do not infer unprovided repository source.

Manifest (exactly five changed paths, including untracked new files):
- llm-last-mile/runtime-refactor/04-contracts-and-gates.md — SHA-256 3ee3e6830ecfb49fc06c2792f4e7543e64a2ff561e26e9e7a6353a545487499f
- llm-last-mile/runtime-refactor/contracts/retained-worker-manifest-v1.md — new — SHA-256 4fdf0277ee0f16dd372809e65bf6c368c36fa4a89ba4ef12a6f757ce1bbca00a
- llm-last-mile/runtime-refactor/contracts/dispatch-policy-narrowing-patch-v1.md — new — SHA-256 e600e2b6a7f6c47fb44446c8d21e032663995204f82b59c39902e589b705c0dd
- llm-last-mile/runtime-refactor/index/README.md — SHA-256 5eeaf3da3d0d07636db9045cae5612e29e38fd3af7a3b0913f7c489ed736675b
- llm-last-mile/runtime-refactor/migration/extraction-ledger.md — SHA-256 23daa72489a3a237bdc9cee9fc23ecfa6d03b6dcd8eace8979622f21482f4b99

Supported inputs and behavior: GitHub-flavored Markdown documentation with ATX headings and generated heading anchors, relative Markdown links with optional fragments, fenced Rust blocks, Markdown tables whose row/cell order is semantic, ordered lists, inline code/literals, and HTML comments used as extraction-body hash markers. The legacy root remains a compatibility projection until a separately authorized D12 cutover. Canonical owners are normative; root compatibility headings/anchors must continue to resolve.
Supported platforms and dialects: repository Markdown rendered with GitHub-style heading slug behavior and validated by the project's Markdown link/anchor checker. No runtime, Rust implementation, script, platform, parser, dispatch, or policy execution behavior is in scope.

In-scope invariants:
1. Exactly one canonical owner exists for each of RetainedWorkerManifestV1 and DispatchPolicyNarrowingPatchV1; they are one atomic D10 landing family.
2. Each canonical body is byte-for-byte identical to its frozen live-root source span, preserving names/version suffixes, Rust field order, enum variant order, ordered rules, fenced code, literals, negative requirements, rejection behavior, semantic status, and authority boundaries.
3. Frozen facts: RetainedWorkerManifestV1 source/body is 1576 bytes, SHA-256 cd25ec1c19548b8442e01f753517c117cab9a350cb828de8214a351bdc09ee07; DispatchPolicyNarrowingPatchV1 source/body is 810 bytes, SHA-256 c0fd22a7c630c30e8b6fca976645d84dc7515b435af759c4ecc0bc8b8871da4b.
4. RetainedWorkerManifestV1 must preserve rules 1–5, including maximum worker cap, fork narrowing/no broadening, parked identity/resume continuity, world-generation mismatch invalidation/no silent rebind, and the later-lifecycle ownership boundary that cannot mutate immutable B1 accepted-turn identity.
5. DispatchPolicyNarrowingPatchV1 must preserve the complete schema, DispatchCapabilitySubjectV1 enum and variant order, and the rule that V1 RestrictedPolicyPatchV1 may contain only world_fs fields; unknown or non-world_fs fields are rejected, not ignored.
6. Root headings/legacy anchors `6-retainedworkermanifestv1` and `7-dispatchpolicynarrowingpatchv1` must remain unique and resolving, with only their duplicated bodies replaced by truthful links to the two canonical owners.
7. Navigation contains exactly one correct row per owner. Ledger contains exactly one truthful D10 row per owner with exact source lines/bytes/hashes and complete family-atomic rollback restoring both root bodies and removing both owners, both index rows, and both ledger rows together.
8. Receipt acceptance source, WorldRuntimeAdapterExecutionEnvelopeV1, prior D3/D5-D10 owners, review artifacts, and every path outside the exact five-path fence remain stable.
9. Patch claims no implementation, evidence, gate satisfaction, promotion, dispatch/successor authority, D10 completion, D11 authority, or D12 cutover.
10. No Markdown normalization, compound-contract splitting, packet-local D5/D6 re-extraction, or unrelated edits are allowed.

Required material consequence: a blocking finding must show that a supported repository reader materially loses, alters, duplicates, or misidentifies contract semantics/canonical ownership; that a required link/anchor/provenance/atomic rollback breaks; that status/authority becomes false; that the exact path fence is violated; or that clean application/reversal fails. Style preferences, optional wording, broader redesign, theoretical unsupported Markdown behavior, and unchanged baseline defects are not material.
Blocking threshold: report only reachable, patch-causal P1 or P2 defects that violate an in-scope invariant and meet the material-consequence threshold. Body/hash mismatch, field/variant/rule reordering or loss, weakened rejection/narrowing requirement, wrong owner/target, broken required anchor/link, incomplete atomic rollback, false authority/status, or scope breach qualifies. Label non-blocking observations deferred; they must not change the verdict.
Accepted prior findings: none.
Deferred or out-of-scope concerns: D3 review-governance; packet-local D5/D6 content already canonically owned; prior D9 slices/tasks; D11 evidence decomposition; D12 root cutover; Rust/scripts/runtime/platform behavior; roadmap or reviewed ZIP changes; global Markdown formatting; unrelated pre-existing issues; style-only changes; implementation and successor dispatch.
Project sources: only this prompt, manifest, invariants, validation evidence, and the complete patch below are available. Excluded source is unavailable and must not be inferred.

Validation evidence:
- exact five-path fence including both untracked files; no staged files: PASS
- `git diff --check`: PASS
- no-index whitespace checks for both new owners: ordinary difference exit, zero diagnostics
- Markdown link/anchor validation: 1,847 links checked, 0 failures
- both root legacy anchors and both owner anchors: unique/resolving
- marker-body equality and hashes: 1576/1576 bytes cd25... equal; 810/810 bytes c0fd... equal
- field order, enum order, numbered-rule order, fenced code, literals, world_fs-only rejection rule, and negative requirements: PASS
- root pointers, exactly one index row each, exactly one ledger row each, and family-atomic rollback text: PASS
- prior owners/review artifacts and all outside paths: stable
- patch forward-apply/exact five-file content match and reverse-apply/exact clean-baseline restoration: PASS
- validation log SHA-256: 3e86fc05c750616c65536b72cabfb57c313a1633f6eea0a629d8d9c922dcd103

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return findings first by severity. For every blocking finding, state exact file/hunk, supported-input reachability, violated invariant, material consequence, and patch causality. State explicitly when there are no qualifying findings. End with exactly one of:
VERDICT: APPROVED
VERDICT: CHANGES REQUIRED

Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

COMPLETE CONTENT-ADDRESSED PATCH (SHA-256 5e3878b743106c01c0870419ff697a466909ae4505e1f2d14dec5c14c53532a7):
```diff
diff --git a/llm-last-mile/runtime-refactor/04-contracts-and-gates.md b/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
index a9e7c32..f136cb6 100644
--- a/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
+++ b/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
@@ -102,68 +102,11 @@ Canonical content: [`b1-b2-1/contracts-and-gates.md#5-receipt-acceptance-source`
&#32;
 ## 6. `RetainedWorkerManifestV1`
&#32;
-```rust
-struct RetainedWorkerManifestV1 {
-    schema_version: u32,
-    retained_participant_id: String,
-    orchestration_session_id: String,
-    orchestrator_participant_id: String,
-    parent_retained_participant_id: Option<String>,
-    target_backend_id: String,
-    world_id: String,
-    world_generation: u64,
-    runtime_family: String,
-    resume_handle_ref: Option<ResumeHandleRefV1>,
-    worker_policy_cap_ref: PolicySnapshotRefV1,
-    worker_policy_cap_hash: String,
-    worker_policy_cap_snapshot: PolicySnapshotV3,
-    config_projection_identity: ConfigProjectionIdentityV1,
-    config_projection_ref: ConfigProjectionRefV1,
-    execution_envelope_ref: ExecutionEnvelopeRefV1,
-    lifecycle_state: RetainedWorkerLifecycleStateV1,
-    active_turn_ref: Option<ActiveRunRefV1>,
-    manifest_revision: u64,
-    created_at: Timestamp,
-    updated_at: Timestamp,
-}
-```
-
-Rules:
-
-1. Spawn-time effective policy, including spawn narrowing, becomes the maximum worker cap.
-2. Fork inherits the source cap by default and may narrow further; it cannot broaden.
-3. Clean turn exit may park the worker. It must not delete retained identity or resume continuity.
-4. World generation mismatch makes the worker unroutable/invalidated; it does not silently rebind.
-5. `active_turn_ref` is a later RetainedWorkerRuntime lifecycle reference to the immutable B1
-   accepted record. B1 does not create, update, or close it; the later retained-lifecycle packet
-   owns that separate atomic lifecycle transition without mutating accepted-turn identity.
+Canonical content: [`contracts/retained-worker-manifest-v1.md#6-retainedworkermanifestv1`](contracts/retained-worker-manifest-v1.md#6-retainedworkermanifestv1).
&#32;
 ## 7. `DispatchPolicyNarrowingPatchV1`
&#32;
-```rust
-struct DispatchPolicyNarrowingPatchV1 {
-    schema_version: u32,
-    request_id: String,
-    orchestration_session_id: String,
-    caller_participant_id: String,
-    target_backend_id: String,
-    target_world: WorldBindingRefV1,
-    applies_to: DispatchCapabilitySubjectV1,
-    parent_policy_ref: PolicyRefV1,
-    parent_policy_revision: String,
-    restricted_policy_patch: RestrictedPolicyPatchV1,
-    reason: Option<String>,
-}
-
-enum DispatchCapabilitySubjectV1 {
-    EphemeralTask,
-    RetainedWorkerSpawn,
-    RetainedWorkerTurn { retained_participant_id: String },
-    RetainedWorkerFork { source_participant_id: String },
-}
-```
-
-V1 `RestrictedPolicyPatchV1` may contain only `world_fs` fields. Unknown or non-`world_fs` fields are rejected, not ignored.
+Canonical content: [`contracts/dispatch-policy-narrowing-patch-v1.md#7-dispatchpolicynarrowingpatchv1`](contracts/dispatch-policy-narrowing-patch-v1.md#7-dispatchpolicynarrowingpatchv1).
&#32;
 ## 8. `WorldRuntimeAdapterExecutionEnvelopeV1`
&#32;
diff --git a/llm-last-mile/runtime-refactor/contracts/retained-worker-manifest-v1.md b/llm-last-mile/runtime-refactor/contracts/retained-worker-manifest-v1.md
new file mode 100644
index 0000000..afbc010
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/contracts/retained-worker-manifest-v1.md
@@ -0,0 +1,46 @@
+**Kind:** contract
+**Status:** canonical
+**Canonical for:** complete extracted `RetainedWorkerManifestV1` schema and rules 1–5 covering spawn-time cap narrowing, fork narrowing, parked identity continuity, world-generation invalidation, and the immutable B1 accepted-record boundary
+**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#6-retainedworkermanifestv1`](../04-contracts-and-gates.md#6-retainedworkermanifestv1), baseline lines 103–140; the exact 1576-byte source body is preserved between the boundary markers below
+**Baseline span SHA-256:** `cd25ec1c19548b8442e01f753517c117cab9a350cb828de8214a351bdc09ee07`
+
+<!-- exact-extracted-body:start -->
+## 6. `RetainedWorkerManifestV1`
+
+```rust
+struct RetainedWorkerManifestV1 {
+    schema_version: u32,
+    retained_participant_id: String,
+    orchestration_session_id: String,
+    orchestrator_participant_id: String,
+    parent_retained_participant_id: Option<String>,
+    target_backend_id: String,
+    world_id: String,
+    world_generation: u64,
+    runtime_family: String,
+    resume_handle_ref: Option<ResumeHandleRefV1>,
+    worker_policy_cap_ref: PolicySnapshotRefV1,
+    worker_policy_cap_hash: String,
+    worker_policy_cap_snapshot: PolicySnapshotV3,
+    config_projection_identity: ConfigProjectionIdentityV1,
+    config_projection_ref: ConfigProjectionRefV1,
+    execution_envelope_ref: ExecutionEnvelopeRefV1,
+    lifecycle_state: RetainedWorkerLifecycleStateV1,
+    active_turn_ref: Option<ActiveRunRefV1>,
+    manifest_revision: u64,
+    created_at: Timestamp,
+    updated_at: Timestamp,
+}
+```
+
+Rules:
+
+1. Spawn-time effective policy, including spawn narrowing, becomes the maximum worker cap.
+2. Fork inherits the source cap by default and may narrow further; it cannot broaden.
+3. Clean turn exit may park the worker. It must not delete retained identity or resume continuity.
+4. World generation mismatch makes the worker unroutable/invalidated; it does not silently rebind.
+5. `active_turn_ref` is a later RetainedWorkerRuntime lifecycle reference to the immutable B1
+   accepted record. B1 does not create, update, or close it; the later retained-lifecycle packet
+   owns that separate atomic lifecycle transition without mutating accepted-turn identity.
+
+<!-- exact-extracted-body:end -->
diff --git a/llm-last-mile/runtime-refactor/contracts/dispatch-policy-narrowing-patch-v1.md b/llm-last-mile/runtime-refactor/contracts/dispatch-policy-narrowing-patch-v1.md
new file mode 100644
index 0000000..b108367
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/contracts/dispatch-policy-narrowing-patch-v1.md
@@ -0,0 +1,35 @@
+**Kind:** contract
+**Status:** canonical
+**Canonical for:** complete extracted `DispatchPolicyNarrowingPatchV1` schema, `DispatchCapabilitySubjectV1` enum, and the V1 `RestrictedPolicyPatchV1` `world_fs`-only rejection rule
+**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#7-dispatchpolicynarrowingpatchv1`](../04-contracts-and-gates.md#7-dispatchpolicynarrowingpatchv1), baseline lines 141–167; the exact 810-byte source body is preserved between the boundary markers below
+**Baseline span SHA-256:** `c0fd22a7c630c30e8b6fca976645d84dc7515b435af759c4ecc0bc8b8871da4b`
+
+<!-- exact-extracted-body:start -->
+## 7. `DispatchPolicyNarrowingPatchV1`
+
+```rust
+struct DispatchPolicyNarrowingPatchV1 {
+    schema_version: u32,
+    request_id: String,
+    orchestration_session_id: String,
+    caller_participant_id: String,
+    target_backend_id: String,
+    target_world: WorldBindingRefV1,
+    applies_to: DispatchCapabilitySubjectV1,
+    parent_policy_ref: PolicyRefV1,
+    parent_policy_revision: String,
+    restricted_policy_patch: RestrictedPolicyPatchV1,
+    reason: Option<String>,
+}
+
+enum DispatchCapabilitySubjectV1 {
+    EphemeralTask,
+    RetainedWorkerSpawn,
+    RetainedWorkerTurn { retained_participant_id: String },
+    RetainedWorkerFork { source_participant_id: String },
+}
+```
+
+V1 `RestrictedPolicyPatchV1` may contain only `world_fs` fields. Unknown or non-`world_fs` fields are rejected, not ignored.
+
+<!-- exact-extracted-body:end -->
diff --git a/llm-last-mile/runtime-refactor/index/README.md b/llm-last-mile/runtime-refactor/index/README.md
index 00c862d..a41a479 100644
--- a/llm-last-mile/runtime-refactor/index/README.md
+++ b/llm-last-mile/runtime-refactor/index/README.md
@@ -21,6 +21,8 @@
 | `Deferred retained-spawn admission recovery contract` | contract | [`contracts/deferred-retained-spawn-admission-recovery-contract.md`](../contracts/deferred-retained-spawn-admission-recovery-contract.md) | `canonical extracted owner for the complete deferred retained-spawn admission recovery contract, paired V1 recovery schemas, compatibility boundaries, route exclusions, allowlists, and recorded-result limits` | Supersedes only root-canonical ownership of the extracted `Deferred retained-spawn admission recovery contract` span; the two prerequisite compatibility anchors, `HostExecutionEpisodeV1`, and the following `B3.2a-WA ExactBoundWorldOwnershipAdoptionV1` owner remain unchanged. | [`04`](../04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract) |
 | `ActiveEphemeralTaskReceiptV1` | contract | [`contracts/active-ephemeral-task-receipt-v1.md`](../contracts/active-ephemeral-task-receipt-v1.md) | `canonical extracted contract owner for the complete schema, state transitions, terminal monotonicity, result-class literals, and explicit NeedsRetainedFollowup constraints` | Supersedes only root-canonical ownership of the extracted `ActiveEphemeralTaskReceiptV1` span; `Receipt acceptance source` and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1) |
 | `ActiveRetainedTurnReceiptV1` | contract | [`contracts/active-retained-turn-receipt-v1.md`](../contracts/active-retained-turn-receipt-v1.md) | `canonical extracted contract owner for the complete schema, state transitions, and rules 1–3 covering the single active cancelable turn, Parked continuity, and the host/worker posture boundary` | Supersedes only root-canonical ownership of the extracted `ActiveRetainedTurnReceiptV1` span; `Receipt acceptance source` and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#4-activeretainedturnreceiptv1) |
+| `RetainedWorkerManifestV1` | contract | [`contracts/retained-worker-manifest-v1.md`](../contracts/retained-worker-manifest-v1.md) | `canonical extracted contract owner for the complete schema and rules 1–5 covering worker-cap narrowing, parked identity continuity, world-generation invalidation, and the immutable B1 accepted-record boundary` | Supersedes only root-canonical ownership of the extracted `RetainedWorkerManifestV1` span; `Receipt acceptance source`, `DispatchPolicyNarrowingPatchV1`, and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#6-retainedworkermanifestv1) |
+| `DispatchPolicyNarrowingPatchV1` | contract | [`contracts/dispatch-policy-narrowing-patch-v1.md`](../contracts/dispatch-policy-narrowing-patch-v1.md) | `canonical extracted contract owner for the complete schema, `DispatchCapabilitySubjectV1` enum, and the V1 `RestrictedPolicyPatchV1` world_fs-only rejection rule` | Supersedes only root-canonical ownership of the extracted `DispatchPolicyNarrowingPatchV1` span; `RetainedWorkerManifestV1` and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#7-dispatchpolicynarrowingpatchv1) |
 | `shared-target-architecture` | architecture index | [`architecture/README.md`](../architecture/README.md) | `non-authoritative navigation for the extracted executive target decision, authority map, numbered invariants, and stable review question` | Supersedes only root-canonical ownership of the extracted shared architecture spans; packet-family-local D5/D6 forwarders remain at their existing owners. | [`01`](../01-target-architecture.md#executive-decision), [`01`](../01-target-architecture.md#authority-map), [`01`](../01-target-architecture.md#non-negotiable-invariants), [`01`](../01-target-architecture.md#review-question) |
 | `shared-seam-crosswalk` | seam index | [`seams/README.md`](../seams/README.md) | `canonical shared seam-crosswalk rules plus extracted seam-family navigation for host/session, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation` | Supersedes only the extracted root reading-rule/classification spans and the extracted host/session authority, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation family rows; A0 and the existing D6 `HostSessionAuthority`, `WorldWorkerMessagingProtocol`, and `ObligationLedger` compatibility rows remain at their current owners. | [`02`](../02-seam-crosswalk.md#reading-rule), [`02`](../02-seam-crosswalk.md#a-host-authority-and-ingress), [`02`](../02-seam-crosswalk.md#b-dispatch-policy-receipts-and-retained-runtime), [`02`](../02-seam-crosswalk.md#c-obligations-and-host-re-engagement), [`02`](../02-seam-crosswalk.md#d-uaa-realization-projection-and-side-effect-mediation), [`02`](../02-seam-crosswalk.md#classification-consequences) |
 | `shared-slice-map` | slice index | [`slices/README.md`](../slices/README.md) | `canonical shared sequencing/dependency and slice-closeout owner plus non-authoritative D9 Track A–E navigation and A1/A1.4 projection index` | Supersedes only root-canonical ownership of the extracted shared sequencing/closeout spans plus the extracted Track A–E and A1/A1.4 projection spans; controlling schedule authority remains with decisions, packets, gates, and `current.md`. | [`03`](../03-phase-slice-map.md#sequencing-rules), [`03`](../03-phase-slice-map.md#track-a--authority-and-surface-neutrality), [`03`](../03-phase-slice-map.md#a1-bounded-packet-decomposition), [`03`](../03-phase-slice-map.md#track-b--world-dispatch-receipts-supervision-and-cancel), [`03`](../03-phase-slice-map.md#track-c--obligations-inbox-auto-attach-and-router-attach), [`03`](../03-phase-slice-map.md#track-d--uaa-execution-envelope-and-side-effect-mediation), [`03`](../03-phase-slice-map.md#track-e--dispatch-scoped-policy-narrowing-and-config-projection), [`03`](../03-phase-slice-map.md#slice-closeout-minimum) |
diff --git a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
index 16fd610..f5566e6 100644
--- a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
+++ b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
@@ -188,3 +188,5 @@ This ledger records content-preserving authority transfers while retaining requi
 | D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | Deferred retained-spawn admission recovery contract | [`deferred-retained-spawn-admission-recovery-contract`](../04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract) | lines 71–213 | `d903299bdd345316fd0943d28abc300a1cef001ce0744746f72f53607eb64c12` | [`../contracts/deferred-retained-spawn-admission-recovery-contract.md`](../contracts/deferred-retained-spawn-admission-recovery-contract.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 9896-byte source span at `04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract`; remove `contracts/deferred-retained-spawn-admission-recovery-contract.md`; remove the exact `Deferred retained-spawn admission recovery contract` row from `index/README.md`; remove this D10 ledger entry; leave `HostExecutionEpisodeV1`, the two prerequisite compatibility anchors, all D3/D5–D9 owners, `index/current.md`, `review-control/`, later D10 units, and every path outside the four-path fence unchanged |
 | D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 3. `ActiveEphemeralTaskReceiptV1` | [`3-activeephemeraltaskreceiptv1`](../04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1) | lines 91–138 | `c997752654f041de820210f03c86706551b44389994df5e8e0341aa41a34fa3b` | [`../contracts/active-ephemeral-task-receipt-v1.md`](../contracts/active-ephemeral-task-receipt-v1.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1557-byte source span at `04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1` and the exact 1443-byte source span at `04-contracts-and-gates.md#4-activeretainedturnreceiptv1`; remove `contracts/active-ephemeral-task-receipt-v1.md` and `contracts/active-retained-turn-receipt-v1.md`; remove the exact `ActiveEphemeralTaskReceiptV1` and `ActiveRetainedTurnReceiptV1` rows from `index/README.md`; remove both D10 ledger entries for this batch; leave `HostExecutionEpisodeV1`, deferred retained-spawn admission recovery, `Receipt acceptance source`, all D3/D5–D9 owners, `index/current.md`, `review-control/`, remaining D10 units, and every path outside the five-path fence unchanged |
 | D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 4. `ActiveRetainedTurnReceiptV1` | [`4-activeretainedturnreceiptv1`](../04-contracts-and-gates.md#4-activeretainedturnreceiptv1) | lines 139–183 | `2cc996111d82d296f89bdd88f80df5c0b559bdb305aec546881b749ace93f67e` | [`../contracts/active-retained-turn-receipt-v1.md`](../contracts/active-retained-turn-receipt-v1.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1557-byte source span at `04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1` and the exact 1443-byte source span at `04-contracts-and-gates.md#4-activeretainedturnreceiptv1`; remove `contracts/active-ephemeral-task-receipt-v1.md` and `contracts/active-retained-turn-receipt-v1.md`; remove the exact `ActiveEphemeralTaskReceiptV1` and `ActiveRetainedTurnReceiptV1` rows from `index/README.md`; remove both D10 ledger entries for this batch; leave `HostExecutionEpisodeV1`, deferred retained-spawn admission recovery, `Receipt acceptance source`, all D3/D5–D9 owners, `index/current.md`, `review-control/`, remaining D10 units, and every path outside the five-path fence unchanged |
+| D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 6. `RetainedWorkerManifestV1` | [`6-retainedworkermanifestv1`](../04-contracts-and-gates.md#6-retainedworkermanifestv1) | lines 103–140 (1576 bytes) | `cd25ec1c19548b8442e01f753517c117cab9a350cb828de8214a351bdc09ee07` | [`../contracts/retained-worker-manifest-v1.md`](../contracts/retained-worker-manifest-v1.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1576-byte source span at `04-contracts-and-gates.md#6-retainedworkermanifestv1` and the exact 810-byte source span at `04-contracts-and-gates.md#7-dispatchpolicynarrowingpatchv1`; remove `contracts/retained-worker-manifest-v1.md` and `contracts/dispatch-policy-narrowing-patch-v1.md`; remove the exact `RetainedWorkerManifestV1` and `DispatchPolicyNarrowingPatchV1` rows from `index/README.md`; remove both D10 ledger entries for this batch; leave `Receipt acceptance source`, `WorldRuntimeAdapterExecutionEnvelopeV1`, all D3/D5–D10 owners, `index/current.md`, `review-control/`, remaining later D10 units, and every path outside the five-path fence unchanged |
+| D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 7. `DispatchPolicyNarrowingPatchV1` | [`7-dispatchpolicynarrowingpatchv1`](../04-contracts-and-gates.md#7-dispatchpolicynarrowingpatchv1) | lines 141–167 (810 bytes) | `c0fd22a7c630c30e8b6fca976645d84dc7515b435af759c4ecc0bc8b8871da4b` | [`../contracts/dispatch-policy-narrowing-patch-v1.md`](../contracts/dispatch-policy-narrowing-patch-v1.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1576-byte source span at `04-contracts-and-gates.md#6-retainedworkermanifestv1` and the exact 810-byte source span at `04-contracts-and-gates.md#7-dispatchpolicynarrowingpatchv1`; remove `contracts/retained-worker-manifest-v1.md` and `contracts/dispatch-policy-narrowing-patch-v1.md`; remove the exact `RetainedWorkerManifestV1` and `DispatchPolicyNarrowingPatchV1` rows from `index/README.md`; remove both D10 ledger entries for this batch; leave `Receipt acceptance source`, `WorldRuntimeAdapterExecutionEnvelopeV1`, all D3/D5–D10 owners, `index/current.md`, `review-control/`, remaining later D10 units, and every path outside the five-path fence unchanged |
```
``````

</details>

<details>
<summary>Initial-range review prompt exact bytes (base64 UTF-8)</summary>

```text
UmV2aWV3IG1vZGU6IGluaXRpYWwtcmFuZ2UKUmV2aWV3IGNvbnRleHQ6IGluZGVwZW5kZW50IGZy
ZXNoIGNvbnZlcnNhdGlvbgpSZXZpZXcgYm91bmRhcnk6IGV4YWN0IGJhc2VsaW5lIGNvbW1pdCA2
NmRkMWZlMjc1MTQ1NzE0YzQxNTczNjhhZjIyNTBlMGMwYTYwMTk4LCB0cmVlIGM4Njg2ZDE3OGZh
MGNmZDc5NmRhZWE3ZjIzNDcwNDBmNTYzNzZiOTUsIHBsdXMgdGhlIGNvbXBsZXRlIGNvbnRlbnQt
YWRkcmVzc2VkIGNhbmRpZGF0ZSBwYXRjaCBiZWxvdy4gQ2FuZGlkYXRlIHBhdGNoIFNIQS0yNTY6
IDVlMzg3OGI3NDMxMDZjMDFjMDg3MDQxOWZmNjk3YTQ2NjkwOWFlNDUwNWUxZjJkMTRkZWM1YzE0
YzUzNTMyYTcuClJldmlldyB0YXJnZXQ6IHRoZSBjb21wbGV0ZSBib3VuZGVkIEQxMCBkb2N1bWVu
dGF0aW9uIGV4dHJhY3Rpb24gYmF0Y2ggZm9yIHRoZSBpbnNlcGFyYWJsZSBtYW5pZmVzdC9kaXNw
YXRjaC1wb2xpY3kgZmFtaWx5LiBSZXZpZXcgZXZlcnkgaHVuayBhbmQgYm90aCBuZXcgZmlsZXM7
IGRvIG5vdCBpbmZlciB1bnByb3ZpZGVkIHJlcG9zaXRvcnkgc291cmNlLgoKTWFuaWZlc3QgKGV4
YWN0bHkgZml2ZSBjaGFuZ2VkIHBhdGhzLCBpbmNsdWRpbmcgdW50cmFja2VkIG5ldyBmaWxlcyk6
Ci0gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMu
bWQg4oCUIFNIQS0yNTYgM2VlM2U2ODMwZWNmYjQ5ZmMwNmMyNzkyZjRlNzU0M2U2NGEyZmY1NjFl
MjZlOWU3YTYzNTNhNTQ1NDg3NDk5ZgotIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9j
b250cmFjdHMvcmV0YWluZWQtd29ya2VyLW1hbmlmZXN0LXYxLm1kIOKAlCBuZXcg4oCUIFNIQS0y
NTYgNGZkZjAyNzdlZTBmMTZkZDM3MjgwOWU2NWJmNmMzNjhjMzZmYTRhODliYTRlZjEyYTZmNzU3
Y2UxYmJjYTAwYQotIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9jb250cmFjdHMvZGlz
cGF0Y2gtcG9saWN5LW5hcnJvd2luZy1wYXRjaC12MS5tZCDigJQgbmV3IOKAlCBTSEEtMjU2IGU2
MDBlMmI2YTdmNmM0N2ZiNDQ0NDZjOGQyMWUwMzI2NjM5OTUyMDRmODJiNTljMzk5MDJlNTg5Yjcw
NWMwZGQKLSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvaW5kZXgvUkVBRE1FLm1kIOKA
lCBTSEEtMjU2IDVlZWFmM2RhM2QwZDA3NjM2ZGI5MDQ1Y2FlNTYxMmUyOWUzOGZkM2FmN2EzYjA5
MTNmN2M0ODllZDczNjY3NWIKLSBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0
aW9uL2V4dHJhY3Rpb24tbGVkZ2VyLm1kIOKAlCBTSEEtMjU2IDIzZGFhNzI0ODlhM2EyMzdiZGM5
Y2VlOWZjMjNlY2ZhNmQwM2I2ZGNkOGVhY2U4OTc5NjIyZjIxNDgyZjRiOTkKClN1cHBvcnRlZCBp
bnB1dHMgYW5kIGJlaGF2aW9yOiBHaXRIdWItZmxhdm9yZWQgTWFya2Rvd24gZG9jdW1lbnRhdGlv
biB3aXRoIEFUWCBoZWFkaW5ncyBhbmQgZ2VuZXJhdGVkIGhlYWRpbmcgYW5jaG9ycywgcmVsYXRp
dmUgTWFya2Rvd24gbGlua3Mgd2l0aCBvcHRpb25hbCBmcmFnbWVudHMsIGZlbmNlZCBSdXN0IGJs
b2NrcywgTWFya2Rvd24gdGFibGVzIHdob3NlIHJvdy9jZWxsIG9yZGVyIGlzIHNlbWFudGljLCBv
cmRlcmVkIGxpc3RzLCBpbmxpbmUgY29kZS9saXRlcmFscywgYW5kIEhUTUwgY29tbWVudHMgdXNl
ZCBhcyBleHRyYWN0aW9uLWJvZHkgaGFzaCBtYXJrZXJzLiBUaGUgbGVnYWN5IHJvb3QgcmVtYWlu
cyBhIGNvbXBhdGliaWxpdHkgcHJvamVjdGlvbiB1bnRpbCBhIHNlcGFyYXRlbHkgYXV0aG9yaXpl
ZCBEMTIgY3V0b3Zlci4gQ2Fub25pY2FsIG93bmVycyBhcmUgbm9ybWF0aXZlOyByb290IGNvbXBh
dGliaWxpdHkgaGVhZGluZ3MvYW5jaG9ycyBtdXN0IGNvbnRpbnVlIHRvIHJlc29sdmUuClN1cHBv
cnRlZCBwbGF0Zm9ybXMgYW5kIGRpYWxlY3RzOiByZXBvc2l0b3J5IE1hcmtkb3duIHJlbmRlcmVk
IHdpdGggR2l0SHViLXN0eWxlIGhlYWRpbmcgc2x1ZyBiZWhhdmlvciBhbmQgdmFsaWRhdGVkIGJ5
IHRoZSBwcm9qZWN0J3MgTWFya2Rvd24gbGluay9hbmNob3IgY2hlY2tlci4gTm8gcnVudGltZSwg
UnVzdCBpbXBsZW1lbnRhdGlvbiwgc2NyaXB0LCBwbGF0Zm9ybSwgcGFyc2VyLCBkaXNwYXRjaCwg
b3IgcG9saWN5IGV4ZWN1dGlvbiBiZWhhdmlvciBpcyBpbiBzY29wZS4KCkluLXNjb3BlIGludmFy
aWFudHM6CjEuIEV4YWN0bHkgb25lIGNhbm9uaWNhbCBvd25lciBleGlzdHMgZm9yIGVhY2ggb2Yg
UmV0YWluZWRXb3JrZXJNYW5pZmVzdFYxIGFuZCBEaXNwYXRjaFBvbGljeU5hcnJvd2luZ1BhdGNo
VjE7IHRoZXkgYXJlIG9uZSBhdG9taWMgRDEwIGxhbmRpbmcgZmFtaWx5LgoyLiBFYWNoIGNhbm9u
aWNhbCBib2R5IGlzIGJ5dGUtZm9yLWJ5dGUgaWRlbnRpY2FsIHRvIGl0cyBmcm96ZW4gbGl2ZS1y
b290IHNvdXJjZSBzcGFuLCBwcmVzZXJ2aW5nIG5hbWVzL3ZlcnNpb24gc3VmZml4ZXMsIFJ1c3Qg
ZmllbGQgb3JkZXIsIGVudW0gdmFyaWFudCBvcmRlciwgb3JkZXJlZCBydWxlcywgZmVuY2VkIGNv
ZGUsIGxpdGVyYWxzLCBuZWdhdGl2ZSByZXF1aXJlbWVudHMsIHJlamVjdGlvbiBiZWhhdmlvciwg
c2VtYW50aWMgc3RhdHVzLCBhbmQgYXV0aG9yaXR5IGJvdW5kYXJpZXMuCjMuIEZyb3plbiBmYWN0
czogUmV0YWluZWRXb3JrZXJNYW5pZmVzdFYxIHNvdXJjZS9ib2R5IGlzIDE1NzYgYnl0ZXMsIFNI
QS0yNTYgY2QyNWVjMWMxOTU0OGI4NDQyZTAxZjc1MzUxN2MxMTdjYWI5YTM1MGNiODI4ZGU4MjE0
YTM1MWJkYzA5ZWUwNzsgRGlzcGF0Y2hQb2xpY3lOYXJyb3dpbmdQYXRjaFYxIHNvdXJjZS9ib2R5
IGlzIDgxMCBieXRlcywgU0hBLTI1NiBjMGZkMjJhN2M2MzBjMzBlOGI2ZmNhOTc2NjQ1ZDg0ZGM3
NTE1YjQzNWFmNzU5YzRlY2MwYmM4Yjg4NzFkYTRiLgo0LiBSZXRhaW5lZFdvcmtlck1hbmlmZXN0
VjEgbXVzdCBwcmVzZXJ2ZSBydWxlcyAx4oCTNSwgaW5jbHVkaW5nIG1heGltdW0gd29ya2VyIGNh
cCwgZm9yayBuYXJyb3dpbmcvbm8gYnJvYWRlbmluZywgcGFya2VkIGlkZW50aXR5L3Jlc3VtZSBj
b250aW51aXR5LCB3b3JsZC1nZW5lcmF0aW9uIG1pc21hdGNoIGludmFsaWRhdGlvbi9ubyBzaWxl
bnQgcmViaW5kLCBhbmQgdGhlIGxhdGVyLWxpZmVjeWNsZSBvd25lcnNoaXAgYm91bmRhcnkgdGhh
dCBjYW5ub3QgbXV0YXRlIGltbXV0YWJsZSBCMSBhY2NlcHRlZC10dXJuIGlkZW50aXR5Lgo1LiBE
aXNwYXRjaFBvbGljeU5hcnJvd2luZ1BhdGNoVjEgbXVzdCBwcmVzZXJ2ZSB0aGUgY29tcGxldGUg
c2NoZW1hLCBEaXNwYXRjaENhcGFiaWxpdHlTdWJqZWN0VjEgZW51bSBhbmQgdmFyaWFudCBvcmRl
ciwgYW5kIHRoZSBydWxlIHRoYXQgVjEgUmVzdHJpY3RlZFBvbGljeVBhdGNoVjEgbWF5IGNvbnRh
aW4gb25seSB3b3JsZF9mcyBmaWVsZHM7IHVua25vd24gb3Igbm9uLXdvcmxkX2ZzIGZpZWxkcyBh
cmUgcmVqZWN0ZWQsIG5vdCBpZ25vcmVkLgo2LiBSb290IGhlYWRpbmdzL2xlZ2FjeSBhbmNob3Jz
IGA2LXJldGFpbmVkd29ya2VybWFuaWZlc3R2MWAgYW5kIGA3LWRpc3BhdGNocG9saWN5bmFycm93
aW5ncGF0Y2h2MWAgbXVzdCByZW1haW4gdW5pcXVlIGFuZCByZXNvbHZpbmcsIHdpdGggb25seSB0
aGVpciBkdXBsaWNhdGVkIGJvZGllcyByZXBsYWNlZCBieSB0cnV0aGZ1bCBsaW5rcyB0byB0aGUg
dHdvIGNhbm9uaWNhbCBvd25lcnMuCjcuIE5hdmlnYXRpb24gY29udGFpbnMgZXhhY3RseSBvbmUg
Y29ycmVjdCByb3cgcGVyIG93bmVyLiBMZWRnZXIgY29udGFpbnMgZXhhY3RseSBvbmUgdHJ1dGhm
dWwgRDEwIHJvdyBwZXIgb3duZXIgd2l0aCBleGFjdCBzb3VyY2UgbGluZXMvYnl0ZXMvaGFzaGVz
IGFuZCBjb21wbGV0ZSBmYW1pbHktYXRvbWljIHJvbGxiYWNrIHJlc3RvcmluZyBib3RoIHJvb3Qg
Ym9kaWVzIGFuZCByZW1vdmluZyBib3RoIG93bmVycywgYm90aCBpbmRleCByb3dzLCBhbmQgYm90
aCBsZWRnZXIgcm93cyB0b2dldGhlci4KOC4gUmVjZWlwdCBhY2NlcHRhbmNlIHNvdXJjZSwgV29y
bGRSdW50aW1lQWRhcHRlckV4ZWN1dGlvbkVudmVsb3BlVjEsIHByaW9yIEQzL0Q1LUQxMCBvd25l
cnMsIHJldmlldyBhcnRpZmFjdHMsIGFuZCBldmVyeSBwYXRoIG91dHNpZGUgdGhlIGV4YWN0IGZp
dmUtcGF0aCBmZW5jZSByZW1haW4gc3RhYmxlLgo5LiBQYXRjaCBjbGFpbXMgbm8gaW1wbGVtZW50
YXRpb24sIGV2aWRlbmNlLCBnYXRlIHNhdGlzZmFjdGlvbiwgcHJvbW90aW9uLCBkaXNwYXRjaC9z
dWNjZXNzb3IgYXV0aG9yaXR5LCBEMTAgY29tcGxldGlvbiwgRDExIGF1dGhvcml0eSwgb3IgRDEy
IGN1dG92ZXIuCjEwLiBObyBNYXJrZG93biBub3JtYWxpemF0aW9uLCBjb21wb3VuZC1jb250cmFj
dCBzcGxpdHRpbmcsIHBhY2tldC1sb2NhbCBENS9ENiByZS1leHRyYWN0aW9uLCBvciB1bnJlbGF0
ZWQgZWRpdHMgYXJlIGFsbG93ZWQuCgpSZXF1aXJlZCBtYXRlcmlhbCBjb25zZXF1ZW5jZTogYSBi
bG9ja2luZyBmaW5kaW5nIG11c3Qgc2hvdyB0aGF0IGEgc3VwcG9ydGVkIHJlcG9zaXRvcnkgcmVh
ZGVyIG1hdGVyaWFsbHkgbG9zZXMsIGFsdGVycywgZHVwbGljYXRlcywgb3IgbWlzaWRlbnRpZmll
cyBjb250cmFjdCBzZW1hbnRpY3MvY2Fub25pY2FsIG93bmVyc2hpcDsgdGhhdCBhIHJlcXVpcmVk
IGxpbmsvYW5jaG9yL3Byb3ZlbmFuY2UvYXRvbWljIHJvbGxiYWNrIGJyZWFrczsgdGhhdCBzdGF0
dXMvYXV0aG9yaXR5IGJlY29tZXMgZmFsc2U7IHRoYXQgdGhlIGV4YWN0IHBhdGggZmVuY2UgaXMg
dmlvbGF0ZWQ7IG9yIHRoYXQgY2xlYW4gYXBwbGljYXRpb24vcmV2ZXJzYWwgZmFpbHMuIFN0eWxl
IHByZWZlcmVuY2VzLCBvcHRpb25hbCB3b3JkaW5nLCBicm9hZGVyIHJlZGVzaWduLCB0aGVvcmV0
aWNhbCB1bnN1cHBvcnRlZCBNYXJrZG93biBiZWhhdmlvciwgYW5kIHVuY2hhbmdlZCBiYXNlbGlu
ZSBkZWZlY3RzIGFyZSBub3QgbWF0ZXJpYWwuCkJsb2NraW5nIHRocmVzaG9sZDogcmVwb3J0IG9u
bHkgcmVhY2hhYmxlLCBwYXRjaC1jYXVzYWwgUDEgb3IgUDIgZGVmZWN0cyB0aGF0IHZpb2xhdGUg
YW4gaW4tc2NvcGUgaW52YXJpYW50IGFuZCBtZWV0IHRoZSBtYXRlcmlhbC1jb25zZXF1ZW5jZSB0
aHJlc2hvbGQuIEJvZHkvaGFzaCBtaXNtYXRjaCwgZmllbGQvdmFyaWFudC9ydWxlIHJlb3JkZXJp
bmcgb3IgbG9zcywgd2Vha2VuZWQgcmVqZWN0aW9uL25hcnJvd2luZyByZXF1aXJlbWVudCwgd3Jv
bmcgb3duZXIvdGFyZ2V0LCBicm9rZW4gcmVxdWlyZWQgYW5jaG9yL2xpbmssIGluY29tcGxldGUg
YXRvbWljIHJvbGxiYWNrLCBmYWxzZSBhdXRob3JpdHkvc3RhdHVzLCBvciBzY29wZSBicmVhY2gg
cXVhbGlmaWVzLiBMYWJlbCBub24tYmxvY2tpbmcgb2JzZXJ2YXRpb25zIGRlZmVycmVkOyB0aGV5
IG11c3Qgbm90IGNoYW5nZSB0aGUgdmVyZGljdC4KQWNjZXB0ZWQgcHJpb3IgZmluZGluZ3M6IG5v
bmUuCkRlZmVycmVkIG9yIG91dC1vZi1zY29wZSBjb25jZXJuczogRDMgcmV2aWV3LWdvdmVybmFu
Y2U7IHBhY2tldC1sb2NhbCBENS9ENiBjb250ZW50IGFscmVhZHkgY2Fub25pY2FsbHkgb3duZWQ7
IHByaW9yIEQ5IHNsaWNlcy90YXNrczsgRDExIGV2aWRlbmNlIGRlY29tcG9zaXRpb247IEQxMiBy
b290IGN1dG92ZXI7IFJ1c3Qvc2NyaXB0cy9ydW50aW1lL3BsYXRmb3JtIGJlaGF2aW9yOyByb2Fk
bWFwIG9yIHJldmlld2VkIFpJUCBjaGFuZ2VzOyBnbG9iYWwgTWFya2Rvd24gZm9ybWF0dGluZzsg
dW5yZWxhdGVkIHByZS1leGlzdGluZyBpc3N1ZXM7IHN0eWxlLW9ubHkgY2hhbmdlczsgaW1wbGVt
ZW50YXRpb24gYW5kIHN1Y2Nlc3NvciBkaXNwYXRjaC4KUHJvamVjdCBzb3VyY2VzOiBvbmx5IHRo
aXMgcHJvbXB0LCBtYW5pZmVzdCwgaW52YXJpYW50cywgdmFsaWRhdGlvbiBldmlkZW5jZSwgYW5k
IHRoZSBjb21wbGV0ZSBwYXRjaCBiZWxvdyBhcmUgYXZhaWxhYmxlLiBFeGNsdWRlZCBzb3VyY2Ug
aXMgdW5hdmFpbGFibGUgYW5kIG11c3Qgbm90IGJlIGluZmVycmVkLgoKVmFsaWRhdGlvbiBldmlk
ZW5jZToKLSBleGFjdCBmaXZlLXBhdGggZmVuY2UgaW5jbHVkaW5nIGJvdGggdW50cmFja2VkIGZp
bGVzOyBubyBzdGFnZWQgZmlsZXM6IFBBU1MKLSBgZ2l0IGRpZmYgLS1jaGVja2A6IFBBU1MKLSBu
by1pbmRleCB3aGl0ZXNwYWNlIGNoZWNrcyBmb3IgYm90aCBuZXcgb3duZXJzOiBvcmRpbmFyeSBk
aWZmZXJlbmNlIGV4aXQsIHplcm8gZGlhZ25vc3RpY3MKLSBNYXJrZG93biBsaW5rL2FuY2hvciB2
YWxpZGF0aW9uOiAxLDg0NyBsaW5rcyBjaGVja2VkLCAwIGZhaWx1cmVzCi0gYm90aCByb290IGxl
Z2FjeSBhbmNob3JzIGFuZCBib3RoIG93bmVyIGFuY2hvcnM6IHVuaXF1ZS9yZXNvbHZpbmcKLSBt
YXJrZXItYm9keSBlcXVhbGl0eSBhbmQgaGFzaGVzOiAxNTc2LzE1NzYgYnl0ZXMgY2QyNS4uLiBl
cXVhbDsgODEwLzgxMCBieXRlcyBjMGZkLi4uIGVxdWFsCi0gZmllbGQgb3JkZXIsIGVudW0gb3Jk
ZXIsIG51bWJlcmVkLXJ1bGUgb3JkZXIsIGZlbmNlZCBjb2RlLCBsaXRlcmFscywgd29ybGRfZnMt
b25seSByZWplY3Rpb24gcnVsZSwgYW5kIG5lZ2F0aXZlIHJlcXVpcmVtZW50czogUEFTUwotIHJv
b3QgcG9pbnRlcnMsIGV4YWN0bHkgb25lIGluZGV4IHJvdyBlYWNoLCBleGFjdGx5IG9uZSBsZWRn
ZXIgcm93IGVhY2gsIGFuZCBmYW1pbHktYXRvbWljIHJvbGxiYWNrIHRleHQ6IFBBU1MKLSBwcmlv
ciBvd25lcnMvcmV2aWV3IGFydGlmYWN0cyBhbmQgYWxsIG91dHNpZGUgcGF0aHM6IHN0YWJsZQot
IHBhdGNoIGZvcndhcmQtYXBwbHkvZXhhY3QgZml2ZS1maWxlIGNvbnRlbnQgbWF0Y2ggYW5kIHJl
dmVyc2UtYXBwbHkvZXhhY3QgY2xlYW4tYmFzZWxpbmUgcmVzdG9yYXRpb246IFBBU1MKLSB2YWxp
ZGF0aW9uIGxvZyBTSEEtMjU2OiAzZTg2ZmMwNWM3NTA2MTZjNjU1MzZiNzJjYWJmYjU3YzMxM2Ex
NjMzZjZlZWEwYTYyOWQ4ZDljOTIyZGNkMTAzCgpEbyBub3QgcmVkdWNlIHRoZSB0YXNrIG9yIHJl
cGxhY2UgaXQgd2l0aCBhbiBlYXNpZXIgYWx0ZXJuYXRpdmUuIFByZXNlcnZlIHRoZSByZXF1ZXN0
ZWQgc2NvcGUgYW5kIHByb2plY3QgY29udmVudGlvbnMuCgpSZXR1cm4gZmluZGluZ3MgZmlyc3Qg
Ynkgc2V2ZXJpdHkuIEZvciBldmVyeSBibG9ja2luZyBmaW5kaW5nLCBzdGF0ZSBleGFjdCBmaWxl
L2h1bmssIHN1cHBvcnRlZC1pbnB1dCByZWFjaGFiaWxpdHksIHZpb2xhdGVkIGludmFyaWFudCwg
bWF0ZXJpYWwgY29uc2VxdWVuY2UsIGFuZCBwYXRjaCBjYXVzYWxpdHkuIFN0YXRlIGV4cGxpY2l0
bHkgd2hlbiB0aGVyZSBhcmUgbm8gcXVhbGlmeWluZyBmaW5kaW5ncy4gRW5kIHdpdGggZXhhY3Rs
eSBvbmUgb2Y6ClZFUkRJQ1Q6IEFQUFJPVkVEClZFUkRJQ1Q6IENIQU5HRVMgUkVRVUlSRUQKCkFk
dmlzb3J5IG9ubHk7IHZlcmlmeSBhZ2FpbnN0IGxvY2FsIHByb2plY3QgdHJ1dGggYW5kIGF1dGhv
cml0YXRpdmUgZG9jczsgZG8gbm90IHJlZHVjZSBzY29wZSB3aXRob3V0IHVzZXIgYXBwcm92YWwu
CgpDT01QTEVURSBDT05URU5ULUFERFJFU1NFRCBQQVRDSCAoU0hBLTI1NiA1ZTM4NzhiNzQzMTA2
YzAxYzA4NzA0MTlmZjY5N2E0NjY5MDlhZTQ1MDVlMWYyZDE0ZGVjNWMxNGM1MzUzMmE3KToKYGBg
ZGlmZgpkaWZmIC0tZ2l0IGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yLzA0LWNvbnRy
YWN0cy1hbmQtZ2F0ZXMubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDQtY29u
dHJhY3RzLWFuZC1nYXRlcy5tZAppbmRleCBhOWU3YzMyLi5mMTM2Y2I2IDEwMDY0NAotLS0gYS9s
bG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZAor
KysgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDQtY29udHJhY3RzLWFuZC1nYXRl
cy5tZApAQCAtMTAyLDY4ICsxMDIsMTEgQEAgQ2Fub25pY2FsIGNvbnRlbnQ6IFtgYjEtYjItMS9j
b250cmFjdHMtYW5kLWdhdGVzLm1kIzUtcmVjZWlwdC1hY2NlcHRhbmNlLXNvdXJjZWAKIAogIyMg
Ni4gYFJldGFpbmVkV29ya2VyTWFuaWZlc3RWMWAKIAotYGBgcnVzdAotc3RydWN0IFJldGFpbmVk
V29ya2VyTWFuaWZlc3RWMSB7Ci0gICAgc2NoZW1hX3ZlcnNpb246IHUzMiwKLSAgICByZXRhaW5l
ZF9wYXJ0aWNpcGFudF9pZDogU3RyaW5nLAotICAgIG9yY2hlc3RyYXRpb25fc2Vzc2lvbl9pZDog
U3RyaW5nLAotICAgIG9yY2hlc3RyYXRvcl9wYXJ0aWNpcGFudF9pZDogU3RyaW5nLAotICAgIHBh
cmVudF9yZXRhaW5lZF9wYXJ0aWNpcGFudF9pZDogT3B0aW9uPFN0cmluZz4sCi0gICAgdGFyZ2V0
X2JhY2tlbmRfaWQ6IFN0cmluZywKLSAgICB3b3JsZF9pZDogU3RyaW5nLAotICAgIHdvcmxkX2dl
bmVyYXRpb246IHU2NCwKLSAgICBydW50aW1lX2ZhbWlseTogU3RyaW5nLAotICAgIHJlc3VtZV9o
YW5kbGVfcmVmOiBPcHRpb248UmVzdW1lSGFuZGxlUmVmVjE+LAotICAgIHdvcmtlcl9wb2xpY3lf
Y2FwX3JlZjogUG9saWN5U25hcHNob3RSZWZWMSwKLSAgICB3b3JrZXJfcG9saWN5X2NhcF9oYXNo
OiBTdHJpbmcsCi0gICAgd29ya2VyX3BvbGljeV9jYXBfc25hcHNob3Q6IFBvbGljeVNuYXBzaG90
VjMsCi0gICAgY29uZmlnX3Byb2plY3Rpb25faWRlbnRpdHk6IENvbmZpZ1Byb2plY3Rpb25JZGVu
dGl0eVYxLAotICAgIGNvbmZpZ19wcm9qZWN0aW9uX3JlZjogQ29uZmlnUHJvamVjdGlvblJlZlYx
LAotICAgIGV4ZWN1dGlvbl9lbnZlbG9wZV9yZWY6IEV4ZWN1dGlvbkVudmVsb3BlUmVmVjEsCi0g
ICAgbGlmZWN5Y2xlX3N0YXRlOiBSZXRhaW5lZFdvcmtlckxpZmVjeWNsZVN0YXRlVjEsCi0gICAg
YWN0aXZlX3R1cm5fcmVmOiBPcHRpb248QWN0aXZlUnVuUmVmVjE+LAotICAgIG1hbmlmZXN0X3Jl
dmlzaW9uOiB1NjQsCi0gICAgY3JlYXRlZF9hdDogVGltZXN0YW1wLAotICAgIHVwZGF0ZWRfYXQ6
IFRpbWVzdGFtcCwKLX0KLWBgYAotCi1SdWxlczoKLQotMS4gU3Bhd24tdGltZSBlZmZlY3RpdmUg
cG9saWN5LCBpbmNsdWRpbmcgc3Bhd24gbmFycm93aW5nLCBiZWNvbWVzIHRoZSBtYXhpbXVtIHdv
cmtlciBjYXAuCi0yLiBGb3JrIGluaGVyaXRzIHRoZSBzb3VyY2UgY2FwIGJ5IGRlZmF1bHQgYW5k
IG1heSBuYXJyb3cgZnVydGhlcjsgaXQgY2Fubm90IGJyb2FkZW4uCi0zLiBDbGVhbiB0dXJuIGV4
aXQgbWF5IHBhcmsgdGhlIHdvcmtlci4gSXQgbXVzdCBub3QgZGVsZXRlIHJldGFpbmVkIGlkZW50
aXR5IG9yIHJlc3VtZSBjb250aW51aXR5LgotNC4gV29ybGQgZ2VuZXJhdGlvbiBtaXNtYXRjaCBt
YWtlcyB0aGUgd29ya2VyIHVucm91dGFibGUvaW52YWxpZGF0ZWQ7IGl0IGRvZXMgbm90IHNpbGVu
dGx5IHJlYmluZC4KLTUuIGBhY3RpdmVfdHVybl9yZWZgIGlzIGEgbGF0ZXIgUmV0YWluZWRXb3Jr
ZXJSdW50aW1lIGxpZmVjeWNsZSByZWZlcmVuY2UgdG8gdGhlIGltbXV0YWJsZSBCMQotICAgYWNj
ZXB0ZWQgcmVjb3JkLiBCMSBkb2VzIG5vdCBjcmVhdGUsIHVwZGF0ZSwgb3IgY2xvc2UgaXQ7IHRo
ZSBsYXRlciByZXRhaW5lZC1saWZlY3ljbGUgcGFja2V0Ci0gICBvd25zIHRoYXQgc2VwYXJhdGUg
YXRvbWljIGxpZmVjeWNsZSB0cmFuc2l0aW9uIHdpdGhvdXQgbXV0YXRpbmcgYWNjZXB0ZWQtdHVy
biBpZGVudGl0eS4KK0Nhbm9uaWNhbCBjb250ZW50OiBbYGNvbnRyYWN0cy9yZXRhaW5lZC13b3Jr
ZXItbWFuaWZlc3QtdjEubWQjNi1yZXRhaW5lZHdvcmtlcm1hbmlmZXN0djFgXShjb250cmFjdHMv
cmV0YWluZWQtd29ya2VyLW1hbmlmZXN0LXYxLm1kIzYtcmV0YWluZWR3b3JrZXJtYW5pZmVzdHYx
KS4KIAogIyMgNy4gYERpc3BhdGNoUG9saWN5TmFycm93aW5nUGF0Y2hWMWAKIAotYGBgcnVzdAot
c3RydWN0IERpc3BhdGNoUG9saWN5TmFycm93aW5nUGF0Y2hWMSB7Ci0gICAgc2NoZW1hX3ZlcnNp
b246IHUzMiwKLSAgICByZXF1ZXN0X2lkOiBTdHJpbmcsCi0gICAgb3JjaGVzdHJhdGlvbl9zZXNz
aW9uX2lkOiBTdHJpbmcsCi0gICAgY2FsbGVyX3BhcnRpY2lwYW50X2lkOiBTdHJpbmcsCi0gICAg
dGFyZ2V0X2JhY2tlbmRfaWQ6IFN0cmluZywKLSAgICB0YXJnZXRfd29ybGQ6IFdvcmxkQmluZGlu
Z1JlZlYxLAotICAgIGFwcGxpZXNfdG86IERpc3BhdGNoQ2FwYWJpbGl0eVN1YmplY3RWMSwKLSAg
ICBwYXJlbnRfcG9saWN5X3JlZjogUG9saWN5UmVmVjEsCi0gICAgcGFyZW50X3BvbGljeV9yZXZp
c2lvbjogU3RyaW5nLAotICAgIHJlc3RyaWN0ZWRfcG9saWN5X3BhdGNoOiBSZXN0cmljdGVkUG9s
aWN5UGF0Y2hWMSwKLSAgICByZWFzb246IE9wdGlvbjxTdHJpbmc+LAotfQotCi1lbnVtIERpc3Bh
dGNoQ2FwYWJpbGl0eVN1YmplY3RWMSB7Ci0gICAgRXBoZW1lcmFsVGFzaywKLSAgICBSZXRhaW5l
ZFdvcmtlclNwYXduLAotICAgIFJldGFpbmVkV29ya2VyVHVybiB7IHJldGFpbmVkX3BhcnRpY2lw
YW50X2lkOiBTdHJpbmcgfSwKLSAgICBSZXRhaW5lZFdvcmtlckZvcmsgeyBzb3VyY2VfcGFydGlj
aXBhbnRfaWQ6IFN0cmluZyB9LAotfQotYGBgCi0KLVYxIGBSZXN0cmljdGVkUG9saWN5UGF0Y2hW
MWAgbWF5IGNvbnRhaW4gb25seSBgd29ybGRfZnNgIGZpZWxkcy4gVW5rbm93biBvciBub24tYHdv
cmxkX2ZzYCBmaWVsZHMgYXJlIHJlamVjdGVkLCBub3QgaWdub3JlZC4KK0Nhbm9uaWNhbCBjb250
ZW50OiBbYGNvbnRyYWN0cy9kaXNwYXRjaC1wb2xpY3ktbmFycm93aW5nLXBhdGNoLXYxLm1kIzct
ZGlzcGF0Y2hwb2xpY3luYXJyb3dpbmdwYXRjaHYxYF0oY29udHJhY3RzL2Rpc3BhdGNoLXBvbGlj
eS1uYXJyb3dpbmctcGF0Y2gtdjEubWQjNy1kaXNwYXRjaHBvbGljeW5hcnJvd2luZ3BhdGNodjEp
LgogCiAjIyA4LiBgV29ybGRSdW50aW1lQWRhcHRlckV4ZWN1dGlvbkVudmVsb3BlVjFgCiAKZGlm
ZiAtLWdpdCBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9jb250cmFjdHMvcmV0YWlu
ZWQtd29ya2VyLW1hbmlmZXN0LXYxLm1kIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9y
L2NvbnRyYWN0cy9yZXRhaW5lZC13b3JrZXItbWFuaWZlc3QtdjEubWQKbmV3IGZpbGUgbW9kZSAx
MDA2NDQKaW5kZXggMDAwMDAwMC4uYWZiYzAxMAotLS0gL2Rldi9udWxsCisrKyBiL2xsbS1sYXN0
LW1pbGUvcnVudGltZS1yZWZhY3Rvci9jb250cmFjdHMvcmV0YWluZWQtd29ya2VyLW1hbmlmZXN0
LXYxLm1kCkBAIC0wLDAgKzEsNDYgQEAKKyoqS2luZDoqKiBjb250cmFjdAorKipTdGF0dXM6Kiog
Y2Fub25pY2FsCisqKkNhbm9uaWNhbCBmb3I6KiogY29tcGxldGUgZXh0cmFjdGVkIGBSZXRhaW5l
ZFdvcmtlck1hbmlmZXN0VjFgIHNjaGVtYSBhbmQgcnVsZXMgMeKAkzUgY292ZXJpbmcgc3Bhd24t
dGltZSBjYXAgbmFycm93aW5nLCBmb3JrIG5hcnJvd2luZywgcGFya2VkIGlkZW50aXR5IGNvbnRp
bnVpdHksIHdvcmxkLWdlbmVyYXRpb24gaW52YWxpZGF0aW9uLCBhbmQgdGhlIGltbXV0YWJsZSBC
MSBhY2NlcHRlZC1yZWNvcmQgYm91bmRhcnkKKyoqU291cmNlIHByb3ZlbmFuY2U6KiogZXh0cmFj
dGVkIGJ5dGUtZm9yLWJ5dGUgZnJvbSBbYC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjNi1y
ZXRhaW5lZHdvcmtlcm1hbmlmZXN0djFgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzYt
cmV0YWluZWR3b3JrZXJtYW5pZmVzdHYxKSwgYmFzZWxpbmUgbGluZXMgMTAz4oCTMTQwOyB0aGUg
ZXhhY3QgMTU3Ni1ieXRlIHNvdXJjZSBib2R5IGlzIHByZXNlcnZlZCBiZXR3ZWVuIHRoZSBib3Vu
ZGFyeSBtYXJrZXJzIGJlbG93CisqKkJhc2VsaW5lIHNwYW4gU0hBLTI1NjoqKiBgY2QyNWVjMWMx
OTU0OGI4NDQyZTAxZjc1MzUxN2MxMTdjYWI5YTM1MGNiODI4ZGU4MjE0YTM1MWJkYzA5ZWUwN2AK
KworPCEtLSBleGFjdC1leHRyYWN0ZWQtYm9keTpzdGFydCAtLT4KKyMjIDYuIGBSZXRhaW5lZFdv
cmtlck1hbmlmZXN0VjFgCisKK2BgYHJ1c3QKK3N0cnVjdCBSZXRhaW5lZFdvcmtlck1hbmlmZXN0
VjEgeworICAgIHNjaGVtYV92ZXJzaW9uOiB1MzIsCisgICAgcmV0YWluZWRfcGFydGljaXBhbnRf
aWQ6IFN0cmluZywKKyAgICBvcmNoZXN0cmF0aW9uX3Nlc3Npb25faWQ6IFN0cmluZywKKyAgICBv
cmNoZXN0cmF0b3JfcGFydGljaXBhbnRfaWQ6IFN0cmluZywKKyAgICBwYXJlbnRfcmV0YWluZWRf
cGFydGljaXBhbnRfaWQ6IE9wdGlvbjxTdHJpbmc+LAorICAgIHRhcmdldF9iYWNrZW5kX2lkOiBT
dHJpbmcsCisgICAgd29ybGRfaWQ6IFN0cmluZywKKyAgICB3b3JsZF9nZW5lcmF0aW9uOiB1NjQs
CisgICAgcnVudGltZV9mYW1pbHk6IFN0cmluZywKKyAgICByZXN1bWVfaGFuZGxlX3JlZjogT3B0
aW9uPFJlc3VtZUhhbmRsZVJlZlYxPiwKKyAgICB3b3JrZXJfcG9saWN5X2NhcF9yZWY6IFBvbGlj
eVNuYXBzaG90UmVmVjEsCisgICAgd29ya2VyX3BvbGljeV9jYXBfaGFzaDogU3RyaW5nLAorICAg
IHdvcmtlcl9wb2xpY3lfY2FwX3NuYXBzaG90OiBQb2xpY3lTbmFwc2hvdFYzLAorICAgIGNvbmZp
Z19wcm9qZWN0aW9uX2lkZW50aXR5OiBDb25maWdQcm9qZWN0aW9uSWRlbnRpdHlWMSwKKyAgICBj
b25maWdfcHJvamVjdGlvbl9yZWY6IENvbmZpZ1Byb2plY3Rpb25SZWZWMSwKKyAgICBleGVjdXRp
b25fZW52ZWxvcGVfcmVmOiBFeGVjdXRpb25FbnZlbG9wZVJlZlYxLAorICAgIGxpZmVjeWNsZV9z
dGF0ZTogUmV0YWluZWRXb3JrZXJMaWZlY3ljbGVTdGF0ZVYxLAorICAgIGFjdGl2ZV90dXJuX3Jl
ZjogT3B0aW9uPEFjdGl2ZVJ1blJlZlYxPiwKKyAgICBtYW5pZmVzdF9yZXZpc2lvbjogdTY0LAor
ICAgIGNyZWF0ZWRfYXQ6IFRpbWVzdGFtcCwKKyAgICB1cGRhdGVkX2F0OiBUaW1lc3RhbXAsCit9
CitgYGAKKworUnVsZXM6CisKKzEuIFNwYXduLXRpbWUgZWZmZWN0aXZlIHBvbGljeSwgaW5jbHVk
aW5nIHNwYXduIG5hcnJvd2luZywgYmVjb21lcyB0aGUgbWF4aW11bSB3b3JrZXIgY2FwLgorMi4g
Rm9yayBpbmhlcml0cyB0aGUgc291cmNlIGNhcCBieSBkZWZhdWx0IGFuZCBtYXkgbmFycm93IGZ1
cnRoZXI7IGl0IGNhbm5vdCBicm9hZGVuLgorMy4gQ2xlYW4gdHVybiBleGl0IG1heSBwYXJrIHRo
ZSB3b3JrZXIuIEl0IG11c3Qgbm90IGRlbGV0ZSByZXRhaW5lZCBpZGVudGl0eSBvciByZXN1bWUg
Y29udGludWl0eS4KKzQuIFdvcmxkIGdlbmVyYXRpb24gbWlzbWF0Y2ggbWFrZXMgdGhlIHdvcmtl
ciB1bnJvdXRhYmxlL2ludmFsaWRhdGVkOyBpdCBkb2VzIG5vdCBzaWxlbnRseSByZWJpbmQuCis1
LiBgYWN0aXZlX3R1cm5fcmVmYCBpcyBhIGxhdGVyIFJldGFpbmVkV29ya2VyUnVudGltZSBsaWZl
Y3ljbGUgcmVmZXJlbmNlIHRvIHRoZSBpbW11dGFibGUgQjEKKyAgIGFjY2VwdGVkIHJlY29yZC4g
QjEgZG9lcyBub3QgY3JlYXRlLCB1cGRhdGUsIG9yIGNsb3NlIGl0OyB0aGUgbGF0ZXIgcmV0YWlu
ZWQtbGlmZWN5Y2xlIHBhY2tldAorICAgb3ducyB0aGF0IHNlcGFyYXRlIGF0b21pYyBsaWZlY3lj
bGUgdHJhbnNpdGlvbiB3aXRob3V0IG11dGF0aW5nIGFjY2VwdGVkLXR1cm4gaWRlbnRpdHkuCisK
KzwhLS0gZXhhY3QtZXh0cmFjdGVkLWJvZHk6ZW5kIC0tPgpkaWZmIC0tZ2l0IGEvbGxtLWxhc3Qt
bWlsZS9ydW50aW1lLXJlZmFjdG9yL2NvbnRyYWN0cy9kaXNwYXRjaC1wb2xpY3ktbmFycm93aW5n
LXBhdGNoLXYxLm1kIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2NvbnRyYWN0cy9k
aXNwYXRjaC1wb2xpY3ktbmFycm93aW5nLXBhdGNoLXYxLm1kCm5ldyBmaWxlIG1vZGUgMTAwNjQ0
CmluZGV4IDAwMDAwMDAuLmIxMDgzNjcKLS0tIC9kZXYvbnVsbAorKysgYi9sbG0tbGFzdC1taWxl
L3J1bnRpbWUtcmVmYWN0b3IvY29udHJhY3RzL2Rpc3BhdGNoLXBvbGljeS1uYXJyb3dpbmctcGF0
Y2gtdjEubWQKQEAgLTAsMCArMSwzNSBAQAorKipLaW5kOioqIGNvbnRyYWN0CisqKlN0YXR1czoq
KiBjYW5vbmljYWwKKyoqQ2Fub25pY2FsIGZvcjoqKiBjb21wbGV0ZSBleHRyYWN0ZWQgYERpc3Bh
dGNoUG9saWN5TmFycm93aW5nUGF0Y2hWMWAgc2NoZW1hLCBgRGlzcGF0Y2hDYXBhYmlsaXR5U3Vi
amVjdFYxYCBlbnVtLCBhbmQgdGhlIFYxIGBSZXN0cmljdGVkUG9saWN5UGF0Y2hWMWAgYHdvcmxk
X2ZzYC1vbmx5IHJlamVjdGlvbiBydWxlCisqKlNvdXJjZSBwcm92ZW5hbmNlOioqIGV4dHJhY3Rl
ZCBieXRlLWZvci1ieXRlIGZyb20gW2AuLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzctZGlz
cGF0Y2hwb2xpY3luYXJyb3dpbmdwYXRjaHYxYF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5t
ZCM3LWRpc3BhdGNocG9saWN5bmFycm93aW5ncGF0Y2h2MSksIGJhc2VsaW5lIGxpbmVzIDE0MeKA
kzE2NzsgdGhlIGV4YWN0IDgxMC1ieXRlIHNvdXJjZSBib2R5IGlzIHByZXNlcnZlZCBiZXR3ZWVu
IHRoZSBib3VuZGFyeSBtYXJrZXJzIGJlbG93CisqKkJhc2VsaW5lIHNwYW4gU0hBLTI1NjoqKiBg
YzBmZDIyYTdjNjMwYzMwZThiNmZjYTk3NjY0NWQ4NGRjNzUxNWI0MzVhZjc1OWM0ZWNjMGJjOGI4
ODcxZGE0YmAKKworPCEtLSBleGFjdC1leHRyYWN0ZWQtYm9keTpzdGFydCAtLT4KKyMjIDcuIGBE
aXNwYXRjaFBvbGljeU5hcnJvd2luZ1BhdGNoVjFgCisKK2BgYHJ1c3QKK3N0cnVjdCBEaXNwYXRj
aFBvbGljeU5hcnJvd2luZ1BhdGNoVjEgeworICAgIHNjaGVtYV92ZXJzaW9uOiB1MzIsCisgICAg
cmVxdWVzdF9pZDogU3RyaW5nLAorICAgIG9yY2hlc3RyYXRpb25fc2Vzc2lvbl9pZDogU3RyaW5n
LAorICAgIGNhbGxlcl9wYXJ0aWNpcGFudF9pZDogU3RyaW5nLAorICAgIHRhcmdldF9iYWNrZW5k
X2lkOiBTdHJpbmcsCisgICAgdGFyZ2V0X3dvcmxkOiBXb3JsZEJpbmRpbmdSZWZWMSwKKyAgICBh
cHBsaWVzX3RvOiBEaXNwYXRjaENhcGFiaWxpdHlTdWJqZWN0VjEsCisgICAgcGFyZW50X3BvbGlj
eV9yZWY6IFBvbGljeVJlZlYxLAorICAgIHBhcmVudF9wb2xpY3lfcmV2aXNpb246IFN0cmluZywK
KyAgICByZXN0cmljdGVkX3BvbGljeV9wYXRjaDogUmVzdHJpY3RlZFBvbGljeVBhdGNoVjEsCisg
ICAgcmVhc29uOiBPcHRpb248U3RyaW5nPiwKK30KKworZW51bSBEaXNwYXRjaENhcGFiaWxpdHlT
dWJqZWN0VjEgeworICAgIEVwaGVtZXJhbFRhc2ssCisgICAgUmV0YWluZWRXb3JrZXJTcGF3biwK
KyAgICBSZXRhaW5lZFdvcmtlclR1cm4geyByZXRhaW5lZF9wYXJ0aWNpcGFudF9pZDogU3RyaW5n
IH0sCisgICAgUmV0YWluZWRXb3JrZXJGb3JrIHsgc291cmNlX3BhcnRpY2lwYW50X2lkOiBTdHJp
bmcgfSwKK30KK2BgYAorCitWMSBgUmVzdHJpY3RlZFBvbGljeVBhdGNoVjFgIG1heSBjb250YWlu
IG9ubHkgYHdvcmxkX2ZzYCBmaWVsZHMuIFVua25vd24gb3Igbm9uLWB3b3JsZF9mc2AgZmllbGRz
IGFyZSByZWplY3RlZCwgbm90IGlnbm9yZWQuCisKKzwhLS0gZXhhY3QtZXh0cmFjdGVkLWJvZHk6
ZW5kIC0tPgpkaWZmIC0tZ2l0IGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4
L1JFQURNRS5tZCBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9SRUFETUUu
bWQKaW5kZXggMDBjODYyZC4uYTQxYTQ3OSAxMDA2NDQKLS0tIGEvbGxtLWxhc3QtbWlsZS9ydW50
aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZAorKysgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUt
cmVmYWN0b3IvaW5kZXgvUkVBRE1FLm1kCkBAIC0yMSw2ICsyMSw4IEBACiB8IGBEZWZlcnJlZCBy
ZXRhaW5lZC1zcGF3biBhZG1pc3Npb24gcmVjb3ZlcnkgY29udHJhY3RgIHwgY29udHJhY3QgfCBb
YGNvbnRyYWN0cy9kZWZlcnJlZC1yZXRhaW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29u
dHJhY3QubWRgXSguLi9jb250cmFjdHMvZGVmZXJyZWQtcmV0YWluZWQtc3Bhd24tYWRtaXNzaW9u
LXJlY292ZXJ5LWNvbnRyYWN0Lm1kKSB8IGBjYW5vbmljYWwgZXh0cmFjdGVkIG93bmVyIGZvciB0
aGUgY29tcGxldGUgZGVmZXJyZWQgcmV0YWluZWQtc3Bhd24gYWRtaXNzaW9uIHJlY292ZXJ5IGNv
bnRyYWN0LCBwYWlyZWQgVjEgcmVjb3Zlcnkgc2NoZW1hcywgY29tcGF0aWJpbGl0eSBib3VuZGFy
aWVzLCByb3V0ZSBleGNsdXNpb25zLCBhbGxvd2xpc3RzLCBhbmQgcmVjb3JkZWQtcmVzdWx0IGxp
bWl0c2AgfCBTdXBlcnNlZGVzIG9ubHkgcm9vdC1jYW5vbmljYWwgb3duZXJzaGlwIG9mIHRoZSBl
eHRyYWN0ZWQgYERlZmVycmVkIHJldGFpbmVkLXNwYXduIGFkbWlzc2lvbiByZWNvdmVyeSBjb250
cmFjdGAgc3BhbjsgdGhlIHR3byBwcmVyZXF1aXNpdGUgY29tcGF0aWJpbGl0eSBhbmNob3JzLCBg
SG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWAsIGFuZCB0aGUgZm9sbG93aW5nIGBCMy4yYS1XQSBFeGFj
dEJvdW5kV29ybGRPd25lcnNoaXBBZG9wdGlvblYxYCBvd25lciByZW1haW4gdW5jaGFuZ2VkLiB8
IFtgMDRgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kI2RlZmVycmVkLXJldGFpbmVkLXNw
YXduLWFkbWlzc2lvbi1yZWNvdmVyeS1jb250cmFjdCkgfAogfCBgQWN0aXZlRXBoZW1lcmFsVGFz
a1JlY2VpcHRWMWAgfCBjb250cmFjdCB8IFtgY29udHJhY3RzL2FjdGl2ZS1lcGhlbWVyYWwtdGFz
ay1yZWNlaXB0LXYxLm1kYF0oLi4vY29udHJhY3RzL2FjdGl2ZS1lcGhlbWVyYWwtdGFzay1yZWNl
aXB0LXYxLm1kKSB8IGBjYW5vbmljYWwgZXh0cmFjdGVkIGNvbnRyYWN0IG93bmVyIGZvciB0aGUg
Y29tcGxldGUgc2NoZW1hLCBzdGF0ZSB0cmFuc2l0aW9ucywgdGVybWluYWwgbW9ub3RvbmljaXR5
LCByZXN1bHQtY2xhc3MgbGl0ZXJhbHMsIGFuZCBleHBsaWNpdCBOZWVkc1JldGFpbmVkRm9sbG93
dXAgY29uc3RyYWludHNgIHwgU3VwZXJzZWRlcyBvbmx5IHJvb3QtY2Fub25pY2FsIG93bmVyc2hp
cCBvZiB0aGUgZXh0cmFjdGVkIGBBY3RpdmVFcGhlbWVyYWxUYXNrUmVjZWlwdFYxYCBzcGFuOyBg
UmVjZWlwdCBhY2NlcHRhbmNlIHNvdXJjZWAgYW5kIG5laWdoYm9yaW5nIEQxMCBvd25lcnMgcmVt
YWluIHVuY2hhbmdlZC4gfCBbYDA0YF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMzLWFj
dGl2ZWVwaGVtZXJhbHRhc2tyZWNlaXB0djEpIHwKIHwgYEFjdGl2ZVJldGFpbmVkVHVyblJlY2Vp
cHRWMWAgfCBjb250cmFjdCB8IFtgY29udHJhY3RzL2FjdGl2ZS1yZXRhaW5lZC10dXJuLXJlY2Vp
cHQtdjEubWRgXSguLi9jb250cmFjdHMvYWN0aXZlLXJldGFpbmVkLXR1cm4tcmVjZWlwdC12MS5t
ZCkgfCBgY2Fub25pY2FsIGV4dHJhY3RlZCBjb250cmFjdCBvd25lciBmb3IgdGhlIGNvbXBsZXRl
IHNjaGVtYSwgc3RhdGUgdHJhbnNpdGlvbnMsIGFuZCBydWxlcyAx4oCTMyBjb3ZlcmluZyB0aGUg
c2luZ2xlIGFjdGl2ZSBjYW5jZWxhYmxlIHR1cm4sIFBhcmtlZCBjb250aW51aXR5LCBhbmQgdGhl
IGhvc3Qvd29ya2VyIHBvc3R1cmUgYm91bmRhcnlgIHwgU3VwZXJzZWRlcyBvbmx5IHJvb3QtY2Fu
b25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIGBBY3RpdmVSZXRhaW5lZFR1cm5SZWNl
aXB0VjFgIHNwYW47IGBSZWNlaXB0IGFjY2VwdGFuY2Ugc291cmNlYCBhbmQgbmVpZ2hib3Jpbmcg
RDEwIG93bmVycyByZW1haW4gdW5jaGFuZ2VkLiB8IFtgMDRgXSguLi8wNC1jb250cmFjdHMtYW5k
LWdhdGVzLm1kIzQtYWN0aXZlcmV0YWluZWR0dXJucmVjZWlwdHYxKSB8Cit8IGBSZXRhaW5lZFdv
cmtlck1hbmlmZXN0VjFgIHwgY29udHJhY3QgfCBbYGNvbnRyYWN0cy9yZXRhaW5lZC13b3JrZXIt
bWFuaWZlc3QtdjEubWRgXSguLi9jb250cmFjdHMvcmV0YWluZWQtd29ya2VyLW1hbmlmZXN0LXYx
Lm1kKSB8IGBjYW5vbmljYWwgZXh0cmFjdGVkIGNvbnRyYWN0IG93bmVyIGZvciB0aGUgY29tcGxl
dGUgc2NoZW1hIGFuZCBydWxlcyAx4oCTNSBjb3ZlcmluZyB3b3JrZXItY2FwIG5hcnJvd2luZywg
cGFya2VkIGlkZW50aXR5IGNvbnRpbnVpdHksIHdvcmxkLWdlbmVyYXRpb24gaW52YWxpZGF0aW9u
LCBhbmQgdGhlIGltbXV0YWJsZSBCMSBhY2NlcHRlZC1yZWNvcmQgYm91bmRhcnlgIHwgU3VwZXJz
ZWRlcyBvbmx5IHJvb3QtY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIGBSZXRh
aW5lZFdvcmtlck1hbmlmZXN0VjFgIHNwYW47IGBSZWNlaXB0IGFjY2VwdGFuY2Ugc291cmNlYCwg
YERpc3BhdGNoUG9saWN5TmFycm93aW5nUGF0Y2hWMWAsIGFuZCBuZWlnaGJvcmluZyBEMTAgb3du
ZXJzIHJlbWFpbiB1bmNoYW5nZWQuIHwgW2AwNGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMu
bWQjNi1yZXRhaW5lZHdvcmtlcm1hbmlmZXN0djEpIHwKK3wgYERpc3BhdGNoUG9saWN5TmFycm93
aW5nUGF0Y2hWMWAgfCBjb250cmFjdCB8IFtgY29udHJhY3RzL2Rpc3BhdGNoLXBvbGljeS1uYXJy
b3dpbmctcGF0Y2gtdjEubWRgXSguLi9jb250cmFjdHMvZGlzcGF0Y2gtcG9saWN5LW5hcnJvd2lu
Zy1wYXRjaC12MS5tZCkgfCBgY2Fub25pY2FsIGV4dHJhY3RlZCBjb250cmFjdCBvd25lciBmb3Ig
dGhlIGNvbXBsZXRlIHNjaGVtYSwgYERpc3BhdGNoQ2FwYWJpbGl0eVN1YmplY3RWMWAgZW51bSwg
YW5kIHRoZSBWMSBgUmVzdHJpY3RlZFBvbGljeVBhdGNoVjFgIHdvcmxkX2ZzLW9ubHkgcmVqZWN0
aW9uIHJ1bGVgIHwgU3VwZXJzZWRlcyBvbmx5IHJvb3QtY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0
aGUgZXh0cmFjdGVkIGBEaXNwYXRjaFBvbGljeU5hcnJvd2luZ1BhdGNoVjFgIHNwYW47IGBSZXRh
aW5lZFdvcmtlck1hbmlmZXN0VjFgIGFuZCBuZWlnaGJvcmluZyBEMTAgb3duZXJzIHJlbWFpbiB1
bmNoYW5nZWQuIHwgW2AwNGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjNy1kaXNwYXRj
aHBvbGljeW5hcnJvd2luZ3BhdGNodjEpIHwKIHwgYHNoYXJlZC10YXJnZXQtYXJjaGl0ZWN0dXJl
YCB8IGFyY2hpdGVjdHVyZSBpbmRleCB8IFtgYXJjaGl0ZWN0dXJlL1JFQURNRS5tZGBdKC4uL2Fy
Y2hpdGVjdHVyZS9SRUFETUUubWQpIHwgYG5vbi1hdXRob3JpdGF0aXZlIG5hdmlnYXRpb24gZm9y
IHRoZSBleHRyYWN0ZWQgZXhlY3V0aXZlIHRhcmdldCBkZWNpc2lvbiwgYXV0aG9yaXR5IG1hcCwg
bnVtYmVyZWQgaW52YXJpYW50cywgYW5kIHN0YWJsZSByZXZpZXcgcXVlc3Rpb25gIHwgU3VwZXJz
ZWRlcyBvbmx5IHJvb3QtY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIHNoYXJl
ZCBhcmNoaXRlY3R1cmUgc3BhbnM7IHBhY2tldC1mYW1pbHktbG9jYWwgRDUvRDYgZm9yd2FyZGVy
cyByZW1haW4gYXQgdGhlaXIgZXhpc3Rpbmcgb3duZXJzLiB8IFtgMDFgXSguLi8wMS10YXJnZXQt
YXJjaGl0ZWN0dXJlLm1kI2V4ZWN1dGl2ZS1kZWNpc2lvbiksIFtgMDFgXSguLi8wMS10YXJnZXQt
YXJjaGl0ZWN0dXJlLm1kI2F1dGhvcml0eS1tYXApLCBbYDAxYF0oLi4vMDEtdGFyZ2V0LWFyY2hp
dGVjdHVyZS5tZCNub24tbmVnb3RpYWJsZS1pbnZhcmlhbnRzKSwgW2AwMWBdKC4uLzAxLXRhcmdl
dC1hcmNoaXRlY3R1cmUubWQjcmV2aWV3LXF1ZXN0aW9uKSB8CiB8IGBzaGFyZWQtc2VhbS1jcm9z
c3dhbGtgIHwgc2VhbSBpbmRleCB8IFtgc2VhbXMvUkVBRE1FLm1kYF0oLi4vc2VhbXMvUkVBRE1F
Lm1kKSB8IGBjYW5vbmljYWwgc2hhcmVkIHNlYW0tY3Jvc3N3YWxrIHJ1bGVzIHBsdXMgZXh0cmFj
dGVkIHNlYW0tZmFtaWx5IG5hdmlnYXRpb24gZm9yIGhvc3Qvc2Vzc2lvbiwgcGVyc2lzdGVuY2Uv
Y29tcGF0aWJpbGl0eSwgZGlzcGF0Y2gvZXBpc29kZSB0cmFuc3BvcnQsIHBvbGljeS9uYXJyb3dp
bmcsIHJ1bnRpbWUtZXZlbnQvcmVjZWlwdC9zdXBlcnZpc2lvbi9yZXRhaW5lZC1ydW50aW1lLCBv
YmxpZ2F0aW9ucy9ob3N0IHJlLWVuZ2FnZW1lbnQsIGNvbmZpZ3VyYXRpb24vZ2F0ZXdheSBhZG9w
dGlvbiwgYW5kIFVBQS9wcm92aWRlciByZWFsaXphdGlvbi9zaWRlLWVmZmVjdCBtZWRpYXRpb25g
IHwgU3VwZXJzZWRlcyBvbmx5IHRoZSBleHRyYWN0ZWQgcm9vdCByZWFkaW5nLXJ1bGUvY2xhc3Np
ZmljYXRpb24gc3BhbnMgYW5kIHRoZSBleHRyYWN0ZWQgaG9zdC9zZXNzaW9uIGF1dGhvcml0eSwg
cGVyc2lzdGVuY2UvY29tcGF0aWJpbGl0eSwgZGlzcGF0Y2gvZXBpc29kZSB0cmFuc3BvcnQsIHBv
bGljeS9uYXJyb3dpbmcsIHJ1bnRpbWUtZXZlbnQvcmVjZWlwdC9zdXBlcnZpc2lvbi9yZXRhaW5l
ZC1ydW50aW1lLCBvYmxpZ2F0aW9ucy9ob3N0IHJlLWVuZ2FnZW1lbnQsIGNvbmZpZ3VyYXRpb24v
Z2F0ZXdheSBhZG9wdGlvbiwgYW5kIFVBQS9wcm92aWRlciByZWFsaXphdGlvbi9zaWRlLWVmZmVj
dCBtZWRpYXRpb24gZmFtaWx5IHJvd3M7IEEwIGFuZCB0aGUgZXhpc3RpbmcgRDYgYEhvc3RTZXNz
aW9uQXV0aG9yaXR5YCwgYFdvcmxkV29ya2VyTWVzc2FnaW5nUHJvdG9jb2xgLCBhbmQgYE9ibGln
YXRpb25MZWRnZXJgIGNvbXBhdGliaWxpdHkgcm93cyByZW1haW4gYXQgdGhlaXIgY3VycmVudCBv
d25lcnMuIHwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1kI3JlYWRpbmctcnVsZSksIFtg
MDJgXSguLi8wMi1zZWFtLWNyb3Nzd2Fsay5tZCNhLWhvc3QtYXV0aG9yaXR5LWFuZC1pbmdyZXNz
KSwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1kI2ItZGlzcGF0Y2gtcG9saWN5LXJlY2Vp
cHRzLWFuZC1yZXRhaW5lZC1ydW50aW1lKSwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1k
I2Mtb2JsaWdhdGlvbnMtYW5kLWhvc3QtcmUtZW5nYWdlbWVudCksIFtgMDJgXSguLi8wMi1zZWFt
LWNyb3Nzd2Fsay5tZCNkLXVhYS1yZWFsaXphdGlvbi1wcm9qZWN0aW9uLWFuZC1zaWRlLWVmZmVj
dC1tZWRpYXRpb24pLCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjY2xhc3NpZmljYXRp
b24tY29uc2VxdWVuY2VzKSB8CiB8IGBzaGFyZWQtc2xpY2UtbWFwYCB8IHNsaWNlIGluZGV4IHwg
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
bWQjc2xpY2UtY2xvc2VvdXQtbWluaW11bSkgfApkaWZmIC0tZ2l0IGEvbGxtLWxhc3QtbWlsZS9y
dW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZCBiL2xsbS1sYXN0
LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQKaW5k
ZXggMTZmZDYxMC4uZjU1NjZlNiAxMDA2NDQKLS0tIGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJl
ZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZAorKysgYi9sbG0tbGFzdC1taWxl
L3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJhY3Rpb24tbGVkZ2VyLm1kCkBAIC0xODgs
MyArMTg4LDUgQEAgVGhpcyBsZWRnZXIgcmVjb3JkcyBjb250ZW50LXByZXNlcnZpbmcgYXV0aG9y
aXR5IHRyYW5zZmVycyB3aGlsZSByZXRhaW5pbmcgcmVxdWkKIHwgRDEwIHwgW2AwNC1jb250cmFj
dHMtYW5kLWdhdGVzLm1kYF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCkgfCBEZWZlcnJl
ZCByZXRhaW5lZC1zcGF3biBhZG1pc3Npb24gcmVjb3ZlcnkgY29udHJhY3QgfCBbYGRlZmVycmVk
LXJldGFpbmVkLXNwYXduLWFkbWlzc2lvbi1yZWNvdmVyeS1jb250cmFjdGBdKC4uLzA0LWNvbnRy
YWN0cy1hbmQtZ2F0ZXMubWQjZGVmZXJyZWQtcmV0YWluZWQtc3Bhd24tYWRtaXNzaW9uLXJlY292
ZXJ5LWNvbnRyYWN0KSB8IGxpbmVzIDcx4oCTMjEzIHwgYGQ5MDMyOTliZGQzNDUzMTZmZDA5NDNk
MjhhYmMzMDBhMWNlZjAwMWNlMDc0NDc0NmY3MmY1MzYwN2ViNjRjMTJgIHwgW2AuLi9jb250cmFj
dHMvZGVmZXJyZWQtcmV0YWluZWQtc3Bhd24tYWRtaXNzaW9uLXJlY292ZXJ5LWNvbnRyYWN0Lm1k
YF0oLi4vY29udHJhY3RzL2RlZmVycmVkLXJldGFpbmVkLXNwYXduLWFkbWlzc2lvbi1yZWNvdmVy
eS1jb250cmFjdC5tZCkgfCBjb250cmFjdCB8IGNhbm9uaWNhbCBkZXN0aW5hdGlvbjsgc291cmNl
IGNvbXBhdGliaWxpdHkgYW5jaG9yIHwgbm9uZSB8IHJlc3RvcmUgdGhlIGV4YWN0IDk4OTYtYnl0
ZSBzb3VyY2Ugc3BhbiBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCNkZWZlcnJlZC1yZXRh
aW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29udHJhY3RgOyByZW1vdmUgYGNvbnRyYWN0
cy9kZWZlcnJlZC1yZXRhaW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29udHJhY3QubWRg
OyByZW1vdmUgdGhlIGV4YWN0IGBEZWZlcnJlZCByZXRhaW5lZC1zcGF3biBhZG1pc3Npb24gcmVj
b3ZlcnkgY29udHJhY3RgIHJvdyBmcm9tIGBpbmRleC9SRUFETUUubWRgOyByZW1vdmUgdGhpcyBE
MTAgbGVkZ2VyIGVudHJ5OyBsZWF2ZSBgSG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWAsIHRoZSB0d28g
cHJlcmVxdWlzaXRlIGNvbXBhdGliaWxpdHkgYW5jaG9ycywgYWxsIEQzL0Q14oCTRDkgb3duZXJz
LCBgaW5kZXgvY3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9gLCBsYXRlciBEMTAgdW5pdHMs
IGFuZCBldmVyeSBwYXRoIG91dHNpZGUgdGhlIGZvdXItcGF0aCBmZW5jZSB1bmNoYW5nZWQgfAog
fCBEMTAgfCBbYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWRgXSguLi8wNC1jb250cmFjdHMtYW5k
LWdhdGVzLm1kKSB8IDMuIGBBY3RpdmVFcGhlbWVyYWxUYXNrUmVjZWlwdFYxYCB8IFtgMy1hY3Rp
dmVlcGhlbWVyYWx0YXNrcmVjZWlwdHYxYF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMz
LWFjdGl2ZWVwaGVtZXJhbHRhc2tyZWNlaXB0djEpIHwgbGluZXMgOTHigJMxMzggfCBgYzk5Nzc1
MjY1NGYwNDFkZTgyMDIxMGYwM2M4NjcwNjU1MWI0NDM4OTk5NGRmNWU4ZTAzNDFhYTQxYTM0ZmEz
YmAgfCBbYC4uL2NvbnRyYWN0cy9hY3RpdmUtZXBoZW1lcmFsLXRhc2stcmVjZWlwdC12MS5tZGBd
KC4uL2NvbnRyYWN0cy9hY3RpdmUtZXBoZW1lcmFsLXRhc2stcmVjZWlwdC12MS5tZCkgfCBjb250
cmFjdCB8IGNhbm9uaWNhbCBkZXN0aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9y
IHwgbm9uZSB8IHJlc3RvcmUgdGhlIGV4YWN0IDE1NTctYnl0ZSBzb3VyY2Ugc3BhbiBhdCBgMDQt
Y29udHJhY3RzLWFuZC1nYXRlcy5tZCMzLWFjdGl2ZWVwaGVtZXJhbHRhc2tyZWNlaXB0djFgIGFu
ZCB0aGUgZXhhY3QgMTQ0My1ieXRlIHNvdXJjZSBzcGFuIGF0IGAwNC1jb250cmFjdHMtYW5kLWdh
dGVzLm1kIzQtYWN0aXZlcmV0YWluZWR0dXJucmVjZWlwdHYxYDsgcmVtb3ZlIGBjb250cmFjdHMv
YWN0aXZlLWVwaGVtZXJhbC10YXNrLXJlY2VpcHQtdjEubWRgIGFuZCBgY29udHJhY3RzL2FjdGl2
ZS1yZXRhaW5lZC10dXJuLXJlY2VpcHQtdjEubWRgOyByZW1vdmUgdGhlIGV4YWN0IGBBY3RpdmVF
cGhlbWVyYWxUYXNrUmVjZWlwdFYxYCBhbmQgYEFjdGl2ZVJldGFpbmVkVHVyblJlY2VpcHRWMWAg
cm93cyBmcm9tIGBpbmRleC9SRUFETUUubWRgOyByZW1vdmUgYm90aCBEMTAgbGVkZ2VyIGVudHJp
ZXMgZm9yIHRoaXMgYmF0Y2g7IGxlYXZlIGBIb3N0RXhlY3V0aW9uRXBpc29kZVYxYCwgZGVmZXJy
ZWQgcmV0YWluZWQtc3Bhd24gYWRtaXNzaW9uIHJlY292ZXJ5LCBgUmVjZWlwdCBhY2NlcHRhbmNl
IHNvdXJjZWAsIGFsbCBEMy9ENeKAk0Q5IG93bmVycywgYGluZGV4L2N1cnJlbnQubWRgLCBgcmV2
aWV3LWNvbnRyb2wvYCwgcmVtYWluaW5nIEQxMCB1bml0cywgYW5kIGV2ZXJ5IHBhdGggb3V0c2lk
ZSB0aGUgZml2ZS1wYXRoIGZlbmNlIHVuY2hhbmdlZCB8CiB8IEQxMCB8IFtgMDQtY29udHJhY3Rz
LWFuZC1nYXRlcy5tZGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQpIHwgNC4gYEFjdGl2
ZVJldGFpbmVkVHVyblJlY2VpcHRWMWAgfCBbYDQtYWN0aXZlcmV0YWluZWR0dXJucmVjZWlwdHYx
YF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM0LWFjdGl2ZXJldGFpbmVkdHVybnJlY2Vp
cHR2MSkgfCBsaW5lcyAxMznigJMxODMgfCBgMmNjOTk2MTExZDgyZDI5NmY4OWJkZDg4ZjgwZGY1
YzBiNTU5YmRiMzA1YWVjNTQ2ODgxYjc0OWFjZTkzZjY3ZWAgfCBbYC4uL2NvbnRyYWN0cy9hY3Rp
dmUtcmV0YWluZWQtdHVybi1yZWNlaXB0LXYxLm1kYF0oLi4vY29udHJhY3RzL2FjdGl2ZS1yZXRh
aW5lZC10dXJuLXJlY2VpcHQtdjEubWQpIHwgY29udHJhY3QgfCBjYW5vbmljYWwgZGVzdGluYXRp
b247IHNvdXJjZSBjb21wYXRpYmlsaXR5IGFuY2hvciB8IG5vbmUgfCByZXN0b3JlIHRoZSBleGFj
dCAxNTU3LWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMy1h
Y3RpdmVlcGhlbWVyYWx0YXNrcmVjZWlwdHYxYCBhbmQgdGhlIGV4YWN0IDE0NDMtYnl0ZSBzb3Vy
Y2Ugc3BhbiBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM0LWFjdGl2ZXJldGFpbmVkdHVy
bnJlY2VpcHR2MWA7IHJlbW92ZSBgY29udHJhY3RzL2FjdGl2ZS1lcGhlbWVyYWwtdGFzay1yZWNl
aXB0LXYxLm1kYCBhbmQgYGNvbnRyYWN0cy9hY3RpdmUtcmV0YWluZWQtdHVybi1yZWNlaXB0LXYx
Lm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgQWN0aXZlRXBoZW1lcmFsVGFza1JlY2VpcHRWMWAgYW5k
IGBBY3RpdmVSZXRhaW5lZFR1cm5SZWNlaXB0VjFgIHJvd3MgZnJvbSBgaW5kZXgvUkVBRE1FLm1k
YDsgcmVtb3ZlIGJvdGggRDEwIGxlZGdlciBlbnRyaWVzIGZvciB0aGlzIGJhdGNoOyBsZWF2ZSBg
SG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWAsIGRlZmVycmVkIHJldGFpbmVkLXNwYXduIGFkbWlzc2lv
biByZWNvdmVyeSwgYFJlY2VpcHQgYWNjZXB0YW5jZSBzb3VyY2VgLCBhbGwgRDMvRDXigJNEOSBv
d25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYHJldmlldy1jb250cm9sL2AsIHJlbWFpbmluZyBE
MTAgdW5pdHMsIGFuZCBldmVyeSBwYXRoIG91dHNpZGUgdGhlIGZpdmUtcGF0aCBmZW5jZSB1bmNo
YW5nZWQgfAorfCBEMTAgfCBbYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWRgXSguLi8wNC1jb250
cmFjdHMtYW5kLWdhdGVzLm1kKSB8IDYuIGBSZXRhaW5lZFdvcmtlck1hbmlmZXN0VjFgIHwgW2A2
LXJldGFpbmVkd29ya2VybWFuaWZlc3R2MWBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQj
Ni1yZXRhaW5lZHdvcmtlcm1hbmlmZXN0djEpIHwgbGluZXMgMTAz4oCTMTQwICgxNTc2IGJ5dGVz
KSB8IGBjZDI1ZWMxYzE5NTQ4Yjg0NDJlMDFmNzUzNTE3YzExN2NhYjlhMzUwY2I4MjhkZTgyMTRh
MzUxYmRjMDllZTA3YCB8IFtgLi4vY29udHJhY3RzL3JldGFpbmVkLXdvcmtlci1tYW5pZmVzdC12
MS5tZGBdKC4uL2NvbnRyYWN0cy9yZXRhaW5lZC13b3JrZXItbWFuaWZlc3QtdjEubWQpIHwgY29u
dHJhY3QgfCBjYW5vbmljYWwgZGVzdGluYXRpb247IHNvdXJjZSBjb21wYXRpYmlsaXR5IGFuY2hv
ciB8IG5vbmUgfCByZXN0b3JlIHRoZSBleGFjdCAxNTc2LWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0
LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjNi1yZXRhaW5lZHdvcmtlcm1hbmlmZXN0djFgIGFuZCB0
aGUgZXhhY3QgODEwLWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMu
bWQjNy1kaXNwYXRjaHBvbGljeW5hcnJvd2luZ3BhdGNodjFgOyByZW1vdmUgYGNvbnRyYWN0cy9y
ZXRhaW5lZC13b3JrZXItbWFuaWZlc3QtdjEubWRgIGFuZCBgY29udHJhY3RzL2Rpc3BhdGNoLXBv
bGljeS1uYXJyb3dpbmctcGF0Y2gtdjEubWRgOyByZW1vdmUgdGhlIGV4YWN0IGBSZXRhaW5lZFdv
cmtlck1hbmlmZXN0VjFgIGFuZCBgRGlzcGF0Y2hQb2xpY3lOYXJyb3dpbmdQYXRjaFYxYCByb3dz
IGZyb20gYGluZGV4L1JFQURNRS5tZGA7IHJlbW92ZSBib3RoIEQxMCBsZWRnZXIgZW50cmllcyBm
b3IgdGhpcyBiYXRjaDsgbGVhdmUgYFJlY2VpcHQgYWNjZXB0YW5jZSBzb3VyY2VgLCBgV29ybGRS
dW50aW1lQWRhcHRlckV4ZWN1dGlvbkVudmVsb3BlVjFgLCBhbGwgRDMvRDXigJNEMTAgb3duZXJz
LCBgaW5kZXgvY3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9gLCByZW1haW5pbmcgbGF0ZXIg
RDEwIHVuaXRzLCBhbmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmaXZlLXBhdGggZmVuY2UgdW5j
aGFuZ2VkIHwKK3wgRDEwIHwgW2AwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kYF0oLi4vMDQtY29u
dHJhY3RzLWFuZC1nYXRlcy5tZCkgfCA3LiBgRGlzcGF0Y2hQb2xpY3lOYXJyb3dpbmdQYXRjaFYx
YCB8IFtgNy1kaXNwYXRjaHBvbGljeW5hcnJvd2luZ3BhdGNodjFgXSguLi8wNC1jb250cmFjdHMt
YW5kLWdhdGVzLm1kIzctZGlzcGF0Y2hwb2xpY3luYXJyb3dpbmdwYXRjaHYxKSB8IGxpbmVzIDE0
MeKAkzE2NyAoODEwIGJ5dGVzKSB8IGBjMGZkMjJhN2M2MzBjMzBlOGI2ZmNhOTc2NjQ1ZDg0ZGM3
NTE1YjQzNWFmNzU5YzRlY2MwYmM4Yjg4NzFkYTRiYCB8IFtgLi4vY29udHJhY3RzL2Rpc3BhdGNo
LXBvbGljeS1uYXJyb3dpbmctcGF0Y2gtdjEubWRgXSguLi9jb250cmFjdHMvZGlzcGF0Y2gtcG9s
aWN5LW5hcnJvd2luZy1wYXRjaC12MS5tZCkgfCBjb250cmFjdCB8IGNhbm9uaWNhbCBkZXN0aW5h
dGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9yIHwgbm9uZSB8IHJlc3RvcmUgdGhlIGV4
YWN0IDE1NzYtYnl0ZSBzb3VyY2Ugc3BhbiBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM2
LXJldGFpbmVkd29ya2VybWFuaWZlc3R2MWAgYW5kIHRoZSBleGFjdCA4MTAtYnl0ZSBzb3VyY2Ug
c3BhbiBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM3LWRpc3BhdGNocG9saWN5bmFycm93
aW5ncGF0Y2h2MWA7IHJlbW92ZSBgY29udHJhY3RzL3JldGFpbmVkLXdvcmtlci1tYW5pZmVzdC12
MS5tZGAgYW5kIGBjb250cmFjdHMvZGlzcGF0Y2gtcG9saWN5LW5hcnJvd2luZy1wYXRjaC12MS5t
ZGA7IHJlbW92ZSB0aGUgZXhhY3QgYFJldGFpbmVkV29ya2VyTWFuaWZlc3RWMWAgYW5kIGBEaXNw
YXRjaFBvbGljeU5hcnJvd2luZ1BhdGNoVjFgIHJvd3MgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsg
cmVtb3ZlIGJvdGggRDEwIGxlZGdlciBlbnRyaWVzIGZvciB0aGlzIGJhdGNoOyBsZWF2ZSBgUmVj
ZWlwdCBhY2NlcHRhbmNlIHNvdXJjZWAsIGBXb3JsZFJ1bnRpbWVBZGFwdGVyRXhlY3V0aW9uRW52
ZWxvcGVWMWAsIGFsbCBEMy9ENeKAk0QxMCBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYHJl
dmlldy1jb250cm9sL2AsIHJlbWFpbmluZyBsYXRlciBEMTAgdW5pdHMsIGFuZCBldmVyeSBwYXRo
IG91dHNpZGUgdGhlIGZpdmUtcGF0aCBmZW5jZSB1bmNoYW5nZWQgfApgYGAK
```

</details>

## Preserved review answer

<details>
<summary>Initial-range review answer (rendered copy)</summary>

``````text
No qualifying P1 or P2 findings.

I verified the complete embedded patch is content-addressed correctly: its SHA-256 is 5e3878b743106c01c0870419ff697a466909ae4505e1f2d14dec5c14c53532a7, and it touches exactly the five declared paths.

Both canonical extractions are mechanically exact. RetainedWorkerManifestV1 reconstructs the baseline lines 103–140 as exactly 1576 bytes with SHA-256 cd25ec1c...ee07, preserving the complete field ordering and rules 1–5. DispatchPolicyNarrowingPatchV1 reconstructs baseline lines 141–167 as exactly 810 bytes with SHA-256 c0fd22a7...da4b, preserving the enum ordering and the world_fs-only reject-not-ignore requirement. The marker-bounded bodies match those reconstructed deleted spans byte-for-byte.

The root retains both required legacy headings while replacing only their bodies with links to the corresponding canonical owners. The two index rows point to the correct new owners/root compatibility anchors, and both ledger entries carry the exact source lines, byte counts, hashes, and symmetric family-atomic rollback operations restoring both root bodies while removing both owners, index rows, and ledger entries together.

Deferred, non-blocking: the index and rollback preservation prose uses “remain unchanged”/“all D3/D5–D10 owners … unchanged” broadly enough that “other” would be slightly clearer when referring to the sibling/current D10 pair. The explicit per-owner supersession statements and exact paired rollback operations remove any material ambiguity, so this does not meet the stated blocking threshold.

VERDICT: APPROVED
``````

</details>

<details>
<summary>Initial-range review answer exact bytes (base64 UTF-8)</summary>

```text
Tm8gcXVhbGlmeWluZyBQMSBvciBQMiBmaW5kaW5ncy4KCkkgdmVyaWZpZWQgdGhlIGNvbXBsZXRl
IGVtYmVkZGVkIHBhdGNoIGlzIGNvbnRlbnQtYWRkcmVzc2VkIGNvcnJlY3RseTogaXRzIFNIQS0y
NTYgaXMgNWUzODc4Yjc0MzEwNmMwMWMwODcwNDE5ZmY2OTdhNDY2OTA5YWU0NTA1ZTFmMmQxNGRl
YzVjMTRjNTM1MzJhNywgYW5kIGl0IHRvdWNoZXMgZXhhY3RseSB0aGUgZml2ZSBkZWNsYXJlZCBw
YXRocy4KCkJvdGggY2Fub25pY2FsIGV4dHJhY3Rpb25zIGFyZSBtZWNoYW5pY2FsbHkgZXhhY3Qu
IFJldGFpbmVkV29ya2VyTWFuaWZlc3RWMSByZWNvbnN0cnVjdHMgdGhlIGJhc2VsaW5lIGxpbmVz
IDEwM+KAkzE0MCBhcyBleGFjdGx5IDE1NzYgYnl0ZXMgd2l0aCBTSEEtMjU2IGNkMjVlYzFjLi4u
ZWUwNywgcHJlc2VydmluZyB0aGUgY29tcGxldGUgZmllbGQgb3JkZXJpbmcgYW5kIHJ1bGVzIDHi
gJM1LiBEaXNwYXRjaFBvbGljeU5hcnJvd2luZ1BhdGNoVjEgcmVjb25zdHJ1Y3RzIGJhc2VsaW5l
IGxpbmVzIDE0MeKAkzE2NyBhcyBleGFjdGx5IDgxMCBieXRlcyB3aXRoIFNIQS0yNTYgYzBmZDIy
YTcuLi5kYTRiLCBwcmVzZXJ2aW5nIHRoZSBlbnVtIG9yZGVyaW5nIGFuZCB0aGUgd29ybGRfZnMt
b25seSByZWplY3Qtbm90LWlnbm9yZSByZXF1aXJlbWVudC4gVGhlIG1hcmtlci1ib3VuZGVkIGJv
ZGllcyBtYXRjaCB0aG9zZSByZWNvbnN0cnVjdGVkIGRlbGV0ZWQgc3BhbnMgYnl0ZS1mb3ItYnl0
ZS4KClRoZSByb290IHJldGFpbnMgYm90aCByZXF1aXJlZCBsZWdhY3kgaGVhZGluZ3Mgd2hpbGUg
cmVwbGFjaW5nIG9ubHkgdGhlaXIgYm9kaWVzIHdpdGggbGlua3MgdG8gdGhlIGNvcnJlc3BvbmRp
bmcgY2Fub25pY2FsIG93bmVycy4gVGhlIHR3byBpbmRleCByb3dzIHBvaW50IHRvIHRoZSBjb3Jy
ZWN0IG5ldyBvd25lcnMvcm9vdCBjb21wYXRpYmlsaXR5IGFuY2hvcnMsIGFuZCBib3RoIGxlZGdl
ciBlbnRyaWVzIGNhcnJ5IHRoZSBleGFjdCBzb3VyY2UgbGluZXMsIGJ5dGUgY291bnRzLCBoYXNo
ZXMsIGFuZCBzeW1tZXRyaWMgZmFtaWx5LWF0b21pYyByb2xsYmFjayBvcGVyYXRpb25zIHJlc3Rv
cmluZyBib3RoIHJvb3QgYm9kaWVzIHdoaWxlIHJlbW92aW5nIGJvdGggb3duZXJzLCBpbmRleCBy
b3dzLCBhbmQgbGVkZ2VyIGVudHJpZXMgdG9nZXRoZXIuCgpEZWZlcnJlZCwgbm9uLWJsb2NraW5n
OiB0aGUgaW5kZXggYW5kIHJvbGxiYWNrIHByZXNlcnZhdGlvbiBwcm9zZSB1c2VzIOKAnHJlbWFp
biB1bmNoYW5nZWTigJ0v4oCcYWxsIEQzL0Q14oCTRDEwIG93bmVycyDigKYgdW5jaGFuZ2Vk4oCd
IGJyb2FkbHkgZW5vdWdoIHRoYXQg4oCcb3RoZXLigJ0gd291bGQgYmUgc2xpZ2h0bHkgY2xlYXJl
ciB3aGVuIHJlZmVycmluZyB0byB0aGUgc2libGluZy9jdXJyZW50IEQxMCBwYWlyLiBUaGUgZXhw
bGljaXQgcGVyLW93bmVyIHN1cGVyc2Vzc2lvbiBzdGF0ZW1lbnRzIGFuZCBleGFjdCBwYWlyZWQg
cm9sbGJhY2sgb3BlcmF0aW9ucyByZW1vdmUgYW55IG1hdGVyaWFsIGFtYmlndWl0eSwgc28gdGhp
cyBkb2VzIG5vdCBtZWV0IHRoZSBzdGF0ZWQgYmxvY2tpbmcgdGhyZXNob2xkLgoKVkVSRElDVDog
QVBQUk9WRUQK
```

</details>
