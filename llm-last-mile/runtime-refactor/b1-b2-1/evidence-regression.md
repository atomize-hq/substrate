**Kind:** evidence and regression record
**Stable ID:** `B1-B2.1-family`
**Canonical for:** B0/B1/B2.1 recovery, prerequisite, closeout, proof, and RegressionMasked evidence only
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Source span:** composite of the six preserved root compatibility spans listed in the extraction ledger
**Supersedes:** canonical ownership of the extracted source bodies; source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# B1/B2.1 family evidence and regression record

## B0 closeout evidence

B0 landed the runtime-owned identity and ordering carrier without starting B1, B2.1, B3.1, C1,
or A1.2. `RuntimeFrameIdentityV1`, `RuntimeEventIdentityV1`, and
`RuntimeTerminalIdentityV1` are canonical shared types in `crates/common/src/agent_events.rs` and
are re-exported without a second representation by `crates/transport-api-types/src/lib.rs`.
`ExecuteStreamFrame` carries frame identity on every variant, semantic identity on Event and Exit,
and exact matching terminal identity on Exit. Strict runtime-frame decoding rejects missing,
empty, zero, malformed, unknown nested, conflicting duplicate, and terminal-mismatch identity.
Standalone legacy `AgentEvent` decoding remains bounded-compatible, but a runtime Event cannot
omit identity and no host decoder repairs it.

The exact producer paths are ordinary execution/deny/error/stdout/stderr/status/Exit emission and
`StreamingSink` in `crates/world-service/src/service.rs`, plus retained member launch, wrapper-event,
completion, and submitted-turn emission in `crates/world-service/src/member_runtime.rs`. Each path
constructs one `RuntimeEventStreamProducer` before the first frame. Its UUIDv7 stream ID remains
stable for the accepted stream; frame sequence starts at one and advances once for every
Start/stdout/stderr/Event/Exit/Error frame; semantic sequence starts at one and advances only for
Event and Exit; Exit is the final semantic event and its identity exactly equals terminal identity.
The producer rejects post-terminal/post-error emission. Concurrent stdout/stderr writers retain the
producer lock through channel enqueue, so assigned order cannot be reversed before observation.
Canonical NDJSON clone/re-emission preserves the complete identity and exact bytes.

Carrier-only shell compatibility changed in the allowed consumers. Identity is decoded and
forwarded unchanged. Missing identity fails closed. Error, body/decode failure, EOF, timeout, and
stream exhaustion cannot synthesize completion, and diagnostic `Failed`/`Invalidated` state without
an explicit terminal observation no longer yields `AlreadyTerminal`. This is negative B0 proof, not
B2.1 dedupe, gap detection, journaling, replay acceptance, or restart-safe observation.

Linux proof on 2026-07-13:

- `cargo test -p substrate-common --test agent_hub_event_envelope_schema`: 26 passed;
- `cargo test -p transport-api-types --lib`: 53 passed;
- `cargo test -p world-service --lib`: 120 passed, including ordinary producer, distinct-stream,
  post-terminal, retained completion, canonical replay, and concurrent stdout/stderr ordering cases;
- focused shell carrier tests passed for unchanged identity forwarding, missing-identity rejection,
  Error/EOF non-completion, malformed-Event then Exit rejection, Error then Exit rejection, explicit
  terminal observation, and corrected multi-event fixtures;
- `cargo test -p world-service --test member_runtime_world_placement_v1 -- --nocapture`: one passed,
  one pre-existing full-isolation case explicitly ignored;
- `cargo test -p world-service --test member_runtime_retained_lifecycle_v1 -- --nocapture`: four
  passed;
- `cargo check --workspace --all-targets`, warnings-denied Clippy for `substrate-common`,
  `transport-api-types`, `world-service`, and `shell`, `cargo fmt --all -- --check`, and
  `git diff --check` passed;
- live `substrate world doctor --json` reported `ok = true`, active socket/service, and a successful
  socket probe; a real `substrate --world --shim-skip -c` command completed successfully.

GitNexus detected 214 changed symbols in 12 code/test files, zero resolved execution-flow hits, and
LOW graph risk. Because schema breadth is under-resolved by that index, manual review treated the
carrier change as HIGH breadth and traced every in-scope producer and decoder. Fresh built-in
reviews found and closed concurrent stdout/stderr enqueue reordering, Error/EOF false terminality,
malformed-Event skipping, duplicate fixture sequences, and non-live-state `AlreadyTerminal`.
The final fresh review found no actionable B0 issue.

