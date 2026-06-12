# PROMPTS-06: Packet Orchestration Prompts For Slice 06

Source spec:
- [`SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md`](./SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md)

Source plan:
- [`PLAN-06.md`](./PLAN-06.md)

Source tasks:
- [`TASKS-06.md`](./TASKS-06.md)

Current branch at prompt authoring time: `HEAD`  
Worker implementation skill:
`/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill:
`/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is
grounded only in the live Slice `06` spec/plan/tasks stack, the current
feature-local macOS hardening docs, the current repo truth that Slice `06`
needs, and the official Lima source set that Slice `06` requires.

Because Slice `06` is a source-driven readiness-evidence and docs/script-cutover
slice, these prompts emphasize:

1. bounded readiness-story scope,
2. routed-path-first proof order,
3. explicit breakglass classification for guest-direct flows,
4. strict handoff boundaries to Slice `07` and Slice `12`,
5. commit discipline between implementation, review, and fix rounds.

## Packet 1 Prompt

```text
/goal Land Slice 06 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/config/environment-variables/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Packet 1 only: readiness-order freeze and source gate.
- Do not start Packet 2.
- Keep the slice bounded to freezing the canonical routed-path-first readiness
  order and inventorying the script/doc contradictions that still normalize
  guest-direct proof.

Before editing:
1. Read SPEC-06, PLAN-06, TASKS-06, EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 1 README, milestone 1.3, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-macos-policy-input-parity.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
2. Inspect the current repo-truth evidence in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/USAGE.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/contracts/gateway/operator-contract.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/contracts/gateway/status-schema.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/platform/macos.rs
3. Verify the official Lima sources above before freezing any source-sensitive readiness or operator-path claim.
4. Stay within the default execution boundary. Do not widen into Packet 2 script edits, Packet 3 doc cutover, or later listener/docs-cutover work.

Packet 1 scope:
- Task 1.1: Confirm the authority stack, source gate, and live readiness drift.
- Task 1.2: Freeze the canonical readiness evidence order and scope boundary.

Out of scope:
- Packet 2, Packet 3, or Packet 4 work
- script rewrites
- readiness-oriented doc rewrites
- lifecycle redesign, warm/provision redesign, or listener hardening
- broad gateway contract changes

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- The implementation subagent must keep the work limited to the Slice 06 planning docs and the readiness-order freeze.
- After implementation, run the Packet 1 verification commands:
  - `rg -n "host doctor|world doctor|gateway status|gateway sync|gateway restart|limactl shell|SUBSTRATE_WORLD_SOCKET" scripts/mac/lima-doctor.sh scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md docs/USAGE.md`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
- If implementation is green, commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-06 / PLAN-06 / TASKS-06, the official Lima sources, and the live diff.
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
- the happy-path readiness order is explicit
- the contradiction surfaces are explicit
- `SUBSTRATE_WORLD_SOCKET` remains classified as advanced/test/breakglass on macOS
- the slice has not drifted into script or docs cutover work

Implementation subagent prompt:
/goal Land Slice 06 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md first. Also read EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 1 README, milestone 1.3, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, DESIGN-macos-policy-input-parity.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md. Verify the official Lima source set before freezing any readiness-path claim. Work only on Task 1.1 and Task 1.2. Keep the work bounded to the Slice 06 planning docs and the readiness-order freeze. Do not widen into Packet 2 script changes, Packet 3 doc changes, or later hardening work. Re-run the Packet 1 verification commands and finish by stating whether Packet 1 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Packet 2 is unblocked.

Review subagent prompt:
Review the committed Slice 06 Packet 1 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md, and the official Lima sources required by Slice 06. Review only Packet 1 and the live diff. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 06 Packet 1 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md. Keep fixes limited to Packet 1 findings. Do not widen scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- List the official Lima sources used for the final readiness-order wording.
- State whether the touched file set stayed within the default execution boundary.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 06 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/config/environment-variables/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Packet 2 only: align helper scripts to routed-path-first proof.
- Do not start Packet 3.
- Keep the slice bounded to `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` plus narrowly required Slice 06 doc updates inside the spec stack only.

Before editing:
1. Read SPEC-06, PLAN-06, TASKS-06 and verify Packet 1 is already landed, committed, and checkpoint-green on the current tree.
2. Re-read milestone 1.3, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
3. Inspect the current readiness-proof script surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/platform/macos.rs
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/contracts/gateway/operator-contract.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/contracts/gateway/status-schema.md
4. Verify the official Lima sources above before preserving or reclassifying any guest-direct probe behavior.
5. Stay within the default execution boundary. Do not widen into Packet 3 doc cutover or later lifecycle/listener work.

Packet 2 scope:
- Task 2.1: Reframe `scripts/mac/lima-doctor.sh` around owned doctor proof.
- Task 2.2: Reframe `scripts/mac/smoke.sh` around routed smoke proof first.

Out of scope:
- Packet 3 or Packet 4 work
- top-level readiness docs
- `scripts/mac/lima-warm.sh`
- runtime-code edits unless a direct contradiction forces them and you surface that explicitly
- lifecycle redesign, listener hardening, or broad docs cutover

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- The implementation subagent must keep the script success/failure semantics routed-path-first and must preserve guest-direct probes only as explicit fallback or breakglass.
- After implementation, run the Packet 2 verification commands:
  - `bash -n scripts/mac/lima-doctor.sh`
  - `bash -n scripts/mac/smoke.sh`
  - `rg -n "host doctor|world doctor|limactl shell|systemctl|curl" scripts/mac/lima-doctor.sh`
  - `rg -n "gateway sync|gateway status|gateway restart|world doctor|limactl shell|systemctl|curl" scripts/mac/smoke.sh`
  - `git diff --stat -- scripts/mac/lima-doctor.sh scripts/mac/smoke.sh macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
