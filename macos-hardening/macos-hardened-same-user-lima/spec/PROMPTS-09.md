# PROMPTS-09: Packet Orchestration Prompts For Slice 09

Source spec:
- [`SPEC-09-ingress-cutover-and-explicit-staging-path.md`](./SPEC-09-ingress-cutover-and-explicit-staging-path.md)

Source plan:
- [`PLAN-09.md`](./PLAN-09.md)

Source tasks:
- [`TASKS-09.md`](./TASKS-09.md)

Current branch at prompt authoring time: `HEAD`  
Worker implementation skill:
`/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill:
`/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is
bounded to one packet from Slice `09` and is grounded in the live Slice `09`
spec/plan/tasks stack, the current feature-local macOS hardening docs, the
current repo truth that still carries broad default host-home plus mounted `/src`
ingress assumptions, and the official Lima/Apple sources that Slice `09`
requires.

Because Slice `09` is a source-driven ingress-cutover slice, these prompts
emphasize:

1. actual implementation of the narrowed ingress contract rather than another
   contract-only pass,
2. explicit staged host→guest ingress rooted in approved guest-local writable
   paths,
3. strict deferral of guest-unit sandbox unification to Slice `10`, broader
   operator-surface productization to Slice `11`, and broad docs/breakglass
   cutover to Slice `12`,
4. routed proof preservation for warm, smoke, doctor, and gateway lifecycle,
5. commit discipline between implementation, review, and fix rounds.

## Packet 1 Prompt

