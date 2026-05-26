# Windows WSL World Setup Guide

The owned WSL provisioning helpers are intentionally fail-closed in this slice.

Current posture:

- `scripts/windows/wsl-warm.ps1` exits explicitly instead of importing or mutating a WSL distro.
- `scripts/wsl/provision.sh` exits explicitly instead of installing packages or writing systemd units.
- This is deliberate: the WSL path is not yet aligned with the Linux/macOS placement contract for:
  - `SUBSTRATE_HOME` placement
  - `/run/substrate.sock` ownership and group access
  - managed runtime artifact access under `/run/substrate/substrate-gateway-runtime/`

## What To Do Instead

- For supported world provisioning, use:
  - Linux host-native: `scripts/linux/world-provision.sh`
  - macOS Lima guest: `scripts/mac/lima-warm.sh`
- For a CLI-only workflow inside WSL, install with `--no-world` and do not run the WSL warm/provision helpers.

Example CLI-only install inside WSL:

```bash
curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/substrate/install.sh | bash -s -- --no-world
```

Example dev install inside WSL:

```bash
scripts/substrate/dev-install-substrate.sh --no-world
```

## Expected Failure Mode

If you run the WSL helpers anyway, they should fail early with an explicit unsupported message.

Expected commands:

```powershell
pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .)
```

```bash
scripts/wsl/provision.sh
```

Expected result:

- Exit code `4`
- No package-manager mutation
- No systemd unit rewrite
- No WSL distro import or reprovisioning side effects

## Verification

Use these checks to confirm you stayed on the fail-closed path:

```powershell
$LASTEXITCODE
```

```powershell
wsl -l -v
```

```powershell
Get-ChildItem "$env:LOCALAPPDATA\\Substrate" -ErrorAction SilentlyContinue
```

You should see the helper exit with `4`, and no new WSL world provisioning state should appear as a consequence of these owned scripts.
