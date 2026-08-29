# A1.3-P1 public-adoption A13P1-P2-008 correction — verification evidence

**Captured:** 2026-08-29 on Linux

## Repository and frozen successor-review subject

```text
branch: feat/runtime-refactor-a1-3-p1
baseline commit: 54cf865432fbf7fb3344ddbe83843b81c15c7152
baseline tree: 49bd951f59291ed11186ecd6c48615d97564f9ab
baseline remote tip: 54cf865432fbf7fb3344ddbe83843b81c15c7152
pre-correction candidate: sha256:986cb6ed03475c41427131dcb82ca46ac2ab820a8f4603c7eb949c60d3d9e2fa
successor-review subject: sha256:4f834a7d02e0c1696d0f1adf99fefe472f8e5d7e37c33eba1ed81b51a8c82c91
```

The successor-review fingerprint is the SHA-256 of this sorted product manifest. Review-control
evidence is excluded so that recording the independent result does not mutate the implementation
subject.

```text
BASE 54cf865432fbf7fb3344ddbe83843b81c15c7152
100644 a6d8423b27b6ecc92c2870bc57dc65d442c1be1b crates/shell/src/execution/agent_runtime/control.rs
100644 e50957422bbedc18336de9305a35104f21784f02 crates/shell/src/execution/agents_cmd.rs
100644 8ddf050b740952db400ccd1be3921e7aa5f29ce7 crates/shell/src/execution/orchestrator_world_dispatch.rs
100644 c83a5193ffa45d5b160a6f7d1cb808ac00965cce crates/shell/src/repl/async_repl.rs
100644 5d99f02b4c1de03637dda48e7784b6e750d4152e crates/shell/tests/agent_public_control_surface_v1.rs
```

Subject diff summary:

```text
5 files changed, 4783 insertions(+), 137 deletions(-)
```

The only product blob changed from the preserved `986cb6ed...` candidate is
`crates/shell/src/execution/agents_cmd.rs`. The other four product blobs remain byte-identical to
that candidate.

## A13P1-P2-008 correction

The session/request lock remains only a concurrent-execution guard. Before either public Attach or
ResumeOneTurn can issue an HSA transition, the implementation now durably binds the exact request
commitment and its intent, issuer request, participant, lease, run, claim, and claimant-attempt
identities. Every lock acquisition reloads and validates that durable identity, including an
uncontended retry after the previous launcher has dropped the lock.

The existing HSA store is inspected before issuance. An exact persisted Issued or Claimed intent is
reconstructed through the existing HSA exact-join API and then resumed through claim/apply. An exact
Applied intent joins its committed launch plan. Conflicting request state, substituted identity,
or a different nonterminal public successor for the same session and mode fails closed. A new
transition is issued only when no durable transition exists for the exact request. No liveness,
PID, timeout, EOF, readiness, event-order, or lock-contention inference is used as recovery
authority. Acknowledgement occurs only after the committed/joined result is rendered or returned.

No new ledger, supervisor, coordinator, authority store, or retry subsystem was added. The existing
HSA persistence and exact issue/claim/apply/join mechanisms remain authoritative.

## Deterministic crash-recovery proof

```text
$ cargo test -p shell --lib execution::agents_cmd::tests::public_attach_fault_after_issue_or_claim_reuses_exact_persisted_transition -- --exact
1 passed; 0 failed
$ cargo test -p shell --lib execution::agents_cmd::tests::public_resume_one_turn_fault_after_issue_or_claim_reuses_exact_persisted_transition -- --exact
1 passed; 0 failed
$ cargo test -p shell --lib execution::agents_cmd::tests::concurrent_public_successor_requests_reuse_one_durable_identity_before_issuance -- --exact
1 passed; 0 failed
$ cargo test -p shell --lib execution::agents_cmd::tests::uncontended_public_successor_retry_reuses_persisted_identity_after_launcher_drop -- --exact
1 passed; 0 failed
$ cargo test -p shell --lib execution::agents_cmd::tests::acknowledged_public_successor_exact_joins_then_distinct_request_rotates_identity -- --exact
1 passed; 0 failed
```

