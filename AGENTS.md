# Repository Guidelines

## Orientation
Substrate is the secure execution layer that sits between AI agents and a developer workstation. The top-level binaries in `src/` (`substrate`, `substrate-shim`) orchestrate command capture, policy evaluation, and execution inside isolated "worlds" so that every action is observable, replayable, and compliant with local policy.

## Workspace Layout & Entry Points
- `src/`: thin CLI wrappers; real logic lives in crates.
- `crates/shell`: interactive REPL, PTY plumbing, shim deployment, and world orchestration.
- `crates/shim`: PATH interception, depth tracking (`SHIM_DEPTH`), and structured logging of every spawned process.
- `crates/common`: shared log schema, redaction helpers, filesystem diff types.
- `crates/world*`: platform-specific backends. `world` uses namespaces/cgroups/nftables on Linux; `world-mac-lima` boots a Lima VM and tunnels to `world-agent`; `world-windows-wsl` warms a WSL distro and brokers named-pipe/TCP access (functional but experimental).
- `crates/world-agent`: the in-world daemon exposing REST (`/v1/execute`) and WebSocket (`/v1/stream`) APIs with fs diff collection.
- `crates/agent-api-*`: shared request/response models, async client, and host proxy helpers for third-party agents.
- `crates/broker`, `forwarder`, `host-proxy`, `telemetry-lib`, `replay`, `substrate-graph`: policy decisions, transport glue, telemetry pipelines, replay tooling, and optional graph analytics.
- `docs/`: deep dives (`WORLD.md`, `TRACE.md`, `REPLAY.md`, `TELEMETRY.md`, `CONFIGURATION.md`, `cross-platform/…`).
- `scripts/`: provisioning, doctoring, smoke tests (`linux/world-provision.sh`, `mac/lima-warm.sh`, `mac/smoke.sh`, `windows/wsl-warm.ps1`, `windows/wsl-smoke.ps1`).
- `third_party/reedline`: custom fork providing the REPL; patched via `[patch.crates-io]` in `Cargo.toml`.

## Execution Architecture
1. Shell loads policy via the broker (`substrate_broker::evaluate`), ensures shims are deployed (`ShimDeployer`), and initializes tracing (`substrate_trace::init_trace`).
2. The shim intercepts PATH lookups, writes structured execution events (via `logger::log_execution`), performs fast policy checks, and forwards invocation metadata to trace spans.
3. The shell consults the world backend selected by `world-backend-factory`, issuing `/v1/execute` for non-PTY runs or upgrading to `/v1/stream` for interactive workloads.
4. `world-agent` executes inside a per-session world: Linux uses cgroup v2 + netns + overlayfs; macOS proxies into a Lima VM via VSock/SSH; Windows calls into WSL using named pipes or a TCP forwarder.
5. Results, filesystem diffs, and policy decisions are recorded by `crates/trace`, producing JSONL spans in `~/.substrate/trace.jsonl` (or `SHIM_TRACE_LOG`). Optional replay (`crates/replay`) can rebuild environments from those spans.

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
- Avoid `--all-features` at the workspace level; enable graph features explicitly (e.g., `cargo build -p substrate-graph --features kuzu-dylib`).
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
- Replay and diff analysis live in `docs/REPLAY.md` and `docs/TRACE.md`; update those docs when schema changes.

## Platform Worlds & Provisioning
- **Linux**: Run `scripts/linux/world-provision.sh --profile release` to install `world-agent` as a systemd service and create `/run/substrate.sock`. Validate with `systemctl status substrate-world-agent` and `substrate world doctor --json`.
- **macOS**: `scripts/mac/lima-warm.sh` provisions the `substrate` Lima VM, installs dependencies, and ensures forwarding. `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` validate PTY, REST, and replay flows.
- **Windows**: Warm the WSL backend via `pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .)`; verify with `scripts/windows/wsl-smoke.ps1`. The backend prefers named-pipe transport but can fall back to TCP (`SUBSTRATE_FORWARDER_PORT`).
- Always capture doctor/smoke output in PRs touching `world-*`, `forwarder`, or `host-proxy` crates.

## Testing & Validation Expectations
- Unit tests live alongside sources (`#[cfg(test)] mod tests`); integration suites sit in `crates/*/tests/` (e.g., `crates/shell/tests/integration.rs`, `crates/replay/tests/`). Use `test_<behavior>` naming.
- Run `cargo test -p world-agent`, `cargo test -p substrate-shim`, and targeted `cargo test --test shim_deployment` when editing those components.
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
5. Squash merge is the default. Coordinate release tagging when shipping binaries or policy schema changes.

## Reference Docs & Useful Scripts
- `docs/WORLD.md`: isolation model, transport matrix, API reference.
- `docs/TRACE.md`, `docs/TELEMETRY.md`: span schema, telemetry consumers.
- `docs/REPLAY.md`: deterministic replay workflow.
- `docs/CONFIGURATION.md`: environment flags and policy tuning.
- `docs/cross-platform/wsl_world_setup.md`: deep Windows instructions.
- `scripts/check-host-prereqs.sh`, `scripts/check-container-prereqs.sh`: CI parity for host/container readiness.
- `scripts/dev-entrypoint.sh`: reproducible dev container bootstrap.
- Update these guides when behavior changes; stale docs are treated as regressions.
