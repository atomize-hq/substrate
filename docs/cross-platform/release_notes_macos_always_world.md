# Release Notes — macOS Always-World Parity

## Highlights

- macOS now ships the same "Always World" experience as Linux by running Substrate commands inside a Lima-hosted Linux VM.
- Shell, PTY, replay, shim, and telemetry flows all route through the Lima backend automatically with transport fallbacks.
- Documentation and tooling cover provisioning, health checks, smoke validation, and troubleshooting for the new mac path.

## Requirements

- macOS 13.0+ with Virtualization.framework enabled (`sysctl kern.hv_support` → `1`).
- Homebrew packages: `lima jq openssh coreutils gnused gnu-tar gettext` (ensure `envsubst` and `vsock-proxy` are on `PATH`).
- Rust toolchain installed for building `substrate-world-agent`.

## Setup Flow

1. From the repo root run `scripts/mac/lima-warm.sh` to create or start the `substrate` Lima VM. The helper script applies the default profile with writable mounts (`/src`, `/tmp`), installs required packages, and deploys the systemd unit.
2. Build the agent on the host (`cargo build -p world-agent --release`) and copy it into the guest (`limactl copy ...`, `sudo mv /usr/local/bin/substrate-world-agent`).
3. Enable the service: `limactl shell substrate sudo systemctl enable --now substrate-world-agent`.
4. Validate with `scripts/mac/lima-doctor.sh`; all critical checks must return `[PASS]`.

## Smoke & Diagnostics

- End-to-end verification: `PATH="$(pwd)/target/debug:$PATH" scripts/mac/smoke.sh` (expects replay `fs_diff` to include `world-mac-smoke/file.txt`).
- Guest logs: `substrate sudo journalctl -u substrate-world-agent -n 200` (the CLI shells into Lima automatically); manual fallback `limactl shell substrate sudo journalctl -u substrate-world-agent -n 200`.
- Transport order and fallback: VSock → SSH UDS (`~/.substrate/sock/agent.sock`) → SSH TCP. The shell logs the chosen transport; on failure it emits a single warning and runs the command on the host.

## Environment & Compatibility

- `SUBSTRATE_WORLD` defaults to `enabled` on macOS; set `SUBSTRATE_WORLD=disabled` to bypass the Lima backend for troubleshooting.
- `SUBSTRATE_WORLD_ID` is exported once a session is established for telemetry correlation.
- No mac-only transport override flags; all behavior mirrors Linux defaults.

## Troubleshooting Quick Hits

- Re-run `scripts/mac/lima-doctor.sh` if the shell falls back every command—fix any `[FAIL]` entries before retrying.
- Ensure the agent binary matches the guest architecture (`Exec format error` → rebuild inside Lima).
- If VSock is unavailable, verify `vsock-proxy` exists and the Lima profile uses `vmType: "vz"`; otherwise expect SSH fallback with identical semantics.

## References

- Provisioning profile: `scripts/mac/lima/substrate.yaml`
- mac setup runbook: `docs/dev/mac_world_setup.md`
- World architecture (Linux & mac): `docs/WORLD.md`
- Installation overview with mac section: `docs/INSTALLATION.md`
