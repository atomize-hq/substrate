# macOS World Setup Guide

This guide walks through setting up the Lima-based Linux world backend for Substrate on macOS. This enables every Substrate command to run inside an isolated Linux VM with full telemetry and policy enforcement.

## Prerequisites

### Host Requirements

1. **macOS version**: 13.0 or later (required for Virtualization.framework and VSock support)
   ```sh
   # Verify macOS version
   sw_vers

   # Check virtualization support (should return 1)
   sysctl kern.hv_support

   # Check architecture (1 for Apple Silicon, 0 for Intel)
   sysctl hw.optional.arm64
   ```

2. **Install required tools via Homebrew**:
   ```sh
   brew install lima jq openssh coreutils gnused gnu-tar gettext
   ```

3. **Add gettext to PATH** (for envsubst):
   ```sh
   echo 'export PATH="/opt/homebrew/opt/gettext/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc

   # Verify envsubst is available
   command -v envsubst
   ```

4. **Ensure Rust toolchain is installed** (for building world-service):
   ```sh
   # If not installed:
   curl https://sh.rustup.rs -sSf | sh
   ```

## Installation Steps

### Choosing a Profile (runtime vs. dev)

Two Lima profiles are provided:
- `scripts/mac/lima/substrate.yaml` — runtime defaults (smaller footprint)
- `scripts/mac/lima/substrate-dev.yaml` — development profile (larger CPU/RAM/disk)

For day-to-day use, the runtime profile is sufficient. For heavy development (builds, tracing, debugging), use the dev profile.

To start with the dev profile from the repo root:
```sh
PROJECT="$(pwd)" envsubst < scripts/mac/lima/substrate-dev.yaml > /tmp/substrate-dev.yaml
limactl start --tty=false --name substrate /tmp/substrate-dev.yaml
```

### Step 1: Set up Lima VM

1. **Start the Lima VM** using the provided helper script (runtime defaults):
  ```sh
  # From the substrate repository root
  scripts/mac/lima-warm.sh
  ```

   This script will:
   - Create a new Lima VM named "substrate" with Ubuntu 24.04
   - Install required packages (nftables, iproute2, dnsmasq, etc.)
   - Configure the systemd service for substrate-world-service
   - Mount your home directory (read-only) and project directory (read-write)
   - Configure `/var/lib/substrate`, `/run/substrate`, and `/tmp` as guest writeable paths via `ReadWritePaths` (handled automatically by the provisioning script; no manual edits required)

2. Or, to use the dev profile (heavier resources), start it explicitly as shown above.

3. **Verify VM is running**:
  ```sh
  limactl list substrate
  # Should show status: Running
  ```

### Step 2: Build and Deploy World Service

1. **Compile inside the Lima guest** (recommended):
   ```sh
   # Build from the mounted project directory
   limactl shell substrate bash -lc 'cd /src && cargo build -p world-service --release'
   ```

2. **Install the binary inside the VM**:
   ```sh
   limactl shell substrate sudo install -m755 /src/target/release/world-service /usr/local/bin/substrate-world-service
   ```

   > **Alternative:** Cross-compile on the host using a Linux target (e.g.
   > `cargo build -p world-service --release --target aarch64-unknown-linux-gnu`) and
   > copy the resulting binary into the guest. Avoid copying the default macOS
   > build—Mach-O binaries cannot run inside the Linux VM.

3. **Verify the binary works**:
   ```sh
   limactl shell substrate substrate-world-service --version
   ```

### Step 3: Start World Service

1. **Enable and start the systemd service**:
   ```sh
   limactl shell substrate sudo systemctl daemon-reload
   limactl shell substrate sudo systemctl enable substrate-world-service.service
   limactl shell substrate sudo systemctl enable --now substrate-world-service.socket
   limactl shell substrate sudo systemctl restart substrate-world-service.service
   ```

2. **Verify service is running**:
   ```sh
   limactl shell substrate systemctl status substrate-world-service.socket
   limactl shell substrate systemctl status substrate-world-service.service
   ```

3. **Test the agent API**:
   ```sh
   limactl shell substrate curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities
   # Should return JSON with world backend capabilities
   ```

### Step 4: Run Health Check

Run the comprehensive health check script to ensure everything is configured correctly:

```sh
scripts/mac/lima-doctor.sh
```

All critical checks should show `[PASS]`. If any show `[FAIL]`, see the troubleshooting section below.

## Testing the Setup

### Manual Smoke Test

1. **Test basic command execution** through the agent:
   ```sh
   limactl shell substrate bash -c 'curl --unix-socket /run/substrate.sock -X POST http://localhost/v1/execute \
     -H "Content-Type: application/json" \
     -d "{\"cmd\": \"echo hello world\", \"env\": {}, \"cwd\": \"/tmp\"}"'
   ```

2. **Check agent logs** for any issues:
   ```sh
   limactl shell substrate journalctl -u substrate-world-service -n 50
   ```

### Substrate CLI Smoke Script

Run the scripted end-to-end check from the repo root (uses the repository build of `substrate`):

```sh
PATH="$(pwd)/target/debug:$PATH" scripts/mac/smoke.sh
```

The script performs non-PTY, PTY, and replay runs, then validates that the replayed fs diff includes `world-mac-smoke/file.txt`.

### Netfilter conformance smoke

When you need the opt-in netfilter posture checks, warm the Lima guest with the backend guard enabled:

```sh
SUBSTRATE_WORLD_NETFILTER_ENABLE=1 scripts/mac/lima-warm.sh
```

Then run the posture-aware smoke:

```sh
PATH="$(pwd)/target/debug:$PATH" scripts/mac/smoke.sh --netfilter-conformance
```

This conformance mode creates a temporary no-workspace fixture, sets
`SUBSTRATE_OVERRIDE_WORLD_NET_FILTER=1`, and verifies two postures:

- allow-all (`net_allowed=["*"]`): the probe succeeds and doctor reports `requested=false`, `enabled=false`
- deny-all (`net_allowed=[]`): the probe fails and doctor reports `requested=true`, `enabled=true`, `world_netfilter_enable_present=true`

Use `--log-dir <dir>` if you want the doctor JSON and command transcripts written to a specific
artifact directory. For the full operator playbook, including Linux privileged verification and the
optional named-allowlist walkthrough, see `docs/manual_verification/netfilter_enforcement.md`.

### Using `substrate host doctor` and `substrate world doctor`

Once the VM is provisioned, prefer the CLI doctors for day-to-day checks:

```sh
target/debug/substrate host doctor
target/debug/substrate host doctor --json | jq .

target/debug/substrate world doctor
target/debug/substrate world doctor --json | jq .
```

`substrate host doctor` is host-scoped (limactl + virtualization + VM/service reachability). `substrate world doctor` includes the host report plus world-service-reported “in-world” facts (guest-kernel Landlock support/ABI + world fs strategy probe) via `/v1/doctor/world`. The legacy `scripts/mac/lima-doctor.sh` script remains available for deeper troubleshooting but the CLI commands are the canonical entry points.

## Helper Scripts

- **`scripts/mac/lima-warm.sh`**: Start/create the Lima VM
- **`scripts/mac/lima-stop.sh`**: Stop the Lima VM
- **`scripts/mac/lima-doctor.sh`**: Run comprehensive health checks

## Troubleshooting

### Common Issues and Solutions

| Issue | Diagnosis | Solution |
|-------|-----------|----------|
| Virtualization not available | `sysctl kern.hv_support` returns 0 | Enable virtualization in System Settings → Privacy & Security → Developer Tools |
| Lima VM fails to start | Check `limactl start substrate` output | Ensure sufficient disk space; check `~/Library/Logs/lima/` for detailed logs |
| SSH connection fails | `limactl shell substrate` fails | Run `limactl shell substrate` once to accept host key |
| Agent not responding | `substrate host doctor` shows agent unreachable | Check systemd: `limactl shell substrate systemctl status substrate-world-service.socket` and `.service`, then probe directly in-guest: `limactl shell substrate sudo -n curl --fail --unix-socket /run/substrate.sock http://localhost/v1/doctor/world | jq .` |
| Agent binary missing | Service fails to start | Rebuild and copy binary as shown in Step 2 |
| Permission errors | Socket operations fail | Ensure directories exist with correct permissions: `/run/substrate` (0750) |
| DNS resolution issues | Network operations fail in VM | Check dnsmasq: `limactl shell substrate systemctl status dnsmasq` |
| `sudo: unable to resolve host lima-substrate` | Sudo emits warning due to missing host mapping | `limactl shell substrate sudo bash -lc "grep -q 'lima-substrate' /etc/hosts || echo '127.0.1.1 lima-substrate' >> /etc/hosts"` |
| `Exec format error` starting agent | Copied host-compiled binary into guest | Build inside VM: `limactl shell substrate` → `cargo build -p world-service --release` → copy to `/usr/local/bin/substrate-world-service` |
| SSH UDS not creating local socket | SSH ControlMaster multiplexing interferes | Disable ControlMaster: add `-o ControlMaster=no -o ControlPath=none` |
| TCP forwarding resets | SSH cannot forward TCP→UDS directly | Use SSH UDS; TCP fallback requires a guest TCP↔UDS bridge (e.g., `socat`) |

### Viewing Logs

- **VM provisioning logs**: `limactl start substrate --debug`
- **Agent service logs**: `limactl shell substrate journalctl -u substrate-world-service -f`
- **System logs**: `limactl shell substrate journalctl -n 100`

### Resetting the Environment

If you need to start fresh:

```sh
# Stop and delete the VM
limactl stop substrate
limactl delete substrate

# Remove any cached data
rm -rf ~/.lima/substrate

# Start over from Step 1
scripts/mac/lima-warm.sh
```

## Environment Variables

The shell manages transport detection automatically. The only knobs you should need are:

- `SUBSTRATE_WORLD=disabled`: Temporarily bypass the world (defaults to `enabled`).
- `SUBSTRATE_WORLD_ID`: Set by the shell; useful for correlating spans while debugging.
- `SUBSTRATE_WORLD_NETFILTER_ENABLE=1`: Host-side input for `scripts/mac/lima-warm.sh`; writes
  `WORLD_NETFILTER_ENABLE=1` into the guest `substrate-world-service.service` unit so requested
  netfilter enforcement can be honored.

For a quick guest-env check without reprovisioning, run:

```sh
scripts/mac/lima-warm.sh --check-only
```

The output should say whether the guest systemd service currently includes
`WORLD_NETFILTER_ENABLE=1`.

## Next Steps

Once the Lima VM and world-service are running:

1. The Substrate shell will automatically detect and use the Lima backend on macOS
2. All commands will execute inside the isolated Linux world
3. Telemetry and fs_diff will be collected just like on Linux

For development and debugging:
- Use `RUST_LOG=debug` for verbose agent logs
- Monitor forwarding with `ps aux | grep -E 'vsock|ssh.*substrate'`
- Check socket connectivity with `nc -U ~/.substrate/sock/agent.sock` (once forwarding is active)
