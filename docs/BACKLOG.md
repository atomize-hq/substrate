# Substrate Backlog

Status: living document capturing near-term and upcoming work.
Keep concise, actionable, and security-focused.

## Next

- **P0 – Socket-activated world-agent service (parallelizable --  runs concurrently with: "Replay polish – isolation + verbose scopes")**
  - Current provisioning only installs `substrate-world-agent.service`; the agent binds `/run/substrate.sock` itself and must stay running. Introduce a matching `.socket` unit so systemd listens on the socket, launches the agent on demand, and restarts it transparently.
  - Work items:
    - Update `world-agent` to accept an inherited listener (LISTEN_FDS) in addition to binding directly; keep the existing code path for non-systemd environments.
    - Teach `ensure_world_agent_ready()` to tolerate socket-activated setups (socket already present, service only starts when probed).
    - Extend Linux/Lima/WSL installers, world-enable helper, and uninstall scripts to deploy/remove both `.service` and `.socket` units.
    - Refresh docs/tests to describe the new flow; ensure guidance around sudo/user installs and permissions highlights the benefit.
  - Can run in parallel with other features—touches agent + provisioning scripts but does not block shell, tracing, or UX work.

- **P0 - Replay polish – isolation + verbose scopes (parallelizable -- runs concurrently with: "Socket-activated world-agent service")**
  - *Isolation follow-up:* Most of the Phase 4.5 isolation plan shipped (per-replay netns + nft scoping), but optional enhancements remain: nft cgroup matching fallback, documentation updates (`COMPLETE_FIXES_PHASE4_PRE45.md`), and diagnostic tooling for leftover netns/rules.
  - *Verbose scopes:* When running `substrate --replay --replay-verbose`, show a concise `scopes: [...]` line next to the “world strategy” output so operators can see which policy scopes were exercised per replay.
  - *Clear warnings:* Differentiate shell vs. replay world warnings—shell path messages should explicitly say “shell world-agent path”, while replay warnings keep the `[replay] …` prefix.
  - *Default-to-world replay tests:* Add integration coverage for the default world-on path, `--no-world`, and env opt-out so replay regressions are caught automatically.
  - Treat all of the above as a single replay-focused bucket so backend polish and CLI visibility ship together (shared tests/docs).  


- **P0 - Health command manager mismatch bug (parallelizable -- runs concurrently with: "Socket-activated world-agent service" AND "Replay polish – isolation + verbose scopes")**
  - `substrate health` currently reports “attention required” whenever optional manager detection hooks (direnv, asdf, conda, etc.) aren’t found on the host, even though the host never had them. We only care when the world and host detection disagree (host has a manager, world doesn’t), not when both sides are missing a manager entirely.
  - Fix: adjust health summary logic to only flag mismatches when the host reports a manager and the world fails to mirror it. Missing managers that the host doesn’t have should not trigger an “attention required” status.

- **P1 – Policy-driven world fs mode**
  - Problem: write permissions inside worlds currently depend on systemd hardening + overlay success, not on broker policy. Sensitive repos need a policy bit to force read-only worlds while other projects remain writable, without editing unit files manually.
  - Work: extend broker schema to accept `world.fs_mode = read_only|writable` (global + per-project), plumb into shell/world-agent so PTY + non-PTY sessions honor it, and update docs/doctor to surface the active mode. Systemd units must allow `/home` writes so policy can enforce RO vs writable deterministically.
  - Acceptance: policy defaults to writable; flipping to read-only blocks writes (clear errors, trace telemetry); installers/docs explain the knob and tests cover both modes (with skip notes for hosts lacking overlay/cgroup permissions).

- **P1 fs_diff parity (agent HTTP + PTY)**
  - *Agent HTTP path:* Today only replay/local backends attach `fs_diff`; agent-routed non-PTY commands drop the diff. Extend `agent-api-types::ExecuteResponse` / `world-agent` so `/v1/execute` returns `fs_diff: Option<FsDiff>` and update the shell to record it in completion spans. Acceptance: `fs_diff` shows up in `trace.jsonl` for agent HTTP runs.
  - *PTY sessions:* Interactive runs still lack filesystem diffs. Explore capturing post-exit diffs via overlayfs/copydiff and plumb the result through the PTY telemetry path so REPL + `substrate -i` sessions produce the same audit artifacts as non-PTY commands. Document caveats (long-running PTYs, partial diffs) and add tests to prove PTY diffs land in spans.

