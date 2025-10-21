# World Dependency Sync UX (Draft)

## Context

Every shimmed command runs inside the Lima VM. Host-only binaries (macOS builds) either fail with “command not found” or “Exec format error”. We need a guided flow that installs Linux equivalents in the guest so the first substrate session feels seamless.

## Goals

- Make initial `substrate` usage ergonomic; common tools work out of the box.
- Preserve isolation by running Linux binaries inside the guest.
- Remain explicit/opt-in: users see what gets installed and can customize.
- Provide ongoing tools to sync newly-added host commands.

## Proposed CLI Enhancements

| Command | Purpose | Notes |
| --- | --- | --- |
| `substrate host <cmd…>` | Run a command on the host (world disabled) | Escape hatch for macOS-only binaries |
| `substrate world deps status` | Show host vs guest tool availability | Manifest + actual command checks |
| `substrate world deps install <tool…>` | Install specified tools in the guest | Uses declarative recipes |
| `substrate world deps sync [--all]` | Install all missing tools that exist on host | Optional prompt/flag |
| `substrate world deps sync --reverse <tool>` | Optional: copy guest tool back to host | Low priority |
| `substrate world deps add <tool> --detect '<cmd>' --install '<script>'` | User-supplied recipes | Stored in overlay manifest |

### Installer Integration

- `install-substrate.sh --sync-deps` prompts after shim deployment: “Install Linux versions of git/node/python? [Y/n]”
- Users can rerun `substrate world deps sync` later.

## Manifest Concept

```yaml
tools:
  git:
    detect: "git --version"
    install:
      apt: "sudo apt-get update && sudo apt-get install -y git"
  node:
    detect: "node --version"
    install:
      custom: |
        curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
        sudo apt-get install -y nodejs
  python:
    detect: "python3 --version"
    install:
      apt: "sudo apt-get install -y python3 python3-venv python3-pip"
```

- Base manifest ships in repo (`scripts/install/deps.yaml`).
- Optional user overlay (`~/.substrate/deps.local.yaml`).
- Support providers: `apt`, `snap`, `cargo`, `custom` script, etc.

## UX Walkthrough

1. **Installer** – Detect host tools, prompt to install equivalents inside Lima.
2. **Interactive** – `substrate> world deps status`, then `substrate> world deps install node npm` as needed.
3. **Host command** – `substrate> host claude --version` for occasional macOS-only binaries.
4. **Custom recipe** – `substrate> world deps add codex --detect 'codex --version' --install 'curl …'`.

## Technical Tasks

1. Manifest loader (serde) supporting base + overlay.
2. Detection of host vs guest command availability.
3. Installer flag/prompt for initial sync.
4. New `substrate` subcommands: `world deps status/install/sync/add`.
5. Recipe execution helpers (apt/custom), with basic logging.
6. Docs updates (installation guide + usage doc).
7. Optional future work: reverse sync, GUI/log output, etc.

## Next Steps

1. Confirm command naming and manifest format.
2. Build manifest loader and detection utilities.
3. Implement CLI subcommands and installer integration.
4. Document workflow once shipped.
