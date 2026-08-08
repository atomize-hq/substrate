META IDENTITY BINDING AND START AUTHORITY

Bind exactly:

- increment_task_thread_id: 019fdfa4-ba96-70e0-b68d-1d0a4cf32a83
- increment_task_host_id: local
- exact_task_worktree: /Users/spensermcconnell/.codex/worktrees/2f78/substrate
- dispatch_nonce: 7c15251a50d1f1f9326a43027c009ad73e003931ebb9bf6fe6f723eb07050f65
- hydrated repository-local skill suite: 25 skills / 54 files / sha256 e3de93f358594033ed59696a9624d2d0ac2fd0c2aa7ddf4a532810a05b262282
- expected source: 3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774 / d59cac92405f06024cfcde0a2c22a4cf1255c3d7
- model/reasoning for this task and every subagent: gpt-5.6-terra / Extra High
- start_authorized: true

The following rendered repository-local increment prompt is authoritative. Begin only this packet now.

---

Use the repository-local $orchestrate-top-level-tasks skill and every skill required by the
increment contract. Never use or install a global copy. This initial prompt is self-contained;
before identity binding, the meta orchestrator must hydrate and verify the complete repository-
local suite at `.agents/skills` in the assigned worktree. If the bound follow-up arrives while that
local suite is absent, incomplete, or its persisted digest does not match, stop with
BLOCKED_CONTRADICTION before any work.

ROLE

You are the fresh top-level increment orchestrator for AUX-R3-MAC-AARCH64-GUEST-COMPILE-CORRECTION. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-a1-1d-5r3-mac-20260806-41f97e1c570d
- dispatch_nonce: 7c15251a50d1f1f9326a43027c009ad73e003931ebb9bf6fe6f723eb07050f65
- meta_thread_id: 019fd8c3-b12c-7e73-919f-898600ab0f64
- meta_host_id: local
- increment: AUX-R3-MAC-AARCH64-GUEST-COMPILE-CORRECTION
- packet_id: AUX-R3-MAC-AARCH64-GUEST-COMPILE-CORRECTION
- next_increment: EVIDENCE:R3-MAC-IMP-01

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

/Users/spensermcconnell/.codex/worktrees/2f78/substrate

Never mutate these protected checkouts:

- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
- /Users/spensermcconnell/.codex/worktrees/eb49/substrate
- /Users/spensermcconnell/.codex/worktrees/fa11/substrate

Canonical starting state:

- publication mode: declared by the increment contract
- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: 3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774
- expected base tree: d59cac92405f06024cfcde0a2c22a4cf1255c3d7
- required ancestor: 270f6e55e1a94b7e2f9b2667e605980d2e50579c

Before editing, query the publication-mode authority: the live remote for `remote`, or the exact
local branch ref and worktree for `local`. Verify the exact base, tree, ancestry, cleanliness, and
any required index state. If they differ, send BASE_DRIFT or BLOCKED_CONTRADICTION. Do not
reconcile, merge, rebase, reset, clean, or force-push.

SUBAGENT ORCHESTRATION

Use repository-required model/reasoning settings for every subagent. Complete required pre-edit
impact analysis before any subagent edits an existing symbol. Give editing subagents mutually
exclusive ownership when practical. Use fresh read-only subagents for independent review. Do not
allow a reviewer to review implementation it authored.

INCREMENT CONTRACT

# AUX-R3-MAC-AARCH64-GUEST-COMPILE-CORRECTION

## Objective

Publish the smallest source correction that makes the exact R3 MAC aarch64 Lima guest executor
compile under `aarch64-unknown-linux-gnu`. The independently verified failure is four Rust `E0308`
errors because `CString::as_ptr()` is `*const u8` for this target while the local Linux FFI
declarations hard-code `*const i8`. Use the target-correct C character type; do not change runtime
behavior, authority, selectors, transport, lifecycle, pairing, or platform scope.

## Exact authority

- Publication: remote normal fast-forward to
  `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`.
- Expected base: `3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774` /
  `d59cac92405f06024cfcde0a2c22a4cf1255c3d7`.
- Required ancestor: `270f6e55e1a94b7e2f9b2667e605980d2e50579c`.
- Amendment: `0038-r3-mac-aarch64-guest-compile-correction`.
- User override: every implementation/review subagent must use `gpt-5.6-terra` at Extra High.
- Next increment: `EVIDENCE:R3-MAC-IMP-01`; do not dispatch or run it.

## Exact path and symbol fence

Allowed implementation/test paths:

1. `src/bin/substrate-lifecycle-linux.rs`
2. `tests/mac/aarch64_guest_compile_r3.sh`

Allowed review metadata paths:

3. `llm-last-mile/runtime-refactor/review-control/r3-mac-aarch64-guest-compile-correction-review-authority-security.md`
4. `llm-last-mile/runtime-refactor/review-control/r3-mac-aarch64-guest-compile-correction-review-lifecycle-convergence.md`
5. `llm-last-mile/runtime-refactor/review-control/r3-mac-aarch64-guest-compile-correction-review-allowlist-evidence.md`
6. `llm-last-mile/runtime-refactor/review-control/r3-mac-aarch64-guest-compile-correction-review-cycle-record.json`

Within the Rust binary, edit only the Linux `extern "C"` path pointer types for `open`, `openat`,
and `linkat`, plus imports strictly necessary to use the target-correct C character type. Do not
edit the four callers unless fresh proof shows the declaration correction alone is insufficient.
No other product symbol or path is authorized.

