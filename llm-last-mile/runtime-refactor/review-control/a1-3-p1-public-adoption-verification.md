# A1.3-P1 remaining public adoption — verification evidence

**Captured:** 2026-08-29 on Linux

## Repository and frozen review subject

```text
branch: feat/runtime-refactor-a1-3-p1
baseline commit: 54cf865432fbf7fb3344ddbe83843b81c15c7152
baseline tree: 49bd951f59291ed11186ecd6c48615d97564f9ab
baseline remote tip: 54cf865432fbf7fb3344ddbe83843b81c15c7152
subject fingerprint: sha256:986cb6ed03475c41427131dcb82ca46ac2ab820a8f4603c7eb949c60d3d9e2fa
```

The fingerprint is the SHA-256 of this sorted manifest. Review-control evidence is excluded from
the implementation subject so that recording the independent result does not invalidate it.

```text
BASE 54cf865432fbf7fb3344ddbe83843b81c15c7152
100644 a6d8423b27b6ecc92c2870bc57dc65d442c1be1b crates/shell/src/execution/agent_runtime/control.rs
100644 9699a5fb4476ed3de27c6e2322fbf9a985bedcb9 crates/shell/src/execution/agents_cmd.rs
100644 8ddf050b740952db400ccd1be3921e7aa5f29ce7 crates/shell/src/execution/orchestrator_world_dispatch.rs
100644 c83a5193ffa45d5b160a6f7d1cb808ac00965cce crates/shell/src/repl/async_repl.rs
100644 5d99f02b4c1de03637dda48e7784b6e750d4152e crates/shell/tests/agent_public_control_surface_v1.rs
```

Subject diff summary:

```text
5 files changed, 3965 insertions(+), 126 deletions(-)
```

## Behavioral proof

- Linux public `run_reattach` builds and launches the existing HSA `Attach` successor. Linux public
  `run_turn` builds and launches the existing HSA `ResumeOneTurn` successor; no new Turn transition
  exists. The two remaining direct legacy-launch calls in these entry points are under
  `#[cfg(not(target_os = "linux"))]` compatibility branches.
- Both successor plans carry the exact applied intent, expected authority revision, root/lease,
  descriptor, workspace/world binding, target/run identity, and the real durable resume handle
  settled by Start. The hidden runtime exact-joins those commitments before accepting evidence.
- Attach success comes only from an authenticated typed startup-ownership `Accepted` event.
  Resume success comes only from authenticated provider lifecycle evidence plus the exact retained
  acceptance/obligation-ledger terminal cut. PID, readiness, EOF, timeout, status ordering, and
  ordinal inference are not success authorities.
- The authority-managed Attach runtime no longer persists legacy participant/session snapshots or
  heartbeats before HSA acceptance. It binds the durable UAA transport in memory and resolves only
  the exact HSA startup-ownership event. A stable session/request guard is acquired before reading
  or issuing HSA state; exact concurrent calls reuse one durable intent/issuer/claim/attempt identity
  and join the same Issued, Claimed, or Applied work. The exact transition-intent launch guard is
  inherited by the helper across exec, with inheritance enabled only in the forked child's pre-exec
  hook, so launcher exit cannot release coordination while that helper remains live and unrelated
  concurrent spawns cannot retain the descriptor. Concurrent followers wait for and join
  authenticated HSA completion without spawning or replacing the leader's transports. A completed
  Attach is not joinable for a second sequential helper launch.
- Exact `turn.failed` and error-result lifecycle evidence now routes through the existing
  `TargetFailedAfterInputAcceptance` terminal-failure outcome. It exact-joins the authenticated
  retained acceptance correlation and terminal actor evidence; provider failure alone cannot
  invent a terminal cut.
- Existing HSA transition coverage rejects stale revisions, substituted or reordered evidence,
  mismatched handles, and wrong transition identities, and proves exact duplicate issue, claim,
  apply, acceptance, reconciliation, and settlement joins.
- `prepare_retained_acceptance_submission` selects the unique current applied ResumeOneTurn
  correlation from exact HSA authority, and the retained acceptance context and worker submit
  request clone that authorized value unchanged.
- The completed Start-continuity path remains the real-prompt path, settles the native durable
  provider handle, treats ambiguous submission as `PromptSubmissionIndeterminate`, and makes an
  indeterminate retry perform zero provider submissions.

