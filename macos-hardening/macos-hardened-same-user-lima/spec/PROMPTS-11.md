# PROMPTS-11: Packet Orchestration Prompts For Slice 11

Source spec:
- [`SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md`](./SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md)

Source plan:
- [`PLAN-11.md`](./PLAN-11.md)

Source tasks:
- [`TASKS-11.md`](./TASKS-11.md)

Current branch at prompt authoring time: `feat/macos-hardening`  
Worker implementation skill:
`/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill:
`/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is
bounded to one packet from Slice `11` and preserves the repo-standard
implementation -> review -> fix -> commit choreography the user requested:

1. spawn a fresh GPT-5.4 high implementation subagent whose prompt begins with
   `/goal ` and explicitly uses `$incremental-implementation`,
2. commit the implementation changes before review,
3. spawn a fresh GPT-5.4 high review subagent using
   `$code-review-and-quality`,
4. if review flags issues, spawn a fresh GPT-5.4 high fix subagent whose
   prompt begins with `/goal ` and explicitly uses
   `$incremental-implementation`,
5. commit after each fix round before re-review,
6. do not advance to the next packet until the current packet is committed and
   review-clean.

Because Slice `11` is the bounded operator-contract/productization seam after
Slice `10`, these prompts emphasize:

1. keeping the supported path Substrate-first,
2. classifying helper-backed flows honestly as `supported` or
   `degraded-but-supported`,
3. preserving raw guest and bypass flows as `breakglass`,
4. centering routed doctor/gateway/smoke/orchestration evidence,
5. deferring the broad docs/breakglass rewrite to Slice `12`.

## Packet 1 Prompt

```text
/goal Land Slice 11 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/phase-3-substrate-owned-operations/README.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md

Required official source set for this packet:
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/config/mount/

Mission:
- Land Packet 1 only: source gate, live operator inventory, and command-matrix decision.
- Do not start Packet 2.
- Keep the slice bounded to Task 1.1 and Task 1.2.

Before editing:
1. Read SPEC-11, PLAN-11, TASKS-11, EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 3 README, milestone 3.1, milestone 3.2, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-lima-transport-contract.md, and DESIGN-macos-guest-unit-source-of-truth.md.
2. Inspect the current operator-surface repo truth in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/USAGE.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/orchestration-smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/workspace_cmd.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/builtins/world_enable/
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/builtins/world_gateway.rs
3. Verify the official source set above before freezing any guest-access, SSH, or mount-sensitive operator claim.
4. Stay within Packet 1 scope. Do not widen into Packet 2 contract edits, Packet 3 productization/code work, or Packet 4 closeout/handoff work.

Packet 1 scope:
- Task 1.1: Confirm the authority stack, source gate, and live operator inventory.
- Task 1.2: Freeze the operator matrix and sync/lifecycle classification direction.

Out of scope:
- Packet 2, Packet 3, or Packet 4 work
- edits outside the Slice 11 planning docs unless a minimal corrective assist is proven mandatory
- full docs/setup/troubleshooting cutover
- reopening Slice 09 ingress design or Slice 10 unit design
- Linux or WSL owned-operations work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- The implementation subagent must keep the work limited to the Slice 11 planning docs and the source-gated operator-matrix freeze.
- After implementation, run the Packet 1 verification commands:
  - `rg -n "host doctor|world doctor|world gateway (sync|status|restart)|world enable|workspace sync|SUBSTRATE_WORLD_SOCKET|limactl shell|systemctl|journalctl" docs/USAGE.md docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh scripts/mac/smoke.sh scripts/mac/orchestration-smoke.sh crates/shell/src/execution/workspace_cmd.rs crates/shell/src/builtins/world_enable crates/shell/src/builtins/world_gateway.rs`
  - `rg -n "supported|degraded-but-supported|breakglass|workspace sync|world enable|SUBSTRATE_WORLD_SOCKET|limactl shell|world gateway|world doctor|host doctor" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md`
  - `git status --short`
- If implementation is green, commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-11 / PLAN-11 / TASKS-11, the official source set above, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not begin Packet 2 until Packet 1 is committed and review-clean.

Packet 1 checkpoint:
- the targeted official source gate is explicit
- the live operator inventory is explicit
- the owned/degraded/breakglass matrix is explicit
- the normal sync/copy direction is explicit
- the slice has not widened into implementation changes outside the planning docs

Implementation subagent prompt:
/goal Land Slice 11 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md first. Also read EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 3 README, milestone 3.1, milestone 3.2, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-lima-transport-contract.md, and DESIGN-macos-guest-unit-source-of-truth.md. Verify the official Lima source set before freezing any guest-access, SSH, or mount claim. Work only on Task 1.1 and Task 1.2. Keep the work bounded to the Slice 11 planning docs and the source-gated operator-matrix freeze. Do not widen into Packet 2/3/4 or into implementation changes outside the planning docs. Re-run the Packet 1 verification commands and finish by stating whether Packet 1 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Packet 2 is unblocked.

Review subagent prompt:
Review the committed Slice 11 Packet 1 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md, and the official source set required by Slice 11. Review only Packet 1 and the live diff. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 11 Packet 1 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md. Keep fixes limited to Packet 1 findings. Do not widen scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- List the official Lima sources used for the final operator-matrix wording.
- State whether the touched file set stayed within the Packet 1 execution boundary.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 11 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/config/mount/

Mission:
- Land Packet 2 only: owned command matrix and contract consolidation.
- Do not start Packet 3.
- Keep the slice bounded to Task 2.1 and Task 2.2.

Before editing:
1. Read SPEC-11, PLAN-11, TASKS-11 and verify Packet 1 is already landed, committed, and checkpoint-green on the current tree.
2. Re-read the Phase 3 README, milestone 3.1, milestone 3.2, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-lima-transport-contract.md, and DESIGN-macos-guest-unit-source-of-truth.md.
3. Re-inspect the current contract and helper surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/USAGE.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/contracts/gateway/operator-contract.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
4. Verify the official Lima source set above before preserving or reclassifying any guest-access or mount-sensitive operator claim.
5. Stay within Packet 2 scope. Do not widen into Packet 3 code/productization work, Packet 4 closeout, or the full Slice 12 docs rewrite.

Packet 2 scope:
- Task 2.1: Consolidate the primary owned command contract.
- Task 2.2: Reclassify helper-backed lifecycle surfaces honestly.

Out of scope:
- Packet 3 or Packet 4 work
- broad docs/setup/troubleshooting rewrites
- code changes under `crates/shell/` unless a narrow corrective assist is required to keep the Packet 2 contract honest
- reopening Slice 09 ingress or Slice 10 unit decisions
- Linux or WSL owned-operations work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- The implementation subagent must keep the work focused on `docs/USAGE.md`, `docs/contracts/gateway/operator-contract.md`, `scripts/mac/lima-doctor.sh`, and `scripts/mac/lima-warm.sh`.
- After implementation, run the Packet 2 verification commands:
  - `rg -n "host doctor|world doctor|world gateway (sync|status|restart)|world enable|workspace sync|supported|degraded-but-supported|breakglass" docs/USAGE.md docs/contracts/gateway/operator-contract.md`
  - `rg -n "supported|degraded-but-supported|breakglass|world doctor|world enable|workspace sync|SUBSTRATE_WORLD_SOCKET" scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh`
  - `bash -n scripts/mac/lima-doctor.sh`
  - `bash -n scripts/mac/lima-warm.sh`
  - `git diff --stat -- docs/USAGE.md docs/contracts/gateway/operator-contract.md scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh`
  - `git status --short`
- If implementation is green, commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-11 / PLAN-11 / TASKS-11, the official source set above, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not begin Packet 3 until Packet 2 is committed and review-clean.

Packet 2 checkpoint:
- the primary operator contract is readable from one owned-command matrix
- helper-backed lifecycle flows no longer displace the owned CLI path
- raw guest access remains outside the default supported path
- the slice has not widened into the full Slice 12 doc rewrite

Implementation subagent prompt:
/goal Land Slice 11 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md first. Verify Packet 1 is already green. Work only on Task 2.1 and Task 2.2. Keep the work bounded to the owned command matrix and helper-surface reclassification. Do not widen into Packet 3 code/productization, Packet 4 closeout, or the full Slice 12 docs rewrite. Re-run the Packet 2 verification commands and finish by stating whether Packet 2 is checkpoint-green, what files changed, what verification ran, and whether Packet 3 is unblocked.

Review subagent prompt:
Review the committed Slice 11 Packet 2 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md, and the official source set required by Slice 11. Review only Packet 2 and the live diff. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 11 Packet 2 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the primary operator contract is readable from one owned-command matrix.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 11 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/config/mount/

Mission:
- Land Packet 3 only: minimal productization and evidence alignment.
- Do not start Packet 4.
- Keep the slice bounded to Task 3.1 and Task 3.2.

Before editing:
1. Read SPEC-11, PLAN-11, TASKS-11 and verify Packet 2 is already landed, committed, and checkpoint-green on the current tree.
2. Re-read the Phase 3 README, milestone 3.1, milestone 3.2, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-lima-transport-contract.md, and DESIGN-macos-guest-unit-source-of-truth.md.
3. Re-inspect the current productization and evidence surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/builtins/world_enable/
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/workspace_cmd.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/platform/macos.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/orchestration-smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
4. Verify the official Lima source set above before preserving or reclassifying any guest-access or mount-sensitive behavior claim.
5. If `target/debug/substrate` is missing, build it before running the CLI verification commands.
6. Stay within Packet 3 scope. Do not widen into Packet 4 closeout, the full Slice 12 narrative rewrite, or a broad new lifecycle command family.

Packet 3 scope:
- Task 3.1: Align normal sync/copy and lifecycle entrypoints with the frozen contract.
- Task 3.2: Center the validation wall on the owned path.

Out of scope:
- Packet 4 work
- full docs/setup/troubleshooting rewrite
- reopening Packet 2 contract decisions except for narrowly required review-critical fixes
- cross-platform owned-operations redesign
- a broad new lifecycle command family when minimal productization of existing surfaces is sufficient

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- The implementation subagent must keep the work focused on `crates/shell/src/builtins/world_enable/`, `crates/shell/src/execution/workspace_cmd.rs`, `crates/shell/src/execution/platform/macos.rs`, `scripts/mac/smoke.sh`, `scripts/mac/orchestration-smoke.sh`, and `scripts/mac/lima-doctor.sh`.
- After implementation, run the Packet 3 verification commands:
  - `cargo test -p shell`
  - `tmp="$(mktemp -d)"; mkdir -p "$tmp/substrate-home/scripts/substrate"; cp scripts/substrate/world-enable.sh "$tmp/substrate-home/scripts/substrate/world-enable.sh"; chmod +x "$tmp/substrate-home/scripts/substrate/world-enable.sh"; target/debug/substrate world enable --home "$tmp/substrate-home" --dry-run; rc=$?; rm -rf "$tmp"; exit $rc`
  - `bin="$(pwd)/target/debug/substrate"; tmp="$(mktemp -d)"; ws="$tmp/ws"; mkdir -p "$ws"; "$bin" workspace init "$ws" >/dev/null && (cd "$ws" && "$bin" workspace sync --dry-run); rc=$?; rm -rf "$tmp"; exit $rc`
  - `bash -n scripts/mac/smoke.sh`
  - `bash -n scripts/mac/orchestration-smoke.sh`
  - `target/debug/substrate host doctor --json | jq .`
  - `target/debug/substrate world doctor --json | jq .`
  - `scripts/mac/smoke.sh --gateway-conformance`
  - `git diff --stat -- crates/shell/src/builtins/world_enable crates/shell/src/execution/workspace_cmd.rs crates/shell/src/execution/platform/macos.rs scripts/mac/smoke.sh scripts/mac/orchestration-smoke.sh scripts/mac/lima-doctor.sh`
  - `git status --short`
- If implementation is green, commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-11 / PLAN-11 / TASKS-11, the official source set above, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not begin Packet 4 until Packet 3 is committed and review-clean.

Packet 3 checkpoint:
- the owned or degraded operator entrypoints are consistent across docs, messaging, and behavior
- the evidence wall is routed-first
- shared-owner/orchestration proof remains supported where already landed
- the slice has not widened into the full Slice 12 narrative rewrite

Implementation subagent prompt:
/goal Land Slice 11 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md first. Verify Packet 2 is already green. Work only on Task 3.1 and Task 3.2. Keep the work bounded to minimal productization and evidence alignment on the existing macOS operator surfaces. Do not widen into Packet 4 closeout, the full Slice 12 rewrite, or a broad new lifecycle command family. Re-run the Packet 3 verification commands and finish by stating whether Packet 3 is checkpoint-green, what files changed, what verification ran, and whether Packet 4 is unblocked.

Review subagent prompt:
Review the committed Slice 11 Packet 3 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md, and the official source set required by Slice 11. Review only Packet 3 and the live diff. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 11 Packet 3 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the evidence wall is routed-first.
- State whether Packet 4 is unblocked.
- If anything is not green, say explicitly that Packet 4 must not begin.
```

