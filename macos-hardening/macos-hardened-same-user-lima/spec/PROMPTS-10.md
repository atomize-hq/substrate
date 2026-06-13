# PROMPTS-10: Packet Orchestration Prompts For Slice 10

Source spec:
- [`SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md`](./SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md)

Source plan:
- [`PLAN-10.md`](./PLAN-10.md)

Source tasks:
- [`TASKS-10.md`](./TASKS-10.md)

Current branch at prompt authoring time: `HEAD`  
Worker implementation skill:
`/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill:
`/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is
bounded to one packet from Slice `10` and is grounded in the live Slice `10`
spec/plan/tasks stack, the current feature-local macOS hardening docs, the
current repo truth that still carries guest-unit authority drift, and the
official source set that Slice `10` requires.

Because Slice `10` is a source-driven guest-unit-authority and sandbox
unification slice, these prompts emphasize:

1. bounded unit-authority scope,
2. one canonical guest service/socket contract,
3. preservation of Slice `07` socket-first listener truth,
4. preservation of Slice `09` staged-workspace and guest-local writable-root
   truth,
5. strict deferral of Slice `11` lifecycle/sync productization and Slice `12`
   broad breakglass/docs cutover,
6. commit discipline between implementation, review, and fix rounds.

## Packet 1 Prompt