- **P1 – Record `world_id` for REPL/local exec spans**
  - Today `trace.jsonl` only shows `world_id` when we route a command through the world agent (non-PTY/PTY). Interactive REPL commands and other local exec paths still run in the world but log `world_id: null`, which makes auditing isolation harder.
  - Work: when the shell initializes a world session, capture the active world ID (if any) and populate it in the span metadata even for local/REPL commands. Ensure both shim and shell telemetry include it so every span reflects whether it executed in a world.
  - Considerations:
    - Plumb the ID through `ShellConfig`/`WorldRootSettings` so the logging layer can access it regardless of execution path.
    - Update tests to assert the field is set when worlds are enabled, and note the behavior change in tracing docs so operators know how to interpret the new data.

- **P1 – Capture caging/anchor in spans + replay verbose**
  - Problem: replay verbose output can’t show whether the recorded command was caged or what anchor was used because spans don’t persist that state.
  - Work: persist `caged` (bool) and `anchor_cwd` (string) in the span/replay_context, thread into replay reconstruction, and print in `--replay-verbose` with a warning when the replayed environment can’t honor the recorded caging/anchor.
  - Acceptance: trace entries include the new fields for caged sessions; replay verbose shows caging/anchor and warns on mismatch; docs/tests updated to cover the fields.

- **P1 - TCP bridge for agent (Cross-platform: Linux/macOS/Windows)**
  - Goal: Provide an optional loopback TCP endpoint that bridges to the agent UDS for environments/tools requiring TCP.
  - Current state:
    - **macOS** – already implemented via `vsock-proxy` (Lima forwarder exposes `127.0.0.1:17788`) with SSH-UDS fallback.
    - **Windows/WSL** – implemented by `substrate-forwarder` when launched with `--tcp-bridge` (scripts default to `127.0.0.1:17788`).
    - **Linux** – missing: needs optional systemd socket/service (e.g., `socat`) to expose `127.0.0.1:17788 → /run/substrate.sock`, plus installer/world-enable wiring and docs.
  - Security: Loopback only; document that it is a fallback, not a replacement (no TLS/external exposure).
  - Acceptance (Linux parity):
    - `curl -sSf http://127.0.0.1:17788/v1/capabilities` works whenever the bridge is enabled.
    - Shell auto-selects TCP only when UDS is unavailable and the bridge is active, matching macOS/Windows behavior.

- **P1.5 - JSON Mode Plan**
  -  Plan: '/home/spenser/__Active_code/substrate/docs/project_management/next/json-mode/json_mode_plan.md'

- **P2 – World deps install UX cleanup**
  - `substrate world deps install` only works for tools defined in the world-deps inventory (canonical manager list in `config/manager_hooks.yaml`, plus the layered overrides under `world-deps.yaml` / `world-deps.local.yaml`). Users who try to install language-level packages (pip/npm) or tools missing from the inventory get confusing errors, especially on dev installs where `$HOME` is read-only inside the world.
  - Improvements:
    - Document exactly what the manifest covers and how to install unsupported packages safely (e.g., virtualenv/pip --target).
    - Consider adding backends for common package managers so commands like `substrate world deps install pip:package` can proxy installations into the writable anchor.
    - Ensure dev-install paths and env detection work inside the world so the helper can locate binaries/scripts.
  - Goal: make `world deps` a predictable way to provision world tooling or gracefully direct users to the correct workflow when an item isn’t supported yet.
  - Parity gap: health/doctor currently reports `parity: "unknown"` for optional managers (nvm/pyenv/bun/direnv/asdf/etc.) because world-side detectors are missing. Add world detection (or consume doctor world data) so parity can report synced vs. attention-required when the agent is healthy.

- **P2 – Session listing/resume UX**
  - Pain: Each `substrate` invocation mints a fresh session ID recorded in `trace.jsonl`. Users who want to resume a previous REPL session or correlate spans currently have to parse that file manually or export `SHIM_SESSION_ID` themselves.
  - Idea:
    - `substrate sessions list` (or similar) to show recent session IDs, timestamps, and last known working directories by parsing `trace.jsonl`.
    - `substrate sessions resume <session_id>` (or a `--resume` flag) that sets `SHIM_SESSION_ID` before launching the shell so new spans append to that session. Replay remains per-span, but this improves DX for continuing a workflow.
  - Considerations:
    - The trace log can grow; listing needs to be efficient (maybe keep an index or show the last N sessions).
    - Handle missing/invalid directories gracefully when resuming and document what context is restored (only tracing metadata, not the world state).
    - Update docs to explain session IDs vs. span IDs and the new resume workflow.

