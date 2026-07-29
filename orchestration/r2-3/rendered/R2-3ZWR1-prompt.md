Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3ZWR1. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: c013b5f07611ca8347b09688715f915e6a1291352d49816c8f483796b1405c32
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3ZWR1
- packet_id: A1.1d-5R2-3ZWR1
- next_increment: EVIDENCE:R2-3Z

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

the task-assigned native-Windows Codex worktree

Never mutate these protected checkouts:

- /home/spenser/__Active_code/substrate
- /home/spenser/__Active_code/substrate-r2-3
- /home/spenser/__Active_code/substrate-r2-3-meta-orchestration
- C:\Users\spmcc\Documents\__Project_Code\substrate-r2-3

Canonical starting state:

- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: 58462482126609ad87e59b896f5d22437461d144
- expected base tree: 131cb88fcd22cf739eac5c29a2d6e7d3dded43f0
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3ZWR1

This is the user-authorized narrow Windows repair inserted after R2-3T and before the R2-3Z
native-evidence gate. It owns only the two contradictions recorded in
`orchestration/r2-3/receipts/R2-3Z-WIN-EVIDENCE-diagnostic-2.json`.

Use the available skills appropriate to source analysis, test-driven repair, GitNexus impact
analysis, security review, code review, and git publication. This top-level task and every
subagent must run at Standard/default speed, never Fast. Every subagent must use GPT-5.4 with
Extra High reasoning. If a required skill is not installed on the Windows host, report that
honestly and follow this self-contained contract; missing skill installation is not authority to
widen scope.

SELECTED OUTCOME

Repair only:

1. the accepted-source Windows compilation defects that prevent `cargo build --locked -p shell
   --lib` from reaching a successful native-Windows library build; and
2. the tracked W2Only assertion at `scripts/windows/prefix-mapping-r2-3.Tests.ps1:2033` so it uses
   a literal, honest regex without strict-mode interpolation or a spurious backslash before
   `$resolvedProject`.

Preserve the accepted R2-3A through R2-3T behavior. This increment repairs cfg/build hygiene and
the focused regression proof only. It does not change platform mapping, transport selection,
forwarder behavior, provisioning, lifecycle, replay, shim, trace, policy, deletion ownership, or
any R3 surface.

EXACT COMPLETION CLAIM

R2-3ZWR1 completes only the narrow repair prerequisite needed to rerun native R2-3Z evidence. It
does not complete either native evidence gate, R2-3Z, the R2-3 parent packet, or any R3 row.

EXACT ALLOWLIST

Production changes are limited to:

- `crates/shell/src/repl/async_repl.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`

Test changes are limited to:

- `scripts/windows/prefix-mapping-r2-3.Tests.ps1`
- colocated `#[cfg(test)]` modules already within the three allowed Rust production files, only
  when mechanically necessary to prove the named cfg/build repair.

Do not edit `scripts/windows/start-forwarder.ps1`; the diagnostic proved its release-first,
debug-fallback behavior is correct. Do not edit any other shell source, test, script, manifest,
`Cargo.lock`, control document, generated analyzer file, `AGENTS.md`, or `CLAUDE.md`.

If a successful native `cargo build --locked -p shell --lib` requires any additional tracked file,
stop with `BLOCKED_SCOPE_EXPANSION`. Report the exact next compiler diagnostic, required file and
symbol, why the four-file allowlist cannot repair it, and a copy-ready continuation prompt. Do not
silently widen the allowlist.

PRE-EDIT BARRIER AND IMPACT

Before editing:

1. Verify the task-assigned Windows worktree is clean and exactly at commit
   `58462482126609ad87e59b896f5d22437461d144`, tree
   `131cb88fcd22cf739eac5c29a2d6e7d3dded43f0`, target ref
   `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`, 0 ahead/0 behind, with
   required ancestor `0f1e147fb735791b44a65099a65167cbdc1803af`.
2. Verify the live remote still equals that exact commit and tree.
3. Make the GitNexus index current for the exact task checkout. Restore analyzer-only
   `AGENTS.md`/`CLAUDE.md` churn before implementation.
4. Run file-qualified upstream impact analysis, including tests, for every existing symbol that
   may be edited. Treat graph undercoverage of Windows cfg paths as undercoverage, not proof.
5. Reproduce and record the first actionable diagnostics from:
   - `cargo build --locked -p shell --lib`; and
   - `pwsh -NoProfile -File scripts/windows/prefix-mapping-r2-3.Tests.ps1 -W2Only`.
6. Inspect exact callers/cfg definitions for:
   - `PreparedSpawnWorldWorkerBootstrap`;
   - `WorldWorkReceiptRegistry`;
   - `fork_world_worker`;
   - `resolve_internal_dispatch_context`;
   - `enforce_world_dispatch_steering_policy`;
   - the line-2033 release-binary assertion; and
   - the unchanged release-first/debug-fallback block in `start-forwarder.ps1`.

Do not begin by suppressing the entire shell crate, REPL, receipt registry, or orchestrator
dispatch module on Windows. Use the smallest cfg-alignment repair consistent with existing
platform support and fail-closed behavior.

REPAIR CONTRACT

For the Rust build defects:

- cfg-gate imports, impls, parameters, statements, or functions only where the referenced
  platform-specific definitions are unavailable;
- preserve existing Linux and macOS code paths byte-for-byte where practical;
- preserve Windows fail-closed behavior for unsupported world-worker operations;
- do not make Linux-only dispatch helpers callable on Windows;
- do not replace missing authority with defaults, ambient state, or placeholder values;
- do not add public APIs, compatibility shims, feature flags, manifests, or dependencies; and
- after each repair, rerun the native Windows library build and stop on the first required
  out-of-allowlist change.

