# Handoff: SPEC-62 World Worker Retained Debug

## Session Metadata
- Created: 2026-06-22 21:14:55
- Project: /home/spenser/__Active_code/substrate
- Branch: feat/internal-host-orchestrator-world-dispatch-bootstrap
- Session duration: ~2.5 hours across investigation, docs/test updates, and live repro analysis

### Recent Commits (for context)
  - 230e6d8a fix: hide private codex bridge env details
  - 841b6079 fix: hide private codex bridge env details
  - c3c482c0 fix: hide private codex bridge env details
  - 4cbe9cd1 fix: fail closed on missing codex bridge input
  - 8f5905dc fix: mark codex workspaces untrusted

## Handoff Chain

- **Continues from**: None (fresh start)
- **Supersedes**: None

> This is the first handoff for this task.

## Current State Summary

We were debugging the remaining failure mode in `llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md` after the June 21 direct `cli:codex-world` bootstrap/model issue was already fixed. The important nuance is that plain `substrate` world sessions and host-orchestrated world members are not proving the same thing. Plain `substrate` and `substrate -c` clearly persist files in a reusable world overlay (`note.md` is visible from later world sessions but not the host), while host-orchestrated `spawn_world_worker` / `continue_world_worker` is failing on the retained follow-up path with `agent_wrapper_error` before useful work completes. Current best diagnosis: the remaining bug is in the retained Codex world-member lifecycle/continuation contract, not a general inability for world sessions to write files.

## Codebase Understanding

### Architecture Overview

There are two materially different Linux world paths relevant to this bug. First, generic `substrate` command execution uses a reusable overlay-backed world rooted at the current workspace/anchor and persists writes inside the world until explicit reconciliation. Second, host-orchestrated world-member dispatch uses the world-service member runtime seam: `spawn_world_worker` goes through `crates/shell/src/execution/orchestrator_world_dispatch.rs` into `crates/world-service/src/member_runtime.rs`, which launches a world-scoped Codex runtime, captures a surfaced `uaa_session_id`, and later routes `continue_world_worker` through `MemberTurnSubmitRequestV1` plus `agent_api.session.resume.v1`. The shell-side public tooling proves transport and typed request shape, but does not yet prove that a real Codex-backed retained world member can survive a clean bootstrap completion and be resumed correctly later.

### Critical Files

| File | Purpose | Relevance |
|------|---------|-----------|
| `crates/world-service/src/member_runtime.rs` | World-member bootstrap and submitted-turn lifecycle | Central to retained worker registration, `uaa_session_id` capture, submit-turn resume behavior, and current suspected lifecycle mismatch |
| `crates/shell/src/execution/orchestrator_world_dispatch.rs` | Host orchestration entrypoint for `run_world_task`, `spawn_world_worker`, `continue_world_worker`, `stop_world_worker` | Proves where retained bootstrap exits are accepted, where follow-up submit is issued, and where higher-level assumptions live |
| `crates/shell/src/execution/prompt_fulfillment.rs` | Host toolbox preamble and injected toolbox env | Important because toolbox injection was suspected as a cause and then ruled out |
| `crates/world-service/tests/member_runtime_world_placement_v1.rs` | Linux regression coverage for member runtime placement | Added regression to pin full-isolation placement semantics discovered during investigation |
| `docs/USAGE.md` | User-facing world/sync behavior docs | Updated to document that full-isolation member writes are not host-visible until reconciliation and host-absolute paths are invalid write targets |
| `llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md` | Working spec / manual smoke record | Updated to reflect sync/reconciliation reality instead of expecting direct host-visible writes |
| `~/.substrate/run/agent-hub/sessions/019ef14b-c425-7a73-810f-7e610da233ff/session.json` | User’s live orchestration session state | Confirms authoritative `world_id`, `world_generation`, and allowed world backend |
| `~/.substrate/run/agent-hub/sessions/019ef14b-c425-7a73-810f-7e610da233ff/participants/*.json` | Persisted retained worker records | Showed workers with surfaced `uaa_session_id`, including one still marked `ready` after failed follow-up |

### Key Patterns Discovered

- Linux world behavior is often split between “generic shell world reuse” and “shared orchestration world reuse”; similar symptoms can come from different layers.
- The world-service member runtime currently models retention in-process via `active_members`, while public orchestration status is persisted separately in shell-side participant records.
- Plain `substrate` overlay persistence does not imply host visibility; host visibility depends on a reconciliation path.
- The shell tests around public world follow-up mostly prove typed routing and request shape, not a real Codex resume lifecycle.
- This repo has both product/spec docs and regression tests living close to implementation; when a behavior is uncovered, pinning it in docs plus a test is expected even before the deeper product fix lands.

## Work Completed

### Tasks Finished

