EVIDENCE TASK IDENTITY BINDING AND READ-ONLY START AUTHORITY

Bind this exact identity to the initialization-barrier dispatch already in your task:

- evidence_task_thread_id: `019fd9ad-3edc-7c52-b107-ecc814f1c97d`
- evidence_task_host_id: `local`
- exact assigned worktree: `/Users/spensermcconnell/.codex/worktrees/2fdb/substrate`
- orchestration_id: `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d`
- evidence_id: `EVIDENCE:R3-MAC-IMP-01`
- dispatch_nonce: `1669f74ede86799f7097bf2a2317d1b5f44f535bf0fa10171d500cfe4cf4e3a4`
- meta return task: `019fd8c3-b12c-7e73-919f-898600ab0f64` on `local`
- source implementation task: `019fd97a-3740-7261-a7e8-0008845709b5` on `local`
- model/reasoning: `gpt-5.6-terra` / Extra High (`xhigh`)

The worktree was created detached at exact commit
`d8a65fc8890dd37584aaeac2984c906188e5f06e` and tree
`d8f25cc993f264f0c67fe3e00180eb393e41b2e3`; it was clean before and after skill hydration. Live
`origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap` was independently
rechecked equal to that commit immediately before this binding.

The repository-local `orchestrate-top-level-tasks` skill was hydrated and independently verified:

- source: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.agents/skills/orchestrate-top-level-tasks`
- target: `/Users/spensermcconnell/.codex/worktrees/2fdb/substrate/.agents/skills/orchestrate-top-level-tasks`
- file count: `26`
- tree digest: `61d99190f680d3738fa824348e671e8eb84b2729bf31952ef16031c6ba24d903`

The bound contract digest is
`sha256:57a0ea338c678410ed862115c4d8201e245bada34c05b1bc3c6fd7c2c9635226`; the rendered initial
prompt digest is `sha256:8e46bb78271b09d045b466813cb6451910de5f4552aecf9fed91f4fd99fa940b`.
The durable binding record is at
`/Users/spensermcconnell/.codex/worktrees/eb49/substrate/orchestration/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/increments/EVIDENCE-R3-MAC-IMP-01.binding.json`.

Start authority is now granted for this read-only native evidence task only. Re-verify the full
binding, live remote, source checkout, platform prerequisites, external artifact boundary, and
operator/TTY boundary before actions. Never mutate either the assigned source checkout or the
protected saved checkout. Do not spawn subagents. If the live human pairing boundary or another
required prerequisite is unavailable, do not weaken the gate: return the exact structured
`BLOCKED_PLATFORM_HANDOFF_REQUIRED` receipt and complete handoff. If the available native run
fails or restoration is inexact, return `BLOCKED_NATIVE_EVIDENCE`.

On success, validate the external artifact and evidence receipt exactly as specified, and make
the receipt send to the meta task your final tool action. Do not begin MAC-CLOSEOUT, WIN, UNIX, or
any final native gate.