Proof limitations are explicit: no native macOS run was required or performed; producer canonical
re-emission is proven, but durable reconnect/replay acceptance belongs to B2.1; the broad shell
library run remained non-green because 171 legacy StateStore/lifecycle tests fail at the existing
authority-writer preflight, and one adjacent obligation-era fixture fails before stream construction
at the same legacy StateStore-root preflight. Neither is counted as B0 proof. The B0-owned clauses of
`RG-EVENT-01`, prerequisite clauses of `RG-SUP-01` and `RG-MSG-01`, and carrier clause of
`RG-OBS-01` are satisfied. The recovered B1 receipt core, B2.1 durable consumer core,
replay/startup activation, authority-store correction, and B1/B2.1-0 are review-clean, while
the later joint production closeout is now recorded against the bound 2026-08-03 source snapshot.
The A1.2a-WB,
A1.2a-S, B1/B2.1-R0, B3.2a, and B3.2a-WA blockers are satisfied. B3.1 is ready; C1 and A1.2b are
not. A1.2a, A1.2a-WB, and A1.2a-S are landed; the preserved broad A1.2
checkpoint remains
untouched at
`18bea80b75ad2c59c7b635851b14552e380585f2`. No seam is promoted.


## B1 production-path sequencing blocker and disposition

The preserved donor checkpoint proved proposal, exact acknowledgement, and activated-store-safe
`WorldWorkReceiptRegistry` persistence. Before recovery, the real ephemeral `Start` branch called
`persist_world_work_acceptance`; after it succeeded, the next active-work operation was
`AgentRuntimeStateStore::register_active_ephemeral_world_task`. That legacy sessions-collection
writer correctly rejects an activated HostSessionAuthority store. The failure must not be ignored,
and activated-store rejection must not be weakened.

The legacy record was also the inspect/cancel route; ephemeral cancel additionally waited on
the process-local `ActiveEphemeralTerminalWaitTracker`, and `ActiveEphemeralWorldTaskGuard::drop`
deleted the record. The retained-turn stream had no corresponding durable accepted-run observer and
remained foreground-owned through `Exit`. Deleting registration alone would therefore regress
inspect/cancel/wait behavior, while moving those mixed responsibilities into the receipt registry,
a fixed active-task side table, or generic activated-store writer would violate ownership.

The completed prerequisite repair kept the B1-3a/B1-3b receipt core → B2.1-1/2/3 supervisor branch
independent while A1.1e → A1.2a current-authority protocol → A1.2a-WB → A1.2a-S bounded
internal Start adoption → B1/B2.1-R0 → B3.2a → B3.2a-WA builds the authority/retained branch. They first join
at B1/B2.1-0 action-scoped dispatch
preparation → one joint B1/B2.1 production closeout → B3.1 →
C1 → A1.2b. The receipt/supervisor cores may be review-clean without closing B1.
`WorldWorkReceiptRegistry` retains proposal/immutable acceptance truth;
`WorldWorkExecutionSupervisor` owns active observation, journal, terminal reconciliation, restart,
and caller-drop survival; `WorldDispatchControl` consumes those truths for bounded compatibility;
HostSessionAuthority/StateStore supply only their explicitly bounded authority/read and physical
persistence capabilities. B1/B2.1-0 now routes the named B-owned production actions without the
legacy active-task writer; the later joint closeout still owns complete production integration
proof and no early return was introduced.


## B1/B2.1 `RegressionMasked` stop and ownership disposition

The post-review production-ingress audit selected **Case B — canonical production input is
missing** because A1.1e provided only an exact reader whose `resolve_exact` required an
already-current `SessionNamespaceRecordV1::Authority`; at that audit point no production path
created the authority. A1.2a and A1.2a-S now boundedly satisfy that creator/adopter gap for the
ordinary internal greenfield host path. The prepared dispatcher still consumes legacy
session/caller/target record shapes absent from the exact retained-target read, and
`live_retained_worker_count` drives steering even though
`retained_worker_refs` carry no live/terminal state. Neither a legacy `authoritative_live`
composite nor a count of all refs is an acceptable replacement.

The audit therefore rejects the earlier ownership claim that
`prepare_orchestrator_world_dispatch` belongs to A1.2. A1.2a owns only greenfield Start
issuance/application and exact read, A1.2a-S owns only bounded internal Start adoption, and the
prepared dispatcher remains a B-owned consumer after those and the retained prerequisites exist.