- If implementation is green, commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-06 / PLAN-06 / TASKS-06, the official Lima sources, and the live diff.
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
- `scripts/mac/lima-doctor.sh` fails on routed readiness problems before guest-direct success can mask them
- `scripts/mac/smoke.sh` proves routed doctor/gateway readiness before guest-direct diagnosis
- guest-direct probes remain only as explicit fallback or breakglass
- the slice stayed within the Packet 2 script boundary

Implementation subagent prompt:
/goal Land Slice 06 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md first. Verify Packet 1 is already green. Work only on Task 2.1 and Task 2.2. Keep the work bounded to `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh`. Preserve guest-direct checks only as explicit fallback or breakglass. Do not widen into Packet 3 doc rewrites, runtime-code redesign, or later hardening work. Re-run the Packet 2 verification commands and finish by stating whether Packet 2 is checkpoint-green, what files changed, what verification ran, and whether Packet 3 is unblocked.

Review subagent prompt:
Review the committed Slice 06 Packet 2 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md, and the official Lima sources required by Slice 06. Review only Packet 2 and the live diff. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 06 Packet 2 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether routed-path failures can still be masked by guest-direct success or not.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 06 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/config/environment-variables/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Slice 06 Packet 3 only: cut over readiness-oriented docs and make small owned-doctor clarity changes only if required.
- Do not start Packet 4.
- Keep the slice bounded to readiness-oriented docs plus a narrowly justified `crates/shell/src/execution/platform/macos.rs` adjustment only if the cutover truly needs it.

Before editing:
1. Read SPEC-06, PLAN-06, TASKS-06 and verify Packets 1 and 2 are already landed, committed, and checkpoint-green on the current tree.
2. Re-read milestone 1.3, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
3. Inspect the current readiness-oriented doc and doctor surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/USAGE.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/contracts/gateway/operator-contract.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/contracts/gateway/status-schema.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/platform/macos.rs
4. If you need to edit any runtime symbol in `crates/shell/src/execution/platform/macos.rs`, run the GitNexus impact gate first using `GITNEXUS_HOME=/tmp/gitnexus-ff74-only` and surface any `HIGH` or `CRITICAL` results before proceeding.
5. Stay within the default execution boundary. Do not widen into Packet 4 validation-only work except where tiny cleanup is strictly required, and do not widen into later lifecycle/listener/docs-cutover slices.

