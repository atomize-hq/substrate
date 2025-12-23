# Substrate Uninstall / Teardown Guide (v0.2.0-beta)

Use this guide to remove Substrate, its shims, and world backends on macOS,
Linux, or Windows (WSL). Each section references the bundled
`scripts/substrate/uninstall.sh` script (wrapper around
`uninstall-substrate.sh`), which you can run directly via
curl or from the release archive.

```bash
curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/substrate/uninstall.sh | bash
```

The `uninstall.sh` script is a thin wrapper that prefers the uninstaller shipped
inside your installed Substrate version (under `~/.substrate/versions/<version>/scripts/substrate/`).
If Substrate is not installed or the local uninstaller is missing, the wrapper
downloads `uninstall-substrate.sh` (and the loader UI when available) pinned to
the **latest GitHub release tag** (not `main`) to avoid drift.

## Common Cleanup Steps

Regardless of platform, the script:

1. Stops running `substrate` processes (`pkill -f substrate`).
2. Deletes `~/.substrate*` state (manager env/init files, `config.toml`,
   bundled manifests under `~/.substrate/versions/<version>/config/`, shims,
   history, locks, etc.).
3. Clears the shell command hash table (`hash -r`).

Since the installer no longer edits `.bashrc`, `.zshrc`, or other rc files, no
shell snippets are touched during removal. If you previously added custom PATH
entries or aliases, remove them manually after running the script.

## Linux & Windows (WSL)

### Scripted Removal

```bash
sudo env HOME="$HOME" ./scripts/substrate/uninstall.sh
```

Additional automated steps on systemd hosts:

- Stops and disables the `substrate-world-agent` systemd service.
- Stops/disables `substrate-world-agent.socket` as well.
- Removes `/etc/systemd/system/substrate-world-agent.{service,socket}` and reloads
  systemd.
- Deletes `/usr/local/bin/substrate-world-agent`, `/var/lib/substrate`, and the
  `/run/substrate` runtime directory.

### Manual Verification

```bash
systemctl status substrate-world-agent.socket   # Should be "Unit not found"
systemctl status substrate-world-agent.service  # Should be "Unit not found"
ls -l /usr/local/bin | grep substrate        # No world-agent binary
ls ~/.substrate                              # Should report "No such file"
```

For WSL, run the script inside the distribution where you installed Substrate.
Shut down the distro afterwards (`wsl --shutdown`) if you want to reclaim memory
immediately.

## macOS (arm64)

### Scripted Removal

```bash
sudo env HOME="$HOME" ./scripts/substrate/uninstall.sh
```

macOS-specific actions:

- Stops and deletes the `substrate` Lima VM via `limactl`.
- Leaves the system `/usr/local/bin` untouched unless you manually copied
  binaries there. Remove any custom symlinks yourself.

### Manual Verification

```bash
limactl list | grep substrate            # Should return nothing
ls -l /usr/local/bin | grep substrate    # Remove leftover symlinks if present
which -a substrate                       # Should report "no substrate"
```

## Windows Host (PowerShell)

```powershell
pwsh -File scripts/windows/uninstall-substrate.ps1 -RemoveWSLDistro
```

- Omit `-RemoveWSLDistro` to keep the distro but stop services and remove PATH
  updates.
- The script stops the host forwarder, removes the PowerShell profile snippet,
  deletes `$env:LOCALAPPDATA\Substrate`, disables the agent inside WSL, and
  optionally unregisters the distro.

## Troubleshooting

- **Permission denied removing system files (macOS/Linux)**: rerun the script
  with a user that has `sudo` privileges (`sudo ./scripts/substrate/uninstall.sh`).
- **Permission denied removing system files (Windows)**: rerun the PowerShell
  script in an elevated prompt.
- **Systemd reports lingering unit**: check `/etc/systemd/system` for a cached
  unit installed by an older release, remove it manually, and run
  `sudo systemctl daemon-reload`.
- **Lima VM still appears**: run `limactl delete substrate` manually; confirm no
  snapshot or template locks are present in `~/Library/Application Support/Lima`.

After removal, open a fresh shell session to ensure environment variables are
cleared.