The corrected acyclic sequence keeps the preserved **B1/B2.1 core** branch independent while
**A1.1e → A1.2a current-authority protocol → A1.2a-WB Host/world-binding correction →
A1.2a-S bounded internal Start adoption →
B1/B2.1-R0 canonical retained target protocol → B3.2a retained creation/admission →
B3.2a-WA exact bound-world ownership adoption** builds the
authority/retained branch. They first join at **B1/B2.1-0 action-scoped dispatch preparation** →
joint closeout → B3.1 → C1 → **A1.2b
successor/post-turn completion**. A1.2a owns only the strict greenfield V1-to-V2 root upgrade,
followed by greenfield Start reservation/issuance/claim, single application/initial authority
birth, crash-safe exact retry, and a typed read result joining the applied caller descriptor and
bound home/store to exact authority. A semantic/non-greenfield V1 root cannot be converted.
A1.2a-S alone adopts that result on the ordinary internal host bootstrap: the identity-free proposal
is applied after the real dormant-launch adapter has the optional world binding, and only the
applied result constructs the existing fully materialized `PreparedAgentRuntime`. It carries the
bound capability to the live toolbox and suppresses activated-store legacy
session/participant/snapshot writes while leaving startup ownership Pending. Fork/member prepared
runtime paths remain unchanged. It does not adopt hidden-owner plans, public
Attach/Resume, startup-result reconciliation, or post-turn behavior.
B1/B2.1-R0 supplies only the durable canonical retained-target registration protocol; by design it
has no production ingress and cannot satisfy full-dispatcher proof alone. Its complete immutable
descriptor/resume/worker graph is published only after an HSA-owned issuer-request reservation
validates the caller-fixed participant plan and fixes every remaining replay identity/commitment,
then becomes authority only through the sole HSA-owned
atomic lineage/ref mutation, request completion, and non-transition proof. Crash and lost-response
retry rejoin that index; no later messaging or lifecycle semantics are introduced. B3.2a is the
separate RetainedWorkerRuntime-owned production bridge: both real Spawn adapters first atomically
fingerprint the complete validated request/authority/runtime/policy plan under the separate
RetainedWorkerRuntime admission key, whose crash-stable lifetime is independent of HSA keys; count all durable
nonterminal admission slots, enforce the cap, and persist fixed participant/bootstrap-run identity.
Only one durable per-session registration head fixes current authority. Queued `SlotReserved`
records with no current head are valid and remain live for cap accounting. Reconciliation of the
completed current head advances only that record and releases the head; it never automatically
promotes another slot. Only exact re-presentation of the complete canonical request for the
lowest-sequence queued slot can acquire the next head after re-verifying its stored HMAC and the
complete admission-to-current R0-only ancestry. The registry stores no request/prompt/payload
preimage; digest equality alone cannot promote. Changed bytes conflict without mutation, later
slots cannot overtake, and caller/PID/helper/socket/endpoint/timeout/EOF/liveness/observer state
cannot acquire, replace, renew, or steal. An abandoned earliest slot conservatively blocks later
slots until its exact request retries. Remaining B3.2 owns exact durable admission-state resolution
and restart reconciliation, while B4 owns the user/tool-facing inspect/cancel verb and distinct
outcomes; B3.2a adds neither protocol. The bridge passes the head plan to R0, exact-joins R0 before
transport, and carries one transport-neutral typed equality proof through the direct dispatcher and
live internal-toolbox adapter into the real transport-api `Service::execute_stream` member branch
and `MemberRuntimeManager::launch` validation. The live adapter cannot allocate a retry-local
participant or invoke activated-store legacy writers. Exact Registered
truth makes the target routable; only exact B0 terminal truth removes it from the live count, while
every ambiguous interruption stays nonterminal and counted. Spawn policy/outcome/events remain
unchanged.

The first bounded Linux authority-managed live Spawn then exposed `RG-WORLD-ADOPT-01`. The exact HSA
binding and typed launch proof named the already-running generic REPL world; world-service inferred
shared-owner `AttachOrCreate`, created a different world, and placement validation rejected the
mismatch before member creation. This is correct fail-closed behavior but blocks product proof. The
durable admission remained `InterruptedNonterminal`, and the B3.2a-WA correction was required not
to rewrite it or report launch success. The docs-first correction assigns only physical ownership metadata to world-service/Linux
backend: after existing strict `Some(exact proof)` validation, adopt the exact HSA-bound generic
world as the same-ID/generation shared owner under trusted crash-durable publication, then launch.
Exact retry joins; conflicts fail without mutation; no alternate world, HSA rebinding, world-api
field/schema, request preimage, liveness authority, compatibility-path change, recovery/resend, or
platform claim is permitted. Adoption does not prove transport submission, Registered, routability,
terminal success, or member launch, and no seam is promoted.

The B3.2a typed-carrier compatibility allowlist includes one exact mechanical correction because
`#[serde(default)]` does not initialize Rust struct literals. RunWorldTask and ForkWorldWorker may
set the new optional proof field to explicit `None` only in their existing transport builders, and
macOS `convert_member_dispatch` may set it to explicit `None` only while preserving its existing
world-api conversion. Those paths gain no retained-worker launch authority and retain every prior
identity, lineage, backend, protocol, run, session, world, prompt, runtime, route, policy,
placement, lifecycle, error, and outcome behavior. No world-api edit or macOS authority-managed
Spawn adoption is authorized. Authority-managed B3.2a Spawn alone requires `Some(exact proof)` and
fails closed before process creation on missing, malformed, or mismatched proof; no default helper,
side table, environment carrier, alternate route, or hidden proof synthesis is permitted. This
mechanical correction closes only the compiler-required literal scope and does not complete
B3.2a.

