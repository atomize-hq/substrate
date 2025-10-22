# Windows Installer & Uninstall Parity Prompt (Substrate v0.2.0-beta)

You are taking over the Windows host / WSL installer work for Substrate. The
macOS and Linux installers are now unified under `scripts/substrate/` and the
release bundles have been updated accordingly. Your goal: deliver a Windows
experience that matches feature parity, packaging layout, and documentation with
those existing flows.

## Repository Orientation

Start from repo root (`/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`).
Familiarize yourself with the following locations before making changes:

- `scripts/substrate/install-substrate.sh` – canonical installer logic (macOS & Linux)
- `scripts/substrate/uninstall-substrate.sh` – teardown logic mirrored in docs
- `docs/INSTALLATION.md`, `docs/UNINSTALL.md` – current platform guidance
- `release/0.2.0-beta/` – beta bundle layout and regenerated `SHA256SUMS`
- `docs/project_management/next/world-deps-sync.md` – installer integration plans
- `docs/project_management/standards/rustStandards.md` – coding / doc standards
- `docs/WORLD.md`, `docs/cross-platform/wsl_world_setup.md`, `docs/cross-platform/wsl_world_troubleshooting.md` – WSL backend expectations
- `scripts/windows/` – existing warm, doctor, smoke scripts (`wsl-warm.ps1`, `wsl-smoke.ps1`, etc.)
- `crates/world-windows-wsl`, `crates/forwarder`, `crates/host-proxy` – Windows transport implementation
- `release/0.2.0-beta/linux_x86_64/` – reference bundle structure (bin/, docs/, scripts/)

## Windows / WSL Installer Deliverables

1. **PowerShell Installer**
   - Create a canonical script (e.g., `scripts/windows/install-substrate.ps1`) that:
     - Detects Windows version, verifies WSL is enabled, and checks that the target distro has systemd enabled (or guides the user to enable it).
     - Downloads the Windows ZIP or consumes a local `-Archive` override (mirrors Linux flags `-Version`, `-Prefix`, `-NoWorld`, `-NoShims`, `-DryRun`, `-DistroName`).
     - Installs host binaries to `%LOCALAPPDATA%\Substrate`, writes a profile helper (`substrate-profile.ps1`) that prepends `bin\` and `shims\`, and records the original PATH.
     - Skips shim deployment when `-NoShims` is present; otherwise runs `substrate.exe --shim-deploy` once.
     - If world provisioning is enabled, warms/imports `substrate-wsl`, installs the Linux `world-agent` binary inside WSL (using the bundled copy if Cargo isn’t available), and starts the forwarder.
     - Runs `substrate.exe world doctor --json` using only the installed bin directory plus the original PATH (no shim directory) so diagnostic checks mirror the Linux/macOS installers.
   - Ensure the installer can be invoked via `powershell -ExecutionPolicy Bypass -File ...` and via `iex (irm ...)` once a short URL is provided. Use the GitHub raw URL placeholder for now.

2. **PowerShell Uninstaller**
   - Implement `scripts/windows/uninstall-substrate.ps1` to:
     - Stop/remove Windows services/tasks (forwarder, any scheduled jobs, PATH env modifications).
     - Remove `%ProgramData%` / `%LocalAppData%` state, shim directories, shared config.
     - Tear down the `substrate-wsl` distro (prompt confirmation) and clean `/run/substrate.sock` proxies.
     - Optionally remove generated WSL logs and forwarded sockets.
     - Provide dry-run mode to list actions without executing.

3. **Release Packaging**
   - Mirror the bundle layout used for Linux in a new Windows archive under `release/0.2.0-beta/windows_x86_64/` (or similar):
     - `bin\` – Windows host executables (`substrate.exe`, `substrate-forwarder.exe`, etc.).
     - `docs\` – Windows quickstart + README referencing PowerShell installer.
     - `scripts\` – include `windows\install-substrate.ps1`, `windows\uninstall-substrate.ps1`, plus warm/doctor utilities.
   - Regenerate tarball/zip (likely `.zip` for Windows) and update `release/0.2.0-beta/SHA256SUMS` with hashes for all artifacts (macOS, Linux, Windows).

4. **Documentation Updates**
   - `docs/INSTALLATION.md`: add comprehensive Windows host instructions mirroring Linux/macOS, including `curl | bash` equivalent (`iex (irm ...)`).
   - `docs/UNINSTALL.md`: document Windows uninstall steps and verification.
   - Add/update quickstart docs inside the Windows bundle (similar to `release/0.2.0-beta/linux_x86_64/docs/QUICKSTART.md`).
   - Reference new scripts in `docs/project_management/next/world-deps-sync.md`, `docs/WORLD.md`, and troubleshooting guides.

5. **Validation Evidence**
   - Capture dry-run, full install, and uninstall transcripts (PowerShell output).
   - Run `substrate world doctor --json` inside Windows Terminal and attach logs.
   - Execute `scripts/windows/wsl-smoke.ps1` and document pass/fail results.
   - Ensure `SHA256SUMS` matches newly generated artifacts.

## Key Considerations

- Maintain parity with shell installer flags (`--Version`, `--Prefix`, `--NoWorld`, `--NoShims`, `--DryRun`) for consistency across platforms.
- Support both online (downloads from GitHub releases) and offline (`--Archive` pointing at local bundle) workflows.
- Reuse existing WSL warm/doctor logic where possible (see `scripts/windows/wsl-warm.ps1`, `docs/cross-platform/wsl_world_setup.md`).
- Integrate Windows service management via `New-Service` or `schtasks` per design constraints documented in `docs/cross-platform/transport_parity_design.md`.
- Use the same GitHub raw URLs for now when instructing users to download scripts; a friendly domain will replace them later.
- Ensure environment variable changes are reversible by the uninstall script (PowerShell profile edits, registry PATH modifications).
- Follow `docs/project_management/standards/rustStandards.md` for comment style, error handling, and testing expectations.

## Success Criteria Checklist

- [ ] `scripts/windows/install-substrate.ps1` + `scripts/windows/uninstall-substrate.ps1` implemented, linted, and documented.
- [ ] Windows bundle under `release/0.2.0-beta/windows_x86_64/` structured like other platforms.
- [ ] `SHA256SUMS` includes macOS, Linux, and Windows artifacts with regenerated hashes.
- [ ] `docs/INSTALLATION.md` and `docs/UNINSTALL.md` describe Windows workflows using new scripts.
- [ ] `docs/` cross-platform references updated to point at the new PowerShell tooling.
- [ ] Validation logs captured for install/uninstall + doctor + smoke tests on Windows host.
- [ ] Final summary detailing what was tested, remaining gaps, and deployment notes.

Treat this prompt as the canonical handoff for Windows parity work. EOF