## Packet 4 Prompt

```text
/goal Land Slice 11 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/config/mount/

Mission:
- Land Slice 11 Packet 4 only: final verification and Slice 12 handoff.
- Do not widen into a broad new docs pass.
- Keep the slice bounded to Task 4.1 and Task 4.2.

Before editing:
1. Read SPEC-11, PLAN-11, TASKS-11 and verify Packet 3 is already landed, committed, and checkpoint-green on the current tree.
2. Re-read the Phase 3 README, milestone 3.1, milestone 3.2, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
3. Inspect the full Slice 11 touched surface and current git state:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/USAGE.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/contracts/gateway/operator-contract.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/orchestration-smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/builtins/world_enable/
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/workspace_cmd.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/platform/macos.rs
4. Stay within Packet 4 scope. Do not widen into a full Slice 12 docs/breakglass rewrite.

Packet 4 scope:
- Task 4.1: Final scope and coherence check.
- Task 4.2: Record the Slice 12 handoff honestly.

Out of scope:
- any new feature work beyond Slice 11 closeout
- broad macOS setup/troubleshooting rewrite
- relabeling the full macOS docs corpus
- reopening Packet 2 or Packet 3 except for narrow required closeout fixes

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- The implementation subagent must keep the work focused on final Slice 11 coherence, rerun evidence, and explicit Slice 12 handoff wording.
- After implementation, run the Packet 4 verification commands:
  - `git diff --stat -- docs/USAGE.md docs/contracts/gateway/operator-contract.md scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh scripts/mac/smoke.sh scripts/mac/orchestration-smoke.sh crates/shell/src/builtins/world_enable crates/shell/src/execution/workspace_cmd.rs crates/shell/src/execution/platform/macos.rs macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md`
  - `git status --short`
  - rerun the Packet 2 verification commands
  - rerun the Packet 3 verification commands
  - `scripts/mac/lima-doctor.sh`
  - `scripts/mac/smoke.sh`
  - `scripts/mac/orchestration-smoke.sh`
- If implementation is green, commit the Packet 4 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-11 / PLAN-11 / TASKS-11 and the live diff.
- The review subagent must verify that Slice 11 does not overclaim a broader docs cutover than it actually landed.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not claim Slice 11 closed until Packet 4 is committed and review-clean.

Packet 4 checkpoint:
- the final diff stayed within the allowed execution boundary unless a clearly justified minimal assist was required
- Slice 11 claims only the operator-contract/productization seam
- Slice 12 remains explicitly deferred for the broad docs/breakglass cutover
- checkpoint-green status is tied to rerun Packet 2 / Packet 3 evidence

Implementation subagent prompt:
/goal Land Slice 11 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md first. Verify Packet 3 is already green. Work only on Task 4.1 and Task 4.2. Keep the work bounded to final Slice 11 coherence, rerun evidence, and explicit Slice 12 handoff wording. Do not widen into a broad Slice 12 docs/breakglass rewrite. Re-run the Packet 4 verification commands and finish by stating whether Slice 11 is checkpoint-green, what files changed, what verification ran, and whether Slice 12 is now the only remaining seam.

Review subagent prompt:
Review the committed Slice 11 Packet 4 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md. Review only Packet 4 and the live diff. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 11 Packet 4 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the touched file set stayed within the allowed Slice 11 execution boundary.
- State explicitly that Slice 12 is still the remaining broad docs/breakglass cutover seam.
- If anything is not green, say explicitly that Slice 11 is not yet closed.
```
