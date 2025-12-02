# Usage Guide

Daily usage patterns and integration examples for Substrate.

## Shell Usage

### Interactive Mode

```bash
substrate
# Substrate Shell v0.1.0
# Session ID: 018d1234-5678-7abc-def0-123456789abc
# Logging to: ~/.substrate/trace.jsonl
substrate> git status
substrate> npm test
substrate> exit
```

### Legacy REPL (fallback)

The async, event-driven REPL now powers interactive sessions by default and
streams agent output without busy-spinning. If you hit a regression, you can
drop back to the synchronous Reedline loop with:

```bash
substrate --legacy-repl
```

Use this flag only for troubleshooting; new features land exclusively in the
async path.

### Command Execution

```bash
# Single command
substrate -c "git status && npm test"

# Script execution with state preservation
substrate -f deploy.sh

# Pipe commands
echo "date" | substrate

# CI mode with strict error handling
substrate --ci -c "make test"
```

### Built-in Commands

The shell includes cross-platform built-ins handled without spawning a child process:
- `cd [dir]` - Change directory (supports `cd -`)
- `pwd` - Print working directory
- `export VAR=value` - Set environment variables
- `unset VAR` - Remove environment variables

`exit` / `quit` are intercepted by the interactive REPL loop and are not routed
through the builtin handler when running non-interactively (`substrate -c`,
scripts, etc.).

## PTY Support

Substrate automatically uses PTY for interactive commands:

### Automatic Detection

**TUI Applications**: `vim`, `less`, `top`, `htop`, `fzf`, `lazygit`
**Interactive Shells**: `bash`, `zsh` (without `-c` flag)
**Container Commands**: `docker run -it`, `kubectl exec -it`
**Git Interactive**: `git add -p`, `git rebase -i`, `git commit`

### Manual Control

```bash
# Force PTY for any command
substrate -c ":pty ls -la"

# Environment controls
export SUBSTRATE_FORCE_PTY=1      # Force PTY globally
export SUBSTRATE_DISABLE_PTY=1    # Disable PTY globally
export SUBSTRATE_PTY_DEBUG=1      # Enable PTY debug logging
```

## Command Interception & Manager Health

### Automatic Deployment

`substrate` still deploys shims on demand, but the installer no longer edits
`.bashrc`, `.zshrc`, or PowerShell profiles. Each CLI invocation builds a clean
PATH (`~/.substrate/shims:$SHIM_ORIGINAL_PATH`), exports
`SUBSTRATE_MANAGER_INIT`/`SUBSTRATE_MANAGER_ENV`, and sources the generated
manager snippet before executing your command. Host shells therefore keep their
original PATH and dotfiles.

```bash
substrate --shim-status   # show version + path
substrate --shim-deploy   # redeploy (idempotent)
substrate --shim-remove   # delete ~/.substrate/shims
SUBSTRATE_NO_SHIMS=1 substrate   # skip deployment for this invocation
```

On Linux, `substrate --shim-status[ --json]` also reports whether the world-agent
socket is managed by systemd socket activation (`substrate-world-agent.socket`)
or a manual listener, making it easier to distinguish provisioned services from
legacy bindings.

### PATH Isolation & Legacy Hosts

Compare the host vs Substrate PATH at any time:

```bash
printf "host PATH -> %s
" "$PATH"
substrate -c 'printf "substrate PATH -> %s
" "$PATH"'
```

Need to re-export manager snippets into a legacy shell? Run
`substrate shim repair --manager <name> --yes` to append the delimited block to
`~/.substrate_bashenv` (with a `.bak`). The CLI never edits the file unless you
explicitly request a repair.

### Manager Manifest & Overlays

- Shipping manifest: `config/manager_hooks.yaml`
- User overlay: `~/.substrate/manager_hooks.local.yaml`
- Override for tests/automation: `SUBSTRATE_MANAGER_MANIFEST=/path/to/manifest`

Each entry defines detect probes, init snippets, repair hints, and guest install
recipes. The manifest is platform-aware, and Substrate records the chosen path
in shim doctor output for transparency.

### Shim Doctor & Repair

```bash
substrate shim doctor              # text report (manifest, PATH, managers, hints)
substrate shim doctor --json | jq '.path'
substrate shim repair --manager nvm --yes
```

Doctor mode respects `HOME`, `USERPROFILE`, `SUBSTRATE_MANAGER_MANIFEST`, and
`SHIM_TRACE_LOG`, so tests can point it at temporary directories without touching
real dotfiles. Repair writes `~/.substrate_bashenv` (creating
`~/.substrate_bashenv.bak`) but leaves everything else untouched.

### Health Snapshots

Capture full host/guest readiness with the aggregated health command:

```bash
substrate health                # text summary (managers + world doctor + world deps)
substrate health --json | jq '.summary'
substrate shim doctor --json    # detailed shim-centric payload
```

