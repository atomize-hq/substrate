Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3ZH1. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: e4423f8ee31677b96ad201349ea6c808c94748e75216b82b134e28fd7946a194
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3ZH1
- packet_id: A1.1d-5R2-3ZH1
- next_increment: R2-3ZM5

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
- /home/spenser/.codex/worktrees/e847/substrate-r2-3
- /home/spenser/.codex/worktrees/3b8f/substrate-r2-3
- /home/spenser/.codex/worktrees/4712/substrate-r2-3
- /home/spenser/.codex/worktrees/acb8/substrate
- C:\Users\spmcc\Documents\__Project_Code\substrate-r2-3

Canonical starting state:

- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: 56e0a8582d562bd7e60e8f4348b4d596e1b2b36e
- expected base tree: de567b39283733403c089d18fe81ae5d81d081ee
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3ZH1

This is the isolated host-inbox test-harness repair required before the macOS closeout cleanup.
It owns one private test helper in one Rust source file. It does not own production StateStore or
authority-root behavior.

This top-level task and every subagent must run at Standard/default speed, never Fast. Every
subagent must use GPT-5.4 with Extra High reasoning.

MISSION

Land one atomic, test-only repair in
`crates/shell/src/execution/host_inbox_materialization.rs` so its private `tests::with_store`
helper creates the test StateStore beneath an authenticated trusted root instead of default
temporary-directory ancestry. The exact focused host-inbox group must pass in the formerly
failing default-temp environment, and the direct shell library suite must remain at the accepted
48-failure decomposition landed by R2-3ZT1.

BOUND SOURCE

- commit: `56e0a8582d562bd7e60e8f4348b4d596e1b2b36e`
- tree: `de567b39283733403c089d18fe81ae5d81d081ee`
- target ref: `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- required ancestor: `0f1e147fb735791b44a65099a65167cbdc1803af`

Before editing, verify exact base/tree/ref/ancestor, clean task checkout, 0 ahead/0 behind, and the
current GitNexus index. Analyzer-only `AGENTS.md`/`CLAUDE.md` count churn must be restored to the
starting HEAD and excluded from publication. Any other pre-existing diff is a hard stop.

EXACT TRACKED FILE ALLOWLIST

- `crates/shell/src/execution/host_inbox_materialization.rs`

No other Rust, test, fixture, script, runner, control-pack, orchestration, platform, manifest,
lockfile, or tracked file may change.

EXACT SYMBOL AND BEHAVIOR ALLOWLIST

Inside the private `#[cfg(test)]` module, change only `tests::with_store` and the minimum test-only
imports needed by that helper. The repair may:

- resolve an authenticated parent from `XDG_RUNTIME_DIR`, otherwise `HOME/.cache`;
- create the selected parent when necessary;
- create the disposable StateStore root with `tempfile::tempdir_in`;
- set Unix mode `0700`; and
- install that explicit root through the existing test environment guard.

Use the smallest implementation consistent with neighboring accepted shell harnesses. Preserve
the helper callback contract and temporary-directory lifetime. Do not change any production body,
StateStore preflight, authority-root validation, error behavior, routing/materialization semantics,
or any other test helper.

HISTORICAL CAUSAL EVIDENCE

The prior clean-base reproduction ran the focused host-inbox group before this helper repair and
observed seven tests fail at `host_inbox_materialization.rs` with:

`persist detached orchestrator: legacy authority writer preflight failed`

caused by:

`open retained legacy StateStore root`

The stale helper used plain `tempfile::tempdir()` under default temporary ancestry. A preserved
unpublished attempt subsequently changed only this helper and improved the group to eight passing
tests. Treat those records as read-only causal evidence, not as publication authority. Reproduce
the failure freshly on the bound base before editing under an external default-temp/untrusted-root
environment. If the failure cannot be reproduced or has a different cause, stop with
`BLOCKED_CONTRADICTION` rather than copying historical WIP blindly.

The historical WIP checkout `/home/spenser/.codex/worktrees/e847/substrate-r2-3` is protected and
read-only. It may be inspected only with read-only Git commands if still present. Do not run tests,
formatters, analyzers, restores, staging, commits, handoffs, or cleanup there. Re-derive the repair
from the accepted source and causal proof; do not transfer unrelated macOS WIP.

IMPACT AND SOURCE CLOSURE

