# ChatGPT Pro advisory review: runtime-refactor D10 HostExecutionEpisodeV1

- Date: 2026-08-26
- Bound baseline commit/tree: `29bec951ce58b2b94e4eaaa6cd4ed8f233295269` / `bcbdce96347721392b52e9eee43067c9769f369d`
- Candidate implementation subagent (`gpt-5.4`, Extra High): `/root/d10_host_execution_episode_landing`
- Root/orchestrator review owner: `/root`
- Independent review chat: https://chatgpt.com/c/6a8f23e3-f2cc-83ea-a6b0-507f2b1ea16f
- Review mode: `initial-range`
- Review context: independent fresh conversation
- ChatGPT account surface: `Pro`
- Visible reasoning-effort control: `Extra High`
- Exact model label: not exposed by the current visible ChatGPT UI
- Candidate patch: `sha256:c2f6ecea86a0fb460bde837fedeed6398f79e7c2b0bb881d97a172deb00122b9` at `/private/tmp/d10-host-execution-episode-v1-candidate.patch`
- Review prompt: `sha256:9e4b33548fa5f0e8995f483ccf040dbefa52d20913f5e91bf027bc5b028dbc73` at `/private/tmp/d10-host-execution-episode-v1-chatgpt-pro-initial-review-prompt.txt`
- Review answer: `sha256:e2d7995f553c1bf99098e3fe48a81c222bd6ced4dcf5021354fff77f4bf021c9` at `/private/tmp/d10-host-execution-episode-v1-chatgpt-pro-initial-review-answer.txt`
- Imported validation log: `sha256:b73228a33293b5cf0dbea650e2a415bddc09a552cd1c4c006bc212ef3fbc7737` at `/private/tmp/d10-host-execution-episode-v1-validation.log`
- Exact embedded source body: `1645 bytes`, `sha256:28e86e2e7e29b458280875f921ca327a469621ac96bd7e592a0a176027aa0117`
- Review verdict: `APPROVED`
- Review finding summary: `No qualifying findings.`
- Remediation rounds: `0`
- Commit posture: candidate plus this closeout will be committed together atomically after local validation; not yet committed
- Push posture: `not pushed`

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Candidate scope and outcome

This independently reviewed substantive D10 unit extracts `HostExecutionEpisodeV1` into `llm-last-mile/runtime-refactor/contracts/host-execution-episode-v1.md`, replaces the root body in `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` with a shallow compatibility pointer, adds the single truthful `HostExecutionEpisodeV1` index row, and adds the matching D10 extraction-ledger row with the exact source-span and rollback contract. `HostExecutionEpisodeV1` is complete locally and approved after initial-range review with zero remediation. D10 remains explicitly in progress and incomplete; the next substantive D10 unit is `Deferred retained-spawn admission recovery contract`; later D10 units remain queued; D11 remains blocked until D10 is fully complete and separately authorized; D12 remains blocked by D11 and is separately unauthorized; no successor dispatch authority is granted.

## Candidate manifest and digests

Manifest for the independently reviewed candidate state:

- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`: `sha256:0e9c698bfd11c8db6f1ebdc3bc5eedffe922c7105353528ff0a7459e0ff3964e`
- `llm-last-mile/runtime-refactor/contracts/host-execution-episode-v1.md`: `sha256:e96cf5af7a34b589cdda7c29dcd364f03598c382cbd61e68e47d6bba19cff3cf`
- `llm-last-mile/runtime-refactor/index/README.md`: `sha256:b11cf30867c275d8943ae2409df1bfa70c6d452d1d03f41f52dea87855487cb2`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`: `sha256:6e3c9e4f0a63dcc5b4834e4159180d2e6cf5bc8b0b6b16eacbd781a8e16c747f`

Imported review artifacts:

- Candidate patch: `sha256:c2f6ecea86a0fb460bde837fedeed6398f79e7c2b0bb881d97a172deb00122b9`
- Review prompt: `sha256:9e4b33548fa5f0e8995f483ccf040dbefa52d20913f5e91bf027bc5b028dbc73`
- Review answer: `sha256:e2d7995f553c1bf99098e3fe48a81c222bd6ced4dcf5021354fff77f4bf021c9`
- Imported validation log: `sha256:b73228a33293b5cf0dbea650e2a415bddc09a552cd1c4c006bc212ef3fbc7737`
- Exact embedded source body: `1645 bytes`, `sha256:28e86e2e7e29b458280875f921ca327a469621ac96bd7e592a0a176027aa0117`

## Exact source-body proof

- Baseline `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` lines `51–100` are exactly `1645` bytes with SHA-256 `28e86e2e7e29b458280875f921ca327a469621ac96bd7e592a0a176027aa0117`.
- The UTF-8 bytes strictly between `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` in `llm-last-mile/runtime-refactor/contracts/host-execution-episode-v1.md` are also exactly `1645` bytes with SHA-256 `28e86e2e7e29b458280875f921ca327a469621ac96bd7e592a0a176027aa0117`.
- The baseline source span and the owner-body bytes are byte-identical, including the final blank separator before the end marker.

## Validation evidence

- The supplied review answer states `No qualifying findings.` and ends with `VERDICT: APPROVED`.
- The candidate patch changes exactly the four imported candidate paths and no others.
- Imported candidate digests, candidate-patch SHA-256, prompt SHA-256, answer SHA-256, validation-log SHA-256, and exact source-body SHA-256 were all reverified during this closeout.
- The rendered prompt and rendered answer below are whitespace-clean: every trailing literal space from the imported UTF-8 files is rendered as `&#32;` so this repository-local Markdown artifact stays clean under `git diff --check` while the base64 blocks preserve the exact bytes.
- The rendered prompt and rendered answer were both revalidated against the imported files, and the embedded base64 blocks decode exactly back to the imported UTF-8 bytes without silently correcting the captured `\HostExecutionEpisodeV1``` answer-rendering oddity.
- Final changed-path fence including untracked files is exactly these six paths:
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/contracts/host-execution-episode-v1.md`
- `llm-last-mile/runtime-refactor/index/README.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `docs/guidance/2026-08-21-runtime-refactor-control-plane-decomposition-execution-tracker.md`
- `docs/guidance/2026-08-26-runtime-refactor-d10-host-execution-episode-v1-chatgpt-pro-review.md`
- `git diff --check` passed for the final closeout state.
- `git diff --no-index --check /dev/null docs/guidance/2026-08-26-runtime-refactor-d10-host-execution-episode-v1-chatgpt-pro-review.md` reported no whitespace diagnostics.
- Complete combined candidate+closeout patch was generated at `/private/tmp/d10-host-execution-episode-v1-closeout.patch` with a SHA-256 sidecar, and forward/reverse apply against disposable clean baseline exports both passed.
- No commit, stage, or push is performed during this closeout.

## Final verdict

`HostExecutionEpisodeV1` is complete locally and **APPROVED** after initial-range review with zero remediation. D10 remains explicitly in progress and incomplete; the next substantive D10 unit is `Deferred retained-spawn admission recovery contract`; later D10 units remain queued; D11 remains blocked until D10 is fully complete and separately authorized; D12 remains blocked by D11 and is separately unauthorized; no successor dispatch authority is granted.

## Preserved review prompt

<details>
<summary>Initial-range review prompt (rendered copy)</summary>

````text
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: exact baseline commit `29bec951ce58b2b94e4eaaa6cd4ed8f233295269`, baseline tree `bcbdce96347721392b52e9eee43067c9769f369d`, plus the complete content-addressed uncommitted patch below. Complete patch SHA-256: `c2f6ecea86a0fb460bde837fedeed6398f79e7c2b0bb881d97a172deb00122b9`.
Review target: the complete bounded D10 `HostExecutionEpisodeV1` contract extraction landing, including the untracked new canonical owner. Review every changed line in all four files.

Supported inputs and behavior: repository GitHub-style Markdown links and heading fragments; one root compatibility heading/pointer for the historical anchor; one canonical Markdown owner; one navigation row; one extraction-ledger row with exact source provenance and atomic rollback. The canonical owner uses non-rendering HTML comments `<!-- exact-extracted-body:start -->` and `<!-- exact-extracted-body:end -->` only as byte-extraction boundaries. The 1645 bytes strictly between those markers are the exact canonical contract body and are content-addressed below. The end marker prevents a new blank line at EOF without altering any source-body byte.
Supported platforms and dialects: repository Markdown only under the documented GitHub-style heading-fragment validator. Rust/text fences are preserved literals, not compiled runtime implementation. No runtime, Rust compiler, scripts, platform behavior, external-site renderer, or D11/D12 behavior is in scope.

In-scope invariants:
1. There is exactly one canonical owner for `HostExecutionEpisodeV1`: `llm-last-mile/runtime-refactor/contracts/host-execution-episode-v1.md`.
2. Root `04-contracts-and-gates.md` retains the exact legacy heading/anchor `## 2. HostExecutionEpisodeV1` (backticks in the file) and replaces only its former substantive body with a shallow pointer to the canonical owner.
3. The exact live source span was baseline lines 51–100 inclusive, 1645 bytes, SHA-256 `28e86e2e7e29b458280875f921ca327a469621ac96bd7e592a0a176027aa0117`. The bytes between owner boundary markers must match that source artifact exactly, including the final blank separator.
4. Stable `V1` names, struct field order/spelling/types/punctuation, Rust fence and `// exactly 1`, episode-kind literal order, transport-status literal order, rules 1–6, line wrapping, negative requirements, and authority/successor boundaries remain exact.
5. The index adds exactly one truthful contract row and does not change or reorder unrelated rows.
6. The ledger adds exactly one D10 row with live source locator `lines 51–100`, exact source hash, correct root/destination anchors, representation/delta truth, and complete atomic rollback instructions.
7. Rollback restores the exact 1645-byte source span at the root heading, removes the new owner and exact index/ledger rows, and leaves the committed pre-span compatibility anchors, D3/D5–D9 owners, `index/current.md`, `review-control/`, every other D10 unit, and every path outside the four-path fence unchanged.
8. All runtime-refactor repository-relative Markdown links and anchors resolve after the patch.
9. The patch does not convert documentation to implementation or evidence to gate satisfaction, change authority/status, approve later D10 units, dispatch successors, begin D11, or perform D12 root cutover.
10. Exact changed-path fence including untracked files is the following manifest and nothing else:
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md` SHA-256 `0e9c698bfd11c8db6f1ebdc3bc5eedffe922c7105353528ff0a7459e0ff3964e`
- `llm-last-mile/runtime-refactor/contracts/host-execution-episode-v1.md` SHA-256 `e96cf5af7a34b589cdda7c29dcd364f03598c382cbd61e68e47d6bba19cff3cf`
- `llm-last-mile/runtime-refactor/index/README.md` SHA-256 `b11cf30867c275d8943ae2409df1bfa70c6d452d1d03f41f52dea87855487cb2`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md` SHA-256 `6e3c9e4f0a63dcc5b4834e4159180d2e6cf5bc8b0b6b16eacbd781a8e16c747f`

Required material consequence: a blocking finding must show a reachable broken supported Markdown link/anchor; loss/reordering/change of a named version, ordered field, literal, fence, numbered rule, negative requirement, exception, status, authority or successor boundary; a second/missing canonical owner; false provenance/hash/rollback/navigation; or an in-scope path/scope violation caused by this patch.
Blocking threshold: only reachable, material P1 or P2 defects introduced by this patch and causally tied to an in-scope invariant. Unchanged pre-existing behavior and stylistic/editorial preferences are non-blocking.
Accepted prior findings: none.
Deferred or out-of-scope concerns: the neighboring transition-intent, runtime-event, retained-worker, packet-local, and authority-required material already owned by D3/D5/D6; every other substantive D10 contract/gate unit; D11 evidence decomposition; D12 root cutover; runtime/Rust/scripts/platform behavior; roadmap or reviewed ZIP; review governance; D9 slices/tasks; broad Markdown normalization; unrelated pre-existing issues.
Project sources: the full review input is this prompt's exact baseline identity, content-addressed four-file manifest, complete patch, source-body digest, and validation evidence. Excluded source is unavailable and must not be inferred. Read the complete patch before judging the resulting manifest.