Each public-mode crash test deterministically injects failure after issue/before claim and after
claim/before apply, drops the actual in-memory request-lock guard, and retries without contention.
For both Attach and ResumeOneTurn it proves:

- the exact persisted transition, intent, issuer request, session, participant, lease, run, claim,
  claimant attempt, expected revision, launch-plan, handle, and evidence identities are reused;
- exactly one intent exists for the request and no successor transition is issued;
- the original Issued or Claimed transition reaches Applied;
- an Applied and then acknowledged transition exact-joins the same committed result; and
- a mismatched request commitment and a substituted persisted transition identity fail closed
  without changing HSA state.

The fault hook is test-only thread-local state; there is no production environment or process-exit
fault switch.

## Passing preserved A1.3-P1 gates

```text
$ cargo check -p shell --lib
exit 0
$ cargo test -p shell --test agent_public_control_surface_v1 --no-run
exit 0
$ cargo build -p substrate --bin substrate --bin substrate-shim
exit 0
$ cargo test -p shell --lib execution::agent_runtime::host_session_authority::transition_tests:: -- --nocapture
49 passed; 0 failed
$ cargo test -p shell --lib authority_successor_launch_guard_survives_parent_drop_across_exec -- --nocapture
1 passed; 0 failed
$ cargo test -p shell --lib hidden_owner_helper_start_ -- --nocapture
2 passed; 0 failed
$ cargo test -p shell --lib start_continuity_ -- --nocapture
3 passed; 0 failed
$ cargo test -p shell --lib obligation_ledger -- --nocapture
20 passed; 0 failed
$ cargo test -p shell --lib build_attach_launch_plan_ -- --nocapture
2 passed; 0 failed
$ cargo test -p shell --lib close_prepared_internal_continue_ -- --nocapture
5 passed; 0 failed
$ cargo test -p shell --lib accepted_retained_typed_event_validation_precedes_generic_journaling -- --nocapture
1 passed; 0 failed
$ cargo test -p shell --lib b21_retained_snapshot_is_pending_until_exact_terminal_cut_then_completes -- --nocapture
1 passed; 0 failed
$ cargo test -p shell --lib hidden_owner_helper_start_records_ambiguous_handoff_as_indeterminate_without_replay -- --nocapture
1 passed; 0 failed
$ cargo fmt --all -- --check
exit 0
$ git diff --check
exit 0
```

These retained gates cover public Attach/ResumeOneTurn routing, authenticated actor evidence,
retained-turn correlation and terminal cut, stable concurrent request identity, Start continuity,
PromptSubmissionIndeterminate zero replay, HSA transition/claim/apply/settlement, obligation-ledger
state, response-delivery closeout, and both requested Linux binaries.

## Differential inherited failures

An untouched archive of `54cf865432fbf7fb3344ddbe83843b81c15c7152` was tested at
`/tmp/substrate-a13p1-baseline.pO1C1J` with target directory
`/tmp/substrate-a13p1-baseline-target`.

```text
$ cargo build -p substrate
candidate: exit 101; excluded substrate-lifecycle-macos target, 33 errors
baseline:  exit 101; same target, 33 errors, and diagnostic classes

$ cargo test -p shell --lib approval_response_is_sent_only_after_completion_post_state_and_audit_persist -- --nocapture
candidate: 3 failures; open activated versioned authority layout
baseline:  same 3 failures and error class

$ cargo test -p shell --lib clarification_response_is_sent_only_after_completion_post_state_and_audit_persist -- --nocapture
candidate: 3 failures; open activated versioned authority layout
baseline:  same 3 failures and error class
```

