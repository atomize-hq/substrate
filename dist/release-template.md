# ${RELEASE_TAG}

## Quickstart

- **Linux / macOS** (ships binaries + Lima/WSL helpers):

  ```bash
  curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/${RELEASE_TAG}/scripts/substrate/install.sh | bash
  ```

- **Windows (PowerShell, run as Administrator):**

  ```powershell
  iwr https://raw.githubusercontent.com/atomize-hq/substrate/${RELEASE_TAG}/scripts/windows/install-substrate.ps1 -UseBasicParsing | iex
  ```

- **Uninstall (all platforms):**
  - Linux/macOS: `curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/${RELEASE_TAG}/scripts/substrate/uninstall.sh | bash`
  - Windows: `iwr https://raw.githubusercontent.com/atomize-hq/substrate/${RELEASE_TAG}/scripts/windows/uninstall-substrate.ps1 -UseBasicParsing | iex`

Refer to `docs/INSTALLATION.md` and `docs/WORLD.md` for advanced flags, prereqs, and Lima/WSL troubleshooting flows.

## Bundles

| OS | Arch | Bundle |
| --- | --- | --- |
| Linux | x86_64 | [substrate-v${VERSION}-linux_x86_64.tar.gz](https://github.com/atomize-hq/substrate/releases/download/${RELEASE_TAG}/substrate-v${VERSION}-linux_x86_64.tar.gz) |
| Linux | aarch64 | [substrate-v${VERSION}-linux_aarch64.tar.gz](https://github.com/atomize-hq/substrate/releases/download/${RELEASE_TAG}/substrate-v${VERSION}-linux_aarch64.tar.gz) |
| macOS | x86_64 | [substrate-v${VERSION}-macos_x86_64.tar.gz](https://github.com/atomize-hq/substrate/releases/download/${RELEASE_TAG}/substrate-v${VERSION}-macos_x86_64.tar.gz) |
| macOS | arm64 | [substrate-v${VERSION}-macos_arm64.tar.gz](https://github.com/atomize-hq/substrate/releases/download/${RELEASE_TAG}/substrate-v${VERSION}-macos_arm64.tar.gz) |
| Windows | x86_64 | [substrate-v${VERSION}-windows_x86_64.zip](https://github.com/atomize-hq/substrate/releases/download/${RELEASE_TAG}/substrate-v${VERSION}-windows_x86_64.zip) |

Each bundle includes `substrate`, `substrate-shim`, `world-agent` (Linux guest binaries live under `bin/linux` for macOS/Windows), and platform helpers like `host-proxy` and `substrate-forwarder.exe`.

## Support + Checksums

- Docs & helper scripts: [substrate-support.tar.gz](https://github.com/atomize-hq/substrate/releases/download/${RELEASE_TAG}/substrate-support.tar.gz) · [substrate-support.zip](https://github.com/atomize-hq/substrate/releases/download/${RELEASE_TAG}/substrate-support.zip)
- Checksums: [SHA256SUMS](https://github.com/atomize-hq/substrate/releases/download/${RELEASE_TAG}/SHA256SUMS)
- Verify on Linux/macOS: `curl -fsSLO https://github.com/atomize-hq/substrate/releases/download/${RELEASE_TAG}/SHA256SUMS && sha256sum --check SHA256SUMS`
- Verify on Windows (PowerShell): `Get-FileHash -Algorithm SHA256 .\substrate-v${VERSION}-windows_x86_64.zip`

---

Post any regressions or provisioning issues to the tracker with the bundle name, platform, and the doctor/smoke output. Happy hacking! :rocket:
