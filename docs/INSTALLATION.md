# Installation Guide (v0.2.0-beta)

Substrate ships release bundles with a cross-platform installer that deploys the
CLI, shim launcher, and world backend in one step. The same script is used by
our `curl | bash` flow and the offline bundles published under
`https://releases.atomizehq.com/substrate/`.

## Supported Platforms

- **Linux**: systemd-based distributions with `sudo`, `curl`, `tar`, and `jq`
  available. The world backend runs as a systemd service (`substrate-world-agent`).
- **Windows 11 / 10 (22H2+) with WSL2 + systemd**: run the Linux installer from
  inside the WSL distribution after enabling systemd via `/etc/wsl.conf`.
- **macOS 14+ (arm64)**: requires Apple Virtualization Framework and Lima (the
  installer verifies both).

> ℹ️ Windows PowerShell automation for the host is forthcoming. Today, the
> Windows host workflow is "install from within WSL" using the Linux steps
> below.

## Quick Install (Release Bundles)

### Linux / WSL (systemd)

```bash
curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/substrate/install-substrate.sh | bash
```

The installer will:

1. Download `substrate-v<version>-linux_<arch>.tar.gz` from the release bucket
2. Place the bundle under `~/.substrate/versions/<version>`
3. Link `~/.substrate/bin/*` and prepend shims to your shell PATH via
   `~/.substrate_bashenv`
4. Install `substrate-world-agent` under `/usr/local/bin` and manage the
   systemd service (`/etc/systemd/system/substrate-world-agent.service`)
5. Run `substrate world doctor --json` for a final readiness report

**Prerequisites**

- PID 1 must be `systemd` (`ps -p 1 -o comm=`). On WSL, enable systemd by adding
  `boot.systemd=true` under `[boot]` in `/etc/wsl.conf`, then `wsl --shutdown`.
- `sudo`, `curl`, `tar`, and `jq` must be available on the host.

During installation the script:
- Sanitises the current PATH (removes any stale shims) and records it in
  `SHIM_ORIGINAL_PATH`.
- Deploys fresh shims and writes `~/.substrate_bashenv`, including a trampoline
  if `BASH_ENV` was already set, so automated shells inherit the right PATH.
- Installs `substrate-world-agent` as a systemd service and runs
  `substrate world doctor --json` **without** the shim directory in either PATH
  or `SHIM_ORIGINAL_PATH`, mirroring the macOS installer’s behaviour and
  avoiding self-referential shim lookups during the doctor check.

**Offline install**

```bash
./scripts/substrate/install-substrate.sh --archive /path/to/substrate-v0.2.0-beta-linux_x86_64.tar.gz
```

Use the copy of `scripts/substrate/install-substrate.sh` shipped inside the bundle. The script
accepts the same flags as the hosted version (`--version`, `--prefix`,
`--no-world`, `--no-shims`, `--dry-run`).

### macOS (arm64)

```bash
curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/substrate/install-substrate.sh | bash
```

The macOS flow mirrors the Linux installer but additionally:

- Verifies Apple Virtualization Framework support (`kern.hv_support == 1`)
- Requires the Lima CLI (`limactl`) to be installed beforehand
- Provisions the Lima VM (`scripts/mac/lima-warm.sh`) and copies the Linux
  `world-agent` into the guest

**Manual Lima preparation** is documented in `docs/WORLD.md`.

### Windows Host (PowerShell)

```powershell
pwsh -File scripts/windows/install-substrate.ps1
```

- Flags mirror the Unix installer: `-Version`, `-Prefix`, `-Archive`,
  `-NoWorld`, `-NoShims`, `-DryRun`, `-DistroName`.
- Defaults to `$env:LOCALAPPDATA\Substrate` and provisions the
  `substrate-wsl` distro unless `-NoWorld` is supplied.
- Hosted one-liner:
  ```powershell
  irm https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/windows/install-substrate.ps1 | iex
  ```
- Requires WSL2 (with systemd enabled inside the distro) and PowerShell 7+.

## Post-Install Checks

After the script completes:

- **macOS / Linux / WSL:**
  ```bash
  source ~/.substrate_bashenv
  substrate --version
  substrate --shim-status
  substrate world doctor --json | jq '.'
  ```
  (The installer already ran the doctor using PATH values without the shim
  directory, so this re-run should match what the script saw.)
- **Windows:**
  ```powershell
  . "$env:LOCALAPPDATA\Substrate\substrate-profile.ps1"
  substrate.exe --shim-status
  substrate.exe world doctor --json | ConvertFrom-Json | Select-Object status,message
  ```

If `world doctor` surfaces failures, consult `docs/WORLD.md` and the troubleshooting
appendices for the relevant platform.

## Installer Options Reference

| Flag | Purpose |
| ---- | ------- |
| `--version <semver>` | Install a specific published release (default: `0.2.0-beta`) |
| `--prefix <path>` | Override the installation prefix (default: `~/.substrate`) |
| `--no-world` | Skip provisioning the world backend (requires manual setup) |
| `--no-shims` | Skip shim deployment (useful for CI images) |
| `--dry-run` | Print all actions without executing them |
| `--archive <path>` | Install from a local tarball instead of downloading |

When using a non-default prefix, remember to export
`PATH="<prefix>/shims:<prefix>/bin:$PATH"` in your shell or automation.

## Manual Build (Developers)

Developers working on Substrate itself can still build from source:

```bash
git clone https://github.com/atomize-hq/substrate.git
cd substrate
cargo build --release
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

To run the locally built binaries without using the installer:

```bash
export PATH="$(pwd)/target/release:$PATH"
substrate --shim-deploy
substrate world doctor
```

This path is primarily intended for contributors; production installations
should prefer the release installer so that policies, shims, and world services
stay in sync with bundled expectations.

## Troubleshooting Highlights

- **No shim interception**: ensure `~/.substrate/shims` is first in `PATH`, then
  run `hash -r`.
- **World doctor fails (Linux/WSL)**: confirm `systemctl status
  substrate-world-agent` reports `active (running)` and that `/run/substrate.sock`
  exists (`sudo ls -l /run/substrate.sock`).
- **WSL systemd disabled**: edit `/etc/wsl.conf`, set `[boot]\nsystemd=true`, run
  `wsl --shutdown`, and reopen the distribution.
- **macOS virtualization disabled**: enable "Virtualization" in System Settings
  → Privacy & Security, then rerun the installer.

For deeper diagnostics, tail `~/.substrate/trace.jsonl` and rerun
`substrate world doctor --json` to capture a fresh health report.
