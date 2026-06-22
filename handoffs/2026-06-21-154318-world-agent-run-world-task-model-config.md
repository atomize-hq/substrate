# Handoff: World Agent `run_world_task` Model Config Diagnosis

## Session Metadata
- Created: 2026-06-21 15:43:18
- Project: /home/spenser/__Active_code/substrate
- Branch: feat/internal-host-orchestrator-world-dispatch-bootstrap
- Session duration: about 1 hour

### Recent Commits (for context)
  - 87129df3 chore: format slice 61 packet 4 control test
  - e974d6c7 fix: keep structured stderr fallback visible
  - e4ead231 fix: bound structured prompt fallback output
  - 57aa7ecc fix: harden public prompt structured fallback
  - 0bb33fef chore: refresh gitnexus stats and format tests

## Handoff Chain

- **Continues from**: [2026-06-21-135249-world-agent-run-world-task-auth-seeding.md](./2026-06-21-135249-world-agent-run-world-task-auth-seeding.md)
  - Previous title: World Agent `run_world_task` Auth Seeding
- **Supersedes**: None

> Read the previous handoff first for the broader auth-seeding investigation. This handoff captures the live rebuilt smoke, validates that the prior patch was active, and narrows the remaining failure to missing isolated-home config rather than missing auth state.

## Current State Summary

The user rebuilt and reran the public bootstrap smoke with `~/.substrate/bin/substrate agent start --backend cli:codex-world --scope world ...`. The host orchestrator successfully created a world-bound session, used the toolbox Unix socket directly, and successfully dispatched `run_world_task` to `cli:codex-world`. The world member still failed with exit status `1` even for a trivial `Reply with READY only.` probe. This session confirmed that the installed `substrate` and `substrate-world-service` binaries already contain the June 21 auth-seeding patch, so the failure is not stale deployment. The strongest new root cause is that the patch seeds `auth.json` into isolated `CODEX_HOME` but does not seed `config.toml`, causing direct Codex world-member runs to fall back to the default model `gpt-5.3-codex`, which the current ChatGPT-backed Codex auth on this machine cannot use.

## Codebase Understanding

### Architecture Overview

There are still two distinct runtime paths that matter:

1. **Managed gateway runtime path**:
   - `substrate world gateway sync|status|restart`
   - `world-service` prepares `GatewayRuntimeStartContext`
   - integrated auth is validated on the request
   - `world-service` passes auth through `SUBSTRATE_LLM_AUTH_BUNDLE_FD`
   - the managed `substrate-gateway` server process reads the FD once at startup

2. **Direct member-dispatch path**:
   - host orchestrator -> toolbox Unix socket -> `run_world_task` / `spawn_world_worker`
   - shell builds `member_dispatch`
   - `world-service` routes straight into `member_runtime.launch(...)`
   - member runtime uses `PromptFulfillmentBridge::for_member_backend(...)`
   - direct Codex runtime uses isolated `CODEX_HOME`
   - this path does not carry `integrated_auth`, does not use the FD auth handoff, and does not go through the managed server startup seam

The June 21 patch only addressed one missing input for the direct member path: it copied login state from host `~/.codex/auth.json` into a temporary isolated `CODEX_HOME`. This session proved that direct Codex execution also depends on the model/runtime config in `~/.codex/config.toml`, at least on this machine and Codex build.

### Critical Files