- [x] Confirmed that `run_world_task` host-tool dispatch reaches `cli:codex-world`, but file side effects are not host-visible through that path today.
- [x] Identified and documented the full-isolation placement behavior: relative writes land in the authoritative overlay, while host-absolute writes fail.
- [x] Added a Linux regression test in `crates/world-service/tests/member_runtime_world_placement_v1.rs` to pin those placement semantics.
- [x] Updated `docs/USAGE.md` and the SPEC-62 closeout notes to reflect overlay/sync reality instead of direct host-visible writes.
- [x] Reproduced and analyzed the newer retained-worker failure mode where `continue_world_worker` fails with `agent_wrapper_error` before returning useful output.
- [x] Proved that plain `substrate` / `substrate -c` world sessions persist overlay writes (`note.md`) even outside declared workspace sync.
- [x] Proved that raw world-deps Codex (`/var/lib/substrate/world-deps/bin/codex`) can bootstrap and later `resume` successfully in isolation, including with bogus toolbox env injected.

### Files Modified

| File | Changes | Rationale |
|------|---------|-----------|
| `llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md` | Updated manual smoke expectations to use relative world writes plus reconciliation instead of assuming immediate host-visible writes | Keep the spec aligned with the actual full-isolation contract already present in code |
| `docs/USAGE.md` | Added note about full-isolation member writes becoming host-visible only after sync/reconciliation, and about host-absolute write targets being invalid from inside the member runtime | Prevent future operator confusion from the same symptom |
| `crates/world-service/tests/member_runtime_world_placement_v1.rs` | Added helper plus Linux regression asserting relative writes land in overlay and host-absolute writes fail under full isolation | Lock in the nuanced placement behavior uncovered by live debugging |
| `AGENTS.md` | Pre-existing user modification, not changed as part of this debugging work | Included by the scaffold because it was already dirty in the worktree |
| `CLAUDE.md` | Pre-existing user modification, not changed as part of this debugging work | Included by the scaffold because it was already dirty in the worktree |

### Decisions Made

| Decision | Options Considered | Rationale |
|----------|-------------------|-----------|
| Treat plain `substrate` overlay persistence and retained world-member failure as separate layers | Collapse them into one “world writes broken” diagnosis, or split them | The logs show plain world sessions persist fine while retained worker follow-up fails earlier at the agent-wrapper layer |
| Do not conclude that host orchestration tools are generally unusable in world | Broad failure claim vs narrower retained-path bug | `run_world_task` transport works, plain world overlay writes work, and raw world Codex resume works; the broken area is narrower |
| Add a regression for placement semantics before fixing retained lifecycle | Wait for full retained-worker fix first | The placement contract had already been discovered and was worth pinning independently |
| Use `.claude/handoffs` instead of default `.codex/handoffs` | Force default path or manually create elsewhere | In this repo `.codex` is a file, so the default handoff path is invalid |

## Pending Work

### Immediate Next Steps

1. Trace the retained-member lifecycle in `crates/world-service/src/member_runtime.rs` and decide whether a clean bootstrap completion should keep the member retained by `uaa_session_id` instead of unregistering it from `active_members`.
2. Add or extend a real-world regression around `spawn_world_worker` followed by `continue_world_worker` using a Codex-like runtime that exits after bootstrap and resumes by session handle, because current tests mainly prove typed submit routing.
3. Reconcile product behavior around overlay-to-host sync outside declared workspaces: either enable authoritative reconciliation without `workspace init` or clearly separate “persistent world overlay” from “syncable workspace”.

### Blockers/Open Questions

- [ ] Does `MemberRuntimeManager::launch` incorrectly assume the bootstrap process itself remains authoritative/live for retention, even though real `codex exec` exits after the first turn?
- [ ] Is shell-side persisted participant state overstating liveness/readiness compared with world-service runtime state, creating a stale “ready” retained worker record?
- [ ] Should `spawn_world_worker` wait for a different closeout condition than first `registered` event before claiming the worker is ready for `continue_world_worker`?
- [ ] Should workspace reconciliation support plain overlay worlds that were never initialized with `substrate workspace init`, given that those overlays already persist writes?

### Deferred Items

- Validate or redesign `run_world_task` host-visible filesystem reconciliation. Deferred because the more urgent break is the retained world-worker follow-up path.
- Any broad documentation cleanup beyond `docs/USAGE.md` and the SPEC. Deferred until retained lifecycle semantics are finalized.

## Context for Resuming Agent

### Important Context

Do not resume this work from the assumption that “world writes are broken.” That is too broad and it will send you down the wrong path. The strongest current evidence is:

1. Plain `substrate` and `substrate -c` commands are definitely running inside a reusable world overlay, not on the host. The user proved this with:
   - `which git` inside plain `substrate` returning `/usr/sbin/git`
   - host `which git` returning `/usr/bin/git`
   - `touch note.md` inside `substrate`, then host `ls` not showing it, then later `substrate` sessions still seeing `note.md`