## Pre-edit analysis

Attempt repository-defined GitNexus impact analysis before editing each existing symbol. Do not run
`npx gitnexus analyze` when it would rewrite tracked `AGENTS.md` or `CLAUDE.md`; if the exact task
worktree index is unavailable or stale, record the degraded result and perform complete manual
caller/callee, FFI ABI, cfg, and execution-path analysis. Warn and stop on any genuine HIGH/CRITICAL
impact outside this exact guest-executor closure.

## Implementation and deterministic proof

Use TDD: first add/run the focused regression so the exact base reproduces the aarch64 Linux target
compile failure, then make the smallest correction and prove green. The installed platform inputs
are read-only prerequisites:

- Rust toolchain `1.89.0-aarch64-apple-darwin` with target `aarch64-unknown-linux-gnu`;
- Zig `/opt/homebrew/opt/zig/bin/zig`;
- external linker wrapper
  `/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/AUX-R3-MAC-AARCH64-TOOLCHAIN-PREP/d514db6915877692c799086ce6c96daad6bcd0417d87a1bdf9ba44c2c578da53/linker/aarch64-linux-gnu-zig-cc`.

Required checks:

1. `cargo check --locked --offline --target aarch64-unknown-linux-gnu --bin substrate-lifecycle-linux`.
2. Exact locked offline `cargo build` for that target/binary using an external `CARGO_TARGET_DIR`
   and the explicit external linker wrapper.
3. Verify the built output is Linux ELF64 AArch64 and record its digest.
4. Run the focused existing R6 guest-entrypoint tests that are valid on this macOS host.
5. Run `cargo fmt --all -- --check`, the new regression script, and `git diff --check`.
6. Prove Linux selected runtime bodies other than the ABI-correct declaration are byte-identical;
   prove all Windows/WSL and macOS host product paths are byte-identical to base.
7. Enforce the exact six-path allowlist and scan changed bytes for secrets.

The external build output and logs must stay outside every repository checkout. Do not edit Cargo
manifests, lockfiles, repository Cargo configuration, or generated context files. Do not install,
run native lifecycle actions, or mutate the protected checkout.

## Causal cascading review

Freeze the product/test subject fingerprint and run the repository-defined causal review sequence:
one fresh discovery review or same-subject three-lens burst, then one different fresh closure review.
At most two supplemental cycles are allowed only for P1/P2 findings directly caused or unmasked by
the immediately preceding remediation under unchanged authority. Remediate valid P1/P2 only.
Record P3/P4 in the repository inventory only if required by current repo rules; they do not create
remediation cycles. Validate the review record with `validate_review_cycle.py`. Terminal CLEAN and
zero open P1/P2 are mandatory.

Review must explicitly confirm:

- the FFI types match the target's `c_char` ABI on aarch64 Linux;
- no runtime action, authority, input, selector, Linux-host behavior, Windows behavior, or macOS-host
  behavior changed;
- the focused test would have caught the exact four base failures;
- the exact path/symbol fence and external-build boundary hold.

## Publication and receipt

Run required change detection; use exact manual fallback if GitNexus cannot bind this task worktree.
Reverify the live remote still equals the expected base. Publish exactly one conventional commit by
normal fast-forward push. Verify remote equality, 0/0 divergence, clean task worktree/index, exact
changed paths, final subject fingerprint, and validated review record digest.

Return one `codex.top-level-task-receipt.v1` with status `LANDED_CLEAN`, exact task identity, base,
landed commit/tree/ref, changed files, subject fingerprint, review disposition, checks, change
detection, containment proof, and `next_increment: EVIDENCE:R3-MAC-IMP-01`. Do not run evidence,
MAC-CLOSEOUT, or any successor.

COMMON TERMINAL CONTRACT

Before publication:

1. Complete every increment-specific check and proof gate.
2. Verify the exact file/symbol/test allowlist.
3. Run required change detection.
4. Complete and validate the bounded review sequence.
5. Require zero open blocking findings.
6. Fetch/query the live target again and require it still equals the expected base.

For publication mode `remote`, when authorized by the increment contract, create one atomic commit
and perform a normal fast-forward push of HEAD to refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap. Never force-push. Verify the live
remote equals the landed commit, refresh required indexes, and finish clean.

For publication mode `local`, create exactly the contract-authorized two-commit chain:
`expected base -> reviewed work commit -> mechanical control commit`. Keep the isolated branch and
worktree clean. Do not merge or push. Record the local branch ref, worktree, both commits/trees,
both exact changed-file inventories, their sorted union, and an unchanged remote observation.

Send a codex.top-level-task-receipt.v1 message to the meta task. For remote success, use
LANDED_CLEAN. For local success, use CLOSED_CLEAN. Include your bound increment-task thread/host
IDs, expected base, exact publication identity, changed paths, subject fingerprint, validated
review record and digest, finding disposition, checks, change detection, clean status, and next
increment.

For failure, send the exact blocked status, evidence, required authority or platform, and a
complete continuation/handoff prompt. Preserve the task-assigned worktree exactly; do not clean,
reset, discard, or request archival of a blocked task.

The send_message_to_thread call is your final tool action. After it succeeds, make no more tool
calls or repository changes. Return only the human-readable final report.

Do not generate the next increment prompt and do not begin EVIDENCE:R3-MAC-IMP-01. The meta
orchestrator owns independent verification and subsequent dispatch.