```text
/goal Land Slice 09 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/mount/
- https://lima-vm.io/docs/config/vmtype/
- https://lima-vm.io/docs/faq/
- https://lima-vm.io/docs/releases/breaking/
- https://lima-vm.io/docs/reference/limactl_copy/
- https://lima-vm.io/docs/reference/limactl_create/
- https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice

Mission:
- Land Packet 1 only: source gate, staging-root decision, and live dependency inventory.
- Do not start Packet 2.
- Keep the slice bounded to freezing the official-source-backed execution boundary,
  the live ingress-dependent repo surfaces, and the preferred guest-local staging
  root direction.

Before editing:
1. Read SPEC-09, PLAN-09, TASKS-09, EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 2 README,
   milestone 2.2, milestone 3.1, DESIGN-macos-ingress-and-mount-contract.md,
   DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md,
   DESIGN-macos-guest-unit-source-of-truth.md, and
   DESIGN-supported-mode-and-breakglass-taxonomy.md.
2. Inspect the current repo-truth ingress-bearing surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/substrate.yaml
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/workspace_cmd.rs
3. Verify the official Lima/Apple sources above before freezing any source-sensitive
   mount, VZ, directory-sharing, or host↔guest copy claim.
4. Stay within Packet 1 scope. Do not widen into actual mount-profile edits,
   warm-path edits, smoke-path edits, or docs outside the Slice 09 planning stack.

Packet 1 scope:
- Task 1.1: Confirm the authority stack, source gate, and live ingress dependencies.
- Task 1.2: Freeze the guest-local staging-root and minimal implementation boundary.

Out of scope:
- Packet 2, Packet 3, or Packet 4 work
- actual `scripts/mac/lima/substrate.yaml` changes
- actual `scripts/mac/lima-warm.sh` changes
- actual `scripts/mac/smoke.sh` changes
- actual `docs/WORLD.md` or `docs/reference/world/platforms/macos-lima-setup.md` changes
- guest-unit sandbox/source-of-truth work
- broad operator-surface productization
- broad docs/breakglass cutover

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- The implementation subagent must keep the work limited to the Slice 09 planning docs and the source-gated staging-root / implementation-boundary freeze.
- After implementation, run the Packet 1 verification commands:
  - `rg -n "workspace sync|/var/lib/substrate|/tmp|staged|stage|copy" crates/shell/src/execution/workspace_cmd.rs scripts/mac/lima-warm.sh macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md`
  - `git status --short`
- If implementation is green, commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-09 / PLAN-09 / TASKS-09, the official Lima/Apple sources, and the live diff.
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
- the official source gate is explicit
- the live ingress-dependent repo surfaces are explicit
- the preferred guest-local staging-root direction is explicit
- the slice has not drifted into Packet 2/3/4 or into actual ingress implementation

Implementation subagent prompt:
/goal Land Slice 09 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md first. Also read EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 2 README, milestone 2.2, milestone 3.1, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-macos-guest-unit-source-of-truth.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md. Verify the official Lima/Apple source set before freezing any staging-root or ingress claim. Work only on Task 1.1 and Task 1.2. Keep the work bounded to the Slice 09 planning docs and the source-gated staging-root / implementation-boundary freeze. Do not widen into actual mount-profile edits, warm-path edits, smoke-path edits, or docs outside the Slice 09 planning stack. Re-run the Packet 1 verification commands and finish by stating whether Packet 1 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Packet 2 is unblocked.

Review subagent prompt:
Review the committed Slice 09 Packet 1 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md, and the official Lima/Apple sources required by Slice 09. Review only Packet 1 and the live diff. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 09 Packet 1 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md. Keep fixes limited to Packet 1 findings. Do not widen scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- List the official Lima/Apple sources used for the final staging-root / execution-boundary wording.
- State whether the touched file set stayed within the default execution boundary.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 09 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/mount/
- https://lima-vm.io/docs/config/vmtype/
- https://lima-vm.io/docs/faq/
- https://lima-vm.io/docs/releases/breaking/
- https://lima-vm.io/docs/reference/limactl_copy/
- https://lima-vm.io/docs/reference/limactl_create/
- https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice

Mission:
- Land Packet 2 only: mount-profile and warm-path cutover.
- Do not start Packet 3.
- Keep the slice bounded to removing broad default host-home visibility,
  removing default `/src` dependence from warm/repair, and replacing those
  assumptions with explicit staged guest-local ingress.

Before editing:
1. Read SPEC-09, PLAN-09, TASKS-09 and verify Packet 1 is already landed,
   committed, and checkpoint-green on the current tree.
2. Re-read the Phase 2 README, milestone 2.2, milestone 3.1,
   DESIGN-macos-ingress-and-mount-contract.md,
   DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md,
   DESIGN-macos-guest-unit-source-of-truth.md, and
   DESIGN-supported-mode-and-breakglass-taxonomy.md.
3. Re-inspect the current implementation surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/substrate.yaml
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/workspace_cmd.rs
4. Verify the official Lima/Apple sources above before preserving or replacing any mount,
   host-home, `/src`, staging, or host↔guest copy claim.
5. Stay within Packet 2 scope. Do not widen into smoke-harness changes, broad docs changes,
   guest-unit sandbox work, or broad CLI/operator-surface productization.

Packet 2 scope:
- Task 2.1: Narrow the Lima mount profile to the hardened default.
- Task 2.2: Replace warm-path `/src` dependence with explicit staged ingress.

Out of scope:
- Packet 3 or Packet 4 work
- smoke-harness changes beyond what is strictly required to keep Packet 2 coherent
- broad `docs/WORLD.md` or `docs/reference/world/platforms/macos-lima-setup.md` edits
- guest-unit source-of-truth or sandbox unification
- broad lifecycle/sync productization beyond a minimal assist if strictly required
- support-taxonomy rewrite

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- The implementation subagent must keep the work focused on `scripts/mac/lima/substrate.yaml` and `scripts/mac/lima-warm.sh` unless a minimal justified assist is required.
- After implementation, run the Packet 2 verification commands:
  - `rg -n "mounts:|location: \"\\$HOME\"|location: \"\\$PROJECT\"|mountPoint: \"/src\"|writable:" scripts/mac/lima/substrate.yaml`
  - `rg -n "ensure_repo_mount|/src|limactl copy|Cargo.lock|cargo build|staged|stage|/var/lib/substrate" scripts/mac/lima-warm.sh`
  - `bash -n scripts/mac/lima-warm.sh`
  - `git diff --stat -- scripts/mac/lima/substrate.yaml scripts/mac/lima-warm.sh`
  - `git status --short`
- If implementation is green, commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-09 / PLAN-09 / TASKS-09, the official Lima/Apple sources, and the live diff.
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
- broad host-home visibility is gone from the hardened default
- mounted `/src` is no longer the required warm-path ingress model
- the replacement staged ingress path is explicit and guest-local
- the slice stayed within the Packet 2 boundary or used only a clearly justified minimal assist

Implementation subagent prompt:
/goal Land Slice 09 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md first. Verify Packet 1 is already green. Verify the official Lima/Apple source set before preserving or replacing any mount or staging claim. Work only on Task 2.1 and Task 2.2. Keep the work bounded to the mount-profile and warm-path cutover. Do not widen into smoke-harness changes, broad docs changes, Slice 10 sandbox work, or Slice 11 operator-surface productization unless a minimal assist proves strictly required. Re-run the Packet 2 verification commands and finish by stating whether Packet 2 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Packet 3 is unblocked.

Review subagent prompt:
Review the committed Slice 09 Packet 2 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md, and the official Lima/Apple sources required by Slice 09. Review only Packet 2 and the live diff. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 09 Packet 2 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether broad host-home visibility is gone from the hardened default.
- State whether mounted `/src` is no longer the required warm-path ingress model.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 09 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/mount/
- https://lima-vm.io/docs/config/vmtype/
- https://lima-vm.io/docs/faq/
- https://lima-vm.io/docs/releases/breaking/
- https://lima-vm.io/docs/reference/limactl_copy/
- https://lima-vm.io/docs/reference/limactl_create/
- https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice

Mission:
- Land Packet 3 only: smoke and minimal doc-truth cutover.
- Do not start Packet 4.
- Keep the slice bounded to adapting the smoke harness to the staged ingress path
  and removing only the doc statements made false by the ingress cutover.

Before editing:
1. Read SPEC-09, PLAN-09, TASKS-09 and verify Packet 2 is already landed,
   committed, and checkpoint-green on the current tree.
2. Re-read the Phase 2 README, milestone 2.2, milestone 3.1,
   DESIGN-macos-ingress-and-mount-contract.md,
   DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md,
   DESIGN-macos-guest-unit-source-of-truth.md, and
   DESIGN-supported-mode-and-breakglass-taxonomy.md.
3. Re-inspect the current proof and doc surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
4. Verify the official Lima/Apple sources above before preserving or replacing any claim
   about default mounts, direct guest access, or host↔guest staging behavior.
5. Stay within Packet 3 scope. Do not widen into Packet 4 closeout work,
   guest-unit sandbox work, or a broader operator-story/docs cutover.

Packet 3 scope:
- Task 3.1: Replace the smoke harness’s mounted-workspace proof.
- Task 3.2: Perform only the minimal doc truth corrections required by the cutover.

Out of scope:
- Packet 4 work
- reopening Packet 2 mount-profile or warm-path decisions unless a review-critical fix demands it
- full docs cutover or support-taxonomy rewrite
- broad lifecycle/sync productization
- guest-unit source-of-truth or sandbox unification

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- The implementation subagent must keep the work focused on `scripts/mac/smoke.sh`, `docs/WORLD.md`, and `docs/reference/world/platforms/macos-lima-setup.md`.
- After implementation, run the Packet 3 verification commands:
  - `rg -n "/src|gateway sync|gateway status|gateway restart|host doctor|world doctor|world-mac-smoke|staged|stage" scripts/mac/smoke.sh`
  - `bash -n scripts/mac/smoke.sh`
  - `rg -n "/src|mount|HOME|limactl shell substrate bash -lc 'cd /src|mirrors the host repo checkout|broad host-home" docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md`
  - `git diff --stat -- scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md`
  - `git status --short`
- If implementation is green, commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-09 / PLAN-09 / TASKS-09, the official Lima/Apple sources, and the live diff.
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
- the smoke harness no longer encodes mounted `/src` as the normal path
- the docs no longer say false things about the hardened default ingress
- routed doctor/gateway proof priority is preserved
- the slice stayed within the Packet 3 boundary rather than widening into broad docs/operator cutover work

Implementation subagent prompt:
/goal Land Slice 09 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md first. Verify Packet 2 is already green. Verify the official Lima/Apple source set before preserving or replacing any mount or staging claim. Work only on Task 3.1 and Task 3.2. Keep the work bounded to the smoke and minimal doc-truth cutover. Do not widen into Packet 4 closeout, Slice 10 sandbox work, Slice 11 productization, or Slice 12 broad docs cutover. Re-run the Packet 3 verification commands and finish by stating whether Packet 3 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Packet 4 is unblocked.

Review subagent prompt:
Review the committed Slice 09 Packet 3 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md, and the official Lima/Apple sources required by Slice 09. Review only Packet 3 and the live diff. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 09 Packet 3 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the smoke harness no longer encodes mounted `/src` as the normal path.
- State whether the docs no longer say false things about the hardened default ingress.
- State whether Packet 4 is unblocked.
- If anything is not green, say explicitly that Packet 4 must not begin.
```

