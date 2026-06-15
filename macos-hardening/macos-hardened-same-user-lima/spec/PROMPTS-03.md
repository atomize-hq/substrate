# PROMPTS-03: Packet Orchestration Prompts For Slice 03

Source spec:
- [`SPEC-03-canonical-guest-endpoint-and-transport-contract.md`](./SPEC-03-canonical-guest-endpoint-and-transport-contract.md)

Source plan:
- [`PLAN-03.md`](./PLAN-03.md)

Source tasks:
- [`TASKS-03.md`](./TASKS-03.md)

Current branch at prompt authoring time: `HEAD`  
Worker implementation skill:
`/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill:
`/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is
grounded only in the live Slice `03` spec/plan/tasks stack, the current
feature-local macOS hardening docs, the relevant runtime code surfaces, and the
official Lima source set that Slice `03` requires.

Because Slice `03` is a source-driven runtime-contract slice, these prompts
emphasize:

1. bounded transport-contract scope,
2. official-Lima source verification before transport decisions,
3. mandatory GitNexus impact analysis before symbol edits,
4. mandatory `gitnexus_detect_changes()` before commits,
5. strict handoff boundaries to Slice `04`,
6. commit discipline between implementation, review, and fix rounds.

## Packet 1 Prompt

```text
/goal Land Slice 03 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/config/vmtype/vz/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Packet 1 only: shared transport authority and stale-constant freeze.
- Do not start Packet 2.
- Keep the slice bounded to source-backed transport authority centralization in world-mac-lima.

Before editing:
1. Read SPEC-03, PLAN-03, TASKS-03, EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 1 README, milestone 1.1, DESIGN-macos-lima-transport-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
2. Inspect current repo-truth drift in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/transport.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/forwarding.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/lib.rs
3. Verify the official Lima sources above before freezing any transport-contract claim.
4. Run GitNexus impact analysis before editing any touched symbol. If the index is stale, refresh it first.
5. Stay within the default execution boundary. Do not widen into shell doctor/readiness, top-level macOS docs, or Slice 04 consumer convergence.

Packet 1 scope:
- Task 1.1: Confirm the authority stack, source gate, and symbol-impact gate.
- Task 1.2: Centralize the canonical transport authority in world-mac-lima.

Out of scope:
- Packet 2 or Packet 3 work
- shell doctor/readiness convergence
- top-level macOS docs rewrites
- backend policy parity or later hardening work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- The implementation subagent must run GitNexus impact analysis before symbol edits.
- After implementation, run the Packet 1 verification commands:
  - `rg -n "7788|17788|/run/substrate.sock|agent.sock" crates/world-mac-lima/src`
  - `cargo test -p world-mac-lima -- --nocapture`
  - `git diff --stat -- crates/world-mac-lima macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
- If implementation is green, run `gitnexus_detect_changes()` and commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-03 / PLAN-03 / TASKS-03, the official Lima sources, the GitNexus impact output, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, rerun `gitnexus_detect_changes()`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not begin Packet 2 until Packet 1 is committed and review-clean.

Packet 1 checkpoint:
- there is one obvious transport authority for the macOS backend
- stale `7788` is gone from the centralized transport surface
- the slice has not drifted into Slice 04

Implementation subagent prompt:
/goal Land Slice 03 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md first. Also read EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 1 README, milestone 1.1, DESIGN-macos-lima-transport-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md. Verify the official Lima source set before freezing any transport decision. Run GitNexus impact analysis before editing touched symbols. Work only on Task 1.1 and Task 1.2. Keep the slice limited to centralizing the transport authority in world-mac-lima. Do not widen into Packet 2, shell doctor/readiness work, or top-level macOS docs. Re-run the Packet 1 verification commands and finish by stating whether Packet 1 is checkpoint-green, what files changed, what verification ran, what official sources were used, what GitNexus impact/detect-changes showed, and whether Packet 2 is unblocked.

Review subagent prompt:
Review the committed Slice 03 Packet 1 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md, the official Lima sources, and the GitNexus impact output. Review only Packet 1 and the live diff. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 03 Packet 1 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md. Keep fixes limited to Packet 1 findings. Re-run the relevant Packet 1 verification commands and `gitnexus_detect_changes()`. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- List the official Lima sources used for the final transport wording.
- State what GitNexus impact/detect-changes reported.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 03 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/config/vmtype/vz/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Packet 2 only: backend adoption and minimal shell-facing alignment.
- Do not start Packet 3.
- Keep the slice bounded to removing direct backend/shell transport contradiction.

Before editing:
1. Read SPEC-03, PLAN-03, TASKS-03 and verify Packet 1 is already landed, committed, and checkpoint-green.
2. Re-read milestone 1.1 and DESIGN-macos-lima-transport-contract.md.
3. Inspect the current implementation surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/lib.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/forwarding.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/transport.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/platform_world/mod.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/builtins/world_gateway.rs
4. Run GitNexus impact analysis before editing any touched symbol.
5. Stay within the default execution boundary. Do not widen into `crates/shell/src/execution/platform/macos.rs`, `world_ops.rs`, `world_persistent_session.rs`, or top-level docs unless a direct contradiction forces it.

Packet 2 scope:
- Task 2.1: Remove backend-local transport drift.
- Task 2.2: Align shell-visible transport mapping without absorbing Slice 04.

