# PROMPTS-12: Packet Orchestration Prompts For Slice 12

Source spec:
- [`SPEC-12-breakglass-reclassification-and-doc-cutover.md`](./SPEC-12-breakglass-reclassification-and-doc-cutover.md)

Source plan:
- [`PLAN-12.md`](./PLAN-12.md)

Source tasks:
- [`TASKS-12.md`](./TASKS-12.md)

Current branch at prompt authoring time: `feat/macos-hardening`  
Worker implementation skill:
`/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill:
`/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is
bounded to one packet from Slice `12` and preserves the repo-standard
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

Because Slice `12` is the final macOS same-user Lima docs-cutover and
breakglass-classification seam, these prompts emphasize:

1. consuming the Slice `11` operator matrix as fixed input,
2. leading the default docs path with Substrate-owned commands,
3. classifying guest-admin and bypass flows explicitly as
   `degraded-but-supported` or `breakglass`,
4. preserving the truthful Slice `11` sync/copy stance,
5. keeping the same-user limitation explicit,
6. avoiding any silent widening into a new CLI family or reopened runtime
   design.

## Packet 1 Prompt

```text
/goal Land Slice 12 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/phase-3-substrate-owned-operations/README.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md

Required official source set for this packet:
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/config/mount/

Mission:
- Land Packet 1 only: source gate, live doc inventory, and classification freeze.
- Do not start Packet 2.
- Keep the slice bounded to Task 1.1 and Task 1.2.

Before editing:
1. Read SPEC-12, PLAN-12, TASKS-12, EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 3 README, milestone 3.2, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, DESIGN-macos-ingress-and-mount-contract.md, and DESIGN-macos-lima-transport-contract.md.
2. Inspect the current docs and helper truth in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/USAGE.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
3. Verify the official Lima source set above before freezing any guest-access, SSH, or mount-sensitive claim.
4. Preserve Slice 11’s operator-matrix truth, including that `substrate workspace sync` is not yet the frozen normal macOS sync/copy contract unless live repo proof says otherwise.
5. Stay within Packet 1 scope. Do not widen into broad docs edits yet.

Packet 1 scope:
- Task 1.1: Confirm the authority stack, source gate, and live doc inventory.
- Task 1.2: Freeze the docs-cutover classification direction.

Out of scope:
- Packet 2, Packet 3, or Packet 4 work
- broad edits to `docs/reference/world/platforms/macos-lima-setup.md` or `docs/WORLD.md`
- new CLI/lifecycle family design
- reopening Slice 09, Slice 10, or Slice 11 design decisions beyond narrowly correcting documented repo truth
- Linux or WSL docs/runtime work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- The implementation subagent must keep the work limited to the Slice 12 planning docs and the classification/source-gate freeze.
- After implementation, run the Packet 1 verification commands:
  - `rg -n "limactl shell|SSH|systemctl|journalctl|curl --unix-socket|SUBSTRATE_WORLD_SOCKET|host doctor|world doctor|world gateway|world enable|world deps current sync|workspace sync|supported|degraded-but-supported|breakglass" docs/reference/world/platforms/macos-lima-setup.md docs/WORLD.md docs/USAGE.md scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh scripts/mac/smoke.sh`
  - `rg -n "limactl shell|SSH|Filesystem mounts|workspace sync|breakglass|degraded-but-supported|SUBSTRATE_WORLD_SOCKET" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md`
  - `git status --short`
- If implementation is green, commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-12 / PLAN-12 / TASKS-12, the official source set above, and the live diff.
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
- the live docs drift inventory is explicit
- the supported/degraded/breakglass cutover direction is explicit
- the Slice 11 sync/copy truth is explicitly preserved
- the slice has not widened into broad doc edits

Implementation subagent prompt:
/goal Land Slice 12 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md first. Also read EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 3 README, milestone 3.2, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, DESIGN-macos-ingress-and-mount-contract.md, and DESIGN-macos-lima-transport-contract.md. Verify the official Lima source set before freezing any guest-access, SSH, or mount claim. Work only on Task 1.1 and Task 1.2. Keep the work bounded to the Slice 12 planning docs and the source-gated classification freeze. Do not widen into Packet 2/3/4 or into the broad docs cutover itself. Re-run the Packet 1 verification commands and finish by stating whether Packet 1 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Packet 2 is unblocked.

Review subagent prompt:
Review the committed Slice 12 Packet 1 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md, and the official source set required by Slice 12. Review only Packet 1 and the live diff. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 12 Packet 1 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md. Keep fixes limited to Packet 1 findings. Do not widen scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- List the official Lima sources used for the final classification wording.
- State whether the touched file set stayed within the Packet 1 execution boundary.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 12 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/config/mount/

Mission:
- Land Packet 2 only: primary docs cutover.
- Do not start Packet 3.
- Keep the slice bounded to Task 2.1 and Task 2.2.

Before editing:
1. Read SPEC-12, PLAN-12, TASKS-12 and verify Packet 1 is already landed, committed, and checkpoint-green on the current tree.
2. Re-read the Phase 3 README, milestone 3.2, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, DESIGN-macos-ingress-and-mount-contract.md, and DESIGN-macos-lima-transport-contract.md.
3. Re-inspect the current primary docs surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/USAGE.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/contracts/gateway/operator-contract.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/contracts/gateway/status-schema.md
4. Verify the official Lima source set above before preserving or reclassifying any guest-access or mount-sensitive operator claim.
5. Keep the Slice 11 operator matrix fixed unless a live contradiction is proven.
6. Stay within Packet 2 scope. Do not widen into helper/code alignment or Packet 4 closeout.

Packet 2 scope:
- Task 2.1: Cut over `macos-lima-setup.md` to the owned-path-first narrative.
- Task 2.2: Cut over the macOS sections of `docs/WORLD.md`.

Out of scope:
- Packet 3 or Packet 4 work
- helper/help-text changes outside minimal wording required to state the contract honestly
- new CLI/lifecycle family design
- reopening Slice 11 sync/copy truth
- cross-platform docs cleanup

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- The implementation subagent must keep the work focused on `docs/reference/world/platforms/macos-lima-setup.md` and `docs/WORLD.md`, with `docs/USAGE.md` and the gateway contract docs treated as review-only authorities unless a minimal corrective assist is required.
- After implementation, run the Packet 2 verification commands:
  - `rg -n "host doctor|world doctor|world gateway|world enable|world deps current sync|limactl shell|systemctl|journalctl|curl --unix-socket|breakglass|degraded-but-supported" docs/reference/world/platforms/macos-lima-setup.md`
  - `rg -n "host doctor|world doctor|world gateway|SUBSTRATE_WORLD_SOCKET|limactl shell|journalctl|systemctl|breakglass|degraded-but-supported" docs/WORLD.md`
  - `git diff --stat -- docs/reference/world/platforms/macos-lima-setup.md docs/WORLD.md`
  - `git status --short`
- If implementation is green, commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-12 / PLAN-12 / TASKS-12, the official source set above, and the live diff.
- The review subagent must verify that the docs now lead with the owned path and that breakglass material is clearly separated from the default happy path.
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
- the primary docs lead with the Slice 11 owned path
- breakglass material is no longer mixed into the default happy path
- same-user limitation wording remains explicit
- the slice has not widened into helper/code alignment work

Implementation subagent prompt:
/goal Land Slice 12 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md first. Verify Packet 1 is already green. Work only on Task 2.1 and Task 2.2. Keep the work bounded to the primary docs cutover in `docs/reference/world/platforms/macos-lima-setup.md` and `docs/WORLD.md`. Do not widen into helper/help-text alignment, Packet 4 closeout, or a new CLI family. Re-run the Packet 2 verification commands and finish by stating whether Packet 2 is checkpoint-green, what files changed, what verification ran, and whether Packet 3 is unblocked.

Review subagent prompt:
Review the committed Slice 12 Packet 2 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md, and the official source set required by Slice 12. Review only Packet 2 and the live diff. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 12 Packet 2 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the primary docs now lead with the owned operator path.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 12 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/config/mount/

Mission:
- Land Packet 3 only: helper/help-text alignment and escalation cleanup.
- Do not start Packet 4.
- Keep the slice bounded to Task 3.1 and Task 3.2.

Before editing:
1. Read SPEC-12, PLAN-12, TASKS-12 and verify Packet 2 is already landed, committed, and checkpoint-green on the current tree.
2. Re-read the Phase 3 README, milestone 3.2, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, DESIGN-macos-ingress-and-mount-contract.md, and DESIGN-macos-lima-transport-contract.md.
3. Re-inspect the helper/evidence surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
4. Verify the official Lima source set above before preserving or reclassifying any guest-access or mount-sensitive behavior claim.
5. If `target/debug/substrate` is missing, build it before running the CLI verification commands.
6. Stay within Packet 3 scope. Do not widen into Packet 4 closeout or new runtime/command-family work.

Packet 3 scope:
- Task 3.1: Align helper messaging with the owned-path-first docs contract.
- Task 3.2: Preserve the evidence wall and truthful escalation path.

Out of scope:
- Packet 4 work
- reopening Packet 2 docs decisions except for narrow consistency fixes
- new lifecycle/sync command design
- cross-platform docs/runtime cleanup

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- The implementation subagent must keep the work focused on `scripts/mac/lima-doctor.sh`, `scripts/mac/lima-warm.sh`, and `scripts/mac/smoke.sh`, with the Packet 2 docs used as the contract authority.
- After implementation, run the Packet 3 verification commands:
  - `rg -n "supported|degraded-but-supported|breakglass|host doctor|world doctor|world gateway|world enable|world deps current sync|workspace sync|SUBSTRATE_WORLD_SOCKET" scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh scripts/mac/smoke.sh`
  - `bash -n scripts/mac/lima-doctor.sh`
  - `bash -n scripts/mac/lima-warm.sh`
  - `bash -n scripts/mac/smoke.sh`
  - `target/debug/substrate host doctor --json | jq .`
  - `target/debug/substrate world doctor --json | jq .`
  - `target/debug/substrate world gateway status --json | jq .`
  - `scripts/mac/lima-doctor.sh`
  - `scripts/mac/smoke.sh --gateway-conformance`
  - `git diff --stat -- scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh scripts/mac/smoke.sh`
  - `git status --short`
- If implementation is green, commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-12 / PLAN-12 / TASKS-12, the official source set above, and the live diff.
- The review subagent must verify that the helper messaging matches the docs contract and that routed doctor/gateway/smoke proof remains the primary validation wall.
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
- docs and helper/help text agree on the owned-path-first contract
- breakglass escalation is explicit and consistent
- the validation wall remains routed-first
- the slice has not widened into a new runtime or command-family seam

Implementation subagent prompt:
/goal Land Slice 12 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md first. Verify Packet 2 is already green. Work only on Task 3.1 and Task 3.2. Keep the work bounded to helper/help-text alignment and routed-first escalation cleanup. Do not widen into Packet 4 closeout or a new runtime/command-family seam. Re-run the Packet 3 verification commands and finish by stating whether Packet 3 is checkpoint-green, what files changed, what verification ran, and whether Packet 4 is unblocked.

Review subagent prompt:
Review the committed Slice 12 Packet 3 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md, and the official source set required by Slice 12. Review only Packet 3 and the live diff. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 12 Packet 3 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the validation wall remains routed-first.
- State whether Packet 4 is unblocked.
- If anything is not green, say explicitly that Packet 4 must not begin.
```

