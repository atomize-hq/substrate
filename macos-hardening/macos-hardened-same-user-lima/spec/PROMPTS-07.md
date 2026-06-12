# PROMPTS-07: Packet Orchestration Prompts For Slice 07

Source spec:
- [`SPEC-07-remove-default-extra-listener-surface.md`](./SPEC-07-remove-default-extra-listener-surface.md)

Source plan:
- [`PLAN-07.md`](./PLAN-07.md)

Source tasks:
- [`TASKS-07.md`](./TASKS-07.md)

Current branch at prompt authoring time: `HEAD`  
Worker implementation skill:
`/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill:
`/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is
bounded to one packet from Slice `07` and is grounded in the live Slice `07`
spec/plan/tasks stack, the current feature-local macOS hardening docs, the
current repo truth that still carries the extra-listener drift, and the official
Lima sources that Slice `07` requires.

Because Slice `07` is a source-driven listener-surface hardening slice, these
prompts emphasize:

1. bounded listener-default scope,
2. removal of default `SUBSTRATE_AGENT_TCP_PORT=61337` injection,
3. strict separation between guest listener truth and retained host
   compatibility routing,
4. explicit deferral of ingress/mount, guest-unit-source-of-truth, and broad
   docs-cutover work,
5. commit discipline between implementation, review, and fix rounds.

## Packet 1 Prompt

```text
/goal Land Slice 07 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/config/environment-variables/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Packet 1 only: listener-contract freeze and source gate.
- Do not start Packet 2.
- Keep the slice bounded to freezing the socket-only hardened default decision
  and inventorying the remaining script/doc/runtime contradiction surfaces.

Before editing:
1. Read SPEC-07, PLAN-07, TASKS-07, EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 2 README, milestone 2.1, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-guest-unit-source-of-truth.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
2. Inspect the current repo-truth evidence in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/substrate.yaml
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-service/src/lib.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/transport.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
3. Verify the official Lima sources above before freezing any source-sensitive listener or fallback claim.
4. Stay within the default execution boundary. Do not widen into Packet 2 implementation changes, Packet 3 docs changes, or later ingress/unit work.

Packet 1 scope:
- Task 1.1: Confirm the authority stack, source gate, and live listener drift.
- Task 1.2: Freeze the canonical listener decision and scope boundary.

Out of scope:
- Packet 2, Packet 3, or Packet 4 work
- script changes outside planning artifacts
- doc rewrites outside the Slice 07 planning stack
- ingress/mount redesign
- guest-unit source-of-truth redesign
- broad transport redesign

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- The implementation subagent must keep the work limited to the Slice 07 planning docs and source-gated listener freeze.
- After implementation, run the Packet 1 verification commands:
  - `rg -n "SUBSTRATE_AGENT_TCP_PORT|61337|17788|SUBSTRATE_WORLD_SOCKET|/run/substrate.sock" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
- If implementation is green, commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-07 / PLAN-07 / TASKS-07, the official Lima sources, and the live diff.
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
- the hardened default guest listener decision is explicit
- the contradiction surfaces are explicit
- retained host compatibility routing is clearly not the guest listener contract
- the slice has not drifted into Packet 2/3 implementation work

Implementation subagent prompt:
/goal Land Slice 07 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md first. Also read EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 2 README, milestone 2.1, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-guest-unit-source-of-truth.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md. Verify the required official Lima source set before freezing any listener-path claim. Work only on Task 1.1 and Task 1.2. Keep the work bounded to the Slice 07 planning docs and the source-gated listener freeze. Do not widen into warm-script edits, Packet 2 implementation, Packet 3 docs work, or later ingress/unit work. Re-run the Packet 1 verification commands and finish by stating whether Packet 1 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Packet 2 is unblocked.

Review subagent prompt:
Review the committed Slice 07 Packet 1 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md, and the official Lima sources required by Slice 07. Review only Packet 1 and the live diff. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 07 Packet 1 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md. Keep fixes limited to Packet 1 findings. Do not widen scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- List the official Lima sources used for the final listener-contract wording.
- State whether the touched file set stayed within the default execution boundary.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 07 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/config/environment-variables/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Packet 2 only: remove default guest TCP enablement from warm/repair.
- Do not start Packet 3.
- Keep the slice bounded to `scripts/mac/lima-warm.sh` plus only narrowly needed listener-evidence adjustments in `scripts/mac/lima-doctor.sh` or `scripts/mac/smoke.sh`.

