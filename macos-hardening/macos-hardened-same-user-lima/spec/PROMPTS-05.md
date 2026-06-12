# PROMPTS-05: Packet Orchestration Prompts For Slice 05

Source spec:
- [`SPEC-05-backend-policy-input-parity.md`](./SPEC-05-backend-policy-input-parity.md)

Source plan:
- [`PLAN-05.md`](./PLAN-05.md)

Source tasks:
- [`TASKS-05.md`](./TASKS-05.md)

Current branch at prompt authoring time: `HEAD`  
Worker implementation skill:
`/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill:
`/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is
grounded only in the live Slice `05` spec/plan/tasks stack, the current
feature-local macOS hardening docs, and the current repo truth that Slice `05`
needs.

Because Slice `05` is a repo-first backend-contract and parity slice, these
prompts emphasize:

1. bounded shared-contract and backend scope,
2. explicit GitNexus impact / detect-changes gates,
3. shell/broker-resolved policy truth remaining authoritative,
4. strict handoff boundaries to Slice `06`,
5. commit discipline between implementation, review, and fix rounds.

## Packet 1 Prompt

```text
/goal Land Slice 05 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Mission:
- Land Packet 1 only: Carrier contract freeze and symbol-impact gate.
- Do not start Packet 2.
- Keep the slice bounded to freezing the backend-facing parity carrier seam and
  surfacing the exact blast radius before code edits begin.

Before editing:
1. Read SPEC-05, PLAN-05, TASKS-05, EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 1 README, milestone 1.2, milestone 1.3, DESIGN-macos-policy-input-parity.md, DESIGN-macos-lima-transport-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
2. Inspect the current repo-truth evidence in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-api/src/lib.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/lib.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/policy_snapshot.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/repl/async_repl.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/replay/src/replay/executor.rs
3. Run the Packet 1 GitNexus gate using `GITNEXUS_HOME=/tmp/gitnexus-ff74-only`. If the index is stale, refresh it before continuing.
4. Stay within the default execution boundary. Do not widen into Packet 2 contract changes, Packet 3 backend implementation, or Slice 06 docs/script cutover.

Packet 1 scope:
- Task 1.1: Confirm the authority stack, repo-floor gap, and symbol-impact gate.
- Task 1.2: Freeze the backend-facing parity carrier boundary.

Out of scope:
- Packet 2, Packet 3, or Packet 4 work
- transport-contract redesign
- helper scripts or top-level docs
- broker policy-language redesign
- broad cross-backend semantic work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- The implementation subagent must run the GitNexus symbol-impact gate before any relevant edits:
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus status`
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus context WorldSpec --repo substrate --file crates/world-api/src/lib.rs`
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus context ExecRequest --repo substrate --file crates/world-api/src/lib.rs`
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus context convert_exec_request --repo substrate --file crates/world-mac-lima/src/lib.rs`
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus context apply_policy --repo substrate --file crates/world-mac-lima/src/lib.rs`
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus impact 'Struct:crates/world-api/src/lib.rs:WorldSpec' --repo substrate --direction upstream --depth 3 --include-tests`
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus impact 'Struct:crates/world-api/src/lib.rs:ExecRequest' --repo substrate --direction upstream --depth 3 --include-tests`
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus impact 'Function:crates/world-mac-lima/src/lib.rs:convert_exec_request' --repo substrate --direction upstream --depth 3 --include-tests`
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus impact 'Function:crates/world-mac-lima/src/lib.rs:apply_policy' --repo substrate --direction upstream --depth 3 --include-tests`
- After implementation, run the Packet 1 verification commands:
  - `rg -n "WorldSpec|ExecRequest|policy_snapshot|world_network|apply_policy|convert_exec_request" crates/world-api/src/lib.rs crates/world-mac-lima/src/lib.rs crates/shell/src/execution/policy_snapshot.rs crates/shell/src/execution/routing/dispatch/world_ops.rs crates/shell/src/repl/async_repl.rs crates/replay/src/replay/executor.rs`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec crates/world-api/src/lib.rs`
  - `git status --short`
- If implementation is green, commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-05 / PLAN-05 / TASKS-05, the GitNexus outputs, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, rerun the relevant GitNexus commands if the touched symbols changed, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not begin Packet 2 until Packet 1 is committed and review-clean.

Packet 1 checkpoint:
- there is one obvious backend-facing parity carrier to aim at
- the affected symbols and fallout surfaces are explicit
- any `HIGH` / `CRITICAL` GitNexus warnings were surfaced before edits
- the slice has not drifted into Packet 2 or Slice 06 work

Implementation subagent prompt:
/goal Land Slice 05 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md first. Also read EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 1 README, milestone 1.2, milestone 1.3, DESIGN-macos-policy-input-parity.md, DESIGN-macos-lima-transport-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md. Work only on Task 1.1 and Task 1.2. Run the Packet 1 GitNexus status/context/impact gate before any relevant edits and surface any `HIGH` or `CRITICAL` impact results explicitly. Keep the work bounded to freezing the carrier seam and documenting the blast radius. Do not widen into Packet 2 contract widening, Packet 3 backend implementation, or Slice 06 readiness/docs work. Re-run the Packet 1 verification commands and finish by stating whether Packet 1 is checkpoint-green, what files changed, what verification ran, what GitNexus reported, and whether Packet 2 is unblocked.

Review subagent prompt:
Review the committed Slice 05 Packet 1 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md, the live diff, and the Packet 1 GitNexus outputs. Review only Packet 1. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 05 Packet 1 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md. Keep fixes limited to Packet 1 findings. Re-run the relevant Packet 1 verification commands and rerun any Packet 1 GitNexus commands needed by the touched symbols. Final message must state which findings were fixed, what verification ran, what GitNexus reported, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- List the exact GitNexus commands run and summarize the impact results.
- State whether the touched file set stayed within Packet 1 scope.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 05 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Mission:
- Land Packet 2 only: Widen the shared backend contract.
- Do not start Packet 3.
- Keep the slice bounded to adding the minimum shared parity carrier and aligning authoritative builders to feed it.

Before editing:
1. Read SPEC-05, PLAN-05, TASKS-05 and verify Packet 1 is already landed, committed, and checkpoint-green on the current tree.
2. Re-read the Phase 1 README, milestone 1.2, milestone 1.3, and DESIGN-macos-policy-input-parity.md.
3. Inspect the current shared-contract and builder surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-api/src/lib.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/lib.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/policy_snapshot.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/repl/async_repl.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/builtins/world_gateway.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/replay/src/replay/executor.rs
4. Stay within the default execution boundary. Do not widen into Packet 3 backend parity implementation or Slice 06 docs/script cutover.

Packet 2 scope:
- Task 2.1: Add the minimum shared parity carrier.
- Task 2.2: Align authoritative builders to the widened carrier without creating a second policy source of truth.

Out of scope:
- Packet 3 or Packet 4 work
- helper scripts or top-level docs
- transport-contract redesign
- broker policy-language redesign
- nonessential cross-backend semantic work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- Before editing any symbol, rerun the relevant Packet 1 GitNexus impact gate for the symbols Packet 2 will touch. Surface any `HIGH` or `CRITICAL` results before proceeding.
- After implementation, run the Packet 2 verification commands:
  - `cargo test -p world-api shared_world_contract_round_trips_with_canonical_shape -- --nocapture`
  - `cargo test -p shell world_network_policy_canonicalizes_snapshot_net_allowed -- --nocapture`
  - `cargo test -p shell world_network_policy_requests_isolation_for_restrictive_allowlist -- --nocapture`
  - `rg -n "WorldSpec|ExecRequest|policy_snapshot|world_network" crates/world-api/src/lib.rs crates/replay/src/replay/executor.rs crates/world-mac-lima/src/lib.rs`
  - `git diff --stat -- crates/world-api/src/lib.rs crates/shell/src/execution/policy_snapshot.rs crates/shell/src/execution/routing/dispatch/world_ops.rs crates/shell/src/repl/async_repl.rs crates/shell/src/builtins/world_gateway.rs crates/replay/src/replay/executor.rs macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