The carrier widening also requires exactly seven mechanical internal initialization sites. In
`orchestrator_world_dispatch.rs`, `fork_world_worker` and
`continue_world_worker_fork_command_bootstrap_after_delivery` pass only explicit `None` to the
widened stream helper. In `async_repl.rs`, `apply_greenfield_host_start_from_authority`,
`prepare_hidden_owner_helper_runtime`,
`start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx`,
`prepare_fork_child_runtime_startup_for_descriptor`, and
`prepare_member_runtime_startup_for_descriptor` only initialize, destructure, or preserve the new
optional retained-worker authority fields as `None`. This is compiler-required argument/field
plumbing, not authority adoption: host Start keeps its established host-session authority,
hidden-helper behavior is unchanged, fork stays outside B3.2a adoption, and legacy member
preparation stays distinct from `prepare_member_runtime_startup_from_authority_registration`.
Only the latter may carry `Some(exact proof/admission context)` for authority-managed retained
Spawn. Missing authority on that managed path fails closed with no legacy fallback. No ownership,
policy, lifecycle, transport, identity, lineage, error, outcome, or runtime behavior changes, and
no further symbol or structural scope is authorized. That mechanical authorization did not itself
complete B3.2a; the recorded B3.2a/B3.2a-WA result below now does.

B1/B2.1-0 removes the
missing live-retained field and legacy session/caller inputs from B-owned RunWorldTask, ordinary
retained ContinueWorldWorker, and ephemeral accepted-task Inspect/Cancel/Wait preparation while
leaving `WorkerContinueForkCommand`, retained Inspect/Cancel/Stop, fork, the already-landed B3.2a
spawn creation/admission bridge, the intervening B3.2a-WA exact same-world physical-ownership
bridge, and remaining retained lifecycle steering unchanged and unpromoted. A1.2b first owns the separately reviewed strict
V2-to-V3 root/intent/state extension after B1/B3.1/C1 types are available, then retains
Attach/Resume, startup/post-turn reconciliation, obligation-cut consumption, release, retry, and
optional world-work correlation after C1. Its startup resolution
may join the original Start application revision to current authority only through the unique
contiguous R0 registration-proof chain; acceptance leaves current authority unchanged and terminal
reconciliation preserves every R0-added lineage member/ref. Arbitrary stale revision still fails.
The broad preserved
A1.2 checkpoint is not restored or modified.

The current preserved runtime WIP does not satisfy the differential gate. Donor diff and review
evidence show fourteen historical tests whose production-level proof was masked: the eleven
previously identified dispatcher/inspect/cancel/wait tests plus two guard-drop tests and one
tool-invocation test. The exact inventory is:

1. `dispatch_contract_inspect_world_worker_ephemeral_returns_authoritative_snapshot_without_mutation`
2. `dispatch_contract_inspect_world_worker_ephemeral_resolves_exact_session_binding_when_task_run_id_is_reused`
3. `dispatch_contract_inspect_world_worker_ephemeral_fails_closed_for_unknown_task_run_id`
4. `dispatch_contract_inspect_world_worker_ephemeral_fails_closed_for_stale_linkage`
5. `dispatch_contract_inspect_world_worker_ephemeral_fails_closed_for_backend_mismatch`
6. `dispatch_contract_inspect_world_worker_ephemeral_fails_closed_for_world_binding_mismatch`
7. `dispatch_contract_inspect_world_worker_ephemeral_teardown_removes_routability`
8. `dispatch_contract_cancel_world_work_ephemeral_routes_exact_active_task_over_execute_cancel`
9. `active_ephemeral_terminal_wait_allows_multiple_waiters_to_observe_same_terminal_truth`
10. `dispatch_contract_cancel_world_work_ephemeral_retry_reuses_shared_terminal_truth`
11. `dispatch_contract_cancel_world_work_ephemeral_fails_closed_when_execute_cancel_is_not_delivered`
12. `active_ephemeral_terminal_wait_registration_guard_releases_non_happy_path_registrations`
13. `active_ephemeral_terminal_truth_guard_publishes_failed_terminal_truth_on_drop_after_start`
14. `dispatch_contract_adapter_follow_up_resolution_uses_authoritative_active_task_state`

All fourteen retain these exact historical names and their original production-level assertions.
Tests 1–11 must re-enter the full dispatcher. Tests 12–13 must prove accepted and supervised truth
survives actual guard/waiter destruction; aborting a helper waiter or calling a lower-level owner
directly is not equivalent. Test 14 must exercise the real host tool-invocation-to-dispatch route;
establish current authority through production HostSessionAuthority APIs, acceptance through
`WorldWorkReceiptRegistry`, and observation/terminal truth through
`WorldWorkExecutionSupervisor`; and prove no legacy active-task writer or resolver is used. Direct
receipt/supervisor authority resolution followed by request construction is not equivalent.
Direct resolver, transport, receipt, supervisor, or helper substitutions remain
`RegressionMasked`, even when the lower-level owner is correct.