Before editing:
1. Read SPEC-07, PLAN-07, TASKS-07 and verify Packet 1 is already landed, committed, and checkpoint-green on the current tree.
2. Re-read milestone 2.1, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-guest-unit-source-of-truth.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
3. Inspect the current listener-bearing surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/substrate.yaml
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-service/src/lib.rs
4. Verify the official Lima sources above before preserving or reclassifying any compatibility or guest-direct listener claim.
5. Stay within the default execution boundary. Do not widen into Packet 3 docs work, transport redesign, or guest-unit source-of-truth redesign.

Packet 2 scope:
- Task 2.1: Remove default `SUBSTRATE_AGENT_TCP_PORT` injection from `scripts/mac/lima-warm.sh`.
- Task 2.2: Keep listener evidence honest after the warm/repair cutover.

Out of scope:
- Packet 3 or Packet 4 work
- template/YAML unification
- `crates/world-mac-lima/` or `crates/world-service/` redesign unless a direct contradiction forces a narrowly justified change
- ingress/mount work
- broad doc rewrites

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- The implementation subagent must keep the guest listener default socket-only and must not silently turn retained host compatibility routing into supported default behavior.
- After implementation, run the Packet 2 verification commands:
  - `bash -n scripts/mac/lima-warm.sh`
  - `bash -n scripts/mac/lima-doctor.sh scripts/mac/smoke.sh`
  - `rg -n "SUBSTRATE_AGENT_TCP_PORT|61337|SUBSTRATE_WORLD_SOCKET|/run/substrate.sock" scripts/mac/lima-warm.sh`
  - `rg -n "61337|SUBSTRATE_AGENT_TCP_PORT|17788|guest-direct|breakglass|compatibility" scripts/mac/lima-doctor.sh scripts/mac/smoke.sh`
  - `git diff --stat -- scripts/mac/lima-warm.sh scripts/mac/lima-doctor.sh scripts/mac/smoke.sh macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
- If implementation is green, commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-07 / PLAN-07 / TASKS-07, the official Lima sources, and the live diff.
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
- the warm/repair path no longer injects `SUBSTRATE_AGENT_TCP_PORT=61337` by default
- `/run/substrate.sock` remains the intended guest listener contract
- any helper evidence still touched does not imply guest TCP is required
- the slice stayed within the Packet 2 implementation boundary

Implementation subagent prompt:
/goal Land Slice 07 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md first. Verify Packet 1 is already green. Work only on Task 2.1 and Task 2.2. Keep the work bounded to `scripts/mac/lima-warm.sh` and only narrowly necessary listener-evidence adjustments in `scripts/mac/lima-doctor.sh` or `scripts/mac/smoke.sh`. Do not widen into Packet 3 docs changes, transport redesign, template unification, or later ingress/unit work. Re-run the Packet 2 verification commands and finish by stating whether Packet 2 is checkpoint-green, what files changed, what verification ran, and whether Packet 3 is unblocked.

Review subagent prompt:
Review the committed Slice 07 Packet 2 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md, and the official Lima sources required by Slice 07. Review only Packet 2 and the live diff. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 07 Packet 2 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the warm/repair path still enables guest TCP by default or not.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 07 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/config/environment-variables/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Packet 3 only: cut over listener-oriented docs.
- Do not start Packet 4.
- Keep the slice bounded to `docs/WORLD.md` and `docs/reference/world/platforms/macos-lima-setup.md` plus only narrowly required Slice 07 planning-stack touchups.

Before editing:
1. Read SPEC-07, PLAN-07, TASKS-07 and verify Packets 1 and 2 are already landed, committed, and checkpoint-green on the current tree.
2. Re-read milestone 2.1, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
3. Inspect the current listener-oriented doc surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
4. Verify the official Lima sources above before preserving or reclassifying any compatibility TCP, SSH, or guest-direct troubleshooting claim.
5. Stay within the default execution boundary. Do not widen into Packet 4 closeout work or later ingress/unit/docs-cutover seams.

Packet 3 scope:
- Task 3.1: Update `docs/WORLD.md` to describe the tightened listener posture honestly.
- Task 3.2: Update macOS setup docs to stop normalizing guest TCP.

Out of scope:
- Packet 4 work
- `scripts/mac/lima-warm.sh` implementation changes
- transport/runtime redesign
- ingress/mount work
- guest-unit source-of-truth redesign
- broad feature-wide docs cutover

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- The implementation subagent must keep guest listener truth distinct from retained host compatibility routing and guest-direct breakglass access.
- After implementation, run the Packet 3 verification commands:
  - `rg -n "17788|61337|SUBSTRATE_AGENT_TCP_PORT|SUBSTRATE_WORLD_SOCKET|/run/substrate.sock|breakglass|compatibility" docs/WORLD.md`
  - `rg -n "17788|61337|TCP|curl --unix-socket|limactl shell|ssh|breakglass|compatibility" docs/reference/world/platforms/macos-lima-setup.md`
  - `git diff --stat -- docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