The public integration target compiles with `--no-run`. Its execution remains blocked by the
inherited all-binary fixture and frozen public Start authority-layout precondition described in the
immutable predecessor verification. No candidate-specific regression was observed.

## GitNexus and direct-flow inspection

GitNexus was refreshed in this exact worktree before the final review subject was frozen. Pre-edit
upstream impact was HIGH for the public request state/guard/acquisition and authority-successor
builder, with one to three direct callers and three to four affected processes; that HIGH result
was reported before editing. `run_turn` and `run_reattach` were LOW. The correction therefore stayed
at the existing recovery seam in `agents_cmd.rs`.

The required final `gitnexus_detect_changes(scope=all)` against the exact worktree path reports
HIGH aggregate breadth: five changed product files, 105 mapped symbols, and 11 existing execution
flows. The flows are the preserved public reattach, public turn, and attach-plan paths. Direct
caller inspection for new or private helpers shows only `run_turn`, `run_reattach`, the existing
successor-plan builder, result acknowledgements, and focused tests. No product path outside the edit
fence and no frozen Start/Stop/auto-attach helper appears in a new hunk.

## Frozen bytes and immutable predecessor evidence

```text
AUTO_ATTACH_SHA256=70e8e344347144971721d3c2ef70a261352282b0b7f2b7788231b297f0944b5e
LAUNCH_HIDDEN_OWNER_HELPER_SENTINEL=f796453a5f16f856d6a278b38694d9521c2f0011d4e4a4ba34160408aeea31c0
BRACE_BALANCED_HELPER_BODY_SHA256=4eff154a3289a77207c1ede241f5d35115577b38d03ab9fba03b6bd2d246f6da
FROZEN_BYTES_OK
```

`auto_attach.rs` is byte-identical to the baseline. The complete
`launch_hidden_owner_helper` function is also byte-identical to the baseline under an exact
brace-balanced extraction; the body hash above matches in both trees. The inherited sentinel is
preserved as recorded by the predecessor verification.

The predecessor `A1.3-P1-public-adoption` review cycle remains immutable and terminal at
`bounded_stop / budget_exhausted`. Its accepted finding `A13P1-P2-008` is the sole authority for
this distinct successor correction sequence. Its six artifact hashes remain:

```text
537448b8692509ed243be1ef916a95ab249a3fd713391c83c4eaefd417c44c01  a1-3-p1-public-adoption-closure-review.txt
64c8f08a496adbc9e454f0c91bc589a31107056a73238592c4813c9dbe29a5e9  a1-3-p1-public-adoption-discovery-review.txt
d5c1602a0744143c8ea8a6785d17ddafa644cfe4f6e180b3786658c2a192b82a  a1-3-p1-public-adoption-review-cycle.json
f0db9bd62886ea1da78fbbea1b733f25994cd67bf2aa69d15f76949e7b0518f3  a1-3-p1-public-adoption-supplemental-1-review.txt
2ff153e9c575968ac797d41ff931ab748697f4c0ac1596637b799ee84d1d0faf  a1-3-p1-public-adoption-supplemental-2-review.txt
eaca250eed8061051617cbd7b3463f346011e025ab6898ce2965375eff1f94f6  a1-3-p1-public-adoption-verification.md
```

## Successor review result

The one authorized independent successor discovery review returned `VERDICT: FINDINGS` for the
exact frozen product subject. It confirmed that the correction closes A13P1-P2-008's pre-apply
identity loss, then reported `A13P1-P2-009` (P2): after durable actor-owned completion and release
of the inherited launch guard, an uncontended exact retry can acquire a new leader permit and spawn
again because durable completion is checked only by the contended wait path. The required bounded
remedy and missing no-respawn proof are preserved verbatim in
`a1-3-p1-public-adoption-correction-discovery-review.txt`.

The distinct successor record is therefore `bounded_stop / authority_required`. The CLEAN landing
condition was not reached, so no local commit was created.
