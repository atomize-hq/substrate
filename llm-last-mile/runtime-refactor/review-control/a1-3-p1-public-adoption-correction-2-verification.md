# A1.3-P1 public-adoption A13P1-P2-009 correction — verification evidence

**Captured:** 2026-08-29 on Linux

## Repository and frozen successor-review subject

```text
branch: feat/runtime-refactor-a1-3-p1
baseline commit: 54cf865432fbf7fb3344ddbe83843b81c15c7152
baseline tree: 49bd951f59291ed11186ecd6c48615d97564f9ab
baseline remote tip: 54cf865432fbf7fb3344ddbe83843b81c15c7152
pre-correction candidate: sha256:4f834a7d02e0c1696d0f1adf99fefe472f8e5d7e37c33eba1ed81b51a8c82c91
successor-review subject: sha256:f0831632c68213d4d216cddeb29ede41e51a139458cfbd68e602e1dc631ace3e
```

The successor-review fingerprint is the SHA-256 of this sorted product manifest. Review-control
evidence is excluded so that recording the independent result does not mutate the implementation
subject.

```text
BASE 54cf865432fbf7fb3344ddbe83843b81c15c7152
100644 850f1771618a1e4aa2ee37995927924c18bb7c36 crates/shell/src/execution/agent_runtime/control.rs
100644 a0ed24879d8cba38dd8c6b4ebc8f321be072df57 crates/shell/src/execution/agents_cmd.rs
100644 8ddf050b740952db400ccd1be3921e7aa5f29ce7 crates/shell/src/execution/orchestrator_world_dispatch.rs
100644 c83a5193ffa45d5b160a6f7d1cb808ac00965cce crates/shell/src/repl/async_repl.rs
100644 5d99f02b4c1de03637dda48e7784b6e750d4152e crates/shell/tests/agent_public_control_surface_v1.rs
```

Subject diff summary:

```text
5 files changed, 5138 insertions(+), 175 deletions(-)
```

Relative to the preserved `4f834a7d...` candidate, the P2-009 correction changes production
bytes only in `crates/shell/src/execution/agent_runtime/control.rs` and adds focused colocated tests
only in `crates/shell/src/execution/agents_cmd.rs`. The other three product blobs remain
byte-identical to that preserved candidate. The accepted A13P1-P2-008 correction and its tests
remain intact.

## A13P1-P2-009 correction

At the shared authority-managed successor launch-guard seam, the implementation now invokes the
existing durable completion lookup and exact-join validator before attempting guard acquisition.
An exact committed result returns the existing joined-completed permit immediately, without
acquiring leadership or launching a helper. If the first lookup is incomplete, the code acquires
the guard and repeats the same durable lookup before it can return a leader permit. Completion in
that race window drops the guard and returns the joined-completed permit. Leadership is returned
only when both checks confirm there is no durable committed result.

The durable completion lookup verifies the full request, operation, session, participant,
expected-revision, transition, claim, launch-plan, handle, evidence, and committed-result identity.
A mismatch fails closed. The correction neither treats guard contention as evidence nor uses PID,
liveness, timeout, EOF, readiness, or event order as durable authority. It adds no lock, ledger,
supervisor, coordinator, authority store, or retry subsystem. The P2-008 path for exact durable
Issued or Claimed recovery remains unchanged.

## Deterministic completion/guard proof

The two new tests each exercise both public successor modes (`Attach` and `ResumeOneTurn`):

```text
$ cargo test -p shell --lib execution::agents_cmd::tests::completed_public_successor_retry_joins_without_uncontended_relaunch -- --exact
1 passed; 0 failed
$ cargo test -p shell --lib execution::agents_cmd::tests::public_successor_completion_between_probe_and_guard_acquisition_joins_without_relaunch -- --exact
1 passed; 0 failed
```

The first test durably applies and settles the original request, releases the original in-memory
guard, and retries with the guard free. It proves the exact committed result joins, the returned
permit is not a leader permit, helper launch is represented by the existing joined receipt with
PID zero, and the HSA root remains byte-identical with exactly the original intent/transition.
It also substitutes the request identity and proves the mismatch fails closed without HSA change.

The race test uses a test-only, thread-local, one-shot hook after successful guard acquisition and
before the post-acquisition durable lookup. The initial lookup observes incomplete, the hook
durably applies and settles the exact request, and the second lookup exact-joins it. The test again
proves no leader/helper launch and no new HSA identity. There is no production fault switch.

The tests were first run against the pre-correction acquisition logic and failed at the intended
no-leadership assertions. They pass with the two durable checks in place.

## Passing preserved A1.3-P1 gates

