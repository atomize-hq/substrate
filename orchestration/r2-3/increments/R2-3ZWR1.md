CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3ZWR1

This is the user-authorized narrow Windows repair inserted after R2-3T and before the R2-3Z
native-evidence gate. It owns only the two contradictions recorded in
`orchestration/r2-3/receipts/R2-3Z-WIN-EVIDENCE-diagnostic-2.json`.

NARROW SCOPE AMENDMENT

After the first native-Windows attempt stopped cleanly with
`BLOCKED_SCOPE_EXPANSION`, the user explicitly authorized the two additional production files
identified in `orchestration/r2-3/receipts/R2-3ZWR1-attempt-1.json`. The amendment owns only:

- the private `world_windows_wsl::transport` constant references in
  `crates/shell/src/execution/platform_world/windows.rs`; and
- the Windows-unavailable `anyhow::Context` import needed by
  `crates/shell/src/builtins/world_enable/runner.rs`.

No other scope or authority changes.

SECOND NARROW SCOPE AMENDMENT

After the six-file native-Windows repair pass stopped cleanly with
`BLOCKED_SCOPE_EXPANSION`, the user explicitly authorized the one additional production file
identified in `orchestration/r2-3/receipts/R2-3ZWR1-attempt-3.json`:

- `crates/shell/src/execution/home_bootstrap.rs`, solely to make the existing
  `BootstrapOwner` construction valid when its Unix-only field is absent on Windows.

The existing six-file unstaged WIP in the task-assigned `102c` worktree must be preserved. No
other file, behavior, or authority is added.

THIRD NARROW SCOPE AMENDMENT

After the seven-file native-Windows repair made the product library build and W2Only proof pass,
the required filtered shell test build stopped cleanly with `BLOCKED_SCOPE_EXPANSION`. The user
explicitly authorized the four additional Windows cfg/test-build surfaces identified in
`orchestration/r2-3/receipts/R2-3ZWR1-attempt-4.json`:

- `crates/shell/src/builtins/world_deps/surfaces.rs`, only for the reported Windows test-build
  gating of Unix-only imports/helpers and the corresponding authenticated world-deps test
  surface;
- `crates/shell/src/execution/agent_runtime/host_session_authority/transition_tests.rs`, only for
  the reported Windows test-build gating of Unix-only test support;
- `crates/shell/src/execution/host_inbox_materialization.rs`, only for the reported Windows
  test-build gating of Unix-only test support; and
- `crates/shell/src/execution/pty/io/types.rs`, only for the reported Windows console-handle type
  mismatch in the library test build.

The existing seven-file unstaged WIP in the task-assigned `102c` worktree must be preserved
exactly. This amendment authorizes only the minimum cfg/type corrections required for the
previously mandated `cargo test --locked -p shell "windows_tests::" -- --nocapture` build to
compile and run. It does not authorize suppressing the command, removing tests, changing
production behavior, or widening into any further file.

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
   `$resolvedProject`; and
3. the four explicitly authorized Windows cfg/test-build defects that prevent the required
   filtered shell test command from compiling and reaching its `windows_tests::` bodies.

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
- `crates/shell/src/execution/platform_world/windows.rs`
- `crates/shell/src/builtins/world_enable/runner.rs`
- `crates/shell/src/execution/home_bootstrap.rs`
- `crates/shell/src/builtins/world_deps/surfaces.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/transition_tests.rs`
- `crates/shell/src/execution/host_inbox_materialization.rs`
- `crates/shell/src/execution/pty/io/types.rs`

Test changes are limited to:

- `scripts/windows/prefix-mapping-r2-3.Tests.ps1`
- colocated `#[cfg(test)]` modules already within the ten allowed Rust files, only
  when mechanically necessary to prove the named cfg/build repair.

Do not edit `scripts/windows/start-forwarder.ps1`; the diagnostic proved its release-first,
debug-fallback behavior is correct. Do not edit any other shell source, test, script, manifest,
`Cargo.lock`, control document, generated analyzer file, `AGENTS.md`, or `CLAUDE.md`.

If either the successful native `cargo build --locked -p shell --lib` gate or the required
filtered shell test build requires any additional tracked file beyond these eleven files, stop
with `BLOCKED_SCOPE_EXPANSION`. Report the exact next compiler diagnostic, required file and
symbol, why the eleven-file allowlist cannot repair it, and a copy-ready continuation prompt. Do
not silently widen the allowlist.

PRE-EDIT BARRIER AND IMPACT

Before editing:

1. This is an authorized same-task resume with preserved WIP. Verify the task-assigned Windows
   worktree HEAD is exactly at commit
   `58462482126609ad87e59b896f5d22437461d144`, tree
   `131cb88fcd22cf739eac5c29a2d6e7d3dded43f0`, target ref
   `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`, 0 ahead/0 behind, with
   required ancestor `0f1e147fb735791b44a65099a65167cbdc1803af`. Require exactly the seven
   previously authorized dirty unstaged paths recorded in the attempt-4 receipt and no staged,
   untracked, analyzer-only, or other tracked changes. Do not reset, clean, recreate, reconcile,
   or discard that WIP.
2. Verify the live remote still equals that exact commit and tree.
3. Make the GitNexus index current for the exact task checkout. Restore analyzer-only
   `AGENTS.md`/`CLAUDE.md` churn before implementation.
4. Run file-qualified upstream impact analysis, including tests, for every existing symbol that
   may be edited. Treat graph undercoverage of Windows cfg paths as undercoverage, not proof.
5. Reproduce and record the first actionable diagnostics from:
   - the already-passing `cargo build --locked -p shell --lib`;
   - the already-passing
     `pwsh -NoProfile -File scripts/windows/prefix-mapping-r2-3.Tests.ps1 -W2Only`; and
   - the blocked `cargo test --locked -p shell "windows_tests::" -- --nocapture`.
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
- repair the two `world_windows_wsl::transport` constant references without making the private
  module public or adding a new public API;
- make `anyhow::Context` available on Windows only as required by the existing `.context(...)`
  call, without changing the world-enable operation;
- make the existing `BootstrapOwner` construction valid when its Unix-only field is absent on
  Windows, without adding ambient identity, changing Unix ownership validation, or changing a
  public API;
- in `world_deps/surfaces.rs`, gate only the reported Unix-only imports, helpers, and dependent
  test surface that cannot compile on Windows; do not weaken authenticated world-deps binding on
  any supported platform;
- in `host_session_authority/transition_tests.rs` and `host_inbox_materialization.rs`, gate only
  the reported Unix-only test support that cannot compile on Windows; do not remove or bypass
  Windows-capable assertions;
- in `pty/io/types.rs`, correct only the reported Windows console-handle test-build type mismatch
  while preserving runtime PTY behavior and Unix handle behavior;
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

If the filtered shell test build exposes a distinct cfg/test defect outside the amended allowlist,
report it honestly. It does not authorize editing another file or weakening the required command;
use `BLOCKED_SCOPE_EXPANSION` when that surface is required for the increment's proof.

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
- a required repair lies outside the exact eleven-file allowlist;
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
