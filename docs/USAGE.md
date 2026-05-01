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

Normal interactive `exit` / `quit` ends the REPL with process exit code `0`. If the controlling terminal is revoked or disconnected after startup, the operator contract is the `0` versus `1` split: Substrate exits with code `1` and may emit one best-effort abnormal-terminal-loss diagnostic before it terminates. The landed macOS revoke harness in `crates/shell/tests/repl_tty_disconnect_macos.rs` is the highest-confidence proof for that path, not a blanket platform guarantee.

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

### Agent Commands

The successor agent control-plane surface lives under the singular
`substrate agent` namespace:

```bash
substrate agent list
substrate agent list --json
substrate agent status --scope world
substrate agent doctor --json
substrate agent toolbox status --json
substrate agent toolbox env
```

`substrate agents validate` remains available as the inventory-validation
compatibility leaf. It does not alias `substrate agent list`, `status`,
`doctor`, or `toolbox ...`.

Operator-visible identity rules on these surfaces:
- `backend_id` always renders as `<kind>:<agent_id>`.
- Pure-agent list rows omit `provider`, `auth_authority`, `world_id`, and `world_generation`.
- Pure-agent status rows omit `provider` and `auth_authority`.
- `world_id` and `world_generation` render only for world-scoped pure-agent session rows.
- Nested gateway-backed status rows stay separate from pure-agent rows and are the only rows that publish `provider` and `auth_authority`.
- Nested gateway-backed status rows depend on valid trace-side `parent_run_id` correlation; stale historical nested rows are ignored, and malformed selected-surface rows fail closed.
- `substrate agent doctor` now includes a `runtime_realizability` check after `orchestrator_selection`. For the selected orchestrator, it fail-closes on unsupported `config.kind`, unsupported `cli.mode`, unsupported shell-owned backend mapping, or an unresolved `config.cli.binary`.
- `substrate agent toolbox status` is a pre-runtime introspection surface: it projects the effective toolbox posture, the selected orchestrator identity, and either the active per-session UDS endpoint or the deterministic endpoint template when no orchestrator session is active yet.
- `substrate agent toolbox env` emits `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` and `SUBSTRATE_AGENT_TOOLBOX_VERSION` only when a current pure-agent orchestrator session is present; otherwise it fails closed with a specific exit code.

When the async REPL owns a shell-scoped orchestrator session, live session discovery is backed by the store-owned session root:
- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json`
- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/participants/<participant_id>.json`

`substrate agent status` and `substrate agent toolbox ...` resolve live state from those session records first. `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>.json`, `~/.substrate/run/agent-hub/participants/*.json`, and `~/.substrate/run/agent-hub/handles/*.json` remains compatibility input only during the cutover and must not be treated as live-state authority.

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
substrate health             # text summary (shim doctor + world backend + world deps)
substrate health --json | jq '.summary'
substrate shim doctor --json # detailed shim-centric payload
```

`substrate health` is a thin summary over `substrate shim doctor` and the canonical machine-readable inputs are `.shim.world.status` and `.shim.world_deps.status`:
- `summary.missing_managers` – managers not detected on the host.
- `summary.world_ok` / `summary.world_error` – world backend health when the world is enabled.
- `summary.world_deps_missing` / `summary.world_deps_blocked` – enabled deps missing or blocked in the world.
- `summary.world_deps_error` – world deps snapshot unavailable (world backend down, etc).
- Disabled mode sets `summary.world_ok = null`, omits `summary.world_error` and `summary.world_deps_error`, and keeps `summary.world_deps_missing` / `summary.world_deps_blocked` empty.

Example (Linux, bash):

```bash
$ substrate health
== substrate health ==
Managers detected: 4/5
  Not detected on host (info): bun
World backend: healthy
World deps: missing (1): asdf
  Next: run `substrate world deps current sync` then `substrate world deps current list applied`
Hints recorded: 0
Overall status: attention required
  - world deps missing (enabled): asdf
