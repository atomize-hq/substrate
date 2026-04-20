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

The `install.sh` script is a thin wrapper that downloads the full installer
(`install-substrate.sh`) plus a loader UI. When no `--version` is provided it
pins those helper downloads to the **latest GitHub release tag** (not `main`)
to avoid drift while still defaulting to the latest release. Use
`SUBSTRATE_INSTALL_REF` only for development overrides.

For Linux hosted installs, `scripts/substrate/install.sh` and
`scripts/substrate/install-substrate.sh` share one operator-facing contract:

- Exact selection precedence:
  1. `--pkg-manager`
  2. `PKG_MANAGER`
  3. os-release mapping
  4. `PATH` probe in fixed order `apt-get -> dnf -> yum -> pacman -> zypper`
- Stable decision-line template:
  - `Detected distro: <id> (like: <id_like>), using package manager: <pkg_manager> (source: <flag|env|os_release|path_probe>)`
- Fixed multi-manager warning posture:
  - `Multiple supported package managers found in PATH: <manager_list>; selecting <selected> by fixed probe order (apt-get -> dnf -> yum -> pacman -> zypper). Override with --pkg-manager <apt-get|dnf|yum|pacman|zypper> or PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>.`
  - When the multi-manager `PATH` warning is required, the warning line appears before the decision line.
- Feature-specific exit and remediation posture:
  - Exit `2`: invalid `--pkg-manager` value or invalid `PKG_MANAGER` value; rerun with one of `apt-get`, `dnf`, `yum`, `pacman`, `zypper` or remove the invalid override.
  - Exit `3`: a manager selected by `--pkg-manager` or `PKG_MANAGER` was not found in `PATH`; install that manager or rerun with another allowed manager from the fixed vocabulary.
  - Exit `4`: no supported manager was selected after os-release mapping and `PATH` probing; install the missing prerequisites manually and rerun, or rerun with `--pkg-manager <apt-get|dnf|yum|pacman|zypper>` or `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>`.
- Wrapper parity:
  - For this feature's explicit contract branches, `scripts/substrate/install.sh` preserves direct installer exits `0`, `2`, `3`, and `4`.
  - `scripts/substrate/install.sh` does not collapse those feature-specific non-zero exits to `1`.
- Platform scope:
  - Linux: this contract applies.
  - macOS: no behavior change under ADR-0031.
  - Windows: no behavior change under ADR-0031.

The installer will:

1. Download `substrate-v<version>-linux_<arch>.tar.gz` from the release bucket
2. Place the bundle under `~/.substrate/versions/<version>`
3. Link `~/.substrate/bin/*` and stage shims in `~/.substrate/shims/`
   (host shells remain untouched—Substrate injects the shim directory at runtime)
4. Stage bundled manifests under
   `~/.substrate/versions/<version>/config/` (`manager_hooks.yaml` plus a legacy
   `world-deps.yaml` kept for backwards compatibility). `substrate world deps`
   (packages/bundles contract) reads inventory from `$SUBSTRATE_HOME/deps/` and
   `<workspace_root>/.substrate/deps/` and ignores legacy `world-deps.yaml` overlays.
5. Generate the runtime manager files (`~/.substrate/manager_init.sh`,
   `~/.substrate/manager_env.sh`) so Substrate-owned shells can source managers
   on demand. The manager env script also exports `SUBSTRATE_WORLD` and
   `SUBSTRATE_WORLD_ENABLED` so shims know whether isolation is active.
6. Write install metadata to `~/.substrate/config.yaml`
   (`install.world_enabled: true` unless `--no-world` is provided). The
   metadata is consumed by `substrate world enable` and shims/CLI commands that
   need to detect pass-through mode.
7. Install `substrate-world-agent` and `substrate-gateway` under `/usr/local/bin` and manage the
   systemd `.service` + `.socket` units (`/etc/systemd/system/substrate-world-agent.{service,socket}`)
8. Run `substrate world doctor --json` for a final readiness report
9. Ensure the `substrate` group exists on Linux hosts, add the invoking user,
   and reload the socket/service units so `/run/substrate` is recreated as
   `root:substrate` with `0750` permissions, `/run/substrate.sock` is recreated as
   `root:substrate` with `0660` permissions, and managed gateway runtime artifacts
   under `/run/substrate/substrate-gateway-runtime/` stay group-readable (`0750`
   directories, `0640` files). The installer prints
   `loginctl enable-linger <user>` guidance so socket activation survives
   logout/reboots.

