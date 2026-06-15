# PROMPTS-08: Packet Orchestration Prompts For Slice 08

Source spec:
- [`SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`](./SPEC-08-ingress-inventory-and-narrowed-mount-contract.md)

Source plan:
- [`PLAN-08.md`](./PLAN-08.md)

Source tasks:
- [`TASKS-08.md`](./TASKS-08.md)

Current branch at prompt authoring time: `HEAD`  
Worker implementation skill:
`/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill:
`/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is
bounded to one packet from Slice `08` and is grounded in the live Slice `08`
spec/plan/tasks stack, the current feature-local macOS hardening docs, the
current repo truth that still carries broad mount/ingress assumptions, and the
official Lima/Apple sources that Slice `08` requires.

Because Slice `08` is a source-driven ingress-inventory and narrowed-contract
slice, these prompts emphasize:

1. bounded docs/contract-only scope,
2. explicit inventory of current host-home and `/src` ingress,
3. path-by-path classification instead of convenience-based reasoning,
4. strict deferral of actual mount minimization/sync implementation to Slice
   `09`,
5. commit discipline between implementation, review, and fix rounds.

## Packet 1 Prompt

```text
/goal Land Slice 08 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/mount/
- https://lima-vm.io/docs/config/vmtype/
- https://lima-vm.io/docs/faq/
- https://lima-vm.io/docs/releases/breaking/
- https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice

Mission:
- Land Packet 1 only: source gate and live ingress inventory.
- Do not start Packet 2.
- Keep the slice bounded to freezing the current default ingress inventory,
  ingress classes, and source-backed reasoning inside the Slice 08 planning
  docs only.

Before editing:
1. Read SPEC-08, PLAN-08, TASKS-08, EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 2 README, milestone 2.2, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, and DESIGN-macos-guest-unit-source-of-truth.md.
2. Inspect the current repo-truth ingress surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/substrate.yaml
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/builtins/world_gateway.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-service/src/gateway_runtime.rs
3. Verify the official Lima/Apple sources above before freezing any source-sensitive mount, VZ, or directory-sharing claim.
4. Stay within the default execution boundary. Do not widen into actual mount-profile edits, sync/copy implementation, Packet 2 classification work, or later sandbox/docs-cutover work.

Packet 1 scope:
- Task 1.1: Confirm the authority stack, source gate, and live ingress posture.
- Task 1.2: Freeze the current mount inventory and ingress classes.

Out of scope:
- Packet 2, Packet 3, or Packet 4 work
- edits outside the Slice 08 planning stack
- actual `scripts/mac/lima/substrate.yaml` changes
- actual `scripts/mac/lima-warm.sh` changes
- sync/copy implementation
- guest-unit sandbox changes
- broad docs cutover

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- The implementation subagent must keep the work limited to the Slice 08 planning docs and the source-gated ingress inventory freeze.
- After implementation, run the Packet 1 verification commands:
  - `rg -n "mounts:|location: \"\\$HOME\"|location: \"\\$PROJECT\"|mountPoint: \"/src\"" scripts/mac/lima/substrate.yaml`
  - `rg -n "/src|HOME|gateway-runtime|integrated auth|breakglass" scripts/mac/lima-warm.sh scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`
  - `git status --short`
- If implementation is green, commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-08 / PLAN-08 / TASKS-08, the official Lima/Apple sources, and the live diff.
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
- the current default ingress inventory is explicit
- the ingress classes are explicit
- the slice has not drifted into Packet 2/3/4 or into actual mount/sync implementation

Implementation subagent prompt:
/goal Land Slice 08 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md first. Also read EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 2 README, milestone 2.2, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, and DESIGN-macos-guest-unit-source-of-truth.md. Verify the official Lima/Apple source set before freezing any ingress claim. Work only on Task 1.1 and Task 1.2. Keep the work bounded to the Slice 08 planning docs and the source-gated ingress inventory freeze. Do not widen into actual mount edits, sync/copy implementation, Packet 2 classification work, or later sandbox/docs-cutover work. Re-run the Packet 1 verification commands and finish by stating whether Packet 1 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Packet 2 is unblocked.

Review subagent prompt:
Review the committed Slice 08 Packet 1 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md, and the official Lima/Apple sources required by Slice 08. Review only Packet 1 and the live diff. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 08 Packet 1 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md. Keep fixes limited to Packet 1 findings. Do not widen scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- List the official Lima/Apple sources used for the final ingress-inventory wording.
- State whether the touched file set stayed within the default execution boundary.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 08 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/mount/
- https://lima-vm.io/docs/config/vmtype/
- https://lima-vm.io/docs/faq/
- https://lima-vm.io/docs/releases/breaking/
- https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice

Mission:
- Land Packet 2 only: classify ingress and freeze the narrowed decision matrix.
- Do not start Packet 3.
- Keep the slice bounded to the Slice 08 planning docs only.

Before editing:
1. Read SPEC-08, PLAN-08, TASKS-08 and verify Packet 1 is already landed, committed, and checkpoint-green on the current tree.
2. Re-read milestone 2.2, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, and DESIGN-macos-guest-unit-source-of-truth.md.
3. Re-inspect the current ingress-bearing repo-truth surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/substrate.yaml
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/builtins/world_gateway.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-service/src/gateway_runtime.rs
4. Verify the official Lima/Apple sources above before preserving or reclassifying any mounted host-home or `/src` claim.
5. Stay within the default execution boundary. Do not widen into actual mount-profile edits, sync/copy implementation, or later validation/doc-consumer work.

Packet 2 scope:
- Task 2.1: Build the ingress classification matrix.
- Task 2.2: Freeze the narrowed default decision for host-home visibility and `/src`.

Out of scope:
- Packet 3 or Packet 4 work
- edits outside the Slice 08 planning stack
- actual Lima profile changes
- actual warm/smoke/docs/runtime code edits
- new sync/copy machinery
- sandbox/unit changes

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- The implementation subagent must make the classification matrix explicit and must not treat “currently mounted” as equivalent to “hardened default.”
- After implementation, run the Packet 2 verification commands:
  - `rg -n "workspace|auth|runtime|troubleshooting|direct mount|sync/copy|breakglass" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md`
  - `rg -n "\\$HOME|/src|path-by-path|justif|sync/copy|breakglass" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`
  - `git status --short`
- If implementation is green, commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-08 / PLAN-08 / TASKS-08, the official Lima/Apple sources, and the live diff.
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
- every ingress class has an intended hardened posture
- broad host-home visibility is no longer implicit
- `/src` is decomposed into concrete supported needs
- the slice stayed within the Packet 2 docs/contract boundary

Implementation subagent prompt:
/goal Land Slice 08 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md first. Verify Packet 1 is already green. Work only on Task 2.1 and Task 2.2. Keep the work bounded to the Slice 08 planning docs. Make the classification matrix explicit and path-by-path. Do not widen into actual mount edits, sync/copy implementation, Packet 3 validation/doc-consumer work, or later sandbox/docs-cutover work. Re-run the Packet 2 verification commands and finish by stating whether Packet 2 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Packet 3 is unblocked.

Review subagent prompt:
Review the committed Slice 08 Packet 2 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md, and the official Lima/Apple sources required by Slice 08. Review only Packet 2 and the live diff. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 08 Packet 2 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether broad host-home visibility remains implicit or not.
- State whether `/src` is now decomposed into concrete supported needs.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 08 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/mount/
- https://lima-vm.io/docs/config/vmtype/
- https://lima-vm.io/docs/faq/
- https://lima-vm.io/docs/releases/breaking/
- https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice

Mission:
- Land Packet 3 only: freeze validation and future doc-cutover expectations.
- Do not start Packet 4.
- Keep the slice bounded to the Slice 08 planning docs only.

Before editing:
1. Read SPEC-08, PLAN-08, TASKS-08 and verify Packets 1 and 2 are already landed, committed, and checkpoint-green on the current tree.
2. Re-read milestone 2.2, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, and DESIGN-macos-guest-unit-source-of-truth.md.
3. Re-inspect the current downstream validation/doc-consumer surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/builtins/world_gateway.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-service/src/gateway_runtime.rs
4. Verify the official Lima/Apple sources above before preserving or freezing any validation expectation that depends on mount or VZ semantics.
5. Stay within the default execution boundary. Do not widen into actual mount-profile edits, sync/copy implementation, or full downstream docs/sandbox changes.

Packet 3 scope:
- Task 3.1: Define the later validation surfaces explicitly.
- Task 3.2: Record the future doc and sandbox consumers without widening into them.

Out of scope:
- Packet 4 work
- edits outside the Slice 08 planning stack
- actual `docs/WORLD.md` or `macos-lima-setup.md` edits
- actual sandbox/unit changes
- actual mount or sync implementation

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- The implementation subagent must keep the validation expectations explicit without promising downstream changes that Slice 08 does not make.
- After implementation, run the Packet 3 verification commands:
  - `rg -n "lima-warm|smoke|gateway sync|gateway status|diagnostics|proof" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md`
  - `rg -n "docs/WORLD.md|macos-lima-setup|Slice 09|Slice 10|ProtectHome|ReadWritePaths" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`
  - `git status --short`
- If implementation is green, commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-08 / PLAN-08 / TASKS-08, the official Lima/Apple sources, and the live diff.
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
- later proof surfaces are explicit
- downstream doc and sandbox consumers are named
- the slice has not widened into actual mount minimization, docs cutover, or sandbox changes

Implementation subagent prompt:
/goal Land Slice 08 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md first. Verify Packets 1 and 2 are already green. Work only on Task 3.1 and Task 3.2. Keep the work bounded to the Slice 08 planning docs. Do not widen into actual mount edits, sync/copy implementation, or downstream docs/sandbox changes. Re-run the Packet 3 verification commands and finish by stating whether Packet 3 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Packet 4 is unblocked.

Review subagent prompt:
Review the committed Slice 08 Packet 3 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md, and the official Lima/Apple sources required by Slice 08. Review only Packet 3 and the live diff. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 08 Packet 3 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the later validation surfaces are explicit.
- State whether downstream doc and sandbox consumers are explicit.
- State whether Packet 4 is unblocked.
- If anything is not green, say explicitly that Packet 4 must not begin.
```