- **P2 – World-disabled UX (no-world installs)**
  - Pain: When Substrate is installed with `--no-world`, commands still probe systemd/agent sockets and emit warnings (inactive socket, agent replay unavailable, cgroup/netns/overlay fallbacks) even though the world backend is intentionally disabled.
  - Improvements:
    - `substrate world doctor` should short-circuit when world is disabled and report a clear “world disabled/not provisioned” status (JSON/text) without probing systemd/socket.
    - Replay should detect disabled/missing world early and emit a single “world disabled/no agent installed; staying host-only” message instead of attempting world creation and logging cgroup/netns/overlay warnings.
    - `world deps` and related subcommands should surface that the world is disabled by install/config/env before implying a broken agent.
  - Goal: make host-only installs quiet and explicit, avoiding misleading “agent unavailable” warnings when the world was intentionally skipped.

- **P3 – Interactive configuration commands**
  - Add shell built-ins/commands (`:config`, `:profile load`, `:world status`,
    `:shims status`, etc.) to view and adjust settings without restarting.
  - Surface doctor/shim status inside the REPL so users don’t have to exit.

- **P3 – Interactive install/setup walkthrough**
  - New users currently have to manually edit configs or run individual commands to understand world/caged/anchor/policy settings. An interactive setup wizard (e.g., `substrate setup`) could walk them through enabling/disabling worlds, picking anchor modes/paths, choosing caged vs uncaged behavior, configuring manager snippets, and reviewing policy defaults.
  - Considerations: support non-interactive modes (`--defaults`, `--profile`), validate paths, respect existing configs (idempotent), and clearly document the choices made.

- **P4 – macOS bootstrap automation**
  - Build a signed installer script (or helper binary) that validates macOS version/virtualization support, installs required Homebrew packages (`lima`, `jq`, `openssh`, `coreutils`, `gnused`, `gnu-tar`, `gettext`), provisions Lima, deploys `substrate-world-agent`, and runs doctor checks automatically.
  - Acceptance: Fresh macOS host can run a single installer command and end up with the Lima VM, agent, and smoke test ready without manual steps.

- **P4 – macOS installer dependency automation**
  - Enhance `scripts/substrate/install-substrate.sh` to auto-install required macOS tools (e.g., `envsubst` via Homebrew `gettext`) when missing, falling back to clear guidance if no supported package manager is detected.
  - Acceptance: Fresh macOS host without gettext can run the installer end-to-end without manual prerequisite setup. 

- **P4 – Cleanup skips for already-removed worlds**
  - `substrate world cleanup` still enumerates every historical `wld_*` entry and prints “manual” nft/cgroup commands even when those resources no longer exist (the helper treats ENOENT as failure). Users who already deleted the leftovers get spammed with scary warnings on each run.
  - Improvements:
    - Probe nft tables, netns entries, and cgroup directories before attempting removal; skip entries that are already absent.
    - Treat ENOENT from `nft delete`/`rm -rf` as success so the CLI reports “all clear” when nothing remains.
    - Summarize results with a clean “nothing to clean” message instead of reprinting stale IDs.
  - Acceptance: running `substrate world cleanup` on a host with no lingering worlds produces a success summary with no manual commands; lingering resources still result in actionable instructions.

- **P5 – Doctor/health output UX**
  - The current `substrate world doctor`, `substrate shim doctor`, and `substrate health` outputs are verbose but not well structured: actionable issues are buried in large sections (e.g., listing every missing manager even when the host never had them), and the ordering makes it hard to see what needs attention.
  - Improvements:
    - Reorganize sections so failures/warnings surface first, followed by informational details.
    - Clearly separate “host missing manager” vs. “host has manager, world missing it”.
    - Consider a concise summary view with optional `--verbose` output for the full details.
  - Goal: make the doctor/health commands feel like a coherent report where users can quickly identify what needs fixing. 

- **P5 – Consolidate world enable toggle under `[world]`**
  - Today the on/off switch for world isolation lives under `[install]` as `install.world_enabled`. That placement dates back to the installer metadata phase, but it creates a split experience: all other runtime knobs (`anchor_mode`, `caged`, etc.) live under `[world]` while the most important one sits elsewhere. As we expand world configuration, the ergonomics would be cleaner if users could manage every world option via `world.*` keys.
  - Proposed direction: introduce `world.enable` (boolean) as the canonical key, treat `install.world_enabled` as a compatibility alias, and update docs/installers to write/read the new location. That aligns the schema semantically (all world behavior in one table) and makes `substrate config show/set` more intuitive (`world.enable=false` instead of `install.world_enabled=false`).
  - Migration considerations: loaders should prefer `world.enable` when present but still honor the old key, `substrate config set` needs to write both until we formally deprecate the installer key, and docs/tooling (install scripts, doctor output) must call out the new name plus the legacy alias so existing installs don’t break.

- **Later – Document `--uncaged` as a diagnostics-only escape when worlds are enabled**
  - Users occasionally expect `--uncaged` to grant broader filesystem access inside the world backend, but in reality it just removes the anchor guard; the process remains confined to the world’s overlay. We should update docs (`docs/CONFIGURATION.md`, `docs/USAGE.md`, `docs/WORLD.md`) and CLI help to clearly state that `--uncaged` inside a world is intended for Substrate troubleshooting (inspecting overlay internals, diff dirs, etc.) and shouldn’t be used for normal workflows.
  - Acceptance: CLI help text and documentation call out that uncaging a world session doesn’t break isolation and is mainly for debugging; examples show `--uncaged` as a dev tool rather than a standard option.

- **Later – Command hook engine**
  - Implement the hook system described in `docs/project_management/future/COMMAND_HOOKS_IMPLEMENTATION_PLAN.md`
    (hook definitions, matcher/condition/action pipeline, shim & shell integration).
  - Provide `~/.substrate/hooks.yaml` scaffolding plus CLI commands to list/enable/disable hooks.  

- **Later – Investigate scoped/named worlds (project/user/global)**
  - Idea: today every world session anchors to the launch directory and gets its own ephemeral overlay (`world.anchor_mode=project`). Consider adding explicit scopes such as “user world” or “global world” so multiple commands can share a long-lived overlay (faster warm-up, persistent tooling state) while still staying isolated from the host.
  - Considerations:
    - **Isolation & policy:** reusing a user/global world mixes state between repos. We’d need opt-in plus policy controls to avoid leaking data between unrelated projects.
    - **Lifecycle:** define commands to create/reset/destroy scoped worlds (`substrate world reset --scope=user`?) and document how cleanup works.
    - **CLI/Config UI:** extend config/flags (e.g., `world.scope = project|user|global`) and expose how this interacts with existing `anchor_mode`/`anchor_path`. For experimentation we could leverage `world.anchor_mode=custom` pointing at a shared directory to simulate the behavior.
    - **Backend changes:** the Linux backend would need to cache overlays keyed by scope instead of always generating new `wld_*` directories.
  - Not a priority now, but worth capturing so we can explore the UX/security balance later; any prototype should highlight how to emulate it today via `world.anchor_mode=custom` before adding official scopes.

- **Later – Shell env regression tests should force `SUBSTRATE_HOME`**
  - Pain: the `shell_env_*` integration tests only override `HOME`, so a user who has `SUBSTRATE_HOME` exported (e.g., after running the dev installer) leaks their real `~/.substrate` into the test harness. That causes false failures (`PATH` not shimmed, overlay snippets missing) whenever host metadata differs from the fixture expectations.
  - Fix: update `ShellEnvFixture`/`substrate_command_for_home` to set `SUBSTRATE_HOME` to the temporary home it creates (and pre-create config directories). The tests then stay hermetic even if the developer’s environment has custom prefixes.
  - Scope: treat as maintenance/reliability work—no product changes, just test harness hardening. Add a quick note in `DEVELOPMENT.md` once fixed so contributors know the tests are self-contained.

- **Later – Graph intelligence backend**
  - Finish the Kuzu-backed implementation outlined in `docs/project_management/future/PHASE_4_5_ADVANCED_FEATURES_PLAN.md`:
    ingestion pipeline, schema, query interface, and CLI.
  - Replace the current mock-only `substrate-graph` crate with a real backend.

- **Later – CLI help/flag audit**
  - Several subcommands inherit global flags that don’t make sense in context (e.g., `substrate world cleanup --help` shows `--world/--no-world` despite being world-only). Audit command help/flag surfaces, hide or override irrelevant globals, and ensure summaries/examples reflect intended usage.
  - Scope: review top-level/global flags vs per-command applicability, adjust clap definitions to suppress noise, and refresh docs/USAGE snippets with cleaned help output.
  - Acceptance: per-command help only shows relevant options; cleanup/help examples confirm noisy flags are removed or clearly explained.



## Hardening / Quality
- Document backend divergence
  - Clarify why shell uses world-agent and replay uses LinuxLocalBackend; outline future convergence plan.