```text
/goal Land Slice 10 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://www.freedesktop.org/software/systemd/man/systemd.exec.html
- https://www.freedesktop.org/software/systemd/man/systemd.socket.html
- https://lima-vm.io/docs/reference/limactl_copy/
- https://lima-vm.io/docs/config/mount/

Mission:
- Land Packet 1 only: source gate, drift inventory, and canonical-source decision.
- Do not start Packet 2.
- Keep the slice bounded to freezing the authoritative guest-unit direction and
  inventorying the current bootstrap-vs-repair service/socket drift.

Before editing:
1. Read SPEC-10, PLAN-10, TASKS-10, EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 2 README, milestone 2.3, the Phase 3 README, milestone 3.1, DESIGN-macos-guest-unit-source-of-truth.md, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
2. Inspect the current repo-truth evidence in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/substrate.yaml
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
3. Verify the required official sources above before freezing any systemd sandbox or unit-authority claim.
4. Stay within the default execution boundary. Do not widen into Packet 2 implementation changes, Packet 3 validation/docs work, or later Slice 11 / Slice 12 work.

Packet 1 scope:
- Task 1.1: Confirm the authority stack, source gate, and live unit drift.
- Task 1.2: Freeze the canonical-source mechanism and target sandbox contract.

Out of scope:
- Packet 2, Packet 3, or Packet 4 work
- any `scripts/mac/lima/` implementation changes outside the Slice 10 planning docs
- Rust/backend code changes
- Phase 3 lifecycle/sync productization
- broad docs cutover or breakglass relabeling
- ingress redesign after Slice 09

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- The implementation subagent must keep the work limited to the Slice 10 planning docs and the source-gated unit-authority freeze.
- After implementation, run the Packet 1 verification commands:
  - `rg -n "substrate-world-service\.(service|socket)|ProtectHome=|ReadWritePaths=|RuntimeDirectory=|StateDirectory=|CapabilityBoundingSet=|AmbientCapabilities=|Environment=" scripts/mac/lima/substrate.yaml scripts/mac/lima-warm.sh`
  - `rg -n "staged-workspace|SUBSTRATE_HOME|SUBSTRATE_WORLD_SOCKET|WORLD_NETFILTER_ENABLE|ProtectHome|ReadWritePaths|CapabilityBoundingSet|AmbientCapabilities" scripts/mac/lima/substrate.yaml scripts/mac/lima-warm.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec`
  - `git status --short`
- If implementation is green, commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-10 / PLAN-10 / TASKS-10, the official sources above, and the live diff.
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
- the live hardening-critical unit drift is explicit
- the authoritative source mechanism is explicit
- the slice has not drifted into Packet 2/3 implementation work

Implementation subagent prompt:
/goal Land Slice 10 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md first. Also read EXECUTION-RUBRIC.md, ROADMAP.md, the Phase 2 README, milestone 2.3, the Phase 3 README, milestone 3.1, DESIGN-macos-guest-unit-source-of-truth.md, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md. Verify the required official source set before freezing any unit-authority or sandbox claim. Work only on Task 1.1 and Task 1.2. Keep the work bounded to the Slice 10 planning docs and the source-gated unit-authority freeze. Do not widen into Packet 2 implementation, Packet 3 validation/docs changes, or later Slice 11 / Slice 12 work. Re-run the Packet 1 verification commands and finish by stating whether Packet 1 is checkpoint-green, what files changed, what verification ran, what official sources were used, and whether Packet 2 is unblocked.

Review subagent prompt:
Review the committed Slice 10 Packet 1 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md, and the official source set required by Slice 10. Review only Packet 1 and the live diff. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 10 Packet 1 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md. Keep fixes limited to Packet 1 findings. Do not widen scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- List the official sources used for the final unit-authority and sandbox wording.
- State whether the touched file set stayed within the default execution boundary.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 10 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://www.freedesktop.org/software/systemd/man/systemd.exec.html
- https://www.freedesktop.org/software/systemd/man/systemd.socket.html
- https://lima-vm.io/docs/reference/limactl_copy/
- https://lima-vm.io/docs/config/mount/

Mission:
- Land Packet 2 only: canonical unit source and create/repair convergence.
- Do not start Packet 3.
- Keep the slice bounded to `scripts/mac/lima-warm.sh`, `scripts/mac/lima/substrate.yaml`, and the new canonical unit-source files under `scripts/mac/lima/`.

Before editing:
1. Read SPEC-10, PLAN-10, TASKS-10 and verify Packet 1 is already landed, committed, and checkpoint-green on the current tree.
2. Re-read milestone 2.3, the Phase 3 README, milestone 3.1, DESIGN-macos-guest-unit-source-of-truth.md, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
3. Inspect the current service/socket authority surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/substrate.yaml
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
4. Verify the required official sources above before preserving or changing any sandbox, capability, environment, or socket contract.
5. Stay within the default execution boundary. Do not widen into Packet 3 validation/docs work, Rust/backend code, or broader Phase 3 productization.

Packet 2 scope:
- Task 2.1: Introduce the canonical checked-in unit authority.
- Task 2.2: Remove bootstrap-vs-repair service drift.

Out of scope:
- Packet 3 or Packet 4 work
- `crates/world-service/` or other Rust/backend changes unless a direct contradiction forces a narrowly justified change
- broad docs work
- ingress redesign after Slice 09
- broad lifecycle/sync productization

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- The implementation subagent must preserve the Slice 07 socket-first listener result and the Slice 09 staged-workspace / guest-local writable-root result.
- After implementation, run the Packet 2 verification commands:
  - `rg -n "substrate-world-service\.(service|socket)|ProtectHome=|ReadWritePaths=|RuntimeDirectory=|StateDirectory=|SocketMode=|SocketGroup=" scripts/mac/lima scripts/mac/lima-warm.sh`
  - `rg -n "cat >/etc/systemd/system/substrate-world-service|tee /etc/systemd/system/substrate-world-service|ProtectHome=|ReadWritePaths=|CapabilityBoundingSet=|AmbientCapabilities=" scripts/mac/lima/substrate.yaml scripts/mac/lima-warm.sh scripts/mac/lima`
  - `bash -n scripts/mac/lima-warm.sh`
  - `git diff --stat -- scripts/mac/lima/substrate.yaml scripts/mac/lima-warm.sh scripts/mac/lima`
  - `git status --short`
- If implementation is green, commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-10 / PLAN-10 / TASKS-10, the official sources above, and the live diff.
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
- there is no longer dual handwritten guest unit authority
- the final service/socket contract is explicit and reviewable
- Slice 07 and Slice 09 results remain preserved
- the slice stayed within the Packet 2 implementation boundary

Implementation subagent prompt:
/goal Land Slice 10 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md first. Verify Packet 1 is already green. Work only on Task 2.1 and Task 2.2. Keep the work bounded to `scripts/mac/lima/substrate.yaml`, `scripts/mac/lima-warm.sh`, and the new canonical unit-source files under `scripts/mac/lima/`. Preserve the Slice 07 socket-first listener contract and the Slice 09 staged-workspace and guest-local writable-root contract. Do not widen into Packet 3 validation/docs changes, Rust/backend redesign, or later Slice 11 / Slice 12 work. Re-run the Packet 2 verification commands and finish by stating whether Packet 2 is checkpoint-green, what files changed, what verification ran, and whether Packet 3 is unblocked.

Review subagent prompt:
Review the committed Slice 10 Packet 2 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md, and the official source set required by Slice 10. Review only Packet 2 and the live diff. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 10 Packet 2 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether there is any remaining bootstrap-vs-repair unit drift or not.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 10 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://www.freedesktop.org/software/systemd/man/systemd.exec.html
- https://www.freedesktop.org/software/systemd/man/systemd.socket.html
- https://lima-vm.io/docs/reference/limactl_copy/
- https://lima-vm.io/docs/config/mount/

Mission:
- Land Packet 3 only: validation parity and minimal doc-truth cutover.
- Do not start Packet 4.
- Keep the slice bounded to `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`, `docs/WORLD.md`, and `docs/reference/world/platforms/macos-lima-setup.md`.

Before editing:
1. Read SPEC-10, PLAN-10, TASKS-10 and verify Packets 1 and 2 are already landed, committed, and checkpoint-green on the current tree.
2. Re-read milestone 2.3, the Phase 3 README, milestone 3.1, DESIGN-macos-guest-unit-source-of-truth.md, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
3. Inspect the current validation and doc-truth surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/
4. Verify the required official sources above before preserving or changing any claim about authoritative unit parity, sandbox semantics, or guest-visible writable paths.
5. Stay within the default execution boundary. Do not widen into Packet 4 closeout work, Rust/backend changes, or broader Phase 3 productization.

Packet 3 scope:
- Task 3.1: Add rendered-unit parity validation to doctor/smoke surfaces.
- Task 3.2: Perform only the minimal doc truth corrections required by unit unification.

Out of scope:
- Packet 4 work
- broader docs cutover
- lifecycle/sync productization
- ingress redesign
- unrelated cross-platform service-installation work
- Rust/backend changes unless a direct contradiction forces a narrowly justified change

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- The implementation subagent must keep the routed proof priority intact while adding rendered-unit parity validation.
- After implementation, run the Packet 3 verification commands:
  - `rg -n "substrate-world-service\.(service|socket)|ProtectHome|ReadWritePaths|RuntimeDirectory|StateDirectory|SocketMode|SocketGroup|systemctl cat|systemctl show|cmp|sha256sum" scripts/mac/lima-doctor.sh scripts/mac/smoke.sh`
  - `bash -n scripts/mac/lima-doctor.sh`
  - `bash -n scripts/mac/smoke.sh`
  - `rg -n "substrate-world-service\.(service|socket)|ProtectHome|ReadWritePaths|SUBSTRATE_HOME|WORLD_NETFILTER_ENABLE|authoritative|canonical" docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md`
  - `git diff --stat -- scripts/mac/lima-doctor.sh scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md`
  - `git status --short`
- If implementation is green, commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-10 / PLAN-10 / TASKS-10, the official sources above, and the live diff.
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
- doctor and/or smoke can prove rendered-unit parity
- the docs no longer describe dual unit authority as acceptable
- routed proof priority is preserved
- the slice stayed within the Packet 3 validation/docs boundary

Implementation subagent prompt:
/goal Land Slice 10 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md first. Verify Packets 1 and 2 are already green. Work only on Task 3.1 and Task 3.2. Keep the work bounded to `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`, `docs/WORLD.md`, and `docs/reference/world/platforms/macos-lima-setup.md`. Preserve routed proof priority while adding rendered-unit parity validation. Do not widen into Packet 4 closeout work, Rust/backend redesign, or later Slice 11 / Slice 12 work. Re-run the Packet 3 verification commands and finish by stating whether Packet 3 is checkpoint-green, what files changed, what verification ran, and whether Packet 4 is unblocked.

Review subagent prompt:
Review the committed Slice 10 Packet 3 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md, and the official source set required by Slice 10. Review only Packet 3 and the live diff. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 10 Packet 3 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether rendered-unit parity is now proven by the supported validation surfaces or not.
- State whether Packet 4 is unblocked.
- If anything is not green, say explicitly that Packet 4 must not begin.
```