The bounded control-pack correction authorizes only
`tool_invocation_contract.rs::resolve_follow_up_dispatch_authority_v1`'s active-task branch,
mechanical imports for that branch, mechanical relocation of the existing shared pre-`match`
legacy caller/world resolution unchanged into the retained-worker branch, the unchanged-name
historical test 14, and focused negative tests. The active branch must prove exact task-ID reuse across sessions, unknown task, stale
receipt/supervisor linkage, caller/backend mismatch, world ID/generation mismatch, acceptance
without a supervisor claim, unresolved producer replay, and exact terminal truth. The retained
branch must prove unchanged compatibility behavior, target selection, and error categories. The
function signature and callers remain unchanged. The approved GitNexus HIGH impact is bounded to
12 direct callers, four affected execution processes, and 14 impacted symbols; any broader impact
is a new stop. These additions sharpen `RG-RECEIPT-03`, `RG-SUP-01`, `RG-SUP-02`, `RG-OBS-01`, and
`RG-DIFF-01` but resolve none of them.

Test renaming, replacement, newly ignored status, assertion deletion or weakening, or lower-level
substitution cannot count as `FailToPass` proof. Within the bounded closeout, only the named
ephemeral accepted-task controls are eligible for failure-to-pass remediation; retained
Inspect/Cancel/Stop may only preserve their exact historical failure and normalized signature until
their later lifecycle owner lands. The historical duplicate-registration test may remain a
legitimate isolated fixture correction only if its production semantics and exact assertions are
unchanged; it does not count as receipt/supervisor remediation merely because the fixture becomes
runnable.

This disposition sharpens the open gates without closing them:

- `RG-RECEIPT-03` requires full-dispatcher exact-session/world/cross-session reuse proof for the
  named ephemeral accepted-task controls over immutable acceptance and supervisor truth, including
  actual caller/waiter/guard drop survival. A direct lower-level owner call cannot stand in for the
  dropped production guard or waiter. It does not promote retained control lifecycle.
- `RG-SUP-01` requires the accepted production path and full compatibility dispatcher to use the
  exact claim/journal without a legacy active-task registration or resolver fallback for the
  action-scoped closeout paths. The tool adapter may read but cannot create, mutate, terminalize,
  or substitute for that supervisor truth.
- `RG-SUP-02` requires restart, actual waiter/guard-drop, terminal-closeout, and
  acceptance-retention proof through the named ephemeral production integration path; replay
  unavailability after world-service restart remains durably unresolved rather than terminalized,
  and retained closeout remains later-owned. The active-task tool projection must preserve the
  distinct unresolved-producer state.
- `RG-OBS-01` requires the real tool-to-dispatch and prepared-dispatch paths to join the A1.2a-S production-bound current-authority read,
  R0's HSA-proven retained target where applicable, B1 acceptance, and B2.1 observation identity;
  episode/liveness composites and compatibility
  projections are not observability authority. Test 14 must reach that join through the real host
  tool route; a direct call to the corrected resolver is not production proof.
- `RG-DIFF-01` requires all fourteen historical names and original production-level assertions.
  The eleven dispatcher tests re-enter the full dispatcher, the two guard tests exercise actual
  drop, and the tool test uses the real tool-to-dispatch route. Retained control cases may remain
  `FailToSameFailure`, but direct resolver/transport/receipt/supervisor substitutions remain
  `RegressionMasked` even when they exercise individually correct owner APIs. Test 14 additionally
  retains its distinct unknown/stale/backend/world/nonterminal/terminal/unresolved outcomes and
  cannot weaken or replace them with a generic result.

Only a fresh broad differential classified `ExpectedBaselineResolution` after that real-path proof
may close the joint packet. `PassToFail`, `FailToChangedFailure`, `Removed`,
`RenamedOrSubstituted`, `NewFail`, and `NewIgnored` must all remain zero, and every retained failure
must preserve its normalized signature.

