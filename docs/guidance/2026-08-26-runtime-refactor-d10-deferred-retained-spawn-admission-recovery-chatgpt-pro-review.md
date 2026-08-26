# ChatGPT Pro advisory review: runtime-refactor D10 Deferred retained-spawn admission recovery contract

- Date: 2026-08-26
- Bound baseline commit/tree: `6d9cac4d2943d7aa5ef5825cc099492d166e5f4a` / `1902a8a42ae87dff07b2f0a5fa5d741bdfa186e1`
- Candidate implementation subagent (`gpt-5.4`, Extra High): `/root/d10_deferred_admission_recovery_landing`
- Root/orchestrator review owner: `/root`
- Independent review chat: https://chatgpt.com/c/6a8f2aef-1a64-83ea-8aac-4b599f6be973
- Review mode: `initial-range`
- Review context: independent fresh conversation
- ChatGPT account surface: `Pro`
- Visible reasoning-effort control: `Extra High`
- Exact model label: not exposed by the current visible ChatGPT UI
- Candidate patch: `sha256:8dd20ed56bdd05256217317d9578ab49f3a90b7775f340addf6a2bd9e3277115` at `/private/tmp/d10-deferred-retained-spawn-admission-recovery-candidate.patch`
- Review prompt: `sha256:de4efabd18030182e874eb3b51f058da958addbdc152a9eefb17be25e7540bb1` at `/private/tmp/d10-deferred-retained-spawn-admission-recovery-chatgpt-pro-initial-review-prompt.txt`
- Review answer: `sha256:3b430e3d0a3a269cbf26815495b7885ee23d918ec5e8506f4eb3e545efdb6439` at `/private/tmp/d10-deferred-retained-spawn-admission-recovery-chatgpt-pro-initial-review-answer.txt`
- Imported validation log: `sha256:541dcd821ec5cc1000f8d76506d95feb7ca63f664cbefb58662e807624c2b78d` at `/private/tmp/d10-deferred-retained-spawn-admission-recovery-validation.log`
- Exact embedded source body: `9896 bytes`, `sha256:d903299bdd345316fd0943d28abc300a1cef001ce0744746f72f53607eb64c12`
- Review verdict: `APPROVED`
- Review finding summary: `No qualifying P1/P2 findings.`
- Remediation rounds: `0`
- Commit posture: candidate plus this closeout will be committed together atomically after local validation; not yet committed
- Push posture: `not pushed`

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Candidate scope and outcome

This independently reviewed substantive D10 unit extracts the full `Deferred retained-spawn admission recovery contract` into `llm-last-mile/runtime-refactor/contracts/deferred-retained-spawn-admission-recovery-contract.md`, replaces the root span in `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` with a shallow compatibility pointer, adds the single truthful index row, and adds the matching D10 extraction-ledger row. `Deferred retained-spawn admission recovery contract` is complete locally and approved after initial-range review with zero remediation. D10 remains explicitly in progress and incomplete; the next substantive D10 unit is `ActiveEphemeralTaskReceiptV1`; later D10 units remain queued; D11 remains blocked until D10 is fully complete and separately authorized; D12 remains blocked by D11 and is separately unauthorized; no successor dispatch authority is granted.

## Candidate manifest and digests

Manifest for the independently reviewed candidate state:

- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`: `sha256:18aaa354871c8e4569f0ad17610f203ac7a23f96b72685b560ef87f4da77ba54`
- `llm-last-mile/runtime-refactor/contracts/deferred-retained-spawn-admission-recovery-contract.md`: `sha256:06fd68735582d0ddee76a14602b99ead6a6df89affbd1036d4f36613ebd0d9c2`
- `llm-last-mile/runtime-refactor/index/README.md`: `sha256:90899d57ec4bfcb1f70a93010a1bb6df646d312055f3175ad1cad7b489099759`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`: `sha256:281776f8e5a281501b193f38b0239cfa24e1cd1ff68ad43499057a462bb95bc5`

Imported review artifacts:

- Candidate patch: `sha256:8dd20ed56bdd05256217317d9578ab49f3a90b7775f340addf6a2bd9e3277115`
- Review prompt: `sha256:de4efabd18030182e874eb3b51f058da958addbdc152a9eefb17be25e7540bb1`
- Review answer: `sha256:3b430e3d0a3a269cbf26815495b7885ee23d918ec5e8506f4eb3e545efdb6439`
- Imported validation log: `sha256:541dcd821ec5cc1000f8d76506d95feb7ca63f664cbefb58662e807624c2b78d`
- Exact embedded source body: `9896 bytes`, `sha256:d903299bdd345316fd0943d28abc300a1cef001ce0744746f72f53607eb64c12`

## Exact source-body proof

- Baseline `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` lines `71–213` are exactly `9896` bytes with SHA-256 `d903299bdd345316fd0943d28abc300a1cef001ce0744746f72f53607eb64c12`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` in `llm-last-mile/runtime-refactor/contracts/deferred-retained-spawn-admission-recovery-contract.md` are also exactly `9896` bytes with SHA-256 `d903299bdd345316fd0943d28abc300a1cef001ce0744746f72f53607eb64c12`.
- The baseline source span and the owner-body bytes are byte-identical, including the final blank separator before the end marker.

## Validation evidence

- The supplied review answer states `No qualifying P1/P2 findings.` and ends with `VERDICT: APPROVED`.
- The candidate patch changes exactly the four imported candidate paths and no others.
- Imported candidate digests, candidate-patch SHA-256, prompt SHA-256, answer SHA-256, validation-log SHA-256, and exact source-body SHA-256 were all reverified during this closeout.
- The rendered prompt and rendered answer below are whitespace-clean: every trailing literal space from the imported UTF-8 files is rendered as `&#32;` so this repository-local Markdown artifact stays clean under `git diff --check` while the base64 blocks preserve the exact bytes.
- The rendered prompt and rendered answer were both revalidated against the imported files, and the embedded base64 blocks decode exactly back to the imported UTF-8 bytes without silently correcting the captured answer whitespace.
- Final changed-path fence including untracked files is exactly these six paths:
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/contracts/deferred-retained-spawn-admission-recovery-contract.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `docs/guidance/2026-08-26-runtime-refactor-d10-deferred-retained-spawn-admission-recovery-chatgpt-pro-review.md`
- `git diff --check` passed for the final closeout state.
- `git diff --no-index --check /dev/null docs/guidance/2026-08-26-runtime-refactor-d10-deferred-retained-spawn-admission-recovery-chatgpt-pro-review.md` reported no whitespace diagnostics.
- Tracker truth validation passed for only `Deferred retained-spawn admission recovery contract` approved, D10 still incomplete, next substantive D10 unit `ActiveEphemeralTaskReceiptV1`, later D10 units queued, D11 blocked, D12 separately unauthorized, and no successor dispatch authority.
- Complete combined candidate+closeout patch was generated at `/private/tmp/d10-deferred-retained-spawn-admission-recovery-closeout.patch` with a SHA-256 sidecar, and forward/reverse apply against disposable clean baseline exports both passed.
- No commit, stage, or push is performed during this closeout.

## Final verdict

`Deferred retained-spawn admission recovery contract` is complete locally and **APPROVED** after initial-range review with zero remediation. D10 remains explicitly in progress and incomplete; the next substantive D10 unit is `ActiveEphemeralTaskReceiptV1`; later D10 units remain queued; D11 remains blocked until D10 is fully complete and separately authorized; D12 remains blocked by D11 and is separately unauthorized; no successor dispatch authority is granted.

## Preserved review prompt

<details>
<summary>Initial-range review prompt (rendered copy)</summary>

<pre>
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: exact baseline commit `6d9cac4d2943d7aa5ef5825cc099492d166e5f4a`, baseline tree `1902a8a42ae87dff07b2f0a5fa5d741bdfa186e1`, plus the complete content-addressed uncommitted patch below. Complete patch SHA-256: `8dd20ed56bdd05256217317d9578ab49f3a90b7775f340addf6a2bd9e3277115`.
Review target: the complete bounded D10 extraction of the entire large compound `Deferred retained-spawn admission recovery contract`, including the untracked owner. Review all four files and do not split or reduce the compound unit.

Supported inputs and behavior: repository GitHub-style Markdown links/fragments; one root legacy heading/pointer; one canonical compound Markdown owner; one index row; one extraction-ledger row. Non-rendering HTML boundary comments delimit the exact canonical source-body bytes. Rust/text content is literal documentation. The source-body bytes include durable recovery preconditions, semantic outcomes, two paired V1 schemas, HMAC/carrier equality rules, compatibility `None` versus authority-managed `Some` boundaries, exact route exclusions, fixture/production-symbol allowlists, failed-closed rules, and recorded-result limitations.
Supported platforms and dialects: repository Markdown only under the local GitHub-style fragment validator. No Rust compilation, runtime, script, platform, external renderer, D11, or D12 behavior is supported/in scope.

In-scope invariants:
1. The complete compound unit remains inseparable and has exactly one canonical owner: `contracts/deferred-retained-spawn-admission-recovery-contract.md`.
2. Root retains exactly one legacy heading/anchor and replaces only its substantive body with a shallow pointer; following `B3.2a-WA ExactBoundWorldOwnershipAdoptionV1` and preceding projections remain unchanged.
3. Baseline source is exactly lines 71–213 inclusive, 9896 bytes, SHA-256 `d903299bdd345316fd0943d28abc300a1cef001ce0744746f72f53607eb64c12`. Bytes strictly between owner markers match it exactly, including the final blank separator.
4. Every heading, paragraph, line wrap, blank line, Rust fence, ordered field, stable V1 name, `hmac-sha-256` literal, filename, function/symbol name, allowlist, route exclusion, `Some`/`None` rule, negative requirement, failed/blocked/remaining semantic status, authority boundary, validation/production-path sequence, and literal hash such as `d0a70727c2bec2b2d6fe0754ea469c4682684dda` is exact by byte identity.
5. `RetainedWorkerAdmissionCommitmentCarrierV1` and `RetainedWorkerLaunchAuthorityProofV1` remain paired in this compound owner; D6 packet references are unchanged and not misrepresented as second canonical definitions.
6. Index adds exactly one truthful contract row with unchanged unrelated row order.
7. Ledger adds exactly one truthful D10 row with source locator `lines 71–213`, source digest, correct anchors/destination/representation/delta, and atomic rollback.
8. Rollback restores the exact 9896-byte span, removes only the owner and exact index/ledger rows, and preserves HostExecutionEpisodeV1, the two prerequisite anchors, D3/D5–D9 owners, `index/current.md`, `review-control/`, later D10 units, and all other paths.
9. All runtime-refactor repository-relative Markdown links/anchors resolve.
10. The patch does not implement runtime behavior, broaden mechanical exceptions/allowlists, convert evidence into gate satisfaction, grant authority, approve successors/later D10, begin D11, or perform D12.
11. Exact changed-path manifest including untracked is:
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` SHA-256 `18aaa354871c8e4569f0ad17610f203ac7a23f96b72685b560ef87f4da77ba54`
- `llm-last-mile/runtime-refactor/contracts/deferred-retained-spawn-admission-recovery-contract.md` SHA-256 `06fd68735582d0ddee76a14602b99ead6a6df89affbd1036d4f36613ebd0d9c2`
- `llm-last-mile/runtime-refactor/index/README.md` SHA-256 `90899d57ec4bfcb1f70a93010a1bb6df646d312055f3175ad1cad7b489099759`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md` SHA-256 `281776f8e5a281501b193f38b0239cfa24e1cd1ff68ad43499057a462bb95bc5`

Required material consequence: blocking requires a reachable broken supported link/anchor; lost/reordered/changed source byte or compound relationship; altered named version, field, literal, code fence, allowlist/exclusion, negative/fail-closed rule, status/authority boundary, hash, provenance or rollback; duplicate/missing canonical owner; or path/scope violation caused by this patch.
Blocking threshold: only reachable material P1/P2 defects introduced by this patch. Unchanged pre-existing behavior and stylistic preferences are non-blocking.
Accepted prior findings: none.
Deferred or out-of-scope concerns: prior D3/D5–D9 owners, packet-family material, HostExecutionEpisodeV1, following B3.2a-WA, all later D10 units, D11, D12, runtime/Rust/scripts/platform work, roadmap/ZIP, review governance, Markdown normalization, unrelated pre-existing issues.
Project sources: this prompt contains the exact baseline, four-file manifest, complete patch, body digest, and validation evidence. Excluded source is unavailable and must not be inferred. Read the complete patch before judging the result.

Validation evidence:
- four-path inventory including untracked and no staged files: pass.
- `git diff --check`: pass.
- new owner no-index whitespace check: no diagnostics (expected nonzero diff status only).
- exact body extraction: 9896 bytes, SHA-256 `d903299bdd345316fd0943d28abc300a1cef001ce0744746f72f53607eb64c12`, byte-equal to source.
- explicit paired-schema/ordered-field/fence/literal/path/symbol/allowlist/negative-exclusion/Some-None/hash/status/authority checks: pass.
- Markdown validator: `LINKS_CHECKED 1819 FAILURES 0`.
- root/destination anchors unique; root/index/ledger links resolve.
- prior owner/review-artifact stability checks: pass.
- complete patch forward/reverse apply against disposable clean exports: pass.
- validation log SHA-256 `541dcd821ec5cc1000f8d76506d95feb7ca63f664cbefb58662e807624c2b78d`.
- complete patch SHA-256 `8dd20ed56bdd05256217317d9578ab49f3a90b7775f340addf6a2bd9e3277115`.

Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return findings first by severity, including file/heading, reachability, violated invariant, material consequence, and patch causality. State explicitly when none qualify. End with exactly `VERDICT: APPROVED` or `VERDICT: CHANGES REQUIRED`.

Complete patch (UTF-8, includes all untracked files):
```diff
diff --git a/llm-last-mile/runtime-refactor/04-contracts-and-gates.md b/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
index a9cdf586b..7b80bac59 100644
--- a/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
+++ b/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
@@ -70,146 +70,7 @@ Canonical content: [`b1-b2-1/contracts-and-gates.md#b1b21-bounded-read-only-disp
&#32;
 ### Deferred retained-spawn admission recovery contract