## Packet 4 Prompt

```text
/goal Land Slice 10 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md

Required official source set for this packet:
- https://www.freedesktop.org/software/systemd/man/systemd.exec.html
- https://www.freedesktop.org/software/systemd/man/systemd.socket.html
- https://lima-vm.io/docs/reference/limactl_copy/
- https://lima-vm.io/docs/config/mount/

Mission:
- Land Slice 10 Packet 4 only: final validation and downstream handoff.
- This packet is closeout only; do not reopen Packets 1-3 except for narrowly required fixes uncovered by rerun verification.
- Keep the slice bounded to the final coherence pass, rerun verification wall, and explicit handoff wording to Slice 11 and Slice 12.

Before editing:
1. Read SPEC-10, PLAN-10, TASKS-10 and verify Packets 1-3 are already landed, committed, and checkpoint-green on the current tree.
2. Re-read milestone 2.3, the Phase 3 README, milestone 3.1, DESIGN-macos-guest-unit-source-of-truth.md, DESIGN-macos-ingress-and-mount-contract.md, DESIGN-macos-lima-transport-contract.md, DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
3. Inspect the final closeout surfaces in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/substrate.yaml
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-warm.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima/
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/lima-doctor.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/scripts/mac/smoke.sh
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/WORLD.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/docs/reference/world/platforms/macos-lima-setup.md
4. Verify the required official sources above before making any final claim about the landed unit source-of-truth or sandbox contract.
5. Stay within the default execution boundary. Do not widen into Slice 11 productization or Slice 12 broad docs/breakglass cutover.

Packet 4 scope:
- Task 4.1: Final scope and coherence check.
- Task 4.2: Record the downstream handoff honestly.

Out of scope:
- new feature work
- broad rewrites of the earlier packets
- lifecycle/sync productization
- broad breakglass/docs cutover
- unrelated Rust/backend or cross-platform work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- The implementation subagent may make only narrowly required fixes discovered by the rerun verification wall; otherwise keep the packet to closeout wording and coherence.
- After implementation, run the Packet 4 verification commands:
  - `git diff --stat -- scripts/mac/lima/substrate.yaml scripts/mac/lima-warm.sh scripts/mac/lima scripts/mac/lima-doctor.sh scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md`
  - `git status --short`
  - rerun the Packet 2 verification commands for `scripts/mac/lima/substrate.yaml`, `scripts/mac/lima-warm.sh`, and the canonical unit-source files
  - rerun the Packet 3 verification commands for `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`, `docs/WORLD.md`, and `docs/reference/world/platforms/macos-lima-setup.md`
  - `scripts/mac/lima-warm.sh --check-only`
  - `scripts/mac/lima-doctor.sh`
- If implementation is green, commit the Packet 4 closeout work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-10 / PLAN-10 / TASKS-10, the official sources above, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.
- Do not begin Slice 11 work until Packet 4 is committed and review-clean.

Packet 4 checkpoint:
- the final diff stays within the allowed execution boundary unless a narrowly justified fix was required
- the slice does not overclaim broader Phase 3 productization
- the closeout ties checkpoint-green status to the rerun verification wall
- Slice 11 and Slice 12 boundaries remain explicit

Implementation subagent prompt:
/goal Land Slice 10 Packet 4 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md first. Verify Packets 1-3 are already green. Work only on Task 4.1 and Task 4.2. Keep this packet to final validation and downstream handoff, except for narrowly required fixes discovered by the rerun verification wall. Do not widen into Slice 11 productization, Slice 12 broad docs cutover, or unrelated redesign. Re-run the Packet 4 verification commands and finish by stating whether Packet 4 is checkpoint-green, what files changed, what verification ran, whether the touched file set stayed in bounds, and whether Slice 11 is unblocked.

Review subagent prompt:
Review the committed Slice 10 Packet 4 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md, and the official source set required by Slice 10. Review only Packet 4 and the live diff. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 10 Packet 4 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the final closeout honestly keeps Slice 11 and Slice 12 deferred.
- State whether Slice 11 is unblocked.
- If anything is not green, say explicitly that Slice 11 must not begin.
```