- Heavy isolation wiring (optional)
  - Provide a mode to invoke `LinuxIsolation::apply()` for non-PTY when capabilities allow; keep current lightweight/PTy approach as default.
  - Acceptance: gated by capability checks; clear degrade messages.
- Linux world fs last-resort fallback: copy-diff execution
  - If kernel overlayfs is unhealthy and `fuse-overlayfs` is unavailable, allow a copy-diff strategy to keep world usable for basic workflows.
  - Safety requirement: copy-diff MUST preserve the project bind-mount enforcement (private mount namespace bind of the work tree onto the host project path) to prevent absolute-path escapes.
  - Observability: record selected strategy + fallback reason in trace and surface in `substrate world doctor --json`.
- Docs consistency
  - Document non-PTY agent auto-start behavior and fallback; update REPLAY/WORLD runbooks accordingly.

## Later
- Policy/broker expansion
  - Richer allowlist/egress policy plumbing into world sessions.


## Later - Windows Transport — Cross‑Cutting Backlog
User‑Scoped Named Pipe Default

Context: We standardize on a single host pipe
`\\.\\pipe\\substrate-agent`. On shared or multi‑session Windows systems this
name can be owned by a stray forwarder, an IT‑deployed service, or another
user session. While the “single‑instance guard + friendly error” mitigates
confusion at startup, adopting a user‑scoped default eliminates cross‑user
collisions by design.

Goal: Make the default forwarder pipe name user‑scoped (for example,
`\\.\\pipe\\substrate-agent-<SID>`), while preserving backward compatibility
and explicit overrides.

Design details
- Default name schema: `\\.\\pipe\\substrate-agent-<SID>` where `<SID>` is the
  current user’s Windows SID; fallback to `<USERNAME>` if SID retrieval fails.
  Keep `\\.\\pipe\\substrate-agent` as a documented legacy alias.
- Backward compatibility: If `SUBSTRATE_AGENT_PIPE` (or equivalent) is set to
  the legacy name, honor it. Doctor/warm should detect conflicts and print
  remediation.
- Display/telemetry: Sanitize endpoints in logs/telemetry (avoid leaking SID);
  use `substrate-agent-<user>` in human‑readable fields and keep the full path
  in debug logs only.
- Security: Maintain the current SDDL (SY/BA/IU GA) and
  `reject_remote_clients(true)`. Document that user‑scoped names reduce
  accidental cross‑session access but do not replace ACLs.

Code changes required (touch points)
- `crates/forwarder/src/main.rs`
  - Default CLI `--pipe` value to the user‑scoped name computed at runtime.
  - Emit both the effective path and a redacted display endpoint in logs.
- `crates/host-proxy/src/lib.rs`
  - Change `DEFAULT_AGENT_PIPE` to resolve at runtime via a helper that
    computes the user‑scoped name when no explicit config is supplied.
  - Update serde/display to accept both legacy and user‑scoped forms.
- `crates/world-windows-wsl/src/lib.rs`
  - Ensure `build_agent_client()` resolves the same default and exposes the
    chosen endpoint for telemetry.
- `crates/agent-api-client` (tests)
  - Update unit tests that assert the hard‑coded legacy pipe to accept the
    dynamic, user‑scoped default.
- `crates/shell` (tests/telemetry)
  - Adjust Windows tests that check `transport.endpoint` to tolerate the new
    default (or inject override during tests).

Scripts and docs
- `scripts/windows/wsl-warm.ps1`, `wsl-doctor.ps1`, `wsl-stop.ps1`
  - Default to the user‑scoped pipe; support `-PipePath` override and surface
    both the effective path and display value.
- Documentation updates
  - `docs/dev/windows_host_transport_plan.md`: call out the user‑scoped default
    and rationale.
  - `docs/dev/wsl_world_setup.md`: show the new default in examples and explain
    overrides.
  - `docs/dev/wsl_world_troubleshooting.md`: update entries for conflicts and
    how to identify the owning process (handle.exe), plus how to opt back to
    the legacy name for compatibility.
  - `docs/dev/windows_transport_external_overview.md`: update architecture
    diagrams and narrative to mention user‑scoped default.

Migration and compatibility strategy
- Soft roll‑out: keep the legacy name documented; scripts accept both and print
  a deprecation‑style hint when the legacy name is detected.
- Env/config knobs: add `SUBSTRATE_AGENT_PIPE` or forwarder TOML `pipe_path` to
  force legacy behavior for environments that depend on it.