- If implementation is green, commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-05 / PLAN-05 / TASKS-05, the relevant GitNexus outputs, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, rerun any relevant GitNexus impact commands if touched symbols changed, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not begin Packet 3 until Packet 2 is committed and review-clean.

Packet 2 checkpoint:
- the shared backend contract can represent the required parity inputs
- authoritative shell/broker resolution still feeds that contract
- no macOS-only hidden side channel was introduced

Implementation subagent prompt:
/goal Land Slice 05 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md first. Verify Packet 1 is already green. Work only on Task 2.1 and Task 2.2. Keep the work bounded to widening the minimum shared parity carrier and feeding it from authoritative shell/broker-resolved inputs. Do not widen into Packet 3 backend implementation or Slice 06 readiness/docs work. Re-run the Packet 2 verification commands and finish by stating whether Packet 2 is checkpoint-green, what files changed, what verification ran, and whether Packet 3 is unblocked.

Review subagent prompt:
Review the committed Slice 05 Packet 2 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md, the live diff, and the relevant GitNexus outputs. Review only Packet 2. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 05 Packet 2 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the shared carrier is explicit and whether authoritative shell/broker resolution still feeds it.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 05 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Mission:
- Land Packet 3 only: Implement backend parity semantics.
- Do not start Packet 4.
- Keep the slice bounded to removing permissive local policy synthesis, carrying `world_network`, and making `apply_policy(...)` semantically real.

