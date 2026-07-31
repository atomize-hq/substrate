Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3ZT1. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: 773da0f680e7bf330ae22d133c587f96ca9c8e506b90845003e3d7acc6a0fa22
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3ZT1
- packet_id: A1.1d-5R2-3ZT1
- next_increment: R2-3ZH1

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

the fresh Linux Codex worktree provisioned from the remote-tracking product ref

Never mutate these protected checkouts:

- /home/spenser/__Active_code/substrate
- /home/spenser/__Active_code/substrate-r2-3
- /home/spenser/__Active_code/substrate-r2-3-meta-orchestration
- /home/spenser/.codex/worktrees/3b8f/substrate-r2-3
- /home/spenser/.codex/worktrees/4712/substrate-r2-3
- /home/spenser/.codex/worktrees/acb8/substrate
- C:\Users\spmcc\Documents\__Project_Code\substrate-r2-3

Canonical starting state:

- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: 3f909efd1b94bbde99bae160de75958a5447485e
- expected base tree: 1829774bf998bb41f7710a9d38e82bdadf62b4af
- required ancestor: 0f1e147fb735791b44a65099a65167cbdc1803af

Before editing, fetch/query the live remote and verify the exact base, tree, ancestry, cleanliness,
0 ahead/0 behind, and current GitNexus index. If they differ, send BASE_DRIFT or
BLOCKED_CONTRADICTION. Do not reconcile, merge, rebase, reset, clean, or force-push.

SUBAGENT ORCHESTRATION

Use repository-required model/reasoning settings for every subagent. Complete required pre-edit
impact analysis before any subagent edits an existing symbol. Give editing subagents mutually
exclusive ownership when practical. Use fresh read-only subagents for independent review. Do not
allow a reviewer to review implementation it authored.

INCREMENT CONTRACT

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3ZT1

This is the retained-worker test-helper repair required before the remaining host-inbox and macOS
closeout work. It owns one Rust source file and one additional direct shell-library failure only.

This top-level task and every subagent must run at Standard/default speed, never Fast. Every
subagent must use GPT-5.4 with Extra High reasoning.

MISSION

Land one atomic, test-only repair in
`crates/shell/src/execution/agent_runtime/retained_worker_runtime.rs` so the nested
`admission_head_subprocess_worker` receives the explicit trace context required by the accepted
unbound-trace contract. The focused queued-promotion test must pass, and the direct shell library
suite must return from the current 46-failure diagnostic bucket to the exact frozen 45-failure
inventory.

BOUND SOURCE