- CI/tests: set `SUBSTRATE_AGENT_PIPE=\\.\\pipe\\substrate-agent-test` in CI to
  avoid SID dependencies in headless runners.

Acceptance criteria
- Fresh Windows setup uses the user‑scoped default end‑to‑end (forwarder,
  doctor/warm, shell, host‑proxy).
- Legacy name still works when explicitly configured; friendly warnings appear
  with migration hints.
- No collisions between different signed‑in users or sessions by default.
- Updated docs are markdownlint clean; evidence log captures a successful warm
  run using the user‑scoped default.

Risks / considerations
- Third‑party automations hard‑coding the legacy name will need overrides;
  mitigate via clear warnings and a deprecation window.
- Some admin workflows may prefer a machine‑wide pipe; document how to force
  the legacy name alongside hardening advice (service account, SDDL, audit).
- Backlog – Stateful replay & session branching
  - Today replay simply re-runs a single span’s command in the current working tree; it doesn’t restore prior state or let you walk an entire session. To make replay genuinely useful for “time travel” debugging, we need:
    - The ability to list/replay a contiguous range of spans (e.g., “replay spans 3–7 of session X in order”).
    - Snapshot/rollback support so you can restore the filesystem/environment to how it looked at a given span before replaying or branching.
    - Branching semantics (fork a “what-if” timeline from a session and track it as a branch of the original session).
  - Considerations:
    - Requires capturing enough state (e.g., snapshots, artifacts, or overlays) to restore the workspace reliably.
    - Needs a UX for selecting span ranges, stepping through spans, and labeling branches.
    - Coordination with world backend: snapshots might leverage overlayfs layers or copy-diff artifacts.



## DONE -- IMPLEMENTED

- ~~Top Priority – Global configuration UX~~ **(Done)**
  - Implementation: `substrate config init` scaffolds `~/.substrate/config.yaml`, `config show` renders YAML/JSON with redaction hooks, and `config set` applies multi-key updates atomically with schema validation.
  - Docs/installer output highlight the new subcommands across Linux/macOS/Windows, and the precedence stack (flags → directory config → global config → env) remains unchanged.

- ~~Auto-start world-agent on shell startup (Linux)~~ **(Done)**
  - Implementation: `run_shell()` now initializes the Linux backend, flips
    `SUBSTRATE_WORLD=enabled`, sets `SUBSTRATE_WORLD_ID`, and uses
    `ensure_session()` before handling commands (`crates/shell/src/lib.rs:1576-1620`).
  - Notes: macOS and Windows paths share the same default-on behavior; the Linux
    helper still attempts to spawn `world-agent` if `/run/substrate.sock` is
    stale (`crates/shell/src/lib.rs:3680-3687`).

- ~~Non-PTY agent auto-start parity (shell)~~ **(Done)**
  - Implementation: Non-PTY routing now calls `ensure_world_agent_ready()` when
    worlds are enabled (Linux HTTP path) and records transport metadata for
    macOS/Windows agent calls (`crates/shell/src/lib.rs:3680-3703`, `3560-3663`).
  - Result: The shell only falls back to host execution after a single warning
    when the agent cannot be reached; routine runs stay in-world by default.

- ~~Top Priority – Fix REPL busy-spin / async agent output~~  **(Done)**
  - Resolve the Reedline/crossterm busy loop that pegs a CPU core when idle (gate on TTY,
    introduce backoff, or adopt the async event loop from the Phase 4 concurrent output design).
  - See `docs/project_management/future/PHASE_4_CONCURRENT_OUTPUT_DESIGN.md` for context and remediation notes.
  - Implement the async agent output path so events can stream without prompt corruption.

- ~~macOS binary distribution~~ **(Done)**
  - Publish prebuilt `substrate` and `substrate-world-agent` artifacts (e.g., GitHub releases or Homebrew formula) so operators do not need a Rust toolchain to onboard.
  - Acceptance: Installer/bootstrap script can download versioned binaries; manual instructions reference the published artifacts instead of local builds.

- ~~macOS backend enablement (Lima)~~ — **DONE (2025-09)**
  - Implemented `world-mac-lima` agent calls via `agent-api-client` with VSock/SSH fallbacks.
  - Shell now routes macOS commands through Lima, ensures VM/forwarding, and mirrors Linux telemetry.
  - Acceptance met: `scripts/mac/smoke.sh` validates non-PTY, PTY, and replay; docs refreshed (`docs/WORLD.md`, `docs/dev/mac_world_setup.md`, `docs/INSTALLATION.md`).