## Packet 4 Prompt

```text
/goal Land Slice 09 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/mount/
- https://lima-vm.io/docs/config/vmtype/
- https://lima-vm.io/docs/faq/
- https://lima-vm.io/docs/releases/breaking/
- https://lima-vm.io/docs/reference/limactl_copy/
- https://lima-vm.io/docs/reference/limactl_create/
- https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice

Mission:
- Land Packet 4 only: final validation and downstream handoff.
- Finish Slice 09 honestly without widening it.
- Keep the slice bounded to final scope/coherence validation plus explicit handoff into Slice 10, Slice 11, and Slice 12.

Before editing:
1. Read SPEC-09, PLAN-09, TASKS-09 and verify Packet 3 is already landed,
   committed, and checkpoint-green on the current tree.
2. Re-read the Packet 2 and Packet 3 outcomes in the current tree so the final
   handoff language matches what actually landed.
3. Inspect the full touched-file set and current diff state.
4. Verify the official Lima/Apple sources above if any final wording still makes
   source-sensitive claims about mounts, staging, or default guest exposure.
5. Stay within Packet 4 scope. Do not reopen earlier packet implementation unless a review-critical fix is required to make the final closeout honest.

Packet 4 scope:
- Task 4.1: Final scope and coherence check.
- Task 4.2: Record the downstream handoff honestly.

Out of scope:
- reopening Packet 1/2/3 unless a review-critical fix is required
- new lifecycle/sync productization beyond the already-landed packet work
- guest-unit source-of-truth or sandbox unification
- broad support-taxonomy or docs/breakglass cutover

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- The implementation subagent must keep the work focused on final validation, final wording, and exact downstream boundaries.
- After implementation, run the Packet 4 verification commands:
  - `git diff --stat`
  - `git status --short`
  - manual review that the final wording explicitly preserves Slice 10 / Slice 11 / Slice 12 boundaries
- If implementation is green, commit the Packet 4 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-09 / PLAN-09 / TASKS-09, the official Lima/Apple sources where still relevant, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not declare Slice 09 complete until Packet 4 is committed and review-clean.

Packet 4 checkpoint:
- the hardened default no longer depends on broad host-home visibility or a mounted `/src` checkout
- the explicit staged ingress path is clear and validated
- routed warm/smoke/doctor/gateway proofs remain green
- downstream boundaries into Slice `10`, Slice `11`, and Slice `12` are explicit and honest

Implementation subagent prompt:
/goal Land Slice 09 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md first. Verify Packet 3 is already green. Work only on Task 4.1 and Task 4.2. Keep the work bounded to final validation, final wording, and exact handoff boundaries into Slice 10, Slice 11, and Slice 12. Do not widen into new implementation work unless a review-critical fix is required to make the closeout honest. Re-run the Packet 4 verification commands and finish by stating whether Packet 4 is checkpoint-green, what files changed, what verification ran, what official sources were used if any still mattered, and whether Slice 09 is fully ready for downstream implementation.

Review subagent prompt:
Review the committed Slice 09 Packet 4 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md, and the live diff. Review only Packet 4 and the final closeout wording. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 09 Packet 4 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the final wording preserves the Slice 10 / Slice 11 / Slice 12 boundaries explicitly.
- State whether Slice 09 is fully ready for downstream implementation.
- If anything is not green, say explicitly that Slice 09 must not be treated as fully closed out.
```
