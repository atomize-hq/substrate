# A1.3-P1 Start continuity correction — verification evidence

**Captured:** 2026-08-29 on Linux

## Environment and repository binding

```text
$ uname -a
Linux spenser-linux 6.16.8-1-MANJARO #1 SMP PREEMPT_DYNAMIC Fri, 19 Sep 2025 16:09:36 +0000 x86_64 GNU/Linux
$ rustc --version
rustc 1.89.0 (29483883e 2025-08-04)
$ cargo --version
cargo 1.89.0 (c24e10642 2025-06-23)
$ git rev-parse --show-toplevel
/home/spenser/__Active_code/.worktrees/substrate-runtime-refactor-a1-3-p1
$ git branch --show-current
feat/runtime-refactor-a1-3-p1
$ git rev-parse HEAD
28b4055f04557349020f44a2973c043f1508d6e1
$ git rev-parse HEAD^{tree}
f263981152ed3d8c9a373145dfa4261e4ee2d2f6
```

The comparison checkout used for every failing broad command was a detached, untouched worktree at
`28b4055f04557349020f44a2973c043f1508d6e1`, located at
`/tmp/substrate-baseline-28b4055f` for this verification run.

## Passing focused proof

```text
$ cargo fmt --all -- --check
exit 0
$ git diff --check
exit 0
$ cargo build -p substrate --bin substrate --bin substrate-shim
Finished `dev` profile; exit 0
$ cargo test -p shell --lib execution::agent_runtime::host_session_authority::transition_tests:: -- --nocapture
test result: ok. 48 passed; 0 failed; 0 ignored; 1343 filtered out
$ cargo test -p shell --lib hidden_owner_helper_start_submits_real_prompt_once_and_settles_hsa -- --nocapture
test result: ok. 1 passed; 0 failed
$ cargo test -p shell --lib start_settlement_rejects_data_less_or_message_only_wrapper_statuses -- --nocapture
test result: ok. 1 passed; 0 failed
$ cargo test -p shell --lib start_lifecycle_rejects_phase_regression_after_exact_turn_start -- --nocapture
test result: ok. 1 passed; 0 failed
$ cargo test -p shell --lib committed_start_response_is_acknowledged_only_after_successful_exact_rendering -- --nocapture
test result: ok. 1 passed; 0 failed
$ cargo test -p shell --lib strict_start_terminal_renderer_propagates_delivery_write_and_flush_failures -- --nocapture
test result: ok. 1 passed; 0 failed
$ cargo test -p shell --lib claude_durable_start_preserves_noninteractive_flags_and_typed_lifecycle -- --nocapture
test result: ok. 1 passed; 0 failed
$ cargo test -p shell --lib b21_retained_snapshot_is_pending_until_exact_terminal_cut_then_completes -- --nocapture
test result: ok. 1 passed; 0 failed
```

The 48 HSA transition tests include the following focused proof:

- exact registration and settlement retry joins, including rejection of stale authority revisions,
  substituted exchange/completion evidence, reordering, and conflicting retries;
- crash-boundary states for prompt-not-submitted, exchange-in-progress, registered-but-unsettled,
  settled-before-response, response delivery, and exact duplicate convergence;
- `ParkedResumable`, authenticated canonical-ledger `AwaitingAttention`, and exact terminal
  settlement outcomes, with backend identity unable to select posture; and
- issuance, claim, and application of the existing HSA `ResumeOneTurn` protocol against the handle
  produced by Start, plus the pre-existing successor Attach/Resume transition coverage.

The real-wrapper tests launch fake Codex and Claude processes through the provider-native Start
adapters and assert one nonempty caller prompt, no bootstrap/empty/synthetic prompt, the real native
thread/turn identity, registered continuation, and durable HSA settlement. The Claude proof also
checks the required noninteractive `--permission-mode bypassPermissions` invocation. Wrapper-status
and lifecycle-regression tests prove that ordinal, data-less, message-only, reordered, repeated, and
substituted provider lifecycle rows cannot settle Start.

Public Start prepares and commits its exact HSA `PromptNotSubmitted` transaction before opening the
transport that can submit the caller prompt. The child runtime exact-joins that committed
transaction and authenticated authority origin instead of replaying Start issue/claim/application.
Its Start-only launch plan, startup socket, private control socket, and toolbox socket live under
`runtime-control/durable-start`, outside the guarded legacy `run/` authority tree; public Turn,
Attach, and Stop retain their existing launch and authority paths.