Validation evidence:
- exact four-path inventory including untracked: pass; no staged files.
- `git diff --check`: pass.
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/contracts/host-execution-episode-v1.md`: no whitespace diagnostics (expected nonzero diff status only).
- byte extraction between owner boundary markers: exactly 1645 bytes and SHA-256 `28e86e2e7e29b458280875f921ca327a469621ac96bd7e592a0a176027aa0117`, byte-equal to the live source artifact.
- ordered-field, fenced-code, literal-order, rule 1–6, negative-requirement, authority-boundary, and stable-version checks: pass by exact byte equality plus explicit anchor checks.
- runtime-refactor Markdown validator: `LINKS_CHECKED 1812 FAILURES 0`.
- root and destination anchors each exist exactly once; index and ledger links resolve.
- explicit prior-owner and review-artifact stability checks: pass for `review-control/`, D5/D6 packet-family directories, `architecture/`, `seams/`, `slices/`, and `docs/guidance/`.
- complete patch forward-apply from a clean disposable export: pass.
- complete patch reverse-apply from the modified disposable export: pass.
- validation log SHA-256: `b73228a33293b5cf0dbea650e2a415bddc09a552cd1c4c006bc212ef3fbc7737`.
- complete patch SHA-256: `c2f6ecea86a0fb460bde837fedeed6398f79e7c2b0bb881d97a172deb00122b9`.

Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return findings first by severity. For every finding include file/heading, supported-input reachability, violated invariant, material consequence, and patch causality. State explicitly when there are no qualifying findings. End with exactly one verdict line: `VERDICT: APPROVED` or `VERDICT: CHANGES REQUIRED`.

Complete patch (UTF-8, includes the untracked new owner; no omitted files):
```diff
diff --git a/llm-last-mile/runtime-refactor/04-contracts-and-gates.md b/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
index 40016726c..a9cdf586b 100644
--- a/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
+++ b/llm-last-mile/runtime-refactor/04-contracts-and-gates.md
@@ -50,53 +50,7 @@ Compatibility anchor only; canonical content: [`a1-2-earlier-histories/contracts
&#32;
 ## 2. `HostExecutionEpisodeV1`
&#32;
-```rust
-struct HostExecutionEpisodeV1 {
-    schema_version: u32,                 // exactly 1
-    episode_id: String,
-    kind: HostExecutionEpisodeKindV1,
-    orchestration_session_id: String,
-    observed_authority_revision: u64,
-    backend_id: Option<String>,
-    process_ref: Option<ProcessRefV1>,
-    transport_status: HostExecutionEpisodeTransportStatusV1,
-    started_at: Timestamp,
-    last_heartbeat_at: Option<Timestamp>,
-    ended_at: Option<Timestamp>,
-    exit_observation: Option<EpisodeExitObservationV1>,
-}
-```
-
-Episode kinds:
-
-```text
-ReplAttachedEpisode
-HiddenOwnerHelperStartEpisode
-HiddenOwnerHelperAttachEpisode
-HiddenOwnerHelperResumeOneTurnEpisode
-RuntimeToolboxEpisode
-SyntheticOrRecoveredEpisode
-```
-
-Transport status:
-
-```text
-Available
-UnavailableButDurableAuthorityExists
-UnavailableAndNoAuthoritativeRoute
-StaleOrOrphaned
-```
-
-Rules:
-
-1. Episodes submit observations and requested transitions to `HostSessionAuthority`; they never write durable posture directly.
-2. An observation whose `observed_authority_revision` is stale cannot mutate authority.
-3. Episode exit does not delete session, worker, binding, receipt, or obligation truth.
-4. Private transport success may accelerate delivery; it does not define durable success.
-5. PID, process/helper presence, active handles, readiness, and prompt-stream state are episode or
-   transport observations only; their absence does not erase `ParkedResumable` authority.
-6. Episode construction and launch follow durable transition application and cannot reset a parked
-   session to `Allocating` or authorize a successor participant.
+Canonical content: [`contracts/host-execution-episode-v1.md#2-hostexecutionepisodev1`](contracts/host-execution-episode-v1.md#2-hostexecutionepisodev1).
&#32;
 ## 2A. Runtime event identity and ordering carrier
&#32;
diff --git a/llm-last-mile/runtime-refactor/index/README.md b/llm-last-mile/runtime-refactor/index/README.md
index ac9a7edd9..a00b82170 100644
--- a/llm-last-mile/runtime-refactor/index/README.md
+++ b/llm-last-mile/runtime-refactor/index/README.md
@@ -17,6 +17,7 @@
 | `macOS-dev-parity-family` | lane index | [`macos-dev-parity/README.md`](../macos-dev-parity/README.md) | `non-authoritative navigation for the macOS developer-parity decision, extracted projections, and lane-local gate` | Assembles the current decision, extracted `00`/`02`/`03`/`05` projections, and the lane-local gate without authorizing Phase 2, Linux-first successor work, protected-lifecycle revival, or Windows dispatch. | [`00`](../00-README.md#macos-developer-parity-lane-2026-08-19-scoped), [`02`](../02-seam-crosswalk.md#current-cross-lane-scheduling-and-ownership-2026-08-20-controlling), [`03`](../03-phase-slice-map.md#current-linux-first-scheduling-reset-2026-08-20-controlling), [`05`](../05-debug-regression-ledger.md#current-cross-lane-regression-ledger-2026-08-20-controlling) |
 | `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` | gate | [`gates/authority-required-macos-dev-parity.md`](../gates/authority-required-macos-dev-parity.md) | lane-local authority-required gate; Phase 1 grants no Phase 2 authority | Does not block or authorize the Linux-first runtime-refactor sequence. | [`04`](../04-contracts-and-gates.md#authority_requiredmacos_dev_parity-contract-2026-08-19-macos-lane) |
 | `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` | gate | [`gates/authority-required-runtime-refactor-reentry.md`](../gates/authority-required-runtime-refactor-reentry.md) | closed documentation/control-plane selection gate | Closed as selection history; A1.3-P1 is the separate active packet. | [`04`](../04-contracts-and-gates.md#authority_requiredruntime_refactor_reentry-contract-2026-08-20-closed-selection-record) |
+| `HostExecutionEpisodeV1` | contract | [`contracts/host-execution-episode-v1.md`](../contracts/host-execution-episode-v1.md) | `canonical extracted contract owner for the complete schema, episode-kind literals, transport-status literals, and rules 1–6` | Supersedes only root-canonical ownership of the extracted `HostExecutionEpisodeV1` span; the committed pre-span compatibility anchors and neighboring `2A`/`2B` owners remain unchanged. | [`04`](../04-contracts-and-gates.md#2-hostexecutionepisodev1) |
 | `shared-target-architecture` | architecture index | [`architecture/README.md`](../architecture/README.md) | `non-authoritative navigation for the extracted executive target decision, authority map, numbered invariants, and stable review question` | Supersedes only root-canonical ownership of the extracted shared architecture spans; packet-family-local D5/D6 forwarders remain at their existing owners. | [`01`](../01-target-architecture.md#executive-decision), [`01`](../01-target-architecture.md#authority-map), [`01`](../01-target-architecture.md#non-negotiable-invariants), [`01`](../01-target-architecture.md#review-question) |
 | `shared-seam-crosswalk` | seam index | [`seams/README.md`](../seams/README.md) | `canonical shared seam-crosswalk rules plus extracted seam-family navigation for host/session, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation` | Supersedes only the extracted root reading-rule/classification spans and the extracted host/session authority, persistence/compatibility, dispatch/episode transport, policy/narrowing, runtime-event/receipt/supervision/retained-runtime, obligations/host re-engagement, configuration/gateway adoption, and UAA/provider realization/side-effect mediation family rows; A0 and the existing D6 `HostSessionAuthority`, `WorldWorkerMessagingProtocol`, and `ObligationLedger` compatibility rows remain at their current owners. | [`02`](../02-seam-crosswalk.md#reading-rule), [`02`](../02-seam-crosswalk.md#a-host-authority-and-ingress), [`02`](../02-seam-crosswalk.md#b-dispatch-policy-receipts-and-retained-runtime), [`02`](../02-seam-crosswalk.md#c-obligations-and-host-re-engagement), [`02`](../02-seam-crosswalk.md#d-uaa-realization-projection-and-side-effect-mediation), [`02`](../02-seam-crosswalk.md#classification-consequences) |
 | `shared-slice-map` | slice index | [`slices/README.md`](../slices/README.md) | `canonical shared sequencing/dependency and slice-closeout owner plus non-authoritative D9 Track A–E navigation and A1/A1.4 projection index` | Supersedes only root-canonical ownership of the extracted shared sequencing/closeout spans plus the extracted Track A–E and A1/A1.4 projection spans; controlling schedule authority remains with decisions, packets, gates, and `current.md`. | [`03`](../03-phase-slice-map.md#sequencing-rules), [`03`](../03-phase-slice-map.md#track-a--authority-and-surface-neutrality), [`03`](../03-phase-slice-map.md#a1-bounded-packet-decomposition), [`03`](../03-phase-slice-map.md#track-b--world-dispatch-receipts-supervision-and-cancel), [`03`](../03-phase-slice-map.md#track-c--obligations-inbox-auto-attach-and-router-attach), [`03`](../03-phase-slice-map.md#track-d--uaa-execution-envelope-and-side-effect-mediation), [`03`](../03-phase-slice-map.md#track-e--dispatch-scoped-policy-narrowing-and-config-projection), [`03`](../03-phase-slice-map.md#slice-closeout-minimum) |
diff --git a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
index f9aff61bc..c46a9f38d 100644
--- a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
+++ b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
@@ -184,3 +184,4 @@ This ledger records content-preserving authority transfers while retaining requi
 | D9 | [`03-phase-slice-map.md`](../03-phase-slice-map.md) | E3 — AgentConfigProjectionService and gateway adoption | [`track-e--dispatch-scoped-policy-narrowing-and-config-projection`](../03-phase-slice-map.md#track-e--dispatch-scoped-policy-narrowing-and-config-projection) | line 222 | `93b0ff582a573e29cea9b2f30f49bfc0278c383f1e564301afdf94579e5672e5` | [`../slices/e3-agent-config-projection-and-gateway-adoption.md`](../slices/e3-agent-config-projection-and-gateway-adoption.md) | slice row | canonical destination; source compatibility table row | none | restore the exact extracted D9 source bodies/rows in `03-phase-slice-map.md`; remove `slices/` and all D9-added files beneath it; remove the `shared-slice-map` index row from `index/README.md`; remove all matching D9 ledger entries; leave D5–D8 canonical owners, `index/current.md`, and `review-control/` unchanged |
 | D9 | [`03-phase-slice-map.md`](../03-phase-slice-map.md) | E4 — Host-visible write/sync contract | [`track-e--dispatch-scoped-policy-narrowing-and-config-projection`](../03-phase-slice-map.md#track-e--dispatch-scoped-policy-narrowing-and-config-projection) | line 223 | `c98a5df3f09735abc5cbefec285c7b5dc55fca5c67cfdf5bf6bd3f458d017893` | [`../slices/e4-host-visible-write-sync-contract.md`](../slices/e4-host-visible-write-sync-contract.md) | slice row | canonical destination; source compatibility table row | none | restore the exact extracted D9 source bodies/rows in `03-phase-slice-map.md`; remove `slices/` and all D9-added files beneath it; remove the `shared-slice-map` index row from `index/README.md`; remove all matching D9 ledger entries; leave D5–D8 canonical owners, `index/current.md`, and `review-control/` unchanged |
 | D9 | [`03-phase-slice-map.md`](../03-phase-slice-map.md) | A1.4 — bounded auto-attach producer adoption and regression closure | [`a1-bounded-packet-decomposition`](../03-phase-slice-map.md#a1-bounded-packet-decomposition) | line 145 | `58d7982b23a7b2f5f41f3e0b5170996e0f3b331dfbbcbb5875d02e801bd5d408` | [`../slices/tasks/a1-4-auto-attach-producer-adoption.md`](../slices/tasks/a1-4-auto-attach-producer-adoption.md) | task row | canonical destination; source compatibility table row | none | restore the exact extracted D9 source bodies/rows in `03-phase-slice-map.md`; remove `slices/` and all D9-added files beneath it; remove the `shared-slice-map` index row from `index/README.md`; remove all matching D9 ledger entries; leave D5–D8 canonical owners, `index/current.md`, and `review-control/` unchanged |
+| D10 | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | 2. `HostExecutionEpisodeV1` | [`2-hostexecutionepisodev1`](../04-contracts-and-gates.md#2-hostexecutionepisodev1) | lines 51–100 | `28e86e2e7e29b458280875f921ca327a469621ac96bd7e592a0a176027aa0117` | [`../contracts/host-execution-episode-v1.md`](../contracts/host-execution-episode-v1.md) | contract | canonical destination; source compatibility anchor | none | restore the exact 1645-byte source span at `04-contracts-and-gates.md#2-hostexecutionepisodev1`; remove `contracts/host-execution-episode-v1.md`; remove the exact `HostExecutionEpisodeV1` row from `index/README.md`; remove this D10 ledger entry; leave the committed pre-span compatibility anchors, D3/D5–D9 owners, `index/current.md`, `review-control/`, all other D10 units, and every path outside the four-path fence unchanged |
diff --git a/llm-last-mile/runtime-refactor/contracts/host-execution-episode-v1.md b/llm-last-mile/runtime-refactor/contracts/host-execution-episode-v1.md
new file mode 100644
index 000000000..5b784027a
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/contracts/host-execution-episode-v1.md
@@ -0,0 +1,58 @@
+**Kind:** contract
+**Status:** canonical
+**Canonical for:** complete extracted `HostExecutionEpisodeV1` schema, episode-kind literals, transport-status literals, and rules 1–6
+**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#2-hostexecutionepisodev1`](../04-contracts-and-gates.md#2-hostexecutionepisodev1), baseline lines 51–100; the exact 1645-byte source body is preserved between the boundary markers below
+**Baseline span SHA-256:** `28e86e2e7e29b458280875f921ca327a469621ac96bd7e592a0a176027aa0117`
+
+<!-- exact-extracted-body:start -->
+## 2. `HostExecutionEpisodeV1`
+
+```rust
+struct HostExecutionEpisodeV1 {
+    schema_version: u32,                 // exactly 1
+    episode_id: String,
+    kind: HostExecutionEpisodeKindV1,
+    orchestration_session_id: String,
+    observed_authority_revision: u64,
+    backend_id: Option<String>,
+    process_ref: Option<ProcessRefV1>,
+    transport_status: HostExecutionEpisodeTransportStatusV1,
+    started_at: Timestamp,
+    last_heartbeat_at: Option<Timestamp>,
+    ended_at: Option<Timestamp>,
+    exit_observation: Option<EpisodeExitObservationV1>,
+}
+```
+
+Episode kinds:
+
+```text
+ReplAttachedEpisode
+HiddenOwnerHelperStartEpisode
+HiddenOwnerHelperAttachEpisode
+HiddenOwnerHelperResumeOneTurnEpisode
+RuntimeToolboxEpisode
+SyntheticOrRecoveredEpisode
+```
+
+Transport status:
+
+```text
+Available
+UnavailableButDurableAuthorityExists
+UnavailableAndNoAuthoritativeRoute
+StaleOrOrphaned
+```
+
+Rules:
+
+1. Episodes submit observations and requested transitions to `HostSessionAuthority`; they never write durable posture directly.
+2. An observation whose `observed_authority_revision` is stale cannot mutate authority.
+3. Episode exit does not delete session, worker, binding, receipt, or obligation truth.
+4. Private transport success may accelerate delivery; it does not define durable success.
+5. PID, process/helper presence, active handles, readiness, and prompt-stream state are episode or
+   transport observations only; their absence does not erase `ParkedResumable` authority.
+6. Episode construction and launch follow durable transition application and cannot reset a parked
+   session to `Allocating` or authorize a successor participant.
+
+<!-- exact-extracted-body:end -->
```
````

</details>

<details>
<summary>Initial-range review prompt exact bytes (base64 UTF-8)</summary>

```text
UmV2aWV3IG1vZGU6IGluaXRpYWwtcmFuZ2UKUmV2aWV3IGNvbnRleHQ6IGluZGVwZW5kZW50IGZyZXNoIGNvbnZlcnNhdGlvbgpSZXZpZXcgYm91bmRhcnk6IGV4YWN0IGJhc2VsaW5lIGNvbW1pdCBgMjliZWM5NTFjZTU4YjJiOTRlNGVhYWE2Y2Q0ZWQ4ZjIzMzI5NTI2OWAsIGJhc2VsaW5lIHRyZWUgYGJjYmRjZTk2MzQ3NzIxMzkyYjUyZTllZWU0MzA2N2M5NzY5ZjM2OWRgLCBwbHVzIHRoZSBjb21wbGV0ZSBjb250ZW50LWFkZHJlc3NlZCB1bmNvbW1pdHRlZCBwYXRjaCBiZWxvdy4gQ29tcGxldGUgcGF0Y2ggU0hBLTI1NjogYGMyZjZlY2VhODZhMGZiNDYwYmRlODM3ZmVkZWVkNjM5OGY3OWU3YzJiMGJiODgxZDk3YTE3MmRlYjAwMTIyYjlgLgpSZXZpZXcgdGFyZ2V0OiB0aGUgY29tcGxldGUgYm91bmRlZCBEMTAgYEhvc3RFeGVjdXRpb25FcGlzb2RlVjFgIGNvbnRyYWN0IGV4dHJhY3Rpb24gbGFuZGluZywgaW5jbHVkaW5nIHRoZSB1bnRyYWNrZWQgbmV3IGNhbm9uaWNhbCBvd25lci4gUmV2aWV3IGV2ZXJ5IGNoYW5nZWQgbGluZSBpbiBhbGwgZm91ciBmaWxlcy4KClN1cHBvcnRlZCBpbnB1dHMgYW5kIGJlaGF2aW9yOiByZXBvc2l0b3J5IEdpdEh1Yi1zdHlsZSBNYXJrZG93biBsaW5rcyBhbmQgaGVhZGluZyBmcmFnbWVudHM7IG9uZSByb290IGNvbXBhdGliaWxpdHkgaGVhZGluZy9wb2ludGVyIGZvciB0aGUgaGlzdG9yaWNhbCBhbmNob3I7IG9uZSBjYW5vbmljYWwgTWFya2Rvd24gb3duZXI7IG9uZSBuYXZpZ2F0aW9uIHJvdzsgb25lIGV4dHJhY3Rpb24tbGVkZ2VyIHJvdyB3aXRoIGV4YWN0IHNvdXJjZSBwcm92ZW5hbmNlIGFuZCBhdG9taWMgcm9sbGJhY2suIFRoZSBjYW5vbmljYWwgb3duZXIgdXNlcyBub24tcmVuZGVyaW5nIEhUTUwgY29tbWVudHMgYDwhLS0gZXhhY3QtZXh0cmFjdGVkLWJvZHk6c3RhcnQgLS0+YCBhbmQgYDwhLS0gZXhhY3QtZXh0cmFjdGVkLWJvZHk6ZW5kIC0tPmAgb25seSBhcyBieXRlLWV4dHJhY3Rpb24gYm91bmRhcmllcy4gVGhlIDE2NDUgYnl0ZXMgc3RyaWN0bHkgYmV0d2VlbiB0aG9zZSBtYXJrZXJzIGFyZSB0aGUgZXhhY3QgY2Fub25pY2FsIGNvbnRyYWN0IGJvZHkgYW5kIGFyZSBjb250ZW50LWFkZHJlc3NlZCBiZWxvdy4gVGhlIGVuZCBtYXJrZXIgcHJldmVudHMgYSBuZXcgYmxhbmsgbGluZSBhdCBFT0Ygd2l0aG91dCBhbHRlcmluZyBhbnkgc291cmNlLWJvZHkgYnl0ZS4KU3VwcG9ydGVkIHBsYXRmb3JtcyBhbmQgZGlhbGVjdHM6IHJlcG9zaXRvcnkgTWFya2Rvd24gb25seSB1bmRlciB0aGUgZG9jdW1lbnRlZCBHaXRIdWItc3R5bGUgaGVhZGluZy1mcmFnbWVudCB2YWxpZGF0b3IuIFJ1c3QvdGV4dCBmZW5jZXMgYXJlIHByZXNlcnZlZCBsaXRlcmFscywgbm90IGNvbXBpbGVkIHJ1bnRpbWUgaW1wbGVtZW50YXRpb24uIE5vIHJ1bnRpbWUsIFJ1c3QgY29tcGlsZXIsIHNjcmlwdHMsIHBsYXRmb3JtIGJlaGF2aW9yLCBleHRlcm5hbC1zaXRlIHJlbmRlcmVyLCBvciBEMTEvRDEyIGJlaGF2aW9yIGlzIGluIHNjb3BlLgoKSW4tc2NvcGUgaW52YXJpYW50czoKMS4gVGhlcmUgaXMgZXhhY3RseSBvbmUgY2Fub25pY2FsIG93bmVyIGZvciBgSG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWA6IGBsbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvY29udHJhY3RzL2hvc3QtZXhlY3V0aW9uLWVwaXNvZGUtdjEubWRgLgoyLiBSb290IGAwNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kYCByZXRhaW5zIHRoZSBleGFjdCBsZWdhY3kgaGVhZGluZy9hbmNob3IgYCMjIDIuIEhvc3RFeGVjdXRpb25FcGlzb2RlVjFgIChiYWNrdGlja3MgaW4gdGhlIGZpbGUpIGFuZCByZXBsYWNlcyBvbmx5IGl0cyBmb3JtZXIgc3Vic3RhbnRpdmUgYm9keSB3aXRoIGEgc2hhbGxvdyBwb2ludGVyIHRvIHRoZSBjYW5vbmljYWwgb3duZXIuCjMuIFRoZSBleGFjdCBsaXZlIHNvdXJjZSBzcGFuIHdhcyBiYXNlbGluZSBsaW5lcyA1MeKAkzEwMCBpbmNsdXNpdmUsIDE2NDUgYnl0ZXMsIFNIQS0yNTYgYDI4ZTg2ZTJlN2UyOWI0NTgyODA4NzVmOTIxY2EzMjdhNDY5NjIxYWM5NmJkN2U1OTJhMGExNzYwMjdhYTAxMTdgLiBUaGUgYnl0ZXMgYmV0d2VlbiBvd25lciBib3VuZGFyeSBtYXJrZXJzIG11c3QgbWF0Y2ggdGhhdCBzb3VyY2UgYXJ0aWZhY3QgZXhhY3RseSwgaW5jbHVkaW5nIHRoZSBmaW5hbCBibGFuayBzZXBhcmF0b3IuCjQuIFN0YWJsZSBgVjFgIG5hbWVzLCBzdHJ1Y3QgZmllbGQgb3JkZXIvc3BlbGxpbmcvdHlwZXMvcHVuY3R1YXRpb24sIFJ1c3QgZmVuY2UgYW5kIGAvLyBleGFjdGx5IDFgLCBlcGlzb2RlLWtpbmQgbGl0ZXJhbCBvcmRlciwgdHJhbnNwb3J0LXN0YXR1cyBsaXRlcmFsIG9yZGVyLCBydWxlcyAx4oCTNiwgbGluZSB3cmFwcGluZywgbmVnYXRpdmUgcmVxdWlyZW1lbnRzLCBhbmQgYXV0aG9yaXR5L3N1Y2Nlc3NvciBib3VuZGFyaWVzIHJlbWFpbiBleGFjdC4KNS4gVGhlIGluZGV4IGFkZHMgZXhhY3RseSBvbmUgdHJ1dGhmdWwgY29udHJhY3Qgcm93IGFuZCBkb2VzIG5vdCBjaGFuZ2Ugb3IgcmVvcmRlciB1bnJlbGF0ZWQgcm93cy4KNi4gVGhlIGxlZGdlciBhZGRzIGV4YWN0bHkgb25lIEQxMCByb3cgd2l0aCBsaXZlIHNvdXJjZSBsb2NhdG9yIGBsaW5lcyA1MeKAkzEwMGAsIGV4YWN0IHNvdXJjZSBoYXNoLCBjb3JyZWN0IHJvb3QvZGVzdGluYXRpb24gYW5jaG9ycywgcmVwcmVzZW50YXRpb24vZGVsdGEgdHJ1dGgsIGFuZCBjb21wbGV0ZSBhdG9taWMgcm9sbGJhY2sgaW5zdHJ1Y3Rpb25zLgo3LiBSb2xsYmFjayByZXN0b3JlcyB0aGUgZXhhY3QgMTY0NS1ieXRlIHNvdXJjZSBzcGFuIGF0IHRoZSByb290IGhlYWRpbmcsIHJlbW92ZXMgdGhlIG5ldyBvd25lciBhbmQgZXhhY3QgaW5kZXgvbGVkZ2VyIHJvd3MsIGFuZCBsZWF2ZXMgdGhlIGNvbW1pdHRlZCBwcmUtc3BhbiBjb21wYXRpYmlsaXR5IGFuY2hvcnMsIEQzL0Q14oCTRDkgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGByZXZpZXctY29udHJvbC9gLCBldmVyeSBvdGhlciBEMTAgdW5pdCwgYW5kIGV2ZXJ5IHBhdGggb3V0c2lkZSB0aGUgZm91ci1wYXRoIGZlbmNlIHVuY2hhbmdlZC4KOC4gQWxsIHJ1bnRpbWUtcmVmYWN0b3IgcmVwb3NpdG9yeS1yZWxhdGl2ZSBNYXJrZG93biBsaW5rcyBhbmQgYW5jaG9ycyByZXNvbHZlIGFmdGVyIHRoZSBwYXRjaC4KOS4gVGhlIHBhdGNoIGRvZXMgbm90IGNvbnZlcnQgZG9jdW1lbnRhdGlvbiB0byBpbXBsZW1lbnRhdGlvbiBvciBldmlkZW5jZSB0byBnYXRlIHNhdGlzZmFjdGlvbiwgY2hhbmdlIGF1dGhvcml0eS9zdGF0dXMsIGFwcHJvdmUgbGF0ZXIgRDEwIHVuaXRzLCBkaXNwYXRjaCBzdWNjZXNzb3JzLCBiZWdpbiBEMTEsIG9yIHBlcmZvcm0gRDEyIHJvb3QgY3V0b3Zlci4KMTAuIEV4YWN0IGNoYW5nZWQtcGF0aCBmZW5jZSBpbmNsdWRpbmcgdW50cmFja2VkIGZpbGVzIGlzIHRoZSBmb2xsb3dpbmcgbWFuaWZlc3QgYW5kIG5vdGhpbmcgZWxzZToKLSBgbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWRgIFNIQS0yNTYgYDBlOWM2OThiZmQxMWM4ZGI2ZjFlYmRjM2JjNWVlZGZmZTkyMmM3MTA1MzUzNTI4ZmYwYTc0NTllMGZmMzk2NGVgCi0gYGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9jb250cmFjdHMvaG9zdC1leGVjdXRpb24tZXBpc29kZS12MS5tZGAgU0hBLTI1NiBgZTk2Y2Y1YWY3YTM0YjU4OWNkZGE3YzI5ZGNkMzY0ZjAzNTk4YzM4MmNiZDYxZTY4ZTQ3ZDZiYmExOWNmZjNjZmAKLSBgbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL2luZGV4L1JFQURNRS5tZGAgU0hBLTI1NiBgYjExY2YzMDg2N2MyNzVkODk0M2FlMjQwOWRmMWJmYTcwYzZkNDUyZDFkMDNmNDFmNTJkZWE4Nzg1NTQ4N2NiMmAKLSBgbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZGAgU0hBLTI1NiBgNmUzYzllNGYwYTYzZGNjNWI0ODM0ZTQxNTkxODBkMmU2Y2Y1YmM4YjBiNmIxNmVhY2JkNzgxYThlMTZjNzQ3ZmAKClJlcXVpcmVkIG1hdGVyaWFsIGNvbnNlcXVlbmNlOiBhIGJsb2NraW5nIGZpbmRpbmcgbXVzdCBzaG93IGEgcmVhY2hhYmxlIGJyb2tlbiBzdXBwb3J0ZWQgTWFya2Rvd24gbGluay9hbmNob3I7IGxvc3MvcmVvcmRlcmluZy9jaGFuZ2Ugb2YgYSBuYW1lZCB2ZXJzaW9uLCBvcmRlcmVkIGZpZWxkLCBsaXRlcmFsLCBmZW5jZSwgbnVtYmVyZWQgcnVsZSwgbmVnYXRpdmUgcmVxdWlyZW1lbnQsIGV4Y2VwdGlvbiwgc3RhdHVzLCBhdXRob3JpdHkgb3Igc3VjY2Vzc29yIGJvdW5kYXJ5OyBhIHNlY29uZC9taXNzaW5nIGNhbm9uaWNhbCBvd25lcjsgZmFsc2UgcHJvdmVuYW5jZS9oYXNoL3JvbGxiYWNrL25hdmlnYXRpb247IG9yIGFuIGluLXNjb3BlIHBhdGgvc2NvcGUgdmlvbGF0aW9uIGNhdXNlZCBieSB0aGlzIHBhdGNoLgpCbG9ja2luZyB0aHJlc2hvbGQ6IG9ubHkgcmVhY2hhYmxlLCBtYXRlcmlhbCBQMSBvciBQMiBkZWZlY3RzIGludHJvZHVjZWQgYnkgdGhpcyBwYXRjaCBhbmQgY2F1c2FsbHkgdGllZCB0byBhbiBpbi1zY29wZSBpbnZhcmlhbnQuIFVuY2hhbmdlZCBwcmUtZXhpc3RpbmcgYmVoYXZpb3IgYW5kIHN0eWxpc3RpYy9lZGl0b3JpYWwgcHJlZmVyZW5jZXMgYXJlIG5vbi1ibG9ja2luZy4KQWNjZXB0ZWQgcHJpb3IgZmluZGluZ3M6IG5vbmUuCkRlZmVycmVkIG9yIG91dC1vZi1zY29wZSBjb25jZXJuczogdGhlIG5laWdoYm9yaW5nIHRyYW5zaXRpb24taW50ZW50LCBydW50aW1lLWV2ZW50LCByZXRhaW5lZC13b3JrZXIsIHBhY2tldC1sb2NhbCwgYW5kIGF1dGhvcml0eS1yZXF1aXJlZCBtYXRlcmlhbCBhbHJlYWR5IG93bmVkIGJ5IEQzL0Q1L0Q2OyBldmVyeSBvdGhlciBzdWJzdGFudGl2ZSBEMTAgY29udHJhY3QvZ2F0ZSB1bml0OyBEMTEgZXZpZGVuY2UgZGVjb21wb3NpdGlvbjsgRDEyIHJvb3QgY3V0b3ZlcjsgcnVudGltZS9SdXN0L3NjcmlwdHMvcGxhdGZvcm0gYmVoYXZpb3I7IHJvYWRtYXAgb3IgcmV2aWV3ZWQgWklQOyByZXZpZXcgZ292ZXJuYW5jZTsgRDkgc2xpY2VzL3Rhc2tzOyBicm9hZCBNYXJrZG93biBub3JtYWxpemF0aW9uOyB1bnJlbGF0ZWQgcHJlLWV4aXN0aW5nIGlzc3Vlcy4KUHJvamVjdCBzb3VyY2VzOiB0aGUgZnVsbCByZXZpZXcgaW5wdXQgaXMgdGhpcyBwcm9tcHQncyBleGFjdCBiYXNlbGluZSBpZGVudGl0eSwgY29udGVudC1hZGRyZXNzZWQgZm91ci1maWxlIG1hbmlmZXN0LCBjb21wbGV0ZSBwYXRjaCwgc291cmNlLWJvZHkgZGlnZXN0LCBhbmQgdmFsaWRhdGlvbiBldmlkZW5jZS4gRXhjbHVkZWQgc291cmNlIGlzIHVuYXZhaWxhYmxlIGFuZCBtdXN0IG5vdCBiZSBpbmZlcnJlZC4gUmVhZCB0aGUgY29tcGxldGUgcGF0Y2ggYmVmb3JlIGp1ZGdpbmcgdGhlIHJlc3VsdGluZyBtYW5pZmVzdC4KClZhbGlkYXRpb24gZXZpZGVuY2U6Ci0gZXhhY3QgZm91ci1wYXRoIGludmVudG9yeSBpbmNsdWRpbmcgdW50cmFja2VkOiBwYXNzOyBubyBzdGFnZWQgZmlsZXMuCi0gYGdpdCBkaWZmIC0tY2hlY2tgOiBwYXNzLgotIGBnaXQgZGlmZiAtLW5vLWluZGV4IC0tY2hlY2sgL2Rldi9udWxsIGxsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9jb250cmFjdHMvaG9zdC1leGVjdXRpb24tZXBpc29kZS12MS5tZGA6IG5vIHdoaXRlc3BhY2UgZGlhZ25vc3RpY3MgKGV4cGVjdGVkIG5vbnplcm8gZGlmZiBzdGF0dXMgb25seSkuCi0gYnl0ZSBleHRyYWN0aW9uIGJldHdlZW4gb3duZXIgYm91bmRhcnkgbWFya2VyczogZXhhY3RseSAxNjQ1IGJ5dGVzIGFuZCBTSEEtMjU2IGAyOGU4NmUyZTdlMjliNDU4MjgwODc1ZjkyMWNhMzI3YTQ2OTYyMWFjOTZiZDdlNTkyYTBhMTc2MDI3YWEwMTE3YCwgYnl0ZS1lcXVhbCB0byB0aGUgbGl2ZSBzb3VyY2UgYXJ0aWZhY3QuCi0gb3JkZXJlZC1maWVsZCwgZmVuY2VkLWNvZGUsIGxpdGVyYWwtb3JkZXIsIHJ1bGUgMeKAkzYsIG5lZ2F0aXZlLXJlcXVpcmVtZW50LCBhdXRob3JpdHktYm91bmRhcnksIGFuZCBzdGFibGUtdmVyc2lvbiBjaGVja3M6IHBhc3MgYnkgZXhhY3QgYnl0ZSBlcXVhbGl0eSBwbHVzIGV4cGxpY2l0IGFuY2hvciBjaGVja3MuCi0gcnVudGltZS1yZWZhY3RvciBNYXJrZG93biB2YWxpZGF0b3I6IGBMSU5LU19DSEVDS0VEIDE4MTIgRkFJTFVSRVMgMGAuCi0gcm9vdCBhbmQgZGVzdGluYXRpb24gYW5jaG9ycyBlYWNoIGV4aXN0IGV4YWN0bHkgb25jZTsgaW5kZXggYW5kIGxlZGdlciBsaW5rcyByZXNvbHZlLgotIGV4cGxpY2l0IHByaW9yLW93bmVyIGFuZCByZXZpZXctYXJ0aWZhY3Qgc3RhYmlsaXR5IGNoZWNrczogcGFzcyBmb3IgYHJldmlldy1jb250cm9sL2AsIEQ1L0Q2IHBhY2tldC1mYW1pbHkgZGlyZWN0b3JpZXMsIGBhcmNoaXRlY3R1cmUvYCwgYHNlYW1zL2AsIGBzbGljZXMvYCwgYW5kIGBkb2NzL2d1aWRhbmNlL2AuCi0gY29tcGxldGUgcGF0Y2ggZm9yd2FyZC1hcHBseSBmcm9tIGEgY2xlYW4gZGlzcG9zYWJsZSBleHBvcnQ6IHBhc3MuCi0gY29tcGxldGUgcGF0Y2ggcmV2ZXJzZS1hcHBseSBmcm9tIHRoZSBtb2RpZmllZCBkaXNwb3NhYmxlIGV4cG9ydDogcGFzcy4KLSB2YWxpZGF0aW9uIGxvZyBTSEEtMjU2OiBgYjczMjI4YTMzMjkzYjVjZjBkYmVhNjUwZTJhNDE1YmRkYzA5YTU1MmNkMWM0YzAwNmJjMjEyZWYzZmJjNzczN2AuCi0gY29tcGxldGUgcGF0Y2ggU0hBLTI1NjogYGMyZjZlY2VhODZhMGZiNDYwYmRlODM3ZmVkZWVkNjM5OGY3OWU3YzJiMGJiODgxZDk3YTE3MmRlYjAwMTIyYjlgLgoKQWR2aXNvcnkgb25seTsgdmVyaWZ5IGFnYWluc3QgbG9jYWwgcHJvamVjdCB0cnV0aCBhbmQgYXV0aG9yaXRhdGl2ZSBkb2NzOyBkbyBub3QgcmVkdWNlIHNjb3BlIHdpdGhvdXQgdXNlciBhcHByb3ZhbC4KCkRvIG5vdCByZWR1Y2UgdGhlIHRhc2sgb3IgcmVwbGFjZSBpdCB3aXRoIGFuIGVhc2llciBhbHRlcm5hdGl2ZS4gUHJlc2VydmUgdGhlIHJlcXVlc3RlZCBzY29wZSBhbmQgcHJvamVjdCBjb252ZW50aW9ucy4KClJldHVybiBmaW5kaW5ncyBmaXJzdCBieSBzZXZlcml0eS4gRm9yIGV2ZXJ5IGZpbmRpbmcgaW5jbHVkZSBmaWxlL2hlYWRpbmcsIHN1cHBvcnRlZC1pbnB1dCByZWFjaGFiaWxpdHksIHZpb2xhdGVkIGludmFyaWFudCwgbWF0ZXJpYWwgY29uc2VxdWVuY2UsIGFuZCBwYXRjaCBjYXVzYWxpdHkuIFN0YXRlIGV4cGxpY2l0bHkgd2hlbiB0aGVyZSBhcmUgbm8gcXVhbGlmeWluZyBmaW5kaW5ncy4gRW5kIHdpdGggZXhhY3RseSBvbmUgdmVyZGljdCBsaW5lOiBgVkVSRElDVDogQVBQUk9WRURgIG9yIGBWRVJESUNUOiBDSEFOR0VTIFJFUVVJUkVEYC4KCkNvbXBsZXRlIHBhdGNoIChVVEYtOCwgaW5jbHVkZXMgdGhlIHVudHJhY2tlZCBuZXcgb3duZXI7IG5vIG9taXR0ZWQgZmlsZXMpOgpgYGBkaWZmCmRpZmYgLS1naXQgYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kCmluZGV4IDQwMDE2NzI2Yy4uYTljZGY1ODZiIDEwMDY0NAotLS0gYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZAorKysgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZApAQCAtNTAsNTMgKzUwLDcgQEAgQ29tcGF0aWJpbGl0eSBhbmNob3Igb25seTsgY2Fub25pY2FsIGNvbnRlbnQ6IFtgYTEtMi1lYXJsaWVyLWhpc3Rvcmllcy9jb250cmFjdHMKIAogIyMgMi4gYEhvc3RFeGVjdXRpb25FcGlzb2RlVjFgCiAKLWBgYHJ1c3QKLXN0cnVjdCBIb3N0RXhlY3V0aW9uRXBpc29kZVYxIHsKLSAgICBzY2hlbWFfdmVyc2lvbjogdTMyLCAgICAgICAgICAgICAgICAgLy8gZXhhY3RseSAxCi0gICAgZXBpc29kZV9pZDogU3RyaW5nLAotICAgIGtpbmQ6IEhvc3RFeGVjdXRpb25FcGlzb2RlS2luZFYxLAotICAgIG9yY2hlc3RyYXRpb25fc2Vzc2lvbl9pZDogU3RyaW5nLAotICAgIG9ic2VydmVkX2F1dGhvcml0eV9yZXZpc2lvbjogdTY0LAotICAgIGJhY2tlbmRfaWQ6IE9wdGlvbjxTdHJpbmc+LAotICAgIHByb2Nlc3NfcmVmOiBPcHRpb248UHJvY2Vzc1JlZlYxPiwKLSAgICB0cmFuc3BvcnRfc3RhdHVzOiBIb3N0RXhlY3V0aW9uRXBpc29kZVRyYW5zcG9ydFN0YXR1c1YxLAotICAgIHN0YXJ0ZWRfYXQ6IFRpbWVzdGFtcCwKLSAgICBsYXN0X2hlYXJ0YmVhdF9hdDogT3B0aW9uPFRpbWVzdGFtcD4sCi0gICAgZW5kZWRfYXQ6IE9wdGlvbjxUaW1lc3RhbXA+LAotICAgIGV4aXRfb2JzZXJ2YXRpb246IE9wdGlvbjxFcGlzb2RlRXhpdE9ic2VydmF0aW9uVjE+LAotfQotYGBgCi0KLUVwaXNvZGUga2luZHM6Ci0KLWBgYHRleHQKLVJlcGxBdHRhY2hlZEVwaXNvZGUKLUhpZGRlbk93bmVySGVscGVyU3RhcnRFcGlzb2RlCi1IaWRkZW5Pd25lckhlbHBlckF0dGFjaEVwaXNvZGUKLUhpZGRlbk93bmVySGVscGVyUmVzdW1lT25lVHVybkVwaXNvZGUKLVJ1bnRpbWVUb29sYm94RXBpc29kZQotU3ludGhldGljT3JSZWNvdmVyZWRFcGlzb2RlCi1gYGAKLQotVHJhbnNwb3J0IHN0YXR1czoKLQotYGBgdGV4dAotQXZhaWxhYmxlCi1VbmF2YWlsYWJsZUJ1dER1cmFibGVBdXRob3JpdHlFeGlzdHMKLVVuYXZhaWxhYmxlQW5kTm9BdXRob3JpdGF0aXZlUm91dGUKLVN0YWxlT3JPcnBoYW5lZAotYGBgCi0KLVJ1bGVzOgotCi0xLiBFcGlzb2RlcyBzdWJtaXQgb2JzZXJ2YXRpb25zIGFuZCByZXF1ZXN0ZWQgdHJhbnNpdGlvbnMgdG8gYEhvc3RTZXNzaW9uQXV0aG9yaXR5YDsgdGhleSBuZXZlciB3cml0ZSBkdXJhYmxlIHBvc3R1cmUgZGlyZWN0bHkuCi0yLiBBbiBvYnNlcnZhdGlvbiB3aG9zZSBgb2JzZXJ2ZWRfYXV0aG9yaXR5X3JldmlzaW9uYCBpcyBzdGFsZSBjYW5ub3QgbXV0YXRlIGF1dGhvcml0eS4KLTMuIEVwaXNvZGUgZXhpdCBkb2VzIG5vdCBkZWxldGUgc2Vzc2lvbiwgd29ya2VyLCBiaW5kaW5nLCByZWNlaXB0LCBvciBvYmxpZ2F0aW9uIHRydXRoLgotNC4gUHJpdmF0ZSB0cmFuc3BvcnQgc3VjY2VzcyBtYXkgYWNjZWxlcmF0ZSBkZWxpdmVyeTsgaXQgZG9lcyBub3QgZGVmaW5lIGR1cmFibGUgc3VjY2Vzcy4KLTUuIFBJRCwgcHJvY2Vzcy9oZWxwZXIgcHJlc2VuY2UsIGFjdGl2ZSBoYW5kbGVzLCByZWFkaW5lc3MsIGFuZCBwcm9tcHQtc3RyZWFtIHN0YXRlIGFyZSBlcGlzb2RlIG9yCi0gICB0cmFuc3BvcnQgb2JzZXJ2YXRpb25zIG9ubHk7IHRoZWlyIGFic2VuY2UgZG9lcyBub3QgZXJhc2UgYFBhcmtlZFJlc3VtYWJsZWAgYXV0aG9yaXR5LgotNi4gRXBpc29kZSBjb25zdHJ1Y3Rpb24gYW5kIGxhdW5jaCBmb2xsb3cgZHVyYWJsZSB0cmFuc2l0aW9uIGFwcGxpY2F0aW9uIGFuZCBjYW5ub3QgcmVzZXQgYSBwYXJrZWQKLSAgIHNlc3Npb24gdG8gYEFsbG9jYXRpbmdgIG9yIGF1dGhvcml6ZSBhIHN1Y2Nlc3NvciBwYXJ0aWNpcGFudC4KK0Nhbm9uaWNhbCBjb250ZW50OiBbYGNvbnRyYWN0cy9ob3N0LWV4ZWN1dGlvbi1lcGlzb2RlLXYxLm1kIzItaG9zdGV4ZWN1dGlvbmVwaXNvZGV2MWBdKGNvbnRyYWN0cy9ob3N0LWV4ZWN1dGlvbi1lcGlzb2RlLXYxLm1kIzItaG9zdGV4ZWN1dGlvbmVwaXNvZGV2MSkuCiAKICMjIDJBLiBSdW50aW1lIGV2ZW50IGlkZW50aXR5IGFuZCBvcmRlcmluZyBjYXJyaWVyCiAKZGlmZiAtLWdpdCBhL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9SRUFETUUubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvaW5kZXgvUkVBRE1FLm1kCmluZGV4IGFjOWE3ZWRkOS4uYTAwYjgyMTcwIDEwMDY0NAotLS0gYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvaW5kZXgvUkVBRE1FLm1kCisrKyBiL2xsbS1sYXN0LW1pbGUvcnVudGltZS1yZWZhY3Rvci9pbmRleC9SRUFETUUubWQKQEAgLTE3LDYgKzE3LDcgQEAKIHwgYG1hY09TLWRldi1wYXJpdHktZmFtaWx5YCB8IGxhbmUgaW5kZXggfCBbYG1hY29zLWRldi1wYXJpdHkvUkVBRE1FLm1kYF0oLi4vbWFjb3MtZGV2LXBhcml0eS9SRUFETUUubWQpIHwgYG5vbi1hdXRob3JpdGF0aXZlIG5hdmlnYXRpb24gZm9yIHRoZSBtYWNPUyBkZXZlbG9wZXItcGFyaXR5IGRlY2lzaW9uLCBleHRyYWN0ZWQgcHJvamVjdGlvbnMsIGFuZCBsYW5lLWxvY2FsIGdhdGVgIHwgQXNzZW1ibGVzIHRoZSBjdXJyZW50IGRlY2lzaW9uLCBleHRyYWN0ZWQgYDAwYC9gMDJgL2AwM2AvYDA1YCBwcm9qZWN0aW9ucywgYW5kIHRoZSBsYW5lLWxvY2FsIGdhdGUgd2l0aG91dCBhdXRob3JpemluZyBQaGFzZSAyLCBMaW51eC1maXJzdCBzdWNjZXNzb3Igd29yaywgcHJvdGVjdGVkLWxpZmVjeWNsZSByZXZpdmFsLCBvciBXaW5kb3dzIGRpc3BhdGNoLiB8IFtgMDBgXSguLi8wMC1SRUFETUUubWQjbWFjb3MtZGV2ZWxvcGVyLXBhcml0eS1sYW5lLTIwMjYtMDgtMTktc2NvcGVkKSwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1kI2N1cnJlbnQtY3Jvc3MtbGFuZS1zY2hlZHVsaW5nLWFuZC1vd25lcnNoaXAtMjAyNi0wOC0yMC1jb250cm9sbGluZyksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjY3VycmVudC1saW51eC1maXJzdC1zY2hlZHVsaW5nLXJlc2V0LTIwMjYtMDgtMjAtY29udHJvbGxpbmcpLCBbYDA1YF0oLi4vMDUtZGVidWctcmVncmVzc2lvbi1sZWRnZXIubWQjY3VycmVudC1jcm9zcy1sYW5lLXJlZ3Jlc3Npb24tbGVkZ2VyLTIwMjYtMDgtMjAtY29udHJvbGxpbmcpIHwKIHwgYEFVVEhPUklUWV9SRVFVSVJFRDpNQUNPU19ERVZfUEFSSVRZYCB8IGdhdGUgfCBbYGdhdGVzL2F1dGhvcml0eS1yZXF1aXJlZC1tYWNvcy1kZXYtcGFyaXR5Lm1kYF0oLi4vZ2F0ZXMvYXV0aG9yaXR5LXJlcXVpcmVkLW1hY29zLWRldi1wYXJpdHkubWQpIHwgbGFuZS1sb2NhbCBhdXRob3JpdHktcmVxdWlyZWQgZ2F0ZTsgUGhhc2UgMSBncmFudHMgbm8gUGhhc2UgMiBhdXRob3JpdHkgfCBEb2VzIG5vdCBibG9jayBvciBhdXRob3JpemUgdGhlIExpbnV4LWZpcnN0IHJ1bnRpbWUtcmVmYWN0b3Igc2VxdWVuY2UuIHwgW2AwNGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjYXV0aG9yaXR5X3JlcXVpcmVkbWFjb3NfZGV2X3Bhcml0eS1jb250cmFjdC0yMDI2LTA4LTE5LW1hY29zLWxhbmUpIHwKIHwgYEFVVEhPUklUWV9SRVFVSVJFRDpSVU5USU1FX1JFRkFDVE9SX1JFRU5UUllgIHwgZ2F0ZSB8IFtgZ2F0ZXMvYXV0aG9yaXR5LXJlcXVpcmVkLXJ1bnRpbWUtcmVmYWN0b3ItcmVlbnRyeS5tZGBdKC4uL2dhdGVzL2F1dGhvcml0eS1yZXF1aXJlZC1ydW50aW1lLXJlZmFjdG9yLXJlZW50cnkubWQpIHwgY2xvc2VkIGRvY3VtZW50YXRpb24vY29udHJvbC1wbGFuZSBzZWxlY3Rpb24gZ2F0ZSB8IENsb3NlZCBhcyBzZWxlY3Rpb24gaGlzdG9yeTsgQTEuMy1QMSBpcyB0aGUgc2VwYXJhdGUgYWN0aXZlIHBhY2tldC4gfCBbYDA0YF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCNhdXRob3JpdHlfcmVxdWlyZWRydW50aW1lX3JlZmFjdG9yX3JlZW50cnktY29udHJhY3QtMjAyNi0wOC0yMC1jbG9zZWQtc2VsZWN0aW9uLXJlY29yZCkgfAorfCBgSG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWAgfCBjb250cmFjdCB8IFtgY29udHJhY3RzL2hvc3QtZXhlY3V0aW9uLWVwaXNvZGUtdjEubWRgXSguLi9jb250cmFjdHMvaG9zdC1leGVjdXRpb24tZXBpc29kZS12MS5tZCkgfCBgY2Fub25pY2FsIGV4dHJhY3RlZCBjb250cmFjdCBvd25lciBmb3IgdGhlIGNvbXBsZXRlIHNjaGVtYSwgZXBpc29kZS1raW5kIGxpdGVyYWxzLCB0cmFuc3BvcnQtc3RhdHVzIGxpdGVyYWxzLCBhbmQgcnVsZXMgMeKAkzZgIHwgU3VwZXJzZWRlcyBvbmx5IHJvb3QtY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIGBIb3N0RXhlY3V0aW9uRXBpc29kZVYxYCBzcGFuOyB0aGUgY29tbWl0dGVkIHByZS1zcGFuIGNvbXBhdGliaWxpdHkgYW5jaG9ycyBhbmQgbmVpZ2hib3JpbmcgYDJBYC9gMkJgIG93bmVycyByZW1haW4gdW5jaGFuZ2VkLiB8IFtgMDRgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzItaG9zdGV4ZWN1dGlvbmVwaXNvZGV2MSkgfAogfCBgc2hhcmVkLXRhcmdldC1hcmNoaXRlY3R1cmVgIHwgYXJjaGl0ZWN0dXJlIGluZGV4IHwgW2BhcmNoaXRlY3R1cmUvUkVBRE1FLm1kYF0oLi4vYXJjaGl0ZWN0dXJlL1JFQURNRS5tZCkgfCBgbm9uLWF1dGhvcml0YXRpdmUgbmF2aWdhdGlvbiBmb3IgdGhlIGV4dHJhY3RlZCBleGVjdXRpdmUgdGFyZ2V0IGRlY2lzaW9uLCBhdXRob3JpdHkgbWFwLCBudW1iZXJlZCBpbnZhcmlhbnRzLCBhbmQgc3RhYmxlIHJldmlldyBxdWVzdGlvbmAgfCBTdXBlcnNlZGVzIG9ubHkgcm9vdC1jYW5vbmljYWwgb3duZXJzaGlwIG9mIHRoZSBleHRyYWN0ZWQgc2hhcmVkIGFyY2hpdGVjdHVyZSBzcGFuczsgcGFja2V0LWZhbWlseS1sb2NhbCBENS9ENiBmb3J3YXJkZXJzIHJlbWFpbiBhdCB0aGVpciBleGlzdGluZyBvd25lcnMuIHwgW2AwMWBdKC4uLzAxLXRhcmdldC1hcmNoaXRlY3R1cmUubWQjZXhlY3V0aXZlLWRlY2lzaW9uKSwgW2AwMWBdKC4uLzAxLXRhcmdldC1hcmNoaXRlY3R1cmUubWQjYXV0aG9yaXR5LW1hcCksIFtgMDFgXSguLi8wMS10YXJnZXQtYXJjaGl0ZWN0dXJlLm1kI25vbi1uZWdvdGlhYmxlLWludmFyaWFudHMpLCBbYDAxYF0oLi4vMDEtdGFyZ2V0LWFyY2hpdGVjdHVyZS5tZCNyZXZpZXctcXVlc3Rpb24pIHwKIHwgYHNoYXJlZC1zZWFtLWNyb3Nzd2Fsa2AgfCBzZWFtIGluZGV4IHwgW2BzZWFtcy9SRUFETUUubWRgXSguLi9zZWFtcy9SRUFETUUubWQpIHwgYGNhbm9uaWNhbCBzaGFyZWQgc2VhbS1jcm9zc3dhbGsgcnVsZXMgcGx1cyBleHRyYWN0ZWQgc2VhbS1mYW1pbHkgbmF2aWdhdGlvbiBmb3IgaG9zdC9zZXNzaW9uLCBwZXJzaXN0ZW5jZS9jb21wYXRpYmlsaXR5LCBkaXNwYXRjaC9lcGlzb2RlIHRyYW5zcG9ydCwgcG9saWN5L25hcnJvd2luZywgcnVudGltZS1ldmVudC9yZWNlaXB0L3N1cGVydmlzaW9uL3JldGFpbmVkLXJ1bnRpbWUsIG9ibGlnYXRpb25zL2hvc3QgcmUtZW5nYWdlbWVudCwgY29uZmlndXJhdGlvbi9nYXRld2F5IGFkb3B0aW9uLCBhbmQgVUFBL3Byb3ZpZGVyIHJlYWxpemF0aW9uL3NpZGUtZWZmZWN0IG1lZGlhdGlvbmAgfCBTdXBlcnNlZGVzIG9ubHkgdGhlIGV4dHJhY3RlZCByb290IHJlYWRpbmctcnVsZS9jbGFzc2lmaWNhdGlvbiBzcGFucyBhbmQgdGhlIGV4dHJhY3RlZCBob3N0L3Nlc3Npb24gYXV0aG9yaXR5LCBwZXJzaXN0ZW5jZS9jb21wYXRpYmlsaXR5LCBkaXNwYXRjaC9lcGlzb2RlIHRyYW5zcG9ydCwgcG9saWN5L25hcnJvd2luZywgcnVudGltZS1ldmVudC9yZWNlaXB0L3N1cGVydmlzaW9uL3JldGFpbmVkLXJ1bnRpbWUsIG9ibGlnYXRpb25zL2hvc3QgcmUtZW5nYWdlbWVudCwgY29uZmlndXJhdGlvbi9nYXRld2F5IGFkb3B0aW9uLCBhbmQgVUFBL3Byb3ZpZGVyIHJlYWxpemF0aW9uL3NpZGUtZWZmZWN0IG1lZGlhdGlvbiBmYW1pbHkgcm93czsgQTAgYW5kIHRoZSBleGlzdGluZyBENiBgSG9zdFNlc3Npb25BdXRob3JpdHlgLCBgV29ybGRXb3JrZXJNZXNzYWdpbmdQcm90b2NvbGAsIGFuZCBgT2JsaWdhdGlvbkxlZGdlcmAgY29tcGF0aWJpbGl0eSByb3dzIHJlbWFpbiBhdCB0aGVpciBjdXJyZW50IG93bmVycy4gfCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjcmVhZGluZy1ydWxlKSwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1kI2EtaG9zdC1hdXRob3JpdHktYW5kLWluZ3Jlc3MpLCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjYi1kaXNwYXRjaC1wb2xpY3ktcmVjZWlwdHMtYW5kLXJldGFpbmVkLXJ1bnRpbWUpLCBbYDAyYF0oLi4vMDItc2VhbS1jcm9zc3dhbGsubWQjYy1vYmxpZ2F0aW9ucy1hbmQtaG9zdC1yZS1lbmdhZ2VtZW50KSwgW2AwMmBdKC4uLzAyLXNlYW0tY3Jvc3N3YWxrLm1kI2QtdWFhLXJlYWxpemF0aW9uLXByb2plY3Rpb24tYW5kLXNpZGUtZWZmZWN0LW1lZGlhdGlvbiksIFtgMDJgXSguLi8wMi1zZWFtLWNyb3Nzd2Fsay5tZCNjbGFzc2lmaWNhdGlvbi1jb25zZXF1ZW5jZXMpIHwKIHwgYHNoYXJlZC1zbGljZS1tYXBgIHwgc2xpY2UgaW5kZXggfCBbYHNsaWNlcy9SRUFETUUubWRgXSguLi9zbGljZXMvUkVBRE1FLm1kKSB8IGBjYW5vbmljYWwgc2hhcmVkIHNlcXVlbmNpbmcvZGVwZW5kZW5jeSBhbmQgc2xpY2UtY2xvc2VvdXQgb3duZXIgcGx1cyBub24tYXV0aG9yaXRhdGl2ZSBEOSBUcmFjayBB4oCTRSBuYXZpZ2F0aW9uIGFuZCBBMS9BMS40IHByb2plY3Rpb24gaW5kZXhgIHwgU3VwZXJzZWRlcyBvbmx5IHJvb3QtY2Fub25pY2FsIG93bmVyc2hpcCBvZiB0aGUgZXh0cmFjdGVkIHNoYXJlZCBzZXF1ZW5jaW5nL2Nsb3Nlb3V0IHNwYW5zIHBsdXMgdGhlIGV4dHJhY3RlZCBUcmFjayBB4oCTRSBhbmQgQTEvQTEuNCBwcm9qZWN0aW9uIHNwYW5zOyBjb250cm9sbGluZyBzY2hlZHVsZSBhdXRob3JpdHkgcmVtYWlucyB3aXRoIGRlY2lzaW9ucywgcGFja2V0cywgZ2F0ZXMsIGFuZCBgY3VycmVudC5tZGAuIHwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCNzZXF1ZW5jaW5nLXJ1bGVzKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCN0cmFjay1hLS1hdXRob3JpdHktYW5kLXN1cmZhY2UtbmV1dHJhbGl0eSksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjYTEtYm91bmRlZC1wYWNrZXQtZGVjb21wb3NpdGlvbiksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjdHJhY2stYi0td29ybGQtZGlzcGF0Y2gtcmVjZWlwdHMtc3VwZXJ2aXNpb24tYW5kLWNhbmNlbCksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjdHJhY2stYy0tb2JsaWdhdGlvbnMtaW5ib3gtYXV0by1hdHRhY2gtYW5kLXJvdXRlci1hdHRhY2gpLCBbYDAzYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3RyYWNrLWQtLXVhYS1leGVjdXRpb24tZW52ZWxvcGUtYW5kLXNpZGUtZWZmZWN0LW1lZGlhdGlvbiksIFtgMDNgXSguLi8wMy1waGFzZS1zbGljZS1tYXAubWQjdHJhY2stZS0tZGlzcGF0Y2gtc2NvcGVkLXBvbGljeS1uYXJyb3dpbmctYW5kLWNvbmZpZy1wcm9qZWN0aW9uKSwgW2AwM2BdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCNzbGljZS1jbG9zZW91dC1taW5pbXVtKSB8CmRpZmYgLS1naXQgYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJhY3Rpb24tbGVkZ2VyLm1kIGIvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZAppbmRleCBmOWFmZjYxYmMuLmM0NmE5ZjM4ZCAxMDA2NDQKLS0tIGEvbGxtLWxhc3QtbWlsZS9ydW50aW1lLXJlZmFjdG9yL21pZ3JhdGlvbi9leHRyYWN0aW9uLWxlZGdlci5tZAorKysgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvbWlncmF0aW9uL2V4dHJhY3Rpb24tbGVkZ2VyLm1kCkBAIC0xODQsMyArMTg0LDQgQEAgVGhpcyBsZWRnZXIgcmVjb3JkcyBjb250ZW50LXByZXNlcnZpbmcgYXV0aG9yaXR5IHRyYW5zZmVycyB3aGlsZSByZXRhaW5pbmcgcmVxdWkKIHwgRDkgfCBbYDAzLXBoYXNlLXNsaWNlLW1hcC5tZGBdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCkgfCBFMyDigJQgQWdlbnRDb25maWdQcm9qZWN0aW9uU2VydmljZSBhbmQgZ2F0ZXdheSBhZG9wdGlvbiB8IFtgdHJhY2stZS0tZGlzcGF0Y2gtc2NvcGVkLXBvbGljeS1uYXJyb3dpbmctYW5kLWNvbmZpZy1wcm9qZWN0aW9uYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3RyYWNrLWUtLWRpc3BhdGNoLXNjb3BlZC1wb2xpY3ktbmFycm93aW5nLWFuZC1jb25maWctcHJvamVjdGlvbikgfCBsaW5lIDIyMiB8IGA5M2IwZmY1ODJhNTczZTI5Y2VhOWIyZjMwZjQ5YmZjMDI3OGMzODNmMWU1NjQzMDFhZmRmOTQ1NzllNTY3MmU1YCB8IFtgLi4vc2xpY2VzL2UzLWFnZW50LWNvbmZpZy1wcm9qZWN0aW9uLWFuZC1nYXRld2F5LWFkb3B0aW9uLm1kYF0oLi4vc2xpY2VzL2UzLWFnZW50LWNvbmZpZy1wcm9qZWN0aW9uLWFuZC1nYXRld2F5LWFkb3B0aW9uLm1kKSB8IHNsaWNlIHJvdyB8IGNhbm9uaWNhbCBkZXN0aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgdGFibGUgcm93IHwgbm9uZSB8IHJlc3RvcmUgdGhlIGV4YWN0IGV4dHJhY3RlZCBEOSBzb3VyY2UgYm9kaWVzL3Jvd3MgaW4gYDAzLXBoYXNlLXNsaWNlLW1hcC5tZGA7IHJlbW92ZSBgc2xpY2VzL2AgYW5kIGFsbCBEOS1hZGRlZCBmaWxlcyBiZW5lYXRoIGl0OyByZW1vdmUgdGhlIGBzaGFyZWQtc2xpY2UtbWFwYCBpbmRleCByb3cgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsgcmVtb3ZlIGFsbCBtYXRjaGluZyBEOSBsZWRnZXIgZW50cmllczsgbGVhdmUgRDXigJNEOCBjYW5vbmljYWwgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGFuZCBgcmV2aWV3LWNvbnRyb2wvYCB1bmNoYW5nZWQgfAogfCBEOSB8IFtgMDMtcGhhc2Utc2xpY2UtbWFwLm1kYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kKSB8IEU0IOKAlCBIb3N0LXZpc2libGUgd3JpdGUvc3luYyBjb250cmFjdCB8IFtgdHJhY2stZS0tZGlzcGF0Y2gtc2NvcGVkLXBvbGljeS1uYXJyb3dpbmctYW5kLWNvbmZpZy1wcm9qZWN0aW9uYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kI3RyYWNrLWUtLWRpc3BhdGNoLXNjb3BlZC1wb2xpY3ktbmFycm93aW5nLWFuZC1jb25maWctcHJvamVjdGlvbikgfCBsaW5lIDIyMyB8IGBjOThhNWRmM2YwOTczNWFiYzVjYmVmZWMyODVjN2I1ZGM1NWZjYTVjNjdjZmRmNWJmNmJkM2Y0NThkMDE3ODkzYCB8IFtgLi4vc2xpY2VzL2U0LWhvc3QtdmlzaWJsZS13cml0ZS1zeW5jLWNvbnRyYWN0Lm1kYF0oLi4vc2xpY2VzL2U0LWhvc3QtdmlzaWJsZS13cml0ZS1zeW5jLWNvbnRyYWN0Lm1kKSB8IHNsaWNlIHJvdyB8IGNhbm9uaWNhbCBkZXN0aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgdGFibGUgcm93IHwgbm9uZSB8IHJlc3RvcmUgdGhlIGV4YWN0IGV4dHJhY3RlZCBEOSBzb3VyY2UgYm9kaWVzL3Jvd3MgaW4gYDAzLXBoYXNlLXNsaWNlLW1hcC5tZGA7IHJlbW92ZSBgc2xpY2VzL2AgYW5kIGFsbCBEOS1hZGRlZCBmaWxlcyBiZW5lYXRoIGl0OyByZW1vdmUgdGhlIGBzaGFyZWQtc2xpY2UtbWFwYCBpbmRleCByb3cgZnJvbSBgaW5kZXgvUkVBRE1FLm1kYDsgcmVtb3ZlIGFsbCBtYXRjaGluZyBEOSBsZWRnZXIgZW50cmllczsgbGVhdmUgRDXigJNEOCBjYW5vbmljYWwgb3duZXJzLCBgaW5kZXgvY3VycmVudC5tZGAsIGFuZCBgcmV2aWV3LWNvbnRyb2wvYCB1bmNoYW5nZWQgfAogfCBEOSB8IFtgMDMtcGhhc2Utc2xpY2UtbWFwLm1kYF0oLi4vMDMtcGhhc2Utc2xpY2UtbWFwLm1kKSB8IEExLjQg4oCUIGJvdW5kZWQgYXV0by1hdHRhY2ggcHJvZHVjZXIgYWRvcHRpb24gYW5kIHJlZ3Jlc3Npb24gY2xvc3VyZSB8IFtgYTEtYm91bmRlZC1wYWNrZXQtZGVjb21wb3NpdGlvbmBdKC4uLzAzLXBoYXNlLXNsaWNlLW1hcC5tZCNhMS1ib3VuZGVkLXBhY2tldC1kZWNvbXBvc2l0aW9uKSB8IGxpbmUgMTQ1IHwgYDU4ZDc5ODJiMjNhN2IyZjVmNDFmM2UwYjUxNzA5OTZlMGYzYjMzMWRmYmJjYmI1ODc1ZDAyZTgwMWJkNWQ0MDhgIHwgW2AuLi9zbGljZXMvdGFza3MvYTEtNC1hdXRvLWF0dGFjaC1wcm9kdWNlci1hZG9wdGlvbi5tZGBdKC4uL3NsaWNlcy90YXNrcy9hMS00LWF1dG8tYXR0YWNoLXByb2R1Y2VyLWFkb3B0aW9uLm1kKSB8IHRhc2sgcm93IHwgY2Fub25pY2FsIGRlc3RpbmF0aW9uOyBzb3VyY2UgY29tcGF0aWJpbGl0eSB0YWJsZSByb3cgfCBub25lIHwgcmVzdG9yZSB0aGUgZXhhY3QgZXh0cmFjdGVkIEQ5IHNvdXJjZSBib2RpZXMvcm93cyBpbiBgMDMtcGhhc2Utc2xpY2UtbWFwLm1kYDsgcmVtb3ZlIGBzbGljZXMvYCBhbmQgYWxsIEQ5LWFkZGVkIGZpbGVzIGJlbmVhdGggaXQ7IHJlbW92ZSB0aGUgYHNoYXJlZC1zbGljZS1tYXBgIGluZGV4IHJvdyBmcm9tIGBpbmRleC9SRUFETUUubWRgOyByZW1vdmUgYWxsIG1hdGNoaW5nIEQ5IGxlZGdlciBlbnRyaWVzOyBsZWF2ZSBENeKAk0Q4IGNhbm9uaWNhbCBvd25lcnMsIGBpbmRleC9jdXJyZW50Lm1kYCwgYW5kIGByZXZpZXctY29udHJvbC9gIHVuY2hhbmdlZCB8Cit8IEQxMCB8IFtgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZGBdKC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQpIHwgMi4gYEhvc3RFeGVjdXRpb25FcGlzb2RlVjFgIHwgW2AyLWhvc3RleGVjdXRpb25lcGlzb2RldjFgXSguLi8wNC1jb250cmFjdHMtYW5kLWdhdGVzLm1kIzItaG9zdGV4ZWN1dGlvbmVwaXNvZGV2MSkgfCBsaW5lcyA1MeKAkzEwMCB8IGAyOGU4NmUyZTdlMjliNDU4MjgwODc1ZjkyMWNhMzI3YTQ2OTYyMWFjOTZiZDdlNTkyYTBhMTc2MDI3YWEwMTE3YCB8IFtgLi4vY29udHJhY3RzL2hvc3QtZXhlY3V0aW9uLWVwaXNvZGUtdjEubWRgXSguLi9jb250cmFjdHMvaG9zdC1leGVjdXRpb24tZXBpc29kZS12MS5tZCkgfCBjb250cmFjdCB8IGNhbm9uaWNhbCBkZXN0aW5hdGlvbjsgc291cmNlIGNvbXBhdGliaWxpdHkgYW5jaG9yIHwgbm9uZSB8IHJlc3RvcmUgdGhlIGV4YWN0IDE2NDUtYnl0ZSBzb3VyY2Ugc3BhbiBhdCBgMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMyLWhvc3RleGVjdXRpb25lcGlzb2RldjFgOyByZW1vdmUgYGNvbnRyYWN0cy9ob3N0LWV4ZWN1dGlvbi1lcGlzb2RlLXYxLm1kYDsgcmVtb3ZlIHRoZSBleGFjdCBgSG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWAgcm93IGZyb20gYGluZGV4L1JFQURNRS5tZGA7IHJlbW92ZSB0aGlzIEQxMCBsZWRnZXIgZW50cnk7IGxlYXZlIHRoZSBjb21taXR0ZWQgcHJlLXNwYW4gY29tcGF0aWJpbGl0eSBhbmNob3JzLCBEMy9ENeKAk0Q5IG93bmVycywgYGluZGV4L2N1cnJlbnQubWRgLCBgcmV2aWV3LWNvbnRyb2wvYCwgYWxsIG90aGVyIEQxMCB1bml0cywgYW5kIGV2ZXJ5IHBhdGggb3V0c2lkZSB0aGUgZm91ci1wYXRoIGZlbmNlIHVuY2hhbmdlZCB8CmRpZmYgLS1naXQgYS9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvY29udHJhY3RzL2hvc3QtZXhlY3V0aW9uLWVwaXNvZGUtdjEubWQgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvY29udHJhY3RzL2hvc3QtZXhlY3V0aW9uLWVwaXNvZGUtdjEubWQKbmV3IGZpbGUgbW9kZSAxMDA2NDQKaW5kZXggMDAwMDAwMDAwLi41Yjc4NDAyN2EKLS0tIC9kZXYvbnVsbAorKysgYi9sbG0tbGFzdC1taWxlL3J1bnRpbWUtcmVmYWN0b3IvY29udHJhY3RzL2hvc3QtZXhlY3V0aW9uLWVwaXNvZGUtdjEubWQKQEAgLTAsMCArMSw1OCBAQAorKipLaW5kOioqIGNvbnRyYWN0CisqKlN0YXR1czoqKiBjYW5vbmljYWwKKyoqQ2Fub25pY2FsIGZvcjoqKiBjb21wbGV0ZSBleHRyYWN0ZWQgYEhvc3RFeGVjdXRpb25FcGlzb2RlVjFgIHNjaGVtYSwgZXBpc29kZS1raW5kIGxpdGVyYWxzLCB0cmFuc3BvcnQtc3RhdHVzIGxpdGVyYWxzLCBhbmQgcnVsZXMgMeKAkzYKKyoqU291cmNlIHByb3ZlbmFuY2U6KiogZXh0cmFjdGVkIGJ5dGUtZm9yLWJ5dGUgZnJvbSBbYC4uLzA0LWNvbnRyYWN0cy1hbmQtZ2F0ZXMubWQjMi1ob3N0ZXhlY3V0aW9uZXBpc29kZXYxYF0oLi4vMDQtY29udHJhY3RzLWFuZC1nYXRlcy5tZCMyLWhvc3RleGVjdXRpb25lcGlzb2RldjEpLCBiYXNlbGluZSBsaW5lcyA1MeKAkzEwMDsgdGhlIGV4YWN0IDE2NDUtYnl0ZSBzb3VyY2UgYm9keSBpcyBwcmVzZXJ2ZWQgYmV0d2VlbiB0aGUgYm91bmRhcnkgbWFya2VycyBiZWxvdworKipCYXNlbGluZSBzcGFuIFNIQS0yNTY6KiogYDI4ZTg2ZTJlN2UyOWI0NTgyODA4NzVmOTIxY2EzMjdhNDY5NjIxYWM5NmJkN2U1OTJhMGExNzYwMjdhYTAxMTdgCisKKzwhLS0gZXhhY3QtZXh0cmFjdGVkLWJvZHk6c3RhcnQgLS0+CisjIyAyLiBgSG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWAKKworYGBgcnVzdAorc3RydWN0IEhvc3RFeGVjdXRpb25FcGlzb2RlVjEgeworICAgIHNjaGVtYV92ZXJzaW9uOiB1MzIsICAgICAgICAgICAgICAgICAvLyBleGFjdGx5IDEKKyAgICBlcGlzb2RlX2lkOiBTdHJpbmcsCisgICAga2luZDogSG9zdEV4ZWN1dGlvbkVwaXNvZGVLaW5kVjEsCisgICAgb3JjaGVzdHJhdGlvbl9zZXNzaW9uX2lkOiBTdHJpbmcsCisgICAgb2JzZXJ2ZWRfYXV0aG9yaXR5X3JldmlzaW9uOiB1NjQsCisgICAgYmFja2VuZF9pZDogT3B0aW9uPFN0cmluZz4sCisgICAgcHJvY2Vzc19yZWY6IE9wdGlvbjxQcm9jZXNzUmVmVjE+LAorICAgIHRyYW5zcG9ydF9zdGF0dXM6IEhvc3RFeGVjdXRpb25FcGlzb2RlVHJhbnNwb3J0U3RhdHVzVjEsCisgICAgc3RhcnRlZF9hdDogVGltZXN0YW1wLAorICAgIGxhc3RfaGVhcnRiZWF0X2F0OiBPcHRpb248VGltZXN0YW1wPiwKKyAgICBlbmRlZF9hdDogT3B0aW9uPFRpbWVzdGFtcD4sCisgICAgZXhpdF9vYnNlcnZhdGlvbjogT3B0aW9uPEVwaXNvZGVFeGl0T2JzZXJ2YXRpb25WMT4sCit9CitgYGAKKworRXBpc29kZSBraW5kczoKKworYGBgdGV4dAorUmVwbEF0dGFjaGVkRXBpc29kZQorSGlkZGVuT3duZXJIZWxwZXJTdGFydEVwaXNvZGUKK0hpZGRlbk93bmVySGVscGVyQXR0YWNoRXBpc29kZQorSGlkZGVuT3duZXJIZWxwZXJSZXN1bWVPbmVUdXJuRXBpc29kZQorUnVudGltZVRvb2xib3hFcGlzb2RlCitTeW50aGV0aWNPclJlY292ZXJlZEVwaXNvZGUKK2BgYAorCitUcmFuc3BvcnQgc3RhdHVzOgorCitgYGB0ZXh0CitBdmFpbGFibGUKK1VuYXZhaWxhYmxlQnV0RHVyYWJsZUF1dGhvcml0eUV4aXN0cworVW5hdmFpbGFibGVBbmROb0F1dGhvcml0YXRpdmVSb3V0ZQorU3RhbGVPck9ycGhhbmVkCitgYGAKKworUnVsZXM6CisKKzEuIEVwaXNvZGVzIHN1Ym1pdCBvYnNlcnZhdGlvbnMgYW5kIHJlcXVlc3RlZCB0cmFuc2l0aW9ucyB0byBgSG9zdFNlc3Npb25BdXRob3JpdHlgOyB0aGV5IG5ldmVyIHdyaXRlIGR1cmFibGUgcG9zdHVyZSBkaXJlY3RseS4KKzIuIEFuIG9ic2VydmF0aW9uIHdob3NlIGBvYnNlcnZlZF9hdXRob3JpdHlfcmV2aXNpb25gIGlzIHN0YWxlIGNhbm5vdCBtdXRhdGUgYXV0aG9yaXR5LgorMy4gRXBpc29kZSBleGl0IGRvZXMgbm90IGRlbGV0ZSBzZXNzaW9uLCB3b3JrZXIsIGJpbmRpbmcsIHJlY2VpcHQsIG9yIG9ibGlnYXRpb24gdHJ1dGguCis0LiBQcml2YXRlIHRyYW5zcG9ydCBzdWNjZXNzIG1heSBhY2NlbGVyYXRlIGRlbGl2ZXJ5OyBpdCBkb2VzIG5vdCBkZWZpbmUgZHVyYWJsZSBzdWNjZXNzLgorNS4gUElELCBwcm9jZXNzL2hlbHBlciBwcmVzZW5jZSwgYWN0aXZlIGhhbmRsZXMsIHJlYWRpbmVzcywgYW5kIHByb21wdC1zdHJlYW0gc3RhdGUgYXJlIGVwaXNvZGUgb3IKKyAgIHRyYW5zcG9ydCBvYnNlcnZhdGlvbnMgb25seTsgdGhlaXIgYWJzZW5jZSBkb2VzIG5vdCBlcmFzZSBgUGFya2VkUmVzdW1hYmxlYCBhdXRob3JpdHkuCis2LiBFcGlzb2RlIGNvbnN0cnVjdGlvbiBhbmQgbGF1bmNoIGZvbGxvdyBkdXJhYmxlIHRyYW5zaXRpb24gYXBwbGljYXRpb24gYW5kIGNhbm5vdCByZXNldCBhIHBhcmtlZAorICAgc2Vzc2lvbiB0byBgQWxsb2NhdGluZ2Agb3IgYXV0aG9yaXplIGEgc3VjY2Vzc29yIHBhcnRpY2lwYW50LgorCis8IS0tIGV4YWN0LWV4dHJhY3RlZC1ib2R5OmVuZCAtLT4KYGBgCg==
```

</details>

## Preserved review answer

<details>
<summary>Initial-range review answer (rendered copy)</summary>

````text
No qualifying findings.

I found no reachable, material P1 or P2 defect introduced by this patch. The D10 landing satisfies the stated review invariants:

The extracted owner body is exactly 1,645 bytes and SHA-256 28e86e2e7e29b458280875f921ca327a469621ac96bd7e592a0a176027aa0117; I independently reconstructed the deleted baseline span from the patch and confirmed it is byte-identical to the bytes between the new owner's boundary markers. This preserves the V1 schema, field ordering, literals, fences, line wrapping, rules 1–6, negative requirements, and authority/successor boundaries.&#32;

The root retains the legacy ## 2. \HostExecutionEpisodeV1`` heading and replaces its substantive body only with a shallow link to the canonical owner. The resulting root and destination fragments are consistent with the specified GitHub-style slug behavior.&#32;

The index makes exactly one additive HostExecutionEpisodeV1 navigation change, correctly describes the authority transfer, and leaves the surrounding rows untouched.&#32;

The ledger makes exactly one D10 addition with the stated lines 51–100 provenance, source digest, compatibility-anchor representation, and rollback scope. Nothing in that rollback instruction would require alteration of the excluded owners or paths.&#32;

I also independently reproduced the complete embedded patch SHA-256 as c2f6ecea86a0fb460bde837fedeed6398f79e7c2b0bb881d97a172deb00122b9, matching the declared review boundary. The supplied validation additionally reports 1,812 Markdown links checked with zero failures and successful forward/reverse application.&#32;

VERDICT: APPROVED
````

</details>

<details>
<summary>Initial-range review answer exact bytes (base64 UTF-8)</summary>

```text
Tm8gcXVhbGlmeWluZyBmaW5kaW5ncy4KCkkgZm91bmQgbm8gcmVhY2hhYmxlLCBtYXRlcmlhbCBQMSBvciBQMiBkZWZlY3QgaW50cm9kdWNlZCBieSB0aGlzIHBhdGNoLiBUaGUgRDEwIGxhbmRpbmcgc2F0aXNmaWVzIHRoZSBzdGF0ZWQgcmV2aWV3IGludmFyaWFudHM6CgpUaGUgZXh0cmFjdGVkIG93bmVyIGJvZHkgaXMgZXhhY3RseSAxLDY0NSBieXRlcyBhbmQgU0hBLTI1NiAyOGU4NmUyZTdlMjliNDU4MjgwODc1ZjkyMWNhMzI3YTQ2OTYyMWFjOTZiZDdlNTkyYTBhMTc2MDI3YWEwMTE3OyBJIGluZGVwZW5kZW50bHkgcmVjb25zdHJ1Y3RlZCB0aGUgZGVsZXRlZCBiYXNlbGluZSBzcGFuIGZyb20gdGhlIHBhdGNoIGFuZCBjb25maXJtZWQgaXQgaXMgYnl0ZS1pZGVudGljYWwgdG8gdGhlIGJ5dGVzIGJldHdlZW4gdGhlIG5ldyBvd25lcidzIGJvdW5kYXJ5IG1hcmtlcnMuIFRoaXMgcHJlc2VydmVzIHRoZSBWMSBzY2hlbWEsIGZpZWxkIG9yZGVyaW5nLCBsaXRlcmFscywgZmVuY2VzLCBsaW5lIHdyYXBwaW5nLCBydWxlcyAx4oCTNiwgbmVnYXRpdmUgcmVxdWlyZW1lbnRzLCBhbmQgYXV0aG9yaXR5L3N1Y2Nlc3NvciBib3VuZGFyaWVzLiAKClRoZSByb290IHJldGFpbnMgdGhlIGxlZ2FjeSAjIyAyLiBcSG9zdEV4ZWN1dGlvbkVwaXNvZGVWMWBgIGhlYWRpbmcgYW5kIHJlcGxhY2VzIGl0cyBzdWJzdGFudGl2ZSBib2R5IG9ubHkgd2l0aCBhIHNoYWxsb3cgbGluayB0byB0aGUgY2Fub25pY2FsIG93bmVyLiBUaGUgcmVzdWx0aW5nIHJvb3QgYW5kIGRlc3RpbmF0aW9uIGZyYWdtZW50cyBhcmUgY29uc2lzdGVudCB3aXRoIHRoZSBzcGVjaWZpZWQgR2l0SHViLXN0eWxlIHNsdWcgYmVoYXZpb3IuIAoKVGhlIGluZGV4IG1ha2VzIGV4YWN0bHkgb25lIGFkZGl0aXZlIEhvc3RFeGVjdXRpb25FcGlzb2RlVjEgbmF2aWdhdGlvbiBjaGFuZ2UsIGNvcnJlY3RseSBkZXNjcmliZXMgdGhlIGF1dGhvcml0eSB0cmFuc2ZlciwgYW5kIGxlYXZlcyB0aGUgc3Vycm91bmRpbmcgcm93cyB1bnRvdWNoZWQuIAoKVGhlIGxlZGdlciBtYWtlcyBleGFjdGx5IG9uZSBEMTAgYWRkaXRpb24gd2l0aCB0aGUgc3RhdGVkIGxpbmVzIDUx4oCTMTAwIHByb3ZlbmFuY2UsIHNvdXJjZSBkaWdlc3QsIGNvbXBhdGliaWxpdHktYW5jaG9yIHJlcHJlc2VudGF0aW9uLCBhbmQgcm9sbGJhY2sgc2NvcGUuIE5vdGhpbmcgaW4gdGhhdCByb2xsYmFjayBpbnN0cnVjdGlvbiB3b3VsZCByZXF1aXJlIGFsdGVyYXRpb24gb2YgdGhlIGV4Y2x1ZGVkIG93bmVycyBvciBwYXRocy4gCgpJIGFsc28gaW5kZXBlbmRlbnRseSByZXByb2R1Y2VkIHRoZSBjb21wbGV0ZSBlbWJlZGRlZCBwYXRjaCBTSEEtMjU2IGFzIGMyZjZlY2VhODZhMGZiNDYwYmRlODM3ZmVkZWVkNjM5OGY3OWU3YzJiMGJiODgxZDk3YTE3MmRlYjAwMTIyYjksIG1hdGNoaW5nIHRoZSBkZWNsYXJlZCByZXZpZXcgYm91bmRhcnkuIFRoZSBzdXBwbGllZCB2YWxpZGF0aW9uIGFkZGl0aW9uYWxseSByZXBvcnRzIDEsODEyIE1hcmtkb3duIGxpbmtzIGNoZWNrZWQgd2l0aCB6ZXJvIGZhaWx1cmVzIGFuZCBzdWNjZXNzZnVsIGZvcndhcmQvcmV2ZXJzZSBhcHBsaWNhdGlvbi4gCgpWRVJESUNUOiBBUFBST1ZFRAo=
```

</details>