Both commands honor the same overrides (`HOME`, `SUBSTRATE_MANAGER_MANIFEST`,
`SHIM_TRACE_LOG`). Drop fixture files into `~/.substrate/health/world_doctor.json`
and `~/.substrate/health/world_deps.json` when you need deterministic outputs in
tests or support bundles.

### Manager Hints & Telemetry

When shims intercept a failing command they emit `manager_hint` events in
`~/.substrate/trace.jsonl` (and attach the latest hint inside doctor output).
Use those records to confirm hint deduplication or to troubleshoot missing
managers:

```bash
jq 'select(.manager_hint) | .manager_hint' ~/.substrate/trace.jsonl | tail -5
```

Enable verbose logging with `SUBSTRATE_MANAGER_INIT_DEBUG=1` or skip certain
managers entirely via `SUBSTRATE_SKIP_MANAGER_INIT` /
`SUBSTRATE_SKIP_MANAGER_INIT_LIST`.

### Temporary HOMEs & CI

All CLI entry points honor standard overrides so you can stage sandboxes for
integration tests:

```bash
TMP=$PWD/target/tests-tmp/doctor
mkdir -p "$TMP/.substrate"
HOME=$TMP USERPROFILE=$TMP \
  SUBSTRATE_MANAGER_MANIFEST=$TMP/manager_hooks.yaml \
  SHIM_TRACE_LOG=$TMP/.substrate/trace.jsonl \
  substrate shim doctor --json
```

Windows follows the same pattern using `USERPROFILE`.

### Claude Code / Editor Integrations

Assistants that expect a sourced `BASH_ENV` can still rely on the generated
`~/.substrate/manager_env.sh`, but most workflows simply launch `substrate -c` or
`substrate` interactively and let the pass-through model set everything up. If
an agent needs to seed host shells, run `substrate shim repair` for the relevant
manager and point `BASH_ENV` at `~/.substrate_bashenv` explicitly.

### World Commands

- `substrate world doctor [--json]` – host readiness report (Linux namespaces,
  macOS Lima, Windows WSL)
- `substrate --no-world ...` – run commands directly on the host (no isolation)
- `substrate --world ...` – force world isolation for a single invocation even
  when install/config/env disables it (metadata remains unchanged)

- `substrate world enable` – provision the backend later if `--no-world` was
  used at install time
- `substrate world deps status|install|sync` – inspect and copy host toolchains
  into the guest once B3 reach parity (CLI scaffolding is already wired up)

World root (anchor) precedence, highest wins: CLI flags
(`--anchor-mode/--anchor-path`, legacy `--world-root-mode/--world-root-path`),
`.substrate/settings.toml` in the launch directory, `~/.substrate/config.toml`
`[world]`, environment variables `SUBSTRATE_ANCHOR_MODE/PATH` (legacy
`SUBSTRATE_WORLD_ROOT_MODE/PATH`), then the default `project` mode (rooted at
the launch directory). Modes: `project` anchors to the launch directory,
`follow-cwd` tracks your working directory, and `custom` uses the path supplied
via `anchor_path` or `--anchor-path`.

The installer and `substrate world enable` keep `~/.substrate/config.toml`
(`[install].world_enabled = true/false`) and rewrite `~/.substrate/manager_env.sh`
so `SUBSTRATE_WORLD`/`SUBSTRATE_WORLD_ENABLED` reflect the latest state without
needing to source dotfiles manually.

## Configuration CLI

Before the REPL starts, Substrate exposes a `config` command group for managing
`~/.substrate/config.toml` (or `%USERPROFILE%\.substrate\config.toml` on
Windows). The current verbs are `config init`, which scaffolds/regenerates the
file, `config show`, which prints it in TOML (or JSON) form, and `config set`,
which edits dotted keys without opening an editor:

```bash
# Create ~/.substrate/config.toml if it does not exist
substrate config init

# Regenerate the file even if it exists already
substrate config init --force
```

The command honors `SUBSTRATE_HOME`, making it safe to run from tests or
sandboxes. Shell startup and the installer scripts will emit a warning pointing
to `substrate config init` whenever the global config is absent, so re-running
the command is the supported remediation when the metadata is missing or
corrupted.

After the file exists, `substrate config show` prints the full contents with
redaction hooks for any future sensitive values. TOML is the default view:

```bash
$ substrate config show
[install]
world_enabled = true

[world]
anchor_mode = "project"
anchor_path = ""
root_mode = "project"
root_path = ""
caged = true
```

Use `--json` for automation (the same flag works on macOS/Linux and respects
`SUBSTRATE_HOME` overrides). Pipe to `jq` or the equivalent for readability:

```bash
$ substrate config show --json | jq '.'
{
  "install": {
    "world_enabled": true
  },
  "world": {
    "anchor_mode": "project",
    "anchor_path": "",
    "root_mode": "project",
    "root_path": "",
    "caged": true
  }
}
```