Historical A1.2a-WB authorization record: the preceding docs-only correction recorded landed
A1.2a and authorized only the subsequently reviewed A1.2a-WB implementation in `transition.rs`,
`transition_tests.rs`, and the exact `facade.rs::HostSessionAuthority::resolve_current_exact`
boundary. At that point it did not authorize any other facade behavior or A1.2a-S and required WB
to be implemented from its bounded contract rather than by restoring the broad A1.2 checkpoint.
That authorization state is superseded by the recorded review-clean WB and A1.2a-S closeout above.
B1/B2.1-R0 is review-clean through `bb3eefba`; B3.2a plus B3.2a-WA are independently review-clean
through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`. At that point B1/B2.1-0 had not begun, and its
implementation remained unauthorized until the docs-only control-pack correction was independently
review-clean. That historical authorization condition is now satisfied: the correction was
published as `3741cadd`, the bounded runtime result is review-clean through `83101dcb`, and later
packets remain unauthorized.


## B1/B2.1-R0 recorded result

B1/B2.1-R0 started from exact source commit
`f630835ab3715f957b0f49697e104c40d591305e`; preservation branch
`feat/preserve-b1-b2-1-r0-f630835a` was pushed at that exact commit before editing. R0-1 is
`ace6cebd`, `e3768c3f`, `6154dc97`, and `6ec5eb78`; R0-2 is `5582039a`; R0-3 is
`3f4464b7`; R0-4 is `23993184` plus review remediation `0d5225f3`; final integration remediation
is `bb3eefba`. The source range changes only the eight-file subset of the authorized R0 allowlist:
`agent_runtime/{mod.rs,retained_worker_runtime.rs}` plus
`host_session_authority/{facade.rs,store.rs,store_schema.rs,store_tests.rs}` and
`host_session_authority/store/platform/{layout.rs,object_persistence.rs}`.

The component durably reserves the domain-separated retained-registration request before any
object write, fixes the caller-supplied participant and every registration/object identity,
commitment, expected authority value, policy, world, backend, protocol, and timestamp, publishes
only the exact immutable descriptor/resume/worker graph, and atomically applies one non-transition
authority link. Exact resolution proves store/session/lineage/ref/object/policy/world joins and one
unique contiguous Start-to-R0 ancestry whose highest proof revision equals current authority.
Crash/restart windows, lost response, deterministic Reserved-to-Applied publication interleaving,
and identical/conflicting two-process contention converge or fail closed with no unauthorized root
or object mutation. An Applied retry with a different valid authority-store ID is rejected without
mutation. Startup ownership stays Pending and retains its original expected revision.

Final Linux component proof is retained runtime `25 passed / 0 failed`, full
HostSessionAuthority `132 / 0`, explicit strict V1/V2 decoding, object reachability/index/orphan,
key lifecycle/rotation, crash/restart, and cross-process checks green, shell all-target Clippy with
warnings denied green, and `cargo check --workspace --all-targets`, formatting, and diff checks
green. The final serial shell wall is `923 passed / 161 failed / 0 ignored` against exact starting
`898 / 161 / 0`: all 25 additions are R0-only passing tests; `PassToFail`, `NewFail`,
`FailToPass`, removed/renamed/substituted/weakened tests, and ignored-test changes are zero. All 161
inherited failure names and normalized bodies are identical after replacing only generated
`aos_<ID>` values. There is no unexplained production-path `FailToPass`, so `RegressionMasked` did
not trigger.

Fresh read-only packet reviewers `/root/r0_1_clean_review`, `/root/r0_2_review`,
`/root/r0_3_review`, and `/root/r0_4_rereview` returned CLEAN. The first R0-4 reviewer found the
Reserved-to-Applied publication race fixed by `0d5225f3`; the first final integration reviewer
found the Applied retry store-ID gap fixed by `bb3eefba`. Fresh reviewer
`/root/r0_final_integration_rereview` returned final CLEAN. This is component proof only: R0 has no
production ingress caller, supplies no e2e or live doctor/smoke proof, and has no native macOS
claim. No seam was promoted by R0. B3.2a plus B3.2a-WA are now independently review-clean through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda` as recorded below.


## B3.2a and B3.2a-WA recorded result

B3.2a began from source commit `69b6cddc61578437d1cb1af95055fdb7e0b9c776`. The docs-first
corrections are queued promotion `8726dface3e314c2e24aa95305cd69f01c8bb389`, abandoned-admission
ownership `ba6253483ccabec151ce6077d1df1ce7067ac42b`, carrier compatibility literals
`e1a248589a0d2cee1cedec33227f8b070515c664`, exact seven-symbol carrier plumbing
`47e9ea0daa7982ac3fc3d4c85384af1b0490d563`, and exact-bound-world adoption
`aa38c66badb46ee8d17855432438de1b66accdce`. The final prerequisite replay preserved the original
20-commit runtime range at `feat/preserve-b3-2a-wa-runtime-1d2543c8` pointing to
`1d2543c8ded723f22b1612bf5cb58be2d86fcc12`, then replayed it in exact order as
`73bdd5640588c685c67e73cc4c9425e44c6213da` through
`106f9c85c60ee1649b5c7ae88b47338da0402d5e`. The preserved and replayed aggregate ordinary patch
SHA-256 is `ed417fdca7c26da46b6a51ba26af3e8939f997e9632eec7608931fe7a514107a`; the binary/full-index
patch SHA-256 is `6c574c1655c1668537a159a01dd1dac120cad666ea9924a0206a05ac4d3ee671`.
No preservation commit was merged.

The replayed range supplies the RetainedWorkerRuntime-owned admission key and registry, exact
complete-request HMAC fingerprint, conservative live cap, fixed participant/bootstrap run,
serialized registration head, exact R0 join, and unique durable transport claim. Reconciliation
advances and releases only the current head. A queued `SlotReserved` record with no head is valid;
only complete exact re-presentation of the earliest request may acquire the head. Changed bytes,
digest-only input, later-slot retry, PID/caller/helper/socket/endpoint/timeout/EOF/liveness state,
and observer loss cannot promote, replace, renew, or steal. Request/prompt/payload preimages are
not persisted. Publication-boundary, restart, contention, exact/conflicting retry, no-steal, cap,
and complete fingerprint mismatch tests are green. `RG-ADMISSION-01` deliberately remains open for
remaining B3.2/B4; B3.2a adds no cancellation, abandonment, reclamation, or recovery protocol.

