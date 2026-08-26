# ChatGPT Pro advisory review: runtime-refactor D10 execution-envelope/secret-handoff family

- Date: 2026-08-26
- Bound baseline commit/tree: `efa43b255feeb8847d2131b76f54c85a8234fc8a` / `c8ce1e025d416a0340cc9c5a9eb8ae4dbf1f6e1a`
- Candidate implementation subagent: `/root/d10_execution_secret_landing` (`gpt-5.4`, Extra High; display name not exposed)
- Closeout subagent: `/root/d10_execution_secret_closeout`
- Root reviewer/adjudicator: `/root`
- Independent review chat: https://chatgpt.com/c/6a8f59ad-95f4-83ea-8287-4f575752b248
- Review mode: `initial-range`
- Review context: independent fresh conversation
- ChatGPT account surface: `Pro`
- Visible reasoning-effort control: `Extra High`
- Exact model label: not exposed by the current visible ChatGPT UI
- Candidate patch: `sha256:c9f8d90d04defb36f48eb3d51762fdd98c31972bdea502cbe569eac1160a1544` at `/private/tmp/d10-execution-secret-family-candidate.patch`
- Review prompt: `sha256:536a905ca67636034f1b3853fa0659138b1f7492f9be195dc2801fb91636e1b4` at `/private/tmp/d10-execution-secret-family-chatgpt-pro-initial-review-prompt.txt`
- Review answer: `sha256:3c6db626c0a80542232f41cfc120b22eaad0a78a1f744a61addbca8fc82b10b9` at `/private/tmp/d10-execution-secret-family-chatgpt-pro-initial-review-answer.txt`
- Imported validation log: `sha256:2e60e11b35165e4036f192b52d05ee0db9d2f8a359ebd9e6b8bb6227228b5d4a` at `/private/tmp/d10-execution-secret-family-validation.log`
- Exact embedded source body (`WorldRuntimeAdapterExecutionEnvelopeV1`): baseline lines `111–175`, `2783 bytes`, `sha256:cdb755ac00cea99be0327f87547fd586f1c6335c470113cea03307cc8e3d8eac`
- Exact embedded source body (`LaunchTimeSecretHandoffV1`): baseline lines `176–258`, `4968 bytes`, `sha256:0a5a13799969f165c3acca200dfd5f28bf2e74ccbfeb71356176caecec78b0cd`
- Review verdict: `APPROVED`
- Review finding summary: `No qualifying P1/P2 findings.`
- Remediation rounds: `0`
- Review adjudication posture: root agent owns review/adjudication; subagents do not self-approve
- Commit posture: the candidate remains uncommitted and unstaged during closeout; root will validate, run GitNexus, and commit the seven-path batch atomically; not yet committed
- Push posture: `not pushed`
- Successor authority posture: no successor dispatch authority is granted

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Candidate scope and outcome

This independently reviewed substantive D10 family extracts `WorldRuntimeAdapterExecutionEnvelopeV1` and `LaunchTimeSecretHandoffV1` into `llm-last-mile/runtime-refactor/contracts/world-runtime-adapter-execution-envelope-v1.md` and `llm-last-mile/runtime-refactor/contracts/launch-time-secret-handoff-v1.md`, replaces only the two root spans in `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` with shallow compatibility pointers, adds the two truthful index rows, and adds the two matching D10 extraction-ledger rows. The family preserves the positive landed `GatewayAuthBundleV1` secure-FD carrier fact while keeping the durable orchestration record and joined adoption incomplete. This documentation extraction is not implementation or gate satisfaction. The execution-envelope/secret-handoff family is complete locally and approved after initial-range review with zero remediation and is landed pending one atomic commit. D10 remains explicitly in progress and incomplete; the exact next/final batched substantive D10 unit is `Cancel outcome categories` + `Final-receipt immutable PolicySnapshotV3 acceptance rules` + `Dispatch narrowing monotonicity rules` + `Contract promotion gates`; no later D10 unit should be claimed complete unless an exhaustive audit proves it; D11 remains blocked pending full D10 completion and separate authorization; D12 remains blocked by D11 and is separately unauthorized; no successor dispatch authority is granted.

## Prospective seven-path landing manifest

Prospective landing manifest for this independently reviewed family:

- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/contracts/world-runtime-adapter-execution-envelope-v1.md`
- `llm-last-mile/runtime-refactor/contracts/launch-time-secret-handoff-v1.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `docs/guidance/2026-08-26-runtime-refactor-d10-execution-envelope-secret-handoff-family-chatgpt-pro-review.md`

## Candidate path digests and imported review artifacts

Candidate path digests:

- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`: `sha256:b19abe901ae54a681b1ae95d25697d43e47bfa621a5c4af916fe46218a7ed97e`
- `llm-last-mile/runtime-refactor/contracts/world-runtime-adapter-execution-envelope-v1.md`: `sha256:e9bf40b72241151c5191a038318d94c14d778add0ebcbdebd89d2356f59d4c0f`
- `llm-last-mile/runtime-refactor/contracts/launch-time-secret-handoff-v1.md`: `sha256:c7480e659bd2a41a4b7b33c50b473d4476b29de00553ed710cb8f406d1ee0954`
- `llm-last-mile/runtime-refactor/index/README.md`: `sha256:0bd2d5ea44d1d21c9997ac6c0ecb9ee7def2245de9b2d32f7b84474df23ed7ea`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`: `sha256:1630eae40e50682f7d8eba612e2e1b666f35e4bbb9116786bf2783c37bcddc63`

Imported review artifacts:

- Candidate patch: `sha256:c9f8d90d04defb36f48eb3d51762fdd98c31972bdea502cbe569eac1160a1544`
- Review prompt: `sha256:536a905ca67636034f1b3853fa0659138b1f7492f9be195dc2801fb91636e1b4`
- Review answer: `sha256:3c6db626c0a80542232f41cfc120b22eaad0a78a1f744a61addbca8fc82b10b9`
- Imported validation log: `sha256:2e60e11b35165e4036f192b52d05ee0db9d2f8a359ebd9e6b8bb6227228b5d4a`
- Exact embedded source body (`WorldRuntimeAdapterExecutionEnvelopeV1`): `2783 bytes`, `sha256:cdb755ac00cea99be0327f87547fd586f1c6335c470113cea03307cc8e3d8eac`
- Exact embedded source body (`LaunchTimeSecretHandoffV1`): `4968 bytes`, `sha256:0a5a13799969f165c3acca200dfd5f28bf2e74ccbfeb71356176caecec78b0cd`

## Exact source-body proof

- Baseline `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` lines `111–175` are exactly `2783` bytes with SHA-256 `cdb755ac00cea99be0327f87547fd586f1c6335c470113cea03307cc8e3d8eac`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` in `llm-last-mile/runtime-refactor/contracts/world-runtime-adapter-execution-envelope-v1.md` are also exactly `2783` bytes with SHA-256 `cdb755ac00cea99be0327f87547fd586f1c6335c470113cea03307cc8e3d8eac`.
- Baseline `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` lines `176–258` are exactly `4968` bytes with SHA-256 `0a5a13799969f165c3acca200dfd5f28bf2e74ccbfeb71356176caecec78b0cd`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` in `llm-last-mile/runtime-refactor/contracts/launch-time-secret-handoff-v1.md` are also exactly `4968` bytes with SHA-256 `0a5a13799969f165c3acca200dfd5f28bf2e74ccbfeb71356176caecec78b0cd`.
- The baseline source spans and the two owner-body byte ranges are byte-identical, including the final blank separator before each end marker.
- The two ledger entries preserve paired-family atomic rollback, and the `LaunchTimeSecretHandoffV1` owner preserves the positive landed `GatewayAuthBundleV1` secure-FD carrier fact while keeping durable orchestration/joined adoption incomplete.

## Validation evidence

- The supplied review answer states `No qualifying P1/P2 findings.` and ends with `VERDICT: APPROVED`.
- The imported validation log confirms unique root anchors for `8-worldruntimeadapterexecutionenvelopev1` and `9-launchtimesecrethandoffv1`, unique owner anchors for both canonical files, exact marker-body equality for both owners, exactly one index row and one ledger row per owner, the exact five-path candidate fence, and clean candidate-patch forward/reverse application.
- The candidate patch changes exactly the five imported candidate paths and no others.
- Candidate file hashes, candidate-patch SHA-256, prompt SHA-256, answer SHA-256, validation-log SHA-256, and both exact source-body SHA-256 digests were all reverified during this closeout.
- The rendered prompt and rendered answer below are whitespace-clean: every trailing literal space from the imported UTF-8 prompt or answer is rendered as `&#32;` so this repository-local Markdown artifact stays clean under `git diff --check` while the base64 blocks preserve the exact bytes.
- The rendered prompt and rendered answer were both revalidated against the imported files, and the embedded base64 blocks decode exactly back to the imported UTF-8 bytes without silently correcting any captured whitespace, diff fences, or wording.
- Final changed-path fence including untracked files is exactly these seven paths:
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/contracts/world-runtime-adapter-execution-envelope-v1.md`
- `llm-last-mile/runtime-refactor/contracts/launch-time-secret-handoff-v1.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `docs/guidance/2026-08-26-runtime-refactor-d10-execution-envelope-secret-handoff-family-chatgpt-pro-review.md`
- `git diff --check` passed for the final closeout state.
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/contracts/world-runtime-adapter-execution-envelope-v1.md` reported no whitespace diagnostics.
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/contracts/launch-time-secret-handoff-v1.md` reported no whitespace diagnostics.
- `git diff --no-index --check /dev/null docs/guidance/2026-08-26-runtime-refactor-d10-execution-envelope-secret-handoff-family-chatgpt-pro-review.md` reported no whitespace diagnostics.
- Tracker truth validation passed for only the execution-envelope/secret-handoff family approved, D10 still incomplete, the exact next/final batched unit `Cancel outcome categories` + `Final-receipt immutable PolicySnapshotV3 acceptance rules` + `Dispatch narrowing monotonicity rules` + `Contract promotion gates`, no later D10 unit claimed complete without exhaustive audit, D11 blocked pending full D10 completion and separate authorization, D12 blocked by D11 and separately unauthorized, and no successor dispatch authority.
- Prior D3/D5–D10 owner/review-artifact stability passed via the exact seven-path fence.
- Runtime-refactor Markdown link/anchor validation passed with `LINKS_CHECKED 1861 FAILURES 0`; tracker and review-record link checks also passed.
- Complete combined candidate+closeout patch was generated at `/private/tmp/d10-execution-secret-family-closeout.patch` with a SHA-256 sidecar, and forward/reverse apply against disposable clean baseline exports both passed.
- No commit, stage, or push is performed during this closeout.

## Atomic rollback instructions

1. If the prospective seven-path batch has been committed, reverse that single batch commit atomically (for example, `git revert --no-commit <batch-commit>`) and verify that only the seven paths above are touched.
2. If the batch is still uncommitted, restore the tracked paths from the parent baseline and remove the three untracked files: `git restore --source=HEAD -- llm-last-mile/runtime-refactor/04-contracts-and-gates.md llm-last-mile/runtime-refactor/index/README.md llm-last-mile/runtime-refactor/migration/extraction-ledger.md docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md` then `rm -f llm-last-mile/runtime-refactor/contracts/world-runtime-adapter-execution-envelope-v1.md llm-last-mile/runtime-refactor/contracts/launch-time-secret-handoff-v1.md docs/guidance/2026-08-26-runtime-refactor-d10-execution-envelope-secret-handoff-family-chatgpt-pro-review.md`.
3. After either rollback path, confirm together that both `04-contracts-and-gates.md` bodies are restored, both canonical owner files are removed, both `index/README.md` rows are removed, both `migration/extraction-ledger.md` rows are removed, the tracker summary/queue/closeout-row additions are reverted, and the review record is removed.
4. Preserve the positive landed `GatewayAuthBundleV1` secure-FD carrier fact while reverting this documentation family; do not convert rollback into a claim that joined durable orchestration/adoption is complete or that the carrier was absent.
5. Never strand one family member: both owners, both root compatibility pointers, both index rows, both ledger rows, the tracker update, and this review record roll back together or not at all.

## Final verdict

`WorldRuntimeAdapterExecutionEnvelopeV1` + `LaunchTimeSecretHandoffV1` is complete locally and **APPROVED** after initial-range review with zero remediation and is landed pending one atomic commit, but D10 remains explicitly in progress and incomplete; the exact next/final batched substantive D10 unit is `Cancel outcome categories` + `Final-receipt immutable PolicySnapshotV3 acceptance rules` + `Dispatch narrowing monotonicity rules` + `Contract promotion gates`; no later D10 unit should be claimed complete unless an exhaustive audit proves it; D11 remains blocked pending full D10 completion and separate authorization; D12 remains blocked by D11 and is separately unauthorized; no successor dispatch authority is granted.

## Preserved review prompt

<details>
<summary>Initial-range review prompt (rendered copy)</summary>

``````text
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: exact baseline commit efa43b255feeb8847d2131b76f54c85a8234fc8a, tree c8ce1e025d416a0340cc9c5a9eb8ae4dbf1f6e1a, plus the complete content-addressed candidate patch below. Candidate patch SHA-256: c9f8d90d04defb36f48eb3d51762fdd98c31972bdea502cbe569eac1160a1544.
Review target: complete bounded D10 documentation extraction for the inseparable WorldRuntimeAdapterExecutionEnvelopeV1 + LaunchTimeSecretHandoffV1 family. Review every hunk and both new files. Do not infer unprovided source.

Manifest (exactly five paths including untracked new files):
- llm-last-mile/runtime-refactor/04-contracts-and-gates.md — SHA-256 b19abe901ae54a681b1ae95d25697d43e47bfa621a5c4af916fe46218a7ed97e
- llm-last-mile/runtime-refactor/contracts/world-runtime-adapter-execution-envelope-v1.md — new — SHA-256 e9bf40b72241151c5191a038318d94c14d778add0ebcbdebd89d2356f59d4c0f
- llm-last-mile/runtime-refactor/contracts/launch-time-secret-handoff-v1.md — new — SHA-256 c7480e659bd2a41a4b7b33c50b473d4476b29de00553ed710cb8f406d1ee0954
- llm-last-mile/runtime-refactor/index/README.md — SHA-256 0bd2d5ea44d1d21c9997ac6c0ecb9ee7def2245de9b2d32f7b84474df23ed7ea
- llm-last-mile/runtime-refactor/migration/extraction-ledger.md — SHA-256 1630eae40e50682f7d8eba612e2e1b666f35e4bbb9116786bf2783c37bcddc63

Supported inputs and behavior: GitHub-flavored Markdown with ATX headings/generated anchors, relative links/fragments, fenced Rust/text blocks, ordered lists, Markdown tables with semantic row/cell order, inline code/literals, and HTML body-hash markers. Legacy root remains a compatibility projection until separately authorized D12. Canonical owners are normative; root headings/anchors remain supported.
Supported platforms and dialects: repository Markdown with GitHub-style heading slugs and the project link/anchor checker. Runtime/Rust/script/platform/parser/credential transport behavior is not changed by this patch.

In-scope invariants:
1. Exactly one canonical owner exists for each named V1 contract; the two contracts land and roll back together because SecureGatewayHandoff references LaunchTimeSecretHandoffV1 and exact gateway/world generation.
2. Canonical bodies are byte-identical to frozen root spans. WorldRuntimeAdapterExecutionEnvelopeV1 is 2783 bytes, SHA-256 cdb755ac00cea99be0327f87547fd586f1c6335c470113cea03307cc8e3d8eac. LaunchTimeSecretHandoffV1 is 4968 bytes, SHA-256 0a5a13799969f165c3acca200dfd5f28bf2e74ccbfeb71356176caecec78b0cd.
3. Preserve stable V1 names, schema/comment text, field order, enum/variant order, envelope kinds, world binding, immutable policy, and command_broker_required requirements.
4. Preserve the D1-before-D2 CompatibilityUnproven exception exactly: named/logged compatibility may continue, but cannot claim mediation, satisfy caging/broker gates, or promote; BrokerRequired fails closed for unsupported declared channels.
5. Preserve credential-posture invariants 1–4, especially same-world-generation gateway/handoff binding, no ambient credentials, CompatibilityCopyBridge non-satisfaction, and UAA receiving endpoint/session contract rather than raw secret FD/payload.
6. Preserve allowed side-effect-channel literal order and the rule that absent channels are disabled.
7. Preserve LaunchTimeSecretHandoffV1's exact status boundary: GatewayAuthBundleV1 secure-FD carrier mechanics are a positive landed primitive; the durable orchestration record/joined adoption remains incomplete. Evidence must not be converted into full gate satisfaction, and the existing carrier must not be described as absent or reimplemented.
8. Preserve all four remaining adoption steps, the no-cosmetic-reimplementation requirement, delivery/state enum order, transition text, terminal/reuse rule, and numbered handoff rules 1–11.
9. Preserve every secret-related exclusion and fail-closed requirement: no persistence in records/config/overlays/manifests/traces/logs; opaque non-path/non-digest source ref; no copied secret host homes/config/auth; all four SecureFd booleans true; exact receiver and no UAA-child inheritance; gateway owns application/forwarding; no payload/path/reusable hash/fingerprint in evidence; mismatch/expiry/duplicate/inheritance risk fail closed; copied compatibility cannot satisfy secure gates; retry closes/clears and creates a new handoff.
10. Root legacy anchors `8-worldruntimeadapterexecutionenvelopev1` and `9-launchtimesecrethandoffv1` remain unique and link to correct canonical owners. Exactly one index and ledger row per owner. Ledger records exact source spans/hashes and complete atomic rollback for both.
11. Prior D3/D5-D10 owners/review artifacts, positive carrier evidence, Cancel outcome categories, and paths outside the five-path fence remain stable.
12. Patch claims no implementation, full adoption proof, evidence decomposition, gate satisfaction/promotion, dispatch/successor authority, D10 completion, D11 authority, or D12 cutover; no D5/D6/D9 re-extraction or formatting normalization.

Required material consequence: block only if a reachable patch-causal defect materially loses/alters/duplicates/misidentifies contract semantics or ownership; weakens carrier/adoption status, secret exclusions, fail-closed behavior, exception/non-promotion boundaries, or authority; breaks required link/anchor/provenance/atomic rollback; breaches the five-path fence; or prevents clean application/reversal. Style-only phrasing, optional prose, broader redesigns, unsupported Markdown behavior, and unchanged baseline defects are not material.
Blocking threshold: only P1/P2 findings that are reachable in supported Markdown, patch-causal, violate an invariant, and meet material consequence. Body/hash mismatch, missing/reordered contract content, false carrier/adoption status, weakened credential/secret/fail-closed constraint, broken required link/anchor, incomplete family rollback, false authority/status, or scope breach qualifies. Clearly label other observations deferred without changing verdict.
Accepted prior findings: none.
Deferred/out of scope: runtime/Rust/scripts/platform/credential implementation; proof that existing carrier actually works beyond supplied status; D3 governance; prior D5/D6/D9 owners; D11 evidence decomposition; D12 cutover; roadmap/ZIP; review-control/slices/tasks; global formatting; unrelated pre-existing issues; implementation/successor dispatch.
Project sources: only this prompt, manifest, invariants, evidence, and complete patch are available. Excluded source is unavailable and must not be inferred.

Validation evidence:
- exact five-path fence and no staged files: PASS
- `git diff --check`: PASS
- no-index checks for both owners: ordinary file-difference exit, zero diagnostics
- Markdown links/anchors: 1,861 checked, 0 failures
- root and owner anchors unique; pointers resolve; exactly one index/ledger row each
- marker bodies exact: 2783/2783 cdb7... equal; 4968/4968 0a5a... equal
- field/enum/variant/rule/literal/fence/negative/exclusion/exception and positive-carrier-versus-remaining-work preservation: PASS
- prior owners/review artifacts and outside paths stable
- patch forward apply/exact five-file match and reverse apply/clean-baseline restoration: PASS
- validation log SHA-256: 2e60e11b35165e4036f192b52d05ee0db9d2f8a359ebd9e6b8bb6227228b5d4a

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return findings first by severity. Each blocker must name exact file/hunk, supported-input reachability, violated invariant, material consequence, and patch causality. State explicitly when no qualifying findings exist. End exactly:
VERDICT: APPROVED
or
VERDICT: CHANGES REQUIRED

Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

COMPLETE CONTENT-ADDRESSED PATCH (SHA-256 c9f8d90d04defb36f48eb3d51762fdd98c31972bdea502cbe569eac1160a1544):
```diff
diff --git a/llm-last-mile/runtime-refactor/04-contracts-and-gates.md b/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
index f136cb6c2..16cab4f1a 100644
--- a/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
+++ b/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
@@ -110,151 +110,11 @@ Canonical content: [`contracts/dispatch-policy-narrowing-patch-v1.md#7-dispatchp
&#32;
 ## 8. `WorldRuntimeAdapterExecutionEnvelopeV1`