| File | Purpose | Relevance |
|------|---------|-----------|
| [crates/world-service/src/member_runtime.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/member_runtime.rs) | Direct world member launch path for `member_dispatch` | Current failing seam; seeds `auth.json` into isolated `CODEX_HOME` but does not copy `config.toml` |
| [crates/world-service/src/prompt_fulfillment.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/prompt_fulfillment.rs) | World-member prompt bridge into `substrate_gateway` adapter runtime | Confirms the member path uses the adapter runtime seam directly |
| [crates/gateway/src/adapter_runtime.rs](/home/spenser/__Active_code/substrate/crates/gateway/src/adapter_runtime.rs) | In-process backend registration seam for Codex / Claude Code | Clarifies that prompt runs use gateway crate code without traversing the managed server startup/auth-bundle seam |
| [crates/world-service/src/gateway_runtime.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/gateway_runtime.rs) | Managed gateway server lifecycle | Confirms the FD auth-bundle handoff exists, but only on managed gateway runtime startup |
| [crates/gateway/src/server/mod.rs](/home/spenser/__Active_code/substrate/crates/gateway/src/server/mod.rs) | Managed `substrate-gateway` server startup auth-bundle reader | Reads `SUBSTRATE_LLM_AUTH_BUNDLE_FD`; not on the failing direct member path |
| [crates/transport-api-types/src/lib.rs](/home/spenser/__Active_code/substrate/crates/transport-api-types/src/lib.rs) | Shared request schema, including `MemberDispatchRequestV1` and gateway request types | Proves `member_dispatch` carries no `integrated_auth` payload |
| [crates/shell/src/execution/routing/dispatch/world_ops.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | Shell-side `member_dispatch` request builder | Injects `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME` only when policy allows host credential reads for `cli:codex-world` |
| [handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md](/home/spenser/__Active_code/substrate/handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md) | Previous auth-seeding handoff | Use this for prior diagnosis, patch context, and distinctions between gateway lifecycle auth and direct member runtime auth |
| [~/.codex/config.toml](/home/spenser/.codex/config.toml:1) | User’s real Codex runtime configuration | Contains `model = "gpt-5.4"`; this turned out to be the missing input the isolated world-member `CODEX_HOME` did not receive |

### Key Patterns Discovered

- `member_dispatch` and gateway lifecycle are separate request families with separate auth carriers. Do not assume “uses gateway crate code” means “uses managed gateway server runtime”.
- The installed runtime can be current even if the observed live failure is unchanged. This session confirmed current deployment by checking binary strings for `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME` and `codex-home`.
- On this machine, `auth.json` alone is enough for `codex login status`, but not enough for `codex exec` to succeed under an isolated `CODEX_HOME`.
- The direct world-member failure mode can masquerade as generic exit `1`; the quickest way to narrow it is to reproduce with an isolated `CODEX_HOME` outside Substrate and compare `login status` vs `exec`.
- The orchestrator session prefix `[codex-host]` in public output is about the owning session backend, not proof that the inner `run_world_task` never reached `cli:codex-world`.

## Work Completed

### Tasks Finished

- [x] Reviewed the user’s live rebuilt smoke output and confirmed the toolbox request itself was correct
- [x] Verified the active session record was world-bound and not a `missing_world_binding` failure
- [x] Verified the installed `~/.substrate/bin/substrate` and `/usr/local/bin/substrate-world-service` binaries contain the June 21 auth-seeding patch strings
- [x] Verified the live process handling the smoke was the installed systemd `substrate-world-service`, not a stale debug binary
- [x] Confirmed the direct member path still bypasses the managed gateway startup/auth-bundle seam
- [x] Inspected the real `~/.codex` layout and confirmed `auth.json` exists while `.credentials.json` does not
- [x] Reproduced the failure outside Substrate with an isolated temp `CODEX_HOME` seeded with only `auth.json`
- [x] Proved that isolated `CODEX_HOME` + `auth.json` succeeds for `codex login status` but fails for `codex exec` with model incompatibility
- [x] Proved that isolated `CODEX_HOME` + `auth.json` + `config.toml` makes `codex exec` succeed

### Files Modified

| File | Changes | Rationale |
|------|---------|-----------|
| [handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md](/home/spenser/__Active_code/substrate/handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md) | New continuation handoff | Preserve the live-smoke diagnosis and the isolated-home `config.toml` finding |

No code files were edited in this session. The existing worktree changes are still the earlier June 21 auth-seeding patch set plus repo-local dirty files.

### Decisions Made

| Decision | Options Considered | Rationale |
|----------|-------------------|-----------|
| Treat the rebuilt smoke as a live validation of the existing patch rather than assuming stale deployment | Assume stale binaries; verify installed service and binaries; verify by rerunning from source | The installed binaries and active systemd service already contained the new seed-home strings, so the unchanged failure had to be a real remaining bug |
| Reproduce direct Codex behavior outside Substrate with isolated `CODEX_HOME` | Keep reasoning from logs only; inspect systemd/journal only; run an isolated local probe | The isolated probe cleanly separated login-state sufficiency from execution-state sufficiency and produced the actionable root cause quickly |
| Prioritize missing `config.toml` over missing `.credentials.json` as the immediate blocker | Require `.credentials.json`; assume auth.json alone is enough; test real home contents | The real home does not have `.credentials.json`, yet `login status` works and `exec` succeeds once `config.toml` is copied, so `.credentials.json` is not the blocking input here |
| Keep the larger architectural concern separate from the immediate patch fix | Pivot immediately to reworking member dispatch through the managed gateway server; patch the direct member path enough to make the smoke pass first | The user already agrees the current auth-seeding posture is architecturally wrong; the next practical unblock is still to make the current patch actually function before or while redesign proceeds |

## Pending Work

### Immediate Next Steps

1. Patch [crates/world-service/src/member_runtime.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/member_runtime.rs) so the isolated `CODEX_HOME` seeding copies `config.toml` from host `~/.codex` alongside `auth.json` when available.
2. Add targeted coverage proving direct Codex member bootstrap fails with only `auth.json` on this machine profile and succeeds when `config.toml` is present, or at minimum add a unit test for config copy semantics if the external behavior cannot be exercised in test.
3. Rebuild and redeploy, then rerun the exact public bootstrap smoke the user pasted and confirm `from_the_world_worker.md` is created.

### Blockers/Open Questions

- [ ] Open question: whether the direct member path should also copy any other Codex runtime files beyond `config.toml` and `auth.json` for near-term stability, or whether that would drift further from the intended architecture.
- [ ] Open question: whether the direct member path should explicitly inject a validated model instead of copying `config.toml`, if copying the whole user config is considered too broad.
- [ ] Architectural blocker: the current functioning path would still violate the intended design, because world CLI agents are supposed to consume Substrate-owned auth via one-time handoff and then route through the in-world `substrate-gateway`, not rely on copied local Codex home state.

### Deferred Items

- Reworking `run_world_task` / `spawn_world_worker` to route through the managed in-world gateway runtime rather than the direct member adapter seam was deferred in this session. The session stayed focused on explaining the current live failure in the existing patch.
- Any analogous audit for Claude Code world members was deferred. This session stayed narrowly on Codex because the user explicitly scoped the architecture discussion to Codex first.

## Context for Resuming Agent

### Important Context

The most important new fact from this session is that the June 21 patch is active but incomplete.

What was proven:

1. The user’s live smoke hit the real installed runtime:
   - `~/.substrate/bin/substrate` resolved to `/home/spenser/__Active_code/substrate/target/release/substrate`
   - active systemd service was `/usr/local/bin/substrate-world-service`
   - binary strings in the installed service included:
     - `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME`
     - `codex-home`

2. The public bootstrap and toolbox dispatch path are functioning:
   - the session record at [session.json](/home/spenser/.substrate/run/agent-hub/sessions/019eeb94-cfc3-78f3-be2d-afd6e3069671/session.json:1) shows:
     - `orchestrator_backend_id: "cli:codex-host"`
     - `world_id: "wld_019eeb94-cfcd-77c1-bdd1-36d3f4753c31"`
     - `world_generation: 0`
   - the host orchestrator returned an authoritative `task_run_id`
   - repeated `run_world_task` calls reached `cli:codex-world`

3. The remaining direct member failure is not “no auth state” in the narrow sense:
   - temp isolated `CODEX_HOME` with only copied `auth.json`:
     - `codex login status` -> `Logged in using ChatGPT`
     - `codex exec` -> exit `1` with:
       - `The 'gpt-5.3-codex' model is not supported when using Codex with a ChatGPT account.`
   - temp isolated `CODEX_HOME` with copied `auth.json` plus copied `config.toml`:
     - `codex exec` succeeds and returns `OK`

4. The real user config is carrying the working model:
   - [~/.codex/config.toml](/home/spenser/.codex/config.toml:1) sets:
     - `model = "gpt-5.4"`
   - without that config, isolated Codex falls back to `gpt-5.3-codex`
   - that fallback is incompatible with the current ChatGPT-backed auth on this machine

Therefore:
- The next patch should not start from “auth.json seeding failed”.
- The next patch should start from “direct world-member isolated home is missing `config.toml`, causing an unsupported default model selection”.

### Assumptions Made

- Assumed the isolated temp `CODEX_HOME` probe is faithful enough to the direct world-member runtime because the failing path also uses an isolated `CODEX_HOME` and direct Codex execution.
- Assumed the observed `gpt-5.3-codex` unsupported-model error is the same reason the world member exits `1`, even though the world member launcher cleaned up its temp launcher dir before direct artifact inspection was possible.
- Assumed copying `config.toml` is the minimal near-term fix, because it changed the isolated probe from failing to succeeding without needing any additional credentials file.

### Potential Gotchas

- Do not confuse “gateway crate code is used” with “managed gateway server process is used”. The direct member path still bypasses the managed gateway startup/auth-bundle seam.
- The active systemd service already had a managed `substrate-gateway` child for `cli:codex-host`, but that does not mean the failing `run_world_task` path traversed that child.
- `member_runtime.unregister_member(...)` removes the launcher dir on failure/closeout, so you usually will not find leftover `/tmp/substrate-member-runtime-entry-*` artifacts after the fact.
- The repo worktree is already dirty from the earlier auth-seeding session: do not accidentally attribute those changes to this diagnosis-only session.
- The repo root still contains a file named `.codex`, so keep using `handoffs/` rather than defaulting the handoff skill output into `.codex/handoffs`.

## Environment State

### Tools/Services Used

- `systemctl status substrate-world-service.service substrate-world-service.socket --no-pager`
- `ps -ef`
- `strings`
- `rg`
- `find`
- `jq`
- direct local Codex probes:
  - `codex login status`
  - `codex exec`

### Active Processes

- Systemd-managed `substrate-world-service` is active:
  - main process `/usr/local/bin/substrate-world-service`
- At observation time, the service cgroup also contained a managed `substrate-gateway` child for `cli:codex-host`

### Environment Variables

- `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME`
- `CODEX_HOME`
- `SUBSTRATE_LLM_AUTH_BUNDLE_FD`
- `SUBSTRATE_WORLD_SOCKET`
- `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`
- `SUBSTRATE_AGENT_TOOLBOX_VERSION`
- `HOME`
- `USERPROFILE`

## Related Resources

- [Previous handoff: 2026-06-21-135249-world-agent-run-world-task-auth-seeding.md](./2026-06-21-135249-world-agent-run-world-task-auth-seeding.md)
- [Current handoff session record](/home/spenser/.substrate/run/agent-hub/sessions/019eeb94-cfc3-78f3-be2d-afd6e3069671/session.json:1)
- [Direct member runtime launcher](/home/spenser/__Active_code/substrate/crates/world-service/src/member_runtime.rs)
- [World member prompt bridge](/home/spenser/__Active_code/substrate/crates/world-service/src/prompt_fulfillment.rs)
- [Gateway adapter runtime seam](/home/spenser/__Active_code/substrate/crates/gateway/src/adapter_runtime.rs)
- [Managed gateway runtime startup](/home/spenser/__Active_code/substrate/crates/world-service/src/gateway_runtime.rs)
- [Managed gateway FD auth-bundle reader](/home/spenser/__Active_code/substrate/crates/gateway/src/server/mod.rs)
- [Shared request schema](/home/spenser/__Active_code/substrate/crates/transport-api-types/src/lib.rs)
- [User Codex config carrying working model](/home/spenser/.codex/config.toml:1)
- [Isolated home helper in upstream Codex wrapper](/home/spenser/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unified-agent-api-codex-0.3.7/src/home.rs)

---

**Security Reminder**: This handoff names only file paths, command names, env var identifiers, and non-secret model/config behavior. It does not include token values or secret file contents.
