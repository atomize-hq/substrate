# Release Template

## Highlights
- Summarise the largest changes since the previous release.
- Call out any required upgrade actions or platform notices.

## Quickstart
- **Linux/macOS:** run the installer directly from GitHub (adds `substrate`, deploys shims, and provisions Lima when needed):

  ```bash
  curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/substrate/install-substrate.sh | bash
  ```

  See [docs/INSTALLATION.md](../docs/INSTALLATION.md) for advanced options.

- **Windows (PowerShell):** elevate PowerShell, then run:

  ```powershell
  iwr https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/windows/install-substrate.ps1 -UseBasicParsing | iex
  ```

  The script handles WSL provisioning and forwarder startup. Extra details live in [docs/WORLD.md](../docs/WORLD.md#windows-wsl-backend).

- **Uninstall:** if you hit trouble and want to roll back quickly, use the matching uninstall scripts:
  - Linux/macOS: `curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/substrate/uninstall-substrate.sh | bash`
  - Windows: `iwr https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/windows/uninstall-substrate.ps1 -UseBasicParsing | iex`

- **macOS Lima helpers:** mention [scripts/mac/lima-warm.sh](../scripts/mac/lima-warm.sh), `lima-doctor.sh`, and the Lima walkthrough for troubleshooting.

## Downloads
- Each release attaches one bundle per host platform. Look for assets named `substrate-v<version>-<target>.tar.gz` (Linux/macOS) or `substrate-v<version>-windows_x86_64.zip` alongside `substrate-support.{tar.gz,zip}`.
- The bundled archives already include `substrate`, `substrate-shim`, `host-proxy`, `world-agent` (Linux guest), and `substrate-forwarder.exe` (Windows). The support archive carries docs + helper scripts referenced by the installers.
- Installers now fetch these bundles automatically, so manual downloads are only needed for airgapped hosts or custom validation.

## Full Changelog
- Copy the relevant section from `CHANGELOG.md`.
- Include notable issues or workarounds discovered during nightly validation.