The typed carrier requires `Some(exact RetainedWorkerLaunchAuthorityProofV1)` on both
authority-managed Spawn adapters and strict-validates it in the real transport-api
`Service::execute_stream` member branch and again at `MemberRuntimeManager::launch`. RunWorldTask,
ForkWorldWorker, fork continuation, Host Start, hidden helper, legacy member preparation, and macOS
compatibility conversion carry explicit `None` and gain no retained-worker authority. The sole
durable claimant launches transport; an exact joined claim does not reproject request bytes or
resend. Exact Registered truth makes the admission routable, exact B0 terminal truth terminalizes
it, and interruption/ambiguity remains nonterminal and counted. Authority-managed readiness now
emits the exact Registered session-handle event before any bounded buffered pre-Registered status
event; compatibility `None` retains its old ordering. WA commit
`11933b310c6424fc01e5aea18121f39ddf807f6a` implements exact same-world ownership publication, and
readiness remediation `d0a70727c2bec2b2d6fe0754ea469c4682684dda` closes the final live-path
ordering defect.

The Linux proof wall is green: complete HostSessionAuthority `132/132`, complete
RetainedWorkerRuntime `81/81`, transport-api-types `54/54`, world `113 passed / 1 pre-existing
ignored`, world-service `126/126` library tests plus all applicable integration targets, and the
authority-managed readiness integration `5/5`. The direct internal-toolbox production-adapter test
is green. Workspace all-target check, focused Clippy with warnings denied, formatting, and
`git diff --check` pass. The serial shell differential is exact baseline
`923 passed / 161 failed / 0 ignored` to current `981 / 160 / 0`: 57 added passing shell tests,
zero removed/renamed/substituted/weakened tests, zero `PassToFail`, zero `NewFail`, and one
causally proven `FailToPass`,
`repl::async_repl::tests::orchestrator_world_dispatch_surface_spawns_authoritative_member_runtime`.
All 160 retained failures preserve their normalized first-panic signatures. The additional
readiness regression is a world-service integration test and does not alter that shell inventory.

The final bounded Linux product proof used the reviewed installed world-service binary SHA-256
`9f02fc27fa455c53ed8ab03a49125cb24b7aac38f528b8a8b3326d68d96dcf63`. World doctor returned
`ok: true`; ordinary world execution returned `B3_2A_WA_ORDINARY_REVIEWED_OK`. The production
internal-toolbox Spawn returned one successful receipt for session
`aos_27f14606e443f73927d2f892beabc704`, participant
`rwp_dd8856889e050e3b6d78795c2f1e8ad8`, world
`wld_019f6a96-bdf1-7db1-a5cc-58cbd0f370c9`, generation `0`, and launch span
`spn_019f6a97-ee72-7b92-a30d-7cdc98e7144c`. Backend metadata records that same world as the sole
active shared owner for the same session; no alternate world exists. The unique prompt marker is
absent from authority/admission objects, trace, backend metadata, workspace, service storage,
errors, response, and system journal. Clean REPL shutdown produced exact terminal truth for this
successful smoke; the earlier failed attempt remains separately preserved
`InterruptedNonterminal` and was not rewritten.

Fresh read-only reviewers `/root/b3_2a_3_review_3`, `/root/b3_2a_wa_docs_final_review_5`,
`/root/b3_2a_wa_runtime_review_3`, `/root/b3_2a_readiness_review_5`, and
`/root/b3_2a_final_integration_review_6` returned CLEAN for their final assigned ranges. The final
integration reviewer inspected the complete `aa38c66b..d0a70727` runtime range plus differential
and live artifacts and authorized documentation closeout. B3.2a and B3.2a-WA are review-clean on
Linux. No seam is promoted, no native macOS or Windows proof is claimed, remaining B3.2/B4 work is
unchanged. The later B1/B2.1-0 result is recorded below.


## B1/B2.1 core recovery and B1/B2.1-0 recorded result

The packet began from published source `8aaa1214c07885e66778e9ba045aa6c9472c9768` after the
B2.1 replay/startup contract correction. The active-task adapter authorization landed first as the
docs-only commit `3741caddfa17f8ccb3b6f272731b5df0614a7405`. Runtime preservation remained on
`feat/preserve-b1-b21-0-crossdoc-20260716T175740Z` at
`8234f056b2bb65d5324f8269999d87ca86bbfc1d` and was never merged or pushed as incomplete source.
Because the named checkpoint and cross-document verification skills were unavailable, the packet
used explicit substitutes: remote preservation plus commit/tree/file fingerprints, old-to-new
replay mapping, and a six-file structure/link/table/gate/sequencing/stale-status validation.

