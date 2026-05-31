# Repository Guidelines

## Orientation
Substrate is the secure execution layer that sits between AI agents and a developer workstation. The top-level binaries in `src/` (`substrate`, `substrate-shim`) orchestrate command capture, policy evaluation, and execution inside isolated "worlds" so that every action is observable, replayable, and compliant with local policy.

## Workspace Layout & Entry Points
- `src/`: thin CLI wrappers; real logic lives in crates.
- `crates/shell`: interactive REPL, PTY plumbing, shim deployment, and world orchestration.
- `crates/shim`: PATH interception, depth tracking (`SHIM_DEPTH`), and structured logging of every spawned process.
- `crates/common`: shared log schema, redaction helpers, filesystem diff types.
- `crates/world*`: platform-specific backends. `world` uses namespaces/cgroups/nftables on Linux; `world-mac-lima` boots a Lima VM and tunnels to `world-service`; `world-windows-wsl` warms a WSL distro and brokers named-pipe/TCP access (functional but experimental).
- `crates/world-service`: the in-world daemon exposing REST (`/v1/execute`) and WebSocket (`/v1/stream`) APIs with fs diff collection.
- `crates/agent-api-*`: shared request/response models, async client, and host proxy helpers for third-party agents.
- `crates/broker`, `forwarder`, `host-proxy`, `telemetry-lib`, `replay`, `substrate-graph`: policy decisions, transport glue, telemetry pipelines, replay tooling, and optional graph analytics.
- `docs/`: deep dives (`WORLD.md`, `TRACE.md`, `REPLAY.md`, `TELEMETRY.md`, `CONFIGURATION.md`, `cross-platform/…`).
- `scripts/`: provisioning, doctoring, smoke tests (`linux/world-provision.sh`, `mac/lima-warm.sh`, `mac/smoke.sh`, `windows/wsl-warm.ps1`, `windows/wsl-smoke.ps1`).
- `reedline` (crates.io dependency): upstream line editor used by the async/sync REPL workers.

## Execution Architecture
1. Shell loads policy via the broker (`substrate_broker::evaluate`), ensures shims are deployed (`ShimDeployer`), and initializes tracing (`substrate_trace::init_trace`).
2. The shim intercepts PATH lookups, writes structured execution events (via `logger::log_execution`), performs fast policy checks, and forwards invocation metadata to trace spans.
3. The shell consults the world backend selected by `world-backend-factory`, issuing `/v1/execute` for non-PTY runs or upgrading to `/v1/stream` for interactive workloads. On Linux/macOS this depends on the `substrate-world-service` service (systemd on Linux, Lima guest on macOS); provisioning scripts install and start that service.
4. `world-service` executes inside a per-session world: Linux uses cgroup v2 + netns + overlayfs; macOS proxies into a Lima VM via VSock/SSH; Windows calls into WSL using named pipes or a TCP forwarder.
5. Results, filesystem diffs, and policy decisions are recorded by `crates/trace`, producing JSONL spans in `~/.substrate/trace.jsonl` (or `SHIM_TRACE_LOG`). Replay (`crates/replay`) re-runs spans through the same world backend (Linux replays require CAP_SYS_ADMIN for namespaces/overlay; use `--no-world` / `SUBSTRATE_REPLAY_USE_WORLD=disabled` only if you want to opt out).

## Local Environment & Core Tools
- Rust 1.89+ (MSRV enforced in `Cargo.toml`). Install `rustup component add rustfmt clippy`.
- Platform requirements detailed in `docs/INSTALLATION.md` and `docs/WORLD.md` (e.g., Lima + Virtualization.framework on macOS, WSL + PowerShell 7 on Windows, overlayfs/nft on Linux).
- Optional: Kuzu database for `substrate-graph` (skip unless working on graph analytics).

## Build & Test Workflow
```bash
cargo build --workspace                     # incremental debug builds
cargo build --release                       # optimized binaries
cargo fmt --all -- --check                  # formatting gate
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture       # run full suite with output
cargo bench                                 # exercise hotspots when touching performance-sensitive crates
```
- Avoid `--all-features` at the workspace level; enable graph features explicitly via the graph crate manifest (e.g., `cargo build --manifest-path crates/substrate-graph/Cargo.toml --features kuzu-dylib`).
- Regenerate shims for manual testing with `substrate --shim-deploy`; inspect with `substrate --shim-status`.

## Coding Standards & Patterns
- Four-space indentation, Rust 2021 edition, snake_case functions, SCREAMING_SNAKE_CASE constants, UpperCamelCase types.
- Library code should return `Result<T, anyhow::Error>` and use `anyhow::Context` for rich errors; avoid panics in shared crates.
- Send all structured logs through `crates/common/src/log_schema.rs`. Apply redaction helpers (`redact_sensitive`) instead of ad-hoc filtering.
- Environment toggles use `SUBSTRATE_*` (shell), `SHIM_*` (shim), and `WORLD_*` (backends). Document new toggles in `docs/CONFIGURATION.md`.
- Maintain platform guards: macOS/Windows specific logic belongs behind `#[cfg]` gates (see `crates/shell/src/platform_world.rs`).
- Public APIs need rustdoc examples that pass `cargo test --doc`.

