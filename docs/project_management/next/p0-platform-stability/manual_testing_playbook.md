# P0 Platform Stability – Manual Validation Log

This document captures the full end-to-end validation flow for the socket-activation, replay, and health improvements delivered in `feat/p0-platform-stability`, as well as the debugging steps taken on 2025-12-02 while verifying the Linux provisioning path.

## 1. Validation Checklist

### 1.1 Linux Provisioning & Socket Activation — World enabled

1. Deploy dev build and provision the world-agent:
   ```bash
   ./scripts/substrate/dev-install-substrate.sh --profile release
   ```
2. Verify systemd units and socket:
   ```bash
   sudo systemctl status substrate-world-agent.socket --no-pager
   sudo systemctl status substrate-world-agent.service --no-pager
   ls -l /run/substrate.sock
   ```
3. Probe the agent and shell telemetry:
   ```bash
   curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities | jq .
   substrate world doctor --json | jq '{world_socket, socket_activation}'
   substrate --shim-status | grep 'World socket'
   ```
4. Run the logging tests:
   ```bash
   cargo test -p substrate-shell --test logging
   ```

### 1.2 Linux Provisioning & Socket Activation — Enable World after install

1. Deploy dev build and provision the world-agent:
   ```bash
   ./scripts/substrate/dev-install-substrate.sh --profile release
   ./scripts/linux/world-provision.sh --profile release
   ```
2. Verify systemd units and socket:
   ```bash
   sudo systemctl status substrate-world-agent.socket --no-pager
   sudo systemctl status substrate-world-agent.service --no-pager
   ls -l /run/substrate.sock
   ```
3. Probe the agent and shell telemetry:
   ```bash
   curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities | jq .
   substrate world doctor --json | jq '{world_socket, socket_activation}'
   substrate --shim-status | grep 'World socket'
   ```
4. Run the logging tests:
   ```bash
   cargo test -p substrate-shell --test logging
   ```


### 1.3 Linux Provisioning & Socket Activation — No World

1. Deploy dev build with world disabled:
   ```bash
   ./scripts/substrate/dev-install-substrate.sh --profile release --no-world
   ```
2. Verify systemd units and socket are absent/inactive:
   ```bash
   sudo systemctl status substrate-world-agent.socket --no-pager   # expect inactive or not-found
   sudo systemctl status substrate-world-agent.service --no-pager  # expect inactive or not-found
   ls -l /run/substrate.sock                                       # expect “No such file”
   ```
3. Probe the agent and shell telemetry:
   ```bash
   substrate world doctor --json | jq '{world_enabled, world_socket, warnings}'
   substrate --shim-status | grep -i 'World socket'
   ```
   Expect `world_enabled: false` (or warning about no world) and no socket path.
4. Optional: run the logging tests:
   ```bash
   cargo test -p substrate-shell --test logging
   ```

### 1.3 macOS (Lima) Socket Activation

1. Warm the Lima guest and inspect the socket:
   ```bash
   ./scripts/mac/lima-warm.sh --check-only
   ./scripts/mac/lima-warm.sh
   limactl shell substrate sudo systemctl status substrate-world-agent.socket
   limactl shell substrate world doctor --json | jq '.world_socket'
   ```

### 1.3 Windows/WSL Socket Activation

1. Run the PowerShell warm script with `-WhatIf` and capture logs:
   ```powershell
   pwsh -File scripts\windows\wsl-warm.ps1 -WhatIf
   ```
2. When ready, perform the full warm and verify from WSL:
   ```powershell
   pwsh -File scripts\windows\wsl-warm.ps1
   wsl -d substrate-wsl -- bash -lc "systemctl status substrate-world-agent.socket"
   ```

### 1.4 Replay Isolation & Verbose Scopes

1. World-on replay:
   ```bash
   substrate --replay <SPAN_ID> --replay-verbose
   ```
2. `--no-world` and env opt-out:
   ```bash
   substrate --replay <SPAN_ID> --replay-verbose --no-world
   SUBSTRATE_REPLAY_USE_WORLD=disabled substrate --replay --span <SPAN_ID> --replay-verbose
   ```
3. Cleanup helper:
   ```bash
   substrate world cleanup --diagnose
   substrate world cleanup --apply   # optional
   ```
4. Tests:
   ```bash
   cargo test -p substrate-replay -- --nocapture
   cargo test -p substrate-shell --test replay_world
   ```