Packet 3 scope:
- Task 3.1: Update readiness-oriented macOS docs to lead with owned CLI proof.
- Task 3.2: Make small owned-doctor clarity changes only if the cutover requires them.

Out of scope:
- Packet 4 work except tiny cleanup strictly required by Packet 3
- `scripts/mac/lima-warm.sh`
- backend transport or policy-carrier work
- listener hardening, ingress/mount hardening, or broad feature-wide docs cutover

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- If the implementation subagent edits `crates/shell/src/execution/platform/macos.rs`, it must run before editing:
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus status`
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus context host_doctor_main --repo substrate --file crates/shell/src/execution/platform/macos.rs`
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus context world_doctor_main --repo substrate --file crates/shell/src/execution/platform/macos.rs`
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus impact 'Function:crates/shell/src/execution/platform/macos.rs:host_doctor_main' --repo substrate --direction upstream --depth 3 --include-tests`
  - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus impact 'Function:crates/shell/src/execution/platform/macos.rs:world_doctor_main' --repo substrate --direction upstream --depth 3 --include-tests`
- After implementation, run the Packet 3 verification commands:
  - `rg -n "host doctor|world doctor|gateway status|gateway sync|gateway restart|limactl shell|systemctl|curl|journalctl" docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md`
  - `cargo test -p shell doctor_ok_json -- --nocapture`
  - `cargo test -p shell world_doctor_json_uses_override_vm_name -- --nocapture`
  - `cargo test -p shell world_doctor_json_reports_running_vm_with_inactive_service_as_not_provisioned -- --nocapture`
  - `git diff --stat -- docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md docs/USAGE.md docs/contracts/gateway/operator-contract.md docs/contracts/gateway/status-schema.md crates/shell/src/execution/platform/macos.rs macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
- If implementation is green, commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-06 / PLAN-06 / TASKS-06, the official Lima sources, any Packet 3 GitNexus outputs if runtime symbols changed, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, rerun any relevant GitNexus commands if touched runtime symbols changed, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not begin Packet 4 until Packet 3 is committed and review-clean.

Packet 3 checkpoint:
- the readiness-oriented docs lead with owned CLI doctor/gateway proof
- direct guest commands are clearly exceptional
- any `platform/macos.rs` change remained small and justified
- the slice did not drift into broader docs cutover work

Implementation subagent prompt:
/goal Land Slice 06 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md first. Verify Packets 1 and 2 are already green. Work only on Task 3.1 and Task 3.2. Keep the work bounded to readiness-oriented docs. Only touch `crates/shell/src/execution/platform/macos.rs` if the cutover truly needs clearer routed-versus-fallback doctor evidence, and if so run the Packet 3 GitNexus gate before editing. Do not widen into Packet 4 validation-only work, later lifecycle work, or broader docs-cutover work. Re-run the Packet 3 verification commands and finish by stating whether Packet 3 is checkpoint-green, what files changed, what verification ran, whether any GitNexus gate was triggered, and whether Packet 4 is unblocked.

Review subagent prompt:
Review the committed Slice 06 Packet 3 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md, the official Lima sources required by Slice 06, and the live diff. If `crates/shell/src/execution/platform/macos.rs` changed, include the Packet 3 GitNexus outputs in the review context. Review only Packet 3. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 06 Packet 3 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md. Keep fixes limited to Packet 3 findings. If runtime symbols change, rerun the relevant Packet 3 GitNexus commands before editing or before closeout as needed. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether direct guest readiness instructions still define the happy path or not.
- State whether `crates/shell/src/execution/platform/macos.rs` changed and, if so, summarize the GitNexus gate result.
- State whether Packet 4 is unblocked.
- If anything is not green, say explicitly that Packet 4 must not begin.
```

## Packet 4 Prompt

```text
/goal Land Slice 06 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://lima-vm.io/docs/config/port/
- https://lima-vm.io/docs/usage/ssh/
- https://lima-vm.io/docs/reference/limactl_shell/
- https://lima-vm.io/docs/config/environment-variables/
- https://lima-vm.io/docs/releases/breaking/

