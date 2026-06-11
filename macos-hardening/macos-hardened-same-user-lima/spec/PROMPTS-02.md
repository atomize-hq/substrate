# PROMPTS-02: Packet Orchestration Prompts For Slice 02

Source spec:
- [`SPEC-02-lima-version-floor-and-breakglass-contract.md`](./SPEC-02-lima-version-floor-and-breakglass-contract.md)

Source plan:
- [`PLAN-02.md`](./PLAN-02.md)

Source tasks:
- [`TASKS-02.md`](./TASKS-02.md)

Current branch at prompt authoring time: `HEAD`  
Worker implementation skill:
`/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill:
`/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is
grounded only in the live Slice `02` spec/plan/tasks stack, the current
feature-local macOS hardening docs, and the official Lima source set that
Slice `02` requires.

Because Slice `02` is a source-driven docs-and-contract slice, these prompts
emphasize:

1. bounded document scope,
2. official-Lima source verification before contract changes,
3. explicit breakglass classification,
4. strict handoff boundaries to Slice `03` and Slice `12`,
5. commit discipline between implementation, review, and fix rounds.

## Packet 1 Prompt

```text
/goal Land Slice 02 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/releases/
- https://lima-vm.io/docs/config/vmtype/
- https://lima-vm.io/docs/config/vmtype/vz/
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/config/mount/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Packet 1 only: Supported environment contract freeze.
- Do not start Packet 2.
- Keep the slice bounded to freezing the supported Lima/macOS capability
  contract and version-floor decision criteria.

Before editing:
1. Read SPEC-02, PLAN-02, TASKS-02, EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 0 README, milestone 0.2, DESIGN-supported-mode-and-breakglass-taxonomy.md, and DESIGN-macos-lima-transport-contract.md.
2. Inspect the current repo-truth evidence in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/substrate.yaml
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/forwarding.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/lib.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/platform/macos.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/builtins/world_gateway.rs
3. Verify the official Lima sources above before freezing any version-sensitive or capability-sensitive claim.
4. Stay within the default execution boundary. Do not widen into top-level repo docs or runtime code unless a direct contradiction forces it and you call that out explicitly.

Packet 1 scope:
- Task 1.1: Confirm the authority stack and source gate.
- Task 1.2: Freeze the supported environment contract in precise lifecycle/capability language.

Out of scope:
- Packet 2 or Packet 3 work
- full canonical transport contract work that belongs to Slice 03
- mount, listener, unit, lifecycle, or repo-wide docs-cutover work
- runtime or provisioning edits

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- The implementation subagent must verify the official Lima sources before landing any version-floor claim.
- After implementation, run the Packet 1 verification commands:
  - `rg -n "recent Lima|v2.x|13\\.0|13\\.5|vmType|vz|vsock-proxy" macos-hardening/macos-hardened-same-user-lima/spec`
  - `sed -n '1,220p' scripts/mac/lima/substrate.yaml`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope`
  - `git status --short`
- If implementation is green, commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-02 / PLAN-02 / TASKS-02, the official Lima sources, and the live diff.
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
- the supported environment contract is explicit
- official Lima sources back the version-sensitive claims
- the slice has not drifted into full transport-unification scope

Implementation subagent prompt:
/goal Land Slice 02 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md first. Also read EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 0 README, milestone 0.2, DESIGN-supported-mode-and-breakglass-taxonomy.md, and DESIGN-macos-lima-transport-contract.md. Verify the official Lima source set before freezing any version-sensitive claim. Work only on Task 1.1 and Task 1.2. Keep the slice docs-and-contract only. Do not widen into Packet 2, Slice 03 transport unification, or later docs-cutover work. Re-run the Packet 1 verification commands and finish by stating whether Packet 1 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Packet 2 is unblocked.

Review subagent prompt:
Review the committed Slice 02 Packet 1 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md, and the official Lima sources required by Slice 02. Review only Packet 1 and the live diff. Review across correctness, readability, architecture, security, and performance as they apply to contract docs. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 02 Packet 1 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md. Keep fixes limited to Packet 1 findings. Do not widen scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- List the official Lima sources used for the final contract wording.
- State whether the touched-doc set stayed within the default execution boundary.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 02 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md

Required official source set for this packet:
- https://lima-vm.io/docs/releases/
- https://lima-vm.io/docs/config/vmtype/
- https://lima-vm.io/docs/config/vmtype/vz/
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/config/mount/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Packet 2 only: Breakglass matrix and supported replacement rule.
- Do not start Packet 3.
- Keep the slice bounded to explicit workflow classification and supported replacement framing.

Before editing:
1. Read SPEC-02, PLAN-02, TASKS-02 and verify Packet 1 is already landed, committed, and checkpoint-green on the current tree.
2. Re-read milestone 0.2, DESIGN-supported-mode-and-breakglass-taxonomy.md, and DESIGN-macos-lima-transport-contract.md.
3. Inspect the current breakglass-sensitive repo surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/lib.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/platform/macos.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/builtins/world_gateway.rs
4. Stay within the default feature-local docs boundary unless a contradiction forces escalation.