The public-response tests prove that a committed settlement is not acknowledged until its exact
typed result is rendered, that failed rendering leaves the durable transaction joinable, and that a
retry renders the committed terminal success or exact terminal failure rather than inventing a
generic success. The canonical-ledger integration proof exercises the production Start consumer:
it fails closed before the exact retained-work terminal cut and returns the authenticated canonical
attention result after that cut without changing C1 classification or materialization semantics.

The changed control-plane Markdown check inspected 18 repository-local links/fragments with zero
failures.

## Frozen compatibility bytes

The complete existing `launch_hidden_owner_helper` function body is byte-identical to `HEAD` under
an exact `cmp`; the separately added durable Start launcher does not alter that legacy function.

```text
LAUNCH_HIDDEN_OWNER_HELPER_SENTINEL=0550b021c56125842b6cb3d7048c5f66f2409683e9ac94a81002621b45fe63b2
AUTO_ATTACH_SHA256=70e8e344347144971721d3c2ef70a261352282b0b7f2b7788231b297f0944b5e
FROZEN_BYTES_OK
```

The inherited bounded-stop evidence also remains unchanged during this correction:

```text
7027c02d56c10c20e39ac6aaa99b26c82afd06d82c693666624d2c227be025ce  a1-3-p1-start-continuity-correction-discovery-review.txt
63bbd9b617707cf07aa01c914ad4edad428c775c1d6896aa878a158897aa728b  a1-3-p1-start-continuity-correction-review-cycle.json
```

## Differential broad verification

### Affected public Start integration case

The candidate and untouched baseline ran the exact same command:

```text
$ cargo test -p shell --test agent_public_control_surface_v1 public_start_turn_and_stop_emit_streaming_ndjson_and_authoritative_state -- --nocapture
exit 101
```

Both executions failed before exercising the test body because its shared fixture invokes
`cargo build -p substrate`, which compiles the excluded macOS lifecycle binary on Linux. Both
reported the same 33 errors: missing Unix extension traits and the unavailable
`mac_publish_immutable_file_atomic_no_replace_v1` symbol. Both then reported the same fixture panic
at `crates/shell/tests/common.rs:117`. No candidate-only diagnostic occurred.

### Full public-control integration target

```text
$ cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
candidate: exit 101; 0 passed; 44 failed
baseline:  exit 101; 0 passed; 44 failed
```

Every case stopped at the same shared all-binary build fixture with the same 33 macOS binary errors.

### Full shell test command

```text
$ cargo test -p shell -- --nocapture
candidate: exit 101
baseline:  exit 101
```

Both executions failed during compilation of the pre-existing excluded `managed_lifecycle_v1`
integration target with the same missing `MAC_PUBLISHER_SERVICE_LABEL_V1` at
`src/bin/substrate-lifecycle-control.rs:1452`. No candidate test failed before that build stop.

### Shell clippy wall

```text
$ cargo clippy -p shell --all-targets -- -D warnings
candidate: exit 101; shell lib 12 errors; shell lib test 9 errors
baseline:  exit 101; shell lib 12 errors; shell lib test 9 errors
```

The exact diagnostic set is unchanged: unused/dead macOS-client imports and types under
`crates/shell/src/execution/managed_lifecycle/macos_client.rs`, plus the existing
`clippy::needless_borrow` at `crates/shell/src/builtins/world_deps/mod.rs:236`. A candidate-local
`clippy::nonminimal_bool` diagnostic was corrected before this final differential rerun.

## GitNexus scope detection

The pre-review `gitnexus_detect_changes(scope=all)` reported aggregate HIGH risk because the
preserved correction candidate touches 191 indexed symbols across 20 tracked files. Its 14 affected
flows are confined to the expected prompt-submission/composition path, canonical obligation-ledger
reads, and existing successor Attach/Resume test flows. The HIGH aggregate is a breadth signal; no
unexpected production execution flow was reported. GitNexus also marks `run_turn` as touched
because its shared startup-plan initializer sets the four new Start-only optional fields to `None`;
direct diff inspection confirms that public Turn still uses `launch_hidden_owner_helper` and does
not adopt the HSA `ResumeOneTurn` protocol in this correction. The untracked
`start_continuity.rs` module is not represented in the indexed symbol count and therefore requires
direct review.