Before editing:
1. Read SPEC-05, PLAN-05, TASKS-05 and verify Packets 1 and 2 are already landed, committed, and checkpoint-green on the current tree.
2. Re-read the Phase 1 README, milestone 1.2, milestone 1.3, and DESIGN-macos-policy-input-parity.md.
3. Inspect the current backend parity surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/lib.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-api/src/lib.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/policy_snapshot.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/repl/async_repl.rs
4. Rerun the relevant GitNexus impact gate before touching the Packet 3 symbols. If impact is `HIGH` or `CRITICAL`, surface it before proceeding.
5. Stay within the default execution boundary. Do not widen into Packet 4 final validation-only work except where final cleanup is strictly required, and do not widen into Slice 06 docs/script cutover.

Packet 3 scope:
- Task 3.1: Remove synthetic permissive snapshot generation from `convert_exec_request(...)`.
- Task 3.2: Make `apply_policy(...)` semantically real without widening into docs or transport redesign.

Out of scope:
- Packet 4 work except tiny cleanup strictly required by Packet 3
- helper scripts or top-level docs
- transport redesign
- broker policy-language redesign
- later readiness/docs cutover work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- After implementation, run the Packet 3 verification commands:
  - `cargo test -p world-mac-lima convert_exec_request_propagates_env_fs_mode -- --nocapture`
  - `cargo test -p world-mac-lima -- --nocapture`
  - `rg -n "convert_exec_request|PolicySnapshotV3|world_network|shared_world" crates/world-mac-lima/src/lib.rs`
  - `rg -n "apply_policy|fs_mode|policy_snapshot|world_network" crates/world-mac-lima/src/lib.rs`
  - `git diff --stat -- crates/world-mac-lima/src/lib.rs crates/world-api/src/lib.rs crates/shell/src/execution/policy_snapshot.rs crates/shell/src/execution/routing/dispatch/world_ops.rs crates/shell/src/repl/async_repl.rs macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
- If implementation is green, commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-05 / PLAN-05 / TASKS-05, the relevant GitNexus outputs, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, rerun any relevant GitNexus impact commands if touched symbols changed, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not begin Packet 4 until Packet 3 is committed and review-clean.

Packet 3 checkpoint:
- backend-mediated macOS execution no longer relies on permissive local policy synthesis
- `world_network` and related parity inputs no longer disappear on the backend path
- `apply_policy(...)` is no longer semantically empty
- Slice 06 docs/script cutover remains untouched

Implementation subagent prompt:
/goal Land Slice 05 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md first. Verify Packets 1 and 2 are already green. Work only on Task 3.1 and Task 3.2. Keep the work bounded to backend parity semantics in `world-mac-lima` plus any narrowly required shared-carrier fallout. Do not widen into Slice 06 readiness/docs work. Re-run the Packet 3 verification commands and finish by stating whether Packet 3 is checkpoint-green, what files changed, what verification ran, and whether Packet 4 is unblocked.

Review subagent prompt:
Review the committed Slice 05 Packet 3 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md, the live diff, and the relevant GitNexus outputs. Review only Packet 3. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 05 Packet 3 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md. Keep fixes limited to Packet 3 findings. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether `convert_exec_request(...)` still synthesizes local policy or not.
- State whether `apply_policy(...)` is still semantically empty or not.
- State whether Packet 4 is unblocked.
- If anything is not green, say explicitly that Packet 4 must not begin.
```

