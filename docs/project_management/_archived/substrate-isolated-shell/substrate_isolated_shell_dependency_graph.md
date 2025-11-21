# Substrate Isolated Shell – Dependency Graph

This file describes the ordered dependencies between the new modules, scripts, and behaviors required for the isolated shell rollout. Treat each node as a deliverable; edges denote “must exist before”.

## Legend

- **Node**: `<name> (owner/component)`
- `A → B`: B depends on A.

## Graph (high level)

```
Manifest Parser (common) ──┐
                           │
                           ▼
Manager Init Module (shell) ──→ Shell Env Injection (shell/lib.rs)
                                       │
                                       ▼
Shim Hinting Enhancements (shim)

Shell Env Injection ──→ Shim Doctor CLI ──→ Docs/Config Updates
                                   │
                                   ▼
World CLI Extensions (shell) ──→ Installer Upgrades ──→ World Enable Command

World CLI Extensions ──→ World Deps Sync Runner (world-agent/backends)

Installer Upgrades ──→ Host Config Metadata ──→ `--no-world` Runtime Switches
```

## Detailed Dependencies

1. **Manifest Parser (`crates/common`)**
   - Required by both shell (manager init) and shim (hint table). Must land first.
2. **Manager Init Module (`crates/shell`)**
   - Depends on manifest parser; produces runtime snippet + telemetry state.
   - Needed before we can update shell env injection.
3. **Shell Env Injection (`crates/shell/src/lib.rs`, `async_repl.rs`, scripts)**
   - Requires manager init module and installer changes (to create snippet files).
   - Provides environment for shim commands; must precede shim hinting to ensure `no_world` propagation works.
4. **Shim Hinting (`crates/shim/src/exec.rs`)**
   - Consumes manifest data; depends on env injection for new variables/hints.
5. **Shim Doctor CLI (`crates/shell`)**
   - Requires manager init telemetry + shim hint logs.
   - Coupled with doc/config updates.
6. **World CLI Extensions (enable + deps)**
   - Dependent on manifest (for guest recipes) and shell env injection (for PATH control).
   - Before installer upgrades so upgrade command exists.
7. **Installer Upgrades (`scripts/...`)**
   - Need world CLI to exist (for `world enable`).
   - After shell env injection to know which files to create.
8. **World Deps Sync Runner (world-agent/backends)**
   - Depends on world CLI to issue commands; may require backend support.
9. **Host Config Metadata (`~/.substrate/config.json`)**
   - Created by installer; consumed by shell to set `no_world` default.
10. **`--no-world` Runtime Switches**
    - Depend on host metadata + shell env injection to skip manager snippet, and shim updates to bypass world logic.

## Execution Notes

- Workstreams can proceed concurrently where edges permit (detailed in execution plan). For example, manifest parser + shell manager-init can start together, while installer upgrades wait until world CLI exists.
- Each dependency must have an integration test verifying the contract before downstream work merges.

Use this graph to coordinate PR sequencing and avoid breaking dependent teams.***