```

Machine-readable summary:

```bash
$ substrate health --json | jq '.summary | {ok, missing_managers, world_ok, world_deps_missing, world_deps_blocked, world_deps_error}'
```

When `.shim.world.status` is `disabled` and `.shim.world_deps.status` is `skipped_disabled`, the summary stays non-error and the human output prints the disabled contract lines instead of enabled-world remediation guidance.

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

- `substrate host doctor [--json]` – host readiness for world routing (host prerequisites + transport readiness)
- `substrate world doctor [--json]` – world readiness report (includes host doctor + world-agent facts)
- `substrate world gateway sync|status|restart` – operator gateway lifecycle/status entrypoints.
- `substrate world gateway status` is the stable operator entrypoint for gateway availability and discovery.
- `substrate world gateway status --json` is the authoritative machine-readable status surface for that entrypoint.
- Human-readable `substrate world gateway status` may abbreviate, but it does not redefine the machine-readable meaning.
- `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL` are the stable non-secret wiring exports; they point to Substrate-managed gateway endpoints for in-world clients, not upstream provider endpoints or a direct-host reachability guarantee.
- Exit `4` is reserved for actual absent component results: an explicit runtime `unavailable` response or a missing required gateway/world listener.
- Start and restart windows, connection-refused handoff gaps, and ready-timeout lifecycle failures are transient runtime failures and exit `3`, not `4`.
- Exit `2` remains invalid integration and exit `5` remains policy/safety failure.
- `substrate --no-world ...` – run commands directly on the host (no isolation)
- `substrate --world ...` – force world isolation for a single invocation even
  when install/config/env disables it (metadata remains unchanged)

- `substrate world enable` – provision the backend later if `--no-world` was
  used at install time; add `--provision-deps` to provision APT-backed world deps on supported guest backends
- `substrate world deps current list [available|enabled|applied] [--json]` – inspect the effective inventory/enabled/applied views for the current directory.
- `substrate world deps current sync [--dry-run] [--verbose]` – apply the effective enabled deps list into the world; APT-backed items are probe-only at runtime and remediate to `substrate world enable --provision-deps` when packages are missing.
- `substrate world deps current install <ITEM...> [--dry-run] [--verbose]` – apply specific deps immediately without changing enabled config; APT-backed items are probe-only at runtime.
- `substrate world deps global|workspace add|remove|reset` – edit enabled patches only (no install/uninstall).

World root (anchor) precedence, highest wins: CLI flags
(`--anchor-mode/--anchor-path`, legacy `--world-root-mode/--world-root-path`),
`.substrate/settings.yaml` in the launch directory, `~/.substrate/config.yaml`
`world:`, environment variables `SUBSTRATE_ANCHOR_MODE/PATH` (legacy
`SUBSTRATE_WORLD_ROOT_MODE/PATH`), then the default `project` mode (rooted at
the launch directory). Modes: `project` anchors to the launch directory,
`follow-cwd` tracks your working directory, and `custom` uses the path supplied
via `anchor_path` or `--anchor-path`.

The installer and `substrate world enable` keep `~/.substrate/config.yaml`
(`install.world_enabled: true/false`) and rewrite `~/.substrate/manager_env.sh`
so `SUBSTRATE_WORLD`/`SUBSTRATE_WORLD_ENABLED` reflect the latest state without
needing to source dotfiles manually.

## Configuration CLI

Before the REPL starts, Substrate exposes a `config` command group for managing
`~/.substrate/config.yaml` (or `%USERPROFILE%\.substrate\config.yaml` on
Windows). The current verbs are `config init`, which scaffolds/regenerates the
file, `config show`, which prints it in YAML (or JSON) form, and `config set`,
which edits dotted keys without opening an editor:

```bash
# Create ~/.substrate/config.yaml if it does not exist
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
redaction hooks for any future sensitive values. YAML is the default view:

```bash
$ substrate config show
install:
  world_enabled: true
world:
  anchor_mode: project
  anchor_path: ""
  root_mode: project
  root_path: ""
  caged: true
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
substrate: updated config at /Users/alice/.substrate/config.yaml
  - world.anchor_mode: "project" -> "follow-cwd"
  - world.root_mode (alias): "project" -> "follow-cwd"
  - world.caged: true -> false
```

Pass `--json` for automation; the payload summarizes every field that changed:

```bash
$ substrate config set --json install.world_enabled=false
{
  "config_path": "/Users/alice/.substrate/config.yaml",
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

Use `--no-world` (or `SUBSTRATE_OVERRIDE_WORLD=disabled`) to force pass-through mode temporarily.
`SUBSTRATE_WORLD_ENABLED` is exported state (output-only) and should not be set by users.
Legacy `SUBSTRATE_WORLD_DEPS_MANIFEST` is ignored by `substrate world deps` (packages/bundles contract).
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