For the PowerShell proof defect:

- make the regex literal under `Set-StrictMode -Version Latest`;
- match the actual literal `$forwarderRelease = Join-Path $resolvedProject
  'target/release/substrate-forwarder.exe'` source shape;
- retain the assertions for warning disclosure and release build instructions;
- do not weaken, skip, or conditionally bypass `-W2Only`;
- do not invoke `start-forwarder.ps1`; and
- do not change product behavior merely to satisfy the test.

SEMANTIC FREEZE

Preserve:

- all accepted R2-3 platform mapping and typed-carrier behavior;
- Linux/macOS dispatch, worker, registry, and REPL behavior;
- Windows fail-closed unsupported-operation behavior;
- `start-forwarder.ps1` release-first/debug-fallback and all lifecycle/PID/pipe behavior;
- all provisioning, warm, doctor, forwarder, pipe, WSL, VM, process, timeout, stop, cleanup,
  artifact, and deletion behavior;
- replay, shim, trace, policy, schema, protocol, and transport behavior; and
- every R3-owned lifecycle boundary.

No product process, WSL lifecycle, installer, uninstaller, warm script, forwarder launch, pipe
client, service mutation, PID action, or cleanup action is authorized during proof.

TEST-DRIVEN PROOF

Run at minimum on native Windows:

- PowerShell parser validation for `scripts/windows/prefix-mapping-r2-3.Tests.ps1`;
- `pwsh -NoProfile -File scripts/windows/prefix-mapping-r2-3.Tests.ps1 -W2Only`;
- `cargo fmt --all -- --check`;
- `cargo build --locked -p shell --lib`;
- `cargo check --locked -p shell`;
- `cargo test --locked -p shell "windows_tests::" -- --nocapture`;
- `cargo clippy --locked -p shell --all-targets -- -D warnings`, with any pre-existing
  out-of-allowlist baseline separated precisely from subject failures;
- exact source assertions proving the named Windows cfg references resolve or remain unreachable;
- frozen-source hashing or exact diff proof for the inspected `start-forwarder.ps1`
  release/debug block and all unrelated allowed-file behavior;
- `git diff --check`;
- exact unstaged/staged allowlist checks; and
- pre-publication `gitnexus_detect_changes()` with every reported affected flow inspected.

If the filtered shell test build exposes a distinct, pre-existing test-only cfg defect outside the
allowlist after the production library build succeeds, report it honestly. It does not authorize
editing another file or weakening the required command; use `BLOCKED_SCOPE_EXPANSION` when that
surface is required for the increment's proof.

SUBJECT FINGERPRINT AND BOUNDED REVIEW

After the final subject settles:

1. Build a deterministic manifest containing the expected base plus each allowed staged path's
   mode, blob, stage, and pathname.
2. SHA-256 that manifest and use `sha256:<digest>` as the subject fingerprint.
3. Run fresh read-only review lenses covering:
   - Windows cfg/build correctness and preservation of Linux/macOS behavior;
   - fail-closed authority and unsupported-operation behavior;
   - PowerShell literal-regex honesty and unchanged `start-forwarder.ps1` behavior; and
   - exact allowlist, proof honesty, and non-activation of lifecycle surfaces.
4. Remediate every valid P1/P2 within scope, recompute the fingerprint, and run a different-fresh
   closure review.
5. Causally disposition every P3/P4 against the existing finding inventory when applicable.
6. Validate the final review-cycle JSON and require zero open P1/P2 before publication.

STOP CONDITIONS

Stop with the matching blocked receipt if:

- the base, tree, target ref, ancestry, cleanliness, or 0/0 gate differs;
- a required repair lies outside the exact four-file allowlist;
- the repair changes Linux/macOS behavior, weakens Windows fail-closed behavior, adds ambient
  authority, or changes a public API;
- any platform lifecycle, process, pipe, PID, service, provisioning, cleanup, artifact, or
  deletion behavior becomes reachable or changes;
- required native checks cannot run honestly;
- required GPT-5.4 Extra High review subagents cannot be created; or
- review ends with any open P1/P2.

PUBLICATION AND RECEIPT

Before publication, fetch/query the live target and require it still equals commit
`58462482126609ad87e59b896f5d22437461d144` with tree
`131cb88fcd22cf739eac5c29a2d6e7d3dded43f0`. Stage only the exact authorized files, inspect the
staged diff, rerun the allowlist, proof, fingerprint, and review gates, create one Conventional
Commit, and normal fast-forward push `HEAD` to
`refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`. Never force-push.

Verify the live remote equals the landed commit/tree, refresh GitNexus on the landed tree, restore
only analyzer-generated count churn, and finish with a clean task worktree and 0/0 divergence.

Send one `codex.top-level-task-receipt.v1` terminal message to the meta task. A successful receipt
must use `LANDED_CLEAN`, echo the bound task identity and nonce, include expected/landed
commit/tree, exact changed paths, subject fingerprint, validated review record/digest and finding
disposition, native checks and limitations, GitNexus result, clean status, the exact narrow
completion boundary, and `next_increment: EVIDENCE:R2-3Z`.

The send to the meta task must be the final tool action. Do not dispatch evidence and do not begin
or render R2-3Z.

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

Do not generate the next increment prompt and do not begin EVIDENCE:R2-3Z. The meta
orchestrator owns independent verification and subsequent dispatch.
