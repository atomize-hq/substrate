# Installation Guide (v0.2.0-beta)

Substrate ships release bundles with a cross-platform installer that deploys the
CLI, shim launcher, and world backend in one step. The same script is used by
our `curl | bash` flow and the offline bundles published via
GitHub Releases (`https://github.com/atomize-hq/substrate/releases`).

## Supported Platforms

- **Linux**: systemd-based distributions with `sudo`, `curl`, `tar`, and `jq`
  available. The world backend runs via the `substrate-world-agent.service` +
  `substrate-world-agent.socket` units.
- **Windows 11 / 10 (22H2+) with WSL2 + systemd**: install via the bundled PowerShell script (`scripts/windows/install-substrate.ps1`), which provisions the `substrate-wsl` distro after enabling systemd in `/etc/wsl.conf`. The WSL backend is functional but experimental—expect ongoing updates.
- **macOS 14+ (arm64)**: requires Apple Virtualization Framework and Lima (the
  installer verifies both).

> ℹ️ PowerShell automation is available—use the Windows instructions below to
> install from the host. Manual installation from inside WSL remains an option
> if you prefer the Linux script.

## Quick Install (Release Bundles)

### Linux / WSL (systemd)

```bash
curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/substrate/install.sh | bash
```

The installer will:

1. Download `substrate-v<version>-linux_<arch>.tar.gz` from the release bucket
2. Place the bundle under `~/.substrate/versions/<version>`
3. Link `~/.substrate/bin/*` and stage shims in `~/.substrate/shims/`
   (host shells remain untouched—Substrate injects the shim directory at runtime)
4. Stage bundled manifests under
   `~/.substrate/versions/<version>/config/` (`manager_hooks.yaml` +
   `world-deps.yaml`) so `substrate health` and `world deps` work immediately
5. Generate the runtime manager files (`~/.substrate/manager_init.sh`,
   `~/.substrate/manager_env.sh`) so Substrate-owned shells can source managers
   on demand. The manager env script also exports `SUBSTRATE_WORLD` and
   `SUBSTRATE_WORLD_ENABLED` so shims know whether isolation is active.
6. Write install metadata to `~/.substrate/config.toml`
   (`[install] world_enabled = true` unless `--no-world` is provided). The
   metadata is consumed by `substrate world enable` and shims/CLI commands that
   need to detect pass-through mode.
7. Install `substrate-world-agent` under `/usr/local/bin` and manage the
   systemd `.service` + `.socket` units (`/etc/systemd/system/substrate-world-agent.{service,socket}`)
8. Run `substrate world doctor --json` for a final readiness report

Add `~/.substrate/bin` (or your custom `--prefix` bin directory) to PATH—or
invoke `~/.substrate/bin/substrate` directly—because the installer no longer
edits shell rc files. Supplying `--no-world` skips step 6, writes
`~/.substrate/config.toml` with `[install] world_enabled = false`, and prints
the exact `substrate world enable` command to run when you are ready to
provision the backend. You can still force a single world-isolated run later
with `substrate --world ...` without changing the stored metadata.

### Prerequisites

- PID 1 must be `systemd` (`ps -p 1 -o comm=`). On WSL, enable systemd by adding
  `boot.systemd=true` under `[boot]` in `/etc/wsl.conf`, then `wsl --shutdown`.
- `sudo`, `curl`, `tar`, and `jq` must be available on the host.

During installation the script:

- Records the host PATH (for use as `SHIM_ORIGINAL_PATH`) without mutating it.
- Deploys fresh shims but leaves `.bashrc`, `.zshrc`, `BASH_ENV`, and PowerShell
  profiles alone—runtime helpers source the generated manager snippets only when
  `substrate` is executed.
- Installs `substrate-world-agent` as a systemd service plus socket and runs
  `substrate world doctor --json` (inspect the `world_socket` block) without
  adding the shim directory to PATH to avoid self-referential lookups.

### Offline install

```bash
./scripts/substrate/install.sh --archive /path/to/substrate-v0.2.0-beta-linux_x86_64.tar.gz
```

Use the copy of `scripts/substrate/install.sh` (wrapper around
`install-substrate.sh`) shipped inside the bundle. The script
accepts the same flags as the hosted version (`--version`, `--prefix`,
`--no-world`, `--no-shims`, `--sync-deps`, `--dry-run`, `--archive/--artifact-dir`).

### macOS (arm64)

```bash
curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/substrate/install.sh | bash
```

The macOS flow mirrors the Linux installer but additionally:

- Verifies Apple Virtualization Framework support (`kern.hv_support == 1`)
- Requires the Lima CLI (`limactl`) to be installed beforehand
- Requires `envsubst` (install via `brew install gettext` to provide it)
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

### macOS / Linux / WSL

```bash
substrate --version
substrate --shim-status
substrate shim doctor --json | jq '{path: .path, states: .states}'
substrate health --json | jq '.summary'
substrate world doctor --json | jq '.summary'
```

(The installer already ran the doctor without the shim directory in PATH, so
your output should match unless the host environment changed.)

### Windows

```powershell
substrate.exe --version
substrate.exe --shim-status
substrate.exe shim doctor --json | ConvertFrom-Json | Select-Object path
substrate.exe health --json | ConvertFrom-Json | Select-Object summary
substrate.exe world doctor --json | ConvertFrom-Json | Select-Object status,message
```

If you installed with `--no-world`, run `substrate world enable` once you are
ready to provision the backend (macOS Lima VM, Linux namespaces, or WSL). If
either doctor surfaces failures, consult `docs/WORLD.md` and the troubleshooting
appendices for the relevant platform.

### Verify PATH isolation

```bash
printf "host PATH -> %s\n" "$PATH"
substrate -c 'printf "substrate PATH -> %s\n" "$PATH"'
```

Substrate builds its PATH + manager environment on demand, so the host PATH and
dotfiles stay untouched. Need legacy shells to source managers automatically?
Run `substrate shim repair --manager <name> --yes` to append the recommended
snippet (plus a `.bak` backup) to `~/.substrate_bashenv`.

## Installer Options Reference

| Flag | Purpose |
| ---- | ------- |
| `--version <semver>` | Install a specific published release (default: `0.2.0-beta`) |
| `--prefix <path>` | Override the installation prefix (default: `~/.substrate`) |
| `--no-world` | Skip provisioning the world backend (use `substrate world enable` later) |
| `--no-shims` | Skip shim deployment (useful for CI images) |
| `--sync-deps` | Run `substrate world deps sync --all --verbose` after provisioning completes |
| `--dry-run` | Print all actions without executing them |
| `--archive <path>` | Install from a local tarball instead of downloading |

Add `<prefix>/bin` (default: `~/.substrate/bin`) to PATH so the
`substrate` binary is discoverable. The shim directory (`<prefix>/shims`) is
only injected inside Substrate-managed processes.

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

### Dev Convenience Scripts

The repository also ships helper scripts that mirror the production layout while
using your workspace build artifacts:

```bash
scripts/substrate/dev-install-substrate.sh --profile debug   # or --profile release
source ~/.substrate/dev-shim-env.sh                          # refresh PATH/SHIM_ORIGINAL_PATH

# Later, to remove the dev wiring:
scripts/substrate/dev-uninstall-substrate.sh
```

These scripts build `substrate`, `substrate-shim`, `substrate-forwarder`,
`host-proxy`, and (on supported platforms) `world-agent`, symlink them into
`~/.substrate/bin`, and deploy shim wrappers that point back to
`target/<profile>`. The generated `~/.substrate/dev-shim-env.sh` prepends both
the bin and shim directories to `PATH` while preserving `SHIM_ORIGINAL_PATH`, so
interactive sessions and the world backend see the same clean view of your
tooling.

## Troubleshooting Highlights

- **No shim interception**: run `substrate --shim-status` and compare
  `substrate -c 'which git'` vs `which git`. If only the host shell is missing a
  manager snippet, run `substrate shim repair --manager <name> --yes` to update
  `~/.substrate_bashenv`.
- **Doctor failures**: capture `substrate shim doctor --json` and
  `substrate world doctor --json`; attach both to bug reports so we can spot
  PATH vs kernel/virtualization gaps quickly.
- **World agent inactive (Linux/WSL)**: confirm `systemctl status
  substrate-world-agent.socket` reports `listening`, `systemctl status
  substrate-world-agent.service` reports `active` (or restarts cleanly), and that `/run/substrate.sock`
  exists (`sudo ls -l /run/substrate.sock`). `substrate world doctor --json | jq '.world_socket'`
  and `substrate --shim-status` both spell out whether socket activation is healthy.
- **WSL systemd disabled**: edit `/etc/wsl.conf`, set `[boot]
systemd=true`, run
  `wsl --shutdown`, and reopen the distribution.
- **macOS virtualization disabled**: enable "Virtualization" in System Settings
  → Privacy & Security, then rerun the installer or `substrate world enable`.

For deeper diagnostics, tail `~/.substrate/trace.jsonl` and rerun both
`substrate shim doctor --json` and `substrate world doctor --json` to capture a
fresh health report.