```text
$ cargo check -p shell --lib
exit 0
$ cargo test -p shell --test agent_public_control_surface_v1 --no-run
exit 0
$ cargo build -p substrate --bin substrate --bin substrate-shim
exit 0
$ cargo test -p shell --lib execution::agents_cmd::tests::public_attach_fault_after_issue_or_claim_reuses_exact_persisted_transition -- --exact
1 passed; 0 failed
$ cargo test -p shell --lib execution::agents_cmd::tests::public_resume_one_turn_fault_after_issue_or_claim_reuses_exact_persisted_transition -- --exact
1 passed; 0 failed
$ cargo test -p shell --lib execution::agents_cmd::tests::concurrent_public_successor_requests_reuse_one_durable_identity_before_issuance -- --exact
1 passed; 0 failed
$ cargo test -p shell --lib execution::agents_cmd::tests::acknowledged_public_successor_exact_joins_then_distinct_request_rotates_identity -- --exact
1 passed; 0 failed
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

These gates cover public Attach and ResumeOneTurn routing, retained-turn correlation,
authenticated actor evidence, stable concurrent request identity, exact Applied/settled joins,
the accepted P2-008 issue/claim crash recovery, Start continuity,
PromptSubmissionIndeterminate zero replay, HSA transition/ledger/terminal-cut behavior, response
delivery, and both requested Linux binaries.

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
inherited all-binary fixture and frozen public Start authority-layout precondition documented in
the immutable predecessor evidence. No candidate-specific regression was observed.

## GitNexus and direct-flow inspection

GitNexus was refreshed in this exact worktree after implementation: 39,857 nodes, 81,710 edges,
1,215 clusters, and 300 execution flows. Pre-edit upstream impact was HIGH for
`acquire_authority_managed_successor_launch_permit` (two direct production callers and three
public process groups) and `authority_managed_successor_completion` (three direct callers and the
same groups); that HIGH result was reported before editing.

The required final `gitnexus_detect_changes(scope=all)` against the exact worktree path reports
HIGH aggregate breadth: five changed product files, 116 mapped symbols, and 11 existing execution
flows. Direct caller/flow inspection for candidate-added or private helpers confirms the affected
production acquisition callers are `run_turn` and `run_reattach`; the new race hook has only the
focused test caller and is compiled only under `cfg(test)`. No new product path, authority store,
or frozen exclusion was introduced.

## Frozen bytes and immutable inherited evidence

```text
AUTO_ATTACH_SHA256=70e8e344347144971721d3c2ef70a261352282b0b7f2b7788231b297f0944b5e
LAUNCH_HIDDEN_OWNER_HELPER_SENTINEL=f796453a5f16f856d6a278b38694d9521c2f0011d4e4a4ba34160408aeea31c0
FROZEN_BYTES_OK
```

`auto_attach.rs` is byte-identical to the baseline. Exact extraction and comparison of the
complete `launch_hidden_owner_helper` function is also byte-identical to the baseline. The
inherited sentinel remains the canonical hash recorded by the prior independent review.

The nine inherited public-adoption records were hashed before this successor record was created
and remain immutable:

```text
537448b8692509ed243be1ef916a95ab249a3fd713391c83c4eaefd417c44c01  a1-3-p1-public-adoption-closure-review.txt
9cc3c51dd1b819d013f12170b1a36542f4e0bc066d751ea04d1fcfbda1a735f5  a1-3-p1-public-adoption-correction-discovery-review.txt
3f858dc78e2d3ac9036ae202088c4edd9d026a36e56445676ceed696e358ba1f  a1-3-p1-public-adoption-correction-review-cycle.json
4329cf9bcd26c32e60b2c4fcbe9f5c47d55186d239a05e17d00a5acd9e388843  a1-3-p1-public-adoption-correction-verification.md
64c8f08a496adbc9e454f0c91bc589a31107056a73238592c4813c9dbe29a5e9  a1-3-p1-public-adoption-discovery-review.txt
d5c1602a0744143c8ea8a6785d17ddafa644cfe4f6e180b3786658c2a192b82a  a1-3-p1-public-adoption-review-cycle.json
f0db9bd62886ea1da78fbbea1b733f25994cd67bf2aa69d15f76949e7b0518f3  a1-3-p1-public-adoption-supplemental-1-review.txt
2ff153e9c575968ac797d41ff931ab748697f4c0ac1596637b799ee84d1d0faf  a1-3-p1-public-adoption-supplemental-2-review.txt
eaca250eed8061051617cbd7b3463f346011e025ab6898ce2965375eff1f94f6  a1-3-p1-public-adoption-verification.md
```

The immediately preceding `A1.3-P1-public-adoption-correction` record remains immutable at
`bounded_stop / authority_required`. Its accepted `A13P1-P2-009` finding and this explicit user
authorization establish the scope of this distinct `A1.3-P1-public-adoption-correction-2`
successor sequence. No admission was repeated and A13P1-P2-008 was not reopened.

## Successor review result

The one authorized fresh, read-only successor reviewer independently recomputed the exact
`f0831632...` product subject, reviewed the complete five-file candidate plus the P2-009 delta,
and returned `VERDICT: CLEAN` with no unresolved valid P1 or P2. The reviewer confirmed both
pre-acquisition and post-acquisition durable completion checks, no leadership/helper launch for
exact committed results, full-identity fail-closed behavior, preserved P2-008 recovery, frozen
exclusions, and both frozen hashes. The raw result is preserved in
`a1-3-p1-public-adoption-correction-2-discovery-review.txt`; the validator-bound record is
`a1-3-p1-public-adoption-correction-2-review-cycle.json`.
