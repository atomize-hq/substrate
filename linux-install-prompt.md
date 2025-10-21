# Linux Installer Synchronization Prompt (Substrate v0.2.0-beta)

You are taking over the Linux/WSL installer work for Substrate. The macOS path is complete and serves as reference. Repository root:

`/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`

## Review These Files/Docs First
- `install-substrate.sh` (root)
- `uninstall-substrate.sh` (root)
- `release/0.2.0-beta/` (bundle layout + SHA256SUMS)
- `docs/INSTALLATION.md`, `docs/UNINSTALL.md`
- `docs/project_management/next/world-deps-sync.md` (future work awareness)
- `scripts/mac/lima/*.yaml` (patterns for provisioning)
- `docs/project_management/standards/rustStandards.md`

## Current (macOS) Behavior
- Installs host binaries, deploys shims, provisions Lima, installs Linux world-agent, runs `substrate world doctor`.
- `SHIM_ORIGINAL_PATH` + PATH adjustments follow documentation.
- `uninstall-substrate.sh` removes PATH snippets, `~/.substrate*`, Lima VM.
- Tarball & SHA256SUMS rebuilt to include installer/uninstaller/docs.

## Linux/WSL Work Items
1. Extend `install-substrate.sh` for native Linux & WSL: dependency checks, shim deployment, world provisioning (systemd service), PATH setup, doc alignment.
2. Update `uninstall-substrate.sh` to remove Linux/WSL state (service removal, directories, optional WSL cleanup).
3. Update documentation: Linux/WSL install/uninstall instructions & quickstart coverage in bundle docs.
4. Mirror mac bundle structure for Linux/WSL tarballs; regenerate SHA256SUMS.
5. Testing evidence: dry run logs, actual install, `substrate world doctor`, uninstall validation.

## Constraints
- Conform to `docs/project_management/standards/rustStandards.md`.
- Match mac behavior unless platform-specific differences apply (document them).
- Dependency sync (world deps) is future work; do not implement yet.
- Ensure `curl | bash` UX works from release bundle.

## Deliverables
- Updated `install-substrate.sh` with Linux/WSL flow.
- Updated `uninstall-substrate.sh` covering Linux/WSL.
- Refreshed release artifacts (tarball + SHA256SUMS).
- Documentation updates (`docs/**`).
- Summary of validation steps/logs.

