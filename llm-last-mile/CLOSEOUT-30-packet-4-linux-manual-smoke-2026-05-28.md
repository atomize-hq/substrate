# CLOSEOUT-30 Packet 4 Linux Manual Smoke (2026-05-28)

Status: blocked. This note reopens the Packet 4 closeout because the required Linux manual smoke was not previously landed.

## Assumptions

1. Packet 4 must not weaken the validation contract to compensate for missing manual evidence.
2. Docs-only updates are acceptable in this reopen pass if the runtime behavior itself does not need to change.
3. Real CLI evidence from temporary local fixtures is acceptable for the host-scoped surface, but successful world-backed smoke still requires access to an authoritative shared-world socket.

## Commands Run

### Baseline Linux Runtime Access

1. `target/debug/substrate world doctor --json`
2. `target/debug/substrate host doctor --json`
3. `id -nG "$USER"`
4. `ls -l /run/substrate.sock`

### Host-Scoped Public Root Start Smoke

These commands were run under a temporary fixture rooted at `/tmp/slice30-host-smoke-omyAuU` with:

1. `HOME=/tmp/slice30-host-smoke-omyAuU/home`
2. `SUBSTRATE_HOME=/tmp/slice30-host-smoke-omyAuU/substrate-home`
3. one host-scoped `codex` CLI agent backed by a local fake persistent binary
4. toolbox enabled over UDS

Commands:

1. `target/debug/substrate workspace init /tmp/slice30-host-smoke-omyAuU/workspace --force`
2. `target/debug/substrate agent start --backend cli:codex --scope host --prompt 'hello host smoke' --json`
3. `target/debug/substrate agent status --json`
4. `target/debug/substrate agent toolbox status --json`
5. `target/debug/substrate agent toolbox env --json`
6. `target/debug/substrate agent doctor --json`
7. `target/debug/substrate agent stop --session 019e6c14-e099-75c3-811d-487ece01c944 --json`

### World-Backed Public Root Start Smoke

These commands were run under a temporary fixture rooted at `/tmp/slice30-world-smoke-TCgTWK` with:

1. a host-scoped `codex` orchestrator agent
2. an unscoped `claude_code` backend
3. workspace default scope set to `world`
4. the real Linux world-service socket path left at its default `/run/substrate.sock`

Commands:

1. `target/debug/substrate agent start --backend cli:claude_code --scope world --prompt 'hello explicit world smoke' --json`
2. `target/debug/substrate agent start --backend cli:claude_code --prompt 'hello omitted scope smoke' --json`

## Observed Outcomes

### Baseline Linux Runtime Access

1. `id -nG "$USER"` reported `azureuser adm cdrom sudo dip lxd docker substrate ollama`.
2. `ls -l /run/substrate.sock` reported `srw-rw---- root substrate /run/substrate.sock`.
3. `target/debug/substrate world doctor --json` reported:
   - `host.world_socket.socket_exists = true`
   - `host.world_socket.probe_ok = false`
   - `host.world_socket.probe_error = "Permission denied (os error 13)"`
   - `world.status = "unreachable"`
4. `target/debug/substrate host doctor --json` reported the same socket boundary failure in `host.world_socket.probe_error`.

### Host-Scoped Public Root Start Smoke

1. `agent start --scope host` succeeded and emitted an `accepted` record with:
   - `backend_id = "cli:codex"`
   - `scope = "host"`
2. The same command emitted a `completed` record with:
   - `action = "start"`
   - `turn_outcome = "success"`
   - `session_posture = "active"`
   - `state = "active"`
3. The backing fake agent invocation captured `exec` in argv and the startup prompt text `hello host smoke` on stdin, which confirms the host-scoped public root-start surface remained on the normal host exec path.
4. `agent status --json` remained readable, but by the time it was queried the synthetic host session had already normalized to `posture = "parked_resumable"` with no warnings.
5. `agent toolbox status --json` returned `eligibility.state = "dependency_unavailable"` because no live host-scoped orchestrator participant remained by the time the command ran.
6. `agent toolbox env --json` failed closed with `no live host-scoped orchestrator participant found for the selected orchestrator`.
7. `agent doctor --json` failed at `world_boundary` with `required world-scoped member boundary is unavailable (world.status=unreachable): Permission denied (os error 13)`.

### World-Backed Public Root Start Smoke

1. The explicit world command exited `2` with:
   - `runtime_start_failed: failed to open authoritative shared world for public world start`
2. The omitted-scope world-default command exited `2` with the same error:
   - `runtime_start_failed: failed to open authoritative shared world for public world start`
3. Because no successful world-backed root start could open the authoritative shared world, this pass could not honestly record:
   - a durable world-backed orchestration session birth
   - persisted host attach truth on a successful world-backed session
   - persisted authoritative world binding on a successful world-backed session
   - `agent status --json`, `agent toolbox status --json`, `agent toolbox env --json`, and `agent doctor --json` against a successful world-backed session
   - omitted-scope fallback behavior beyond the fact that the world-routed command hit the same shared-world open seam
   - later host-mediated world dispatch after a successful world-backed start

## Honest Packet 4 Closeout Status

Packet 4 cannot be called honestly closed on 2026-05-28 from this runtime.

Reason:

1. The required Linux manual smoke for the successful world-backed public start path is still blocked at the authoritative shared-world open seam.
2. The exact blocker observed on this machine is the Linux world socket boundary:
   - `/run/substrate.sock` exists
   - `substrate world doctor --json` and `substrate host doctor --json` both report `Permission denied (os error 13)`
   - direct public world-start commands fail before they can create the required durable host-first world-backed session evidence

## What Must Happen Before Packet 4 Can Close Honestly

1. Restore real access to the authoritative shared world from this runtime so `target/debug/substrate world doctor --json` reports a healthy probe.
2. Re-run the explicit Linux manual smoke commands above until the world-backed `agent start --scope world ... --json` path succeeds.
3. Capture the resulting world-backed session evidence, `status`, toolbox, doctor, omitted-scope, and later host-mediated dispatch outcomes in this note or a superseding closeout note without weakening the contract.
