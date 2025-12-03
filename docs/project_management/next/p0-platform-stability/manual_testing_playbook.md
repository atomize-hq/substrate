# P0 Platform Stability – Manual Validation Log

This document captures the full end-to-end validation flow for the socket-activation, replay, and health improvements delivered in `feat/p0-platform-stability`, as well as the debugging steps taken on 2025-12-02 while verifying the Linux provisioning path.

## 1. Validation Checklist

### 1.1 Linux Provisioning & Socket Activation

1. Deploy dev build and provision the world-agent:
   ```bash
   ./scripts/substrate/dev-install-substrate.sh --profile release
   ./scripts/linux/world-provision.sh --profile release
   ```
   (The dev installer always builds from the local tree; there is no skip-build flag. Reuse the same `target` dir to avoid full rebuilds.)
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

### 1.2 macOS (Lima) Socket Activation

1. Warm the Lima guest and inspect the socket:
   ```bash
   ./scripts/mac/lima-warm.sh --check-only
   ./scripts/mac/lima-warm.sh
   limactl shell substrate sudo systemctl status substrate-world-agent.socket
   limactl shell substrate substrate world doctor --json | jq '.world_socket'
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
   substrate --replay --span <SPAN_ID> --replay-verbose
   ```
2. `--no-world` and env opt-out:
   ```bash
   substrate --replay --span <SPAN_ID> --replay-verbose --no-world
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