PowerShell users can call the same command and feed it through `ConvertFrom-Json`
to inspect the object:

```powershell
PS> substrate config show --json | ConvertFrom-Json | Format-List
install : @{world_enabled=True}
world   : @{anchor_mode=project; anchor_path=; root_mode=project; root_path=; caged=True}
```

When the file is missing the command exits non-zero with a reminder to run
`substrate config init`, matching the shell/install warnings.

`substrate config set key=value [...]` updates the same file with type-aware
validation and atomic writes. Provide one or more dotted keys to change several
fields at once:

```bash
$ substrate config set world.anchor_mode=follow-cwd world.caged=false
substrate: updated config at /Users/alice/.substrate/config.toml
  - world.anchor_mode: "project" -> "follow-cwd"
  - world.root_mode (alias): "project" -> "follow-cwd"
  - world.caged: true -> false
```

Pass `--json` for automation; the payload summarizes every field that changed:

```bash
$ substrate config set --json install.world_enabled=false
{
  "config_path": "/Users/alice/.substrate/config.toml",
  "changed": true,
  "changes": [
    {
      "key": "install.world_enabled",
      "alias": false,
      "old_value": true,
      "new_value": false
    }
  ]
}
```

On Windows, wrap each assignment in quotes so PowerShell keeps the backslashes:

```powershell
PS> substrate config set "world.anchor_mode=custom" "world.anchor_path=C:\Workspaces\repo" "world.caged=true"
```

Use `SUBSTRATE_WORLD_ENABLED=0` to force pass-through mode temporarily and
`SUBSTRATE_WORLD_DEPS_MANIFEST` to point world-deps at a custom definition file.
Flags beat config/env: `--world` overrides disabled metadata/env, while
`--no-world` always opts out.
## Log Analysis

Commands are logged in structured JSONL format:

```bash
# View recent commands
tail -5 ~/.substrate/trace.jsonl | jq '.command'

# Analyze session activity
jq 'select(.session_id == "your-session-id")' ~/.substrate/trace.jsonl

# Command frequency analysis
jq -r '.command' ~/.substrate/trace.jsonl | sort | uniq -c | sort -nr

# Performance analysis
jq '.duration_ms // empty' ~/.substrate/trace.jsonl | awk '{sum+=$1} END {print "avg:", sum/NR "ms"}'
```

## Security Features

### Credential Redaction

Automatic redaction of sensitive information:

```bash
# Original command
curl -H "Authorization: Bearer secret123" api.example.com

# Logged as
["curl", "-H", "Authorization: ***", "api.example.com"]

# Disable redaction for debugging
SHIM_LOG_OPTS=raw curl -H "Authorization: Bearer token" api.com
```

### Emergency Bypass

```bash
# Bypass single command
SHIM_BYPASS=1 git status

# Complete system rollback
./scripts/rollback.sh
```

## Troubleshooting

### Debug Mode

```bash
# Enable comprehensive debugging
export RUST_LOG=debug
export SHIM_LOG_OPTS=raw      # or set to resolve to log resolved paths
export SUBSTRATE_PTY_DEBUG=1

# Test with debugging
substrate -c "git status"
```

### Common Solutions

| Problem | Solution |
|---------|----------|
| Commands not intercepted | Check PATH order, run `hash -r` |
| Infinite loops | Ensure shim directory not in `SHIM_ORIGINAL_PATH` |
| PTY issues | Check `SUBSTRATE_DISABLE_PTY`, enable debug mode |
| Permission denied | Verify shim binary permissions |

### Log File Management

```bash
# Check log file permissions
ls -la ~/.substrate/trace.jsonl

# Rotation is built-in and controlled via env:
#   TRACE_LOG_MAX_MB (default 100), TRACE_LOG_KEEP (default 3)
# Use external tools only for archival/compression if needed.
gzip ~/.substrate/trace.jsonl.*  # Optional archival step
```

## Integration Examples

### CI/CD Pipelines

```bash
# Strict error handling
substrate --ci -c "npm test && npm build"

# Continue on error
substrate --ci --no-exit-on-error -f integration-tests.sh
```

### Development Workflows

```bash
# Interactive development
substrate
substrate> git checkout feature-branch
substrate> npm install
substrate> npm test
substrate> git commit -m "Add feature"

# Automated workflows
substrate -f scripts/deploy.sh
```

For more advanced usage patterns and future capabilities, see [VISION.md](VISION.md).

## Graph CLI (mock backend)

Substrate includes a simple graph CLI backed by an in-memory mock service to preview graph features:

```
# Show status
substrate graph status

# Ingest your trace file
substrate graph ingest ~/.substrate/trace.jsonl

# List files changed for a span
substrate graph what-changed <SPAN_ID> --limit 100
```

Notes
- The mock backend does not persist across runs. A Kuzu backend is planned for Phase 4.5.