Donor `feat/b1-b21-post-review-preservation-20260714T133418Z` commit
`2f5a73168832127e4ec25fa0385246d932552566` with parent
`ba802c76383b81fd015d88b8ac31b2d588d9a2c8` was read-only salvage evidence, not a merge source.
Its B1 receipt hunks became `6436289fd9dd55ea516b96ef3299e4055d1ea718`; B2.1-1/2 supervisor
hunks became `c519024bd91b6ca6e332d0b8881f7d13ded940e0`; B2.1-3 replay/startup hunks became
`de727091a39c884044179a89135df3db5d566778`; and the current-source versioned authority-store
binding correction is `717579b0744154d343985ad439fb8756158f376f`. Historical direct-resolver or
transport substitutions, weakened assertions, test-only authority writers, obsolete A1/R0/B3
hunks, and unrelated fixture/platform changes were excluded. The four recovered commits replayed
from `0f641cde`, `051699d6`, `67476ec5`, and `9cd7f6be` to `6436289f`, `c519024b`, `de727091`,
and `717579b0` with identical aggregate binary patch fingerprint and unchanged per-file content.

B1/B2.1-0 is `83101dcbcc750e6e8fb8979bea19f1f777792188`. Its production changes are confined to
`orchestrator_world_dispatch.rs`, `agent_runtime/state_store.rs`, and only the approved active-task
branch plus colocated proof in `agent_runtime/tool_invocation_contract.rs`. The prepared B-owned
view is used only by RunWorldTask, ordinary retained ContinueWorldWorker, and ephemeral accepted-
task Inspect/Cancel/Wait. It exact-joins current HostSessionAuthority, immutable acceptance,
supervisor claim/cursor/terminal state, store/session/caller/backend/world/task identity, and R0+
B3.2a target/routability where applicable. It requires exact authority revision and an accepted
supervisor cursor, performs no legacy active-task or obligation write, and preserves blocking
foreground behavior. Continue-fork, retained Inspect/Cancel/Stop, fork, and the B3.2a/B3.2a-WA
Spawn path retain their compatibility behavior.

The approved GitNexus impact for `resolve_follow_up_dispatch_authority_v1` remained HIGH with 12
direct callers, 14 impacted symbols, and four execution-process roots: the parked/resumable
retained-worker continue case, outside-lineage rejection, successor fork after owner exit, and
retained fork/cancel successor-authority case. Final change detection expanded to 18 downstream
flows but found no new process family; independent review confirmed that adjacent unchanged
symbols in the large colocated diff caused the broader mapping.

Fresh Linux shell proof moved from `981 passed / 160 failed / 0 ignored` across 1141 tests to
`1054 passed / 149 failed / 0 ignored` across 1203 tests. The exact fourteen historical names and
production routes remain: eleven dispatcher tests enter the full dispatcher, guard/waiter tests
exercise real drop, and the tool test enters the real tool-to-dispatch path. All eleven
`FailToPass` transitions have an exact real-route cause; `PassToFail`, `FailToChangedFailure`,
`Removed`, `RenamedOrSubstituted`, `NewFail`, and `NewIgnored` are zero. All 149 retained normalized
failure signatures are byte-identical after normalization. Receipt, supervisor, replay/client,
world-service, full-dispatch action, active-task negative, retained-runtime, and Spawn regressions
are green; workspace all-target check, focused warnings-denied Clippy, formatting, and diff checks
are green. No joint doctor/live-smoke wall is claimed.

Fresh read-only docs reviewers `/root/docs_authority_scope_clean_review` and
`/root/docs_test_impact_clean_review` returned CLEAN for the authorization amendment. Initial
implementation reviewer `/root/implementation_final_review` found stale-revision acceptance,
missing durable-start-cursor validation, and a legacy obligation-writer/panic path; all three were
fixed with focused red/green proof. Fresh reviewers `/root/implementation_impact_review` and
`/root/implementation_remediation_final_review` then returned CLEAN for the approved HIGH-impact
adapter and complete recovered-core/B1/B2.1-0 range.

B1 receipt core recovered/review-clean: **yes**. B2.1 supervisor core recovered/review-clean:
**yes**. B1/B2.1-0 review-clean: **yes**. B1/B2.1 joint production integration closeout:
**complete**. B3.1 complete: **yes**. C1 complete: **yes**. Seam promotions: **none**. Within the
retained-turn follow-on corridor, A1.2b is now the next architectural packet. At the
B1/B2.1-0 closeout, the repository's exact next packet was A1.1d-5R2-1 — Host context construction
and Unix dev propagation; after review-clean R2-2 Routes A–E, the failed integration closeout, and
the completed exact harness closeout, the next packet at that historical checkpoint was
**A1.1d-5R2-2F — Authenticated world-deps and truthful doctor composition**, followed by renewed R2-2
integration closeout, R2-3, R2-4, and R3. The preserved
[`A1.1d-5R2-2F final evidence and decision ledger`](../a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-final-evidence-and-decision-ledger)
supersedes that next-task status. Only the joint closeout's Linux
product-smoke portion waits for those remediations and their required smoke, and its receipt and
supervisor semantics are not reopened.
