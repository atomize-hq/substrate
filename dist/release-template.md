# Release Template

## Highlights
- Summarise the largest changes since the previous release.
- Call out any required upgrade actions or platform notices.

## Quickstart
- Linux/macOS: reference `scripts/substrate/install-substrate.sh` and [docs/INSTALLATION.md](../docs/INSTALLATION.md).
- macOS Lima backend: point to [scripts/mac/lima-warm.sh](../scripts/mac/lima-warm.sh) and the Lima walkthrough.
- Windows (WSL): link to `scripts/windows/wsl-warm.ps1` and [docs/WORLD.md](../docs/WORLD.md#windows-wsl-backend).

## Downloads
- Each release attaches one bundle per host platform. Look for assets named `substrate-v<version>-<target>.tar.gz` (Linux/macOS) or `substrate-v<version>-windows_x86_64.zip` alongside `substrate-support.{tar.gz,zip}`.
- The bundled archives already include `substrate`, `substrate-shim`, `host-proxy`, `world-agent` (Linux guest), and `substrate-forwarder.exe` (Windows). The support archive carries docs + helper scripts referenced by the installers.
- Installers now fetch these bundles automatically, so manual downloads are only needed for airgapped hosts or custom validation.

## Full Changelog
- Copy the relevant section from `CHANGELOG.md`.
- Include notable issues or workarounds discovered during nightly validation.