Before editing every existing function or method, run file-qualified GitNexus upstream impact and
report direct callers, affected processes, modules, and risk. Warn before any HIGH/CRITICAL edit.
Inspect every in-file caller of `tests::with_store`, the exact legacy-authority preflight rejection
path, and neighboring accepted trusted-root test helpers. Source closure must prove that the patch
is test-only and absent from non-test builds. If a production or second-file change is required,
stop with `BLOCKED_SCOPE_EXPANSION`.

REQUIRED FOCUSED PROOF

Keep logs and proof roots outside the checkout. Use deterministic external roots and record their
mode and ownership.

1. On the unmodified bound base, reproduce the historical host-inbox failure with
   `XDG_RUNTIME_DIR` unset and default/untrusted temporary ancestry (for example `TMPDIR=/tmp`),
   retaining sanitized output and the exact failing test names.
2. After the repair, rerun:
   `cargo test --locked -p shell host_inbox_materialization -- --nocapture --test-threads=1`
   under the same formerly failing environment and require all eight focused tests to pass.
3. Run the same focused group with a private mode-0700 external `TMPDIR` and `XDG_RUNTIME_DIR` to
   prove the helper is stable under the accepted direct-wall environment.
4. Prove the helper-created root is beneath the authenticated selected parent, has mode `0700`,
   remains disposable, and does not mutate any caller-visible production state.

DIRECT SHELL-LIBRARY REGRESSION GATE

Do not use `scripts/ci/canonical_shell_wall_runner.py` or any unpublished R2-3ZP3 commit. Run:

`cargo test --locked -p shell --lib -- --nocapture`

with fresh private mode-0700 external `TMPDIR` and `XDG_RUNTIME_DIR`. The nonzero Cargo exit from
accepted failures is expected. Parse the complete raw output without changing repository files
and require:

- exactly `1322 discovered / 1274 passed / 48 failed / 0 ignored`;
- exactly 48 failure names with SHA-256
  `c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9`;
- exactly 48 normalized signatures with SHA-256
  `2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90`;
- the frozen 45-name inventory remains unchanged;
- the only three names outside it remain exactly the separately owned world-deps/doctor failures
  recorded by R2-3ZT1; and
- no `host_inbox_materialization` test appears in the failure set.

Count-only equivalence is insufficient. Do not edit or bless any of the 48 accepted failures.
The three additional world-deps/doctor failures remain broader runtime-refactor debt.

QUALITY GATES

1. `cargo fmt --all -- --check`.
2. `cargo check --locked -p shell`.
3. Run shell Clippy with `-D warnings`. If the accepted base retains only documented inherited
   warnings in untouched files, prove the exact base/head differential adds no warning and record
   the limitation; do not edit those files.
4. `git diff --check` and exact one-file containment.
5. `gitnexus_detect_changes()` with every affected flow inspected and graph undercoverage
   dispositioned by manual source closure.

REVIEW AND PUBLICATION

Use fresh read-only GPT-5.4 Extra High reviewers at Standard/default speed covering:

1. trusted-root correctness, causal repair, and preservation of production authority checks;
2. test-only containment, temporary-root lifetime/permissions, and cross-environment robustness;
3. exact direct-wall inventory/hash honesty and exact one-file publication scope.

Validate the standard bounded review-cycle record with the repository validator. Publication
requires terminal CLEAN, P1=0, P2=0, complete P3/P4 disposition, every required proof above, and
the live target still equal to the expected base. Publish one normal fast-forward commit; never
force-push.

COMPLETION BOUNDARY

Success completes only R2-3ZH1. It does not change production host-inbox or StateStore behavior,
repair the 48 accepted shell failures, land the macOS cleanup, run native evidence, execute R2-3Z,
complete broader runtime-refactor work, or complete any R3 lifecycle/forwarding row.

TERMINAL RECEIPT

Use `increment: "R2-3ZH1"`, `packet_id: "A1.1d-5R2-3ZH1"`, and
`next_increment: "R2-3ZM5"`. A LANDED_CLEAN receipt must include landed commit/tree/ref, the exact
one changed path, subject-manifest fingerprint/path, validated review record/digest, focused
before/after causal proof, exact 48-result direct-wall counts/name/signature hashes, quality checks,
GitNexus result, remote 0/0, and clean checkout. For failure, send exact evidence, required
authority, and a complete handoff prompt. Receipt send is the final tool action.

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

Do not generate the next increment prompt and do not begin R2-3ZM5. The meta
orchestrator owns independent verification and subsequent dispatch.