Out of scope:
- Packet 3 work
- doctor/readiness convergence
- routed consumer convergence beyond the minimum direct contradiction cleanup
- top-level macOS docs rewrites

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- After implementation, run the Packet 2 verification commands:
  - `rg -n "7788|17788|/run/substrate.sock|agent.sock" crates/world-mac-lima/src`
  - `rg -n "7788|17788|agent.sock|SUBSTRATE_WORLD_SOCKET" crates/shell/src/execution/platform_world/mod.rs crates/shell/src/builtins/world_gateway.rs`
  - `cargo test -p world-mac-lima -- --nocapture`
  - `cargo test -p shell macos_gateway_client_endpoint -- --nocapture`
  - `cargo test -p shell unix_and_tcp_transports_format_endpoints -- --nocapture`
  - `git diff --stat -- crates/world-mac-lima crates/shell/src/execution/platform_world/mod.rs crates/shell/src/builtins/world_gateway.rs`
  - `git status --short`
- If implementation is green, run `gitnexus_detect_changes()` and commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-03 / PLAN-03 / TASKS-03, the official Lima sources, the GitNexus impact output, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, rerun `gitnexus_detect_changes()`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not begin Packet 3 until Packet 2 is committed and review-clean.

Packet 2 checkpoint:
- backend and shell-visible transport mappings no longer disagree on the same contract
- any retained `17788` path is explicit compatibility material rather than scattered magic literals
- doctor/readiness convergence still remains Slice 04 work

Implementation subagent prompt:
/goal Land Slice 03 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md first. Verify Packet 1 is already green. Run GitNexus impact analysis before symbol edits. Work only on Task 2.1 and Task 2.2. Keep the edits limited to backend drift removal plus minimal shell-facing alignment. Do not widen into Packet 3, shell doctor/readiness convergence, or top-level macOS docs. Re-run the Packet 2 verification commands and finish by stating whether Packet 2 is checkpoint-green, what files changed, what verification ran, what GitNexus impact/detect-changes showed, and whether Packet 3 is unblocked.

Review subagent prompt:
Review the committed Slice 03 Packet 2 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md. Review only Packet 2 and the live diff, using the official Lima sources and GitNexus impact output as authority. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 03 Packet 2 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands and `gitnexus_detect_changes()`. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State what GitNexus impact/detect-changes reported.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 03 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/config/vmtype/vz/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Slice 03 Packet 3 only: targeted validation and next-slice handoff clarity.
- Do not reopen Packet 1 or Packet 2 except where strictly required by final cleanup.
- Keep the slice bounded to test validation, GitNexus scope verification, and a clean Slice 04 handoff.

Before editing:
1. Read SPEC-03, PLAN-03, TASKS-03 and verify Packets 1 and 2 are already landed, committed, and checkpoint-green.
2. Re-read EXECUTION-RUBRIC.md, ROADMAP.md, milestone 1.1, and the full Slice 03 doc set.
3. Confirm the default next seam is still Slice 04 and that top-level operator-doc cutover remains later work.
4. Stay within the default feature-local planning boundary unless a contradiction forces escalation.

Packet 3 scope:
- Task 3.1: Final targeted regression and GitNexus scope check.
- Task 3.2: Validate explicit deferral to Slice 04.

Out of scope:
- new transport-design changes beyond what review-cleanup strictly requires
- doctor/readiness implementation work
- top-level docs cutover
- later phase hardening work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- After implementation, run the Packet 3 verification commands:
  - `cargo fmt --all -- --check`
  - `cargo test -p world-mac-lima -- --nocapture`
  - `cargo test -p shell macos_gateway_client_endpoint -- --nocapture`
  - `cargo test -p shell unix_and_tcp_transports_format_endpoints -- --nocapture`
  - `git diff --stat -- crates/world-mac-lima crates/shell/src/execution/platform_world/mod.rs crates/shell/src/builtins/world_gateway.rs macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
  - `gitnexus_detect_changes()`
  - `rg -n "Slice 04|doctor|readiness|PTY|non-PTY|docs cutover" macos-hardening/macos-hardened-same-user-lima/spec`
- If implementation is green, commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-03 / PLAN-03 / TASKS-03, the official Lima sources, the GitNexus outputs, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, rerun `gitnexus_detect_changes()`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not declare Slice 03 done until Packet 3 is committed and review-clean.

Packet 3 checkpoint:
- Slice 03 stayed bounded
- GitNexus scope verification is consistent with the intended transport seam
- Slice 04 remains the next honest consumer-convergence slice

Implementation subagent prompt:
/goal Land Slice 03 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md first. Verify Packets 1 and 2 are already green. Work only on Task 3.1 and Task 3.2. Keep any edits limited to final validation cleanup and explicit Slice 04 handoff clarity. Re-run the Packet 3 verification commands and finish by stating whether Slice 03 is checkpoint-green, what files changed, what verification ran, what GitNexus detect-changes reported, and whether Slice 04 is unblocked.

Review subagent prompt:
Review the committed Slice 03 Packet 3 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md. Review only Packet 3 and the live diff. Report findings first with explicit severities. State clearly whether Slice 03 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 03 Packet 3 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md. Keep fixes limited to Packet 3 findings. Re-run the relevant Packet 3 verification commands and `gitnexus_detect_changes()`. Final message must state which findings were fixed, what verification ran, whether Slice 03 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Slice 03 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State what GitNexus detect-changes reported.
- State whether Slice 04 is unblocked.
- If anything is not green, say explicitly that Slice 04 must not begin.
```
