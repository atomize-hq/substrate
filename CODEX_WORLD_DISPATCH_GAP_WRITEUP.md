# Codex World Dispatch Gap Write-Up

Date: 2026-06-16  
Workspace: `/home/spenser/__Active_code/substrate`

## Purpose

This document captures the exact state of the Codex host-orchestrator to world-dispatch work after the recent UDS/sandbox investigation and follow-up live repros.

The goal is to make one thing unambiguous:

1. what was actually broken,
2. what has now been fixed,
3. what was incorrectly assumed about world bindings,
4. and what the current blocker actually is.

## Executive Summary

The current blocker is **not** that host `cli:codex` sessions are failing to persist authoritative world bindings in violation of `SPEC-32` through `SPEC-56`.

The current blocker is:

- explicit `--scope host` still intentionally creates a host-only orchestration session with no world binding, per `SPEC-30`,
- world-backed start via `--scope world` does correctly establish and persist authoritative `world_id` / `world_generation`,
- the host-orchestrator toolbox path can now reach Substrate from a live Codex runtime after the Codex sandbox bypass fix,
- but the first retained world member bootstrap for `cli:codex_world` currently dies with exit status `127` before authoritative registration,
- and the persisted failed member records show that Substrate is trying to launch the world member via the host-resolved NVM Codex path:
  `/home/spenser/.config/nvm/versions/node/v24.11.0/bin/codex`.

That is the exact gap today.

## Part 1: What Was Originally Broken

The original failure was a transport/runtime-posture problem, not a world-binding problem.

In the initial host `cli:codex` repro:

- `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` was exported correctly,
- Codex could see the toolbox endpoint,
- but any Python `AF_UNIX connect()` inside the Codex runtime returned `PermissionError(1, 'Operation not permitted')`,
- including connects to a fresh self-owned `/tmp` Unix socket,
- and `/proc/self/status` showed `Seccomp: 2` and `NoNewPrivs: 1`.

That proved the immediate failure was inside Codex's sandboxed runtime, before any toolbox request reached Substrate.

Relevant write-up:

- `handoffs/2026-06-15-224054-substrate-codex-toolbox-uds-investigation.md`

## Part 2: What Was Changed

Two launch-path changes were made so Substrate stops launching Codex in the UAA default sandbox posture for these orchestration paths.

### 2.1 Gateway adapter defaulted Codex requests into external-sandbox bypass

File:

- `crates/gateway/src/adapter_runtime.rs`

Key lines:

- `crates/gateway/src/adapter_runtime.rs:13`
- `crates/gateway/src/adapter_runtime.rs:48`
- `crates/gateway/src/adapter_runtime.rs:73`
- `crates/gateway/src/adapter_runtime.rs:82`

What changed:

- Codex backend registration now sets `allow_external_sandbox_exec: true`.
- `run_control(...)` now injects `agent_api.exec.external_sandbox.v1=true` by default for Codex requests unless the caller already supplied:
  - an explicit external-sandbox decision, or
  - explicit `backend.codex.exec.*` overrides.

Evidence:

- [adapter_runtime.rs](/home/spenser/__Active_code/substrate/crates/gateway/src/adapter_runtime.rs:13)

### 2.2 Direct Codex attach/resume path now bypasses approvals and sandbox

File:

- `crates/shell/src/execution/prompt_fulfillment.rs`

Key lines:

- `crates/shell/src/execution/prompt_fulfillment.rs:166`
- `crates/shell/src/execution/prompt_fulfillment.rs:175`

What changed:

- direct Codex attach now calls:
  `dangerously_bypass_approvals_and_sandbox(true)`
- instead of using the previous Codex wrapper posture:
  - `approval_policy(Never)`
  - `sandbox_mode(WorkspaceWrite)`

Evidence:

- [prompt_fulfillment.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/prompt_fulfillment.rs:166)

## Part 3: What Was Proven After The Fix

After the above change, a live patched `target/debug/substrate` host `cli:codex` session behaved materially differently.

### 3.1 The old generic AF_UNIX denial is gone

Observed in a live patched host session:

- direct connect to `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` succeeded,
- direct connect to a fresh `/tmp` Unix socket succeeded,
- `/proc/self/status` no longer showed seccomp confinement for the Codex runtime posture:
  - `NoNewPrivs: 0`
  - `Seccomp: 0`

### 3.2 Raw toolbox requests now reach Substrate

When a raw `spawn_world_worker` toolbox envelope was sent from inside Codex over the exported socket:

- the request reached Substrate,
- Substrate returned a typed response,
- and the error was no longer transport-related.

On a host-only session the response was:

- `missing_world_binding: orchestration session ... has no authoritative world binding`

That result was important because it proved:

- transport was fixed,
- request shaping was good enough to reach Substrate,
- and the next failure was semantic, not socket/sandbox.

## Part 4: Why The World-Binding Assumption Was Wrong

The key confusion was assuming that later `SPEC-32+` work meant all public host-root starts should already carry authoritative world binding.

That is **not** what the current contract says.

### 4.1 `SPEC-30` explicitly says `--scope host` bypasses world

Relevant spec lines:

- `llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md:40`
- `llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md:57`
- `llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md:310`
- `llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md:312`

Key contract text:

- `--scope host` explicitly bypasses world.
- `--scope world` creates a host-rooted durable orchestration session and establishes authoritative world session/binding truth before `start` returns.

Evidence:

- [SPEC-30](/home/spenser/__Active_code/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md:40)
- [SPEC-30](/home/spenser/__Active_code/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md:57)
- [SPEC-30](/home/spenser/__Active_code/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md:310)
- [SPEC-30](/home/spenser/__Active_code/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md:312)

### 4.2 `SPEC-32` assumes an authoritative binding already exists for world dispatch

Relevant spec lines:

- `llm-last-mile/SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md:33`
- `llm-last-mile/SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md:51`

Key contract text:

- host-orchestrator internal dispatch validates exact:
  - `orchestration_session_id`
  - `caller_participant_id`
  - `backend_id`
  - authoritative `world_id`
  - authoritative `world_generation`

That is a dispatch contract. It is not a statement that explicit `--scope host` root-starts must synthesize world binding.

Evidence:

- [SPEC-32](/home/spenser/__Active_code/substrate/llm-last-mile/SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md:33)

### 4.3 Manual smoke also says the world binding proof belongs to the world-backed start path

Relevant smoke lines:

- `llm-last-mile/MANUAL-SMOKE-SETS-post-SPEC-32.md:712`

That smoke expectation says:

- on Linux, `--scope world` returns a host-rooted durable session with authoritative world binding.

Evidence:

- [MANUAL-SMOKE-SETS-post-SPEC-32.md](/home/spenser/__Active_code/substrate/llm-last-mile/MANUAL-SMOKE-SETS-post-SPEC-32.md:712)

## Part 5: Current Code Behavior Matches That Contract

### 5.1 Explicit host start remains host-only

File:

- `crates/shell/src/execution/agents_cmd.rs`

Relevant lines:

- `crates/shell/src/execution/agents_cmd.rs:1243`
- `crates/shell/src/execution/agents_cmd.rs:1305`

`build_host_start_launch_plan(...)` leaves:

- `world_id: None`
- `world_generation: None`

Evidence:

- [agents_cmd.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/agents_cmd.rs:1243)

### 5.2 World-backed start creates host-rooted session birth plan, then establishes binding

Relevant lines:

- `crates/shell/src/execution/agents_cmd.rs:324`
- `crates/shell/src/execution/agents_cmd.rs:334`
- `crates/shell/src/execution/agents_cmd.rs:1349`
- `crates/shell/src/execution/agents_cmd.rs:1437`

Flow:

1. `build_world_start_session_birth_plan(...)` constructs a host-rooted helper session.
2. If public scope is `World`, `establish_public_world_start_binding(...)` is invoked.
3. The returned `world_id` / `world_generation` are stamped into the session plan before launch continues.

Evidence:

- [agents_cmd.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/agents_cmd.rs:324)
- [agents_cmd.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/agents_cmd.rs:1437)

## Part 6: Live Evidence That World Binding Actually Works On The World-Backed Path

A later live session started with:

```bash
target/debug/substrate agent start --backend cli:codex_world --scope world --prompt 'Reply with READY only.' --json
```

The persisted session record shows authoritative world binding is present.

Evidence file:

- `/home/spenser/.substrate/run/agent-hub/sessions/019ed079-c3d3-7692-a159-521fbb6894a0.json`

Relevant lines:

- `/home/spenser/.substrate/run/agent-hub/sessions/019ed079-c3d3-7692-a159-521fbb6894a0.json:14`
- `/home/spenser/.substrate/run/agent-hub/sessions/019ed079-c3d3-7692-a159-521fbb6894a0.json:15`

Observed values:

- `world_id: "wld_019ed079-c3d4-7310-9275-e68056e3fe55"`
- `world_generation: 0`

Evidence:

- [/home/spenser/.substrate/run/agent-hub/sessions/019ed079-c3d3-7692-a159-521fbb6894a0.json](/home/spenser/.substrate/run/agent-hub/sessions/019ed079-c3d3-7692-a159-521fbb6894a0.json:14)

This is the critical proof that the missing-world-binding failure on the earlier repro came from using a host-only session, not from broken binding persistence.

## Part 7: What The Current Blocker Actually Is

Once the same raw `spawn_world_worker` request was sent against a world-backed session, the failure moved forward.

It no longer failed with:

- `missing_world_binding`

It failed later with:

- `world-scoped member runtime exited with status 127 before ownership could be established`

This means:

- authoritative binding validation passed,
- Substrate accepted the world-dispatch request far enough to begin member bootstrap,
- but the world member runtime never reached authoritative registration.

## Part 8: Exact Runtime Evidence For The Current Gap

Failed world member record:

- `/home/spenser/.substrate/run/agent-hub/participants/ash_019ece74-ffc8-7890-ab65-0c238b8dba44.json`

Relevant lines:

- participant identity:
  - `/home/spenser/.substrate/run/agent-hub/participants/ash_019ece74-ffc8-7890-ab65-0c238b8dba44.json:2`
  - `/home/spenser/.substrate/run/agent-hub/participants/ash_019ece74-ffc8-7890-ab65-0c238b8dba44.json:5`
- resolved runtime path:
  - `/home/spenser/.substrate/run/agent-hub/participants/ash_019ece74-ffc8-7890-ab65-0c238b8dba44.json:19`
- termination:
  - `/home/spenser/.substrate/run/agent-hub/participants/ash_019ece74-ffc8-7890-ab65-0c238b8dba44.json:33`
  - `/home/spenser/.substrate/run/agent-hub/participants/ash_019ece74-ffc8-7890-ab65-0c238b8dba44.json:37`

Observed values:

- backend: `cli:codex_world`
- execution scope: `world`
- `resolved_binary_path: "/home/spenser/.config/nvm/versions/node/v24.11.0/bin/codex"`
- `termination_reason: "world-scoped member runtime exited with status 127 before ownership could be established"`

Evidence:

- [/home/spenser/.substrate/run/agent-hub/participants/ash_019ece74-ffc8-7890-ab65-0c238b8dba44.json](/home/spenser/.substrate/run/agent-hub/participants/ash_019ece74-ffc8-7890-ab65-0c238b8dba44.json:19)

## Part 9: Why Exit 127 Matters

The exit code is the most important clue.

### 9.1 Host-side runtime selection resolves the binary on the host

File:

- `crates/shell/src/execution/agent_runtime/validator.rs`

Relevant lines:

- `crates/shell/src/execution/agent_runtime/validator.rs:178`
- `crates/shell/src/execution/agent_runtime/validator.rs:186`

`validate_runtime_realizability(...)` does:

- read `config.cli.binary`,
- resolve it via `which::which(...)` on the host,
- and store that absolute path into the runtime descriptor.

Evidence:

- [validator.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/agent_runtime/validator.rs:135)

### 9.2 The world-dispatch transport preserves that resolved binary path

File:

- `crates/shell/src/execution/orchestrator_world_dispatch.rs`

Relevant lines:

- `crates/shell/src/execution/orchestrator_world_dispatch.rs:2738`
- `crates/shell/src/execution/orchestrator_world_dispatch.rs:2745`

`build_spawn_world_worker_transport_request(...)` copies:

- `descriptor.binary_path.display().to_string()`

into the member dispatch transport request.

### 9.3 World execution normalizes PATH to guest-safe values

File:

- `crates/shell/src/execution/routing/dispatch/world_ops.rs`

Relevant lines:

- `crates/shell/src/execution/routing/dispatch/world_ops.rs:70`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs:76`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs:107`

The Linux guest PATH is normalized to:

- guest base PATH
- plus `/var/lib/substrate/world-deps/bin`

This is intentionally not the host user's NVM PATH.

Evidence:

- [world_ops.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:70)

### 9.4 The world launcher would fail with 125 if the binary file itself were missing

Files:

- `crates/world-service/src/member_runtime.rs`
- `crates/world-service/src/gateway_runtime.rs`

Relevant lines:

- `crates/world-service/src/member_runtime.rs:536`
- `crates/world-service/src/gateway_runtime.rs:859`
- `crates/world-service/src/gateway_runtime.rs:924`

Important behavior:

- if `resolved_runtime.binary_path` is not a file, world-service errors before launch,
- the wrapper script itself exits `125` if the binary path is absent in the launch context,
- but the observed failure is `127`.

That strongly suggests:

- the launcher found a file to exec,
- then the runtime failed later during command execution,
- which is consistent with a script/interpreter dependency problem.

Evidence:

- [member_runtime.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/member_runtime.rs:536)
- [gateway_runtime.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/gateway_runtime.rs:859)

## Part 10: Why `cli:codex_world` Is A Particularly Likely Victim

Current agent manifests:

- `/home/spenser/.substrate/agents/codex.yaml`
- `/home/spenser/.substrate/agents/codex_world.yaml`

Both declare:

- `runtime_family: codex`
- `binary: codex`

On this host:

- `which codex` resolves to `/home/spenser/.config/nvm/versions/node/v24.11.0/bin/codex`
- that file is a symlink to a JS launcher
- the launcher begins with `#!/usr/bin/env node`

Observed on host:

- `which codex` -> `/home/spenser/.config/nvm/versions/node/v24.11.0/bin/codex`
- `head -n 1 .../bin/codex` -> `#!/usr/bin/env node`
- `which node` -> `/home/spenser/.config/nvm/versions/node/v24.11.0/bin/node`

This means the current world member bootstrap is trying to reuse:

1. a host-user NVM-managed Codex path,
2. that depends on `node`,
3. where `node` itself is also host-user NVM managed,
4. while the world PATH is intentionally normalized to guest-safe paths plus world-deps.

That is a concrete, coherent explanation for exit `127`.

## Part 11: Exact Statement Of The Gap

The exact gap is:

**Substrate currently treats world-scoped Codex runtime realizability as “host `which codex` succeeded,” but the actual world bootstrap contract requires a guest-visible, guest-executable Codex runtime and any interpreter/runtime it depends on.**

Today those two truths are not aligned.

More concretely:

1. world-dispatch binding/authorization is present on the correct `--scope world` path,
2. Codex UDS access from the host orchestrator is now fixed,
3. but world-scoped member launch still reuses a host-resolved NVM Codex path that is not a stable guest runtime contract,
4. so `spawn_world_worker` fails during member bootstrap before authoritative registration.

## Part 12: What This Is Not

This is not:

- a `SPEC-32` world-binding persistence failure,
- a raw UDS transport failure,
- a Codex-in-host-sandbox `AF_UNIX connect()` denial anymore,
- or evidence that explicit `--scope host` should already be auto-world-bound under the current `SPEC-30` contract.

## Part 13: What Would Need To Change To Close The Gap

There are two logically separate options.

### Option A: Keep the current contract, fix runtime realizability for world-scoped Codex

This is the smallest truthful change.

It would mean:

1. define a guest-visible Codex runtime contract for `cli:codex_world`,
2. likely provision `node` and `codex` into world-deps or another stable guest path,
3. stop treating host `which codex` as sufficient proof for world member launch,
4. fail closed with an explicit remediation error if the guest-visible runtime is not available.

### Option B: Change the root-start contract so host sessions auto-acquire world bindings

This is a product/contract change, not the immediate runtime fix.

If desired, it would require:

1. updating `SPEC-30` semantics,
2. changing host root-start behavior in `build_host_start_launch_plan(...)`,
3. reworking smoke/contract expectations around `--scope host`,
4. and deciding whether explicit host start still means “bypass world”.

This is not the change required to fix the currently observed `127`.

## Part 14: Recommended Immediate Next Step

The next implementation step should be:

1. make world-scoped Codex runtime realizability explicit and fail-closed,
2. surface a clear error when `cli:codex_world` resolves only to a host-local NVM path without a guest-visible Codex/Node contract,
3. then add the actual guest/runtime provisioning path for Codex world members,
4. then re-run the same `spawn_world_worker` end-to-end smoke until a `Registered` event is observed instead of exit `127`.

In short:

- the world-binding path is working where the current contract says it should work,
- the current blocker is the **world runtime bootstrap contract for Codex**, not world binding.

## Primary File Pointers

### Specs

- [llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md](/home/spenser/__Active_code/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md:40)
- [llm-last-mile/SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md](/home/spenser/__Active_code/substrate/llm-last-mile/SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md:33)
- [llm-last-mile/MANUAL-SMOKE-SETS-post-SPEC-32.md](/home/spenser/__Active_code/substrate/llm-last-mile/MANUAL-SMOKE-SETS-post-SPEC-32.md:712)

### Launch-path code

- [crates/gateway/src/adapter_runtime.rs](/home/spenser/__Active_code/substrate/crates/gateway/src/adapter_runtime.rs:13)
- [crates/shell/src/execution/prompt_fulfillment.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/prompt_fulfillment.rs:166)
- [crates/shell/src/execution/agents_cmd.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/agents_cmd.rs:324)
- [crates/shell/src/execution/agents_cmd.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/agents_cmd.rs:1243)
- [crates/shell/src/execution/agents_cmd.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/agents_cmd.rs:1349)

### Runtime/bootstrap code

- [crates/shell/src/execution/agent_runtime/validator.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/agent_runtime/validator.rs:135)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:70)
- [crates/world-service/src/member_runtime.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/member_runtime.rs:536)
- [crates/world-service/src/gateway_runtime.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/gateway_runtime.rs:852)

### Live runtime evidence

- [/home/spenser/.substrate/run/agent-hub/sessions/019ed079-c3d3-7692-a159-521fbb6894a0.json](/home/spenser/.substrate/run/agent-hub/sessions/019ed079-c3d3-7692-a159-521fbb6894a0.json:14)
- [/home/spenser/.substrate/run/agent-hub/participants/ash_019ece74-ffc8-7890-ab65-0c238b8dba44.json](/home/spenser/.substrate/run/agent-hub/participants/ash_019ece74-ffc8-7890-ab65-0c238b8dba44.json:19)