Packet 2 scope:
- Task 2.1: Freeze the breakglass classification for direct guest and host-bypass workflows.
- Task 2.2: Freeze the supported replacement rule and compatibility-path framing.

Out of scope:
- Packet 3 work
- full canonical transport contract work that belongs to Slice 03
- repo-wide docs cutover that belongs to Slice 12
- runtime or provisioning edits

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- After implementation, run the Packet 2 verification commands:
  - `rg -n "limactl shell|SUBSTRATE_WORLD_SOCKET|breakglass|degraded-but-supported|supported" macos-hardening/macos-hardened-same-user-lima/spec`
  - `rg -n "17788|7788|host TCP|compatibility|Substrate-owned commands|gateway status|world doctor|host doctor" macos-hardening/macos-hardened-same-user-lima/spec`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope`
  - `git status --short`
- If implementation is green, commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-02 / PLAN-02 / TASKS-02, the official Lima sources, and the live diff.
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
- breakglass workflows are explicitly classified
- the supported operator story clearly starts from Substrate-owned commands
- retained compatibility probes are not mistaken for the supported default
- the slice still remains feature-local and docs-first

Implementation subagent prompt:
/goal Land Slice 02 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md first. Verify Packet 1 is already green. Work only on Task 2.1 and Task 2.2. Keep the edits docs-only, feature-local, and limited to breakglass classification plus supported replacement framing. Do not widen into Slice 03 transport design or Slice 12 docs cutover. Re-run the Packet 2 verification commands and finish by stating whether Packet 2 is checkpoint-green, what files changed, what verification ran, and whether Packet 3 is unblocked.

Review subagent prompt:
Review the committed Slice 02 Packet 2 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance as they apply to contract docs and planning artifacts. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 02 Packet 2 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the supported replacement rule is explicit and whether retained compatibility probes stayed out of the supported default story.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 02 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/releases/
- https://lima-vm.io/docs/config/vmtype/
- https://lima-vm.io/docs/config/vmtype/vz/
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/config/mount/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Slice 02 Packet 3 only: Explicit deferrals and next-slice handoff clarity.
- Do not reopen Packet 1 or Packet 2 except where final wording cleanup is strictly required.
- Keep the slice bounded to explicit deferrals, coherence review, and leaving the short-prompt planning path clean for Slice 03 and Slice 12.

Before editing:
1. Read SPEC-02, PLAN-02, TASKS-02 and verify Packets 1 and 2 are already landed, committed, and checkpoint-green on the current tree.
2. Re-read EXECUTION-RUBRIC.md, ROADMAP.md, milestone 0.2, and the full Slice 02 doc set.
3. Confirm the default next seam is still Slice 03 and that repo-wide breakglass/docs cutover still belongs to Slice 12.
4. Stay within the feature-local planning boundary unless a contradiction forces escalation.

Packet 3 scope:
- Task 3.1: Validate explicit deferrals to later slices.
- Task 3.2: Final diff and coherence review.

Out of scope:
- new environment-floor decisions not already justified in Packet 1
- new breakglass categories or taxonomy rewrites
- runtime, provisioning, transport, mount, unit, or repo-wide docs-cutover work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- After implementation, run the Packet 3 verification commands:
  - `rg -n "Slice 03|Slice 12|transport|docs cutover|mount|listener|unit|lifecycle" macos-hardening/macos-hardened-same-user-lima/spec`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope`
  - `git status --short`
  - manual coherence review of SPEC-02 / PLAN-02 / TASKS-02 / EXECUTION-RUBRIC.md / ROADMAP.md
- If implementation is green, commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-02 / PLAN-02 / TASKS-02, the live diff, and the packet boundaries.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not move to Slice 03 or any later seam until Packet 3 is committed and review-clean.

Packet 3 checkpoint:
- Slice 02 stayed bounded
- Slice 03 remains the next honest transport seam
- Slice 12 remains the docs-cutover seam
- the planning stack is coherent enough for a short future prompt to continue without hidden assumptions
- PROMPTS-02.md stays consistent with the packet boundaries and review/fix flow

Implementation subagent prompt:
/goal Land Slice 02 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md first. Verify Packets 1 and 2 are already green. Work only on Task 3.1 and Task 3.2. Keep the slice bounded to explicit deferrals, coherence, and next-slice readability. Re-run the Packet 3 verification commands and finish by stating whether Packet 3 is checkpoint-green, what files changed, what verification ran, and whether Slice 03 is unblocked.

Review subagent prompt:
Review the committed Slice 02 Packet 3 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance as they apply to contract docs and planning artifacts. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 02 Packet 3 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether Slice 03 remains the next honest seam and whether Slice 12 stayed deferred.
- State whether the prompt stack for Slice 02 is coherent for future short-prompt continuation.
- If anything is not green, say explicitly that Slice 03 must not begin.
```