## Packet 4 Prompt

```text
/goal Land Slice 08 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/mount/
- https://lima-vm.io/docs/config/vmtype/
- https://lima-vm.io/docs/faq/
- https://lima-vm.io/docs/releases/breaking/
- https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice

Mission:
- Land Packet 4 only: final scope check and next-slice handoff.
- This is the final Packet for Slice 08.
- Keep the slice bounded to final coherence, handoff clarity, and docs-only closeout within the Slice 08 planning docs.

Before editing:
1. Read SPEC-08, PLAN-08, TASKS-08 and verify Packets 1, 2, and 3 are already landed, committed, and checkpoint-green on the current tree.
2. Re-read milestone 2.2, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-supported-mode-and-breakglass-taxonomy.md, and DESIGN-macos-guest-unit-source-of-truth.md.
3. Re-inspect the final Slice 08 planning docs and the downstream seams named in Slice 08:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
4. Verify the official Lima/Apple sources above if any final wording still depends on source-sensitive mount behavior claims.
5. Stay within the default execution boundary. Do not widen into actual mount-profile edits, sync/copy implementation, or downstream docs/sandbox execution.

Packet 4 scope:
- Task 4.1: Final scope and coherence check.
- Task 4.2: Record the handoff boundary honestly.

Out of scope:
- any new implementation beyond the Slice 08 planning docs
- actual Slice 09 work
- actual Slice 10 work
- actual docs cutover
- actual sandbox/unit changes

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- The implementation subagent must keep the closeout honest: Slice 08 landed the ingress inventory/narrowed-contract seam only.
- After implementation, run the Packet 4 verification commands:
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`
  - `git status --short`
  - `rg -n "Slice 09|Slice 10|Slice 12|sync implementation|sandbox unification|docs cutover" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`
- If implementation is green, commit the Packet 4 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-08 / PLAN-08 / TASKS-08, the official Lima/Apple sources, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not begin Slice 09 planning or implementation work until Packet 4 is committed and review-clean.

Packet 4 checkpoint:
- the slice remained docs-only and contract-scoped
- the implementation seam for Slice 09 is obvious
- the deferred downstream consumers are stated plainly
- the final closeout does not imply that actual mount minimization has already landed

Implementation subagent prompt:
/goal Land Slice 08 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md first. Verify Packets 1, 2, and 3 are already green. Work only on Task 4.1 and Task 4.2. Keep the work bounded to the Slice 08 planning docs and the final handoff wording. Do not widen into Slice 09 implementation, Slice 10 sandbox work, or downstream docs cutover. Re-run the Packet 4 verification commands and finish by stating whether Packet 4 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Slice 09 is unblocked.

Review subagent prompt:
Review the committed Slice 08 Packet 4 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md, and the official Lima/Apple sources required by Slice 08. Review only Packet 4 and the live diff. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 08 Packet 4 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the slice stayed docs-only and contract-scoped.
- State whether Slice 09 is unblocked.
- If anything is not green, say explicitly that Slice 09 work must not begin.
```