## Passing focused gates

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
$ cargo test -p shell --lib concurrent_public_successor_requests_reuse_one_durable_identity_before_issuance -- --nocapture
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
$ cargo fmt --all -- --check
exit 0
$ git diff --check
exit 0
```

The updated Linux public-control tests perform a real public Start first, assert the durable native
resume handle, exercise public ResumeOneTurn fail-closed without the retained authenticated actor
cut, and prove zero provider resubmissions on exact or changed retries. The public Attach case
asserts exact HSA revision/transition identity, typed ownership acceptance, real provider handle,
current authority, absence of target legacy StateStore authority writes, compatibility auto-attach
settlement, concurrent exact-retry convergence on one HSA target and one helper PID, and
completed-duplicate rejection. The added retained ResumeOneTurn scenario drives the
production `spawn_world_worker`/`continue_world_worker` path and verifies the committed post-turn
object carries the exact unchanged HSA intent/run/participant correlation. The focused successor
coordination unit cases prove that concurrent public requests reuse one durable pre-issuance
identity and that the child process retains the exact launch guard after the parent drops its copy.

## Supplemental causal basis

Closure finding `A13P1-P2-005` was directly unmasked by the immediately preceding remediation of
discovery finding `A13P1-P2-003`: excluding already-completed Attach from recovery exposed the
separate pending-startup retry window in the same public reattach/authority-managed launcher flow.
The first bounded remedy added a transition-intent launch permit/guard plus concurrent public retry
proof in that exact flow.

Supplemental findings `A13P1-P2-006` and `A13P1-P2-007` were directly caused or exposed by that
permit remediation: the first permit lifetime ended with the parent launcher instead of the live
helper, and its per-intent key began only after randomized HSA issuance. The bounded follow-on keeps
the same public Attach/ResumeOneTurn and authority-managed launcher fence. It transfers the exact
intent guard descriptor to the helper across exec and introduces a session/request guard before HSA
issuance whose persisted exact identities let contended retries join the same Issued, Claimed, or
Applied transition. No new authority source, transition, success inference, or transport replacement
is introduced.

## Differential inherited failures

An untouched baseline archive of `54cf865432fbf7fb3344ddbe83843b81c15c7152` was tested with a
separate target directory at `/tmp/substrate-a13p1-baseline.pO1C1J`.

```text
$ cargo build -p substrate
candidate: exit 101; 33 excluded macOS lifecycle binary errors
baseline:  exit 101; same 33 errors and diagnostic classes

$ cargo test -p shell --test agent_public_control_surface_v1 <focused-public-case> -- --nocapture
candidate: exit 101 before the test body; shared fixture failed while building all substrate bins
baseline:  exit 101 at the same fixture with the same macOS lifecycle errors

$ PATH=<candidate-only all-bin wrapper> cargo test -p shell --test agent_public_control_surface_v1 public_turn_applies_retained_resume_correlation_and_exact_terminal_cut -- --exact --nocapture
candidate: reached the test body through a private installed invocation witness, then failed before
provider launch because frozen public Start queried retryable V2/V3 continuity while its exact root
was still schema V1 (`current authority requires strict StateRootV2 or StateRootV3`)
baseline/source differential: the failing `run_start`, authority bootstrap, and retryable Start
lookup are byte-identical to 54cf8654; this slice did not modify or reopen that frozen path

$ cargo test -p shell --lib approval_response_is_sent_only_after_completion_post_state_and_audit_persist -- --nocapture
candidate: exit 101; open activated versioned authority layout
baseline:  exit 101; same inherited authority-layout precondition failure
```

The candidate-specific retained-terminal-cut and response-delivery lower-level cases listed above
pass. The integration target itself compiles with `--no-run`; execution is blocked by its inherited
all-binary fixture, not by candidate compilation.

## Independent review result

The final permitted `supplemental-causal-2` review returned `VERDICT: FINDINGS` with blocking
finding `A13P1-P2-008`. It independently confirmed that a launcher death after HSA issue or claim
but before apply releases the session/request lock; because an uncontended retry ignores the
persisted identity and generic recovery considers only Applied intents, that retry can issue a
distinct successor and strand the first nonterminal intent. The exact reviewed product fingerprint
was `sha256:986cb6ed03475c41427131dcb82ca46ac2ab820a8f4603c7eb949c60d3d9e2fa`.

The review record is therefore a canonical `bounded_stop` with `budget_exhausted`. No landing commit
is authorized. The bounded remedy is recorded verbatim in
`a1-3-p1-public-adoption-supplemental-2-review.txt`; implementing it requires explicit authority for
a successor review sequence or other packet-authorized continuation.

## GitNexus scope detection

GitNexus was refreshed at the exact baseline before editing. Upstream pre-edit impact was LOW for
`run_turn`, `run_reattach`, `run_owner_helper`, and the attach/resume builders. The existing launch
plan type was MEDIUM because it has seven direct callers, including frozen auto-attach, so the
implementation used a separate successor envelope and did not alter that type. Unindexed private
REPL and retained-acceptance helpers were reconciled through direct caller and flow inspection.
No pre-edit result was HIGH or CRITICAL.

The required pre-review `gitnexus detect-changes --scope all` against this exact worktree reports
CRITICAL aggregate breadth: 5 files, 245 mapped symbols, and 25 affected processes. Direct
zero-context hunk inspection reconciles the report to GitNexus's coarse parsing of large Rust files:
it labels unchanged neighboring Start/list/status/Stop/control symbols as touched, while actual
hunks are confined to the admitted imports, public Turn/Reattach/builders, authority-managed
transport, allowed REPL seams, retained-acceptance projection, and focused tests. No `run_start` or
`run_stop` hunk exists. The reported execution processes are existing broad command/start/list/
doctor paths attributed through those coarse file mappings; no additional out-of-fence product
path appears in the diff.

## Frozen compatibility bytes

```text
AUTO_ATTACH_SHA256=70e8e344347144971721d3c2ef70a261352282b0b7f2b7788231b297f0944b5e
LAUNCH_HIDDEN_OWNER_HELPER_SENTINEL=f796453a5f16f856d6a278b38694d9521c2f0011d4e4a4ba34160408aeea31c0
FROZEN_BYTES_OK
```

`crates/shell/src/execution/agent_runtime/auto_attach.rs` is byte-identical to the baseline. An
exact brace-balanced extraction and `cmp` of the complete `launch_hidden_owner_helper` function is
also byte-identical to the baseline; the sentinel is the SHA-256 of that extraction in both trees.