- If implementation is green, commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-07 / PLAN-07 / TASKS-07, the official Lima sources, and the live diff.
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
- `docs/WORLD.md` no longer implies guest TCP is part of the hardened default
- `docs/reference/world/platforms/macos-lima-setup.md` no longer normalizes guest TCP
- compatibility TCP and guest-direct access are clearly secondary
- the slice stayed within the Packet 3 docs boundary

Implementation subagent prompt:
/goal Land Slice 07 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md first. Verify Packets 1 and 2 are already green. Work only on Task 3.1 and Task 3.2. Keep the work bounded to `docs/WORLD.md` and `docs/reference/world/platforms/macos-lima-setup.md` plus only narrowly required planning-stack touchups. Do not widen into Packet 4 closeout, transport redesign, or later ingress/unit/docs-cutover work. Re-run the Packet 3 verification commands and finish by stating whether Packet 3 is checkpoint-green, what files changed, what verification ran, and whether Packet 4 is unblocked.

Review subagent prompt:
Review the committed Slice 07 Packet 3 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md, and the official Lima sources required by Slice 07. Review only Packet 3 and the live diff. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 07 Packet 3 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether compatibility TCP and guest-direct access are clearly secondary in the touched docs.
- State whether Packet 4 is unblocked.
- If anything is not green, say explicitly that Packet 4 must not begin.
```

## Packet 4 Prompt

```text
/goal Land Slice 07 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/config/environment-variables/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Packet 4 only: final validation and next-slice handoff clarity.
- Do not reopen Packet 1, Packet 2, or Packet 3 except where a strictly required final-scope fix is necessary.
- Keep the slice bounded to final verification, scope policing, and honest handoff language to Slice 08 / 09, Slice 10, and Slice 12.

Before editing:
1. Read SPEC-07, PLAN-07, TASKS-07 and verify Packets 1, 2, and 3 are already landed, committed, and checkpoint-green on the current tree.
2. Re-read EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 2 README, and the full Slice 07 doc set.
3. Inspect the full touched surface for this slice:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/
4. If any runtime symbols changed outside the default script/docs boundary, run the required GitNexus checks before blessing the final packet.
5. Stay within the final validation and handoff boundary. Do not silently absorb ingress/mount, unit-source-of-truth, or broad docs-cutover work.

Packet 4 scope:
- Task 4.1: Final targeted regression and scope check.
- Task 4.2: Record the handoff boundary honestly.

Out of scope:
- new implementation scope beyond narrowly required final fixes
- transport redesign
- ingress/mount implementation
- guest-unit source-of-truth work
- broad feature-wide docs cutover

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- The implementation subagent must treat any required code/doc touch beyond final validation as scope pressure and call it out explicitly.
- After implementation, run the Packet 4 verification commands:
  - `bash -n scripts/mac/lima-warm.sh scripts/mac/lima-doctor.sh scripts/mac/smoke.sh`
  - `git diff --stat -- scripts/mac/lima-warm.sh scripts/mac/lima-doctor.sh scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
  - `gitnexus_detect_changes()` before committing if any runtime symbols changed
- If implementation is green, commit the Packet 4 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-07 / PLAN-07 / TASKS-07, the official Lima sources where relevant, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not claim Slice 07 complete until Packet 4 is committed and review-clean.

Packet 4 checkpoint:
- the slice remained listener-scoped
- verification evidence shows the default guest TCP listener is gone
- handoff language clearly defers ingress/mount to Slice 08 / 09, unit source-of-truth to Slice 10, and broad docs cutover to Slice 12
- no hidden scope expansion remains in the final diff

Implementation subagent prompt:
/goal Land Slice 07 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md first. Verify Packets 1, 2, and 3 are already green. Work only on Task 4.1 and Task 4.2. Keep the work bounded to final validation, scope policing, and honest handoff language. Do not widen into ingress/mount, unit-source-of-truth, or broad docs-cutover work. Re-run the Packet 4 verification commands and finish by stating whether Packet 4 is checkpoint-green, what files changed, what verification ran, and whether Slice 07 is complete.

Review subagent prompt:
Review the committed Slice 07 Packet 4 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md. Review only Packet 4 and the live diff. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 07 Packet 4 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the final diff remained listener-scoped.
- State whether Slice 07 is complete and what later slices remain deferred.
- If anything is not green, say explicitly that Slice 07 is not yet complete.
```