### 1.5 Health Manager Parity

1. CLI checks:
   ```bash
   substrate health
   substrate health --json | jq '.summary.manager_states'
   ```
2. Targeted tests:
   ```bash
   cargo test -p substrate-shell --test shim_health
   cargo test -p substrate-shell --test shim_doctor
   ```

### 1.6 Installer State Tracking & Cleanup (Linux dev/prod)

Run both installer variants end-to-end with cleanup enabled so we validate the new metadata and unit teardown flow.

**Dev flow (builds from the local tree)**
1. Install with a temp prefix and capture logs:
   ```bash
   export PREFIX=/tmp/substrate-manual-dev
   ./scripts/substrate/dev-install-substrate.sh --profile release
   ```
2. Inspect host state metadata:
   ```bash
   jq . "${PREFIX}/install_state.json"
   ```
   Confirm `schema_version=1`, `created_by_installer` reflects the run, `members` lists any users added, and `linger` shows the prior state.
3. Verify units and socket (adjust paths if your distro differs):
   ```bash
   sudo systemctl status substrate-world-agent.socket --no-pager || true
   sudo systemctl status substrate-world-agent.service --no-pager || true
   ls -l /run/substrate.sock || true
   ```
4. Uninstall with cleanup:
   ```bash
   ./scripts/substrate/dev-uninstall-substrate.sh --cleanup-state
   sudo systemctl status substrate-world-agent.socket --no-pager || true
   sudo systemctl status substrate-world-agent.service --no-pager || true
   ```
   Missing-unit status is expected; confirm group membership/linger only changed when `created_by_installer=true`.

**Prod flow (release-style install)**
1. Install to a temp prefix (optionally pin a version with `--version <semver>`):
   ```bash
   export PREFIX=/tmp/substrate-manual-prod
   ./scripts/substrate/install-substrate.sh --prefix "${PREFIX}"
   ```
2. Inspect metadata at `${PREFIX}/install_state.json` and confirm fields as above.
3. Verify world doctor and unit state:
   ```bash
   "${PREFIX}/bin/substrate" world doctor --json | jq '.world_socket'
   sudo systemctl status substrate-world-agent.socket --no-pager || true
   sudo systemctl status substrate-world-agent.service --no-pager || true
   ```
4. Uninstall with cleanup:
   ```bash
   ./scripts/substrate/uninstall-substrate.sh --cleanup-state
   sudo systemctl status substrate-world-agent.socket --no-pager || true
   sudo systemctl status substrate-world-agent.service --no-pager || true
   ```
   Confirm the units are absent and the metadata-driven cleanup removed only installer-added users/linger entries. Preserve systemctl logs (`/tmp/substrate-installer-*/systemctl-*.log` if you ran the harness) with your artifacts.

### 1.7 Replay origin-aware defaults & agent routing

Use a temp trace to keep spans isolated:
```bash
export SHIM_TRACE_LOG=/tmp/r2d-trace.jsonl
rm -f "$SHIM_TRACE_LOG"
```

1. **Record a world span (agent path)**
   ```bash
   SUBSTRATE_WORLD=enabled SUBSTRATE_WORLD_ENABLED=1 \
   ./target/debug/substrate -c 'printf world > /tmp/r2d_world.txt'
   ```
   Confirm `/run/substrate.sock` is healthy (`curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities | jq .`).

2. **Record a host span**
   ```bash
   SUBSTRATE_WORLD=disabled SUBSTRATE_WORLD_ENABLED=0 \
   ./target/debug/substrate -c 'printf host > /tmp/r2d_host.txt'
   ```

3. **Default follows recorded origin**
   ```bash
   ./target/debug/substrate --replay <SPAN_WORLD_ID> --replay-verbose
   ./target/debug/substrate --replay <SPAN_HOST_ID> --replay-verbose
   ```
   Expect world span to route via agent; host span should stay on host with no cgroup/netns warnings.

4. **Flip flag inverts origin**
   ```bash
   ./target/debug/substrate --replay <SPAN_WORLD_ID> --replay-verbose --flip-world
   ./target/debug/substrate --replay <SPAN_HOST_ID> --replay-verbose --flip-world
   ```
   World span should drop to host; host span should use agent/local world path. Verbose output should cite the flip reason.