- commit: `3f909efd1b94bbde99bae160de75958a5447485e`
- tree: `1829774bf998bb41f7710a9d38e82bdadf62b4af`
- target ref: `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- required ancestor: `0f1e147fb735791b44a65099a65167cbdc1803af`

Before editing, verify exact base/tree/ref/ancestor, clean task checkout, 0 ahead/0 behind, and the
current GitNexus index. Analyzer-only `AGENTS.md`/`CLAUDE.md` count churn must be restored to the
starting HEAD and excluded from publication. Any other pre-existing diff is a hard stop.

EXACT TRACKED FILE ALLOWLIST

- `crates/shell/src/execution/agent_runtime/retained_worker_runtime.rs`

No other Rust, test, fixture, script, runner, control-pack, orchestration, platform, or tracked file
may change.

EXACT FAILURE OWNERSHIP

The additional failure to remove is:

`execution::agent_runtime::retained_worker_runtime::tests::queued_promotion_persists_no_request_prompt_payload_or_diagnostic_preimage`

Prior matched base/head classification reproduced it repeatedly and classified the root cause as a
missing explicitly bound trace context in the nested `admission_head_subprocess_worker` path. The
test body and helper are currently in the same allowlisted file. Confirm the failure and causal
path before editing; do not assume that a generic default global context is sufficient under the
accepted explicit trace-authority contract.

AUTHORIZED CHANGE

Make the smallest test-only/helper change that supplies an explicit, deterministic trace context
to the trace-enabled nested subprocess before trace initialization. Preserve all production
retained-worker behavior, admission semantics, prompt/preimage redaction, exact-authority checks,
subprocess isolation, and the final unbound global `init_trace(None)` fail-closed rule.

Do not change trace crate APIs, production runtime code, environment authority, canonical runner
files, frozen 45 failures, or any expectation merely to make the test green. If the repair requires
another file or a production semantic change, stop with `BLOCKED_SCOPE_EXPANSION`.

IMPACT AND SOURCE CLOSURE

Before editing every existing function or method, run GitNexus upstream impact and report direct
callers, affected processes, and risk. Warn before any HIGH/CRITICAL edit. Inspect every in-file
caller of `admission_head_subprocess_worker`, every trace-enabled subprocess setup in this test
module, and the exact trace initialization API used by the accepted source. Source closure must
show the patch is test-only and cannot affect non-test builds.

REQUIRED FOCUSED PROOF

Use private external roots with mode 0700 for `TMPDIR` and `XDG_RUNTIME_DIR`; do not place proof
artifacts in the checkout.

1. Reproduce the focused failure on the unmodified bound base and retain sanitized evidence.
2. After the repair, run at minimum:
   `cargo test --locked -p shell queued_promotion_persists_no_request_prompt_payload_or_diagnostic_preimage -- --nocapture --test-threads=1`.
3. Run the exact nested helper and any neighboring retained-worker tests touched by the causal
   path, serially where global trace state requires isolation.
4. Prove the private prompt marker and diagnostic preimage remain absent from stdout, stderr,
   trace output, persistent admission state, and error text as asserted by the existing test.

DIRECT SHELL-LIBRARY GATE

Do not use `scripts/ci/canonical_shell_wall_runner.py` or any unpublished R2-3ZP3 commit. Run the
direct suite with fresh private external roots:

`cargo test --locked -p shell --lib -- --nocapture`

The nonzero Cargo exit caused by the frozen failures is expected. Parse the complete raw output
without changing repository files and require all of the following:

- exactly `1322 discovered / 1277 passed / 45 failed / 0 ignored`;
- exactly 45 failure names;
- failure-name SHA-256
  `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`;
- exactly 45 normalized signatures;
- normalized-signature SHA-256
  `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`;
- the repaired queued-promotion test is absent from the failure set; and
- no frozen failure is missing, renamed, substituted, or changed.

Count-only equivalence is insufficient. Use the accepted frozen inventory/parser semantics from
the runtime-refactor records read-only; do not edit or invoke the broken authenticated wrapper.
Store raw logs, extracted names, normalized signatures, counts, and hashes outside the checkout.

QUALITY GATES

1. `cargo fmt --all -- --check`.
2. `cargo check --locked -p shell`.
3. Run shell Clippy with `-D warnings`. If the accepted base retains only the documented inherited
   warnings in untouched files, prove the exact base/head differential has no new warning and
   record that limitation; do not edit those files.
4. `git diff --check` and exact one-file containment.
5. `gitnexus_detect_changes()` with every affected process inspected and any graph undercoverage
   dispositioned by manual source closure.

REVIEW AND PUBLICATION

Use fresh read-only GPT-5.4 Extra High reviewers at Standard/default speed covering:

1. explicit trace-authority correctness and preservation of the unbound-trace fail-closed rule;
2. prompt/preimage secrecy, admission semantics, subprocess isolation, and test-only containment;
3. exact direct-wall inventory/hash honesty and exact one-file publication scope.

Validate the standard bounded review-cycle record with the repository validator. Publication
requires terminal CLEAN, P1=0, P2=0, complete P3/P4 disposition, every required proof above, and
the live target still equal to the expected base. Publish one normal fast-forward commit; never
force-push.

COMPLETION BOUNDARY

Success completes only R2-3ZT1. It does not repair the host-inbox helper, land the macOS cleanup,
run native evidence, execute R2-3Z, complete broader runtime-refactor work, or complete any R3 row.

TERMINAL RECEIPT

Use `increment: "R2-3ZT1"`, `packet_id: "A1.1d-5R2-3ZT1"`, and
`next_increment: "R2-3ZH1"`. A LANDED_CLEAN receipt must use the canonical receipt schema and
include landed commit/tree/ref, the exact one changed path, canonical subject-manifest fingerprint
and path, validated standard review record/digest, focused before/after proof, direct shell-wall
counts/name/signature hashes, quality checks, GitNexus result, remote 0/0, and clean checkout. For
failure, send exact evidence, required authority, and a complete handoff prompt. Receipt send is
the final tool action.

COMMON TERMINAL CONTRACT

Before publication:

1. Complete every increment-specific check and proof gate.
2. Verify the exact file/symbol/test allowlist.
3. Run `gitnexus_detect_changes()` and inspect every affected flow.
4. Complete and validate the bounded review sequence.
5. Require zero open blocking findings.
6. Fetch/query the live target again and require it still equals the expected base.

When authorized by the increment contract, create one atomic commit and perform a normal
fast-forward push of `HEAD` to refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap. Never force-push. Verify the live remote equals the
landed commit, refresh GitNexus, restore analyzer-only generated count changes if necessary, and
finish clean.

Send a `codex.top-level-task-receipt.v1` message to the meta task. For success, use
`LANDED_CLEAN` and include your bound increment-task thread/host IDs, expected base, landed
commit/tree, changed paths, subject fingerprint, validated review record and digest, finding
disposition, checks, GitNexus result, clean status, and next increment.

For failure, send the exact blocked status, evidence, required authority or platform, and a
complete continuation/handoff prompt.

The `send_message_to_thread` call is your final tool action. After it succeeds, make no more tool
calls or repository changes. Return only the human-readable final report.

Do not generate the next increment prompt and do not begin R2-3ZH1. The meta
orchestrator owns independent verification and subsequent dispatch.