Mission:
- Land Slice 06 Packet 4 only: final validation and next-slice handoff clarity.
- Do not reopen Packet 1, Packet 2, or Packet 3 beyond tiny fixes strictly required by final validation.
- Keep the slice bounded to final verification, scoped cleanup, and an honest handoff boundary to Slice 07 and Slice 12.

Before editing:
1. Read SPEC-06, PLAN-06, TASKS-06 and verify Packets 1, 2, and 3 are already landed, committed, and checkpoint-green on the current tree.
2. Re-read EXECUTION-RUBRIC.md, ROADMAP.md, milestone 1.3, and the full Slice 06 doc set.
3. Confirm the default next seams are still:
   - Slice 07 for listener-surface hardening
   - Slice 12 for broader breakglass/docs cutover
4. If any runtime symbol in `crates/shell/src/execution/platform/macos.rs` changed during prior packets, be prepared to run `gitnexus_detect_changes()` before final commit.
5. Stay within the Slice 06 validation/handoff boundary unless a final contradiction forces a tiny fix.

Packet 4 scope:
- Task 4.1: Final targeted regression and scope check.
- Task 4.2: Record the handoff boundary honestly.

Out of scope:
- new feature work
- lifecycle redesign
- listener hardening implementation
- broader docs-cutover work
- any wide rewrite of prior packets

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- The implementation subagent may make only tiny fixes strictly required by final validation; otherwise it should remain validation-and-handoff only.
- After implementation, run the Packet 4 verification commands:
  - `bash -n scripts/mac/lima-doctor.sh scripts/mac/smoke.sh`
  - `cargo test -p shell doctor_ok_json -- --nocapture`
  - `cargo test -p shell world_doctor_json_uses_override_vm_name -- --nocapture`
  - `cargo test -p shell world_doctor_json_reports_running_vm_with_inactive_service_as_not_provisioned -- --nocapture`
  - `cargo test -p shell macos_gateway_client_endpoint -- --nocapture`
  - `git diff --stat -- scripts/mac/lima-doctor.sh scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md docs/USAGE.md docs/contracts/gateway/operator-contract.md docs/contracts/gateway/status-schema.md crates/shell/src/execution/platform/macos.rs macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
  - `gitnexus_detect_changes()` before committing if runtime symbols changed
- If implementation is green, commit the Packet 4 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-06 / PLAN-06 / TASKS-06, the official Lima sources, any Packet 4 `gitnexus_detect_changes()` output if runtime symbols changed, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, rerun `gitnexus_detect_changes()` if runtime symbols changed, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not begin any Slice 07 work until Packet 4 is committed and review-clean.

Packet 4 checkpoint:
- targeted verification is green
- the final diff remained readiness-scoped
- the handoff states clearly that Slice 06 landed only the readiness proof-story cutover
- Slice 07 and Slice 12 boundaries remain explicit

Implementation subagent prompt:
/goal Land Slice 06 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md first. Verify Packets 1, 2, and 3 are already green. Work only on Task 4.1 and Task 4.2. Keep the work validation-and-handoff only, with tiny fixes only if final verification strictly requires them. If runtime symbols changed earlier, run `gitnexus_detect_changes()` before final commit. Do not widen into Slice 07 or Slice 12 work. Re-run the Packet 4 verification commands and finish by stating whether Packet 4 is checkpoint-green, what files changed, what verification ran, whether detect-changes was needed, and whether Slice 06 is fully review-clean.

Review subagent prompt:
Review the committed Slice 06 Packet 4 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md, the official Lima sources required by Slice 06, and the live diff. If runtime symbols changed, include the Packet 4 `gitnexus_detect_changes()` output in the review context. Review only Packet 4. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 06 Packet 4 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md. Keep fixes limited to Packet 4 findings and tiny validation-required cleanup only. Re-run the relevant Packet 4 verification commands and rerun `gitnexus_detect_changes()` if runtime symbols changed. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the final diff remained readiness-scoped.
- State explicitly that Slice 07 remains next for listener-surface hardening and Slice 12 remains the broader breakglass/docs-cutover seam.
- State whether Slice 06 is fully review-clean.
- If anything is not green, say explicitly that no next-slice implementation should begin yet.
```