5. **Agent unavailable fallback (single warning)**
   Temporarily point to a dead socket:
   ```bash
   SUBSTRATE_AGENT_ENDPOINT=/tmp/r2d-dead.sock ./target/debug/substrate --replay <SPAN_WORLD_ID> --replay-verbose
   ```
   Expect one `[replay] warn:` about agent failure (with doctor/SUBSTRATE_WORLD_SOCKET guidance) before fallback to overlay/fuse/copy-diff; no repeated cgroup/netns/nft warnings.

6. **Copy-diff root override + warning dedupe**
   ```bash
   SUBSTRATE_COPYDIFF_ROOT=/tmp/r2d-copydiff-root \
   ./target/debug/substrate --replay <SPAN_WORLD_ID> --replay-verbose --flip-world
   ```
   Verify verbose output shows the overridden root and only a single warning if ENOSPC is simulated.

### 1.8 Policy-driven world fs mode (read-only vs writable)

Use a scratch policy so broker reloads without touching user config:
```bash
cat > /tmp/r2e-policy.yaml <<'YAML'
id: r2e-manual
name: R2e manual fs mode policy
world_fs_mode: read_only
fs_read: ['*']
fs_write: []
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
YAML
SUBSTRATE_POLICY=/tmp/r2e-policy.yaml ./target/debug/substrate world doctor --json | jq '{world_fs_mode, world_socket}'
```
Expect `world_fs_mode: "read_only"`. Flip the value to `writable`, rerun doctor, and confirm the change.

1. **Non-PTY read-only enforcement**
   ```bash
   curl --unix-socket /run/substrate.sock \
     -H 'content-type: application/json' \
     --data '{"cmd":"sh -lc \"echo deny > /tmp/r2e-ro.txt\"","pty":false,"agent_id":"manual","world_fs_mode":"read_only"}' \
     http://localhost/v1/execute | jq '{exit, fs_diff}'
   ```
   Expect a non-zero exit and no `fs_diff` since writes are blocked.

2. **PTY writable path**
   ```bash
   curl --unix-socket /run/substrate.sock \
     -H 'content-type: application/json' \
     --data '{"type":"start","cmd":"sh -lc \"echo ok > /tmp/r2e-writable.txt\"","env":{"SUBSTRATE_WORLD_FS_MODE":"writable"}}' \
     http://localhost/v1/stream
   ```
   Exit 0 with `fs_diff` indicating the write.

3. **Shim/trace export**
   ```bash
   SUBSTRATE_POLICY=/tmp/r2e-policy.yaml ./target/debug/substrate -c 'true'
   tail -n1 ~/.substrate/trace.jsonl | jq '.replay_context.world_fs_mode'
   ```
   Confirm spans record the active mode for doctor/replay.

## 2. Debugging Log (2025-12-02 UTC-5)

1. **Initial provisioning** – `dev-install-substrate` installed the socket/service pair, but `/run/substrate.sock` was owned by `root:root 0660`, so non-root CLI calls failed with `Permission denied (os error 13)`.
   - Actions: created `substrate` group and added user (`groupadd -f substrate && usermod -aG substrate $USER`), then restarted the socket unit with `SocketGroup=substrate`.

2. **socket unit override confusion** – a drop-in (`override.conf`) duplicated the `ListenStream` stanza, causing two listeners and “systemd_socket lists path twice”. Solution: rely on the base unit (already updated by the provisioning script) and delete the override.

3. **User session still in old group** – even after adding the group, the shell `id` output showed `gid=spenser`. Fix: start a new shell (`newgrp substrate`) and re-source `dev-shim-env.sh`.

4. **Lingering/cgroup delegation missing** – `substrate -c 'true'` still fell back with `Failed to create cgroup directory`. `loginctl enable-linger spenser` followed by restarting the socket/service resolved the cgroup permission issue.

5. **Stray warning text executing as shell input** – before resolving the agent connectivity, the fallback warning was printed to stdout and interpreted by `/bin/sh -c 'true'`, yielding `sh: line 1: World]: command not found`. Once socket access and cgroup permissions were fixed, the warning disappeared and commands stayed in-world.

6. **Final validation** – after a fresh login (so the session inherited `gid=substrate` and the lingering setup), re-running:
   ```bash
   curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities | jq .
   substrate world doctor --json | jq '.world_socket'
   substrate -c 'true'
   ```
   confirmed socket activation was functioning on Linux. The same commands now execute without fallback, and `substrate world doctor` reports `mode: "socket_activation", probe_ok: true`.

Document any additional platform-specific findings in this file as further manual runs are completed.
