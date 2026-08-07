# AUX-R3-MAC-EVIDENCE-RECOVERY-R2 contract

## Selected outcome

Implement and remotely publish **R2 only** from the landed recovery plan at `bb098b6e265a4b6cadc4b7ca908c8b6ccf59f2b9`. Admit only
the named existing shell/runtime leaves under macOS cfg, make the macOS installer build and copy
exactly `substrate-lifecycle-control` and `substrate-lifecycle-macos` to fixed managed prefix paths,
and prove ordinary Linux host behavior and all Windows source behavior remain unchanged. Do not
begin or partially implement R3, R4, R5, R6, installation, host preparation, or native evidence.

## Authoritative source and skills

Read from the exact expected base before work: `AGENTS.md`, the complete
`llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/{SPEC,PLAN,TASKS}.md`, control-pack files
`03` through `06`, and `review-control/README.md` plus `validate_review_cycle.py`.

Load `.agents/skills/using-agent-skills/SKILL.md` first and apply at minimum
`source-driven-development`, `test-driven-development`, `incremental-implementation`,
`debugging-and-error-recovery`, `code-review-and-quality`, `security-and-hardening`, and
`git-workflow-and-versioning`. Record each loaded path and constraint. Use repository-local skills
only.

## Exact donor boundary

The preserved donor `/Users/spensermcconnell/.codex/worktrees/60cc/substrate` is read-only evidence:

- base/tree: `d8a65fc8890dd37584aaeac2984c906188e5f06e` /
  `d8f25cc993f264f0c67fe3e00180eb393e41b2e3`;
- tracked binary-diff SHA-256:
  `39682fd4415a00ae4099580135a687d3e11858f62f84e9ef6ff3771bafd4b54c`;
- donor `tests/mac/dev_install_compile_surface_r3.sh` SHA-256:
  `4ddc3cbb9758d84b2dd1529a6f9896d108934ac106ff4a197f134e869147b06c`;
- donor `tests/installers/dev_install_bash32_fd_regression.sh` SHA-256:
  `2de2da832cbb38c1722d2075ffcef781d7bb9d23a76318cfd6d3b54fee0b0664`.

Re-hash and inventory it before use. Never mutate, stage, restore, checkout, reset, clean, commit,
archive, or execute tests in the donor. Manually recreate only R2 lines in the fresh task worktree.
Never overwrite the landed R1 descriptor implementation or transplant a donor file wholesale.

## Exact production allowlist

Only these paths and named symbol/cfg families may change:

1. `crates/shell/src/builtins/world_deps/mod.rs` —
   `resolve_current_inventory_view`, exact macOS compile borrow correction only;
2. `crates/shell/src/execution/agent_runtime/mod.rs` — existing
   `world_work_execution_supervisor` module cfg admits macOS only;
3. `crates/shell/src/execution/agent_runtime/state_store.rs` — cfg admission only for the existing
   `SharedWorldMetadataV1`, `ResolvedInternal*`, `WorldWork*`, and
   `BoundAgentRuntimeStateStore*` family; no body/model rewrite;
4. `crates/shell/src/execution/managed_lifecycle/linux_client.rs` —
   `set_close_on_exec_v1` non-Linux fallback only; the Linux implementation/body is byte-identical;
5. `crates/shell/src/execution/orchestrator_world_dispatch.rs` — macOS cfg admission only for
   `PreparedOrchestratorWorldDispatch`, `prepare_orchestrator_world_dispatch`, and the existing
   authority/receipt helper family; bodies unchanged;
6. `crates/shell/src/execution/platform/macos.rs` — exact cfg-local compile/test branch correction;
7. `crates/shell/src/repl/async_repl.rs` — existing `start_remote_member_runtime` cfg admits macOS
   only; body unchanged;
8. `scripts/substrate/dev-install-substrate.sh` — only:
   - MAC `BUILD_FLAGS` appending `--bin substrate-lifecycle-control --bin substrate-lifecycle-macos`;
   - `stage_managed_mac_control_binary_copy` for fixed regular-file copy/managed-manifest handling;
   - `MANAGED_MAC_CONTROL_BINARIES_PATH`; and
   - the MAC-only loop copying exactly those two binaries.

The installer helper/variable/loop clarification above is already-authorized R2 fixed managed-copy
scope. It authorizes no third binary, install action, lifecycle request, selector, endpoint, or
bootstrap behavior.

## Exact test allowlist

- `tests/mac/dev_install_compile_surface_r3.sh` — only exact macOS compile/build-flag assertions
  for the named host and two lifecycle binaries;
- `tests/installers/dev_install_bash32_fd_regression.sh` — preserve every R1 assertion and add only
  exact fake-build/fixed-copy/managed-manifest assertions for those two lifecycle binaries.

## Exact review metadata allowlist

- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r2-review-cycle-record.json`;
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r2-review-authority-security.md`;
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r2-review-lifecycle-convergence.md`;
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r2-review-allowlist-evidence.md`.

The exact final changed-path set is the ten production/test paths plus these four review artifacts.
`06-review-finding-inventory.md` may change only to add/deduplicate a valid unfixed P3/P4; otherwise
it remains byte-identical. Every other path is frozen.

## Hard stops and platform boundary

All common pairing contracts, managed-lifecycle pairing APIs, macOS client, lifecycle binaries,
Cargo manifests/lockfile, Keychain/XPC/Lima/publisher/session code, R3 control docs, evidence files,
`.agents`, orchestration state, and all R3-R6 paths/symbols are frozen. Every Windows client/file
must be byte-identical. Ordinary Linux-host cfg branches and behavior must be byte-identical. No
new dependency, Windows signature adaptation, Linux pairing stub, guest command, real install,
native action, or host mutation is authorized.

Before every existing symbol edit, run GitNexus upstream impact analysis and record callers,
processes, and risk. HIGH/CRITICAL/UNKNOWN results confined solely to the exact named cfg-admission
symbols are anticipated review obligations under amendment 0011; supplement them with complete
manual callers/cfg analysis. Any required unlisted path, symbol, behavior, authority root, or
platform surface returns `BLOCKED_SCOPE_EXPANSION`.

## Required deterministic proof

Run and record at minimum:

1. `/bin/bash tests/installers/dev_install_bash32_fd_regression.sh`;
2. `bash tests/mac/dev_install_compile_surface_r3.sh`;
3. `/bin/bash -n scripts/substrate/dev-install-substrate.sh`;
4. `cargo fmt --all -- --check`;
5. the plan-required macOS target checks, including
   `cargo check --target x86_64-apple-darwin` for the touched compile surface and lifecycle bins;
6. focused native-host compile checks when needed to prove the exact macOS cfg leaves;
7. Linux cfg differential/manual review plus focused Linux checks proving the existing Linux
   bodies and ordinary host routes are unchanged;
8. `cargo check --target x86_64-pc-windows-msvc` for touched crates when the configured toolchain
   supports it, plus byte-identical hashes for every forbidden Windows file; if the plan-required
   target toolchain is unavailable, return the exact platform/toolchain stop rather than editing
   around it or claiming the gate passed;
9. `git diff --check`, exact path/symbol allowlist, changed-byte secret/private-data scan, and
   `gitnexus_detect_changes()` before publication.

All tests/builds use disposable target/temp directories where practical. Run no real installation
or native lifecycle action.

## Causal cascading review

Freeze the exact full implementation subject and fingerprint after deterministic checks. Run one
fresh same-fingerprint discovery review/burst with all three read-only lenses:

1. authority/security — cfg authority reachability, fixed-copy identity, overwrite/manifest safety,
   no new caller-controlled authority;
2. lifecycle/convergence — build/copy retry, partial/failure cleanup, managed manifest, unchanged
   Linux lifecycle behavior;
3. allowlist/evidence — exact paths/symbols, platform hashes, target checks, no R3-R6 or native
   substitution.

Remediate valid P1/P2 only in one consolidated pass. Record valid P3/P4 in
`06-review-finding-inventory.md`; they do not trigger or extend the causal cascade and must not be
automatically fixed. Validate `--next-cycle closure`, then use a different fresh read-only closure
reviewer. Permit at most two supplemental cycles only for P1/P2 directly caused or unmasked by the
immediately preceding remediation under unchanged authority/risk. `CLEAN` ends the loop.
Unresolved P1/P2, invalid lineage, or exhausted budget prohibits publication. P3/P4 remain tracked
non-blocking findings under the explicit user review authority.

Every subagent and reviewer uses `gpt-5.6-terra` with Extra High (`xhigh`) reasoning. Editing
subagents receive disjoint ownership where practical; reviewers are fresh, read-only, and never
review their own work. Validate the review JSON after every cycle and before publication.

## Publication and terminal contract

Publication mode is `remote`. Immediately before publication require the live target still equals
`bb098b6e265a4b6cadc4b7ca908c8b6ccf59f2b9` / `688ae2860d99d6d5cf1019e0d3ba27c7cc26b6cc`. If and only if all gates and terminal review are CLEAN, create exactly one
normal commit and fast-forward push it to `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`. Never merge, rebase, reset, clean,
cherry-pick, amend, rewrite, or force-push. Verify remote equality, 0/0 divergence, and a clean task
worktree/index.

Return `LANDED_CLEAN` with `next_increment` exactly `AUX-R3-MAC-EVIDENCE-RECOVERY-R3`. Standing user authority permits
the meta orchestrator—not this task—to dispatch R3 only after independent receipt, task-terminal,
remote, diff, and review verification. On any block preserve the task/worktree byte-for-byte and
return the exact blocked receipt. Do not create or dispatch R3 or any successor.