If you installed an older version and see `EPERM` for allowlisted writes in `world_fs.isolation=full` + `world_fs.mode=writable`,
rerun the installer (or `scripts/linux/world-provision.sh` / `scripts/mac/lima-warm.sh`) to refresh the systemd units and capabilities.

By default the installer adds `<prefix>/bin` (default: `~/.substrate/bin`) to
your shell PATH by appending a small, idempotent snippet to your rc files
(bash/zsh/fish). Set `SUBSTRATE_INSTALL_NO_PATH=1` to skip this behavior and
invoke `~/.substrate/bin/substrate` directly instead. Supplying `--no-world` skips step 6, writes
`~/.substrate/config.yaml` with `install.world_enabled: false`, and prints
the exact `substrate world enable` command to run when you are ready to
provision the backend. You can still force a single world-isolated run later
with `substrate --world ...` without changing the stored metadata.

### Installer Metadata & Cleanup

- On Linux, both installers (`install-substrate.sh` + `dev-install-substrate.sh`)
  record host-state details in `<effective_prefix>/install_state.json` (default:
  `~/.substrate/install_state.json`). `schema_version = 1` tracks whether the
  `substrate` group existed, which users the installer added, the observed
  `loginctl` lingering state under `host_state.{group,linger}`, and the Linux
  platform metadata under `host_state.platform.os_release.id`,
  `host_state.platform.os_release.id_like`,
  `host_state.platform.pkg_manager.selected`, and
  `host_state.platform.pkg_manager.source`. macOS and Windows do not write this
  Linux host-state file; they only participate in compile/test parity for this
  pack.
- Metadata writes are idempotent; missing or corrupted files only emit warnings
  and never block install/uninstall runs.
- Uninstallers accept `--cleanup-state`/`--auto-cleanup` to remove recorded
  group memberships, drop the `substrate` group when the installer created it
  and no members remain, and disable lingering only when Substrate previously
  enabled it. Without the flag the existing guidance-only behavior remains.
- Upcoming interactive install/uninstall flows will use the same metadata to
  surface opt-in prompts before altering group membership or lingering.

### Prerequisites

- PID 1 must be `systemd` (`ps -p 1 -o comm=`). On WSL, enable systemd by adding
  `boot.systemd=true` under `[boot]` in `/etc/wsl.conf`, then `wsl --shutdown`.
- `sudo`, `curl`, `tar`, and `jq` must be available on the host.

During installation the script:

- Records the host PATH (for use as `SHIM_ORIGINAL_PATH`) without mutating it.
- Deploys fresh shims and appends a small PATH snippet (between `# >>> substrate >>>`
  markers) so `substrate` is callable from your terminal.
- Installs `substrate-world-agent` as a systemd service plus socket, places
  `substrate-gateway` beside it under `/usr/local/bin`, and runs
  `substrate world doctor --json` (inspect the `host.world_socket` block) without
  adding the shim directory to PATH to avoid self-referential lookups.
- Ensures the Linux `substrate` group exists, adds the invoking user when
  possible (printing manual steps otherwise), and restarts the socket/service
  units so `/run/substrate` is `root:substrate 0750`, `/run/substrate.sock` is
  owned by `root:substrate` with `0660` permissions, and managed gateway runtime
  logs/config/manifests under `/run/substrate/substrate-gateway-runtime/` are
  group-readable (`0750` directories, `0640` files). The script reports the
  current `loginctl` lingering status and
  reminds you to run `loginctl enable-linger <user>` so socket activation stays
  live after logout or reboot.

### Offline install

```bash
./scripts/substrate/install.sh --archive /path/to/substrate-v0.2.0-beta-linux_x86_64.tar.gz
```

Use the copy of `scripts/substrate/install.sh` (wrapper around
`install-substrate.sh`) shipped inside the bundle. The script
accepts the same flags as the hosted version (`--version`, `--prefix`,
`--pkg-manager <apt-get|dnf|yum|pacman|zypper>`, `--no-world`, `--no-shims`,
`--sync-deps`, `--dry-run`, `--archive/--artifact-dir`). The same Linux hosted
installer contract for precedence, decision-line output, fixed `PATH`-probe
warning posture, feature-specific remediation, and wrapper exit preservation
applies to the offline wrapper path.

### macOS (arm64)

```bash
curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/substrate/install.sh | bash
```

The macOS flow mirrors the Linux installer but additionally:

- Verifies Apple Virtualization Framework support (`kern.hv_support == 1`)
- Requires the Lima CLI (`limactl`) to be installed beforehand
- Requires `envsubst` (install via `brew install gettext` to provide it)
- Provisions the Lima VM (`scripts/mac/lima-warm.sh`) and copies the Linux
  `world-agent` and `substrate-gateway` binaries into the guest

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
substrate host doctor --json | jq '{ok, world_enabled, host_ok: .host.ok}'
substrate world doctor --json | jq '{ok, world_enabled, host_ok: .host.ok, world: {status: .world.status, ok: .world.ok}}'
```

(The installer already ran the doctor without the shim directory in PATH, so
your output should match unless the host environment changed.)

Snapshot-specific behavior (including `policy_resolution_mode` and snapshot-related trace fields) is specified in `docs/project_management/_archived/world-agent-policy-snapshot/policy-snapshot-spec.md`.

### Windows

```powershell
substrate.exe --version
substrate.exe --shim-status
substrate.exe shim doctor --json | ConvertFrom-Json | Select-Object path
substrate.exe health --json | ConvertFrom-Json | Select-Object summary
substrate.exe world doctor --json | ConvertFrom-Json | Select-Object schema_version,platform,world_enabled,ok
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
| `--pkg-manager <apt-get\|dnf\|yum\|pacman\|zypper>` | Highest-precedence Linux hosted-installer package-manager selector; invalid values exit with code `2`, and a selected manager missing from `PATH` exits with code `3` |
| `--no-world` | Skip provisioning the world backend (use `substrate world enable` later) |
| `--no-shims` | Skip shim deployment (useful for CI images) |
| `--sync-deps` | Run `substrate world deps current sync` after provisioning completes (best-effort; applies the enabled deps list into the world and may remediate APT-backed misses to `substrate world enable --provision-deps`) |
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
`host-proxy`, `substrate-gateway`, and (on supported platforms) `world-agent`,
symlink them into `~/.substrate/bin`, and deploy shim wrappers that point back to
`target/<profile>`. The generated `~/.substrate/dev-shim-env.sh` prepends both
the bin and shim directories to `PATH` while preserving `SHIM_ORIGINAL_PATH`, so
interactive sessions and the world backend see the same clean view of your
tooling.

On Linux, the dev installer also mirrors the production socket-activation
requirements: it creates the `substrate` group if needed, adds the invoking
user (or prints the `sudo usermod -aG substrate <user>` command), rewrites the
socket/service units so `/run/substrate` is `root:substrate 0750`,
`/run/substrate.sock` is `root:substrate 0660`, and managed gateway runtime
artifacts under `/run/substrate/substrate-gateway-runtime/` remain group-readable.
It also reports whether `loginctl enable-linger <user>` still needs to be run.

## Troubleshooting Highlights

- **No shim interception**: run `substrate --shim-status` and compare
  `substrate -c 'which git'` vs `which git`. If only the host shell is missing a
  manager snippet, run `substrate shim repair --manager <name> --yes` to update
  `~/.substrate_bashenv`.
- **World deps not present (Linux/macOS)**: run `substrate health` to see which
  enabled deps are `missing` or `blocked/manual`. Use `substrate world deps current sync`
  and `substrate world deps current list applied` to provision/verify, and
  `substrate world deps current show <name> --explain` for manual blockers.
- **Doctor failures**: capture `substrate shim doctor --json` and
  `substrate world doctor --json`; attach both to bug reports so we can spot
  PATH vs kernel/virtualization gaps quickly.
- **World agent inactive (Linux/WSL)**: confirm `systemctl status
  substrate-world-agent.socket` reports `listening`, `systemctl status
  substrate-world-agent.service` reports `active` (or restarts cleanly), and that `/run/substrate.sock`
  exists as `root substrate 0660` (`sudo ls -l /run/substrate.sock`). If the socket
  shows another group, rerun the installer to refresh the units. Permission issues
  usually mean your user is missing from the `substrate` group—check with
  `id -nG "$USER"`. If gateway lifecycle failures point at
  `/run/substrate/substrate-gateway-runtime/.../stderr.log`, that file should be
  readable to the `substrate` group after reprovision/reinstall.
  `id -nG "$USER"`—or lingering is still disabled (`loginctl enable-linger "$USER"`).
  `substrate world doctor --json | jq '.host.world_socket'` and `substrate --shim-status`
  both spell out whether socket activation is healthy.
- **WSL systemd disabled**: edit `/etc/wsl.conf`, set `[boot]
systemd=true`, run
  `wsl --shutdown`, and reopen the distribution.
- **macOS virtualization disabled**: enable "Virtualization" in System Settings
  → Privacy & Security, then rerun the installer or `substrate world enable`.

For deeper diagnostics, tail `~/.substrate/trace.jsonl` and rerun both
`substrate shim doctor --json` and `substrate world doctor --json` to capture a
fresh health report.