2. The sync gap is real but separate:
   - `substrate workspace sync --dry-run` fails outside initialized workspaces with `substrate: not in a workspace for workspace sync; run \`substrate workspace init\``
   - this means persistent overlay state can exist without a usable host reconciliation command
3. The newer retained-worker failure is not the same as the earlier absolute-path placement failure:
   - earlier retained worker error: host-absolute target path was read-only from inside the world
   - newer failure: even relative-path continue attempts die with `code: agent_wrapper_error` / `codex exited non-zero: ExitStatus(unix_wait_status(256)) (stderr redacted)`
4. Raw Codex resume is not the blocker:
   - `/var/lib/substrate/world-deps/bin/codex` is `0.125.0`
   - host `codex` is `0.141.0`
   - despite the older version, direct isolated probes showed `exec` followed by `resume <session_id>` works successfully under an isolated `CODEX_HOME`
   - it also still works when `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` / version env are injected and the host toolbox contract preamble is present
5. The likely mismatch is lifecycle, not prompt content:
   - world-service `MemberRuntimeManager::launch` registers an `ActiveMemberRuntime`
   - when bootstrap completion resolves, it currently calls `manager.unregister_member(&participant_id);`
   - shell/public orchestration, however, persists retained worker state separately and can still show a worker as `ready`
   - fake tests keep the bootstrap runtime parked/alive, but real `codex exec` exits after finishing a turn and resumes later by session handle

The next agent should focus on whether retention should be modeled by live bootstrap process, by durable surfaced session handle, or by a hybrid contract. That is the most promising path.

### Assumptions Made

- The June 21 direct `cli:codex-world` bootstrap/model failure is already fixed and is not the cause of the current retained-worker issue.
- Session ids, participant ids, and world ids in the user repro are stable enough to use for forensic debugging in local state files.
- The current raw `codex` CLI behavior observed in isolated probes is representative enough to reason about the retained-member lifecycle mismatch.

### Potential Gotchas

- `.codex` in this repo is a file, not a directory. Any script defaulting to `.codex/handoffs` will fail unless you override to `.claude/handoffs`.
- `run_world_task` and retained worker paths have different failure semantics; do not generalize from one to the other.
- A persisted participant JSON showing `state: ready` does not necessarily prove the underlying world-service retained member is still active/live in memory.
- The shell tests in `crates/shell/tests/agent_public_control_surface_v1.rs` are helpful but can mislead because the fake Codex scripts do not perfectly model real Codex process exit/resume behavior.
- `workspace sync` behavior is easy to misread: refusal outside a workspace does not mean the world write never happened.

## Environment State

### Tools/Services Used

- `substrate` CLI on Linux, with world isolation and overlay-backed execution
- Host orchestration session `019ef14b-c425-7a73-810f-7e610da233ff`
- User’s live trace log at `~/.substrate/trace.jsonl`
- World-deps Codex binary `/var/lib/substrate/world-deps/bin/codex` (`codex-cli 0.125.0`)
- Host Codex binary `codex` on PATH (`codex-cli 0.141.0`)
- Session-handoff helper scripts under `/home/spenser/.codex/skills/session-handoff/session-handoff/scripts/`

### Active Processes

- No intentional long-running debug process was left behind from this agent session.
- The user’s broader Substrate environment may still have active orchestration/session state under `~/.substrate/run/agent-hub/sessions/019ef14b-c425-7a73-810f-7e610da233ff/`; verify liveness before assuming any retained participant is still active.

### Environment Variables

- `CODEX_HOME`
- `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`
- `SUBSTRATE_AGENT_TOOLBOX_VERSION`
- `SUBSTRATE_HOME`
- `SUBSTRATE_WORLD_SOCKET`
- `SUBSTRATE_WORLD_ENTRY_BINARY`
- `SUBSTRATE_WORLD_ENTRY_WORKING_DIR`
- `SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH`
- `SUBSTRATE_WORLD_ENTRY_REQUIRE_CGROUP_ATTACH`
- `HOME`
- `XDG_CONFIG_HOME`
- `XDG_DATA_HOME`
- `XDG_CACHE_HOME`

## Related Resources

- `llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md`
- `docs/USAGE.md`
- `crates/world-service/src/member_runtime.rs`
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
- `crates/shell/src/execution/prompt_fulfillment.rs`
- `crates/shell/tests/agent_public_control_surface_v1.rs`
- `crates/world-service/tests/member_runtime_world_placement_v1.rs`
- `~/.substrate/run/agent-hub/sessions/019ef14b-c425-7a73-810f-7e610da233ff/session.json`
- `~/.substrate/run/agent-hub/sessions/019ef14b-c425-7a73-810f-7e610da233ff/participants/`

---

**Security Reminder**: Before finalizing, run `validate_handoff.py` to check for accidental secret exposure.