&#32;
-```rust
-struct WorldRuntimeAdapterExecutionEnvelopeV1 {
-    schema_version: u32,
-    envelope_id: String,
-    kind: AdapterExecutionEnvelopeKindV1,
-    runtime_family: String,
-    guest_entrypoint: PathBuf,
-    runtime_dependency_ref: RuntimeDependencyRefV1,
-    projected_home: Option<PathBuf>,
-    workspace_root: PathBuf,
-    world_id: String,
-    world_generation: u64,
-    retained_participant_id: Option<String>,
-    active_run_id: Option<String>,
-    policy_snapshot_ref: PolicySnapshotRefV1,
-    policy_snapshot_hash: String,
-    env_projection_ref: EnvProjectionRefV1,
-    config_projection_identity: ConfigProjectionIdentityV1,
-    config_projection_ref: ConfigProjectionRefV1,
-    in_world_gateway_ref: Option<InWorldGatewayRefV1>,
-    credential_posture: AdapterCredentialPostureV1,
-    mediation_posture: AdapterMediationPostureV1,
-    command_broker_required: bool,
-    allowed_side_effect_channels: Vec<BrokeredSideEffectChannelV1>,
-}
-
-enum AdapterMediationPostureV1 {
-    BrokerRequired,
-    CompatibilityUnproven { compatibility_mode_id: String },
-}
-
-enum AdapterCredentialPostureV1 {
-    NoCredentialsRequired,
-    SecureGatewayHandoff { secret_handoff_ref: SecretHandoffRefV1 },
-    CompatibilityCopyBridge { compatibility_mode_id: String },
-}
-```
-
-Envelope kinds are `HostOrchestrator` and `WorldMember`. A `WorldMember` envelope requires exact world binding, guest-realizable entrypoint, immutable policy snapshot, explicit credential posture, and `command_broker_required=true` for side-effect-capable UAA runtimes.
-
-During D1-before-D2 staging, `CompatibilityUnproven` may preserve explicitly named and logged existing behavior, but it cannot claim Substrate policy mediation, cannot satisfy UAA caging/broker gates, and cannot promote this seam. `BrokerRequired` requires `command_broker_required=true` and fails closed when any declared side-effect channel lacks broker support.
-
-Credential-posture invariants:
-
-1. `SecureGatewayHandoff` requires an exact `in_world_gateway_ref` in the same world generation and a valid `LaunchTimeSecretHandoffV1` ref.
-2. `NoCredentialsRequired` requires no secret-handoff ref and cannot later discover ambient host credentials.
-3. `CompatibilityCopyBridge` requires a named/logged compatibility mode and cannot satisfy credential, projection, or UAA contract-promotion gates.
-4. The UAA child receives the gateway endpoint/session contract, never the raw secret FD or host credential payload.
-
-Allowed channel values describe broker support, not permission to bypass:
-
-```text
-ShellCommand
-PatchOrApplyEdit
-DirectFileWrite
-McpOrToolCall
-ProcessSpawn
-NetworkOperation
-ProviderNativeSideEffect
-```
-
-Any side-effect channel absent from the envelope is disabled in world scope.
+Canonical content: [`contracts/world-runtime-adapter-execution-envelope-v1.md#8-worldruntimeadapterexecutionenvelopev1`](contracts/world-runtime-adapter-execution-envelope-v1.md#8-worldruntimeadapterexecutionenvelopev1).
&#32;
 ## 9. `LaunchTimeSecretHandoffV1`
&#32;
-### Existing carrier versus remaining adoption work
-
-The current repo already implements the secure carrier mechanics for the managed in-world gateway: `GatewayAuthBundleV1`, an inherited pipe prepared by `world-service`, the pointer environment variable `SUBSTRATE_LLM_AUTH_BUNDLE_FD`, raw-secret env scrubbing, and gateway-side one-time read plus validation. Focused launcher and consumer integration tests make this a positive landed primitive that later slices must reuse and preserve.
-
-`LaunchTimeSecretHandoffV1` adds the orchestration-facing identity, lifecycle, and non-secret evidence needed to join that carrier to an exact world generation, envelope, retained participant, and gateway receiver. The absence of this complete durable record does not mean the FD carrier itself is absent.
-
-Remaining adoption work is to:
-
-1. expose or persist the non-secret handoff reference/state required by the envelope without persisting secret payloads;
-2. point direct world Codex/UAA provider traffic at the exact managed gateway that consumed the handoff;
-3. construct per-worker runtime-native config from Substrate logical config plus accepted policy instead of copied host config/auth; and
-4. prove the complete joined path with production-path smoke/e2e.
-
-Do not replace the existing carrier merely to make its implementation names resemble this control-plane contract. Extend or adapt it only where one of the identity, evidence, fail-closed, or adoption requirements is genuinely missing.
-
-```rust
-struct LaunchTimeSecretHandoffV1 {
-    schema_version: u32,                 // exactly 1
-    handoff_id: String,
-    orchestration_session_id: String,
-    world_id: String,
-    world_generation: u64,
-    retained_participant_id: Option<String>,
-    runtime_family: String,
-
-    // Non-secret authority references only.
-    credential_source_ref: CredentialSourceRefV1,
-    receiving_gateway_ref: InWorldGatewayRefV1,
-    delivery: SecretDeliveryMechanismV1,
-
-    created_at: Timestamp,
-    delivered_at: Option<Timestamp>,
-    consumed_at: Option<Timestamp>,
-    expires_at: Timestamp,
-    state_revision: u64,
-    state: SecretHandoffStateV1,
-    failure_diagnostic_ref: Option<RedactedDiagnosticRefV1>,
-}
-
-enum SecretDeliveryMechanismV1 {
-    SecureFd {
-        fd_name: String,
-        one_time: bool,
-        gateway_receiver_only: bool,
-        deny_child_inheritance: bool,
-        close_after_consume: bool,
-    },
-}
-
-enum SecretHandoffStateV1 {
-    Prepared,
-    Delivered,
-    Consumed,
-    Failed,
-    Expired,
-}
-```
-
-Allowed transitions:
-
-```text
-Prepared -> Delivered -> Consumed
-Prepared|Delivered -> Failed|Expired
-```
-
-`Consumed`, `Failed`, and `Expired` are terminal. Reuse requires a new `handoff_id` and new descriptor.
-
-Launch-time secret handoff rules:
-
-1. Secret material is resolved by host credential authority and must not be persisted in Substrate records, runtime-native config, workspace overlays, manifests, traces, or logs.
-2. `credential_source_ref` is an opaque host-authority reference, not a host filesystem path, credential-store locator exposed to the world, or digest of the secret payload. `fd_name` is a non-secret logical descriptor label.
-3. Contract-correct world execution must not copy host credential files or secret-bearing host config into world-visible `CODEX_HOME`, `.codex`, `config.toml`, auth files, or equivalent runtime homes.
-4. A bounded non-secret runtime config may be rendered from Substrate-owned logical inventory. Copying a host `config.toml` as authority is compatibility bridging, not projection authority.
-5. V1 validation accepts `SecureFd` only when `one_time`, `gateway_receiver_only`, `deny_child_inheritance`, and `close_after_consume` are all `true`.
-6. The secure FD is scoped to the exact `receiving_gateway_ref`, consumed by the in-world Substrate gateway at world launch, closed after consumption, and never inherited by the UAA adapter or its children.
-7. The gateway—not Codex/UAA—owns credential application, gateway session material, and upstream provider forwarding. The UAA talks to the gateway through the envelope's endpoint/session contract.
-8. Logs, receipts, traces, and manifests may contain handoff ID, non-secret refs, state, timestamps, and redacted diagnostics. They must not contain secret payloads, secret-bearing file paths, or reusable hashes/fingerprints derived from the secret payload.
-9. Failure, expiry, receiver mismatch, world-generation mismatch, duplicate consumption, or descriptor inheritance risk fails closed for credential-requiring world adapters.
-10. A compatibility copied-credential mode is temporary, explicitly named and logged, has retirement criteria, and cannot satisfy `ContractCorrectAndProven` or any secure-handoff acceptance gate.
-11. Failed/expired handoffs close the descriptor and clear transient buffers before retry; retry creates a new handoff rather than reopening or replaying the old payload.
+Canonical content: [`contracts/launch-time-secret-handoff-v1.md#9-launchtimesecrethandoffv1`](contracts/launch-time-secret-handoff-v1.md#9-launchtimesecrethandoffv1).
&#32;
 ## 10. Cancel outcome categories
&#32;
diff --git a/llm-last-mile/runtime-refactor/index/README.md b/llm-last-mile/runtime-refactor/index/README.md
index a41a479ed..a558fc56c 100644
--- a/llm-last-mile/runtime-refactor/index/README.md
+++ b/llm-last-mile/runtime-refactor/index/README.md
@@ -23,6 +23,8 @@
 | `ActiveRetainedTurnReceiptV1` | contract | [`contracts/active-retained-turn-receipt-v1.md`](../contracts/active-retained-turn-receipt-v1.md) | `canonical extracted contract owner for the complete schema, state transitions, and rules 1–3 covering the single active cancelable turn, Parked continuity, and the host/worker posture boundary` | Supersedes only root-canonical ownership of the extracted `ActiveRetainedTurnReceiptV1` span; `Receipt acceptance source` and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#4-activeretainedturnreceiptv1) |
 | `RetainedWorkerManifestV1` | contract | [`contracts/retained-worker-manifest-v1.md`](../contracts/retained-worker-manifest-v1.md) | `canonical extracted contract owner for the complete schema and rules 1–5 covering worker-cap narrowing, parked identity continuity, world-generation invalidation, and the immutable B1 accepted-record boundary` | Supersedes only root-canonical ownership of the extracted `RetainedWorkerManifestV1` span; `Receipt acceptance source`, `DispatchPolicyNarrowingPatchV1`, and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#6-retainedworkermanifestv1) |
 | `DispatchPolicyNarrowingPatchV1` | contract | [`contracts/dispatch-policy-narrowing-patch-v1.md`](../contracts/dispatch-policy-narrowing-patch-v1.md) | `canonical extracted contract owner for the complete schema, `DispatchCapabilitySubjectV1` enum, and the V1 `RestrictedPolicyPatchV1` world_fs-only rejection rule` | Supersedes only root-canonical ownership of the extracted `DispatchPolicyNarrowingPatchV1` span; `RetainedWorkerManifestV1` and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#7-dispatchpolicynarrowingpatchv1) |
+| `WorldRuntimeAdapterExecutionEnvelopeV1` | contract | [`contracts/world-runtime-adapter-execution-envelope-v1.md`](../contracts/world-runtime-adapter-execution-envelope-v1.md) | `canonical extracted contract owner for the complete schema, posture enums, world-binding and immutable-policy requirements, D1-before-D2 compatibility boundary, credential-posture invariants 1–4, and allowed side-effect-channel literals` | Supersedes only root-canonical ownership of the extracted `WorldRuntimeAdapterExecutionEnvelopeV1` span; `RetainedWorkerManifestV1`, `DispatchPolicyNarrowingPatchV1`, `LaunchTimeSecretHandoffV1`, and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#8-worldruntimeadapterexecutionenvelopev1) |
+| `LaunchTimeSecretHandoffV1` | contract | [`contracts/launch-time-secret-handoff-v1.md`](../contracts/launch-time-secret-handoff-v1.md) | `canonical extracted contract owner for the existing-carrier-versus-remaining-adoption boundary, complete schema, delivery/state enums, transitions, terminal/reuse rules, and handoff rules 1–11` | Supersedes only root-canonical ownership of the extracted `LaunchTimeSecretHandoffV1` span; `WorldRuntimeAdapterExecutionEnvelopeV1`, `Cancel outcome categories`, and neighboring D10 owners remain unchanged. | [`04`](../04-contracts-and-gates.md#9-launchtimesecrethandoffv1) |
 | `shared-target-architecture` | architecture index | [`architecture/README.md`](../architecture/README.md) | `non-authoritative navigation for the extracted executive target decision, authority map, numbered invariants, and stable review question` | Supersedes only root-canonical ownership of the extracted shared architecture spans; packet-family-local D5/D6 forwarders remain at their existing owners. | [`01`](../01-target-architecture.md#executive-decision), [`01`](../01-target-architecture.md#authority-map), [`01`](../01-target-architecture.md#non-negotiable-invariants), [`01`](../01-target-architecture.md#review-question) |
 | `shared-seam-crosswalk` | seam index | [`seams/README.md`](../seams/README.md) | `canonical shared seam-crosswalk rules plus extracted seam-family navigation for host/session, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation` | Supersedes only the extracted root reading-rule/classification spans and the extracted host/session authority, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation family rows; A0 and the existing D6 `HostSessionAuthority`, `WorldWorkerMessagingProtocol`, and `ObligationLedger` compatibility rows remain at their current owners. | [`02`](../02-seam-crosswalk.md#reading-rule), [`02`](../02-seam-crosswalk.md#a-host-authority-and-ingress), [`02`](../02-seam-crosswalk.md#b-dispatch-policy-receipts-and-retained-runtime), [`02`](../02-seam-crosswalk.md#c-obligations-and-host-re-engagement), [`02`](../02-seam-crosswalk.md#d-uaa-realization-projection-and-side-effect-mediation), [`02`](../02-seam-crosswalk.md#classification-consequences) |
 | `shared-slice-map` | slice index | [`slices/README.md`](../slices/README.md) | `canonical shared sequencing/dependency and slice-closeout owner plus non-authoritative D9 Track A–E navigation and A1/A1.4 projection index` | Supersedes only root-canonical ownership of the extracted shared sequencing/closeout spans plus the extracted Track A–E and A1/A1.4 projection spans; controlling schedule authority remains with decisions, packets, gates, and `current.md`. | [`03`](../03-phase-slice-map.md#sequencing-rules), [`03`](../03-phase-slice-map.md#track-a--authority-and-surface-neutrality), [`03`](../03-phase-slice-map.md#a1-bounded-packet-decomposition), [`03`](../03-phase-slice-map.md#track-b--world-dispatch-receipts-supervision-and-cancel), [`03`](../03-phase-slice-map.md#track-c--obligations-inbox-auto-attach-and-router-attach), [`03`](../03-phase-slice-map.md#track-d--uaa-execution-envelope-and-side-effect-mediation), [`03`](../03-phase-slice-map.md#track-e--dispatch-scoped-policy-narrowing-and-config-projection), [`03`](../03-phase-slice-map.md#slice-closeout-minimum) |
diff --git a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
index f5566e637..5a7daaf28 100644
--- a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
+++ b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
@@ -190,3 +190,5 @@ This ledger records content-preserving authority transfers while retaining requi
 | D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 4. `ActiveRetainedTurnReceiptV1` | [`4-activeretainedturnreceiptv1`](../04-contracts-and-gates.md#4-activeretainedturnreceiptv1) | lines 139–183 | `2cc996111d82d296f89bdd88f80df5c0b559bdb305aec546881b749ace93f67e` | [`../contracts/active-retained-turn-receipt-v1.md`](../contracts/active-retained-turn-receipt-v1.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1557-byte source span at `04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1` and the exact 1443-byte source span at `04-contracts-and-gates.md#4-activeretainedturnreceiptv1`; remove `contracts/active-ephemeral-task-receipt-v1.md` and `contracts/active-retained-turn-receipt-v1.md`; remove the exact `ActiveEphemeralTaskReceiptV1` and `ActiveRetainedTurnReceiptV1` rows from `index/README.md`; remove both D10 ledger entries for this batch; leave `HostExecutionEpisodeV1`, deferred retained-spawn admission recovery, `Receipt acceptance source`, all D3/D5–D9 owners, `index/current.md`, `review-control/`, remaining D10 units, and every path outside the five-path fence unchanged |
 | D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 6. `RetainedWorkerManifestV1` | [`6-retainedworkermanifestv1`](../04-contracts-and-gates.md#6-retainedworkermanifestv1) | lines 103–140 (1576 bytes) | `cd25ec1c19548b8442e01f753517c117cab9a350cb828de8214a351bdc09ee07` | [`../contracts/retained-worker-manifest-v1.md`](../contracts/retained-worker-manifest-v1.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1576-byte source span at `04-contracts-and-gates.md#6-retainedworkermanifestv1` and the exact 810-byte source span at `04-contracts-and-gates.md#7-dispatchpolicynarrowingpatchv1`; remove `contracts/retained-worker-manifest-v1.md` and `contracts/dispatch-policy-narrowing-patch-v1.md`; remove the exact `RetainedWorkerManifestV1` and `DispatchPolicyNarrowingPatchV1` rows from `index/README.md`; remove both D10 ledger entries for this batch; leave `Receipt acceptance source`, `WorldRuntimeAdapterExecutionEnvelopeV1`, all D3/D5–D10 owners, `index/current.md`, `review-control/`, remaining later D10 units, and every path outside the five-path fence unchanged |
 | D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 7. `DispatchPolicyNarrowingPatchV1` | [`7-dispatchpolicynarrowingpatchv1`](../04-contracts-and-gates.md#7-dispatchpolicynarrowingpatchv1) | lines 141–167 (810 bytes) | `c0fd22a7c630c30e8b6fca976645d84dc7515b435af759c4ecc0bc8b8871da4b` | [`../contracts/dispatch-policy-narrowing-patch-v1.md`](../contracts/dispatch-policy-narrowing-patch-v1.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1576-byte source span at `04-contracts-and-gates.md#6-retainedworkermanifestv1` and the exact 810-byte source span at `04-contracts-and-gates.md#7-dispatchpolicynarrowingpatchv1`; remove `contracts/retained-worker-manifest-v1.md` and `contracts/dispatch-policy-narrowing-patch-v1.md`; remove the exact `RetainedWorkerManifestV1` and `DispatchPolicyNarrowingPatchV1` rows from `index/README.md`; remove both D10 ledger entries for this batch; leave `Receipt acceptance source`, `WorldRuntimeAdapterExecutionEnvelopeV1`, all D3/D5–D10 owners, `index/current.md`, `review-control/`, remaining later D10 units, and every path outside the five-path fence unchanged |
+| D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 8. `WorldRuntimeAdapterExecutionEnvelopeV1` | [`8-worldruntimeadapterexecutionenvelopev1`](../04-contracts-and-gates.md#8-worldruntimeadapterexecutionenvelopev1) | lines 111–175 (2783 bytes) | `cdb755ac00cea99be0327f87547fd586f1c6335c470113cea03307cc8e3d8eac` | [`../contracts/world-runtime-adapter-execution-envelope-v1.md`](../contracts/world-runtime-adapter-execution-envelope-v1.md) | contract | canonical destination; source compatibility anchor | paired family atomicity with `LaunchTimeSecretHandoffV1`; preserve the landed `GatewayAuthBundleV1` secure-FD carrier fact without promoting joined adoption | restore the exact 2783-byte source span at `04-contracts-and-gates.md#8-worldruntimeadapterexecutionenvelopev1` and the exact 4968-byte source span at `04-contracts-and-gates.md#9-launchtimesecrethandoffv1`; remove `contracts/world-runtime-adapter-execution-envelope-v1.md` and `contracts/launch-time-secret-handoff-v1.md`; remove the exact `WorldRuntimeAdapterExecutionEnvelopeV1` and `LaunchTimeSecretHandoffV1` rows from `index/README.md`; remove both D10 ledger entries for this batch; leave `RetainedWorkerManifestV1`, `DispatchPolicyNarrowingPatchV1`, `Cancel outcome categories`, all D3/D5–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
+| D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 9. `LaunchTimeSecretHandoffV1` | [`9-launchtimesecrethandoffv1`](../04-contracts-and-gates.md#9-launchtimesecrethandoffv1) | lines 176–258 (4968 bytes) | `0a5a13799969f165c3acca200dfd5f28bf2e74ccbfeb71356176caecec78b0cd` | [`../contracts/launch-time-secret-handoff-v1.md`](../contracts/launch-time-secret-handoff-v1.md) | contract | canonical destination; source compatibility anchor | paired family atomicity with `WorldRuntimeAdapterExecutionEnvelopeV1`; preserve existing secure-carrier evidence and the remaining-adoption boundary without reimplementation claims | restore the exact 2783-byte source span at `04-contracts-and-gates.md#8-worldruntimeadapterexecutionenvelopev1` and the exact 4968-byte source span at `04-contracts-and-gates.md#9-launchtimesecrethandoffv1`; remove `contracts/world-runtime-adapter-execution-envelope-v1.md` and `contracts/launch-time-secret-handoff-v1.md`; remove the exact `WorldRuntimeAdapterExecutionEnvelopeV1` and `LaunchTimeSecretHandoffV1` rows from `index/README.md`; remove both D10 ledger entries for this batch; leave `RetainedWorkerManifestV1`, `DispatchPolicyNarrowingPatchV1`, `Cancel outcome categories`, all D3/D5–D10 owners, `index/current.md`, `review-control/`, and every path outside the five-path fence unchanged |
diff --git a/llm-last-mile/runtime-refactor/contracts/world-runtime-adapter-execution-envelope-v1.md b/llm-last-mile/runtime-refactor/contracts/world-runtime-adapter-execution-envelope-v1.md
new file mode 100644
index 000000000..c071f6d71
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/contracts/world-runtime-adapter-execution-envelope-v1.md
@@ -0,0 +1,73 @@
+**Kind:** contract
+**Status:** canonical
+**Canonical for:** complete extracted `WorldRuntimeAdapterExecutionEnvelopeV1` schema, mediation and credential posture enums, world-binding and immutable-policy requirements, D1-before-D2 compatibility boundary, credential-posture invariants 1–4, and allowed side-effect-channel literals
+**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#8-worldruntimeadapterexecutionenvelopev1`](../04-contracts-and-gates.md#8-worldruntimeadapterexecutionenvelopev1), baseline lines 111–175; the exact 2783-byte source body is preserved between the boundary markers below
+**Baseline span SHA-256:** `cdb755ac00cea99be0327f87547fd586f1c6335c470113cea03307cc8e3d8eac`
+
+<!-- exact-extracted-body:start -->
+## 8. `WorldRuntimeAdapterExecutionEnvelopeV1`
+
+```rust
+struct WorldRuntimeAdapterExecutionEnvelopeV1 {
+    schema_version: u32,
+    envelope_id: String,
+    kind: AdapterExecutionEnvelopeKindV1,
+    runtime_family: String,
+    guest_entrypoint: PathBuf,
+    runtime_dependency_ref: RuntimeDependencyRefV1,
+    projected_home: Option<PathBuf>,
+    workspace_root: PathBuf,
+    world_id: String,
+    world_generation: u64,
+    retained_participant_id: Option<String>,
+    active_run_id: Option<String>,
+    policy_snapshot_ref: PolicySnapshotRefV1,
+    policy_snapshot_hash: String,
+    env_projection_ref: EnvProjectionRefV1,
+    config_projection_identity: ConfigProjectionIdentityV1,
+    config_projection_ref: ConfigProjectionRefV1,
+    in_world_gateway_ref: Option<InWorldGatewayRefV1>,
+    credential_posture: AdapterCredentialPostureV1,
+    mediation_posture: AdapterMediationPostureV1,
+    command_broker_required: bool,
+    allowed_side_effect_channels: Vec<BrokeredSideEffectChannelV1>,
+}
+
+enum AdapterMediationPostureV1 {
+    BrokerRequired,
+    CompatibilityUnproven { compatibility_mode_id: String },
+}
+
+enum AdapterCredentialPostureV1 {
+    NoCredentialsRequired,
+    SecureGatewayHandoff { secret_handoff_ref: SecretHandoffRefV1 },
+    CompatibilityCopyBridge { compatibility_mode_id: String },
+}
+```
+
+Envelope kinds are `HostOrchestrator` and `WorldMember`. A `WorldMember` envelope requires exact world binding, guest-realizable entrypoint, immutable policy snapshot, explicit credential posture, and `command_broker_required=true` for side-effect-capable UAA runtimes.
+
+During D1-before-D2 staging, `CompatibilityUnproven` may preserve explicitly named and logged existing behavior, but it cannot claim Substrate policy mediation, cannot satisfy UAA caging/broker gates, and cannot promote this seam. `BrokerRequired` requires `command_broker_required=true` and fails closed when any declared side-effect channel lacks broker support.
+
+Credential-posture invariants:
+
+1. `SecureGatewayHandoff` requires an exact `in_world_gateway_ref` in the same world generation and a valid `LaunchTimeSecretHandoffV1` ref.
+2. `NoCredentialsRequired` requires no secret-handoff ref and cannot later discover ambient host credentials.
+3. `CompatibilityCopyBridge` requires a named/logged compatibility mode and cannot satisfy credential, projection, or UAA contract-promotion gates.
+4. The UAA child receives the gateway endpoint/session contract, never the raw secret FD or host credential payload.
+
+Allowed channel values describe broker support, not permission to bypass:
+
+```text
+ShellCommand
+PatchOrApplyEdit
+DirectFileWrite
+McpOrToolCall
+ProcessSpawn
+NetworkOperation
+ProviderNativeSideEffect
+```
+
+Any side-effect channel absent from the envelope is disabled in world scope.
+
+<!-- exact-extracted-body:end -->
diff --git a/llm-last-mile/runtime-refactor/contracts/launch-time-secret-handoff-v1.md b/llm-last-mile/runtime-refactor/contracts/launch-time-secret-handoff-v1.md
new file mode 100644
index 000000000..8a5b5ed65
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/contracts/launch-time-secret-handoff-v1.md
@@ -0,0 +1,91 @@
+**Kind:** contract
+**Status:** canonical
+**Canonical for:** complete extracted existing-carrier-versus-remaining-adoption boundary, `LaunchTimeSecretHandoffV1` schema, delivery and state enums, allowed transitions, terminal/reuse rules, and handoff rules 1–11 covering non-secret refs, fail-closed exclusions, compatibility copied-credential status, and retry semantics
+**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#9-launchtimesecrethandoffv1`](../04-contracts-and-gates.md#9-launchtimesecrethandoffv1), baseline lines 176–258; the exact 4968-byte source body is preserved between the boundary markers below
+**Baseline span SHA-256:** `0a5a13799969f165c3acca200dfd5f28bf2e74ccbfeb71356176caecec78b0cd`
+
+<!-- exact-extracted-body:start -->
+## 9. `LaunchTimeSecretHandoffV1`
+
+### Existing carrier versus remaining adoption work
+
+The current repo already implements the secure carrier mechanics for the managed in-world gateway: `GatewayAuthBundleV1`, an inherited pipe prepared by `world-service`, the pointer environment variable `SUBSTRATE_LLM_AUTH_BUNDLE_FD`, raw-secret env scrubbing, and gateway-side one-time read plus validation. Focused launcher and consumer integration tests make this a positive landed primitive that later slices must reuse and preserve.
+
+`LaunchTimeSecretHandoffV1` adds the orchestration-facing identity, lifecycle, and non-secret evidence needed to join that carrier to an exact world generation, envelope, retained participant, and gateway receiver. The absence of this complete durable record does not mean the FD carrier itself is absent.
+
+Remaining adoption work is to:
+
+1. expose or persist the non-secret handoff reference/state required by the envelope without persisting secret payloads;
+2. point direct world Codex/UAA provider traffic at the exact managed gateway that consumed the handoff;
+3. construct per-worker runtime-native config from Substrate logical config plus accepted policy instead of copied host config/auth; and
+4. prove the complete joined path with production-path smoke/e2e.
+
+Do not replace the existing carrier merely to make its implementation names resemble this control-plane contract. Extend or adapt it only where one of the identity, evidence, fail-closed, or adoption requirements is genuinely missing.
+
+```rust
+struct LaunchTimeSecretHandoffV1 {
+    schema_version: u32,                 // exactly 1
+    handoff_id: String,
+    orchestration_session_id: String,
+    world_id: String,
+    world_generation: u64,
+    retained_participant_id: Option<String>,
+    runtime_family: String,
+
+    // Non-secret authority references only.
+    credential_source_ref: CredentialSourceRefV1,
+    receiving_gateway_ref: InWorldGatewayRefV1,
+    delivery: SecretDeliveryMechanismV1,
+
+    created_at: Timestamp,
+    delivered_at: Option<Timestamp>,
+    consumed_at: Option<Timestamp>,
+    expires_at: Timestamp,
+    state_revision: u64,
+    state: SecretHandoffStateV1,
+    failure_diagnostic_ref: Option<RedactedDiagnosticRefV1>,
+}
+
+enum SecretDeliveryMechanismV1 {
+    SecureFd {
+        fd_name: String,
+        one_time: bool,
+        gateway_receiver_only: bool,
+        deny_child_inheritance: bool,
+        close_after_consume: bool,
+    },
+}
+
+enum SecretHandoffStateV1 {
+    Prepared,
+    Delivered,
+    Consumed,
+    Failed,
+    Expired,
+}
+```
+
+Allowed transitions:
+
+```text
+Prepared -> Delivered -> Consumed
+Prepared|Delivered -> Failed|Expired
+```
+
+`Consumed`, `Failed`, and `Expired` are terminal. Reuse requires a new `handoff_id` and new descriptor.
+
+Launch-time secret handoff rules:
+
+1. Secret material is resolved by host credential authority and must not be persisted in Substrate records, runtime-native config, workspace overlays, manifests, traces, or logs.
+2. `credential_source_ref` is an opaque host-authority reference, not a host filesystem path, credential-store locator exposed to the world, or digest of the secret payload. `fd_name` is a non-secret logical descriptor label.
+3. Contract-correct world execution must not copy host credential files or secret-bearing host config into world-visible `CODEX_HOME`, `.codex`, `config.toml`, auth files, or equivalent runtime homes.
+4. A bounded non-secret runtime config may be rendered from Substrate-owned logical inventory. Copying a host `config.toml` as authority is compatibility bridging, not projection authority.
+5. V1 validation accepts `SecureFd` only when `one_time`, `gateway_receiver_only`, `deny_child_inheritance`, and `close_after_consume` are all `true`.
+6. The secure FD is scoped to the exact `receiving_gateway_ref`, consumed by the in-world Substrate gateway at world launch, closed after consumption, and never inherited by the UAA adapter or its children.
+7. The gateway—not Codex/UAA—owns credential application, gateway session material, and upstream provider forwarding. The UAA talks to the gateway through the envelope's endpoint/session contract.
+8. Logs, receipts, traces, and manifests may contain handoff ID, non-secret refs, state, timestamps, and redacted diagnostics. They must not contain secret payloads, secret-bearing file paths, or reusable hashes/fingerprints derived from the secret payload.
+9. Failure, expiry, receiver mismatch, world-generation mismatch, duplicate consumption, or descriptor inheritance risk fails closed for credential-requiring world adapters.
+10. A compatibility copied-credential mode is temporary, explicitly named and logged, has retirement criteria, and cannot satisfy `ContractCorrectAndProven` or any secure-handoff acceptance gate.
+11. Failed/expired handoffs close the descriptor and clear transient buffers before retry; retry creates a new handoff rather than reopening or replaying the old payload.
+
+<!-- exact-extracted-body:end -->
```
``````

</details>

<details>
<summary>Initial-range review prompt exact bytes (base64 UTF-8)</summary>

```text
UmV2aWV3IG1vZGU6IGluaXRpYWwtcmFuZ2UKUmV2aWV3IGNvbnRleHQ6IGluZGVwZW5kZW50IGZy
ZXNoIGNvbnZlcnNhdGlvbgpSZXZpZXcgYm91bmRhcnk6IGV4YWN0IGJhc2VsaW5lIGNvbW1pdCBl
ZmE0M2IyNTVmZWViODg0N2QyMTMxYjc2ZjU0Yzg1YTgyMzRmYzhhLCB0cmVlIGM4Y2UxZTAyNWQ0
MTZhMDM0MGNjOWM1YTllYjhhZTRkYmYxZjZlMWEsIHBsdXMgdGhlIGNvbXBsZXRlIGNvbnRlbnQt
YWRkcmVzc2VkIGNhbmRpZGF0ZSBwYXRjaCBiZWxvdy4gQ2FuZGlkYXRlIHBhdGNoIFNIQS0yNTY6
IGM5ZjhkOTBkMDRkZWZiMzZmNDhlYjNkNTE3NjJmZGQ5OGMzMTk3MmJkZWE1MDJjYmU1NjllYWMx
MTYwYTE1NDQuClJldmlldyB0YXJnZXQ6IGNvbXBsZXRlIGJvdW5kZWQgRDEwIGRvY3VtZW50YXRp
b24gZXh0cmFjdGlvbiBmb3IgdGhlIGluc2VwYXJhYmxlIFdvcmxkUnVudGltZUFkYXB0ZXJFeGVj
dXRpb25FbnZlbG9wZVYxICsgTGF1bmNoVGltZVNlY3JldEhhbmRvZmZWMSBmYW1pbHkuIFJldmll
dyBldmVyeSBodW5rIGFuZCBib3RoIG5ldyBmaWxlcy4gRG8gbm90IGluZmVyIHVucHJvdmlkZWQg
c291cmNlLgoKTWFuaWZlc3QgKGV4YWN0bHkgZml2ZSBwYXRocyBpbmNsdWRpbmcgdW50cmFja2Vk
IG5ldyBmaWxlcyk6Ci0gbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yLzA0LWNvbnRyYWN0
cy1hbmQtZ2F0ZXMubWQg4oCUIFNIQS0yNTYgYjE5YWJlOTAxYWU1NGE2ODFiMWFlOTVkMjU2OTdk
NDNlNDdiZmE2MjFhNWM0YWY5MTZmZTQ2MjE4YTdlZDk3ZQotIGxsbS1sYXN0LW1pbGUvcnVudGlt
ZS1yZWZhY3Rvci9jb250cmFjdHMvd29ybGQtcnVudGltZS1hZGFwdGVyLWV4ZWN1dGlvbi1lbnZl
bG9wZS12MS5tZCDigJQgbmV3IOKAlCBTSEEtMjU2IGU5YmY0MGI3MjI0MTE1MWM1MTkxYTAzODMx
OGQ5NGMxNGQ3NzhhZGQwZWJjYmRlYmQ4OWQyMzU2ZjU5ZDRjMGYKLSBsbG0tbGFzdC1taWxlL3J1
bnRpbWUtcmVmYWN0b3IvY29udHJhY3RzL2xhdW5jaC10aW1lLXNlY3JldC1oYW5kb2ZmLXYxLm1k
IOKAlCBuZXcg4oCUIFNIQS0yNTYgYzc0ODBlNjU5YmQyYTQxYTRiN2IzM2M1MGI0NzNkNDQ3NmIy
OWRlMDA1NTNlZDcxMGNiOGY0MDZkMWVlMDk1NAotIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZh
Y3Rvci9pbmRleC9SRUFETUUubWQg4oCUIFNIQS0yNTYgMGJkMmQ1ZWE0NGQxZDIxYzk5OTdhYzZj
MGVjYjllZTdkZWYyMjQ1ZGU5YjJkMzJmN2I4NDQ3NGRmMjNlZDdlYQotIGxsbS1sYXN0LW1pbGUv
cnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQg4oCUIFNIQS0y
NTYgMTYzMGVhZTQwZTUwNjgyZjdkOGViYTYxMmUyZTFiNjY2ZjM1ZTRiYmI5MTE2Nzg2YmYyNzgz
YzM3YmNkZGM2MwoKU3VwcG9ydGVkIGlucHV0cyBhbmQgYmVoYXZpb3I6IEdpdEh1Yi1mbGF2b3Jl
ZCBNYXJrZG93biB3aXRoIEFUWCBoZWFkaW5ncy9nZW5lcmF0ZWQgYW5jaG9ycywgcmVsYXRpdmUg
bGlua3MvZnJhZ21lbnRzLCBmZW5jZWQgUnVzdC90ZXh0IGJsb2Nrcywgb3JkZXJlZCBsaXN0cywg
TWFya2Rvd24gdGFibGVzIHdpdGggc2VtYW50aWMgcm93L2NlbGwgb3JkZXIsIGlubGluZSBjb2Rl
L2xpdGVyYWxzLCBhbmQgSFRNTCBib2R5LWhhc2ggbWFya2Vycy4gTGVnYWN5IHJvb3QgcmVtYWlu
cyBhIGNvbXBhdGliaWxpdHkgcHJvamVjdGlvbiB1bnRpbCBzZXBhcmF0ZWx5IGF1dGhvcml6ZWQg
RDEyLiBDYW5vbmljYWwgb3duZXJzIGFyZSBub3JtYXRpdmU7IHJvb3QgaGVhZGluZ3MvYW5jaG9y
cyByZW1haW4gc3VwcG9ydGVkLgpTdXBwb3J0ZWQgcGxhdGZvcm1zIGFuZCBkaWFsZWN0czogcmVw
b3NpdG9yeSBNYXJrZG93biB3aXRoIEdpdEh1Yi1zdHlsZSBoZWFkaW5nIHNsdWdzIGFuZCB0aGUg
cHJvamVjdCBsaW5rL2FuY2hvciBjaGVja2VyLiBSdW50aW1lL1J1c3Qvc2NyaXB0L3BsYXRmb3Jt
L3BhcnNlci9jcmVkZW50aWFsIHRyYW5zcG9ydCBiZWhhdmlvciBpcyBub3QgY2hhbmdlZCBieSB0
aGlzIHBhdGNoLgoKSW4tc2NvcGUgaW52YXJpYW50czoKMS4gRXhhY3RseSBvbmUgY2Fub25pY2Fs
IG93bmVyIGV4aXN0cyBmb3IgZWFjaCBuYW1lZCBWMSBjb250cmFjdDsgdGhlIHR3byBjb250cmFj
dHMgbGFuZCBhbmQgcm9sbCBiYWNrIHRvZ2V0aGVyIGJlY2F1c2UgU2VjdXJlR2F0ZXdheUhhbmRv
ZmYgcmVmZXJlbmNlcyBMYXVuY2hUaW1lU2VjcmV0SGFuZG9mZlYxIGFuZCBleGFjdCBnYXRld2F5
L3dvcmxkIGdlbmVyYXRpb24uCjIuIENhbm9uaWNhbCBib2RpZXMgYXJlIGJ5dGUtaWRlbnRpY2Fs
IHRvIGZyb3plbiByb290IHNwYW5zLiBXb3JsZFJ1bnRpbWVBZGFwdGVyRXhlY3V0aW9uRW52ZWxv
cGVWMSBpcyAyNzgzIGJ5dGVzLCBTSEEtMjU2IGNkYjc1NWFjMDBjZWE5OWJlMDMyN2Y4NzU0N2Zk
NTg2ZjFjNjMzNWM0NzAxMTNjZWEwMzMwN2NjOGUzZDhlYWMuIExhdW5jaFRpbWVTZWNyZXRIYW5k
b2ZmVjEgaXMgNDk2OCBieXRlcywgU0hBLTI1NiAwYTVhMTM3OTk5NjlmMTY1YzNhY2NhMjAwZGZk
NWYyOGJmMmU3NGNjYmZlYjcxMzU2MTc2Y2FlY2VjNzhiMGNkLgozLiBQcmVzZXJ2ZSBzdGFibGUg
VjEgbmFtZXMsIHNjaGVtYS9jb21tZW50IHRleHQsIGZpZWxkIG9yZGVyLCBlbnVtL3ZhcmlhbnQg
b3JkZXIsIGVudmVsb3BlIGtpbmRzLCB3b3JsZCBiaW5kaW5nLCBpbW11dGFibGUgcG9saWN5LCBh
bmQgY29tbWFuZF9icm9rZXJfcmVxdWlyZWQgcmVxdWlyZW1lbnRzLgo0LiBQcmVzZXJ2ZSB0aGUg
RDEtYmVmb3JlLUQyIENvbXBhdGliaWxpdHlVbnByb3ZlbiBleGNlcHRpb24gZXhhY3RseTogbmFt
ZWQvbG9nZ2VkIGNvbXBhdGliaWxpdHkgbWF5IGNvbnRpbnVlLCBidXQgY2Fubm90IGNsYWltIG1l
ZGlhdGlvbiwgc2F0aXNmeSBjYWdpbmcvYnJva2VyIGdhdGVzLCBvciBwcm9tb3RlOyBCcm9rZXJS
ZXF1aXJlZCBmYWlscyBjbG9zZWQgZm9yIHVuc3VwcG9ydGVkIGRlY2xhcmVkIGNoYW5uZWxzLgo1
LiBQcmVzZXJ2ZSBjcmVkZW50aWFsLXBvc3R1cmUgaW52YXJpYW50cyAx4oCTNCwgZXNwZWNpYWxs
eSBzYW1lLXdvcmxkLWdlbmVyYXRpb24gZ2F0ZXdheS9oYW5kb2ZmIGJpbmRpbmcsIG5vIGFtYmll
bnQgY3JlZGVudGlhbHMsIENvbXBhdGliaWxpdHlDb3B5QnJpZGdlIG5vbi1zYXRpc2ZhY3Rpb24s
IGFuZCBVQUEgcmVjZWl2aW5nIGVuZHBvaW50L3Nlc3Npb24gY29udHJhY3QgcmF0aGVyIHRoYW4g
cmF3IHNlY3JldCBGRC9wYXlsb2FkLgo2LiBQcmVzZXJ2ZSBhbGxvd2VkIHNpZGUtZWZmZWN0LWNo
YW5uZWwgbGl0ZXJhbCBvcmRlciBhbmQgdGhlIHJ1bGUgdGhhdCBhYnNlbnQgY2hhbm5lbHMgYXJl
IGRpc2FibGVkLgo3LiBQcmVzZXJ2ZSBMYXVuY2hUaW1lU2VjcmV0SGFuZG9mZlYxJ3MgZXhhY3Qg
c3RhdHVzIGJvdW5kYXJ5OiBHYXRld2F5QXV0aEJ1bmRsZVYxIHNlY3VyZS1GRCBjYXJyaWVyIG1l
Y2hhbmljcyBhcmUgYSBwb3NpdGl2ZSBsYW5kZWQgcHJpbWl0aXZlOyB0aGUgZHVyYWJsZSBvcmNo
ZXN0cmF0aW9uIHJlY29yZC9qb2luZWQgYWRvcHRpb24gcmVtYWlucyBpbmNvbXBsZXRlLiBFdmlk
ZW5jZSBtdXN0IG5vdCBiZSBjb252ZXJ0ZWQgaW50byBmdWxsIGdhdGUgc2F0aXNmYWN0aW9uLCBh
bmQgdGhlIGV4aXN0aW5nIGNhcnJpZXIgbXVzdCBub3QgYmUgZGVzY3JpYmVkIGFzIGFic2VudCBv
ciByZWltcGxlbWVudGVkLgo4LiBQcmVzZXJ2ZSBhbGwgZm91ciByZW1haW5pbmcgYWRvcHRpb24g
c3RlcHMsIHRoZSBuby1jb3NtZXRpYy1yZWltcGxlbWVudGF0aW9uIHJlcXVpcmVtZW50LCBkZWxp
dmVyeS9zdGF0ZSBlbnVtIG9yZGVyLCB0cmFuc2l0aW9uIHRleHQsIHRlcm1pbmFsL3JldXNlIHJ1
bGUsIGFuZCBudW1iZXJlZCBoYW5kb2ZmIHJ1bGVzIDHigJMxMS4KOS4gUHJlc2VydmUgZXZlcnkg
c2VjcmV0LXJlbGF0ZWQgZXhjbHVzaW9uIGFuZCBmYWlsLWNsb3NlZCByZXF1aXJlbWVudDogbm8g
cGVyc2lzdGVuY2UgaW4gcmVjb3Jkcy9jb25maWcvb3ZlcmxheXMvbWFuaWZlc3RzL3RyYWNlcy9s
b2dzOyBvcGFxdWUgbm9uLXBhdGgvbm9uLWRpZ2VzdCBzb3VyY2UgcmVmOyBubyBjb3BpZWQgc2Vj
cmV0IGhvc3QgaG9tZXMvY29uZmlnL2F1dGg7IGFsbCBmb3VyIFNlY3VyZUZkIGJvb2xlYW5zIHRy
dWU7IGV4YWN0IHJlY2VpdmVyIGFuZCBubyBVQUEtY2hpbGQgaW5oZXJpdGFuY2U7IGdhdGV3YXkg
b3ducyBhcHBsaWNhdGlvbi9mb3J3YXJkaW5nOyBubyBwYXlsb2FkL3BhdGgvcmV1c2FibGUgaGFz
aC9maW5nZXJwcmludCBpbiBldmlkZW5jZTsgbWlzbWF0Y2gvZXhwaXJ5L2R1cGxpY2F0ZS9pbmhl
cml0YW5jZSByaXNrIGZhaWwgY2xvc2VkOyBjb3BpZWQgY29tcGF0aWJpbGl0eSBjYW5ub3Qgc2F0
aXNmeSBzZWN1cmUgZ2F0ZXM7IHJldHJ5IGNsb3Nlcy9jbGVhcnMgYW5kIGNyZWF0ZXMgYSBuZXcg
aGFuZG9mZi4KMTAuIFJvb3QgbGVnYWN5IGFuY2hvcnMgYDgtd29ybGRydW50aW1lYWRhcHRlcmV4
ZWN1dGlvbmVudmVsb3BldjFgIGFuZCBgOS1sYXVuY2h0aW1lc2VjcmV0aGFuZG9mZnYxYCByZW1h
aW4gdW5pcXVlIGFuZCBsaW5rIHRvIGNvcnJlY3QgY2Fub25pY2FsIG93bmVycy4gRXhhY3RseSBv
bmUgaW5kZXggYW5kIGxlZGdlciByb3cgcGVyIG93bmVyLiBMZWRnZXIgcmVjb3JkcyBleGFjdCBz
b3VyY2Ugc3BhbnMvaGFzaGVzIGFuZCBjb21wbGV0ZSBhdG9taWMgcm9sbGJhY2sgZm9yIGJvdGgu
CjExLiBQcmlvciBEMy9ENS1EMTAgb3duZXJzL3JldmlldyBhcnRpZmFjdHMsIHBvc2l0aXZlIGNh
cnJpZXIgZXZpZGVuY2UsIENhbmNlbCBvdXRjb21lIGNhdGVnb3JpZXMsIGFuZCBwYXRocyBvdXRz
aWRlIHRoZSBmaXZlLXBhdGggZmVuY2UgcmVtYWluIHN0YWJsZS4KMTIuIFBhdGNoIGNsYWltcyBu
byBpbXBsZW1lbnRhdGlvbiwgZnVsbCBhZG9wdGlvbiBwcm9vZiwgZXZpZGVuY2UgZGVjb21wb3Np
dGlvbiwgZ2F0ZSBzYXRpc2ZhY3Rpb24vcHJvbW90aW9uLCBkaXNwYXRjaC9zdWNjZXNzb3IgYXV0
aG9yaXR5LCBEMTAgY29tcGxldGlvbiwgRDExIGF1dGhvcml0eSwgb3IgRDEyIGN1dG92ZXI7IG5v
IEQ1L0Q2L0Q5IHJlLWV4dHJhY3Rpb24gb3IgZm9ybWF0dGluZyBub3JtYWxpemF0aW9uLgoKUmVx
dWlyZWQgbWF0ZXJpYWwgY29uc2VxdWVuY2U6IGJsb2NrIG9ubHkgaWYgYSByZWFjaGFibGUgcGF0
Y2gtY2F1c2FsIGRlZmVjdCBtYXRlcmlhbGx5IGxvc2VzL2FsdGVycy9kdXBsaWNhdGVzL21pc2lk
ZW50aWZpZXMgY29udHJhY3Qgc2VtYW50aWNzIG9yIG93bmVyc2hpcDsgd2Vha2VucyBjYXJyaWVy
L2Fkb3B0aW9uIHN0YXR1cywgc2VjcmV0IGV4Y2x1c2lvbnMsIGZhaWwtY2xvc2VkIGJlaGF2aW9y
LCBleGNlcHRpb24vbm9uLXByb21vdGlvbiBib3VuZGFyaWVzLCBvciBhdXRob3JpdHk7IGJyZWFr
cyByZXF1aXJlZCBsaW5rL2FuY2hvci9wcm92ZW5hbmNlL2F0b21pYyByb2xsYmFjazsgYnJlYWNo
ZXMgdGhlIGZpdmUtcGF0aCBmZW5jZTsgb3IgcHJldmVudHMgY2xlYW4gYXBwbGljYXRpb24vcmV2
ZXJzYWwuIFN0eWxlLW9ubHkgcGhyYXNpbmcsIG9wdGlvbmFsIHByb3NlLCBicm9hZGVyIHJlZGVz
aWducywgdW5zdXBwb3J0ZWQgTWFya2Rvd24gYmVoYXZpb3IsIGFuZCB1bmNoYW5nZWQgYmFzZWxp
bmUgZGVmZWN0cyBhcmUgbm90IG1hdGVyaWFsLgpCbG9ja2luZyB0aHJlc2hvbGQ6IG9ubHkgUDEv
UDIgZmluZGluZ3MgdGhhdCBhcmUgcmVhY2hhYmxlIGluIHN1cHBvcnRlZCBNYXJrZG93biwgcGF0
Y2gtY2F1c2FsLCB2aW9sYXRlIGFuIGludmFyaWFudCwgYW5kIG1lZXQgbWF0ZXJpYWwgY29uc2Vx
dWVuY2UuIEJvZHkvaGFzaCBtaXNtYXRjaCwgbWlzc2luZy9yZW9yZGVyZWQgY29udHJhY3QgY29u
dGVudCwgZmFsc2UgY2Fycmllci9hZG9wdGlvbiBzdGF0dXMsIHdlYWtlbmVkIGNyZWRlbnRpYWwv
c2VjcmV0L2ZhaWwtY2xvc2VkIGNvbnN0cmFpbnQsIGJyb2tlbiByZXF1aXJlZCBsaW5rL2FuY2hv
ciwgaW5jb21wbGV0ZSBmYW1pbHkgcm9sbGJhY2ssIGZhbHNlIGF1dGhvcml0eS9zdGF0dXMsIG9y
IHNjb3BlIGJyZWFjaCBxdWFsaWZpZXMuIENsZWFybHkgbGFiZWwgb3RoZXIgb2JzZXJ2YXRpb25z
IGRlZmVycmVkIHdpdGhvdXQgY2hhbmdpbmcgdmVyZGljdC4KQWNjZXB0ZWQgcHJpb3IgZmluZGlu
Z3M6IG5vbmUuCkRlZmVycmVkL291dCBvZiBzY29wZTogcnVudGltZS9SdXN0L3NjcmlwdHMvcGxh
dGZvcm0vY3JlZGVudGlhbCBpbXBsZW1lbnRhdGlvbjsgcHJvb2YgdGhhdCBleGlzdGluZyBjYXJy
aWVyIGFjdHVhbGx5IHdvcmtzIGJleW9uZCBzdXBwbGllZCBzdGF0dXM7IEQzIGdvdmVybmFuY2U7
IHByaW9yIEQ1L0Q2L0Q5IG93bmVyczsgRDExIGV2aWRlbmNlIGRlY29tcG9zaXRpb247IEQxMiBj
dXRvdmVyOyByb2FkbWFwL1pJUDsgcmV2aWV3LWNvbnRyb2wvc2xpY2VzL3Rhc2tzOyBnbG9iYWwg
Zm9ybWF0dGluZzsgdW5yZWxhdGVkIHByZS1leGlzdGluZyBpc3N1ZXM7IGltcGxlbWVudGF0aW9u
L3N1Y2Nlc3NvciBkaXNwYXRjaC4KUHJvamVjdCBzb3VyY2VzOiBvbmx5IHRoaXMgcHJvbXB0LCBt
YW5pZmVzdCwgaW52YXJpYW50cywgZXZpZGVuY2UsIGFuZCBjb21wbGV0ZSBwYXRjaCBhcmUgYXZh
aWxhYmxlLiBFeGNsdWRlZCBzb3VyY2UgaXMgdW5hdmFpbGFibGUgYW5kIG11c3Qgbm90IGJlIGlu
ZmVycmVkLgoKVmFsaWRhdGlvbiBldmlkZW5jZToKLSBleGFjdCBmaXZlLXBhdGggZmVuY2UgYW5k
IG5vIHN0YWdlZCBmaWxlczogUEFTUwotIGBnaXQgZGlmZiAtLWNoZWNrYDogUEFTUwotIG5vLWlu
ZGV4IGNoZWNrcyBmb3IgYm90aCBvd25lcnM6IG9yZGluYXJ5IGZpbGUtZGlmZmVyZW5jZSBleGl0
LCB6ZXJvIGRpYWdub3N0aWNzCi0gTWFya2Rvd24gbGlua3MvYW5jaG9yczogMSw4NjEgY2hlY2tl
ZCwgMCBmYWlsdXJlcwotIHJvb3QgYW5kIG93bmVyIGFuY2hvcnMgdW5pcXVlOyBwb2ludGVycyBy
ZXNvbHZlOyBleGFjdGx5IG9uZSBpbmRleC9sZWRnZXIgcm93IGVhY2gKLSBtYXJrZXIgYm9kaWVz
IGV4YWN0OiAyNzgzLzI3ODMgY2RiNy4uLiBlcXVhbDsgNDk2OC80OTY4IDBhNWEuLi4gZXF1YWwK
LSBmaWVsZC9lbnVtL3ZhcmlhbnQvcnVsZS9saXRlcmFsL2ZlbmNlL25lZ2F0aXZlL2V4Y2x1c2lv
bi9leGNlcHRpb24gYW5kIHBvc2l0aXZlLWNhcnJpZXItdmVyc3VzLXJlbWFpbmluZy13b3JrIHBy
ZXNlcnZhdGlvbjogUEFTUwotIHByaW9yIG93bmVycy9yZXZpZXcgYXJ0aWZhY3RzIGFuZCBvdXRz
aWRlIHBhdGhzIHN0YWJsZQotIHBhdGNoIGZvcndhcmQgYXBwbHkvZXhhY3QgZml2ZS1maWxlIG1h
dGNoIGFuZCByZXZlcnNlIGFwcGx5L2NsZWFuLWJhc2VsaW5lIHJlc3RvcmF0aW9uOiBQQVNTCi0g
dmFsaWRhdGlvbiBsb2cgU0hBLTI1NjogMmU2MGUxMWIzNTE2NWU0MDM2ZjE5MmI1MmQwNWVlMGRi
OWQyZjhhMzU5ZWJkOWU2YjhiYjYyMjcyMjhiNWQ0YQoKRG8gbm90IHJlZHVjZSB0aGUgdGFzayBv
ciByZXBsYWNlIGl0IHdpdGggYW4gZWFzaWVyIGFsdGVybmF0aXZlLiBQcmVzZXJ2ZSB0aGUgcmVx
dWVzdGVkIHNjb3BlIGFuZCBwcm9qZWN0IGNvbnZlbnRpb25zLgoKUmV0dXJuIGZpbmRpbmdzIGZp
cnN0IGJ5IHNldmVyaXR5LiBFYWNoIGJsb2NrZXIgbXVzdCBuYW1lIGV4YWN0IGZpbGUvaHVuaywg
c3VwcG9ydGVkLWlucHV0IHJlYWNoYWJpbGl0eSwgdmlvbGF0ZWQgaW52YXJpYW50LCBtYXRlcmlh
bCBjb25zZXF1ZW5jZSwgYW5kIHBhdGNoIGNhdXNhbGl0eS4gU3RhdGUgZXhwbGljaXRseSB3aGVu
IG5vIHF1YWxpZnlpbmcgZmluZGluZ3MgZXhpc3QuIEVuZCBleGFjdGx5OgpWRVJESUNUOiBBUFBS
T1ZFRApvcgpWRVJESUNUOiBDSEFOR0VTIFJFUVVJUkVECgpBZHZpc29yeSBvbmx5OyB2ZXJpZnkg
YWdhaW5zdCBsb2NhbCBwcm9qZWN0IHRydXRoIGFuZCBhdXRob3JpdGF0aXZlIGRvY3M7IGRvIG5v
dCByZWR1Y2Ugc2NvcGUgd2l0aG91dCB1c2VyIGFwcHJvdmFsLgoKQ09NUExFVEUgQ09OVEVOVC1B
RERSRVNTRUQgUEFUQ0ggKFNIQS0yNTYgYzlmOGQ5MGQwNGRlZmIzNmY0OGViM2Q1MTc2MmZkZDk4
YzMxOTcyYmRlYTUwMmNiZTU2OWVhYzExNjBhMTU0NCk6CmBgYGRpZmYKZGlmZiAtLWdpdCBhL2xs
bS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIGIv
bGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQK
aW5kZXggZjEzNmNiNmMyLi4xNmNhYjRmMWEgMTAwNjQ0Ci0tLSBhL2xsbS1sYXN0LW1pbGUvcnVu
dGltZS1yZWZhY3Rvci8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kCisrKyBiL2xsbS1sYXN0LW1p
bGUvcnVudGltZS1yZWZhY3Rvci8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kCkBAIC0xMTAsMTUx
ICsxMTAsMTEgQEAgQ2Fub25pY2FsIGNvbnRlbnQ6IFtgY29udHJhY3RzL2Rpc3BhdGNoLXBvbGlj
eS1uYXJyb3dpbmctcGF0Y2gtdjEubWQjNy1kaXNwYXRjaHAKIAogIyMgOC4gYFdvcmxkUnVudGlt
ZUFkYXB0ZXJFeGVjdXRpb25FbnZlbG9wZVYxYAogCi1gYGBydXN0Ci1zdHJ1Y3QgV29ybGRSdW50
aW1lQWRhcHRlckV4ZWN1dGlvbkVudmVsb3BlVjEgewotICAgIHNjaGVtYV92ZXJzaW9uOiB1MzIs
Ci0gICAgZW52ZWxvcGVfaWQ6IFN0cmluZywKLSAgICBraW5kOiBBZGFwdGVyRXhlY3V0aW9uRW52
ZWxvcGVLaW5kVjEsCi0gICAgcnVudGltZV9mYW1pbHk6IFN0cmluZywKLSAgICBndWVzdF9lbnRy
eXBvaW50OiBQYXRoQnVmLAotICAgIHJ1bnRpbWVfZGVwZW5kZW5jeV9yZWY6IFJ1bnRpbWVEZXBl
bmRlbmN5UmVmVjEsCi0gICAgcHJvamVjdGVkX2hvbWU6IE9wdGlvbjxQYXRoQnVmPiwKLSAgICB3
b3Jrc3BhY2Vfcm9vdDogUGF0aEJ1ZiwKLSAgICB3b3JsZF9pZDogU3RyaW5nLAotICAgIHdvcmxk
X2dlbmVyYXRpb246IHU2NCwKLSAgICByZXRhaW5lZF9wYXJ0aWNpcGFudF9pZDogT3B0aW9uPFN0
cmluZz4sCi0gICAgYWN0aXZlX3J1bl9pZDogT3B0aW9uPFN0cmluZz4sCi0gICAgcG9saWN5X3Nu
YXBzaG90X3JlZjogUG9saWN5U25hcHNob3RSZWZWMSwKLSAgICBwb2xpY3lfc25hcHNob3RfaGFz
aDogU3RyaW5nLAotICAgIGVudl9wcm9qZWN0aW9uX3JlZjogRW52UHJvamVjdGlvblJlZlYxLAot
ICAgIGNvbmZpZ19wcm9qZWN0aW9uX2lkZW50aXR5OiBDb25maWdQcm9qZWN0aW9uSWRlbnRpdHlW
MSwKLSAgICBjb25maWdfcHJvamVjdGlvbl9yZWY6IENvbmZpZ1Byb2plY3Rpb25SZWZWMSwKLSAg
ICBpbl93b3JsZF9nYXRld2F5X3JlZjogT3B0aW9uPEluV29ybGRHYXRld2F5UmVmVjE+LAotICAg
IGNyZWRlbnRpYWxfcG9zdHVyZTogQWRhcHRlckNyZWRlbnRpYWxQb3N0dXJlVjEsCi0gICAgbWVk
aWF0aW9uX3Bvc3R1cmU6IEFkYXB0ZXJNZWRpYXRpb25Qb3N0dXJlVjEsCi0gICAgY29tbWFuZF9i
cm9rZXJfcmVxdWlyZWQ6IGJvb2wsCi0gICAgYWxsb3dlZF9zaWRlX2VmZmVjdF9jaGFubmVsczog
VmVjPEJyb2tlcmVkU2lkZUVmZmVjdENoYW5uZWxWMT4sCi19Ci0KLWVudW0gQWRhcHRlck1lZGlh
dGlvblBvc3R1cmVWMSB7Ci0gICAgQnJva2VyUmVxdWlyZWQsCi0gICAgQ29tcGF0aWJpbGl0eVVu
cHJvdmVuIHsgY29tcGF0aWJpbGl0eV9tb2RlX2lkOiBTdHJpbmcgfSwKLX0KLQotZW51bSBBZGFw
dGVyQ3JlZGVudGlhbFBvc3R1cmVWMSB7Ci0gICAgTm9DcmVkZW50aWFsc1JlcXVpcmVkLAotICAg
IFNlY3VyZUdhdGV3YXlIYW5kb2ZmIHsgc2VjcmV0X2hhbmRvZmZfcmVmOiBTZWNyZXRIYW5kb2Zm
UmVmVjEgfSwKLSAgICBDb21wYXRpYmlsaXR5Q29weUJyaWRnZSB7IGNvbXBhdGliaWxpdHlfbW9k
ZV9pZDogU3RyaW5nIH0sCi19Ci1gYGAKLQotRW52ZWxvcGUga2luZHMgYXJlIGBIb3N0T3JjaGVz
dHJhdG9yYCBhbmQgYFdvcmxkTWVtYmVyYC4gQSBgV29ybGRNZW1iZXJgIGVudmVsb3BlIHJlcXVp
cmVzIGV4YWN0IHdvcmxkIGJpbmRpbmcsIGd1ZXN0LXJlYWxpemFibGUgZW50cnlwb2ludCwgaW1t
dXRhYmxlIHBvbGljeSBzbmFwc2hvdCwgZXhwbGljaXQgY3JlZGVudGlhbCBwb3N0dXJlLCBhbmQg
YGNvbW1hbmRfYnJva2VyX3JlcXVpcmVkPXRydWVgIGZvciBzaWRlLWVmZmVjdC1jYXBhYmxlIFVB
QSBydW50aW1lcy4KLQotRHVyaW5nIEQxLWJlZm9yZS1EMiBzdGFnaW5nLCBgQ29tcGF0aWJpbGl0
eVVucHJvdmVuYCBtYXkgcHJlc2VydmUgZXhwbGljaXRseSBuYW1lZCBhbmQgbG9nZ2VkIGV4aXN0
aW5nIGJlaGF2aW9yLCBidXQgaXQgY2Fubm90IGNsYWltIFN1YnN0cmF0ZSBwb2xpY3kgbWVkaWF0
aW9uLCBjYW5ub3Qgc2F0aXNmeSBVQUEgY2FnaW5nL2Jyb2tlciBnYXRlcywgYW5kIGNhbm5vdCBw
cm9tb3RlIHRoaXMgc2VhbS4gYEJyb2tlclJlcXVpcmVkYCByZXF1aXJlcyBgY29tbWFuZF9icm9r
ZXJfcmVxdWlyZWQ9dHJ1ZWAgYW5kIGZhaWxzIGNsb3NlZCB3aGVuIGFueSBkZWNsYXJlZCBzaWRl
LWVmZmVjdCBjaGFubmVsIGxhY2tzIGJyb2tlciBzdXBwb3J0LgotCi1DcmVkZW50aWFsLXBvc3R1
cmUgaW52YXJpYW50czoKLQotMS4gYFNlY3VyZUdhdGV3YXlIYW5kb2ZmYCByZXF1aXJlcyBhbiBl
eGFjdCBgaW5fd29ybGRfZ2F0ZXdheV9yZWZgIGluIHRoZSBzYW1lIHdvcmxkIGdlbmVyYXRpb24g
YW5kIGEgdmFsaWQgYExhdW5jaFRpbWVTZWNyZXRIYW5kb2ZmVjFgIHJlZi4KLTIuIGBOb0NyZWRl
bnRpYWxzUmVxdWlyZWRgIHJlcXVpcmVzIG5vIHNlY3JldC1oYW5kb2ZmIHJlZiBhbmQgY2Fubm90
IGxhdGVyIGRpc2NvdmVyIGFtYmllbnQgaG9zdCBjcmVkZW50aWFscy4KLTMuIGBDb21wYXRpYmls
aXR5Q29weUJyaWRnZWAgcmVxdWlyZXMgYSBuYW1lZC9sb2dnZWQgY29tcGF0aWJpbGl0eSBtb2Rl
IGFuZCBjYW5ub3Qgc2F0aXNmeSBjcmVkZW50aWFsLCBwcm9qZWN0aW9uLCBvciBVQUEgY29udHJh
Y3QtcHJvbW90aW9uIGdhdGVzLgotNC4gVGhlIFVBQSBjaGlsZCByZWNlaXZlcyB0aGUgZ2F0ZXdh
eSBlbmRwb2ludC9zZXNzaW9uIGNvbnRyYWN0LCBuZXZlciB0aGUgcmF3IHNlY3JldCBGRCBvciBo
b3N0IGNyZWRlbnRpYWwgcGF5bG9hZC4KLQotQWxsb3dlZCBjaGFubmVsIHZhbHVlcyBkZXNjcmli
ZSBicm9rZXIgc3VwcG9ydCwgbm90IHBlcm1pc3Npb24gdG8gYnlwYXNzOgotCi1gYGB0ZXh0Ci1T
aGVsbENvbW1hbmQKLVBhdGNoT3JBcHBseUVkaXQKLURpcmVjdEZpbGVXcml0ZQotTWNwT3JUb29s
Q2FsbAotUHJvY2Vzc1NwYXduCi1OZXR3b3JrT3BlcmF0aW9uCi1Qcm92aWRlck5hdGl2ZVNpZGVF
ZmZlY3QKLWBgYAotCi1Bbnkgc2lkZS1lZmZlY3QgY2hhbm5lbCBhYnNlbnQgZnJvbSB0aGUgZW52
ZWxvcGUgaXMgZGlzYWJsZWQgaW4gd29ybGQgc2NvcGUuCitDYW5vbmljYWwgY29udGVudDogW2Bj
b250cmFjdHMvd29ybGQtcnVudGltZS1hZGFwdGVyLWV4ZWN1dGlvbi1lbnZlbG9wZS12MS5tZCM4
LXdvcmxkcnVudGltZWFkYXB0ZXJleGVjdXRpb25lbnZlbG9wZXYxYF0oY29udHJhY3RzL3dvcmxk
LXJ1bnRpbWUtYWRhcHRlci1leGVjdXRpb24tZW52ZWxvcGUtdjEubWQjOC13b3JsZHJ1bnRpbWVh
ZGFwdGVyZXhlY3V0aW9uZW52ZWxvcGV2MSkuCiAKICMjIDkuIGBMYXVuY2hUaW1lU2VjcmV0SGFu
ZG9mZlYxYAogCi0jIyMgRXhpc3RpbmcgY2FycmllciB2ZXJzdXMgcmVtYWluaW5nIGFkb3B0aW9u
IHdvcmsKLQotVGhlIGN1cnJlbnQgcmVwbyBhbHJlYWR5IGltcGxlbWVudHMgdGhlIHNlY3VyZSBj
YXJyaWVyIG1lY2hhbmljcyBmb3IgdGhlIG1hbmFnZWQgaW4td29ybGQgZ2F0ZXdheTogYEdhdGV3
YXlBdXRoQnVuZGxlVjFgLCBhbiBpbmhlcml0ZWQgcGlwZSBwcmVwYXJlZCBieSBgd29ybGQtc2Vy
dmljZWAsIHRoZSBwb2ludGVyIGVudmlyb25tZW50IHZhcmlhYmxlIGBTVUJTVFJBVEVfTExNX0FV
VEhfQlVORExFX0ZEYCwgcmF3LXNlY3JldCBlbnYgc2NydWJiaW5nLCBhbmQgZ2F0ZXdheS1zaWRl
IG9uZS10aW1lIHJlYWQgcGx1cyB2YWxpZGF0aW9uLiBGb2N1c2VkIGxhdW5jaGVyIGFuZCBjb25z
dW1lciBpbnRlZ3JhdGlvbiB0ZXN0cyBtYWtlIHRoaXMgYSBwb3NpdGl2ZSBsYW5kZWQgcHJpbWl0
aXZlIHRoYXQgbGF0ZXIgc2xpY2VzIG11c3QgcmV1c2UgYW5kIHByZXNlcnZlLgotCi1gTGF1bmNo
VGltZVNlY3JldEhhbmRvZmZWMWAgYWRkcyB0aGUgb3JjaGVzdHJhdGlvbi1mYWNpbmcgaWRlbnRp
dHksIGxpZmVjeWNsZSwgYW5kIG5vbi1zZWNyZXQgZXZpZGVuY2UgbmVlZGVkIHRvIGpvaW4gdGhh
dCBjYXJyaWVyIHRvIGFuIGV4YWN0IHdvcmxkIGdlbmVyYXRpb24sIGVudmVsb3BlLCByZXRhaW5l
ZCBwYXJ0aWNpcGFudCwgYW5kIGdhdGV3YXkgcmVjZWl2ZXIuIFRoZSBhYnNlbmNlIG9mIHRoaXMg
Y29tcGxldGUgZHVyYWJsZSByZWNvcmQgZG9lcyBub3QgbWVhbiB0aGUgRkQgY2FycmllciBpdHNl
bGYgaXMgYWJzZW50LgotCi1SZW1haW5pbmcgYWRvcHRpb24gd29yayBpcyB0bzoKLQotMS4gZXhw
b3NlIG9yIHBlcnNpc3QgdGhlIG5vbi1zZWNyZXQgaGFuZG9mZiByZWZlcmVuY2Uvc3RhdGUgcmVx
dWlyZWQgYnkgdGhlIGVudmVsb3BlIHdpdGhvdXQgcGVyc2lzdGluZyBzZWNyZXQgcGF5bG9hZHM7
Ci0yLiBwb2ludCBkaXJlY3Qgd29ybGQgQ29kZXgvVUFBIHByb3ZpZGVyIHRyYWZmaWMgYXQgdGhl
IGV4YWN0IG1hbmFnZWQgZ2F0ZXdheSB0aGF0IGNvbnN1bWVkIHRoZSBoYW5kb2ZmOwotMy4gY29u
c3RydWN0IHBlci13b3JrZXIgcnVudGltZS1uYXRpdmUgY29uZmlnIGZyb20gU3Vic3RyYXRlIGxv
Z2ljYWwgY29uZmlnIHBsdXMgYWNjZXB0ZWQgcG9saWN5IGluc3RlYWQgb2YgY29waWVkIGhvc3Qg
Y29uZmlnL2F1dGg7IGFuZAotNC4gcHJvdmUgdGhlIGNvbXBsZXRlIGpvaW5lZCBwYXRoIHdpdGgg
cHJvZHVjdGlvbi1wYXRoIHNtb2tlL2UyZS4KLQotRG8gbm90IHJlcGxhY2UgdGhlIGV4aXN0aW5n
IGNhcnJpZXIgbWVyZWx5IHRvIG1ha2UgaXRzIGltcGxlbWVudGF0aW9uIG5hbWVzIHJlc2VtYmxl
IHRoaXMgY29udHJvbC1wbGFuZSBjb250cmFjdC4gRXh0ZW5kIG9yIGFkYXB0IGl0IG9ubHkgd2hl
cmUgb25lIG9mIHRoZSBpZGVudGl0eSwgZXZpZGVuY2UsIGZhaWwtY2xvc2VkLCBvciBhZG9wdGlv
biByZXF1aXJlbWVudHMgaXMgZ2VudWluZWx5IG1pc3NpbmcuCi0KLWBgYHJ1c3QKLXN0cnVjdCBM
YXVuY2hUaW1lU2VjcmV0SGFuZG9mZlYxIHsKLSAgICBzY2hlbWFfdmVyc2lvbjogdTMyLCAgICAg
ICAgICAgICAgICAgLy8gZXhhY3RseSAxCi0gICAgaGFuZG9mZl9pZDogU3RyaW5nLAotICAgIG9y
Y2hlc3RyYXRpb25fc2Vzc2lvbl9pZDogU3RyaW5nLAotICAgIHdvcmxkX2lkOiBTdHJpbmcsCi0g
ICAgd29ybGRfZ2VuZXJhdGlvbjogdTY0LAotICAgIHJldGFpbmVkX3BhcnRpY2lwYW50X2lkOiBP
cHRpb248U3RyaW5nPiwKLSAgICBydW50aW1lX2ZhbWlseTogU3RyaW5nLAotCi0gICAgLy8gTm9u
LXNlY3JldCBhdXRob3JpdHkgcmVmZXJlbmNlcyBvbmx5LgotICAgIGNyZWRlbnRpYWxfc291cmNl
X3JlZjogQ3JlZGVudGlhbFNvdXJjZVJlZlYxLAotICAgIHJlY2VpdmluZ19nYXRld2F5X3JlZjog
SW5Xb3JsZEdhdGV3YXlSZWZWMSwKLSAgICBkZWxpdmVyeTogU2VjcmV0RGVsaXZlcnlNZWNoYW5p
c21WMSwKLQotICAgIGNyZWF0ZWRfYXQ6IFRpbWVzdGFtcCwKLSAgICBkZWxpdmVyZWRfYXQ6IE9w
dGlvbjxUaW1lc3RhbXA+LAotICAgIGNvbnN1bWVkX2F0OiBPcHRpb248VGltZXN0YW1wPiwKLSAg
ICBleHBpcmVzX2F0OiBUaW1lc3RhbXAsCi0gICAgc3RhdGVfcmV2aXNpb246IHU2NCwKLSAgICBz
dGF0ZTogU2VjcmV0SGFuZG9mZlN0YXRlVjEsCi0gICAgZmFpbHVyZV9kaWFnbm9zdGljX3JlZjog
T3B0aW9uPFJlZGFjdGVkRGlhZ25vc3RpY1JlZlYxPiwKLX0KLQotZW51bSBTZWNyZXREZWxpdmVy
eU1lY2hhbmlzbVYxIHsKLSAgICBTZWN1cmVGZCB7Ci0gICAgICAgIGZkX25hbWU6IFN0cmluZywK
LSAgICAgICAgb25lX3RpbWU6IGJvb2wsCi0gICAgICAgIGdhdGV3YXlfcmVjZWl2ZXJfb25seTog
Ym9vbCwKLSAgICAgICAgZGVueV9jaGlsZF9pbmhlcml0YW5jZTogYm9vbCwKLSAgICAgICAgY2xv
c2VfYWZ0ZXJfY29uc3VtZTogYm9vbCwKLSAgICB9LAotfQotCi1lbnVtIFNlY3JldEhhbmRvZmZT
dGF0ZVYxIHsKLSAgICBQcmVwYXJlZCwKLSAgICBEZWxpdmVyZWQsCi0gICAgQ29uc3VtZWQsCi0g
ICAgRmFpbGVkLAotICAgIEV4cGlyZWQsCi19Ci1gYGAKLQotQWxsb3dlZCB0cmFuc2l0aW9uczoK
LQotYGBgdGV4dAotUHJlcGFyZWQgLT4gRGVsaXZlcmVkIC0+IENvbnN1bWVkCi1QcmVwYXJlZHxE
ZWxpdmVyZWQgLT4gRmFpbGVkfEV4cGlyZWQKLWBgYAotCi1gQ29uc3VtZWRgLCBgRmFpbGVkYCwg
YW5kIGBFeHBpcmVkYCBhcmUgdGVybWluYWwuIFJldXNlIHJlcXVpcmVzIGEgbmV3IGBoYW5kb2Zm
X2lkYCBhbmQgbmV3IGRlc2NyaXB0b3IuCi0KLUxhdW5jaC10aW1lIHNlY3JldCBoYW5kb2ZmIHJ1
bGVzOgotCi0xLiBTZWNyZXQgbWF0ZXJpYWwgaXMgcmVzb2x2ZWQgYnkgaG9zdCBjcmVkZW50aWFs
IGF1dGhvcml0eSBhbmQgbXVzdCBub3QgYmUgcGVyc2lzdGVkIGluIFN1YnN0cmF0ZSByZWNvcmRz
LCBydW50aW1lLW5hdGl2ZSBjb25maWcsIHdvcmtzcGFjZSBvdmVybGF5cywgbWFuaWZlc3RzLCB0
cmFjZXMsIG9yIGxvZ3MuCi0yLiBgY3JlZGVudGlhbF9zb3VyY2VfcmVmYCBpcyBhbiBvcGFxdWUg
aG9zdC1hdXRob3JpdHkgcmVmZXJlbmNlLCBub3QgYSBob3N0IGZpbGVzeXN0ZW0gcGF0aCwgY3Jl
ZGVudGlhbC1zdG9yZSBsb2NhdG9yIGV4cG9zZWQgdG8gdGhlIHdvcmxkLCBvciBkaWdlc3Qgb2Yg
dGhlIHNlY3JldCBwYXlsb2FkLiBgZmRfbmFtZWAgaXMgYSBub24tc2VjcmV0IGxvZ2ljYWwgZGVz
Y3JpcHRvciBsYWJlbC4KLTMuIENvbnRyYWN0LWNvcnJlY3Qgd29ybGQgZXhlY3V0aW9uIG11c3Qg
bm90IGNvcHkgaG9zdCBjcmVkZW50aWFsIGZpbGVzIG9yIHNlY3JldC1iZWFyaW5nIGhvc3QgY29u
ZmlnIGludG8gd29ybGQtdmlzaWJsZSBgQ09ERVhfSE9NRWAsIGAuY29kZXhgLCBgY29uZmlnLnRv
bWxgLCBhdXRoIGZpbGVzLCBvciBlcXVpdmFsZW50IHJ1bnRpbWUgaG9tZXMuCi00LiBBIGJvdW5k
ZWQgbm9uLXNlY3JldCBydW50aW1lIGNvbmZpZyBtYXkgYmUgcmVuZGVyZWQgZnJvbSBTdWJzdHJh
dGUtb3duZWQgbG9naWNhbCBpbnZlbnRvcnkuIENvcHlpbmcgYSBob3N0IGBjb25maWcudG9tbGAg
YXMgYXV0aG9yaXR5IGlzIGNvbXBhdGliaWxpdHkgYnJpZGdpbmcsIG5vdCBwcm9qZWN0aW9uIGF1
dGhvcml0eS4KLTUuIFYxIHZhbGlkYXRpb24gYWNjZXB0cyBgU2VjdXJlRmRgIG9ubHkgd2hlbiBg
b25lX3RpbWVgLCBgZ2F0ZXdheV9yZWNlaXZlcl9vbmx5YCwgYGRlbnlfY2hpbGRfaW5oZXJpdGFu
Y2VgLCBhbmQgYGNsb3NlX2FmdGVyX2NvbnN1bWVgIGFyZSBhbGwgYHRydWVgLgotNi4gVGhlIHNl
Y3VyZSBGRCBpcyBzY29wZWQgdG8gdGhlIGV4YWN0IGByZWNlaXZpbmdfZ2F0ZXdheV9yZWZgLCBj
b25zdW1lZCBieSB0aGUgaW4td29ybGQgU3Vic3RyYXRlIGdhdGV3YXkgYXQgd29ybGQgbGF1bmNo
LCBjbG9zZWQgYWZ0ZXIgY29uc3VtcHRpb24sIGFuZCBuZXZlciBpbmhlcml0ZWQgYnkgdGhlIFVB
QSBhZGFwdGVyIG9yIGl0cyBjaGlsZHJlbi4KLTcuIFRoZSBnYXRld2F54oCUbm90IENvZGV4L1VB
QeKAlG93bnMgY3JlZGVudGlhbCBhcHBsaWNhdGlvbiwgZ2F0ZXdheSBzZXNzaW9uIG1hdGVyaWFs
LCBhbmQgdXBzdHJlYW0gcHJvdmlkZXIgZm9yd2FyZGluZy4gVGhlIFVBQSB0YWxrcyB0byB0aGUg
Z2F0ZXdheSB0aHJvdWdoIHRoZSBlbnZlbG9wZSdzIGVuZHBvaW50L3Nlc3Npb24gY29udHJhY3Qu
Ci04LiBMb2dzLCByZWNlaXB0cywgdHJhY2VzLCBhbmQgbWFuaWZlc3RzIG1heSBjb250YWluIGhh
bmRvZmYgSUQsIG5vbi1zZWNyZXQgcmVmcywgc3RhdGUsIHRpbWVzdGFtcHMsIGFuZCByZWRhY3Rl
ZCBkaWFnbm9zdGljcy4gVGhleSBtdXN0IG5vdCBjb250YWluIHNlY3JldCBwYXlsb2Fkcywgc2Vj
cmV0LWJlYXJpbmcgZmlsZSBwYXRocywgb3IgcmV1c2FibGUgaGFzaGVzL2ZpbmdlcnByaW50cyBk
ZXJpdmVkIGZyb20gdGhlIHNlY3JldCBwYXlsb2FkLgotOS4gRmFpbHVyZSwgZXhwaXJ5LCByZWNl
aXZlciBtaXNtYXRjaCwgd29ybGQtZ2VuZXJhdGlvbiBtaXNtYXRjaCwgZHVwbGljYXRlIGNvbnN1
bXB0aW9uLCBvciBkZXNjcmlwdG9yIGluaGVyaXRhbmNlIHJpc2sgZmFpbHMgY2xvc2VkIGZvciBj
cmVkZW50aWFsLXJlcXVpcmluZyB3b3JsZCBhZGFwdGVycy4KLTEwLiBBIGNvbXBhdGliaWxpdHkg
Y29waWVkLWNyZWRlbnRpYWwgbW9kZSBpcyB0ZW1wb3JhcnksIGV4cGxpY2l0bHkgbmFtZWQgYW5k
IGxvZ2dlZCwgaGFzIHJldGlyZW1lbnQgY3JpdGVyaWEsIGFuZCBjYW5ub3Qgc2F0aXNmeSBgQ29u
dHJhY3RDb3JyZWN0QW5kUHJvdmVuYCBvciBhbnkgc2VjdXJlLWhhbmRvZmYgYWNjZXB0YW5jZSBn
YXRlLgotMTEuIEZhaWxlZC9leHBpcmVkIGhhbmRvZmZzIGNsb3NlIHRoZSBkZXNjcmlwdG9yIGFu
ZCBjbGVhciB0cmFuc2llbnQgYnVmZmVycyBiZWZvcmUgcmV0cnk7IHJldHJ5IGNyZWF0ZXMgYSBu
ZXcgaGFuZG9mZiByYXRoZXIgdGhhbiByZW9wZW5pbmcgb3IgcmVwbGF5aW5nIHRoZSBvbGQgcGF5
bG9hZC4KK0Nhbm9uaWNhbCBjb250ZW50OiBbYGNvbnRyYWN0cy9sYXVuY2gtdGltZS1zZWNyZXQt
aGFuZG9mZi12MS5tZCM5LWxhdW5jaHRpbWVzZWNyZXRoYW5kb2ZmdjFgXShjb250cmFjdHMvbGF1
bmNoLXRpbWUtc2VjcmV0LWhhbmRvZmYtdjEubWQjOS1sYXVuY2h0aW1lc2VjcmV0aGFuZG9mZnYx
KS4KIAogIyMgMTAuIENhbmNlbCBvdXRjb21lIGNhdGVnb3JpZXMKIApkaWZmIC0tZ2l0IGEvbGxt
LWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZCBiL2xsbS1sYXN0LW1p
bGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9SRUFETUUubWQKaW5kZXggYTQxYTQ3OWVkLi5hNTU4
ZmM1NmMgMTAwNjQ0Ci0tLSBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9S
RUFETUUubWQKKysrIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURN
RS5tZApAQCAtMjMsNiArMjMsOCBAQAogfCBgQWN0aXZlUmV0YWluZWRUdXJuUmVjZWlwdFYxYCB8
IGNvbnRyYWN0IHwgW2Bjb250cmFjdHMvYWN0aXZlLXJldGFpbmVkLXR1cm4tcmVjZWlwdC12MS5t
ZGBdKC4uL2NvbnRyYWN0cy9hY3RpdmUtcmV0YWluZWQtdHVybi1yZWNlaXB0LXYxLm1kKSB8IGBj
YW5vbmljYWwgZXh0cmFjdGVkIGNvbnRyYWN0IG93bmVyIGZvciB0aGUgY29tcGxldGUgc2NoZW1h
LCBzdGF0ZSB0cmFuc2l0aW9ucywgYW5kIHJ1bGVzIDHigJMzIGNvdmVyaW5nIHRoZSBzaW5nbGUg
YWN0aXZlIGNhbmNlbGFibGUgdHVybiwgUGFya2VkIGNvbnRpbnVpdHksIGFuZCB0aGUgaG9zdC93
b3JrZXIgcG9zdHVyZSBib3VuZGFyeWAgfCBTdXBlcnNlZGVzIG9ubHkgcm9vdC1jYW5vbmljYWwg
b3duZXJzaGlwIG9mIHRoZSBleHRyYWN0ZWQgYEFjdGl2ZVJldGFpbmVkVHVyblJlY2VpcHRWMWAg
c3BhbjsgYFJlY2VpcHQgYWNjZXB0YW5jZSBzb3VyY2VgIGFuZCBuZWlnaGJvcmluZyBEMTAgb3du
ZXJzIHJlbWFpbiB1bmNoYW5nZWQuIHwgW2AwNGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMu
bWQjNC1hY3RpdmVyZXRhaW5lZHR1cm5yZWNlaXB0djEpIHwKIHwgYFJldGFpbmVkV29ya2VyTWFu
aWZlc3RWMWAgfCBjb250cmFjdCB8IFtgY29udHJhY3RzL3JldGFpbmVkLXdvcmtlci1tYW5pZmVz
dC12MS5tZGBdKC4uL2NvbnRyYWN0cy9yZXRhaW5lZC13b3JrZXItbWFuaWZlc3QtdjEubWQpIHwg
YGNhbm9uaWNhbCBleHRyYWN0ZWQgY29udHJhY3Qgb3duZXIgZm9yIHRoZSBjb21wbGV0ZSBzY2hl
bWEgYW5kIHJ1bGVzIDHigJM1IGNvdmVyaW5nIHdvcmtlci1jYXAgbmFycm93aW5nLCBwYXJrZWQg
aWRlbnRpdHkgY29udGludWl0eSwgd29ybGQtZ2VuZXJhdGlvbiBpbnZhbGlkYXRpb24sIGFuZCB0
aGUgaW1tdXRhYmxlIEIxIGFjY2VwdGVkLXJlY29yZCBib3VuZGFyeWAgfCBTdXBlcnNlZGVzIG9u
bHkgcm9vdC1jYW5vbmljYWwgb3duZXJzaGlwIG9mIHRoZSBleHRyYWN0ZWQgYFJldGFpbmVkV29y
a2VyTWFuaWZlc3RWMWAgc3BhbjsgYFJlY2VpcHQgYWNjZXB0YW5jZSBzb3VyY2VgLCBgRGlzcGF0
Y2hQb2xpY3lOYXJyb3dpbmdQYXRjaFYxYCwgYW5kIG5laWdoYm9yaW5nIEQxMCBvd25lcnMgcmVt
YWluIHVuY2hhbmdlZC4gfCBbYDA0YF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM2LXJl
dGFpbmVkd29ya2VybWFuaWZlc3R2MSkgfAogfCBgRGlzcGF0Y2hQb2xpY3lOYXJyb3dpbmdQYXRj
aFYxYCB8IGNvbnRyYWN0IHwgW2Bjb250cmFjdHMvZGlzcGF0Y2gtcG9saWN5LW5hcnJvd2luZy1w
YXRjaC12MS5tZGBdKC4uL2NvbnRyYWN0cy9kaXNwYXRjaC1wb2xpY3ktbmFycm93aW5nLXBhdGNo
LXYxLm1kKSB8IGBjYW5vbmljYWwgZXh0cmFjdGVkIGNvbnRyYWN0IG93bmVyIGZvciB0aGUgY29t
cGxldGUgc2NoZW1hLCBgRGlzcGF0Y2hDYXBhYmlsaXR5U3ViamVjdFYxYCBlbnVtLCBhbmQgdGhl
IFYxIGBSZXN0cmljdGVkUG9saWN5UGF0Y2hWMWAgd29ybGRfZnMtb25seSByZWplY3Rpb24gcnVs
ZWAgfCBTdXBlcnNlZGVzIG9ubHkgcm9vdC1jYW5vbmljYWwgb3duZXJzaGlwIG9mIHRoZSBleHRy
YWN0ZWQgYERpc3BhdGNoUG9saWN5TmFycm93aW5nUGF0Y2hWMWAgc3BhbjsgYFJldGFpbmVkV29y
a2VyTWFuaWZlc3RWMWAgYW5kIG5laWdoYm9yaW5nIEQxMCBvd25lcnMgcmVtYWluIHVuY2hhbmdl
ZC4gfCBbYDA0YF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM3LWRpc3BhdGNocG9saWN5
bmFycm93aW5ncGF0Y2h2MSkgfAorfCBgV29ybGRSdW50aW1lQWRhcHRlckV4ZWN1dGlvbkVudmVs
b3BlVjFgIHwgY29udHJhY3QgfCBbYGNvbnRyYWN0cy93b3JsZC1ydW50aW1lLWFkYXB0ZXItZXhl
Y3V0aW9uLWVudmVsb3BlLXYxLm1kYF0oLi4vY29udHJhY3RzL3dvcmxkLXJ1bnRpbWUtYWRhcHRl
ci1leGVjdXRpb24tZW52ZWxvcGUtdjEubWQpIHwgYGNhbm9uaWNhbCBleHRyYWN0ZWQgY29udHJh
Y3Qgb3duZXIgZm9yIHRoZSBjb21wbGV0ZSBzY2hlbWEsIHBvc3R1cmUgZW51bXMsIHdvcmxkLWJp
bmRpbmcgYW5kIGltbXV0YWJsZS1wb2xpY3kgcmVxdWlyZW1lbnRzLCBEMS1iZWZvcmUtRDIgY29t
cGF0aWJpbGl0eSBib3VuZGFyeSwgY3JlZGVudGlhbC1wb3N0dXJlIGludmFyaWFudHMgMeKAkzQs
IGFuZCBhbGxvd2VkIHNpZGUtZWZmZWN0LWNoYW5uZWwgbGl0ZXJhbHNgIHwgU3VwZXJzZWRlcyBv
bmx5IHJvb3QtY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIGBXb3JsZFJ1bnRp
bWVBZGFwdGVyRXhlY3V0aW9uRW52ZWxvcGVWMWAgc3BhbjsgYFJldGFpbmVkV29ya2VyTWFuaWZl
c3RWMWAsIGBEaXNwYXRjaFBvbGljeU5hcnJvd2luZ1BhdGNoVjFgLCBgTGF1bmNoVGltZVNlY3Jl
dEhhbmRvZmZWMWAsIGFuZCBuZWlnaGJvcmluZyBEMTAgb3duZXJzIHJlbWFpbiB1bmNoYW5nZWQu
IHwgW2AwNGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjOC13b3JsZHJ1bnRpbWVhZGFw
dGVyZXhlY3V0aW9uZW52ZWxvcGV2MSkgfAorfCBgTGF1bmNoVGltZVNlY3JldEhhbmRvZmZWMWAg
fCBjb250cmFjdCB8IFtgY29udHJhY3RzL2xhdW5jaC10aW1lLXNlY3JldC1oYW5kb2ZmLXYxLm1k
YF0oLi4vY29udHJhY3RzL2xhdW5jaC10aW1lLXNlY3JldC1oYW5kb2ZmLXYxLm1kKSB8IGBjYW5v
bmljYWwgZXh0cmFjdGVkIGNvbnRyYWN0IG93bmVyIGZvciB0aGUgZXhpc3RpbmctY2Fycmllci12
ZXJzdXMtcmVtYWluaW5nLWFkb3B0aW9uIGJvdW5kYXJ5LCBjb21wbGV0ZSBzY2hlbWEsIGRlbGl2
ZXJ5L3N0YXRlIGVudW1zLCB0cmFuc2l0aW9ucywgdGVybWluYWwvcmV1c2UgcnVsZXMsIGFuZCBo
YW5kb2ZmIHJ1bGVzIDHigJMxMWAgfCBTdXBlcnNlZGVzIG9ubHkgcm9vdC1jYW5vbmljYWwgb3du
ZXJzaGlwIG9mIHRoZSBleHRyYWN0ZWQgYExhdW5jaFRpbWVTZWNyZXRIYW5kb2ZmVjFgIHNwYW47
IGBXb3JsZFJ1bnRpbWVBZGFwdGVyRXhlY3V0aW9uRW52ZWxvcGVWMWAsIGBDYW5jZWwgb3V0Y29t
ZSBjYXRlZ29yaWVzYCwgYW5kIG5laWdoYm9yaW5nIEQxMCBvd25lcnMgcmVtYWluIHVuY2hhbmdl
ZC4gfCBbYDA0YF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM5LWxhdW5jaHRpbWVzZWNy
ZXRoYW5kb2ZmdjEpIHwKIHwgYHNoYXJlZC10YXJnZXQtYXJjaGl0ZWN0dXJlYCB8IGFyY2hpdGVj
dHVyZSBpbmRleCB8IFtgYXJjaGl0ZWN0dXJlL1JFQURNRS5tZGBdKC4uL2FyY2hpdGVjdHVyZS9S
RUFETUUubWQpIHwgYG5vbi1hdXRob3JpdGF0aXZlIG5hdmlnYXRpb24gZm9yIHRoZSBleHRyYWN0
ZWQgZXhlY3V0aXZlIHRhcmdldCBkZWNpc2lvbiwgYXV0aG9yaXR5IG1hcCwgbnVtYmVyZWQgaW52
YXJpYW50cywgYW5kIHN0YWJsZSByZXZpZXcgcXVlc3Rpb25gIHwgU3VwZXJzZWRlcyBvbmx5IHJv
b3QtY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIHNoYXJlZCBhcmNoaXRlY3R1
cmUgc3BhbnM7IHBhY2tldC1mYW1pbHktbG9jYWwgRDUvRDYgZm9yd2FyZGVycyByZW1haW4gYXQg
dGhlaXIgZXhpc3Rpbmcgb3duZXJzLiB8IFtgMDFgXSguLi8wMS10YXJnZXQtYXJjaGl0ZWN0dXJl
Lm1kI2V4ZWN1dGl2ZS1kZWNpc2lvbiksIFtgMDFgXSguLi8wMS10YXJnZXQtYXJjaGl0ZWN0dXJl
Lm1kI2F1dGhvcml0eS1tYXApLCBbYDAxYF0oLi4vMDEtdGFyZ2V0LWFyY2hpdGVjdHVyZS5tZCNu
b24tbmVnb3RpYWJsZS1pbnZhcmlhbnRzKSwgW2AwMWBdKC4uLzAxLXRhcmdldC1hcmNoaXRlY3R1
cmUubWQjcmV2aWV3LXF1ZXN0aW9uKSB8CiB8IGBzaGFyZWQtc2VhbS1jcm9zc3dhbGtgIHwgc2Vh
bSBpbmRleCB8IFtgc2VhbXMvUkVBRE1FLm1kYF0oLi4vc2VhbXMvUkVBRE1FLm1kKSB8IGBjYW5v
bmljYWwgc2hhcmVkIHNlYW0tY3Jvc3N3YWxrIHJ1bGVzIHBsdXMgZXh0cmFjdGVkIHNlYW0tZmFt
aWx5IG5hdmlnYXRpb24gZm9yIGhvc3Qvc2Vzc2lvbiwgcGVyc2lzdGVuY2UvY29tcGF0aWJpbGl0
eSwgZGlzcGF0Y2gvZXBpc29kZSB0cmFuc3BvcnQsIHBvbGljeS9uYXJyb3dpbmcsIHJ1bnRpbWUt
ZXZlbnQvcmVjZWlwdC9zdXBlcnZpc2lvbi9yZXRhaW5lZC1ydW50aW1lLCBvYmxpZ2F0aW9ucy9o
b3N0IHJlLWVuZ2FnZW1lbnQsIGNvbmZpZ3VyYXRpb24vZ2F0ZXdheSBhZG9wdGlvbiwgYW5kIFVB
QS9wcm92aWRlciByZWFsaXphdGlvbi9zaWRlLWVmZmVjdCBtZWRpYXRpb25gIHwgU3VwZXJzZWRl
cyBvbmx5IHRoZSBleHRyYWN0ZWQgcm9vdCByZWFkaW5nLXJ1bGUvY2xhc3NpZmljYXRpb24gc3Bh
bnMgYW5kIHRoZSBleHRyYWN0ZWQgaG9zdC9zZXNzaW9uIGF1dGhvcml0eSwgcGVyc2lzdGVuY2Uv
Y29tcGF0aWJpbGl0eSwgZGlzcGF0Y2gvZXBpc29kZSB0cmFuc3BvcnQsIHBvbGljeS9uYXJyb3dp
bmcsIHJ1bnRpbWUtZXZlbnQvcmVjZWlwdC9zdXBlcnZpc2lvbi9yZXRhaW5lZC1ydW50aW1lLCBv
YmxpZ2F0aW9ucy9ob3N0IHJlLWVuZ2FnZW1lbnQsIGNvbmZpZ3VyYXRpb24vZ2F0ZXdheSBhZG9w
dGlvbiwgYW5kIFVBQS9wcm92aWRlciByZWFsaXphdGlvbi9zaWRlLWVmZmVjdCBtZWRpYXRpb24g
ZmFtaWx5IHJvd3M7IEEwIGFuZCB0aGUgZXhpc3RpbmcgRDYgYEhvc3RTZXNzaW9uQXV0aG9yaXR5
YCwgYFdvcmxkV29ya2VyTWVzc2FnaW5nUHJvdG9jb2xgLCBhbmQgYE9ibGlnYXRpb25MZWRnZXJg
IGNvbXBhdGliaWxpdHkgcm93cyByZW1haW4gYXQgdGhlaXIgY3VycmVudCBvd25lcnMuIHwgW2Aw
MmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1kI3JlYWRpbmctcnVsZSksIFtgMDJgXSguLi8wMi1z
ZWFtLWNyb3Nzd2Fsay5tZCNhLWhvc3QtYXV0aG9yaXR5LWFuZC1pbmdyZXNzKSwgW2AwMmBdKC4u
LzAyLXNlYW0tY3Jvc3N3YWxrLm1kI2ItZGlzcGF0Y2gtcG9saWN5LXJlY2VpcHRzLWFuZC1yZXRh
aW5lZC1ydW50aW1lKSwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1kI2Mtb2JsaWdhdGlv
bnMtYW5kLWhvc3QtcmUtZW5nYWdlbWVudCksIFtgMDJgXSguLi8wMi1zZWFtLWNyb3Nzd2Fsay5t
ZCNkLXVhYS1yZWFsaXphdGlvbi1wcm9qZWN0aW9uLWFuZC1zaWRlLWVmZmVjdC1tZWRpYXRpb24p
LCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjY2xhc3NpZmljYXRpb24tY29uc2VxdWVu
Y2VzKSB8CiB8IGBzaGFyZWQtc2xpY2UtbWFwYCB8IHNsaWNlIGluZGV4IHwgW2BzbGljZXMvUkVB
RE1FLm1kYF0oLi4vc2xpY2VzL1JFQURNRS5tZCkgfCBgY2Fub25pY2FsIHNoYXJlZCBzZXF1ZW5j
aW5nL2RlcGVuZGVuY3kgYW5kIHNsaWNlLWNsb3Nlb3V0IG93bmVyIHBsdXMgbm9uLWF1dGhvcml0
YXRpdmUgRDkgVHJhY2sgQeKAk0UgbmF2aWdhdGlvbiBhbmQgQTEvQTEuNCBwcm9qZWN0aW9uIGlu
ZGV4YCB8IFN1cGVyc2VkZXMgb25seSByb290LWNhbm9uaWNhbCBvd25lcnNoaXAgb2YgdGhlIGV4
dHJhY3RlZCBzaGFyZWQgc2VxdWVuY2luZy9jbG9zZW91dCBzcGFucyBwbHVzIHRoZSBleHRyYWN0
ZWQgVHJhY2sgQeKAk0UgYW5kIEExL0ExLjQgcHJvamVjdGlvbiBzcGFuczsgY29udHJvbGxpbmcg
c2NoZWR1bGUgYXV0aG9yaXR5IHJlbWFpbnMgd2l0aCBkZWNpc2lvbnMsIHBhY2tldHMsIGdhdGVz
LCBhbmQgYGN1cnJlbnQubWRgLiB8IFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjc2Vx
dWVuY2luZy1ydWxlcyksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjdHJhY2stYS0t
YXV0aG9yaXR5LWFuZC1zdXJmYWNlLW5ldXRyYWxpdHkpLCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xp
Y2UtbWFwLm1kI2ExLWJvdW5kZWQtcGFja2V0LWRlY29tcG9zaXRpb24pLCBbYDAzYF0oLi4vMDMt
cGhhc2Utc2xpY2UtbWFwLm1kI3RyYWNrLWItLXdvcmxkLWRpc3BhdGNoLXJlY2VpcHRzLXN1cGVy
dmlzaW9uLWFuZC1jYW5jZWwpLCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3RyYWNr
LWMtLW9ibGlnYXRpb25zLWluYm94LWF1dG8tYXR0YWNoLWFuZC1yb3V0ZXItYXR0YWNoKSwgW2Aw
M2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCN0cmFjay1kLS11YWEtZXhlY3V0aW9uLWVudmVs
b3BlLWFuZC1zaWRlLWVmZmVjdC1tZWRpYXRpb24pLCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2Ut
bWFwLm1kI3RyYWNrLWUtLWRpc3BhdGNoLXNjb3BlZC1wb2xpY3ktbmFycm93aW5nLWFuZC1jb25m
aWctcHJvamVjdGlvbiksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjc2xpY2UtY2xv
c2VvdXQtbWluaW11bSkgfApkaWZmIC0tZ2l0IGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFj
dG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZCBiL2xsbS1sYXN0LW1pbGUvcnVudGlt
ZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQKaW5kZXggZjU1NjZlNjM3
Li41YTdkYWFmMjggMTAwNjQ0Ci0tLSBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9t
aWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQKKysrIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1l
LXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZApAQCAtMTkwLDMgKzE5MCw1
IEBAIFRoaXMgbGVkZ2VyIHJlY29yZHMgY29udGVudC1wcmVzZXJ2aW5nIGF1dGhvcml0eSB0cmFu
c2ZlcnMgd2hpbGUgcmV0YWluaW5nIHJlcXVpCiB8IEQxMCB8IFtgMDQtY29udHJhY3RzLWFuZC1n
YXRlcy5tZGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQpIHwgNC4gYEFjdGl2ZVJldGFp
bmVkVHVyblJlY2VpcHRWMWAgfCBbYDQtYWN0aXZlcmV0YWluZWR0dXJucmVjZWlwdHYxYF0oLi4v
MDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM0LWFjdGl2ZXJldGFpbmVkdHVybnJlY2VpcHR2MSkg
fCBsaW5lcyAxMznigJMxODMgfCBgMmNjOTk2MTExZDgyZDI5NmY4OWJkZDg4ZjgwZGY1YzBiNTU5
YmRiMzA1YWVjNTQ2ODgxYjc0OWFjZTkzZjY3ZWAgfCBbYC4uL2NvbnRyYWN0cy9hY3RpdmUtcmV0
YWluZWQtdHVybi1yZWNlaXB0LXYxLm1kYF0oLi4vY29udHJhY3RzL2FjdGl2ZS1yZXRhaW5lZC10
dXJuLXJlY2VpcHQtdjEubWQpIHwgY29udHJhY3QgfCBjYW5vbmljYWwgZGVzdGluYXRpb247IHNv
dXJjZSBjb21wYXRpYmlsaXR5IGFuY2hvciB8IG5vbmUgfCByZXN0b3JlIHRoZSBleGFjdCAxNTU3
LWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMy1hY3RpdmVl
cGhlbWVyYWx0YXNrcmVjZWlwdHYxYCBhbmQgdGhlIGV4YWN0IDE0NDMtYnl0ZSBzb3VyY2Ugc3Bh
biBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM0LWFjdGl2ZXJldGFpbmVkdHVybnJlY2Vp
cHR2MWA7IHJlbW92ZSBgY29udHJhY3RzL2FjdGl2ZS1lcGhlbWVyYWwtdGFzay1yZWNlaXB0LXYx
Lm1kYCBhbmQgYGNvbnRyYWN0cy9hY3RpdmUtcmV0YWluZWQtdHVybi1yZWNlaXB0LXYxLm1kYDsg
cmVtb3ZlIHRoZSBleGFjdCBgQWN0aXZlRXBoZW1lcmFsVGFza1JlY2VpcHRWMWAgYW5kIGBBY3Rp
dmVSZXRhaW5lZFR1cm5SZWNlaXB0VjFgIHJvd3MgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsgcmVt
b3ZlIGJvdGggRDEwIGxlZGdlciBlbnRyaWVzIGZvciB0aGlzIGJhdGNoOyBsZWF2ZSBgSG9zdEV4
ZWN1dGlvbkVwaXNvZGVWMWAsIGRlZmVycmVkIHJldGFpbmVkLXNwYXduIGFkbWlzc2lvbiByZWNv
dmVyeSwgYFJlY2VpcHQgYWNjZXB0YW5jZSBzb3VyY2VgLCBhbGwgRDMvRDXigJNEOSBvd25lcnMs
IGBpbmRleC9jdXJyZW50Lm1kYCwgYHJldmlldy1jb250cm9sL2AsIHJlbWFpbmluZyBEMTAgdW5p
dHMsIGFuZCBldmVyeSBwYXRoIG91dHNpZGUgdGhlIGZpdmUtcGF0aCBmZW5jZSB1bmNoYW5nZWQg
fAogfCBEMTAgfCBbYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWRgXSguLi8wNC1jb250cmFjdHMt
YW5kLWdhdGVzLm1kKSB8IDYuIGBSZXRhaW5lZFdvcmtlck1hbmlmZXN0VjFgIHwgW2A2LXJldGFp
bmVkd29ya2VybWFuaWZlc3R2MWBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjNi1yZXRh
aW5lZHdvcmtlcm1hbmlmZXN0djEpIHwgbGluZXMgMTAz4oCTMTQwICgxNTc2IGJ5dGVzKSB8IGBj
ZDI1ZWMxYzE5NTQ4Yjg0NDJlMDFmNzUzNTE3YzExN2NhYjlhMzUwY2I4MjhkZTgyMTRhMzUxYmRj
MDllZTA3YCB8IFtgLi4vY29udHJhY3RzL3JldGFpbmVkLXdvcmtlci1tYW5pZmVzdC12MS5tZGBd
KC4uL2NvbnRyYWN0cy9yZXRhaW5lZC13b3JrZXItbWFuaWZlc3QtdjEubWQpIHwgY29udHJhY3Qg
fCBjYW5vbmljYWwgZGVzdGluYXRpb247IHNvdXJjZSBjb21wYXRpYmlsaXR5IGFuY2hvciB8IG5v
bmUgfCByZXN0b3JlIHRoZSBleGFjdCAxNTc2LWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRy
YWN0cy1hbmQtZ2F0ZXMubWQjNi1yZXRhaW5lZHdvcmtlcm1hbmlmZXN0djFgIGFuZCB0aGUgZXhh
Y3QgODEwLWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjNy1k
aXNwYXRjaHBvbGljeW5hcnJvd2luZ3BhdGNodjFgOyByZW1vdmUgYGNvbnRyYWN0cy9yZXRhaW5l
ZC13b3JrZXItbWFuaWZlc3QtdjEubWRgIGFuZCBgY29udHJhY3RzL2Rpc3BhdGNoLXBvbGljeS1u
YXJyb3dpbmctcGF0Y2gtdjEubWRgOyByZW1vdmUgdGhlIGV4YWN0IGBSZXRhaW5lZFdvcmtlck1h
bmlmZXN0VjFgIGFuZCBgRGlzcGF0Y2hQb2xpY3lOYXJyb3dpbmdQYXRjaFYxYCByb3dzIGZyb20g
YGluZGV4L1JFQURNRS5tZGA7IHJlbW92ZSBib3RoIEQxMCBsZWRnZXIgZW50cmllcyBmb3IgdGhp
cyBiYXRjaDsgbGVhdmUgYFJlY2VpcHQgYWNjZXB0YW5jZSBzb3VyY2VgLCBgV29ybGRSdW50aW1l
QWRhcHRlckV4ZWN1dGlvbkVudmVsb3BlVjFgLCBhbGwgRDMvRDXigJNEMTAgb3duZXJzLCBgaW5k
ZXgvY3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9gLCByZW1haW5pbmcgbGF0ZXIgRDEwIHVu
aXRzLCBhbmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmaXZlLXBhdGggZmVuY2UgdW5jaGFuZ2Vk
IHwKIHwgRDEwIHwgW2AwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kYF0oLi4vMDQtY29udHJhY3Rz
LWFuZC1nYXRlcy5tZCkgfCA3LiBgRGlzcGF0Y2hQb2xpY3lOYXJyb3dpbmdQYXRjaFYxYCB8IFtg
Ny1kaXNwYXRjaHBvbGljeW5hcnJvd2luZ3BhdGNodjFgXSguLi8wNC1jb250cmFjdHMtYW5kLWdh
dGVzLm1kIzctZGlzcGF0Y2hwb2xpY3luYXJyb3dpbmdwYXRjaHYxKSB8IGxpbmVzIDE0MeKAkzE2
NyAoODEwIGJ5dGVzKSB8IGBjMGZkMjJhN2M2MzBjMzBlOGI2ZmNhOTc2NjQ1ZDg0ZGM3NTE1YjQz
NWFmNzU5YzRlY2MwYmM4Yjg4NzFkYTRiYCB8IFtgLi4vY29udHJhY3RzL2Rpc3BhdGNoLXBvbGlj
eS1uYXJyb3dpbmctcGF0Y2gtdjEubWRgXSguLi9jb250cmFjdHMvZGlzcGF0Y2gtcG9saWN5LW5h
cnJvd2luZy1wYXRjaC12MS5tZCkgfCBjb250cmFjdCB8IGNhbm9uaWNhbCBkZXN0aW5hdGlvbjsg
c291cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9yIHwgbm9uZSB8IHJlc3RvcmUgdGhlIGV4YWN0IDE1
NzYtYnl0ZSBzb3VyY2Ugc3BhbiBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM2LXJldGFp
bmVkd29ya2VybWFuaWZlc3R2MWAgYW5kIHRoZSBleGFjdCA4MTAtYnl0ZSBzb3VyY2Ugc3BhbiBh
dCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM3LWRpc3BhdGNocG9saWN5bmFycm93aW5ncGF0
Y2h2MWA7IHJlbW92ZSBgY29udHJhY3RzL3JldGFpbmVkLXdvcmtlci1tYW5pZmVzdC12MS5tZGAg
YW5kIGBjb250cmFjdHMvZGlzcGF0Y2gtcG9saWN5LW5hcnJvd2luZy1wYXRjaC12MS5tZGA7IHJl
bW92ZSB0aGUgZXhhY3QgYFJldGFpbmVkV29ya2VyTWFuaWZlc3RWMWAgYW5kIGBEaXNwYXRjaFBv
bGljeU5hcnJvd2luZ1BhdGNoVjFgIHJvd3MgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsgcmVtb3Zl
IGJvdGggRDEwIGxlZGdlciBlbnRyaWVzIGZvciB0aGlzIGJhdGNoOyBsZWF2ZSBgUmVjZWlwdCBh
Y2NlcHRhbmNlIHNvdXJjZWAsIGBXb3JsZFJ1bnRpbWVBZGFwdGVyRXhlY3V0aW9uRW52ZWxvcGVW
MWAsIGFsbCBEMy9ENeKAk0QxMCBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYHJldmlldy1j
b250cm9sL2AsIHJlbWFpbmluZyBsYXRlciBEMTAgdW5pdHMsIGFuZCBldmVyeSBwYXRoIG91dHNp
ZGUgdGhlIGZpdmUtcGF0aCBmZW5jZSB1bmNoYW5nZWQgfAorfCBEMTAgfCBbYDA0LWNvbnRyYWN0
cy1hbmQtZ2F0ZXMubWRgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kKSB8IDguIGBXb3Js
ZFJ1bnRpbWVBZGFwdGVyRXhlY3V0aW9uRW52ZWxvcGVWMWAgfCBbYDgtd29ybGRydW50aW1lYWRh
cHRlcmV4ZWN1dGlvbmVudmVsb3BldjFgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzgt
d29ybGRydW50aW1lYWRhcHRlcmV4ZWN1dGlvbmVudmVsb3BldjEpIHwgbGluZXMgMTEx4oCTMTc1
ICgyNzgzIGJ5dGVzKSB8IGBjZGI3NTVhYzAwY2VhOTliZTAzMjdmODc1NDdmZDU4NmYxYzYzMzVj
NDcwMTEzY2VhMDMzMDdjYzhlM2Q4ZWFjYCB8IFtgLi4vY29udHJhY3RzL3dvcmxkLXJ1bnRpbWUt
YWRhcHRlci1leGVjdXRpb24tZW52ZWxvcGUtdjEubWRgXSguLi9jb250cmFjdHMvd29ybGQtcnVu
dGltZS1hZGFwdGVyLWV4ZWN1dGlvbi1lbnZlbG9wZS12MS5tZCkgfCBjb250cmFjdCB8IGNhbm9u
aWNhbCBkZXN0aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9yIHwgcGFpcmVkIGZh
bWlseSBhdG9taWNpdHkgd2l0aCBgTGF1bmNoVGltZVNlY3JldEhhbmRvZmZWMWA7IHByZXNlcnZl
IHRoZSBsYW5kZWQgYEdhdGV3YXlBdXRoQnVuZGxlVjFgIHNlY3VyZS1GRCBjYXJyaWVyIGZhY3Qg
d2l0aG91dCBwcm9tb3Rpbmcgam9pbmVkIGFkb3B0aW9uIHwgcmVzdG9yZSB0aGUgZXhhY3QgMjc4
My1ieXRlIHNvdXJjZSBzcGFuIGF0IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzgtd29ybGRy
dW50aW1lYWRhcHRlcmV4ZWN1dGlvbmVudmVsb3BldjFgIGFuZCB0aGUgZXhhY3QgNDk2OC1ieXRl
IHNvdXJjZSBzcGFuIGF0IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzktbGF1bmNodGltZXNl
Y3JldGhhbmRvZmZ2MWA7IHJlbW92ZSBgY29udHJhY3RzL3dvcmxkLXJ1bnRpbWUtYWRhcHRlci1l
eGVjdXRpb24tZW52ZWxvcGUtdjEubWRgIGFuZCBgY29udHJhY3RzL2xhdW5jaC10aW1lLXNlY3Jl
dC1oYW5kb2ZmLXYxLm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgV29ybGRSdW50aW1lQWRhcHRlckV4
ZWN1dGlvbkVudmVsb3BlVjFgIGFuZCBgTGF1bmNoVGltZVNlY3JldEhhbmRvZmZWMWAgcm93cyBm
cm9tIGBpbmRleC9SRUFETUUubWRgOyByZW1vdmUgYm90aCBEMTAgbGVkZ2VyIGVudHJpZXMgZm9y
IHRoaXMgYmF0Y2g7IGxlYXZlIGBSZXRhaW5lZFdvcmtlck1hbmlmZXN0VjFgLCBgRGlzcGF0Y2hQ
b2xpY3lOYXJyb3dpbmdQYXRjaFYxYCwgYENhbmNlbCBvdXRjb21lIGNhdGVnb3JpZXNgLCBhbGwg
RDMvRDXigJNEMTAgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9g
LCBhbmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmaXZlLXBhdGggZmVuY2UgdW5jaGFuZ2VkIHwK
K3wgRDEwIHwgW2AwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kYF0oLi4vMDQtY29udHJhY3RzLWFu
ZC1nYXRlcy5tZCkgfCA5LiBgTGF1bmNoVGltZVNlY3JldEhhbmRvZmZWMWAgfCBbYDktbGF1bmNo
dGltZXNlY3JldGhhbmRvZmZ2MWBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjOS1sYXVu
Y2h0aW1lc2VjcmV0aGFuZG9mZnYxKSB8IGxpbmVzIDE3NuKAkzI1OCAoNDk2OCBieXRlcykgfCBg
MGE1YTEzNzk5OTY5ZjE2NWMzYWNjYTIwMGRmZDVmMjhiZjJlNzRjY2JmZWI3MTM1NjE3NmNhZWNl
Yzc4YjBjZGAgfCBbYC4uL2NvbnRyYWN0cy9sYXVuY2gtdGltZS1zZWNyZXQtaGFuZG9mZi12MS5t
ZGBdKC4uL2NvbnRyYWN0cy9sYXVuY2gtdGltZS1zZWNyZXQtaGFuZG9mZi12MS5tZCkgfCBjb250
cmFjdCB8IGNhbm9uaWNhbCBkZXN0aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9y
IHwgcGFpcmVkIGZhbWlseSBhdG9taWNpdHkgd2l0aCBgV29ybGRSdW50aW1lQWRhcHRlckV4ZWN1
dGlvbkVudmVsb3BlVjFgOyBwcmVzZXJ2ZSBleGlzdGluZyBzZWN1cmUtY2FycmllciBldmlkZW5j
ZSBhbmQgdGhlIHJlbWFpbmluZy1hZG9wdGlvbiBib3VuZGFyeSB3aXRob3V0IHJlaW1wbGVtZW50
YXRpb24gY2xhaW1zIHwgcmVzdG9yZSB0aGUgZXhhY3QgMjc4My1ieXRlIHNvdXJjZSBzcGFuIGF0
IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzgtd29ybGRydW50aW1lYWRhcHRlcmV4ZWN1dGlv
bmVudmVsb3BldjFgIGFuZCB0aGUgZXhhY3QgNDk2OC1ieXRlIHNvdXJjZSBzcGFuIGF0IGAwNC1j
b250cmFjdHMtYW5kLWdhdGVzLm1kIzktbGF1bmNodGltZXNlY3JldGhhbmRvZmZ2MWA7IHJlbW92
ZSBgY29udHJhY3RzL3dvcmxkLXJ1bnRpbWUtYWRhcHRlci1leGVjdXRpb24tZW52ZWxvcGUtdjEu
bWRgIGFuZCBgY29udHJhY3RzL2xhdW5jaC10aW1lLXNlY3JldC1oYW5kb2ZmLXYxLm1kYDsgcmVt
b3ZlIHRoZSBleGFjdCBgV29ybGRSdW50aW1lQWRhcHRlckV4ZWN1dGlvbkVudmVsb3BlVjFgIGFu
ZCBgTGF1bmNoVGltZVNlY3JldEhhbmRvZmZWMWAgcm93cyBmcm9tIGBpbmRleC9SRUFETUUubWRg
OyByZW1vdmUgYm90aCBEMTAgbGVkZ2VyIGVudHJpZXMgZm9yIHRoaXMgYmF0Y2g7IGxlYXZlIGBS
ZXRhaW5lZFdvcmtlck1hbmlmZXN0VjFgLCBgRGlzcGF0Y2hQb2xpY3lOYXJyb3dpbmdQYXRjaFYx
YCwgYENhbmNlbCBvdXRjb21lIGNhdGVnb3JpZXNgLCBhbGwgRDMvRDXigJNEMTAgb3duZXJzLCBg
aW5kZXgvY3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9gLCBhbmQgZXZlcnkgcGF0aCBvdXRz
aWRlIHRoZSBmaXZlLXBhdGggZmVuY2UgdW5jaGFuZ2VkIHwKZGlmZiAtLWdpdCBhL2xsbS1sYXN0
LW1pbGUvcnVudGltZS1yZWZhY3Rvci9jb250cmFjdHMvd29ybGQtcnVudGltZS1hZGFwdGVyLWV4
ZWN1dGlvbi1lbnZlbG9wZS12MS5tZCBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9j
b250cmFjdHMvd29ybGQtcnVudGltZS1hZGFwdGVyLWV4ZWN1dGlvbi1lbnZlbG9wZS12MS5tZApu
ZXcgZmlsZSBtb2RlIDEwMDY0NAppbmRleCAwMDAwMDAwMDAuLmMwNzFmNmQ3MQotLS0gL2Rldi9u
dWxsCisrKyBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9jb250cmFjdHMvd29ybGQt
cnVudGltZS1hZGFwdGVyLWV4ZWN1dGlvbi1lbnZlbG9wZS12MS5tZApAQCAtMCwwICsxLDczIEBA
CisqKktpbmQ6KiogY29udHJhY3QKKyoqU3RhdHVzOioqIGNhbm9uaWNhbAorKipDYW5vbmljYWwg
Zm9yOioqIGNvbXBsZXRlIGV4dHJhY3RlZCBgV29ybGRSdW50aW1lQWRhcHRlckV4ZWN1dGlvbkVu
dmVsb3BlVjFgIHNjaGVtYSwgbWVkaWF0aW9uIGFuZCBjcmVkZW50aWFsIHBvc3R1cmUgZW51bXMs
IHdvcmxkLWJpbmRpbmcgYW5kIGltbXV0YWJsZS1wb2xpY3kgcmVxdWlyZW1lbnRzLCBEMS1iZWZv
cmUtRDIgY29tcGF0aWJpbGl0eSBib3VuZGFyeSwgY3JlZGVudGlhbC1wb3N0dXJlIGludmFyaWFu
dHMgMeKAkzQsIGFuZCBhbGxvd2VkIHNpZGUtZWZmZWN0LWNoYW5uZWwgbGl0ZXJhbHMKKyoqU291
cmNlIHByb3ZlbmFuY2U6KiogZXh0cmFjdGVkIGJ5dGUtZm9yLWJ5dGUgZnJvbSBbYC4uLzA0LWNv
bnRyYWN0cy1hbmQtZ2F0ZXMubWQjOC13b3JsZHJ1bnRpbWVhZGFwdGVyZXhlY3V0aW9uZW52ZWxv
cGV2MWBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjOC13b3JsZHJ1bnRpbWVhZGFwdGVy
ZXhlY3V0aW9uZW52ZWxvcGV2MSksIGJhc2VsaW5lIGxpbmVzIDExMeKAkzE3NTsgdGhlIGV4YWN0
IDI3ODMtYnl0ZSBzb3VyY2UgYm9keSBpcyBwcmVzZXJ2ZWQgYmV0d2VlbiB0aGUgYm91bmRhcnkg
bWFya2VycyBiZWxvdworKipCYXNlbGluZSBzcGFuIFNIQS0yNTY6KiogYGNkYjc1NWFjMDBjZWE5
OWJlMDMyN2Y4NzU0N2ZkNTg2ZjFjNjMzNWM0NzAxMTNjZWEwMzMwN2NjOGUzZDhlYWNgCisKKzwh
LS0gZXhhY3QtZXh0cmFjdGVkLWJvZHk6c3RhcnQgLS0+CisjIyA4LiBgV29ybGRSdW50aW1lQWRh
cHRlckV4ZWN1dGlvbkVudmVsb3BlVjFgCisKK2BgYHJ1c3QKK3N0cnVjdCBXb3JsZFJ1bnRpbWVB
ZGFwdGVyRXhlY3V0aW9uRW52ZWxvcGVWMSB7CisgICAgc2NoZW1hX3ZlcnNpb246IHUzMiwKKyAg
ICBlbnZlbG9wZV9pZDogU3RyaW5nLAorICAgIGtpbmQ6IEFkYXB0ZXJFeGVjdXRpb25FbnZlbG9w
ZUtpbmRWMSwKKyAgICBydW50aW1lX2ZhbWlseTogU3RyaW5nLAorICAgIGd1ZXN0X2VudHJ5cG9p
bnQ6IFBhdGhCdWYsCisgICAgcnVudGltZV9kZXBlbmRlbmN5X3JlZjogUnVudGltZURlcGVuZGVu
Y3lSZWZWMSwKKyAgICBwcm9qZWN0ZWRfaG9tZTogT3B0aW9uPFBhdGhCdWY+LAorICAgIHdvcmtz
cGFjZV9yb290OiBQYXRoQnVmLAorICAgIHdvcmxkX2lkOiBTdHJpbmcsCisgICAgd29ybGRfZ2Vu
ZXJhdGlvbjogdTY0LAorICAgIHJldGFpbmVkX3BhcnRpY2lwYW50X2lkOiBPcHRpb248U3RyaW5n
PiwKKyAgICBhY3RpdmVfcnVuX2lkOiBPcHRpb248U3RyaW5nPiwKKyAgICBwb2xpY3lfc25hcHNo
b3RfcmVmOiBQb2xpY3lTbmFwc2hvdFJlZlYxLAorICAgIHBvbGljeV9zbmFwc2hvdF9oYXNoOiBT
dHJpbmcsCisgICAgZW52X3Byb2plY3Rpb25fcmVmOiBFbnZQcm9qZWN0aW9uUmVmVjEsCisgICAg
Y29uZmlnX3Byb2plY3Rpb25faWRlbnRpdHk6IENvbmZpZ1Byb2plY3Rpb25JZGVudGl0eVYxLAor
ICAgIGNvbmZpZ19wcm9qZWN0aW9uX3JlZjogQ29uZmlnUHJvamVjdGlvblJlZlYxLAorICAgIGlu
X3dvcmxkX2dhdGV3YXlfcmVmOiBPcHRpb248SW5Xb3JsZEdhdGV3YXlSZWZWMT4sCisgICAgY3Jl
ZGVudGlhbF9wb3N0dXJlOiBBZGFwdGVyQ3JlZGVudGlhbFBvc3R1cmVWMSwKKyAgICBtZWRpYXRp
b25fcG9zdHVyZTogQWRhcHRlck1lZGlhdGlvblBvc3R1cmVWMSwKKyAgICBjb21tYW5kX2Jyb2tl
cl9yZXF1aXJlZDogYm9vbCwKKyAgICBhbGxvd2VkX3NpZGVfZWZmZWN0X2NoYW5uZWxzOiBWZWM8
QnJva2VyZWRTaWRlRWZmZWN0Q2hhbm5lbFYxPiwKK30KKworZW51bSBBZGFwdGVyTWVkaWF0aW9u
UG9zdHVyZVYxIHsKKyAgICBCcm9rZXJSZXF1aXJlZCwKKyAgICBDb21wYXRpYmlsaXR5VW5wcm92
ZW4geyBjb21wYXRpYmlsaXR5X21vZGVfaWQ6IFN0cmluZyB9LAorfQorCitlbnVtIEFkYXB0ZXJD
cmVkZW50aWFsUG9zdHVyZVYxIHsKKyAgICBOb0NyZWRlbnRpYWxzUmVxdWlyZWQsCisgICAgU2Vj
dXJlR2F0ZXdheUhhbmRvZmYgeyBzZWNyZXRfaGFuZG9mZl9yZWY6IFNlY3JldEhhbmRvZmZSZWZW
MSB9LAorICAgIENvbXBhdGliaWxpdHlDb3B5QnJpZGdlIHsgY29tcGF0aWJpbGl0eV9tb2RlX2lk
OiBTdHJpbmcgfSwKK30KK2BgYAorCitFbnZlbG9wZSBraW5kcyBhcmUgYEhvc3RPcmNoZXN0cmF0
b3JgIGFuZCBgV29ybGRNZW1iZXJgLiBBIGBXb3JsZE1lbWJlcmAgZW52ZWxvcGUgcmVxdWlyZXMg
ZXhhY3Qgd29ybGQgYmluZGluZywgZ3Vlc3QtcmVhbGl6YWJsZSBlbnRyeXBvaW50LCBpbW11dGFi
bGUgcG9saWN5IHNuYXBzaG90LCBleHBsaWNpdCBjcmVkZW50aWFsIHBvc3R1cmUsIGFuZCBgY29t
bWFuZF9icm9rZXJfcmVxdWlyZWQ9dHJ1ZWAgZm9yIHNpZGUtZWZmZWN0LWNhcGFibGUgVUFBIHJ1
bnRpbWVzLgorCitEdXJpbmcgRDEtYmVmb3JlLUQyIHN0YWdpbmcsIGBDb21wYXRpYmlsaXR5VW5w
cm92ZW5gIG1heSBwcmVzZXJ2ZSBleHBsaWNpdGx5IG5hbWVkIGFuZCBsb2dnZWQgZXhpc3Rpbmcg
YmVoYXZpb3IsIGJ1dCBpdCBjYW5ub3QgY2xhaW0gU3Vic3RyYXRlIHBvbGljeSBtZWRpYXRpb24s
IGNhbm5vdCBzYXRpc2Z5IFVBQSBjYWdpbmcvYnJva2VyIGdhdGVzLCBhbmQgY2Fubm90IHByb21v
dGUgdGhpcyBzZWFtLiBgQnJva2VyUmVxdWlyZWRgIHJlcXVpcmVzIGBjb21tYW5kX2Jyb2tlcl9y
ZXF1aXJlZD10cnVlYCBhbmQgZmFpbHMgY2xvc2VkIHdoZW4gYW55IGRlY2xhcmVkIHNpZGUtZWZm
ZWN0IGNoYW5uZWwgbGFja3MgYnJva2VyIHN1cHBvcnQuCisKK0NyZWRlbnRpYWwtcG9zdHVyZSBp
bnZhcmlhbnRzOgorCisxLiBgU2VjdXJlR2F0ZXdheUhhbmRvZmZgIHJlcXVpcmVzIGFuIGV4YWN0
IGBpbl93b3JsZF9nYXRld2F5X3JlZmAgaW4gdGhlIHNhbWUgd29ybGQgZ2VuZXJhdGlvbiBhbmQg
YSB2YWxpZCBgTGF1bmNoVGltZVNlY3JldEhhbmRvZmZWMWAgcmVmLgorMi4gYE5vQ3JlZGVudGlh
bHNSZXF1aXJlZGAgcmVxdWlyZXMgbm8gc2VjcmV0LWhhbmRvZmYgcmVmIGFuZCBjYW5ub3QgbGF0
ZXIgZGlzY292ZXIgYW1iaWVudCBob3N0IGNyZWRlbnRpYWxzLgorMy4gYENvbXBhdGliaWxpdHlD
b3B5QnJpZGdlYCByZXF1aXJlcyBhIG5hbWVkL2xvZ2dlZCBjb21wYXRpYmlsaXR5IG1vZGUgYW5k
IGNhbm5vdCBzYXRpc2Z5IGNyZWRlbnRpYWwsIHByb2plY3Rpb24sIG9yIFVBQSBjb250cmFjdC1w
cm9tb3Rpb24gZ2F0ZXMuCis0LiBUaGUgVUFBIGNoaWxkIHJlY2VpdmVzIHRoZSBnYXRld2F5IGVu
ZHBvaW50L3Nlc3Npb24gY29udHJhY3QsIG5ldmVyIHRoZSByYXcgc2VjcmV0IEZEIG9yIGhvc3Qg
Y3JlZGVudGlhbCBwYXlsb2FkLgorCitBbGxvd2VkIGNoYW5uZWwgdmFsdWVzIGRlc2NyaWJlIGJy
b2tlciBzdXBwb3J0LCBub3QgcGVybWlzc2lvbiB0byBieXBhc3M6CisKK2BgYHRleHQKK1NoZWxs
Q29tbWFuZAorUGF0Y2hPckFwcGx5RWRpdAorRGlyZWN0RmlsZVdyaXRlCitNY3BPclRvb2xDYWxs
CitQcm9jZXNzU3Bhd24KK05ldHdvcmtPcGVyYXRpb24KK1Byb3ZpZGVyTmF0aXZlU2lkZUVmZmVj
dAorYGBgCisKK0FueSBzaWRlLWVmZmVjdCBjaGFubmVsIGFic2VudCBmcm9tIHRoZSBlbnZlbG9w
ZSBpcyBkaXNhYmxlZCBpbiB3b3JsZCBzY29wZS4KKworPCEtLSBleGFjdC1leHRyYWN0ZWQtYm9k
eTplbmQgLS0+CmRpZmYgLS1naXQgYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvY29u
dHJhY3RzL2xhdW5jaC10aW1lLXNlY3JldC1oYW5kb2ZmLXYxLm1kIGIvbGxtLWxhc3QtbWlsZS9y
dW50aW1lLXJlZmFjdG9yL2NvbnRyYWN0cy9sYXVuY2gtdGltZS1zZWNyZXQtaGFuZG9mZi12MS5t
ZApuZXcgZmlsZSBtb2RlIDEwMDY0NAppbmRleCAwMDAwMDAwMDAuLjhhNWI1ZWQ2NQotLS0gL2Rl
di9udWxsCisrKyBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9jb250cmFjdHMvbGF1
bmNoLXRpbWUtc2VjcmV0LWhhbmRvZmYtdjEubWQKQEAgLTAsMCArMSw5MSBAQAorKipLaW5kOioq
IGNvbnRyYWN0CisqKlN0YXR1czoqKiBjYW5vbmljYWwKKyoqQ2Fub25pY2FsIGZvcjoqKiBjb21w
bGV0ZSBleHRyYWN0ZWQgZXhpc3RpbmctY2Fycmllci12ZXJzdXMtcmVtYWluaW5nLWFkb3B0aW9u
IGJvdW5kYXJ5LCBgTGF1bmNoVGltZVNlY3JldEhhbmRvZmZWMWAgc2NoZW1hLCBkZWxpdmVyeSBh
bmQgc3RhdGUgZW51bXMsIGFsbG93ZWQgdHJhbnNpdGlvbnMsIHRlcm1pbmFsL3JldXNlIHJ1bGVz
LCBhbmQgaGFuZG9mZiBydWxlcyAx4oCTMTEgY292ZXJpbmcgbm9uLXNlY3JldCByZWZzLCBmYWls
LWNsb3NlZCBleGNsdXNpb25zLCBjb21wYXRpYmlsaXR5IGNvcGllZC1jcmVkZW50aWFsIHN0YXR1
cywgYW5kIHJldHJ5IHNlbWFudGljcworKipTb3VyY2UgcHJvdmVuYW5jZToqKiBleHRyYWN0ZWQg
Ynl0ZS1mb3ItYnl0ZSBmcm9tIFtgLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCM5LWxhdW5j
aHRpbWVzZWNyZXRoYW5kb2ZmdjFgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzktbGF1
bmNodGltZXNlY3JldGhhbmRvZmZ2MSksIGJhc2VsaW5lIGxpbmVzIDE3NuKAkzI1ODsgdGhlIGV4
YWN0IDQ5NjgtYnl0ZSBzb3VyY2UgYm9keSBpcyBwcmVzZXJ2ZWQgYmV0d2VlbiB0aGUgYm91bmRh
cnkgbWFya2VycyBiZWxvdworKipCYXNlbGluZSBzcGFuIFNIQS0yNTY6KiogYDBhNWExMzc5OTk2
OWYxNjVjM2FjY2EyMDBkZmQ1ZjI4YmYyZTc0Y2NiZmViNzEzNTYxNzZjYWVjZWM3OGIwY2RgCisK
KzwhLS0gZXhhY3QtZXh0cmFjdGVkLWJvZHk6c3RhcnQgLS0+CisjIyA5LiBgTGF1bmNoVGltZVNl
Y3JldEhhbmRvZmZWMWAKKworIyMjIEV4aXN0aW5nIGNhcnJpZXIgdmVyc3VzIHJlbWFpbmluZyBh
ZG9wdGlvbiB3b3JrCisKK1RoZSBjdXJyZW50IHJlcG8gYWxyZWFkeSBpbXBsZW1lbnRzIHRoZSBz
ZWN1cmUgY2FycmllciBtZWNoYW5pY3MgZm9yIHRoZSBtYW5hZ2VkIGluLXdvcmxkIGdhdGV3YXk6
IGBHYXRld2F5QXV0aEJ1bmRsZVYxYCwgYW4gaW5oZXJpdGVkIHBpcGUgcHJlcGFyZWQgYnkgYHdv
cmxkLXNlcnZpY2VgLCB0aGUgcG9pbnRlciBlbnZpcm9ubWVudCB2YXJpYWJsZSBgU1VCU1RSQVRF
X0xMTV9BVVRIX0JVTkRMRV9GRGAsIHJhdy1zZWNyZXQgZW52IHNjcnViYmluZywgYW5kIGdhdGV3
YXktc2lkZSBvbmUtdGltZSByZWFkIHBsdXMgdmFsaWRhdGlvbi4gRm9jdXNlZCBsYXVuY2hlciBh
bmQgY29uc3VtZXIgaW50ZWdyYXRpb24gdGVzdHMgbWFrZSB0aGlzIGEgcG9zaXRpdmUgbGFuZGVk
IHByaW1pdGl2ZSB0aGF0IGxhdGVyIHNsaWNlcyBtdXN0IHJldXNlIGFuZCBwcmVzZXJ2ZS4KKwor
YExhdW5jaFRpbWVTZWNyZXRIYW5kb2ZmVjFgIGFkZHMgdGhlIG9yY2hlc3RyYXRpb24tZmFjaW5n
IGlkZW50aXR5LCBsaWZlY3ljbGUsIGFuZCBub24tc2VjcmV0IGV2aWRlbmNlIG5lZWRlZCB0byBq
b2luIHRoYXQgY2FycmllciB0byBhbiBleGFjdCB3b3JsZCBnZW5lcmF0aW9uLCBlbnZlbG9wZSwg
cmV0YWluZWQgcGFydGljaXBhbnQsIGFuZCBnYXRld2F5IHJlY2VpdmVyLiBUaGUgYWJzZW5jZSBv
ZiB0aGlzIGNvbXBsZXRlIGR1cmFibGUgcmVjb3JkIGRvZXMgbm90IG1lYW4gdGhlIEZEIGNhcnJp
ZXIgaXRzZWxmIGlzIGFic2VudC4KKworUmVtYWluaW5nIGFkb3B0aW9uIHdvcmsgaXMgdG86CisK
KzEuIGV4cG9zZSBvciBwZXJzaXN0IHRoZSBub24tc2VjcmV0IGhhbmRvZmYgcmVmZXJlbmNlL3N0
YXRlIHJlcXVpcmVkIGJ5IHRoZSBlbnZlbG9wZSB3aXRob3V0IHBlcnNpc3Rpbmcgc2VjcmV0IHBh
eWxvYWRzOworMi4gcG9pbnQgZGlyZWN0IHdvcmxkIENvZGV4L1VBQSBwcm92aWRlciB0cmFmZmlj
IGF0IHRoZSBleGFjdCBtYW5hZ2VkIGdhdGV3YXkgdGhhdCBjb25zdW1lZCB0aGUgaGFuZG9mZjsK
KzMuIGNvbnN0cnVjdCBwZXItd29ya2VyIHJ1bnRpbWUtbmF0aXZlIGNvbmZpZyBmcm9tIFN1YnN0
cmF0ZSBsb2dpY2FsIGNvbmZpZyBwbHVzIGFjY2VwdGVkIHBvbGljeSBpbnN0ZWFkIG9mIGNvcGll
ZCBob3N0IGNvbmZpZy9hdXRoOyBhbmQKKzQuIHByb3ZlIHRoZSBjb21wbGV0ZSBqb2luZWQgcGF0
aCB3aXRoIHByb2R1Y3Rpb24tcGF0aCBzbW9rZS9lMmUuCisKK0RvIG5vdCByZXBsYWNlIHRoZSBl
eGlzdGluZyBjYXJyaWVyIG1lcmVseSB0byBtYWtlIGl0cyBpbXBsZW1lbnRhdGlvbiBuYW1lcyBy
ZXNlbWJsZSB0aGlzIGNvbnRyb2wtcGxhbmUgY29udHJhY3QuIEV4dGVuZCBvciBhZGFwdCBpdCBv
bmx5IHdoZXJlIG9uZSBvZiB0aGUgaWRlbnRpdHksIGV2aWRlbmNlLCBmYWlsLWNsb3NlZCwgb3Ig
YWRvcHRpb24gcmVxdWlyZW1lbnRzIGlzIGdlbnVpbmVseSBtaXNzaW5nLgorCitgYGBydXN0Citz
dHJ1Y3QgTGF1bmNoVGltZVNlY3JldEhhbmRvZmZWMSB7CisgICAgc2NoZW1hX3ZlcnNpb246IHUz
MiwgICAgICAgICAgICAgICAgIC8vIGV4YWN0bHkgMQorICAgIGhhbmRvZmZfaWQ6IFN0cmluZywK
KyAgICBvcmNoZXN0cmF0aW9uX3Nlc3Npb25faWQ6IFN0cmluZywKKyAgICB3b3JsZF9pZDogU3Ry
aW5nLAorICAgIHdvcmxkX2dlbmVyYXRpb246IHU2NCwKKyAgICByZXRhaW5lZF9wYXJ0aWNpcGFu
dF9pZDogT3B0aW9uPFN0cmluZz4sCisgICAgcnVudGltZV9mYW1pbHk6IFN0cmluZywKKworICAg
IC8vIE5vbi1zZWNyZXQgYXV0aG9yaXR5IHJlZmVyZW5jZXMgb25seS4KKyAgICBjcmVkZW50aWFs
X3NvdXJjZV9yZWY6IENyZWRlbnRpYWxTb3VyY2VSZWZWMSwKKyAgICByZWNlaXZpbmdfZ2F0ZXdh
eV9yZWY6IEluV29ybGRHYXRld2F5UmVmVjEsCisgICAgZGVsaXZlcnk6IFNlY3JldERlbGl2ZXJ5
TWVjaGFuaXNtVjEsCisKKyAgICBjcmVhdGVkX2F0OiBUaW1lc3RhbXAsCisgICAgZGVsaXZlcmVk
X2F0OiBPcHRpb248VGltZXN0YW1wPiwKKyAgICBjb25zdW1lZF9hdDogT3B0aW9uPFRpbWVzdGFt
cD4sCisgICAgZXhwaXJlc19hdDogVGltZXN0YW1wLAorICAgIHN0YXRlX3JldmlzaW9uOiB1NjQs
CisgICAgc3RhdGU6IFNlY3JldEhhbmRvZmZTdGF0ZVYxLAorICAgIGZhaWx1cmVfZGlhZ25vc3Rp
Y19yZWY6IE9wdGlvbjxSZWRhY3RlZERpYWdub3N0aWNSZWZWMT4sCit9CisKK2VudW0gU2VjcmV0
RGVsaXZlcnlNZWNoYW5pc21WMSB7CisgICAgU2VjdXJlRmQgeworICAgICAgICBmZF9uYW1lOiBT
dHJpbmcsCisgICAgICAgIG9uZV90aW1lOiBib29sLAorICAgICAgICBnYXRld2F5X3JlY2VpdmVy
X29ubHk6IGJvb2wsCisgICAgICAgIGRlbnlfY2hpbGRfaW5oZXJpdGFuY2U6IGJvb2wsCisgICAg
ICAgIGNsb3NlX2FmdGVyX2NvbnN1bWU6IGJvb2wsCisgICAgfSwKK30KKworZW51bSBTZWNyZXRI
YW5kb2ZmU3RhdGVWMSB7CisgICAgUHJlcGFyZWQsCisgICAgRGVsaXZlcmVkLAorICAgIENvbnN1
bWVkLAorICAgIEZhaWxlZCwKKyAgICBFeHBpcmVkLAorfQorYGBgCisKK0FsbG93ZWQgdHJhbnNp
dGlvbnM6CisKK2BgYHRleHQKK1ByZXBhcmVkIC0+IERlbGl2ZXJlZCAtPiBDb25zdW1lZAorUHJl
cGFyZWR8RGVsaXZlcmVkIC0+IEZhaWxlZHxFeHBpcmVkCitgYGAKKworYENvbnN1bWVkYCwgYEZh
aWxlZGAsIGFuZCBgRXhwaXJlZGAgYXJlIHRlcm1pbmFsLiBSZXVzZSByZXF1aXJlcyBhIG5ldyBg
aGFuZG9mZl9pZGAgYW5kIG5ldyBkZXNjcmlwdG9yLgorCitMYXVuY2gtdGltZSBzZWNyZXQgaGFu
ZG9mZiBydWxlczoKKworMS4gU2VjcmV0IG1hdGVyaWFsIGlzIHJlc29sdmVkIGJ5IGhvc3QgY3Jl
ZGVudGlhbCBhdXRob3JpdHkgYW5kIG11c3Qgbm90IGJlIHBlcnNpc3RlZCBpbiBTdWJzdHJhdGUg
cmVjb3JkcywgcnVudGltZS1uYXRpdmUgY29uZmlnLCB3b3Jrc3BhY2Ugb3ZlcmxheXMsIG1hbmlm
ZXN0cywgdHJhY2VzLCBvciBsb2dzLgorMi4gYGNyZWRlbnRpYWxfc291cmNlX3JlZmAgaXMgYW4g
b3BhcXVlIGhvc3QtYXV0aG9yaXR5IHJlZmVyZW5jZSwgbm90IGEgaG9zdCBmaWxlc3lzdGVtIHBh
dGgsIGNyZWRlbnRpYWwtc3RvcmUgbG9jYXRvciBleHBvc2VkIHRvIHRoZSB3b3JsZCwgb3IgZGln
ZXN0IG9mIHRoZSBzZWNyZXQgcGF5bG9hZC4gYGZkX25hbWVgIGlzIGEgbm9uLXNlY3JldCBsb2dp
Y2FsIGRlc2NyaXB0b3IgbGFiZWwuCiszLiBDb250cmFjdC1jb3JyZWN0IHdvcmxkIGV4ZWN1dGlv
biBtdXN0IG5vdCBjb3B5IGhvc3QgY3JlZGVudGlhbCBmaWxlcyBvciBzZWNyZXQtYmVhcmluZyBo
b3N0IGNvbmZpZyBpbnRvIHdvcmxkLXZpc2libGUgYENPREVYX0hPTUVgLCBgLmNvZGV4YCwgYGNv
bmZpZy50b21sYCwgYXV0aCBmaWxlcywgb3IgZXF1aXZhbGVudCBydW50aW1lIGhvbWVzLgorNC4g
QSBib3VuZGVkIG5vbi1zZWNyZXQgcnVudGltZSBjb25maWcgbWF5IGJlIHJlbmRlcmVkIGZyb20g
U3Vic3RyYXRlLW93bmVkIGxvZ2ljYWwgaW52ZW50b3J5LiBDb3B5aW5nIGEgaG9zdCBgY29uZmln
LnRvbWxgIGFzIGF1dGhvcml0eSBpcyBjb21wYXRpYmlsaXR5IGJyaWRnaW5nLCBub3QgcHJvamVj
dGlvbiBhdXRob3JpdHkuCis1LiBWMSB2YWxpZGF0aW9uIGFjY2VwdHMgYFNlY3VyZUZkYCBvbmx5
IHdoZW4gYG9uZV90aW1lYCwgYGdhdGV3YXlfcmVjZWl2ZXJfb25seWAsIGBkZW55X2NoaWxkX2lu
aGVyaXRhbmNlYCwgYW5kIGBjbG9zZV9hZnRlcl9jb25zdW1lYCBhcmUgYWxsIGB0cnVlYC4KKzYu
IFRoZSBzZWN1cmUgRkQgaXMgc2NvcGVkIHRvIHRoZSBleGFjdCBgcmVjZWl2aW5nX2dhdGV3YXlf
cmVmYCwgY29uc3VtZWQgYnkgdGhlIGluLXdvcmxkIFN1YnN0cmF0ZSBnYXRld2F5IGF0IHdvcmxk
IGxhdW5jaCwgY2xvc2VkIGFmdGVyIGNvbnN1bXB0aW9uLCBhbmQgbmV2ZXIgaW5oZXJpdGVkIGJ5
IHRoZSBVQUEgYWRhcHRlciBvciBpdHMgY2hpbGRyZW4uCis3LiBUaGUgZ2F0ZXdheeKAlG5vdCBD
b2RleC9VQUHigJRvd25zIGNyZWRlbnRpYWwgYXBwbGljYXRpb24sIGdhdGV3YXkgc2Vzc2lvbiBt
YXRlcmlhbCwgYW5kIHVwc3RyZWFtIHByb3ZpZGVyIGZvcndhcmRpbmcuIFRoZSBVQUEgdGFsa3Mg
dG8gdGhlIGdhdGV3YXkgdGhyb3VnaCB0aGUgZW52ZWxvcGUncyBlbmRwb2ludC9zZXNzaW9uIGNv
bnRyYWN0LgorOC4gTG9ncywgcmVjZWlwdHMsIHRyYWNlcywgYW5kIG1hbmlmZXN0cyBtYXkgY29u
dGFpbiBoYW5kb2ZmIElELCBub24tc2VjcmV0IHJlZnMsIHN0YXRlLCB0aW1lc3RhbXBzLCBhbmQg
cmVkYWN0ZWQgZGlhZ25vc3RpY3MuIFRoZXkgbXVzdCBub3QgY29udGFpbiBzZWNyZXQgcGF5bG9h
ZHMsIHNlY3JldC1iZWFyaW5nIGZpbGUgcGF0aHMsIG9yIHJldXNhYmxlIGhhc2hlcy9maW5nZXJw
cmludHMgZGVyaXZlZCBmcm9tIHRoZSBzZWNyZXQgcGF5bG9hZC4KKzkuIEZhaWx1cmUsIGV4cGly
eSwgcmVjZWl2ZXIgbWlzbWF0Y2gsIHdvcmxkLWdlbmVyYXRpb24gbWlzbWF0Y2gsIGR1cGxpY2F0
ZSBjb25zdW1wdGlvbiwgb3IgZGVzY3JpcHRvciBpbmhlcml0YW5jZSByaXNrIGZhaWxzIGNsb3Nl
ZCBmb3IgY3JlZGVudGlhbC1yZXF1aXJpbmcgd29ybGQgYWRhcHRlcnMuCisxMC4gQSBjb21wYXRp
YmlsaXR5IGNvcGllZC1jcmVkZW50aWFsIG1vZGUgaXMgdGVtcG9yYXJ5LCBleHBsaWNpdGx5IG5h
bWVkIGFuZCBsb2dnZWQsIGhhcyByZXRpcmVtZW50IGNyaXRlcmlhLCBhbmQgY2Fubm90IHNhdGlz
ZnkgYENvbnRyYWN0Q29ycmVjdEFuZFByb3ZlbmAgb3IgYW55IHNlY3VyZS1oYW5kb2ZmIGFjY2Vw
dGFuY2UgZ2F0ZS4KKzExLiBGYWlsZWQvZXhwaXJlZCBoYW5kb2ZmcyBjbG9zZSB0aGUgZGVzY3Jp
cHRvciBhbmQgY2xlYXIgdHJhbnNpZW50IGJ1ZmZlcnMgYmVmb3JlIHJldHJ5OyByZXRyeSBjcmVh
dGVzIGEgbmV3IGhhbmRvZmYgcmF0aGVyIHRoYW4gcmVvcGVuaW5nIG9yIHJlcGxheWluZyB0aGUg
b2xkIHBheWxvYWQuCisKKzwhLS0gZXhhY3QtZXh0cmFjdGVkLWJvZHk6ZW5kIC0tPgpgYGAK
```

</details>

## Preserved review answer

<details>
<summary>Initial-range review answer (rendered copy)</summary>

``````text
No qualifying P1/P2 findings.

I reviewed the complete bounded five-path patch and both new canonical files. The two extracted bodies are byte-identical to their deleted root spans: WorldRuntimeAdapterExecutionEnvelopeV1 is exactly 2,783 bytes with SHA-256 cdb755...8eac, and LaunchTimeSecretHandoffV1 is exactly 4,968 bytes with SHA-256 0a5a13...b0cd, matching the declared provenance.

The first owner preserves the envelope schema/order, D1-before-D2 CompatibilityUnproven non-promotion boundary, BrokerRequired fail-closed rule, credential-posture invariants 1–4, ordered side-effect-channel literals, and absent-channel disabling.

The second preserves the positive existing-carrier versus incomplete-adoption distinction, all four remaining adoption steps, no-cosmetic-reimplementation requirement, schema/enums/transitions, terminal/reuse semantics, and secret-handoff rules 1–11 including the required exclusions and fail-closed behavior.

The legacy V1 root headings remain compatibility stubs pointing to the corresponding canonical owners, while the index adds one canonical row per contract.

The two ledger entries record the exact source spans/hashes, explicit paired-family atomicity, carrier/adoption boundary, and a rollback that restores both bodies and removes both owners/index rows/ledger rows together.

I found no reachable, patch-causal defect that materially alters contract semantics or ownership, weakens credential/secret/fail-closed constraints, breaks a required anchor/link or provenance record, makes rollback non-atomic, breaches the five-path fence, or overclaims implementation/adoption/gate authority. The supplied validation likewise reports the exact fence, zero Markdown link/anchor failures, exact body hashes, and successful forward/reverse application.

VERDICT: APPROVED
``````

</details>

<details>
<summary>Initial-range review answer exact bytes (base64 UTF-8)</summary>

```text
Tm8gcXVhbGlmeWluZyBQMS9QMiBmaW5kaW5ncy4KCkkgcmV2aWV3ZWQgdGhlIGNvbXBsZXRlIGJv
dW5kZWQgZml2ZS1wYXRoIHBhdGNoIGFuZCBib3RoIG5ldyBjYW5vbmljYWwgZmlsZXMuIFRoZSB0
d28gZXh0cmFjdGVkIGJvZGllcyBhcmUgYnl0ZS1pZGVudGljYWwgdG8gdGhlaXIgZGVsZXRlZCBy
b290IHNwYW5zOiBXb3JsZFJ1bnRpbWVBZGFwdGVyRXhlY3V0aW9uRW52ZWxvcGVWMSBpcyBleGFj
dGx5IDIsNzgzIGJ5dGVzIHdpdGggU0hBLTI1NiBjZGI3NTUuLi44ZWFjLCBhbmQgTGF1bmNoVGlt
ZVNlY3JldEhhbmRvZmZWMSBpcyBleGFjdGx5IDQsOTY4IGJ5dGVzIHdpdGggU0hBLTI1NiAwYTVh
MTMuLi5iMGNkLCBtYXRjaGluZyB0aGUgZGVjbGFyZWQgcHJvdmVuYW5jZS4KClRoZSBmaXJzdCBv
d25lciBwcmVzZXJ2ZXMgdGhlIGVudmVsb3BlIHNjaGVtYS9vcmRlciwgRDEtYmVmb3JlLUQyIENv
bXBhdGliaWxpdHlVbnByb3ZlbiBub24tcHJvbW90aW9uIGJvdW5kYXJ5LCBCcm9rZXJSZXF1aXJl
ZCBmYWlsLWNsb3NlZCBydWxlLCBjcmVkZW50aWFsLXBvc3R1cmUgaW52YXJpYW50cyAx4oCTNCwg
b3JkZXJlZCBzaWRlLWVmZmVjdC1jaGFubmVsIGxpdGVyYWxzLCBhbmQgYWJzZW50LWNoYW5uZWwg
ZGlzYWJsaW5nLgoKVGhlIHNlY29uZCBwcmVzZXJ2ZXMgdGhlIHBvc2l0aXZlIGV4aXN0aW5nLWNh
cnJpZXIgdmVyc3VzIGluY29tcGxldGUtYWRvcHRpb24gZGlzdGluY3Rpb24sIGFsbCBmb3VyIHJl
bWFpbmluZyBhZG9wdGlvbiBzdGVwcywgbm8tY29zbWV0aWMtcmVpbXBsZW1lbnRhdGlvbiByZXF1
aXJlbWVudCwgc2NoZW1hL2VudW1zL3RyYW5zaXRpb25zLCB0ZXJtaW5hbC9yZXVzZSBzZW1hbnRp
Y3MsIGFuZCBzZWNyZXQtaGFuZG9mZiBydWxlcyAx4oCTMTEgaW5jbHVkaW5nIHRoZSByZXF1aXJl
ZCBleGNsdXNpb25zIGFuZCBmYWlsLWNsb3NlZCBiZWhhdmlvci4KClRoZSBsZWdhY3kgVjEgcm9v
dCBoZWFkaW5ncyByZW1haW4gY29tcGF0aWJpbGl0eSBzdHVicyBwb2ludGluZyB0byB0aGUgY29y
cmVzcG9uZGluZyBjYW5vbmljYWwgb3duZXJzLCB3aGlsZSB0aGUgaW5kZXggYWRkcyBvbmUgY2Fu
b25pY2FsIHJvdyBwZXIgY29udHJhY3QuCgpUaGUgdHdvIGxlZGdlciBlbnRyaWVzIHJlY29yZCB0
aGUgZXhhY3Qgc291cmNlIHNwYW5zL2hhc2hlcywgZXhwbGljaXQgcGFpcmVkLWZhbWlseSBhdG9t
aWNpdHksIGNhcnJpZXIvYWRvcHRpb24gYm91bmRhcnksIGFuZCBhIHJvbGxiYWNrIHRoYXQgcmVz
dG9yZXMgYm90aCBib2RpZXMgYW5kIHJlbW92ZXMgYm90aCBvd25lcnMvaW5kZXggcm93cy9sZWRn
ZXIgcm93cyB0b2dldGhlci4KCkkgZm91bmQgbm8gcmVhY2hhYmxlLCBwYXRjaC1jYXVzYWwgZGVm
ZWN0IHRoYXQgbWF0ZXJpYWxseSBhbHRlcnMgY29udHJhY3Qgc2VtYW50aWNzIG9yIG93bmVyc2hp
cCwgd2Vha2VucyBjcmVkZW50aWFsL3NlY3JldC9mYWlsLWNsb3NlZCBjb25zdHJhaW50cywgYnJl
YWtzIGEgcmVxdWlyZWQgYW5jaG9yL2xpbmsgb3IgcHJvdmVuYW5jZSByZWNvcmQsIG1ha2VzIHJv
bGxiYWNrIG5vbi1hdG9taWMsIGJyZWFjaGVzIHRoZSBmaXZlLXBhdGggZmVuY2UsIG9yIG92ZXJj
bGFpbXMgaW1wbGVtZW50YXRpb24vYWRvcHRpb24vZ2F0ZSBhdXRob3JpdHkuIFRoZSBzdXBwbGll
ZCB2YWxpZGF0aW9uIGxpa2V3aXNlIHJlcG9ydHMgdGhlIGV4YWN0IGZlbmNlLCB6ZXJvIE1hcmtk
b3duIGxpbmsvYW5jaG9yIGZhaWx1cmVzLCBleGFjdCBib2R5IGhhc2hlcywgYW5kIHN1Y2Nlc3Nm
dWwgZm9yd2FyZC9yZXZlcnNlIGFwcGxpY2F0aW9uLgoKVkVSRElDVDogQVBQUk9WRUQK
```

</details>
