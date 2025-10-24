# Documentation Audit – February 2025

This report captures discrepancies, dated guidance, and recommended edits across the `docs/` tree (excluding `project_management/`). Each section lists the document, the issue, specific locations, and concrete remediation steps. Line references use 1-based numbering from the current repository state.

---

## ARCHITECTURE.md

- **Shim module layout (lines 49–58)**
  - *Issue*: References `substrate-shim/main.rs` as the binary entry point.
  - *Reality*: The shim binary lives in `src/shim_main.rs:1-24` at the repository root; `crates/shim` exposes library logic only.
  - *Action*: Update the module diagram to drop `main.rs` from the crate tree and note the actual binary entry path.

- **Version tracking (line 41)**
  - *Issue*: States the shim embeds `env!("CARGO_PKG_VERSION")`.
  - *Reality*: The shim exposes `SHIM_VERSION` (`crates/shim/src/lib.rs:201-224`), while deployment continues to compare against `CARGO_PKG_VERSION`.
  - *Action*: Document both values: `SHIM_VERSION` for runtime reporting and `CARGO_PKG_VERSION` for deployment validation.

- **Built-in commands (lines 109–113)**
  - *Issue*: Lists `exit/quit` as built-ins executed through `execute_builtin()`.
  - *Reality*: Built-ins are handled by `handle_builtin()` (`crates/shell/src/lib.rs:4230-4294`) and only cover `cd`, `pwd`, `export`, `unset`. The REPL closes on `exit/quit` without calling the builtin handler (`crates/shell/src/lib.rs:1739-1752`).
  - *Action*: Replace the function name in the text, adjust the built-in list, and clarify that `exit/quit` are handled by the interactive loop only.

- **Logging pipeline (lines 60–73)**
  - *Issue*: Mentions `log_command_start()`/`log_command_complete()` helpers.
  - *Reality*: Shell logging funnels through `log_command_event()` (`crates/shell/src/lib.rs:4667-4699`) per event type.
  - *Action*: Update terminology and reference the current helper.

- **Binary fingerprinting (line 281)**
  - *Issue*: Attributes hashing to `resolver.rs`.
  - *Reality*: Fingerprints are produced by `logger::get_shim_fingerprint()` (`crates/shim/src/logger.rs:244-287`).
  - *Action*: Correct the implementation reference.

- **MSRV (lines 315–316)**
  - *Issue*: States 1.74+.
  - *Reality*: Workspace `Cargo.toml` enforces `rust-version = "1.89"`.
  - *Action*: Update MSRV guidance.

- **Future work reference (line 361)**
  - *Issue*: Points to `OLD_ARCHITECTURE.md`, which no longer exists.
  - *Action*: Remove or replace with an extant planning doc.

---

## BACKLOG.md

- **Auto-start world-agent on shell startup (lines 8–17)**
  - *Issue*: Listed as near-term work.
  - *Reality*: `run_shell` already enables/subscribes to worlds on Linux, macOS (Lima), and Windows (WSL) (`crates/shell/src/lib.rs:1516-1620`).
  - *Action*: Mark as completed or prune.

- **Non-PTY agent auto-start parity (lines 19–25)**
  - *Issue*: Treated as future work.
  - *Reality*: Non-PTY path now calls `ensure_world_agent_ready()` before agent execution (`crates/shell/src/lib.rs:3680-3703`).
  - *Action*: Remove or move to a “done” log.

---

## BROKER.md

- **Hot-reload capability (lines 12–13/148)**
  - *Issue*: Documented as active functionality.
  - *Reality*: `watcher.rs` is compiled only under tests or the `policy-watcher` feature (`crates/broker/src/watcher.rs:1`). No production path enables it by default.
  - *Action*: Qualify as optional/disabled by default; link to enabling instructions if desired.

- **Interactive approvals (line 12)**
  - *Issue*: Presented as always-on.
  - *Reality*: Approval prompts only occur when `observe_only` is false (`crates/broker/src/lib.rs:112-130`), but neither the shell nor shim call `substrate_broker::init` to switch modes.
  - *Action*: Note that approvals require explicit enforcement initialization and aren’t exercised in the default CLI flow today.

---

## CONFIGURATION.md