## Packet 4 Prompt

```text
/goal Land Slice 05 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Mission:
- Land Slice 05 Packet 4 only: Final validation and next-slice handoff clarity.
- Do not reopen Packets 1, 2, or 3 except where strictly required by final cleanup.
- Keep the slice bounded to final validation, GitNexus change-scope verification, and explicit Slice 06 handoff clarity.

Before editing:
1. Read SPEC-05, PLAN-05, TASKS-05 and verify Packets 1, 2, and 3 are already landed, committed, and checkpoint-green on the current tree.
2. Re-read EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 1 README, milestone 1.2, milestone 1.3, and the full Slice 05 doc set.
3. Confirm the default next seam is still Slice 06 and that Slice 05 remains bounded to backend policy parity.
4. Stay within the default execution boundary unless a contradiction forces escalation.

Packet 4 scope:
- Task 4.1: Final targeted regression and GitNexus scope check.
- Task 4.2: Validate explicit deferral to Slice 06.

Out of scope:
- new backend parity design changes beyond what review-cleanup strictly requires
- helper scripts or top-level docs cutover work
- transport redesign
- later phase hardening work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- After implementation, run the Packet 4 verification commands:
  - `cargo fmt --all -- --check`
  - `cargo test -p world-api shared_world_contract_round_trips_with_canonical_shape -- --nocapture`
  - `cargo test -p world-mac-lima -- --nocapture`
  - `cargo test -p shell world_network_policy_canonicalizes_snapshot_net_allowed -- --nocapture`
  - `cargo test -p shell world_network_policy_requests_isolation_for_restrictive_allowlist -- --nocapture`
  - `git diff --stat -- crates/world-api/src/lib.rs crates/world-mac-lima/src/lib.rs crates/shell/src/execution/policy_snapshot.rs crates/shell/src/execution/routing/dispatch/world_ops.rs crates/shell/src/repl/async_repl.rs crates/shell/src/builtins/world_gateway.rs crates/replay/src/replay/executor.rs macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
  - `gitnexus_detect_changes()`
  - `rg -n "Slice 06|doctor|smoke|readiness|docs cutover|policy parity" macos-hardening/macos-hardened-same-user-lima/spec`
- If implementation is green, commit the Packet 4 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-05 / PLAN-05 / TASKS-05, the GitNexus detect-changes output, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, rerun `gitnexus_detect_changes()`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not declare Slice 05 done until Packet 4 is committed and review-clean.

Packet 4 checkpoint:
- Slice 05 stayed bounded
- GitNexus scope verification is consistent with the intended shared-contract and backend-parity seam
- Slice 06 remains the next honest readiness/docs seam

Implementation subagent prompt:
/goal Land Slice 05 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md first. Verify Packets 1, 2, and 3 are already green. Work only on Task 4.1 and Task 4.2. Keep any edits limited to final validation cleanup and explicit Slice 06 handoff clarity. Re-run the Packet 4 verification commands and finish by stating whether Slice 05 is checkpoint-green, what files changed, what verification ran, what GitNexus detect-changes reported, and whether Slice 06 is unblocked.

Review subagent prompt:
Review the committed Slice 05 Packet 4 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md. Review only Packet 4 and the live diff. Report findings first with explicit severities. State clearly whether Slice 05 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 05 Packet 4 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md. Keep fixes limited to Packet 4 findings. Re-run the relevant Packet 4 verification commands and `gitnexus_detect_changes()`. Final message must state which findings were fixed, what verification ran, whether Slice 05 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Slice 05 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State what `gitnexus_detect_changes()` reported.
- State whether Slice 06 is unblocked.
- If anything is not green, say explicitly that Slice 06 must not begin.
```