## Observability & Telemetry
- `crates/trace` is the canonical span writer; it manages file rotation (`TRACE_LOG_MAX_MB`, `TRACE_LOG_KEEP`) and ensures legacy compatibility fields (`command`).
- `telemetry-lib` augments spans with system metrics; integrate via existing hooks rather than new logging formats.
- Inspect spans via `tail -f ~/.substrate/trace.jsonl | jq '.'`; filter denies with `jq 'select(.policy_decision.action == "deny")'`.
- Replay and diff analysis live in `docs/REPLAY.md` and `docs/TRACE.md`; update those docs when schema changes. See `docs/REPLAY.md` for privileged requirements (`SUBSTRATE_REPLAY_USE_WORLD`) and how to use `--no-world` when isolation isn’t available.

## Platform Worlds & Provisioning
- **Linux**: Run `scripts/linux/world-provision.sh --profile release` to install `world-service` plus the matching `.service`/`.socket` units so systemd manages `/run/substrate.sock`. Dev installs invoke this script automatically (sudo prompts expected). The installers now create the `substrate` group, add the invoking user, and reload the socket units so `/run/substrate.sock` is `root:substrate 0660` just like the provisioning script. After provisioning, run `id -nG "$USER"` to confirm the new membership and `loginctl enable-linger "$USER"` so the socket-activated service is available after logout/reboot. Validate with `systemctl status substrate-world-service.socket`, `systemctl status substrate-world-service.service`, and `substrate world doctor --json | jq '.world_socket'`.
- **macOS**: `scripts/mac/lima-warm.sh` provisions the `substrate` Lima VM, installs dependencies, and ensures forwarding. `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` validate PTY, REST, and replay flows.
- **Windows**: Warm the WSL backend via `pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .)`; verify with `scripts/windows/wsl-smoke.ps1`. The backend prefers named-pipe transport but can fall back to TCP (`SUBSTRATE_FORWARDER_PORT`).
- Always capture doctor/smoke output in PRs touching `world-*`, `forwarder`, or `host-proxy` crates.

## Testing & Validation Expectations
- Unit tests live alongside sources (`#[cfg(test)] mod tests`); integration suites sit in `crates/*/tests/` (e.g., `crates/shell/tests/integration.rs`, `crates/replay/tests/`). Use `test_<behavior>` naming.
- Run `cargo test -p world-service`, `cargo test -p substrate-shim`, and targeted `cargo test --test shim_deployment` when editing those components.
- When touching shell/world/shim logic, capture `substrate world doctor --json`, `substrate shim doctor --json`, and `substrate health --json` output in the PR.
- Privileged checks (netns, nftables, cgroups) require `docs/HOWTO_PRIVILEGED_TESTS.md`; document platform coverage in PR descriptions.
- Validate shim lifecycle on every OS touched: `substrate --shim-status`, `substrate --shim-deploy`, `substrate --shim-remove`.
- When altering replay or trace schema, run regression tests in `crates/replay` and update fixtures in `docs/TRACE.md`.

## Policy, Approval & Security Guardrails
- The policy broker (`crates/broker`) loads YAML policies and enforces allow/deny/isolation rules. Profiles can be auto-detected per directory via `ProfileDetector`.
- Observe-only mode is the default; toggled enforcement must be accompanied by policy updates and release notes.
- Approval flows cache decisions locally (`ApprovalCache`); never store secrets in approvals or logs.
- Always scrub credentials using the shared redaction helpers and double-check new span fields for sensitive data.

## Contribution Workflow
1. Branch from `main`; keep changes scoped per PR.
2. Follow Conventional Commit prefixes (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`) limited to 72 characters.
3. Before pushing, run formatting, clippy (with `-D warnings`), full workspace tests, and the relevant doctor/smoke/provision scripts for touched platforms.
4. PR description must include summary, motivation, testing commands + platforms, linked issues, and screenshots/logs for UX changes.
5. Squash merge is the default. Coordinate release tagging when shipping binaries or policy schema changes. Include relevant doctor/health output for world/shim changes.

## Reference Docs & Useful Scripts
- `docs/WORLD.md`: isolation model, transport matrix, API reference.
- `docs/TRACE.md`, `docs/TELEMETRY.md`: span schema, telemetry consumers.
- `docs/REPLAY.md`: deterministic replay workflow.
- `docs/CONFIGURATION.md`: environment flags and policy tuning.
- `docs/cross-platform/wsl_world_setup.md`: deep Windows instructions.
- `scripts/check-host-prereqs.sh`, `scripts/check-container-prereqs.sh`: CI parity for host/container readiness.
- `scripts/dev-entrypoint.sh`: reproducible dev container bootstrap.
- `docs/BACKLOG.md`: living backlog for DX improvements/bugs.
- Update these guides when behavior changes; stale docs are treated as regressions.

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **substrate** (26268 symbols, 52926 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/substrate/context` | Codebase overview, check index freshness |
| `gitnexus://repo/substrate/clusters` | All functional areas |
| `gitnexus://repo/substrate/processes` | All execution flows |
| `gitnexus://repo/substrate/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