- **`SHIM_LOG_OPTS` examples (lines 71, 128)**
  - *Issue*: Demonstrates combined value `raw,resolve`.
  - *Reality*: Code checks for exact `"raw"` or `"resolve"` (`crates/common/src/lib.rs:39-58`; `crates/shim/src/logger.rs:136-187`)—comma-separated values have no effect.
  - *Action*: Provide individual examples and clarify that combinations aren’t parsed.

---

## cross-platform/mac_world_setup.md

- **Agent build step (lines 82–104)**
  - *Issue*: Instructs building `world-agent` on macOS and copying the binary into Lima.
  - *Reality*: `cargo build -p world-agent` on macOS produces a Mach-O binary that fails in the Linux VM. The Lima service expects a Linux binary.
  - *Action*: Update instructions to either build inside the VM (`limactl shell substrate cargo build …`) or cross-compile for Linux before copying.

---

## DEVELOPMENT.md

- **MSRV (line 7)**
  - Align with 1.89 as noted above.

- **Built-in guidance (line 197)**
  - Replace `execute_builtin()` with `handle_builtin()` to match the code path (`crates/shell/src/lib.rs:4230-4294`).

- **`SHIM_LOG_OPTS` advice (line 242)**
  - Remove `raw,resolve` example; mirror the correction in CONFIGURATION.md.

---

## TRACE.md

- **Lazy initialization claim (line 163)**
  - *Issue*: States trace initializes only when `SUBSTRATE_WORLD=enabled`.
  - *Reality*: Both shell and shim call `init_trace(None)` unconditionally (`crates/shell/src/lib.rs:1516-1522`; `crates/shim/src/logger.rs:97-103`).
  - *Action*: Adjust language to describe current behavior or outline future plans if reintroducing lazy init.

---

## USAGE.md

- **Built-in list (lines 37–44)**
  - Remove `exit/quit` from the builtin list; clarify they only work in the interactive loop.

- **`SHIM_LOG_OPTS` example (line 180)**
  - Apply the `raw` vs `resolve` correction.

---

## VISION.md

- **Policy YAML schema (lines 64–75)**
  - Replace nested keys (`commands.allowed`, `fs.write`) with the actual struct fields (`cmd_allowed`, `fs_write`, etc.) from `crates/broker/src/policy.rs`.

- **Graph CLI example (line 84)**
  - `substrate graph what-wrote` is not implemented; supported actions are `ingest`, `status`, and `what-changed` (`crates/shell/src/lib.rs:282-295`). Update the example accordingly.

---

## cross-platform/wsl_world_troubleshooting.md

- **Duplicate index entry (lines 19–20/244)**
  - `T-010 Forwarder log stale` appears twice in the index.
  - Remove the duplicate bullet.

---

## TELEMETRY.md

- **Performance claims (lines 29, 165–166)**
  - Metrics (<10 ms per syscall, ~1 ms init) aren’t backed by benchmarks in the repository.
  - Options: provide measured data or soften the language to reflect aspirational targets.

---

## INSTALLATION.md

- **Windows host automation note (lines 12–18)**
  - Text still says PowerShell automation is “forthcoming” and recommends installing from within WSL.
  - `scripts/windows/install-substrate.ps1` already exists and is documented later in the file (lines 83–116).
  - Update the supported-platform summary to reflect the available installer.

---

## REPLAY.md

- **macOS isolation (line 50)**
  - The doc claims non-Linux platforms fall back to direct execution.
  - Since `world_backend_factory::factory()` returns the Lima backend on macOS (`crates/replay/src/replay.rs:45-108`), replay now uses world isolation there as well.
  - Amend the platform note to clarify: Linux and macOS use isolation; other platforms fall back to direct execution without `fs_diff`.

---

## Additional Notes

- **Global `SHIM_LOG_OPTS` guidance**: Ensure any other references (README, docs outside the audited list) avoid `raw,resolve` phrasing.
- **Completed backlog items**: Consider adding a “Done” appendix or moving delivered backlog entries to release notes to keep the backlog focused on future work.
- **Performance numbers**: Wherever specific latency or throughput numbers are quoted (e.g., Architecture, Telemetry), confirm they match current measurements or restate them as goals.

