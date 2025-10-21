# Substrate Uninstall / Teardown (macOS)

Follow this checklist to completely remove any Substrate installation, the Lima
world backend, and associated shims on macOS arm64.

## Quick Uninstall Script

```bash
curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/uninstall-substrate.sh | bash
```

This script stops running processes, removes PATH snippets, deletes Substrate
state, and tears down the Lima VM. Run it from any terminal, then open a fresh
shell so PATH updates take effect.

## Manual Checklist

### 1. Stop Running Processes

```bash
pgrep -fl substrate      # Inspect any substrate-related processes
pkill -f substrate       # Stop lingering CLI processes
```

If an older shimmed command is still running (for example an editor launched
through the shim), stop it manually before continuing.

### 2. Shut Down and Delete the Lima VM

```bash
limactl list | grep substrate || true
limactl stop substrate       # Gracefully stop the VM if present
limactl delete substrate     # Remove the instance and all state
```

If `limactl delete` reports that no instance exists, you can move on.

### 3. Remove Shims and Local State

If the `substrate` binary is still available:

```bash
substrate --shim-remove   # Cleanly remove all deployed shims
```

Then delete any residual state:

```bash
rm -rf ~/.substrate
rm -f ~/.substrate_bashenv ~/.substrate_history ~/.substrate_preexec
hash -r                  # Clear the shell command cache
```

### 4. Remove Installed Binaries

Tarball / bundle installs typically place binaries under the extracted
directory (for the beta bundle this was `.../macos_arm64/bin`). Remove the
directory that was added to `PATH`:

```bash
rm -rf /path/to/substrate/install/bin
```

If you installed via Cargo:

```bash
cargo uninstall substrate || true
rm -f ~/.cargo/bin/substrate ~/.cargo/bin/supervisor
```

Check for leftover symlinks or copies in common locations:

```bash
ls -l /usr/local/bin | grep substrate
ls -l ~/bin | grep substrate
```

Delete any matches that were created manually.

### 5. Undo PATH Modifications

If you exported the bundle’s `bin` directory earlier, remove that snippet from
your shell rc files:

- `~/.zshrc`
- `~/.bashrc`
- `~/.bash_profile`

After editing, start a new shell session (or `source` the file) so the changes
take effect.

### 6. Verify Removal

Run these checks to ensure nothing remains:

```bash
which -a substrate     # Should report “no substrate in PATH”
substrate --version    # Should report “command not found”
limactl list           # Should not list a “substrate” instance
ls ~/.substrate        # Should print “No such file or directory”
```

All commands should confirm that Substrate binaries, shims, and the Lima world
are gone. When needed, reboot or log out to clear any cached PATH settings for
other sessions.