## Packet 4 Prompt

```text
/goal Land Slice 12 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/config/mount/

Mission:
- Land Slice 12 Packet 4 only: final verification and Phase 3 closeout.
- Do not widen into any new feature or redesign work.
- Keep the slice bounded to Task 4.1 and Task 4.2.

Before editing:
1. Read SPEC-12, PLAN-12, TASKS-12 and verify Packet 3 is already landed, committed, and checkpoint-green on the current tree.
2. Re-read the Phase 3 README, milestone 3.2, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
3. Inspect the full Slice 12 touched surface and current git state:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
4. Keep the closeout honest: Phase 3 closeout here means docs-cutover and classification convergence, not Linux-equivalent ownership isolation.
5. Stay within Packet 4 scope. Do not widen into new runtime work or new docs families outside the Slice 12 boundary.

Packet 4 scope:
- Task 4.1: Final scope and coherence check.
- Task 4.2: Record the final Phase 3 closeout state honestly.

Out of scope:
- any new feature work beyond Slice 12 closeout
- a new CLI/lifecycle family
- reopened Slice 09/10/11/12 design changes except narrow closeout fixes
- Linux or WSL docs/runtime work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- The implementation subagent must keep the work focused on final Slice 12 coherence, rerun evidence, and explicit Phase 3 closeout wording.
- After implementation, run the Packet 4 verification commands:
  - `git diff --stat -- docs/reference/world/platforms/macos-lima-setup.md docs/WORLD.md scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh scripts/mac/smoke.sh macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md`
  - `git status --short`
  - rerun the Packet 2 verification commands
  - rerun the Packet 3 verification commands
  - manual closeout review
- If implementation is green, commit the Packet 4 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-12 / PLAN-12 / TASKS-12 and the live diff.
- The review subagent must verify that the closeout does not overclaim stronger lifecycle, sync, or ownership guarantees than the repo actually provides.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not claim Slice 12 closed until Packet 4 is committed and review-clean.

Packet 4 checkpoint:
- the final diff stayed within the allowed execution boundary unless a clearly justified minimal assist was required
- the docs cutover is checkpointed against rerun Packet 2 and Packet 3 evidence
- the same-user limitation remains explicit
- the closeout does not overclaim stronger ownership isolation than the repo actually provides

Implementation subagent prompt:
/goal Land Slice 12 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md first. Verify Packet 3 is already green. Work only on Task 4.1 and Task 4.2. Keep the work bounded to final Slice 12 coherence, rerun evidence, and explicit Phase 3 closeout wording. Do not widen into new feature work or stronger ownership claims than the repo actually supports. Re-run the Packet 4 verification commands and finish by stating whether Slice 12 is checkpoint-green, what files changed, what verification ran, whether any drift remains, and whether Phase 3 is honestly closed.

Review subagent prompt:
Review the committed Slice 12 Packet 4 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md. Review only Packet 4 and the live diff. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 12 Packet 4 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the touched file set stayed within the allowed Slice 12 execution boundary.
- State whether any drift remains.
- State whether Phase 3 is honestly closed without overstating the same-user hardening result.
- If anything is not green, say explicitly that Slice 12 is not yet closed.
```