&#32;
-B3.2a deliberately stops at conservative exact-retry behavior. It adds no cancellation,
-abandonment, expiry, or liveness-based resolution protocol and adds no runtime enum, schema field,
-or persisted request preimage for future recovery. The remaining B3.2 packet owns every durable
-admission-state transition used to resolve an abandoned admission; `WorldDispatchControl` in B4
-owns the user/tool-facing exact inspect/cancel verb and consumes the RetainedWorkerRuntime result
-without writing admission state itself.
-
-Any later resolution request must exact-join all of the following durable preconditions before a
-state advance: issuer request identity, admission-record identity and revision, orchestration
-session, current exact authority and the relevant admission-to-current ancestry, current policy
-identity and authorization, retained participant, and the exact admission state being resolved.
-The complete canonical Spawn request remains required wherever fingerprint verification or exact
-retry semantics depend on it. PID, timeout, caller presence or disappearance, helper state, socket
-state, endpoint state, EOF, observer loss, and process liveness supply no resolution authority.
-
-Resolution is forward-only against durable protocol truth. It must not delete or roll back an
-already-committed R0 lineage/ref/registration, classify R0 rejection as cancellation, or treat
-stopping an already-created worker as cancelling a pending admission. Partial or already-applied
-R0 registration is reconciled to its exact admission record. A transport claim whose launch or
-registration result is ambiguous remains live and cannot free capacity until exact transport and
-runtime truth is reconciled. Crashes before and after resolution converge on retry; repeated
-resolution exact-joins the same terminal result; and the live admission count decreases only after
-that terminal result is durably published. Only then may the next eligible queued request acquire
-the registration head under the existing earliest-slot rule.
-
-B4 may freeze and return these semantic outcome categories without adding them to the B3.2a
-runtime schema: cancelled before registration; registered before transport; transport ambiguous
-or cancellation pending; already routable or terminal; invalid target; ambiguous target; and
-policy denied. Remaining B3.2 must first provide the durable, restart-safe resolution/reconciliation
-primitive those outcomes consume.
-
-The host-to-world launch carrier is typed and substitution-resistant:
-
-```rust
-struct RetainedWorkerAdmissionCommitmentCarrierV1 {
-    schema_version: u32, // exactly 1
-    algorithm: String,   // exactly &quot;hmac-sha-256&quot;
-    key_id: String,
-    digest_hex: String,
-}
-
-struct RetainedWorkerLaunchAuthorityProofV1 {
-    schema_version: u32, // exactly 1
-    authority_store_id: String,
-    issuer_request_id: String,
-    canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1,
-    registration_id: String,
-    registration_commitment: AuthorityObjectCommitmentV1,
-    authority_revision_after: u64,
-    authority_record_commitment_after: AuthorityObjectCommitmentV1,
-    orchestration_session_id: String,
-    caller_participant_id: String,
-    retained_participant_id: String,
-    bootstrap_run_id: String,
-    transport_claim_id: String,
-    backend_id: String,
-    protocol: String,
-    world_binding: WorldBindingV1,
-    current_policy_ref_id: String,
-    current_policy_revision: String,
-    retained_worker_ref_id: String,
-    retained_worker_commitment: AuthorityObjectCommitmentV1,
-}
-```
-
-Before either transport builder serializes it, the host exact-joins every field to the current HSA
-registration proof, admission record/fingerprint, bound caller, descriptor, policy, and world. The
-carrier is the exact field-for-field, transport-neutral equality projection of the internal
-RetainedWorkerAdmissionCommitmentV1; it contains no secret key and grants no HSA or admission
-mutation authority. The receiving world compares the closed carrier and dispatch fields only; it
-does not verify or reinterpret the host-only HMAC input. The
-optional carrier field on the V1 compatibility request may remain absent only on the explicitly
-pre-activation legacy path; both A1.2a-S authority-managed Spawn producers require it with no
-fallback. `transport-api-types` validates its closed shape. Production member Spawn enters
-`Service::execute_stream`. For authority-managed `Some(exact proof)`, that member branch completes
-the existing strict proof/dispatch equality validation, durably adopts the exact HSA-bound physical
-world through B3.2a-WA, and only then calls `MemberRuntimeManager::launch`, which retains its own
-validation before process creation. Compatibility `None` retains the existing direct path.
-`world-api`, `Service::execute`, and `convert_member_dispatch_request` are not part of this route.
-World-service does not mint or advance HSA or admission truth; the bound host verification supplies
-authority authenticity, B3.2a-WA supplies physical ownership only, and the launch boundary supplies
-exact carrier/request equality.
-
-`#[serde(default)]` on the optional `MemberDispatchRequestV1` proof field preserves wire
-compatibility by supplying `Default::default()` only when that field is absent during
-deserialization. It does not initialize Rust struct literals: each literal must name the field (or
-use explicit Rust struct update syntax, which is not authorized for these fixtures). Therefore the
-B3.2a allowlist additionally permits edits only in
-`crates/world-service/tests/member_runtime_world_placement_v1.rs`,
-`crates/world-service/tests/streamed_execute_cancel_v1.rs`, and
-`crates/world-service/tests/member_runtime_retained_lifecycle_v1.rs`, solely to set the field to
-`None` in existing explicitly pre-activation or legacy literals and prove unchanged compatibility.
-Every other fixture input, test name, assertion, expected outcome, and expected error remains
-unchanged. Any existing test that claims the authority-managed B3.2a route must carry an exact
-valid proof through the canonical production path instead; `None` is not valid there. These test
-files may not introduce a production path, helper default, fixture-only authority, alternate
-proof, alternate transport, side table, or weakened assertion. Authority-managed Spawn still
-requires the exact proof. This mechanical authorization did not itself complete B3.2a; the
-recorded B3.2a/B3.2a-WA result in `05` now does.
-
-That Rust-literal requirement also authorizes exactly three production compatibility
-initializers and nothing else: `build_run_world_task_transport_request` and
-`build_fork_world_worker_transport_request` in
-`crates/shell/src/execution/orchestrator_world_dispatch.rs`, plus `convert_member_dispatch` in
-`crates/world-mac-lima/src/lib.rs`. Each may only name the new optional proof field as explicit
-`None`; every pre-existing backend, protocol, run, session, participant, lineage, world, prompt,
-runtime, route-selection, policy, placement, lifecycle, error, and outcome field remains
-byte-for-byte and behaviorally unchanged. `None` grants no retained-worker launch authority to
-RunWorldTask, ForkWorldWorker, or the macOS world-api conversion. This is not macOS
-authority-managed Spawn adoption and authorizes no edit to `crates/world-api/src/lib.rs`,
-`Service::execute`, or `convert_member_dispatch_request`. No default constructor, side table,
-environment carrier, alternate transport route, or hidden proof synthesis may replace these
-explicit initializers. Both authority-managed B3.2a Spawn producers still require
-`Some(exact RetainedWorkerLaunchAuthorityProofV1)` exact-joined to admission and R0 truth before
-serialization, and missing, malformed, or mismatched proof fails before process creation. This
-mechanical authorization did not itself complete B3.2a; the recorded B3.2a/B3.2a-WA result in `05`
-is review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`.
-
-The same compiler-required widening authorizes only seven additional production-symbol edits. In
-`crates/shell/src/execution/orchestrator_world_dispatch.rs`, `fork_world_worker` and
-`continue_world_worker_fork_command_bootstrap_after_delivery` may only pass explicit `None` for the
-optional retained-worker authority context to the widened stream helper. In
-`crates/shell/src/repl/async_repl.rs`, `apply_greenfield_host_start_from_authority`,
-`prepare_hidden_owner_helper_runtime`,
-`start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx`,
-`prepare_fork_child_runtime_startup_for_descriptor`, and
-`prepare_member_runtime_startup_for_descriptor` may only initialize, destructure, or preserve the
-new optional retained-worker launch-authority proof/admission fields as `None`. Those values grant
-no retained-worker launch authority: fork and fork continuation remain compatibility paths; Host
-Start retains only its A1.2a-S host-session authority; hidden-owner-helper behavior is unchanged;
-and `prepare_member_runtime_startup_for_descriptor` remains legacy pre-activation preparation,
-distinct from the B3.2a-only `prepare_member_runtime_startup_from_authority_registration` path.
-Only that authority-registration preparer may construct the paired
-`Some(exact RetainedWorkerLaunchAuthorityProofV1)` and exact admission context. An
-authority-managed retained Spawn with either value missing fails closed and cannot fall back to the
-legacy preparer or reinterpret generic `None` as compatibility. The seven exceptions change no
-packet ownership, policy, lifecycle, transport, identity, lineage, error, outcome, host-runtime,
-helper, legacy-member, or fork semantics and authorize no other structural change. This mechanical
-authorization did not itself complete B3.2a; the recorded B3.2a/B3.2a-WA result in `05` is
-review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`.
+Canonical content: [`contracts/deferred-retained-spawn-admission-recovery-contract.md#deferred-retained-spawn-admission-recovery-contract`](contracts/deferred-retained-spawn-admission-recovery-contract.md#deferred-retained-spawn-admission-recovery-contract).
&#32;
 ### B3.2a-WA `ExactBoundWorldOwnershipAdoptionV1`
&#32;
diff --git a/llm-last-mile/runtime-refactor/index/README.md b/llm-last-mile/runtime-refactor/index/README.md
index a00b82170..9cf2a1f07 100644
--- a/llm-last-mile/runtime-refactor/index/README.md
+++ b/llm-last-mile/runtime-refactor/index/README.md
@@ -18,6 +18,7 @@
 | `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` | gate | [`gates/authority-required-macos-dev-parity.md`](../gates/authority-required-macos-dev-parity.md) | lane-local authority-required gate; Phase 1 grants no Phase 2 authority | Does not block or authorize the Linux-first runtime-refactor sequence. | [`04`](../04-contracts-and-gates.md#authority_requiredmacos_dev_parity-contract-2026-08-19-macos-lane) |
 | `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` | gate | [`gates/authority-required-runtime-refactor-reentry.md`](../gates/authority-required-runtime-refactor-reentry.md) | closed documentation/control-plane selection gate | Closed as selection history; A1.3-P1 is the separate active packet. | [`04`](../04-contracts-and-gates.md#authority_requiredruntime_refactor_reentry-contract-2026-08-20-closed-selection-record) |
 | `HostExecutionEpisodeV1` | contract | [`contracts/host-execution-episode-v1.md`](../contracts/host-execution-episode-v1.md) | `canonical extracted contract owner for the complete schema, episode-kind literals, transport-status literals, and rules 1–6` | Supersedes only root-canonical ownership of the extracted `HostExecutionEpisodeV1` span; the committed pre-span compatibility anchors and neighboring `2A`/`2B` owners remain unchanged. | [`04`](../04-contracts-and-gates.md#2-hostexecutionepisodev1) |
+| `Deferred retained-spawn admission recovery contract` | contract | [`contracts/deferred-retained-spawn-admission-recovery-contract.md`](../contracts/deferred-retained-spawn-admission-recovery-contract.md) | `canonical extracted owner for the complete deferred retained-spawn admission recovery contract, paired V1 recovery schemas, compatibility boundaries, route exclusions, allowlists, and recorded-result limits` | Supersedes only root-canonical ownership of the extracted `Deferred retained-spawn admission recovery contract` span; the two prerequisite compatibility anchors, `HostExecutionEpisodeV1`, and the following `B3.2a-WA ExactBoundWorldOwnershipAdoptionV1` owner remain unchanged. | [`04`](../04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract) |
 | `shared-target-architecture` | architecture index | [`architecture/README.md`](../architecture/README.md) | `non-authoritative navigation for the extracted executive target decision, authority map, numbered invariants, and stable review question` | Supersedes only root-canonical ownership of the extracted shared architecture spans; packet-family-local D5/D6 forwarders remain at their existing owners. | [`01`](../01-target-architecture.md#executive-decision), [`01`](../01-target-architecture.md#authority-map), [`01`](../01-target-architecture.md#non-negotiable-invariants), [`01`](../01-target-architecture.md#review-question) |
 | `shared-seam-crosswalk` | seam index | [`seams/README.md`](../seams/README.md) | `canonical shared seam-crosswalk rules plus extracted seam-family navigation for host/session, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation` | Supersedes only the extracted root reading-rule/classification spans and the extracted host/session authority, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation family rows; A0 and the existing D6 `HostSessionAuthority`, `WorldWorkerMessagingProtocol`, and `ObligationLedger` compatibility rows remain at their current owners. | [`02`](../02-seam-crosswalk.md#reading-rule), [`02`](../02-seam-crosswalk.md#a-host-authority-and-ingress), [`02`](../02-seam-crosswalk.md#b-dispatch-policy-receipts-and-retained-runtime), [`02`](../02-seam-crosswalk.md#c-obligations-and-host-re-engagement), [`02`](../02-seam-crosswalk.md#d-uaa-realization-projection-and-side-effect-mediation), [`02`](../02-seam-crosswalk.md#classification-consequences) |
 | `shared-slice-map` | slice index | [`slices/README.md`](../slices/README.md) | `canonical shared sequencing/dependency and slice-closeout owner plus non-authoritative D9 Track A–E navigation and A1/A1.4 projection index` | Supersedes only root-canonical ownership of the extracted shared sequencing/closeout spans plus the extracted Track A–E and A1/A1.4 projection spans; controlling schedule authority remains with decisions, packets, gates, and `current.md`. | [`03`](../03-phase-slice-map.md#sequencing-rules), [`03`](../03-phase-slice-map.md#track-a--authority-and-surface-neutrality), [`03`](../03-phase-slice-map.md#a1-bounded-packet-decomposition), [`03`](../03-phase-slice-map.md#track-b--world-dispatch-receipts-supervision-and-cancel), [`03`](../03-phase-slice-map.md#track-c--obligations-inbox-auto-attach-and-router-attach), [`03`](../03-phase-slice-map.md#track-d--uaa-execution-envelope-and-side-effect-mediation), [`03`](../03-phase-slice-map.md#track-e--dispatch-scoped-policy-narrowing-and-config-projection), [`03`](../03-phase-slice-map.md#slice-closeout-minimum) |
diff --git a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
index c46a9f38d..619c1826c 100644
--- a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
+++ b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
@@ -185,3 +185,4 @@ This ledger records content-preserving authority transfers while retaining requi
 | D9 | [`03-phase-slice-map.md`](../03-phase-slice-map.md) | E4 — Host-visible write/sync contract | [`track-e--dispatch-scoped-policy-narrowing-and-config-projection`](../03-phase-slice-map.md#track-e--dispatch-scoped-policy-narrowing-and-config-projection) | line 223 | `c98a5df3f09735abc5cbefec285c7b5dc55fca5c67cfdf5bf6bd3f458d017893` | [`../slices/e4-host-visible-write-sync-contract.md`](../slices/e4-host-visible-write-sync-contract.md) | slice row | canonical destination; source compatibility table row | none | restore the exact extracted D9 source bodies/rows in `03-phase-slice-map.md`; remove `slices/` and all D9-added files beneath it; remove the `shared-slice-map` index row from `index/README.md`; remove all matching D9 ledger entries; leave D5–D8 canonical owners, `index/current.md`, and `review-control/` unchanged |
 | D9 | [`03-phase-slice-map.md`](../03-phase-slice-map.md) | A1.4 — bounded auto-attach producer adoption and regression closure | [`a1-bounded-packet-decomposition`](../03-phase-slice-map.md#a1-bounded-packet-decomposition) | line 145 | `58d7982b23a7b2f5f41f3e0b5170996e0f3b331dfbbcbb5875d02e801bd5d408` | [`../slices/tasks/a1-4-auto-attach-producer-adoption.md`](../slices/tasks/a1-4-auto-attach-producer-adoption.md) | task row | canonical destination; source compatibility table row | none | restore the exact extracted D9 source bodies/rows in `03-phase-slice-map.md`; remove `slices/` and all D9-added files beneath it; remove the `shared-slice-map` index row from `index/README.md`; remove all matching D9 ledger entries; leave D5–D8 canonical owners, `index/current.md`, and `review-control/` unchanged |
 | D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 2. `HostExecutionEpisodeV1` | [`2-hostexecutionepisodev1`](../04-contracts-and-gates.md#2-hostexecutionepisodev1) | lines 51–100 | `28e86e2e7e29b458280875f921ca327a469621ac96bd7e592a0a176027aa0117` | [`../contracts/host-execution-episode-v1.md`](../contracts/host-execution-episode-v1.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1645-byte source span at `04-contracts-and-gates.md#2-hostexecutionepisodev1`; remove `contracts/host-execution-episode-v1.md`; remove the exact `HostExecutionEpisodeV1` row from `index/README.md`; remove this D10 ledger entry; leave the committed pre-span compatibility anchors, D3/D5–D9 owners, `index/current.md`, `review-control/`, all other D10 units, and every path outside the four-path fence unchanged |
+| D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | Deferred retained-spawn admission recovery contract | [`deferred-retained-spawn-admission-recovery-contract`](../04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract) | lines 71–213 | `d903299bdd345316fd0943d28abc300a1cef001ce0744746f72f53607eb64c12` | [`../contracts/deferred-retained-spawn-admission-recovery-contract.md`](../contracts/deferred-retained-spawn-admission-recovery-contract.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 9896-byte source span at `04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract`; remove `contracts/deferred-retained-spawn-admission-recovery-contract.md`; remove the exact `Deferred retained-spawn admission recovery contract` row from `index/README.md`; remove this D10 ledger entry; leave `HostExecutionEpisodeV1`, the two prerequisite compatibility anchors, all D3/D5–D9 owners, `index/current.md`, `review-control/`, later D10 units, and every path outside the four-path fence unchanged |
diff --git a/llm-last-mile/runtime-refactor/contracts/deferred-retained-spawn-admission-recovery-contract.md b/llm-last-mile/runtime-refactor/contracts/deferred-retained-spawn-admission-recovery-contract.md
new file mode 100644
index 000000000..5352eb87d
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/contracts/deferred-retained-spawn-admission-recovery-contract.md
@@ -0,0 +1,151 @@
+**Kind:** contract
+**Status:** canonical
+**Canonical for:** complete extracted deferred retained-spawn admission recovery contract including the recovery preconditions, paired `RetainedWorkerAdmissionCommitmentCarrierV1` / `RetainedWorkerLaunchAuthorityProofV1` schemas, compatibility boundaries, route exclusions, allowlists, and recorded-result limits
+**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract`](../04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract), baseline lines 71–213; the exact 9896-byte source body is preserved between the boundary markers below
+**Baseline span SHA-256:** `d903299bdd345316fd0943d28abc300a1cef001ce0744746f72f53607eb64c12`
+
+&lt;!-- exact-extracted-body:start --&gt;
+### Deferred retained-spawn admission recovery contract
+
+B3.2a deliberately stops at conservative exact-retry behavior. It adds no cancellation,
+abandonment, expiry, or liveness-based resolution protocol and adds no runtime enum, schema field,
+or persisted request preimage for future recovery. The remaining B3.2 packet owns every durable
+admission-state transition used to resolve an abandoned admission; `WorldDispatchControl` in B4
+owns the user/tool-facing exact inspect/cancel verb and consumes the RetainedWorkerRuntime result
+without writing admission state itself.
+
+Any later resolution request must exact-join all of the following durable preconditions before a
+state advance: issuer request identity, admission-record identity and revision, orchestration
+session, current exact authority and the relevant admission-to-current ancestry, current policy
+identity and authorization, retained participant, and the exact admission state being resolved.
+The complete canonical Spawn request remains required wherever fingerprint verification or exact
+retry semantics depend on it. PID, timeout, caller presence or disappearance, helper state, socket
+state, endpoint state, EOF, observer loss, and process liveness supply no resolution authority.
+
+Resolution is forward-only against durable protocol truth. It must not delete or roll back an
+already-committed R0 lineage/ref/registration, classify R0 rejection as cancellation, or treat
+stopping an already-created worker as cancelling a pending admission. Partial or already-applied
+R0 registration is reconciled to its exact admission record. A transport claim whose launch or
+registration result is ambiguous remains live and cannot free capacity until exact transport and
+runtime truth is reconciled. Crashes before and after resolution converge on retry; repeated
+resolution exact-joins the same terminal result; and the live admission count decreases only after
+that terminal result is durably published. Only then may the next eligible queued request acquire
+the registration head under the existing earliest-slot rule.
+
+B4 may freeze and return these semantic outcome categories without adding them to the B3.2a
+runtime schema: cancelled before registration; registered before transport; transport ambiguous
+or cancellation pending; already routable or terminal; invalid target; ambiguous target; and
+policy denied. Remaining B3.2 must first provide the durable, restart-safe resolution/reconciliation
+primitive those outcomes consume.
+
+The host-to-world launch carrier is typed and substitution-resistant:
+
+```rust
+struct RetainedWorkerAdmissionCommitmentCarrierV1 {
+    schema_version: u32, // exactly 1
+    algorithm: String,   // exactly &quot;hmac-sha-256&quot;
+    key_id: String,
+    digest_hex: String,
+}
+
+struct RetainedWorkerLaunchAuthorityProofV1 {
+    schema_version: u32, // exactly 1
+    authority_store_id: String,
+    issuer_request_id: String,
+    canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1,
+    registration_id: String,
+    registration_commitment: AuthorityObjectCommitmentV1,
+    authority_revision_after: u64,
+    authority_record_commitment_after: AuthorityObjectCommitmentV1,
+    orchestration_session_id: String,
+    caller_participant_id: String,
+    retained_participant_id: String,
+    bootstrap_run_id: String,
+    transport_claim_id: String,
+    backend_id: String,
+    protocol: String,
+    world_binding: WorldBindingV1,
+    current_policy_ref_id: String,
+    current_policy_revision: String,
+    retained_worker_ref_id: String,
+    retained_worker_commitment: AuthorityObjectCommitmentV1,
+}
+```
+
+Before either transport builder serializes it, the host exact-joins every field to the current HSA
+registration proof, admission record/fingerprint, bound caller, descriptor, policy, and world. The
+carrier is the exact field-for-field, transport-neutral equality projection of the internal
+RetainedWorkerAdmissionCommitmentV1; it contains no secret key and grants no HSA or admission
+mutation authority. The receiving world compares the closed carrier and dispatch fields only; it
+does not verify or reinterpret the host-only HMAC input. The
+optional carrier field on the V1 compatibility request may remain absent only on the explicitly
+pre-activation legacy path; both A1.2a-S authority-managed Spawn producers require it with no
+fallback. `transport-api-types` validates its closed shape. Production member Spawn enters
+`Service::execute_stream`. For authority-managed `Some(exact proof)`, that member branch completes
+the existing strict proof/dispatch equality validation, durably adopts the exact HSA-bound physical
+world through B3.2a-WA, and only then calls `MemberRuntimeManager::launch`, which retains its own
+validation before process creation. Compatibility `None` retains the existing direct path.
+`world-api`, `Service::execute`, and `convert_member_dispatch_request` are not part of this route.
+World-service does not mint or advance HSA or admission truth; the bound host verification supplies
+authority authenticity, B3.2a-WA supplies physical ownership only, and the launch boundary supplies
+exact carrier/request equality.
+
+`#[serde(default)]` on the optional `MemberDispatchRequestV1` proof field preserves wire
+compatibility by supplying `Default::default()` only when that field is absent during
+deserialization. It does not initialize Rust struct literals: each literal must name the field (or
+use explicit Rust struct update syntax, which is not authorized for these fixtures). Therefore the
+B3.2a allowlist additionally permits edits only in
+`crates/world-service/tests/member_runtime_world_placement_v1.rs`,
+`crates/world-service/tests/streamed_execute_cancel_v1.rs`, and
+`crates/world-service/tests/member_runtime_retained_lifecycle_v1.rs`, solely to set the field to
+`None` in existing explicitly pre-activation or legacy literals and prove unchanged compatibility.
+Every other fixture input, test name, assertion, expected outcome, and expected error remains
+unchanged. Any existing test that claims the authority-managed B3.2a route must carry an exact
+valid proof through the canonical production path instead; `None` is not valid there. These test
+files may not introduce a production path, helper default, fixture-only authority, alternate
+proof, alternate transport, side table, or weakened assertion. Authority-managed Spawn still
+requires the exact proof. This mechanical authorization did not itself complete B3.2a; the
+recorded B3.2a/B3.2a-WA result in `05` now does.
+
+That Rust-literal requirement also authorizes exactly three production compatibility
+initializers and nothing else: `build_run_world_task_transport_request` and
+`build_fork_world_worker_transport_request` in
+`crates/shell/src/execution/orchestrator_world_dispatch.rs`, plus `convert_member_dispatch` in
+`crates/world-mac-lima/src/lib.rs`. Each may only name the new optional proof field as explicit
+`None`; every pre-existing backend, protocol, run, session, participant, lineage, world, prompt,
+runtime, route-selection, policy, placement, lifecycle, error, and outcome field remains
+byte-for-byte and behaviorally unchanged. `None` grants no retained-worker launch authority to
+RunWorldTask, ForkWorldWorker, or the macOS world-api conversion. This is not macOS
+authority-managed Spawn adoption and authorizes no edit to `crates/world-api/src/lib.rs`,
+`Service::execute`, or `convert_member_dispatch_request`. No default constructor, side table,
+environment carrier, alternate transport route, or hidden proof synthesis may replace these
+explicit initializers. Both authority-managed B3.2a Spawn producers still require
+`Some(exact RetainedWorkerLaunchAuthorityProofV1)` exact-joined to admission and R0 truth before
+serialization, and missing, malformed, or mismatched proof fails before process creation. This
+mechanical authorization did not itself complete B3.2a; the recorded B3.2a/B3.2a-WA result in `05`
+is review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`.
+
+The same compiler-required widening authorizes only seven additional production-symbol edits. In
+`crates/shell/src/execution/orchestrator_world_dispatch.rs`, `fork_world_worker` and
+`continue_world_worker_fork_command_bootstrap_after_delivery` may only pass explicit `None` for the
+optional retained-worker authority context to the widened stream helper. In
+`crates/shell/src/repl/async_repl.rs`, `apply_greenfield_host_start_from_authority`,
+`prepare_hidden_owner_helper_runtime`,
+`start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx`,
+`prepare_fork_child_runtime_startup_for_descriptor`, and
+`prepare_member_runtime_startup_for_descriptor` may only initialize, destructure, or preserve the
+new optional retained-worker launch-authority proof/admission fields as `None`. Those values grant
+no retained-worker launch authority: fork and fork continuation remain compatibility paths; Host
+Start retains only its A1.2a-S host-session authority; hidden-owner-helper behavior is unchanged;
+and `prepare_member_runtime_startup_for_descriptor` remains legacy pre-activation preparation,
+distinct from the B3.2a-only `prepare_member_runtime_startup_from_authority_registration` path.
+Only that authority-registration preparer may construct the paired
+`Some(exact RetainedWorkerLaunchAuthorityProofV1)` and exact admission context. An
+authority-managed retained Spawn with either value missing fails closed and cannot fall back to the
+legacy preparer or reinterpret generic `None` as compatibility. The seven exceptions change no
+packet ownership, policy, lifecycle, transport, identity, lineage, error, outcome, host-runtime,
+helper, legacy-member, or fork semantics and authorize no other structural change. This mechanical
+authorization did not itself complete B3.2a; the recorded B3.2a/B3.2a-WA result in `05` is
+review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`.
+
+&lt;!-- exact-extracted-body:end --&gt;
```
</pre>

</details>

<details>
<summary>Initial-range review prompt exact bytes (base64 UTF-8)</summary>

```text
UmV2aWV3IG1vZGU6IGluaXRpYWwtcmFuZ2UKUmV2aWV3IGNvbnRleHQ6IGluZGVwZW5kZW50IGZyZXNoIGNvbnZlcnNhdGlvbgpSZXZpZXcgYm91bmRhcnk6IGV4YWN0IGJhc2VsaW5lIGNvbW1pdCBgNmQ5Y2FjNGQyOTQzZDdhYTVlZjU4MjVjYzA5OTQ5MmQxNjZlNWY0YWAsIGJhc2VsaW5lIHRyZWUgYDE5MDJhOGE0MmFlODdkZmYwN2IyZjBhNWZhNWQ3NDFiZGZhMTg2ZTFgLCBwbHVzIHRoZSBjb21wbGV0ZSBjb250ZW50LWFkZHJlc3NlZCB1bmNvbW1pdHRlZCBwYXRjaCBiZWxvdy4gQ29tcGxldGUgcGF0Y2ggU0hBLTI1NjogYDhkZDIwZWQ1NmJkZDA1MjU2MjE3MzE3ZDk1NzhhYjQ5ZjNhOTBiNzc3NWYzNDBhZGRmNmEyYmQ5ZTMyNzcxMTVgLgpSZXZpZXcgdGFyZ2V0OiB0aGUgY29tcGxldGUgYm91bmRlZCBEMTAgZXh0cmFjdGlvbiBvZiB0aGUgZW50aXJlIGxhcmdlIGNvbXBvdW5kIGBEZWZlcnJlZCByZXRhaW5lZC1zcGF3biBhZG1pc3Npb24gcmVjb3ZlcnkgY29udHJhY3RgLCBpbmNsdWRpbmcgdGhlIHVudHJhY2tlZCBvd25lci4gUmV2aWV3IGFsbCBmb3VyIGZpbGVzIGFuZCBkbyBub3Qgc3BsaXQgb3IgcmVkdWNlIHRoZSBjb21wb3VuZCB1bml0LgoKU3VwcG9ydGVkIGlucHV0cyBhbmQgYmVoYXZpb3I6IHJlcG9zaXRvcnkgR2l0SHViLXN0eWxlIE1hcmtkb3duIGxpbmtzL2ZyYWdtZW50czsgb25lIHJvb3QgbGVnYWN5IGhlYWRpbmcvcG9pbnRlcjsgb25lIGNhbm9uaWNhbCBjb21wb3VuZCBNYXJrZG93biBvd25lcjsgb25lIGluZGV4IHJvdzsgb25lIGV4dHJhY3Rpb24tbGVkZ2VyIHJvdy4gTm9uLXJlbmRlcmluZyBIVE1MIGJvdW5kYXJ5IGNvbW1lbnRzIGRlbGltaXQgdGhlIGV4YWN0IGNhbm9uaWNhbCBzb3VyY2UtYm9keSBieXRlcy4gUnVzdC90ZXh0IGNvbnRlbnQgaXMgbGl0ZXJhbCBkb2N1bWVudGF0aW9uLiBUaGUgc291cmNlLWJvZHkgYnl0ZXMgaW5jbHVkZSBkdXJhYmxlIHJlY292ZXJ5IHByZWNvbmRpdGlvbnMsIHNlbWFudGljIG91dGNvbWVzLCB0d28gcGFpcmVkIFYxIHNjaGVtYXMsIEhNQUMvY2FycmllciBlcXVhbGl0eSBydWxlcywgY29tcGF0aWJpbGl0eSBgTm9uZWAgdmVyc3VzIGF1dGhvcml0eS1tYW5hZ2VkIGBTb21lYCBib3VuZGFyaWVzLCBleGFjdCByb3V0ZSBleGNsdXNpb25zLCBmaXh0dXJlL3Byb2R1Y3Rpb24tc3ltYm9sIGFsbG93bGlzdHMsIGZhaWxlZC1jbG9zZWQgcnVsZXMsIGFuZCByZWNvcmRlZC1yZXN1bHQgbGltaXRhdGlvbnMuClN1cHBvcnRlZCBwbGF0Zm9ybXMgYW5kIGRpYWxlY3RzOiByZXBvc2l0b3J5IE1hcmtkb3duIG9ubHkgdW5kZXIgdGhlIGxvY2FsIEdpdEh1Yi1zdHlsZSBmcmFnbWVudCB2YWxpZGF0b3IuIE5vIFJ1c3QgY29tcGlsYXRpb24sIHJ1bnRpbWUsIHNjcmlwdCwgcGxhdGZvcm0sIGV4dGVybmFsIHJlbmRlcmVyLCBEMTEsIG9yIEQxMiBiZWhhdmlvciBpcyBzdXBwb3J0ZWQvaW4gc2NvcGUuCgpJbi1zY29wZSBpbnZhcmlhbnRzOgoxLiBUaGUgY29tcGxldGUgY29tcG91bmQgdW5pdCByZW1haW5zIGluc2VwYXJhYmxlIGFuZCBoYXMgZXhhY3RseSBvbmUgY2Fub25pY2FsIG93bmVyOiBgY29udHJhY3RzL2RlZmVycmVkLXJldGFpbmVkLXNwYXduLWFkbWlzc2lvbi1yZWNvdmVyeS1jb250cmFjdC5tZGAuCjIuIFJvb3QgcmV0YWlucyBleGFjdGx5IG9uZSBsZWdhY3kgaGVhZGluZy9hbmNob3IgYW5kIHJlcGxhY2VzIG9ubHkgaXRzIHN1YnN0YW50aXZlIGJvZHkgd2l0aCBhIHNoYWxsb3cgcG9pbnRlcjsgZm9sbG93aW5nIGBCMy4yYS1XQSBFeGFjdEJvdW5kV29ybGRPd25lcnNoaXBBZG9wdGlvblYxYCBhbmQgcHJlY2VkaW5nIHByb2plY3Rpb25zIHJlbWFpbiB1bmNoYW5nZWQuCjMuIEJhc2VsaW5lIHNvdXJjZSBpcyBleGFjdGx5IGxpbmVzIDcx4oCTMjEzIGluY2x1c2l2ZSwgOTg5NiBieXRlcywgU0hBLTI1NiBgZDkwMzI5OWJkZDM0NTMxNmZkMDk0M2QyOGFiYzMwMGExY2VmMDAxY2UwNzQ0NzQ2ZjcyZjUzNjA3ZWI2NGMxMmAuIEJ5dGVzIHN0cmljdGx5IGJldHdlZW4gb3duZXIgbWFya2VycyBtYXRjaCBpdCBleGFjdGx5LCBpbmNsdWRpbmcgdGhlIGZpbmFsIGJsYW5rIHNlcGFyYXRvci4KNC4gRXZlcnkgaGVhZGluZywgcGFyYWdyYXBoLCBsaW5lIHdyYXAsIGJsYW5rIGxpbmUsIFJ1c3QgZmVuY2UsIG9yZGVyZWQgZmllbGQsIHN0YWJsZSBWMSBuYW1lLCBgaG1hYy1zaGEtMjU2YCBsaXRlcmFsLCBmaWxlbmFtZSwgZnVuY3Rpb24vc3ltYm9sIG5hbWUsIGFsbG93bGlzdCwgcm91dGUgZXhjbHVzaW9uLCBgU29tZWAvYE5vbmVgIHJ1bGUsIG5lZ2F0aXZlIHJlcXVpcmVtZW50LCBmYWlsZWQvYmxvY2tlZC9yZW1haW5pbmcgc2VtYW50aWMgc3RhdHVzLCBhdXRob3JpdHkgYm91bmRhcnksIHZhbGlkYXRpb24vcHJvZHVjdGlvbi1wYXRoIHNlcXVlbmNlLCBhbmQgbGl0ZXJhbCBoYXNoIHN1Y2ggYXMgYGQwYTcwNzI3YzJiZWMyYjJkNmZlMDc1NGVhNDY5YzQ2ODI2ODRkZGFgIGlzIGV4YWN0IGJ5IGJ5dGUgaWRlbnRpdHkuCjUuIGBSZXRhaW5lZFdvcmtlckFkbWlzc2lvbkNvbW1pdG1lbnRDYXJyaWVyVjFgIGFuZCBgUmV0YWluZWRXb3JrZXJMYXVuY2hBdXRob3JpdHlQcm9vZlYxYCByZW1haW4gcGFpcmVkIGluIHRoaXMgY29tcG91bmQgb3duZXI7IEQ2IHBhY2tldCByZWZlcmVuY2VzIGFyZSB1bmNoYW5nZWQgYW5kIG5vdCBtaXNyZXByZXNlbnRlZCBhcyBzZWNvbmQgY2Fub25pY2FsIGRlZmluaXRpb25zLgo2LiBJbmRleCBhZGRzIGV4YWN0bHkgb25lIHRydXRoZnVsIGNvbnRyYWN0IHJvdyB3aXRoIHVuY2hhbmdlZCB1bnJlbGF0ZWQgcm93IG9yZGVyLgo3LiBMZWRnZXIgYWRkcyBleGFjdGx5IG9uZSB0cnV0aGZ1bCBEMTAgcm93IHdpdGggc291cmNlIGxvY2F0b3IgYGxpbmVzIDcx4oCTMjEzYCwgc291cmNlIGRpZ2VzdCwgY29ycmVjdCBhbmNob3JzL2Rlc3RpbmF0aW9uL3JlcHJlc2VudGF0aW9uL2RlbHRhLCBhbmQgYXRvbWljIHJvbGxiYWNrLgo4LiBSb2xsYmFjayByZXN0b3JlcyB0aGUgZXhhY3QgOTg5Ni1ieXRlIHNwYW4sIHJlbW92ZXMgb25seSB0aGUgb3duZXIgYW5kIGV4YWN0IGluZGV4L2xlZGdlciByb3dzLCBhbmQgcHJlc2VydmVzIEhvc3RFeGVjdXRpb25FcGlzb2RlVjEsIHRoZSB0d28gcHJlcmVxdWlzaXRlIGFuY2hvcnMsIEQzL0Q14oCTRDkgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9gLCBsYXRlciBEMTAgdW5pdHMsIGFuZCBhbGwgb3RoZXIgcGF0aHMuCjkuIEFsbCBydW50aW1lLXJlZmFjdG9yIHJlcG9zaXRvcnktcmVsYXRpdmUgTWFya2Rvd24gbGlua3MvYW5jaG9ycyByZXNvbHZlLgoxMC4gVGhlIHBhdGNoIGRvZXMgbm90IGltcGxlbWVudCBydW50aW1lIGJlaGF2aW9yLCBicm9hZGVuIG1lY2hhbmljYWwgZXhjZXB0aW9ucy9hbGxvd2xpc3RzLCBjb252ZXJ0IGV2aWRlbmNlIGludG8gZ2F0ZSBzYXRpc2ZhY3Rpb24sIGdyYW50IGF1dGhvcml0eSwgYXBwcm92ZSBzdWNjZXNzb3JzL2xhdGVyIEQxMCwgYmVnaW4gRDExLCBvciBwZXJmb3JtIEQxMi4KMTEuIEV4YWN0IGNoYW5nZWQtcGF0aCBtYW5pZmVzdCBpbmNsdWRpbmcgdW50cmFja2VkIGlzOgotIGBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZGAgU0hBLTI1NiBgMThhYWEzNTQ4NzFjOGU0NTY5ZjBhZDE3NjEwZjIwM2FjN2EyM2Y5NmI3MjY4NWI1NjBlZjg3ZjRkYTc3YmE1NGAKLSBgbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2NvbnRyYWN0cy9kZWZlcnJlZC1yZXRhaW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29udHJhY3QubWRgIFNIQS0yNTYgYDA2ZmQ2ODczNTU4MmQwZGRlZTc2YTE0NjAyYjk5ZWFkNmE2ZGY4OWFmZmJkMTAzNmQ0ZjM2NjEzZWJkMGQ5YzJgCi0gYGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9SRUFETUUubWRgIFNIQS0yNTYgYDkwODk5ZDU3ZWM0YmZjYjFmNzBhOTMwMTBhMWJiNmRmNjQ2ZDMxMjA1NWYzMTc1YWQxY2FkN2I0ODkwOTk3NTlgCi0gYGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWRgIFNIQS0yNTYgYDI4MTc3NmY4ZTVhMjgxNTAxYjE5M2YzOGIwMjM5Y2ZhMjRlMWNkMWZmNjhhZDQzNDk5MDU3YTQ2MmJiOTViYzVgCgpSZXF1aXJlZCBtYXRlcmlhbCBjb25zZXF1ZW5jZTogYmxvY2tpbmcgcmVxdWlyZXMgYSByZWFjaGFibGUgYnJva2VuIHN1cHBvcnRlZCBsaW5rL2FuY2hvcjsgbG9zdC9yZW9yZGVyZWQvY2hhbmdlZCBzb3VyY2UgYnl0ZSBvciBjb21wb3VuZCByZWxhdGlvbnNoaXA7IGFsdGVyZWQgbmFtZWQgdmVyc2lvbiwgZmllbGQsIGxpdGVyYWwsIGNvZGUgZmVuY2UsIGFsbG93bGlzdC9leGNsdXNpb24sIG5lZ2F0aXZlL2ZhaWwtY2xvc2VkIHJ1bGUsIHN0YXR1cy9hdXRob3JpdHkgYm91bmRhcnksIGhhc2gsIHByb3ZlbmFuY2Ugb3Igcm9sbGJhY2s7IGR1cGxpY2F0ZS9taXNzaW5nIGNhbm9uaWNhbCBvd25lcjsgb3IgcGF0aC9zY29wZSB2aW9sYXRpb24gY2F1c2VkIGJ5IHRoaXMgcGF0Y2guCkJsb2NraW5nIHRocmVzaG9sZDogb25seSByZWFjaGFibGUgbWF0ZXJpYWwgUDEvUDIgZGVmZWN0cyBpbnRyb2R1Y2VkIGJ5IHRoaXMgcGF0Y2guIFVuY2hhbmdlZCBwcmUtZXhpc3RpbmcgYmVoYXZpb3IgYW5kIHN0eWxpc3RpYyBwcmVmZXJlbmNlcyBhcmUgbm9uLWJsb2NraW5nLgpBY2NlcHRlZCBwcmlvciBmaW5kaW5nczogbm9uZS4KRGVmZXJyZWQgb3Igb3V0LW9mLXNjb3BlIGNvbmNlcm5zOiBwcmlvciBEMy9ENeKAk0Q5IG93bmVycywgcGFja2V0LWZhbWlseSBtYXRlcmlhbCwgSG9zdEV4ZWN1dGlvbkVwaXNvZGVWMSwgZm9sbG93aW5nIEIzLjJhLVdBLCBhbGwgbGF0ZXIgRDEwIHVuaXRzLCBEMTEsIEQxMiwgcnVudGltZS9SdXN0L3NjcmlwdHMvcGxhdGZvcm0gd29yaywgcm9hZG1hcC9aSVAsIHJldmlldyBnb3Zlcm5hbmNlLCBNYXJrZG93biBub3JtYWxpemF0aW9uLCB1bnJlbGF0ZWQgcHJlLWV4aXN0aW5nIGlzc3Vlcy4KUHJvamVjdCBzb3VyY2VzOiB0aGlzIHByb21wdCBjb250YWlucyB0aGUgZXhhY3QgYmFzZWxpbmUsIGZvdXItZmlsZSBtYW5pZmVzdCwgY29tcGxldGUgcGF0Y2gsIGJvZHkgZGlnZXN0LCBhbmQgdmFsaWRhdGlvbiBldmlkZW5jZS4gRXhjbHVkZWQgc291cmNlIGlzIHVuYXZhaWxhYmxlIGFuZCBtdXN0IG5vdCBiZSBpbmZlcnJlZC4gUmVhZCB0aGUgY29tcGxldGUgcGF0Y2ggYmVmb3JlIGp1ZGdpbmcgdGhlIHJlc3VsdC4KClZhbGlkYXRpb24gZXZpZGVuY2U6Ci0gZm91ci1wYXRoIGludmVudG9yeSBpbmNsdWRpbmcgdW50cmFja2VkIGFuZCBubyBzdGFnZWQgZmlsZXM6IHBhc3MuCi0gYGdpdCBkaWZmIC0tY2hlY2tgOiBwYXNzLgotIG5ldyBvd25lciBuby1pbmRleCB3aGl0ZXNwYWNlIGNoZWNrOiBubyBkaWFnbm9zdGljcyAoZXhwZWN0ZWQgbm9uemVybyBkaWZmIHN0YXR1cyBvbmx5KS4KLSBleGFjdCBib2R5IGV4dHJhY3Rpb246IDk4OTYgYnl0ZXMsIFNIQS0yNTYgYGQ5MDMyOTliZGQzNDUzMTZmZDA5NDNkMjhhYmMzMDBhMWNlZjAwMWNlMDc0NDc0NmY3MmY1MzYwN2ViNjRjMTJgLCBieXRlLWVxdWFsIHRvIHNvdXJjZS4KLSBleHBsaWNpdCBwYWlyZWQtc2NoZW1hL29yZGVyZWQtZmllbGQvZmVuY2UvbGl0ZXJhbC9wYXRoL3N5bWJvbC9hbGxvd2xpc3QvbmVnYXRpdmUtZXhjbHVzaW9uL1NvbWUtTm9uZS9oYXNoL3N0YXR1cy9hdXRob3JpdHkgY2hlY2tzOiBwYXNzLgotIE1hcmtkb3duIHZhbGlkYXRvcjogYExJTktTX0NIRUNLRUQgMTgxOSBGQUlMVVJFUyAwYC4KLSByb290L2Rlc3RpbmF0aW9uIGFuY2hvcnMgdW5pcXVlOyByb290L2luZGV4L2xlZGdlciBsaW5rcyByZXNvbHZlLgotIHByaW9yIG93bmVyL3Jldmlldy1hcnRpZmFjdCBzdGFiaWxpdHkgY2hlY2tzOiBwYXNzLgotIGNvbXBsZXRlIHBhdGNoIGZvcndhcmQvcmV2ZXJzZSBhcHBseSBhZ2FpbnN0IGRpc3Bvc2FibGUgY2xlYW4gZXhwb3J0czogcGFzcy4KLSB2YWxpZGF0aW9uIGxvZyBTSEEtMjU2IGA1NDFkY2Q4MjFlYzVjYzEwMDBmOGQ3NjUwNmQ5NWZlYjdjYTYzZjY2NGNiZWZiNTg2NjJlODA3NjI0YzJiNzhkYC4KLSBjb21wbGV0ZSBwYXRjaCBTSEEtMjU2IGA4ZGQyMGVkNTZiZGQwNTI1NjIxNzMxN2Q5NTc4YWI0OWYzYTkwYjc3NzVmMzQwYWRkZjZhMmJkOWUzMjc3MTE1YC4KCkFkdmlzb3J5IG9ubHk7IHZlcmlmeSBhZ2FpbnN0IGxvY2FsIHByb2plY3QgdHJ1dGggYW5kIGF1dGhvcml0YXRpdmUgZG9jczsgZG8gbm90IHJlZHVjZSBzY29wZSB3aXRob3V0IHVzZXIgYXBwcm92YWwuCgpEbyBub3QgcmVkdWNlIHRoZSB0YXNrIG9yIHJlcGxhY2UgaXQgd2l0aCBhbiBlYXNpZXIgYWx0ZXJuYXRpdmUuIFByZXNlcnZlIHRoZSByZXF1ZXN0ZWQgc2NvcGUgYW5kIHByb2plY3QgY29udmVudGlvbnMuCgpSZXR1cm4gZmluZGluZ3MgZmlyc3QgYnkgc2V2ZXJpdHksIGluY2x1ZGluZyBmaWxlL2hlYWRpbmcsIHJlYWNoYWJpbGl0eSwgdmlvbGF0ZWQgaW52YXJpYW50LCBtYXRlcmlhbCBjb25zZXF1ZW5jZSwgYW5kIHBhdGNoIGNhdXNhbGl0eS4gU3RhdGUgZXhwbGljaXRseSB3aGVuIG5vbmUgcXVhbGlmeS4gRW5kIHdpdGggZXhhY3RseSBgVkVSRElDVDogQVBQUk9WRURgIG9yIGBWRVJESUNUOiBDSEFOR0VTIFJFUVVJUkVEYC4KCkNvbXBsZXRlIHBhdGNoIChVVEYtOCwgaW5jbHVkZXMgYWxsIHVudHJhY2tlZCBmaWxlcyk6CmBgYGRpZmYKZGlmZiAtLWdpdCBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQKaW5kZXggYTljZGY1ODZiLi43YjgwYmFjNTkgMTAwNjQ0Ci0tLSBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kCisrKyBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kCkBAIC03MCwxNDYgKzcwLDcgQEAgQ2Fub25pY2FsIGNvbnRlbnQ6IFtgYjEtYjItMS9jb250cmFjdHMtYW5kLWdhdGVzLm1kI2IxYjIxLWJvdW5kZWQtcmVhZC1vbmx5LWRpc3AKIAogIyMjIERlZmVycmVkIHJldGFpbmVkLXNwYXduIGFkbWlzc2lvbiByZWNvdmVyeSBjb250cmFjdAogCi1CMy4yYSBkZWxpYmVyYXRlbHkgc3RvcHMgYXQgY29uc2VydmF0aXZlIGV4YWN0LXJldHJ5IGJlaGF2aW9yLiBJdCBhZGRzIG5vIGNhbmNlbGxhdGlvbiwKLWFiYW5kb25tZW50LCBleHBpcnksIG9yIGxpdmVuZXNzLWJhc2VkIHJlc29sdXRpb24gcHJvdG9jb2wgYW5kIGFkZHMgbm8gcnVudGltZSBlbnVtLCBzY2hlbWEgZmllbGQsCi1vciBwZXJzaXN0ZWQgcmVxdWVzdCBwcmVpbWFnZSBmb3IgZnV0dXJlIHJlY292ZXJ5LiBUaGUgcmVtYWluaW5nIEIzLjIgcGFja2V0IG93bnMgZXZlcnkgZHVyYWJsZQotYWRtaXNzaW9uLXN0YXRlIHRyYW5zaXRpb24gdXNlZCB0byByZXNvbHZlIGFuIGFiYW5kb25lZCBhZG1pc3Npb247IGBXb3JsZERpc3BhdGNoQ29udHJvbGAgaW4gQjQKLW93bnMgdGhlIHVzZXIvdG9vbC1mYWNpbmcgZXhhY3QgaW5zcGVjdC9jYW5jZWwgdmVyYiBhbmQgY29uc3VtZXMgdGhlIFJldGFpbmVkV29ya2VyUnVudGltZSByZXN1bHQKLXdpdGhvdXQgd3JpdGluZyBhZG1pc3Npb24gc3RhdGUgaXRzZWxmLgotCi1BbnkgbGF0ZXIgcmVzb2x1dGlvbiByZXF1ZXN0IG11c3QgZXhhY3Qtam9pbiBhbGwgb2YgdGhlIGZvbGxvd2luZyBkdXJhYmxlIHByZWNvbmRpdGlvbnMgYmVmb3JlIGEKLXN0YXRlIGFkdmFuY2U6IGlzc3VlciByZXF1ZXN0IGlkZW50aXR5LCBhZG1pc3Npb24tcmVjb3JkIGlkZW50aXR5IGFuZCByZXZpc2lvbiwgb3JjaGVzdHJhdGlvbgotc2Vzc2lvbiwgY3VycmVudCBleGFjdCBhdXRob3JpdHkgYW5kIHRoZSByZWxldmFudCBhZG1pc3Npb24tdG8tY3VycmVudCBhbmNlc3RyeSwgY3VycmVudCBwb2xpY3kKLWlkZW50aXR5IGFuZCBhdXRob3JpemF0aW9uLCByZXRhaW5lZCBwYXJ0aWNpcGFudCwgYW5kIHRoZSBleGFjdCBhZG1pc3Npb24gc3RhdGUgYmVpbmcgcmVzb2x2ZWQuCi1UaGUgY29tcGxldGUgY2Fub25pY2FsIFNwYXduIHJlcXVlc3QgcmVtYWlucyByZXF1aXJlZCB3aGVyZXZlciBmaW5nZXJwcmludCB2ZXJpZmljYXRpb24gb3IgZXhhY3QKLXJldHJ5IHNlbWFudGljcyBkZXBlbmQgb24gaXQuIFBJRCwgdGltZW91dCwgY2FsbGVyIHByZXNlbmNlIG9yIGRpc2FwcGVhcmFuY2UsIGhlbHBlciBzdGF0ZSwgc29ja2V0Ci1zdGF0ZSwgZW5kcG9pbnQgc3RhdGUsIEVPRiwgb2JzZXJ2ZXIgbG9zcywgYW5kIHByb2Nlc3MgbGl2ZW5lc3Mgc3VwcGx5IG5vIHJlc29sdXRpb24gYXV0aG9yaXR5LgotCi1SZXNvbHV0aW9uIGlzIGZvcndhcmQtb25seSBhZ2FpbnN0IGR1cmFibGUgcHJvdG9jb2wgdHJ1dGguIEl0IG11c3Qgbm90IGRlbGV0ZSBvciByb2xsIGJhY2sgYW4KLWFscmVhZHktY29tbWl0dGVkIFIwIGxpbmVhZ2UvcmVmL3JlZ2lzdHJhdGlvbiwgY2xhc3NpZnkgUjAgcmVqZWN0aW9uIGFzIGNhbmNlbGxhdGlvbiwgb3IgdHJlYXQKLXN0b3BwaW5nIGFuIGFscmVhZHktY3JlYXRlZCB3b3JrZXIgYXMgY2FuY2VsbGluZyBhIHBlbmRpbmcgYWRtaXNzaW9uLiBQYXJ0aWFsIG9yIGFscmVhZHktYXBwbGllZAotUjAgcmVnaXN0cmF0aW9uIGlzIHJlY29uY2lsZWQgdG8gaXRzIGV4YWN0IGFkbWlzc2lvbiByZWNvcmQuIEEgdHJhbnNwb3J0IGNsYWltIHdob3NlIGxhdW5jaCBvcgotcmVnaXN0cmF0aW9uIHJlc3VsdCBpcyBhbWJpZ3VvdXMgcmVtYWlucyBsaXZlIGFuZCBjYW5ub3QgZnJlZSBjYXBhY2l0eSB1bnRpbCBleGFjdCB0cmFuc3BvcnQgYW5kCi1ydW50aW1lIHRydXRoIGlzIHJlY29uY2lsZWQuIENyYXNoZXMgYmVmb3JlIGFuZCBhZnRlciByZXNvbHV0aW9uIGNvbnZlcmdlIG9uIHJldHJ5OyByZXBlYXRlZAotcmVzb2x1dGlvbiBleGFjdC1qb2lucyB0aGUgc2FtZSB0ZXJtaW5hbCByZXN1bHQ7IGFuZCB0aGUgbGl2ZSBhZG1pc3Npb24gY291bnQgZGVjcmVhc2VzIG9ubHkgYWZ0ZXIKLXRoYXQgdGVybWluYWwgcmVzdWx0IGlzIGR1cmFibHkgcHVibGlzaGVkLiBPbmx5IHRoZW4gbWF5IHRoZSBuZXh0IGVsaWdpYmxlIHF1ZXVlZCByZXF1ZXN0IGFjcXVpcmUKLXRoZSByZWdpc3RyYXRpb24gaGVhZCB1bmRlciB0aGUgZXhpc3RpbmcgZWFybGllc3Qtc2xvdCBydWxlLgotCi1CNCBtYXkgZnJlZXplIGFuZCByZXR1cm4gdGhlc2Ugc2VtYW50aWMgb3V0Y29tZSBjYXRlZ29yaWVzIHdpdGhvdXQgYWRkaW5nIHRoZW0gdG8gdGhlIEIzLjJhCi1ydW50aW1lIHNjaGVtYTogY2FuY2VsbGVkIGJlZm9yZSByZWdpc3RyYXRpb247IHJlZ2lzdGVyZWQgYmVmb3JlIHRyYW5zcG9ydDsgdHJhbnNwb3J0IGFtYmlndW91cwotb3IgY2FuY2VsbGF0aW9uIHBlbmRpbmc7IGFscmVhZHkgcm91dGFibGUgb3IgdGVybWluYWw7IGludmFsaWQgdGFyZ2V0OyBhbWJpZ3VvdXMgdGFyZ2V0OyBhbmQKLXBvbGljeSBkZW5pZWQuIFJlbWFpbmluZyBCMy4yIG11c3QgZmlyc3QgcHJvdmlkZSB0aGUgZHVyYWJsZSwgcmVzdGFydC1zYWZlIHJlc29sdXRpb24vcmVjb25jaWxpYXRpb24KLXByaW1pdGl2ZSB0aG9zZSBvdXRjb21lcyBjb25zdW1lLgotCi1UaGUgaG9zdC10by13b3JsZCBsYXVuY2ggY2FycmllciBpcyB0eXBlZCBhbmQgc3Vic3RpdHV0aW9uLXJlc2lzdGFudDoKLQotYGBgcnVzdAotc3RydWN0IFJldGFpbmVkV29ya2VyQWRtaXNzaW9uQ29tbWl0bWVudENhcnJpZXJWMSB7Ci0gICAgc2NoZW1hX3ZlcnNpb246IHUzMiwgLy8gZXhhY3RseSAxCi0gICAgYWxnb3JpdGhtOiBTdHJpbmcsICAgLy8gZXhhY3RseSAiaG1hYy1zaGEtMjU2IgotICAgIGtleV9pZDogU3RyaW5nLAotICAgIGRpZ2VzdF9oZXg6IFN0cmluZywKLX0KLQotc3RydWN0IFJldGFpbmVkV29ya2VyTGF1bmNoQXV0aG9yaXR5UHJvb2ZWMSB7Ci0gICAgc2NoZW1hX3ZlcnNpb246IHUzMiwgLy8gZXhhY3RseSAxCi0gICAgYXV0aG9yaXR5X3N0b3JlX2lkOiBTdHJpbmcsCi0gICAgaXNzdWVyX3JlcXVlc3RfaWQ6IFN0cmluZywKLSAgICBjYW5vbmljYWxfc3Bhd25fZmluZ2VycHJpbnQ6IFJldGFpbmVkV29ya2VyQWRtaXNzaW9uQ29tbWl0bWVudENhcnJpZXJWMSwKLSAgICByZWdpc3RyYXRpb25faWQ6IFN0cmluZywKLSAgICByZWdpc3RyYXRpb25fY29tbWl0bWVudDogQXV0aG9yaXR5T2JqZWN0Q29tbWl0bWVudFYxLAotICAgIGF1dGhvcml0eV9yZXZpc2lvbl9hZnRlcjogdTY0LAotICAgIGF1dGhvcml0eV9yZWNvcmRfY29tbWl0bWVudF9hZnRlcjogQXV0aG9yaXR5T2JqZWN0Q29tbWl0bWVudFYxLAotICAgIG9yY2hlc3RyYXRpb25fc2Vzc2lvbl9pZDogU3RyaW5nLAotICAgIGNhbGxlcl9wYXJ0aWNpcGFudF9pZDogU3RyaW5nLAotICAgIHJldGFpbmVkX3BhcnRpY2lwYW50X2lkOiBTdHJpbmcsCi0gICAgYm9vdHN0cmFwX3J1bl9pZDogU3RyaW5nLAotICAgIHRyYW5zcG9ydF9jbGFpbV9pZDogU3RyaW5nLAotICAgIGJhY2tlbmRfaWQ6IFN0cmluZywKLSAgICBwcm90b2NvbDogU3RyaW5nLAotICAgIHdvcmxkX2JpbmRpbmc6IFdvcmxkQmluZGluZ1YxLAotICAgIGN1cnJlbnRfcG9saWN5X3JlZl9pZDogU3RyaW5nLAotICAgIGN1cnJlbnRfcG9saWN5X3JldmlzaW9uOiBTdHJpbmcsCi0gICAgcmV0YWluZWRfd29ya2VyX3JlZl9pZDogU3RyaW5nLAotICAgIHJldGFpbmVkX3dvcmtlcl9jb21taXRtZW50OiBBdXRob3JpdHlPYmplY3RDb21taXRtZW50VjEsCi19Ci1gYGAKLQotQmVmb3JlIGVpdGhlciB0cmFuc3BvcnQgYnVpbGRlciBzZXJpYWxpemVzIGl0LCB0aGUgaG9zdCBleGFjdC1qb2lucyBldmVyeSBmaWVsZCB0byB0aGUgY3VycmVudCBIU0EKLXJlZ2lzdHJhdGlvbiBwcm9vZiwgYWRtaXNzaW9uIHJlY29yZC9maW5nZXJwcmludCwgYm91bmQgY2FsbGVyLCBkZXNjcmlwdG9yLCBwb2xpY3ksIGFuZCB3b3JsZC4gVGhlCi1jYXJyaWVyIGlzIHRoZSBleGFjdCBmaWVsZC1mb3ItZmllbGQsIHRyYW5zcG9ydC1uZXV0cmFsIGVxdWFsaXR5IHByb2plY3Rpb24gb2YgdGhlIGludGVybmFsCi1SZXRhaW5lZFdvcmtlckFkbWlzc2lvbkNvbW1pdG1lbnRWMTsgaXQgY29udGFpbnMgbm8gc2VjcmV0IGtleSBhbmQgZ3JhbnRzIG5vIEhTQSBvciBhZG1pc3Npb24KLW11dGF0aW9uIGF1dGhvcml0eS4gVGhlIHJlY2VpdmluZyB3b3JsZCBjb21wYXJlcyB0aGUgY2xvc2VkIGNhcnJpZXIgYW5kIGRpc3BhdGNoIGZpZWxkcyBvbmx5OyBpdAotZG9lcyBub3QgdmVyaWZ5IG9yIHJlaW50ZXJwcmV0IHRoZSBob3N0LW9ubHkgSE1BQyBpbnB1dC4gVGhlCi1vcHRpb25hbCBjYXJyaWVyIGZpZWxkIG9uIHRoZSBWMSBjb21wYXRpYmlsaXR5IHJlcXVlc3QgbWF5IHJlbWFpbiBhYnNlbnQgb25seSBvbiB0aGUgZXhwbGljaXRseQotcHJlLWFjdGl2YXRpb24gbGVnYWN5IHBhdGg7IGJvdGggQTEuMmEtUyBhdXRob3JpdHktbWFuYWdlZCBTcGF3biBwcm9kdWNlcnMgcmVxdWlyZSBpdCB3aXRoIG5vCi1mYWxsYmFjay4gYHRyYW5zcG9ydC1hcGktdHlwZXNgIHZhbGlkYXRlcyBpdHMgY2xvc2VkIHNoYXBlLiBQcm9kdWN0aW9uIG1lbWJlciBTcGF3biBlbnRlcnMKLWBTZXJ2aWNlOjpleGVjdXRlX3N0cmVhbWAuIEZvciBhdXRob3JpdHktbWFuYWdlZCBgU29tZShleGFjdCBwcm9vZilgLCB0aGF0IG1lbWJlciBicmFuY2ggY29tcGxldGVzCi10aGUgZXhpc3Rpbmcgc3RyaWN0IHByb29mL2Rpc3BhdGNoIGVxdWFsaXR5IHZhbGlkYXRpb24sIGR1cmFibHkgYWRvcHRzIHRoZSBleGFjdCBIU0EtYm91bmQgcGh5c2ljYWwKLXdvcmxkIHRocm91Z2ggQjMuMmEtV0EsIGFuZCBvbmx5IHRoZW4gY2FsbHMgYE1lbWJlclJ1bnRpbWVNYW5hZ2VyOjpsYXVuY2hgLCB3aGljaCByZXRhaW5zIGl0cyBvd24KLXZhbGlkYXRpb24gYmVmb3JlIHByb2Nlc3MgY3JlYXRpb24uIENvbXBhdGliaWxpdHkgYE5vbmVgIHJldGFpbnMgdGhlIGV4aXN0aW5nIGRpcmVjdCBwYXRoLgotYHdvcmxkLWFwaWAsIGBTZXJ2aWNlOjpleGVjdXRlYCwgYW5kIGBjb252ZXJ0X21lbWJlcl9kaXNwYXRjaF9yZXF1ZXN0YCBhcmUgbm90IHBhcnQgb2YgdGhpcyByb3V0ZS4KLVdvcmxkLXNlcnZpY2UgZG9lcyBub3QgbWludCBvciBhZHZhbmNlIEhTQSBvciBhZG1pc3Npb24gdHJ1dGg7IHRoZSBib3VuZCBob3N0IHZlcmlmaWNhdGlvbiBzdXBwbGllcwotYXV0aG9yaXR5IGF1dGhlbnRpY2l0eSwgQjMuMmEtV0Egc3VwcGxpZXMgcGh5c2ljYWwgb3duZXJzaGlwIG9ubHksIGFuZCB0aGUgbGF1bmNoIGJvdW5kYXJ5IHN1cHBsaWVzCi1leGFjdCBjYXJyaWVyL3JlcXVlc3QgZXF1YWxpdHkuCi0KLWAjW3NlcmRlKGRlZmF1bHQpXWAgb24gdGhlIG9wdGlvbmFsIGBNZW1iZXJEaXNwYXRjaFJlcXVlc3RWMWAgcHJvb2YgZmllbGQgcHJlc2VydmVzIHdpcmUKLWNvbXBhdGliaWxpdHkgYnkgc3VwcGx5aW5nIGBEZWZhdWx0OjpkZWZhdWx0KClgIG9ubHkgd2hlbiB0aGF0IGZpZWxkIGlzIGFic2VudCBkdXJpbmcKLWRlc2VyaWFsaXphdGlvbi4gSXQgZG9lcyBub3QgaW5pdGlhbGl6ZSBSdXN0IHN0cnVjdCBsaXRlcmFsczogZWFjaCBsaXRlcmFsIG11c3QgbmFtZSB0aGUgZmllbGQgKG9yCi11c2UgZXhwbGljaXQgUnVzdCBzdHJ1Y3QgdXBkYXRlIHN5bnRheCwgd2hpY2ggaXMgbm90IGF1dGhvcml6ZWQgZm9yIHRoZXNlIGZpeHR1cmVzKS4gVGhlcmVmb3JlIHRoZQotQjMuMmEgYWxsb3dsaXN0IGFkZGl0aW9uYWxseSBwZXJtaXRzIGVkaXRzIG9ubHkgaW4KLWBjcmF0ZXMvd29ybGQtc2VydmljZS90ZXN0cy9tZW1iZXJfcnVudGltZV93b3JsZF9wbGFjZW1lbnRfdjEucnNgLAotYGNyYXRlcy93b3JsZC1zZXJ2aWNlL3Rlc3RzL3N0cmVhbWVkX2V4ZWN1dGVfY2FuY2VsX3YxLnJzYCwgYW5kCi1gY3JhdGVzL3dvcmxkLXNlcnZpY2UvdGVzdHMvbWVtYmVyX3J1bnRpbWVfcmV0YWluZWRfbGlmZWN5Y2xlX3YxLnJzYCwgc29sZWx5IHRvIHNldCB0aGUgZmllbGQgdG8KLWBOb25lYCBpbiBleGlzdGluZyBleHBsaWNpdGx5IHByZS1hY3RpdmF0aW9uIG9yIGxlZ2FjeSBsaXRlcmFscyBhbmQgcHJvdmUgdW5jaGFuZ2VkIGNvbXBhdGliaWxpdHkuCi1FdmVyeSBvdGhlciBmaXh0dXJlIGlucHV0LCB0ZXN0IG5hbWUsIGFzc2VydGlvbiwgZXhwZWN0ZWQgb3V0Y29tZSwgYW5kIGV4cGVjdGVkIGVycm9yIHJlbWFpbnMKLXVuY2hhbmdlZC4gQW55IGV4aXN0aW5nIHRlc3QgdGhhdCBjbGFpbXMgdGhlIGF1dGhvcml0eS1tYW5hZ2VkIEIzLjJhIHJvdXRlIG11c3QgY2FycnkgYW4gZXhhY3QKLXZhbGlkIHByb29mIHRocm91Z2ggdGhlIGNhbm9uaWNhbCBwcm9kdWN0aW9uIHBhdGggaW5zdGVhZDsgYE5vbmVgIGlzIG5vdCB2YWxpZCB0aGVyZS4gVGhlc2UgdGVzdAotZmlsZXMgbWF5IG5vdCBpbnRyb2R1Y2UgYSBwcm9kdWN0aW9uIHBhdGgsIGhlbHBlciBkZWZhdWx0LCBmaXh0dXJlLW9ubHkgYXV0aG9yaXR5LCBhbHRlcm5hdGUKLXByb29mLCBhbHRlcm5hdGUgdHJhbnNwb3J0LCBzaWRlIHRhYmxlLCBvciB3ZWFrZW5lZCBhc3NlcnRpb24uIEF1dGhvcml0eS1tYW5hZ2VkIFNwYXduIHN0aWxsCi1yZXF1aXJlcyB0aGUgZXhhY3QgcHJvb2YuIFRoaXMgbWVjaGFuaWNhbCBhdXRob3JpemF0aW9uIGRpZCBub3QgaXRzZWxmIGNvbXBsZXRlIEIzLjJhOyB0aGUKLXJlY29yZGVkIEIzLjJhL0IzLjJhLVdBIHJlc3VsdCBpbiBgMDVgIG5vdyBkb2VzLgotCi1UaGF0IFJ1c3QtbGl0ZXJhbCByZXF1aXJlbWVudCBhbHNvIGF1dGhvcml6ZXMgZXhhY3RseSB0aHJlZSBwcm9kdWN0aW9uIGNvbXBhdGliaWxpdHkKLWluaXRpYWxpemVycyBhbmQgbm90aGluZyBlbHNlOiBgYnVpbGRfcnVuX3dvcmxkX3Rhc2tfdHJhbnNwb3J0X3JlcXVlc3RgIGFuZAotYGJ1aWxkX2Zvcmtfd29ybGRfd29ya2VyX3RyYW5zcG9ydF9yZXF1ZXN0YCBpbgotYGNyYXRlcy9zaGVsbC9zcmMvZXhlY3V0aW9uL29yY2hlc3RyYXRvcl93b3JsZF9kaXNwYXRjaC5yc2AsIHBsdXMgYGNvbnZlcnRfbWVtYmVyX2Rpc3BhdGNoYCBpbgotYGNyYXRlcy93b3JsZC1tYWMtbGltYS9zcmMvbGliLnJzYC4gRWFjaCBtYXkgb25seSBuYW1lIHRoZSBuZXcgb3B0aW9uYWwgcHJvb2YgZmllbGQgYXMgZXhwbGljaXQKLWBOb25lYDsgZXZlcnkgcHJlLWV4aXN0aW5nIGJhY2tlbmQsIHByb3RvY29sLCBydW4sIHNlc3Npb24sIHBhcnRpY2lwYW50LCBsaW5lYWdlLCB3b3JsZCwgcHJvbXB0LAotcnVudGltZSwgcm91dGUtc2VsZWN0aW9uLCBwb2xpY3ksIHBsYWNlbWVudCwgbGlmZWN5Y2xlLCBlcnJvciwgYW5kIG91dGNvbWUgZmllbGQgcmVtYWlucwotYnl0ZS1mb3ItYnl0ZSBhbmQgYmVoYXZpb3JhbGx5IHVuY2hhbmdlZC4gYE5vbmVgIGdyYW50cyBubyByZXRhaW5lZC13b3JrZXIgbGF1bmNoIGF1dGhvcml0eSB0bwotUnVuV29ybGRUYXNrLCBGb3JrV29ybGRXb3JrZXIsIG9yIHRoZSBtYWNPUyB3b3JsZC1hcGkgY29udmVyc2lvbi4gVGhpcyBpcyBub3QgbWFjT1MKLWF1dGhvcml0eS1tYW5hZ2VkIFNwYXduIGFkb3B0aW9uIGFuZCBhdXRob3JpemVzIG5vIGVkaXQgdG8gYGNyYXRlcy93b3JsZC1hcGkvc3JjL2xpYi5yc2AsCi1gU2VydmljZTo6ZXhlY3V0ZWAsIG9yIGBjb252ZXJ0X21lbWJlcl9kaXNwYXRjaF9yZXF1ZXN0YC4gTm8gZGVmYXVsdCBjb25zdHJ1Y3Rvciwgc2lkZSB0YWJsZSwKLWVudmlyb25tZW50IGNhcnJpZXIsIGFsdGVybmF0ZSB0cmFuc3BvcnQgcm91dGUsIG9yIGhpZGRlbiBwcm9vZiBzeW50aGVzaXMgbWF5IHJlcGxhY2UgdGhlc2UKLWV4cGxpY2l0IGluaXRpYWxpemVycy4gQm90aCBhdXRob3JpdHktbWFuYWdlZCBCMy4yYSBTcGF3biBwcm9kdWNlcnMgc3RpbGwgcmVxdWlyZQotYFNvbWUoZXhhY3QgUmV0YWluZWRXb3JrZXJMYXVuY2hBdXRob3JpdHlQcm9vZlYxKWAgZXhhY3Qtam9pbmVkIHRvIGFkbWlzc2lvbiBhbmQgUjAgdHJ1dGggYmVmb3JlCi1zZXJpYWxpemF0aW9uLCBhbmQgbWlzc2luZywgbWFsZm9ybWVkLCBvciBtaXNtYXRjaGVkIHByb29mIGZhaWxzIGJlZm9yZSBwcm9jZXNzIGNyZWF0aW9uLiBUaGlzCi1tZWNoYW5pY2FsIGF1dGhvcml6YXRpb24gZGlkIG5vdCBpdHNlbGYgY29tcGxldGUgQjMuMmE7IHRoZSByZWNvcmRlZCBCMy4yYS9CMy4yYS1XQSByZXN1bHQgaW4gYDA1YAotaXMgcmV2aWV3LWNsZWFuIHRocm91Z2ggYGQwYTcwNzI3YzJiZWMyYjJkNmZlMDc1NGVhNDY5YzQ2ODI2ODRkZGFgLgotCi1UaGUgc2FtZSBjb21waWxlci1yZXF1aXJlZCB3aWRlbmluZyBhdXRob3JpemVzIG9ubHkgc2V2ZW4gYWRkaXRpb25hbCBwcm9kdWN0aW9uLXN5bWJvbCBlZGl0cy4gSW4KLWBjcmF0ZXMvc2hlbGwvc3JjL2V4ZWN1dGlvbi9vcmNoZXN0cmF0b3Jfd29ybGRfZGlzcGF0Y2gucnNgLCBgZm9ya193b3JsZF93b3JrZXJgIGFuZAotYGNvbnRpbnVlX3dvcmxkX3dvcmtlcl9mb3JrX2NvbW1hbmRfYm9vdHN0cmFwX2FmdGVyX2RlbGl2ZXJ5YCBtYXkgb25seSBwYXNzIGV4cGxpY2l0IGBOb25lYCBmb3IgdGhlCi1vcHRpb25hbCByZXRhaW5lZC13b3JrZXIgYXV0aG9yaXR5IGNvbnRleHQgdG8gdGhlIHdpZGVuZWQgc3RyZWFtIGhlbHBlci4gSW4KLWBjcmF0ZXMvc2hlbGwvc3JjL3JlcGwvYXN5bmNfcmVwbC5yc2AsIGBhcHBseV9ncmVlbmZpZWxkX2hvc3Rfc3RhcnRfZnJvbV9hdXRob3JpdHlgLAotYHByZXBhcmVfaGlkZGVuX293bmVyX2hlbHBlcl9ydW50aW1lYCwKLWBzdGFydF9ob3N0X29yY2hlc3RyYXRvcl9ydW50aW1lX3dpdGhfcHJlcGFyZWRfcHJvbXB0X2FuZF90b29sYm94X3JlcXVlc3RfdHhgLAotYHByZXBhcmVfZm9ya19jaGlsZF9ydW50aW1lX3N0YXJ0dXBfZm9yX2Rlc2NyaXB0b3JgLCBhbmQKLWBwcmVwYXJlX21lbWJlcl9ydW50aW1lX3N0YXJ0dXBfZm9yX2Rlc2NyaXB0b3JgIG1heSBvbmx5IGluaXRpYWxpemUsIGRlc3RydWN0dXJlLCBvciBwcmVzZXJ2ZSB0aGUKLW5ldyBvcHRpb25hbCByZXRhaW5lZC13b3JrZXIgbGF1bmNoLWF1dGhvcml0eSBwcm9vZi9hZG1pc3Npb24gZmllbGRzIGFzIGBOb25lYC4gVGhvc2UgdmFsdWVzIGdyYW50Ci1ubyByZXRhaW5lZC13b3JrZXIgbGF1bmNoIGF1dGhvcml0eTogZm9yayBhbmQgZm9yayBjb250aW51YXRpb24gcmVtYWluIGNvbXBhdGliaWxpdHkgcGF0aHM7IEhvc3QKLVN0YXJ0IHJldGFpbnMgb25seSBpdHMgQTEuMmEtUyBob3N0LXNlc3Npb24gYXV0aG9yaXR5OyBoaWRkZW4tb3duZXItaGVscGVyIGJlaGF2aW9yIGlzIHVuY2hhbmdlZDsKLWFuZCBgcHJlcGFyZV9tZW1iZXJfcnVudGltZV9zdGFydHVwX2Zvcl9kZXNjcmlwdG9yYCByZW1haW5zIGxlZ2FjeSBwcmUtYWN0aXZhdGlvbiBwcmVwYXJhdGlvbiwKLWRpc3RpbmN0IGZyb20gdGhlIEIzLjJhLW9ubHkgYHByZXBhcmVfbWVtYmVyX3J1bnRpbWVfc3RhcnR1cF9mcm9tX2F1dGhvcml0eV9yZWdpc3RyYXRpb25gIHBhdGguCi1Pbmx5IHRoYXQgYXV0aG9yaXR5LXJlZ2lzdHJhdGlvbiBwcmVwYXJlciBtYXkgY29uc3RydWN0IHRoZSBwYWlyZWQKLWBTb21lKGV4YWN0IFJldGFpbmVkV29ya2VyTGF1bmNoQXV0aG9yaXR5UHJvb2ZWMSlgIGFuZCBleGFjdCBhZG1pc3Npb24gY29udGV4dC4gQW4KLWF1dGhvcml0eS1tYW5hZ2VkIHJldGFpbmVkIFNwYXduIHdpdGggZWl0aGVyIHZhbHVlIG1pc3NpbmcgZmFpbHMgY2xvc2VkIGFuZCBjYW5ub3QgZmFsbCBiYWNrIHRvIHRoZQotbGVnYWN5IHByZXBhcmVyIG9yIHJlaW50ZXJwcmV0IGdlbmVyaWMgYE5vbmVgIGFzIGNvbXBhdGliaWxpdHkuIFRoZSBzZXZlbiBleGNlcHRpb25zIGNoYW5nZSBubwotcGFja2V0IG93bmVyc2hpcCwgcG9saWN5LCBsaWZlY3ljbGUsIHRyYW5zcG9ydCwgaWRlbnRpdHksIGxpbmVhZ2UsIGVycm9yLCBvdXRjb21lLCBob3N0LXJ1bnRpbWUsCi1oZWxwZXIsIGxlZ2FjeS1tZW1iZXIsIG9yIGZvcmsgc2VtYW50aWNzIGFuZCBhdXRob3JpemUgbm8gb3RoZXIgc3RydWN0dXJhbCBjaGFuZ2UuIFRoaXMgbWVjaGFuaWNhbAotYXV0aG9yaXphdGlvbiBkaWQgbm90IGl0c2VsZiBjb21wbGV0ZSBCMy4yYTsgdGhlIHJlY29yZGVkIEIzLjJhL0IzLjJhLVdBIHJlc3VsdCBpbiBgMDVgIGlzCi1yZXZpZXctY2xlYW4gdGhyb3VnaCBgZDBhNzA3MjdjMmJlYzJiMmQ2ZmUwNzU0ZWE0NjljNDY4MjY4NGRkYWAuCitDYW5vbmljYWwgY29udGVudDogW2Bjb250cmFjdHMvZGVmZXJyZWQtcmV0YWluZWQtc3Bhd24tYWRtaXNzaW9uLXJlY292ZXJ5LWNvbnRyYWN0Lm1kI2RlZmVycmVkLXJldGFpbmVkLXNwYXduLWFkbWlzc2lvbi1yZWNvdmVyeS1jb250cmFjdGBdKGNvbnRyYWN0cy9kZWZlcnJlZC1yZXRhaW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29udHJhY3QubWQjZGVmZXJyZWQtcmV0YWluZWQtc3Bhd24tYWRtaXNzaW9uLXJlY292ZXJ5LWNvbnRyYWN0KS4KIAogIyMjIEIzLjJhLVdBIGBFeGFjdEJvdW5kV29ybGRPd25lcnNoaXBBZG9wdGlvblYxYAogCmRpZmYgLS1naXQgYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvaW5kZXgvUkVBRE1FLm1kIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZAppbmRleCBhMDBiODIxNzAuLjljZjJhMWYwNyAxMDA2NDQKLS0tIGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZAorKysgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvaW5kZXgvUkVBRE1FLm1kCkBAIC0xOCw2ICsxOCw3IEBACiB8IGBBVVRIT1JJVFlfUkVRVUlSRUQ6TUFDT1NfREVWX1BBUklUWWAgfCBnYXRlIHwgW2BnYXRlcy9hdXRob3JpdHktcmVxdWlyZWQtbWFjb3MtZGV2LXBhcml0eS5tZGBdKC4uL2dhdGVzL2F1dGhvcml0eS1yZXF1aXJlZC1tYWNvcy1kZXYtcGFyaXR5Lm1kKSB8IGxhbmUtbG9jYWwgYXV0aG9yaXR5LXJlcXVpcmVkIGdhdGU7IFBoYXNlIDEgZ3JhbnRzIG5vIFBoYXNlIDIgYXV0aG9yaXR5IHwgRG9lcyBub3QgYmxvY2sgb3IgYXV0aG9yaXplIHRoZSBMaW51eC1maXJzdCBydW50aW1lLXJlZmFjdG9yIHNlcXVlbmNlLiB8IFtgMDRgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kI2F1dGhvcml0eV9yZXF1aXJlZG1hY29zX2Rldl9wYXJpdHktY29udHJhY3QtMjAyNi0wOC0xOS1tYWNvcy1sYW5lKSB8CiB8IGBBVVRIT1JJVFlfUkVRVUlSRUQ6UlVOVElNRV9SRUZBQ1RPUl9SRUVOVFJZYCB8IGdhdGUgfCBbYGdhdGVzL2F1dGhvcml0eS1yZXF1aXJlZC1ydW50aW1lLXJlZmFjdG9yLXJlZW50cnkubWRgXSguLi9nYXRlcy9hdXRob3JpdHktcmVxdWlyZWQtcnVudGltZS1yZWZhY3Rvci1yZWVudHJ5Lm1kKSB8IGNsb3NlZCBkb2N1bWVudGF0aW9uL2NvbnRyb2wtcGxhbmUgc2VsZWN0aW9uIGdhdGUgfCBDbG9zZWQgYXMgc2VsZWN0aW9uIGhpc3Rvcnk7IEExLjMtUDEgaXMgdGhlIHNlcGFyYXRlIGFjdGl2ZSBwYWNrZXQuIHwgW2AwNGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjYXV0aG9yaXR5X3JlcXVpcmVkcnVudGltZV9yZWZhY3Rvcl9yZWVudHJ5LWNvbnRyYWN0LTIwMjYtMDgtMjAtY2xvc2VkLXNlbGVjdGlvbi1yZWNvcmQpIHwKIHwgYEhvc3RFeGVjdXRpb25FcGlzb2RlVjFgIHwgY29udHJhY3QgfCBbYGNvbnRyYWN0cy9ob3N0LWV4ZWN1dGlvbi1lcGlzb2RlLXYxLm1kYF0oLi4vY29udHJhY3RzL2hvc3QtZXhlY3V0aW9uLWVwaXNvZGUtdjEubWQpIHwgYGNhbm9uaWNhbCBleHRyYWN0ZWQgY29udHJhY3Qgb3duZXIgZm9yIHRoZSBjb21wbGV0ZSBzY2hlbWEsIGVwaXNvZGUta2luZCBsaXRlcmFscywgdHJhbnNwb3J0LXN0YXR1cyBsaXRlcmFscywgYW5kIHJ1bGVzIDHigJM2YCB8IFN1cGVyc2VkZXMgb25seSByb290LWNhbm9uaWNhbCBvd25lcnNoaXAgb2YgdGhlIGV4dHJhY3RlZCBgSG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWAgc3BhbjsgdGhlIGNvbW1pdHRlZCBwcmUtc3BhbiBjb21wYXRpYmlsaXR5IGFuY2hvcnMgYW5kIG5laWdoYm9yaW5nIGAyQWAvYDJCYCBvd25lcnMgcmVtYWluIHVuY2hhbmdlZC4gfCBbYDA0YF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMyLWhvc3RleGVjdXRpb25lcGlzb2RldjEpIHwKK3wgYERlZmVycmVkIHJldGFpbmVkLXNwYXduIGFkbWlzc2lvbiByZWNvdmVyeSBjb250cmFjdGAgfCBjb250cmFjdCB8IFtgY29udHJhY3RzL2RlZmVycmVkLXJldGFpbmVkLXNwYXduLWFkbWlzc2lvbi1yZWNvdmVyeS1jb250cmFjdC5tZGBdKC4uL2NvbnRyYWN0cy9kZWZlcnJlZC1yZXRhaW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29udHJhY3QubWQpIHwgYGNhbm9uaWNhbCBleHRyYWN0ZWQgb3duZXIgZm9yIHRoZSBjb21wbGV0ZSBkZWZlcnJlZCByZXRhaW5lZC1zcGF3biBhZG1pc3Npb24gcmVjb3ZlcnkgY29udHJhY3QsIHBhaXJlZCBWMSByZWNvdmVyeSBzY2hlbWFzLCBjb21wYXRpYmlsaXR5IGJvdW5kYXJpZXMsIHJvdXRlIGV4Y2x1c2lvbnMsIGFsbG93bGlzdHMsIGFuZCByZWNvcmRlZC1yZXN1bHQgbGltaXRzYCB8IFN1cGVyc2VkZXMgb25seSByb290LWNhbm9uaWNhbCBvd25lcnNoaXAgb2YgdGhlIGV4dHJhY3RlZCBgRGVmZXJyZWQgcmV0YWluZWQtc3Bhd24gYWRtaXNzaW9uIHJlY292ZXJ5IGNvbnRyYWN0YCBzcGFuOyB0aGUgdHdvIHByZXJlcXVpc2l0ZSBjb21wYXRpYmlsaXR5IGFuY2hvcnMsIGBIb3N0RXhlY3V0aW9uRXBpc29kZVYxYCwgYW5kIHRoZSBmb2xsb3dpbmcgYEIzLjJhLVdBIEV4YWN0Qm91bmRXb3JsZE93bmVyc2hpcEFkb3B0aW9uVjFgIG93bmVyIHJlbWFpbiB1bmNoYW5nZWQuIHwgW2AwNGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjZGVmZXJyZWQtcmV0YWluZWQtc3Bhd24tYWRtaXNzaW9uLXJlY292ZXJ5LWNvbnRyYWN0KSB8CiB8IGBzaGFyZWQtdGFyZ2V0LWFyY2hpdGVjdHVyZWAgfCBhcmNoaXRlY3R1cmUgaW5kZXggfCBbYGFyY2hpdGVjdHVyZS9SRUFETUUubWRgXSguLi9hcmNoaXRlY3R1cmUvUkVBRE1FLm1kKSB8IGBub24tYXV0aG9yaXRhdGl2ZSBuYXZpZ2F0aW9uIGZvciB0aGUgZXh0cmFjdGVkIGV4ZWN1dGl2ZSB0YXJnZXQgZGVjaXNpb24sIGF1dGhvcml0eSBtYXAsIG51bWJlcmVkIGludmFyaWFudHMsIGFuZCBzdGFibGUgcmV2aWV3IHF1ZXN0aW9uYCB8IFN1cGVyc2VkZXMgb25seSByb290LWNhbm9uaWNhbCBvd25lcnNoaXAgb2YgdGhlIGV4dHJhY3RlZCBzaGFyZWQgYXJjaGl0ZWN0dXJlIHNwYW5zOyBwYWNrZXQtZmFtaWx5LWxvY2FsIEQ1L0Q2IGZvcndhcmRlcnMgcmVtYWluIGF0IHRoZWlyIGV4aXN0aW5nIG93bmVycy4gfCBbYDAxYF0oLi4vMDEtdGFyZ2V0LWFyY2hpdGVjdHVyZS5tZCNleGVjdXRpdmUtZGVjaXNpb24pLCBbYDAxYF0oLi4vMDEtdGFyZ2V0LWFyY2hpdGVjdHVyZS5tZCNhdXRob3JpdHktbWFwKSwgW2AwMWBdKC4uLzAxLXRhcmdldC1hcmNoaXRlY3R1cmUubWQjbm9uLW5lZ290aWFibGUtaW52YXJpYW50cyksIFtgMDFgXSguLi8wMS10YXJnZXQtYXJjaGl0ZWN0dXJlLm1kI3Jldmlldy1xdWVzdGlvbikgfAogfCBgc2hhcmVkLXNlYW0tY3Jvc3N3YWxrYCB8IHNlYW0gaW5kZXggfCBbYHNlYW1zL1JFQURNRS5tZGBdKC4uL3NlYW1zL1JFQURNRS5tZCkgfCBgY2Fub25pY2FsIHNoYXJlZCBzZWFtLWNyb3Nzd2FsayBydWxlcyBwbHVzIGV4dHJhY3RlZCBzZWFtLWZhbWlseSBuYXZpZ2F0aW9uIGZvciBob3N0L3Nlc3Npb24sIHBlcnNpc3RlbmNlL2NvbXBhdGliaWxpdHksIGRpc3BhdGNoL2VwaXNvZGUgdHJhbnNwb3J0LCBwb2xpY3kvbmFycm93aW5nLCBydW50aW1lLWV2ZW50L3JlY2VpcHQvc3VwZXJ2aXNpb24vcmV0YWluZWQtcnVudGltZSwgb2JsaWdhdGlvbnMvaG9zdCByZS1lbmdhZ2VtZW50LCBjb25maWd1cmF0aW9uL2dhdGV3YXkgYWRvcHRpb24sIGFuZCBVQUEvcHJvdmlkZXIgcmVhbGl6YXRpb24vc2lkZS1lZmZlY3QgbWVkaWF0aW9uYCB8IFN1cGVyc2VkZXMgb25seSB0aGUgZXh0cmFjdGVkIHJvb3QgcmVhZGluZy1ydWxlL2NsYXNzaWZpY2F0aW9uIHNwYW5zIGFuZCB0aGUgZXh0cmFjdGVkIGhvc3Qvc2Vzc2lvbiBhdXRob3JpdHksIHBlcnNpc3RlbmNlL2NvbXBhdGliaWxpdHksIGRpc3BhdGNoL2VwaXNvZGUgdHJhbnNwb3J0LCBwb2xpY3kvbmFycm93aW5nLCBydW50aW1lLWV2ZW50L3JlY2VpcHQvc3VwZXJ2aXNpb24vcmV0YWluZWQtcnVudGltZSwgb2JsaWdhdGlvbnMvaG9zdCByZS1lbmdhZ2VtZW50LCBjb25maWd1cmF0aW9uL2dhdGV3YXkgYWRvcHRpb24sIGFuZCBVQUEvcHJvdmlkZXIgcmVhbGl6YXRpb24vc2lkZS1lZmZlY3QgbWVkaWF0aW9uIGZhbWlseSByb3dzOyBBMCBhbmQgdGhlIGV4aXN0aW5nIEQ2IGBIb3N0U2Vzc2lvbkF1dGhvcml0eWAsIGBXb3JsZFdvcmtlck1lc3NhZ2luZ1Byb3RvY29sYCwgYW5kIGBPYmxpZ2F0aW9uTGVkZ2VyYCBjb21wYXRpYmlsaXR5IHJvd3MgcmVtYWluIGF0IHRoZWlyIGN1cnJlbnQgb3duZXJzLiB8IFtgMDJgXSguLi8wMi1zZWFtLWNyb3Nzd2Fsay5tZCNyZWFkaW5nLXJ1bGUpLCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjYS1ob3N0LWF1dGhvcml0eS1hbmQtaW5ncmVzcyksIFtgMDJgXSguLi8wMi1zZWFtLWNyb3Nzd2Fsay5tZCNiLWRpc3BhdGNoLXBvbGljeS1yZWNlaXB0cy1hbmQtcmV0YWluZWQtcnVudGltZSksIFtgMDJgXSguLi8wMi1zZWFtLWNyb3Nzd2Fsay5tZCNjLW9ibGlnYXRpb25zLWFuZC1ob3N0LXJlLWVuZ2FnZW1lbnQpLCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjZC11YWEtcmVhbGl6YXRpb24tcHJvamVjdGlvbi1hbmQtc2lkZS1lZmZlY3QtbWVkaWF0aW9uKSwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1kI2NsYXNzaWZpY2F0aW9uLWNvbnNlcXVlbmNlcykgfAogfCBgc2hhcmVkLXNsaWNlLW1hcGAgfCBzbGljZSBpbmRleCB8IFtgc2xpY2VzL1JFQURNRS5tZGBdKC4uL3NsaWNlcy9SRUFETUUubWQpIHwgYGNhbm9uaWNhbCBzaGFyZWQgc2VxdWVuY2luZy9kZXBlbmRlbmN5IGFuZCBzbGljZS1jbG9zZW91dCBvd25lciBwbHVzIG5vbi1hdXRob3JpdGF0aXZlIEQ5IFRyYWNrIEHigJNFIG5hdmlnYXRpb24gYW5kIEExL0ExLjQgcHJvamVjdGlvbiBpbmRleGAgfCBTdXBlcnNlZGVzIG9ubHkgcm9vdC1jYW5vbmljYWwgb3duZXJzaGlwIG9mIHRoZSBleHRyYWN0ZWQgc2hhcmVkIHNlcXVlbmNpbmcvY2xvc2VvdXQgc3BhbnMgcGx1cyB0aGUgZXh0cmFjdGVkIFRyYWNrIEHigJNFIGFuZCBBMS9BMS40IHByb2plY3Rpb24gc3BhbnM7IGNvbnRyb2xsaW5nIHNjaGVkdWxlIGF1dGhvcml0eSByZW1haW5zIHdpdGggZGVjaXNpb25zLCBwYWNrZXRzLCBnYXRlcywgYW5kIGBjdXJyZW50Lm1kYC4gfCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3NlcXVlbmNpbmctcnVsZXMpLCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3RyYWNrLWEtLWF1dGhvcml0eS1hbmQtc3VyZmFjZS1uZXV0cmFsaXR5KSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCNhMS1ib3VuZGVkLXBhY2tldC1kZWNvbXBvc2l0aW9uKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCN0cmFjay1iLS13b3JsZC1kaXNwYXRjaC1yZWNlaXB0cy1zdXBlcnZpc2lvbi1hbmQtY2FuY2VsKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCN0cmFjay1jLS1vYmxpZ2F0aW9ucy1pbmJveC1hdXRvLWF0dGFjaC1hbmQtcm91dGVyLWF0dGFjaCksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjdHJhY2stZC0tdWFhLWV4ZWN1dGlvbi1lbnZlbG9wZS1hbmQtc2lkZS1lZmZlY3QtbWVkaWF0aW9uKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCN0cmFjay1lLS1kaXNwYXRjaC1zY29wZWQtcG9saWN5LW5hcnJvd2luZy1hbmQtY29uZmlnLXByb2plY3Rpb24pLCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3NsaWNlLWNsb3Nlb3V0LW1pbmltdW0pIHwKZGlmZiAtLWdpdCBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJhY3Rpb24tbGVkZ2VyLm1kCmluZGV4IGM0NmE5ZjM4ZC4uNjE5YzE4MjZjIDEwMDY0NAotLS0gYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJhY3Rpb24tbGVkZ2VyLm1kCisrKyBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9taWdyYXRpb24vZXh0cmFjdGlvbi1sZWRnZXIubWQKQEAgLTE4NSwzICsxODUsNCBAQCBUaGlzIGxlZGdlciByZWNvcmRzIGNvbnRlbnQtcHJlc2VydmluZyBhdXRob3JpdHkgdHJhbnNmZXJzIHdoaWxlIHJldGFpbmluZyByZXF1aQogfCBEOSB8IFtgMDMtcGhhc2Utc2xpY2UtbWFwLm1kYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kKSB8IEU0IOKAlCBIb3N0LXZpc2libGUgd3JpdGUvc3luYyBjb250cmFjdCB8IFtgdHJhY2stZS0tZGlzcGF0Y2gtc2NvcGVkLXBvbGljeS1uYXJyb3dpbmctYW5kLWNvbmZpZy1wcm9qZWN0aW9uYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3RyYWNrLWUtLWRpc3BhdGNoLXNjb3BlZC1wb2xpY3ktbmFycm93aW5nLWFuZC1jb25maWctcHJvamVjdGlvbikgfCBsaW5lIDIyMyB8IGBjOThhNWRmM2YwOTczNWFiYzVjYmVmZWMyODVjN2I1ZGM1NWZjYTVjNjdjZmRmNWJmNmJkM2Y0NThkMDE3ODkzYCB8IFtgLi4vc2xpY2VzL2U0LWhvc3QtdmlzaWJsZS13cml0ZS1zeW5jLWNvbnRyYWN0Lm1kYF0oLi4vc2xpY2VzL2U0LWhvc3QtdmlzaWJsZS13cml0ZS1zeW5jLWNvbnRyYWN0Lm1kKSB8IHNsaWNlIHJvdyB8IGNhbm9uaWNhbCBkZXN0aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgdGFibGUgcm93IHwgbm9uZSB8IHJlc3RvcmUgdGhlIGV4YWN0IGV4dHJhY3RlZCBEOSBzb3VyY2UgYm9kaWVzL3Jvd3MgaW4gYDAzLXBoYXNlLXNsaWNlLW1hcC5tZGA7IHJlbW92ZSBgc2xpY2VzL2AgYW5kIGFsbCBEOS1hZGRlZCBmaWxlcyBiZW5lYXRoIGl0OyByZW1vdmUgdGhlIGBzaGFyZWQtc2xpY2UtbWFwYCBpbmRleCByb3cgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsgcmVtb3ZlIGFsbCBtYXRjaGluZyBEOSBsZWRnZXIgZW50cmllczsgbGVhdmUgRDXigJNEOCBjYW5vbmljYWwgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGFuZCBgcmV2aWV3LWNvbnRyb2wvYCB1bmNoYW5nZWQgfAogfCBEOSB8IFtgMDMtcGhhc2Utc2xpY2UtbWFwLm1kYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kKSB8IEExLjQg4oCUIGJvdW5kZWQgYXV0by1hdHRhY2ggcHJvZHVjZXIgYWRvcHRpb24gYW5kIHJlZ3Jlc3Npb24gY2xvc3VyZSB8IFtgYTEtYm91bmRlZC1wYWNrZXQtZGVjb21wb3NpdGlvbmBdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCNhMS1ib3VuZGVkLXBhY2tldC1kZWNvbXBvc2l0aW9uKSB8IGxpbmUgMTQ1IHwgYDU4ZDc5ODJiMjNhN2IyZjVmNDFmM2UwYjUxNzA5OTZlMGYzYjMzMWRmYmJjYmI1ODc1ZDAyZTgwMWJkNWQ0MDhgIHwgW2AuLi9zbGljZXMvdGFza3MvYTEtNC1hdXRvLWF0dGFjaC1wcm9kdWNlci1hZG9wdGlvbi5tZGBdKC4uL3NsaWNlcy90YXNrcy9hMS00LWF1dG8tYXR0YWNoLXByb2R1Y2VyLWFkb3B0aW9uLm1kKSB8IHRhc2sgcm93IHwgY2Fub25pY2FsIGRlc3RpbmF0aW9uOyBzb3VyY2UgY29tcGF0aWJpbGl0eSB0YWJsZSByb3cgfCBub25lIHwgcmVzdG9yZSB0aGUgZXhhY3QgZXh0cmFjdGVkIEQ5IHNvdXJjZSBib2RpZXMvcm93cyBpbiBgMDMtcGhhc2Utc2xpY2UtbWFwLm1kYDsgcmVtb3ZlIGBzbGljZXMvYCBhbmQgYWxsIEQ5LWFkZGVkIGZpbGVzIGJlbmVhdGggaXQ7IHJlbW92ZSB0aGUgYHNoYXJlZC1zbGljZS1tYXBgIGluZGV4IHJvdyBmcm9tIGBpbmRleC9SRUFETUUubWRgOyByZW1vdmUgYWxsIG1hdGNoaW5nIEQ5IGxlZGdlciBlbnRyaWVzOyBsZWF2ZSBENeKAk0Q4IGNhbm9uaWNhbCBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYW5kIGByZXZpZXctY29udHJvbC9gIHVuY2hhbmdlZCB8CiB8IEQxMCB8IFtgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQpIHwgMi4gYEhvc3RFeGVjdXRpb25FcGlzb2RlVjFgIHwgW2AyLWhvc3RleGVjdXRpb25lcGlzb2RldjFgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzItaG9zdGV4ZWN1dGlvbmVwaXNvZGV2MSkgfCBsaW5lcyA1MeKAkzEwMCB8IGAyOGU4NmUyZTdlMjliNDU4MjgwODc1ZjkyMWNhMzI3YTQ2OTYyMWFjOTZiZDdlNTkyYTBhMTc2MDI3YWEwMTE3YCB8IFtgLi4vY29udHJhY3RzL2hvc3QtZXhlY3V0aW9uLWVwaXNvZGUtdjEubWRgXSguLi9jb250cmFjdHMvaG9zdC1leGVjdXRpb24tZXBpc29kZS12MS5tZCkgfCBjb250cmFjdCB8IGNhbm9uaWNhbCBkZXN0aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9yIHwgbm9uZSB8IHJlc3RvcmUgdGhlIGV4YWN0IDE2NDUtYnl0ZSBzb3VyY2Ugc3BhbiBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMyLWhvc3RleGVjdXRpb25lcGlzb2RldjFgOyByZW1vdmUgYGNvbnRyYWN0cy9ob3N0LWV4ZWN1dGlvbi1lcGlzb2RlLXYxLm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgSG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWAgcm93IGZyb20gYGluZGV4L1JFQURNRS5tZGA7IHJlbW92ZSB0aGlzIEQxMCBsZWRnZXIgZW50cnk7IGxlYXZlIHRoZSBjb21taXR0ZWQgcHJlLXNwYW4gY29tcGF0aWJpbGl0eSBhbmNob3JzLCBEMy9ENeKAk0Q5IG93bmVycywgYGluZGV4L2N1cnJlbnQubWRgLCBgcmV2aWV3LWNvbnRyb2wvYCwgYWxsIG90aGVyIEQxMCB1bml0cywgYW5kIGV2ZXJ5IHBhdGggb3V0c2lkZSB0aGUgZm91ci1wYXRoIGZlbmNlIHVuY2hhbmdlZCB8Cit8IEQxMCB8IFtgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQpIHwgRGVmZXJyZWQgcmV0YWluZWQtc3Bhd24gYWRtaXNzaW9uIHJlY292ZXJ5IGNvbnRyYWN0IHwgW2BkZWZlcnJlZC1yZXRhaW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29udHJhY3RgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kI2RlZmVycmVkLXJldGFpbmVkLXNwYXduLWFkbWlzc2lvbi1yZWNvdmVyeS1jb250cmFjdCkgfCBsaW5lcyA3MeKAkzIxMyB8IGBkOTAzMjk5YmRkMzQ1MzE2ZmQwOTQzZDI4YWJjMzAwYTFjZWYwMDFjZTA3NDQ3NDZmNzJmNTM2MDdlYjY0YzEyYCB8IFtgLi4vY29udHJhY3RzL2RlZmVycmVkLXJldGFpbmVkLXNwYXduLWFkbWlzc2lvbi1yZWNvdmVyeS1jb250cmFjdC5tZGBdKC4uL2NvbnRyYWN0cy9kZWZlcnJlZC1yZXRhaW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29udHJhY3QubWQpIHwgY29udHJhY3QgfCBjYW5vbmljYWwgZGVzdGluYXRpb247IHNvdXJjZSBjb21wYXRpYmlsaXR5IGFuY2hvciB8IG5vbmUgfCByZXN0b3JlIHRoZSBleGFjdCA5ODk2LWJ5dGUgc291cmNlIHNwYW4gYXQgYDA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjZGVmZXJyZWQtcmV0YWluZWQtc3Bhd24tYWRtaXNzaW9uLXJlY292ZXJ5LWNvbnRyYWN0YDsgcmVtb3ZlIGBjb250cmFjdHMvZGVmZXJyZWQtcmV0YWluZWQtc3Bhd24tYWRtaXNzaW9uLXJlY292ZXJ5LWNvbnRyYWN0Lm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgRGVmZXJyZWQgcmV0YWluZWQtc3Bhd24gYWRtaXNzaW9uIHJlY292ZXJ5IGNvbnRyYWN0YCByb3cgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsgcmVtb3ZlIHRoaXMgRDEwIGxlZGdlciBlbnRyeTsgbGVhdmUgYEhvc3RFeGVjdXRpb25FcGlzb2RlVjFgLCB0aGUgdHdvIHByZXJlcXVpc2l0ZSBjb21wYXRpYmlsaXR5IGFuY2hvcnMsIGFsbCBEMy9ENeKAk0Q5IG93bmVycywgYGluZGV4L2N1cnJlbnQubWRgLCBgcmV2aWV3LWNvbnRyb2wvYCwgbGF0ZXIgRDEwIHVuaXRzLCBhbmQgZXZlcnkgcGF0aCBvdXRzaWRlIHRoZSBmb3VyLXBhdGggZmVuY2UgdW5jaGFuZ2VkIHwKZGlmZiAtLWdpdCBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9jb250cmFjdHMvZGVmZXJyZWQtcmV0YWluZWQtc3Bhd24tYWRtaXNzaW9uLXJlY292ZXJ5LWNvbnRyYWN0Lm1kIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2NvbnRyYWN0cy9kZWZlcnJlZC1yZXRhaW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29udHJhY3QubWQKbmV3IGZpbGUgbW9kZSAxMDA2NDQKaW5kZXggMDAwMDAwMDAwLi41MzUyZWI4N2QKLS0tIC9kZXYvbnVsbAorKysgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvY29udHJhY3RzL2RlZmVycmVkLXJldGFpbmVkLXNwYXduLWFkbWlzc2lvbi1yZWNvdmVyeS1jb250cmFjdC5tZApAQCAtMCwwICsxLDE1MSBAQAorKipLaW5kOioqIGNvbnRyYWN0CisqKlN0YXR1czoqKiBjYW5vbmljYWwKKyoqQ2Fub25pY2FsIGZvcjoqKiBjb21wbGV0ZSBleHRyYWN0ZWQgZGVmZXJyZWQgcmV0YWluZWQtc3Bhd24gYWRtaXNzaW9uIHJlY292ZXJ5IGNvbnRyYWN0IGluY2x1ZGluZyB0aGUgcmVjb3ZlcnkgcHJlY29uZGl0aW9ucywgcGFpcmVkIGBSZXRhaW5lZFdvcmtlckFkbWlzc2lvbkNvbW1pdG1lbnRDYXJyaWVyVjFgIC8gYFJldGFpbmVkV29ya2VyTGF1bmNoQXV0aG9yaXR5UHJvb2ZWMWAgc2NoZW1hcywgY29tcGF0aWJpbGl0eSBib3VuZGFyaWVzLCByb3V0ZSBleGNsdXNpb25zLCBhbGxvd2xpc3RzLCBhbmQgcmVjb3JkZWQtcmVzdWx0IGxpbWl0cworKipTb3VyY2UgcHJvdmVuYW5jZToqKiBleHRyYWN0ZWQgYnl0ZS1mb3ItYnl0ZSBmcm9tIFtgLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCNkZWZlcnJlZC1yZXRhaW5lZC1zcGF3bi1hZG1pc3Npb24tcmVjb3ZlcnktY29udHJhY3RgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kI2RlZmVycmVkLXJldGFpbmVkLXNwYXduLWFkbWlzc2lvbi1yZWNvdmVyeS1jb250cmFjdCksIGJhc2VsaW5lIGxpbmVzIDcx4oCTMjEzOyB0aGUgZXhhY3QgOTg5Ni1ieXRlIHNvdXJjZSBib2R5IGlzIHByZXNlcnZlZCBiZXR3ZWVuIHRoZSBib3VuZGFyeSBtYXJrZXJzIGJlbG93CisqKkJhc2VsaW5lIHNwYW4gU0hBLTI1NjoqKiBgZDkwMzI5OWJkZDM0NTMxNmZkMDk0M2QyOGFiYzMwMGExY2VmMDAxY2UwNzQ0NzQ2ZjcyZjUzNjA3ZWI2NGMxMmAKKworPCEtLSBleGFjdC1leHRyYWN0ZWQtYm9keTpzdGFydCAtLT4KKyMjIyBEZWZlcnJlZCByZXRhaW5lZC1zcGF3biBhZG1pc3Npb24gcmVjb3ZlcnkgY29udHJhY3QKKworQjMuMmEgZGVsaWJlcmF0ZWx5IHN0b3BzIGF0IGNvbnNlcnZhdGl2ZSBleGFjdC1yZXRyeSBiZWhhdmlvci4gSXQgYWRkcyBubyBjYW5jZWxsYXRpb24sCithYmFuZG9ubWVudCwgZXhwaXJ5LCBvciBsaXZlbmVzcy1iYXNlZCByZXNvbHV0aW9uIHByb3RvY29sIGFuZCBhZGRzIG5vIHJ1bnRpbWUgZW51bSwgc2NoZW1hIGZpZWxkLAorb3IgcGVyc2lzdGVkIHJlcXVlc3QgcHJlaW1hZ2UgZm9yIGZ1dHVyZSByZWNvdmVyeS4gVGhlIHJlbWFpbmluZyBCMy4yIHBhY2tldCBvd25zIGV2ZXJ5IGR1cmFibGUKK2FkbWlzc2lvbi1zdGF0ZSB0cmFuc2l0aW9uIHVzZWQgdG8gcmVzb2x2ZSBhbiBhYmFuZG9uZWQgYWRtaXNzaW9uOyBgV29ybGREaXNwYXRjaENvbnRyb2xgIGluIEI0Citvd25zIHRoZSB1c2VyL3Rvb2wtZmFjaW5nIGV4YWN0IGluc3BlY3QvY2FuY2VsIHZlcmIgYW5kIGNvbnN1bWVzIHRoZSBSZXRhaW5lZFdvcmtlclJ1bnRpbWUgcmVzdWx0Cit3aXRob3V0IHdyaXRpbmcgYWRtaXNzaW9uIHN0YXRlIGl0c2VsZi4KKworQW55IGxhdGVyIHJlc29sdXRpb24gcmVxdWVzdCBtdXN0IGV4YWN0LWpvaW4gYWxsIG9mIHRoZSBmb2xsb3dpbmcgZHVyYWJsZSBwcmVjb25kaXRpb25zIGJlZm9yZSBhCitzdGF0ZSBhZHZhbmNlOiBpc3N1ZXIgcmVxdWVzdCBpZGVudGl0eSwgYWRtaXNzaW9uLXJlY29yZCBpZGVudGl0eSBhbmQgcmV2aXNpb24sIG9yY2hlc3RyYXRpb24KK3Nlc3Npb24sIGN1cnJlbnQgZXhhY3QgYXV0aG9yaXR5IGFuZCB0aGUgcmVsZXZhbnQgYWRtaXNzaW9uLXRvLWN1cnJlbnQgYW5jZXN0cnksIGN1cnJlbnQgcG9saWN5CitpZGVudGl0eSBhbmQgYXV0aG9yaXphdGlvbiwgcmV0YWluZWQgcGFydGljaXBhbnQsIGFuZCB0aGUgZXhhY3QgYWRtaXNzaW9uIHN0YXRlIGJlaW5nIHJlc29sdmVkLgorVGhlIGNvbXBsZXRlIGNhbm9uaWNhbCBTcGF3biByZXF1ZXN0IHJlbWFpbnMgcmVxdWlyZWQgd2hlcmV2ZXIgZmluZ2VycHJpbnQgdmVyaWZpY2F0aW9uIG9yIGV4YWN0CityZXRyeSBzZW1hbnRpY3MgZGVwZW5kIG9uIGl0LiBQSUQsIHRpbWVvdXQsIGNhbGxlciBwcmVzZW5jZSBvciBkaXNhcHBlYXJhbmNlLCBoZWxwZXIgc3RhdGUsIHNvY2tldAorc3RhdGUsIGVuZHBvaW50IHN0YXRlLCBFT0YsIG9ic2VydmVyIGxvc3MsIGFuZCBwcm9jZXNzIGxpdmVuZXNzIHN1cHBseSBubyByZXNvbHV0aW9uIGF1dGhvcml0eS4KKworUmVzb2x1dGlvbiBpcyBmb3J3YXJkLW9ubHkgYWdhaW5zdCBkdXJhYmxlIHByb3RvY29sIHRydXRoLiBJdCBtdXN0IG5vdCBkZWxldGUgb3Igcm9sbCBiYWNrIGFuCithbHJlYWR5LWNvbW1pdHRlZCBSMCBsaW5lYWdlL3JlZi9yZWdpc3RyYXRpb24sIGNsYXNzaWZ5IFIwIHJlamVjdGlvbiBhcyBjYW5jZWxsYXRpb24sIG9yIHRyZWF0CitzdG9wcGluZyBhbiBhbHJlYWR5LWNyZWF0ZWQgd29ya2VyIGFzIGNhbmNlbGxpbmcgYSBwZW5kaW5nIGFkbWlzc2lvbi4gUGFydGlhbCBvciBhbHJlYWR5LWFwcGxpZWQKK1IwIHJlZ2lzdHJhdGlvbiBpcyByZWNvbmNpbGVkIHRvIGl0cyBleGFjdCBhZG1pc3Npb24gcmVjb3JkLiBBIHRyYW5zcG9ydCBjbGFpbSB3aG9zZSBsYXVuY2ggb3IKK3JlZ2lzdHJhdGlvbiByZXN1bHQgaXMgYW1iaWd1b3VzIHJlbWFpbnMgbGl2ZSBhbmQgY2Fubm90IGZyZWUgY2FwYWNpdHkgdW50aWwgZXhhY3QgdHJhbnNwb3J0IGFuZAorcnVudGltZSB0cnV0aCBpcyByZWNvbmNpbGVkLiBDcmFzaGVzIGJlZm9yZSBhbmQgYWZ0ZXIgcmVzb2x1dGlvbiBjb252ZXJnZSBvbiByZXRyeTsgcmVwZWF0ZWQKK3Jlc29sdXRpb24gZXhhY3Qtam9pbnMgdGhlIHNhbWUgdGVybWluYWwgcmVzdWx0OyBhbmQgdGhlIGxpdmUgYWRtaXNzaW9uIGNvdW50IGRlY3JlYXNlcyBvbmx5IGFmdGVyCit0aGF0IHRlcm1pbmFsIHJlc3VsdCBpcyBkdXJhYmx5IHB1Ymxpc2hlZC4gT25seSB0aGVuIG1heSB0aGUgbmV4dCBlbGlnaWJsZSBxdWV1ZWQgcmVxdWVzdCBhY3F1aXJlCit0aGUgcmVnaXN0cmF0aW9uIGhlYWQgdW5kZXIgdGhlIGV4aXN0aW5nIGVhcmxpZXN0LXNsb3QgcnVsZS4KKworQjQgbWF5IGZyZWV6ZSBhbmQgcmV0dXJuIHRoZXNlIHNlbWFudGljIG91dGNvbWUgY2F0ZWdvcmllcyB3aXRob3V0IGFkZGluZyB0aGVtIHRvIHRoZSBCMy4yYQorcnVudGltZSBzY2hlbWE6IGNhbmNlbGxlZCBiZWZvcmUgcmVnaXN0cmF0aW9uOyByZWdpc3RlcmVkIGJlZm9yZSB0cmFuc3BvcnQ7IHRyYW5zcG9ydCBhbWJpZ3VvdXMKK29yIGNhbmNlbGxhdGlvbiBwZW5kaW5nOyBhbHJlYWR5IHJvdXRhYmxlIG9yIHRlcm1pbmFsOyBpbnZhbGlkIHRhcmdldDsgYW1iaWd1b3VzIHRhcmdldDsgYW5kCitwb2xpY3kgZGVuaWVkLiBSZW1haW5pbmcgQjMuMiBtdXN0IGZpcnN0IHByb3ZpZGUgdGhlIGR1cmFibGUsIHJlc3RhcnQtc2FmZSByZXNvbHV0aW9uL3JlY29uY2lsaWF0aW9uCitwcmltaXRpdmUgdGhvc2Ugb3V0Y29tZXMgY29uc3VtZS4KKworVGhlIGhvc3QtdG8td29ybGQgbGF1bmNoIGNhcnJpZXIgaXMgdHlwZWQgYW5kIHN1YnN0aXR1dGlvbi1yZXNpc3RhbnQ6CisKK2BgYHJ1c3QKK3N0cnVjdCBSZXRhaW5lZFdvcmtlckFkbWlzc2lvbkNvbW1pdG1lbnRDYXJyaWVyVjEgeworICAgIHNjaGVtYV92ZXJzaW9uOiB1MzIsIC8vIGV4YWN0bHkgMQorICAgIGFsZ29yaXRobTogU3RyaW5nLCAgIC8vIGV4YWN0bHkgImhtYWMtc2hhLTI1NiIKKyAgICBrZXlfaWQ6IFN0cmluZywKKyAgICBkaWdlc3RfaGV4OiBTdHJpbmcsCit9CisKK3N0cnVjdCBSZXRhaW5lZFdvcmtlckxhdW5jaEF1dGhvcml0eVByb29mVjEgeworICAgIHNjaGVtYV92ZXJzaW9uOiB1MzIsIC8vIGV4YWN0bHkgMQorICAgIGF1dGhvcml0eV9zdG9yZV9pZDogU3RyaW5nLAorICAgIGlzc3Vlcl9yZXF1ZXN0X2lkOiBTdHJpbmcsCisgICAgY2Fub25pY2FsX3NwYXduX2ZpbmdlcnByaW50OiBSZXRhaW5lZFdvcmtlckFkbWlzc2lvbkNvbW1pdG1lbnRDYXJyaWVyVjEsCisgICAgcmVnaXN0cmF0aW9uX2lkOiBTdHJpbmcsCisgICAgcmVnaXN0cmF0aW9uX2NvbW1pdG1lbnQ6IEF1dGhvcml0eU9iamVjdENvbW1pdG1lbnRWMSwKKyAgICBhdXRob3JpdHlfcmV2aXNpb25fYWZ0ZXI6IHU2NCwKKyAgICBhdXRob3JpdHlfcmVjb3JkX2NvbW1pdG1lbnRfYWZ0ZXI6IEF1dGhvcml0eU9iamVjdENvbW1pdG1lbnRWMSwKKyAgICBvcmNoZXN0cmF0aW9uX3Nlc3Npb25faWQ6IFN0cmluZywKKyAgICBjYWxsZXJfcGFydGljaXBhbnRfaWQ6IFN0cmluZywKKyAgICByZXRhaW5lZF9wYXJ0aWNpcGFudF9pZDogU3RyaW5nLAorICAgIGJvb3RzdHJhcF9ydW5faWQ6IFN0cmluZywKKyAgICB0cmFuc3BvcnRfY2xhaW1faWQ6IFN0cmluZywKKyAgICBiYWNrZW5kX2lkOiBTdHJpbmcsCisgICAgcHJvdG9jb2w6IFN0cmluZywKKyAgICB3b3JsZF9iaW5kaW5nOiBXb3JsZEJpbmRpbmdWMSwKKyAgICBjdXJyZW50X3BvbGljeV9yZWZfaWQ6IFN0cmluZywKKyAgICBjdXJyZW50X3BvbGljeV9yZXZpc2lvbjogU3RyaW5nLAorICAgIHJldGFpbmVkX3dvcmtlcl9yZWZfaWQ6IFN0cmluZywKKyAgICByZXRhaW5lZF93b3JrZXJfY29tbWl0bWVudDogQXV0aG9yaXR5T2JqZWN0Q29tbWl0bWVudFYxLAorfQorYGBgCisKK0JlZm9yZSBlaXRoZXIgdHJhbnNwb3J0IGJ1aWxkZXIgc2VyaWFsaXplcyBpdCwgdGhlIGhvc3QgZXhhY3Qtam9pbnMgZXZlcnkgZmllbGQgdG8gdGhlIGN1cnJlbnQgSFNBCityZWdpc3RyYXRpb24gcHJvb2YsIGFkbWlzc2lvbiByZWNvcmQvZmluZ2VycHJpbnQsIGJvdW5kIGNhbGxlciwgZGVzY3JpcHRvciwgcG9saWN5LCBhbmQgd29ybGQuIFRoZQorY2FycmllciBpcyB0aGUgZXhhY3QgZmllbGQtZm9yLWZpZWxkLCB0cmFuc3BvcnQtbmV1dHJhbCBlcXVhbGl0eSBwcm9qZWN0aW9uIG9mIHRoZSBpbnRlcm5hbAorUmV0YWluZWRXb3JrZXJBZG1pc3Npb25Db21taXRtZW50VjE7IGl0IGNvbnRhaW5zIG5vIHNlY3JldCBrZXkgYW5kIGdyYW50cyBubyBIU0Egb3IgYWRtaXNzaW9uCittdXRhdGlvbiBhdXRob3JpdHkuIFRoZSByZWNlaXZpbmcgd29ybGQgY29tcGFyZXMgdGhlIGNsb3NlZCBjYXJyaWVyIGFuZCBkaXNwYXRjaCBmaWVsZHMgb25seTsgaXQKK2RvZXMgbm90IHZlcmlmeSBvciByZWludGVycHJldCB0aGUgaG9zdC1vbmx5IEhNQUMgaW5wdXQuIFRoZQorb3B0aW9uYWwgY2FycmllciBmaWVsZCBvbiB0aGUgVjEgY29tcGF0aWJpbGl0eSByZXF1ZXN0IG1heSByZW1haW4gYWJzZW50IG9ubHkgb24gdGhlIGV4cGxpY2l0bHkKK3ByZS1hY3RpdmF0aW9uIGxlZ2FjeSBwYXRoOyBib3RoIEExLjJhLVMgYXV0aG9yaXR5LW1hbmFnZWQgU3Bhd24gcHJvZHVjZXJzIHJlcXVpcmUgaXQgd2l0aCBubworZmFsbGJhY2suIGB0cmFuc3BvcnQtYXBpLXR5cGVzYCB2YWxpZGF0ZXMgaXRzIGNsb3NlZCBzaGFwZS4gUHJvZHVjdGlvbiBtZW1iZXIgU3Bhd24gZW50ZXJzCitgU2VydmljZTo6ZXhlY3V0ZV9zdHJlYW1gLiBGb3IgYXV0aG9yaXR5LW1hbmFnZWQgYFNvbWUoZXhhY3QgcHJvb2YpYCwgdGhhdCBtZW1iZXIgYnJhbmNoIGNvbXBsZXRlcwordGhlIGV4aXN0aW5nIHN0cmljdCBwcm9vZi9kaXNwYXRjaCBlcXVhbGl0eSB2YWxpZGF0aW9uLCBkdXJhYmx5IGFkb3B0cyB0aGUgZXhhY3QgSFNBLWJvdW5kIHBoeXNpY2FsCit3b3JsZCB0aHJvdWdoIEIzLjJhLVdBLCBhbmQgb25seSB0aGVuIGNhbGxzIGBNZW1iZXJSdW50aW1lTWFuYWdlcjo6bGF1bmNoYCwgd2hpY2ggcmV0YWlucyBpdHMgb3duCit2YWxpZGF0aW9uIGJlZm9yZSBwcm9jZXNzIGNyZWF0aW9uLiBDb21wYXRpYmlsaXR5IGBOb25lYCByZXRhaW5zIHRoZSBleGlzdGluZyBkaXJlY3QgcGF0aC4KK2B3b3JsZC1hcGlgLCBgU2VydmljZTo6ZXhlY3V0ZWAsIGFuZCBgY29udmVydF9tZW1iZXJfZGlzcGF0Y2hfcmVxdWVzdGAgYXJlIG5vdCBwYXJ0IG9mIHRoaXMgcm91dGUuCitXb3JsZC1zZXJ2aWNlIGRvZXMgbm90IG1pbnQgb3IgYWR2YW5jZSBIU0Egb3IgYWRtaXNzaW9uIHRydXRoOyB0aGUgYm91bmQgaG9zdCB2ZXJpZmljYXRpb24gc3VwcGxpZXMKK2F1dGhvcml0eSBhdXRoZW50aWNpdHksIEIzLjJhLVdBIHN1cHBsaWVzIHBoeXNpY2FsIG93bmVyc2hpcCBvbmx5LCBhbmQgdGhlIGxhdW5jaCBib3VuZGFyeSBzdXBwbGllcworZXhhY3QgY2Fycmllci9yZXF1ZXN0IGVxdWFsaXR5LgorCitgI1tzZXJkZShkZWZhdWx0KV1gIG9uIHRoZSBvcHRpb25hbCBgTWVtYmVyRGlzcGF0Y2hSZXF1ZXN0VjFgIHByb29mIGZpZWxkIHByZXNlcnZlcyB3aXJlCitjb21wYXRpYmlsaXR5IGJ5IHN1cHBseWluZyBgRGVmYXVsdDo6ZGVmYXVsdCgpYCBvbmx5IHdoZW4gdGhhdCBmaWVsZCBpcyBhYnNlbnQgZHVyaW5nCitkZXNlcmlhbGl6YXRpb24uIEl0IGRvZXMgbm90IGluaXRpYWxpemUgUnVzdCBzdHJ1Y3QgbGl0ZXJhbHM6IGVhY2ggbGl0ZXJhbCBtdXN0IG5hbWUgdGhlIGZpZWxkIChvcgordXNlIGV4cGxpY2l0IFJ1c3Qgc3RydWN0IHVwZGF0ZSBzeW50YXgsIHdoaWNoIGlzIG5vdCBhdXRob3JpemVkIGZvciB0aGVzZSBmaXh0dXJlcykuIFRoZXJlZm9yZSB0aGUKK0IzLjJhIGFsbG93bGlzdCBhZGRpdGlvbmFsbHkgcGVybWl0cyBlZGl0cyBvbmx5IGluCitgY3JhdGVzL3dvcmxkLXNlcnZpY2UvdGVzdHMvbWVtYmVyX3J1bnRpbWVfd29ybGRfcGxhY2VtZW50X3YxLnJzYCwKK2BjcmF0ZXMvd29ybGQtc2VydmljZS90ZXN0cy9zdHJlYW1lZF9leGVjdXRlX2NhbmNlbF92MS5yc2AsIGFuZAorYGNyYXRlcy93b3JsZC1zZXJ2aWNlL3Rlc3RzL21lbWJlcl9ydW50aW1lX3JldGFpbmVkX2xpZmVjeWNsZV92MS5yc2AsIHNvbGVseSB0byBzZXQgdGhlIGZpZWxkIHRvCitgTm9uZWAgaW4gZXhpc3RpbmcgZXhwbGljaXRseSBwcmUtYWN0aXZhdGlvbiBvciBsZWdhY3kgbGl0ZXJhbHMgYW5kIHByb3ZlIHVuY2hhbmdlZCBjb21wYXRpYmlsaXR5LgorRXZlcnkgb3RoZXIgZml4dHVyZSBpbnB1dCwgdGVzdCBuYW1lLCBhc3NlcnRpb24sIGV4cGVjdGVkIG91dGNvbWUsIGFuZCBleHBlY3RlZCBlcnJvciByZW1haW5zCit1bmNoYW5nZWQuIEFueSBleGlzdGluZyB0ZXN0IHRoYXQgY2xhaW1zIHRoZSBhdXRob3JpdHktbWFuYWdlZCBCMy4yYSByb3V0ZSBtdXN0IGNhcnJ5IGFuIGV4YWN0Cit2YWxpZCBwcm9vZiB0aHJvdWdoIHRoZSBjYW5vbmljYWwgcHJvZHVjdGlvbiBwYXRoIGluc3RlYWQ7IGBOb25lYCBpcyBub3QgdmFsaWQgdGhlcmUuIFRoZXNlIHRlc3QKK2ZpbGVzIG1heSBub3QgaW50cm9kdWNlIGEgcHJvZHVjdGlvbiBwYXRoLCBoZWxwZXIgZGVmYXVsdCwgZml4dHVyZS1vbmx5IGF1dGhvcml0eSwgYWx0ZXJuYXRlCitwcm9vZiwgYWx0ZXJuYXRlIHRyYW5zcG9ydCwgc2lkZSB0YWJsZSwgb3Igd2Vha2VuZWQgYXNzZXJ0aW9uLiBBdXRob3JpdHktbWFuYWdlZCBTcGF3biBzdGlsbAorcmVxdWlyZXMgdGhlIGV4YWN0IHByb29mLiBUaGlzIG1lY2hhbmljYWwgYXV0aG9yaXphdGlvbiBkaWQgbm90IGl0c2VsZiBjb21wbGV0ZSBCMy4yYTsgdGhlCityZWNvcmRlZCBCMy4yYS9CMy4yYS1XQSByZXN1bHQgaW4gYDA1YCBub3cgZG9lcy4KKworVGhhdCBSdXN0LWxpdGVyYWwgcmVxdWlyZW1lbnQgYWxzbyBhdXRob3JpemVzIGV4YWN0bHkgdGhyZWUgcHJvZHVjdGlvbiBjb21wYXRpYmlsaXR5Citpbml0aWFsaXplcnMgYW5kIG5vdGhpbmcgZWxzZTogYGJ1aWxkX3J1bl93b3JsZF90YXNrX3RyYW5zcG9ydF9yZXF1ZXN0YCBhbmQKK2BidWlsZF9mb3JrX3dvcmxkX3dvcmtlcl90cmFuc3BvcnRfcmVxdWVzdGAgaW4KK2BjcmF0ZXMvc2hlbGwvc3JjL2V4ZWN1dGlvbi9vcmNoZXN0cmF0b3Jfd29ybGRfZGlzcGF0Y2gucnNgLCBwbHVzIGBjb252ZXJ0X21lbWJlcl9kaXNwYXRjaGAgaW4KK2BjcmF0ZXMvd29ybGQtbWFjLWxpbWEvc3JjL2xpYi5yc2AuIEVhY2ggbWF5IG9ubHkgbmFtZSB0aGUgbmV3IG9wdGlvbmFsIHByb29mIGZpZWxkIGFzIGV4cGxpY2l0CitgTm9uZWA7IGV2ZXJ5IHByZS1leGlzdGluZyBiYWNrZW5kLCBwcm90b2NvbCwgcnVuLCBzZXNzaW9uLCBwYXJ0aWNpcGFudCwgbGluZWFnZSwgd29ybGQsIHByb21wdCwKK3J1bnRpbWUsIHJvdXRlLXNlbGVjdGlvbiwgcG9saWN5LCBwbGFjZW1lbnQsIGxpZmVjeWNsZSwgZXJyb3IsIGFuZCBvdXRjb21lIGZpZWxkIHJlbWFpbnMKK2J5dGUtZm9yLWJ5dGUgYW5kIGJlaGF2aW9yYWxseSB1bmNoYW5nZWQuIGBOb25lYCBncmFudHMgbm8gcmV0YWluZWQtd29ya2VyIGxhdW5jaCBhdXRob3JpdHkgdG8KK1J1bldvcmxkVGFzaywgRm9ya1dvcmxkV29ya2VyLCBvciB0aGUgbWFjT1Mgd29ybGQtYXBpIGNvbnZlcnNpb24uIFRoaXMgaXMgbm90IG1hY09TCithdXRob3JpdHktbWFuYWdlZCBTcGF3biBhZG9wdGlvbiBhbmQgYXV0aG9yaXplcyBubyBlZGl0IHRvIGBjcmF0ZXMvd29ybGQtYXBpL3NyYy9saWIucnNgLAorYFNlcnZpY2U6OmV4ZWN1dGVgLCBvciBgY29udmVydF9tZW1iZXJfZGlzcGF0Y2hfcmVxdWVzdGAuIE5vIGRlZmF1bHQgY29uc3RydWN0b3IsIHNpZGUgdGFibGUsCitlbnZpcm9ubWVudCBjYXJyaWVyLCBhbHRlcm5hdGUgdHJhbnNwb3J0IHJvdXRlLCBvciBoaWRkZW4gcHJvb2Ygc3ludGhlc2lzIG1heSByZXBsYWNlIHRoZXNlCitleHBsaWNpdCBpbml0aWFsaXplcnMuIEJvdGggYXV0aG9yaXR5LW1hbmFnZWQgQjMuMmEgU3Bhd24gcHJvZHVjZXJzIHN0aWxsIHJlcXVpcmUKK2BTb21lKGV4YWN0IFJldGFpbmVkV29ya2VyTGF1bmNoQXV0aG9yaXR5UHJvb2ZWMSlgIGV4YWN0LWpvaW5lZCB0byBhZG1pc3Npb24gYW5kIFIwIHRydXRoIGJlZm9yZQorc2VyaWFsaXphdGlvbiwgYW5kIG1pc3NpbmcsIG1hbGZvcm1lZCwgb3IgbWlzbWF0Y2hlZCBwcm9vZiBmYWlscyBiZWZvcmUgcHJvY2VzcyBjcmVhdGlvbi4gVGhpcworbWVjaGFuaWNhbCBhdXRob3JpemF0aW9uIGRpZCBub3QgaXRzZWxmIGNvbXBsZXRlIEIzLjJhOyB0aGUgcmVjb3JkZWQgQjMuMmEvQjMuMmEtV0EgcmVzdWx0IGluIGAwNWAKK2lzIHJldmlldy1jbGVhbiB0aHJvdWdoIGBkMGE3MDcyN2MyYmVjMmIyZDZmZTA3NTRlYTQ2OWM0NjgyNjg0ZGRhYC4KKworVGhlIHNhbWUgY29tcGlsZXItcmVxdWlyZWQgd2lkZW5pbmcgYXV0aG9yaXplcyBvbmx5IHNldmVuIGFkZGl0aW9uYWwgcHJvZHVjdGlvbi1zeW1ib2wgZWRpdHMuIEluCitgY3JhdGVzL3NoZWxsL3NyYy9leGVjdXRpb24vb3JjaGVzdHJhdG9yX3dvcmxkX2Rpc3BhdGNoLnJzYCwgYGZvcmtfd29ybGRfd29ya2VyYCBhbmQKK2Bjb250aW51ZV93b3JsZF93b3JrZXJfZm9ya19jb21tYW5kX2Jvb3RzdHJhcF9hZnRlcl9kZWxpdmVyeWAgbWF5IG9ubHkgcGFzcyBleHBsaWNpdCBgTm9uZWAgZm9yIHRoZQorb3B0aW9uYWwgcmV0YWluZWQtd29ya2VyIGF1dGhvcml0eSBjb250ZXh0IHRvIHRoZSB3aWRlbmVkIHN0cmVhbSBoZWxwZXIuIEluCitgY3JhdGVzL3NoZWxsL3NyYy9yZXBsL2FzeW5jX3JlcGwucnNgLCBgYXBwbHlfZ3JlZW5maWVsZF9ob3N0X3N0YXJ0X2Zyb21fYXV0aG9yaXR5YCwKK2BwcmVwYXJlX2hpZGRlbl9vd25lcl9oZWxwZXJfcnVudGltZWAsCitgc3RhcnRfaG9zdF9vcmNoZXN0cmF0b3JfcnVudGltZV93aXRoX3ByZXBhcmVkX3Byb21wdF9hbmRfdG9vbGJveF9yZXF1ZXN0X3R4YCwKK2BwcmVwYXJlX2ZvcmtfY2hpbGRfcnVudGltZV9zdGFydHVwX2Zvcl9kZXNjcmlwdG9yYCwgYW5kCitgcHJlcGFyZV9tZW1iZXJfcnVudGltZV9zdGFydHVwX2Zvcl9kZXNjcmlwdG9yYCBtYXkgb25seSBpbml0aWFsaXplLCBkZXN0cnVjdHVyZSwgb3IgcHJlc2VydmUgdGhlCituZXcgb3B0aW9uYWwgcmV0YWluZWQtd29ya2VyIGxhdW5jaC1hdXRob3JpdHkgcHJvb2YvYWRtaXNzaW9uIGZpZWxkcyBhcyBgTm9uZWAuIFRob3NlIHZhbHVlcyBncmFudAorbm8gcmV0YWluZWQtd29ya2VyIGxhdW5jaCBhdXRob3JpdHk6IGZvcmsgYW5kIGZvcmsgY29udGludWF0aW9uIHJlbWFpbiBjb21wYXRpYmlsaXR5IHBhdGhzOyBIb3N0CitTdGFydCByZXRhaW5zIG9ubHkgaXRzIEExLjJhLVMgaG9zdC1zZXNzaW9uIGF1dGhvcml0eTsgaGlkZGVuLW93bmVyLWhlbHBlciBiZWhhdmlvciBpcyB1bmNoYW5nZWQ7CithbmQgYHByZXBhcmVfbWVtYmVyX3J1bnRpbWVfc3RhcnR1cF9mb3JfZGVzY3JpcHRvcmAgcmVtYWlucyBsZWdhY3kgcHJlLWFjdGl2YXRpb24gcHJlcGFyYXRpb24sCitkaXN0aW5jdCBmcm9tIHRoZSBCMy4yYS1vbmx5IGBwcmVwYXJlX21lbWJlcl9ydW50aW1lX3N0YXJ0dXBfZnJvbV9hdXRob3JpdHlfcmVnaXN0cmF0aW9uYCBwYXRoLgorT25seSB0aGF0IGF1dGhvcml0eS1yZWdpc3RyYXRpb24gcHJlcGFyZXIgbWF5IGNvbnN0cnVjdCB0aGUgcGFpcmVkCitgU29tZShleGFjdCBSZXRhaW5lZFdvcmtlckxhdW5jaEF1dGhvcml0eVByb29mVjEpYCBhbmQgZXhhY3QgYWRtaXNzaW9uIGNvbnRleHQuIEFuCithdXRob3JpdHktbWFuYWdlZCByZXRhaW5lZCBTcGF3biB3aXRoIGVpdGhlciB2YWx1ZSBtaXNzaW5nIGZhaWxzIGNsb3NlZCBhbmQgY2Fubm90IGZhbGwgYmFjayB0byB0aGUKK2xlZ2FjeSBwcmVwYXJlciBvciByZWludGVycHJldCBnZW5lcmljIGBOb25lYCBhcyBjb21wYXRpYmlsaXR5LiBUaGUgc2V2ZW4gZXhjZXB0aW9ucyBjaGFuZ2Ugbm8KK3BhY2tldCBvd25lcnNoaXAsIHBvbGljeSwgbGlmZWN5Y2xlLCB0cmFuc3BvcnQsIGlkZW50aXR5LCBsaW5lYWdlLCBlcnJvciwgb3V0Y29tZSwgaG9zdC1ydW50aW1lLAoraGVscGVyLCBsZWdhY3ktbWVtYmVyLCBvciBmb3JrIHNlbWFudGljcyBhbmQgYXV0aG9yaXplIG5vIG90aGVyIHN0cnVjdHVyYWwgY2hhbmdlLiBUaGlzIG1lY2hhbmljYWwKK2F1dGhvcml6YXRpb24gZGlkIG5vdCBpdHNlbGYgY29tcGxldGUgQjMuMmE7IHRoZSByZWNvcmRlZCBCMy4yYS9CMy4yYS1XQSByZXN1bHQgaW4gYDA1YCBpcworcmV2aWV3LWNsZWFuIHRocm91Z2ggYGQwYTcwNzI3YzJiZWMyYjJkNmZlMDc1NGVhNDY5YzQ2ODI2ODRkZGFgLgorCis8IS0tIGV4YWN0LWV4dHJhY3RlZC1ib2R5OmVuZCAtLT4KYGBgCg==
```

</details>

## Preserved review answer

<details>
<summary>Initial-range review answer (rendered copy)</summary>

<pre>
No qualifying P1/P2 findings.

I reviewed the complete four-file D10 patch against the stated material-consequence and blocking thresholds.&#32;

The extraction is internally consistent: the owner-marker body reconstructs to exactly 9,896 bytes, matches the stated SHA-256 d903299bdd345316fd0943d28abc300a1cef001ce0744746f72f53607eb64c12, and is byte-identical to baseline lines 71–213, including the final blank separator. The paired schemas, field ordering, literals, Some/None boundaries, allowlists, exclusions, failure semantics, authority limits, and recorded-result hash remain unchanged.&#32;

The root retains the legacy Deferred retained-spawn admission recovery contract heading and substitutes only the substantive body with a correctly targeted shallow pointer; the following B3.2a-WA heading remains untouched.&#32;

 The added index row and single D10 ledger row identify the same canonical owner, preserve the specified neighboring ownership boundaries, record the correct source locator/digest, and state an atomic rollback confined to this extraction.&#32;

I also independently reproduced the supplied complete-patch SHA-256 8dd20ed56bdd05256217317d9578ab49f3a90b7775f340addf6a2bd9e3277115 and the new-owner file SHA-256 06fd68735582d0ddee76a14602b99ead6a6df89affbd1036d4f36613ebd0d9c2. The supplied link/anchor and forward/reverse-apply validation is consistent with the patch inspection.&#32;

VERDICT: APPROVED
</pre>

</details>

<details>
<summary>Initial-range review answer exact bytes (base64 UTF-8)</summary>

```text
Tm8gcXVhbGlmeWluZyBQMS9QMiBmaW5kaW5ncy4KCkkgcmV2aWV3ZWQgdGhlIGNvbXBsZXRlIGZvdXItZmlsZSBEMTAgcGF0Y2ggYWdhaW5zdCB0aGUgc3RhdGVkIG1hdGVyaWFsLWNvbnNlcXVlbmNlIGFuZCBibG9ja2luZyB0aHJlc2hvbGRzLiAKClRoZSBleHRyYWN0aW9uIGlzIGludGVybmFsbHkgY29uc2lzdGVudDogdGhlIG93bmVyLW1hcmtlciBib2R5IHJlY29uc3RydWN0cyB0byBleGFjdGx5IDksODk2IGJ5dGVzLCBtYXRjaGVzIHRoZSBzdGF0ZWQgU0hBLTI1NiBkOTAzMjk5YmRkMzQ1MzE2ZmQwOTQzZDI4YWJjMzAwYTFjZWYwMDFjZTA3NDQ3NDZmNzJmNTM2MDdlYjY0YzEyLCBhbmQgaXMgYnl0ZS1pZGVudGljYWwgdG8gYmFzZWxpbmUgbGluZXMgNzHigJMyMTMsIGluY2x1ZGluZyB0aGUgZmluYWwgYmxhbmsgc2VwYXJhdG9yLiBUaGUgcGFpcmVkIHNjaGVtYXMsIGZpZWxkIG9yZGVyaW5nLCBsaXRlcmFscywgU29tZS9Ob25lIGJvdW5kYXJpZXMsIGFsbG93bGlzdHMsIGV4Y2x1c2lvbnMsIGZhaWx1cmUgc2VtYW50aWNzLCBhdXRob3JpdHkgbGltaXRzLCBhbmQgcmVjb3JkZWQtcmVzdWx0IGhhc2ggcmVtYWluIHVuY2hhbmdlZC4gCgpUaGUgcm9vdCByZXRhaW5zIHRoZSBsZWdhY3kgRGVmZXJyZWQgcmV0YWluZWQtc3Bhd24gYWRtaXNzaW9uIHJlY292ZXJ5IGNvbnRyYWN0IGhlYWRpbmcgYW5kIHN1YnN0aXR1dGVzIG9ubHkgdGhlIHN1YnN0YW50aXZlIGJvZHkgd2l0aCBhIGNvcnJlY3RseSB0YXJnZXRlZCBzaGFsbG93IHBvaW50ZXI7IHRoZSBmb2xsb3dpbmcgQjMuMmEtV0EgaGVhZGluZyByZW1haW5zIHVudG91Y2hlZC4gCgogVGhlIGFkZGVkIGluZGV4IHJvdyBhbmQgc2luZ2xlIEQxMCBsZWRnZXIgcm93IGlkZW50aWZ5IHRoZSBzYW1lIGNhbm9uaWNhbCBvd25lciwgcHJlc2VydmUgdGhlIHNwZWNpZmllZCBuZWlnaGJvcmluZyBvd25lcnNoaXAgYm91bmRhcmllcywgcmVjb3JkIHRoZSBjb3JyZWN0IHNvdXJjZSBsb2NhdG9yL2RpZ2VzdCwgYW5kIHN0YXRlIGFuIGF0b21pYyByb2xsYmFjayBjb25maW5lZCB0byB0aGlzIGV4dHJhY3Rpb24uIAoKSSBhbHNvIGluZGVwZW5kZW50bHkgcmVwcm9kdWNlZCB0aGUgc3VwcGxpZWQgY29tcGxldGUtcGF0Y2ggU0hBLTI1NiA4ZGQyMGVkNTZiZGQwNTI1NjIxNzMxN2Q5NTc4YWI0OWYzYTkwYjc3NzVmMzQwYWRkZjZhMmJkOWUzMjc3MTE1IGFuZCB0aGUgbmV3LW93bmVyIGZpbGUgU0hBLTI1NiAwNmZkNjg3MzU1ODJkMGRkZWU3NmExNDYwMmI5OWVhZDZhNmRmODlhZmZiZDEwMzZkNGYzNjYxM2ViZDBkOWMyLiBUaGUgc3VwcGxpZWQgbGluay9hbmNob3IgYW5kIGZvcndhcmQvcmV2ZXJzZS1hcHBseSB2YWxpZGF0aW9uIGlzIGNvbnNpc3RlbnQgd2l0aCB0aGUgcGF0Y2ggaW5zcGVjdGlvbi4gCgpWRVJESUNUOiBBUFBST1ZFRAo=
```

</details>
