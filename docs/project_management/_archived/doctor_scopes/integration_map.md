# Integration Map — doctor_scopes (DS0)

Goal:
- Provide an unambiguous “what touches what” map so triad agents know exactly where to work and what to validate.

## Contract surfaces

- CLI:
  - Add: `substrate host doctor [--json]`
  - Change: `substrate world doctor [--json]` semantics and JSON contract
  - Spec: `docs/project_management/_archived/doctor_scopes/DS0-spec.md`
- Agent API:
  - Add: `GET /v1/doctor/world`
  - Types: add `WorldDoctorReportV1` in `crates/agent-api-types`

## Primary code touch points (expected)

- CLI parsing / routing:
  - `crates/shell/src/execution/cli.rs` (new `Host` subcommand + `doctor` verb)
  - `crates/shell/src/execution/platform/mod.rs` (route host/world doctor entry points; compute effective config for subcommands)
- Host doctor implementations:
  - Linux: refactor existing host probes out of `crates/shell/src/execution/platform/linux.rs`
  - macOS: refactor existing macOS doctor out of `crates/shell/src/execution/platform/macos.rs`
  - Windows: new explicit unsupported report (remove “false green” behavior) in `crates/shell/src/execution/platform/windows.rs`
- World doctor (host-side):
  - Use an agent client call to `GET /v1/doctor/world` (no spawning/starting)
  - Prefer reusing existing transport selection logic used for agent execution (but without `ensure_ready` side effects)
- World-agent:
  - Route registration: `crates/world-agent/src/lib.rs` (add route)
  - Handler implementation: `crates/world-agent/src/handlers.rs` (or existing handler module used by router)
  - Probe logic: reuse existing world-side primitives (Landlock detection + overlay enumeration probe) already present in `crates/world`
- Agent API client/types:
  - `crates/agent-api-types/src/lib.rs` (add `WorldDoctorReportV1` structs and serde tests)
  - `crates/agent-api-client/src/lib.rs` (add a method `world_doctor()` returning `WorldDoctorReportV1`)

## Internal consumers that must be updated in DS0

- Health/shim snapshots:
  - `crates/shell/src/builtins/shim_doctor/report.rs` (world doctor snapshot parsing)
- World verify:
  - `crates/shell/src/builtins/world_verify.rs` (doctor parsing for `world_socket` and `ok`)
- Docs:
  - `docs/COMMANDS.md`, `docs/WORLD.md`, `docs/USAGE.md`, `docs/INSTALLATION.md`, `docs/ISOLATION_SUPPORT_MATRIX.md`

## Validation gates (required)

- Local (integration core + final):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - Relevant tests (at least: shell tests that parse doctor JSON; agent-api-types tests)
  - `make integ-checks`
- CI parity:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/doctor-scopes" CI_REMOTE=origin CI_CLEANUP=1`
- Behavior smoke:
  - Linux: `docs/project_management/_archived/doctor_scopes/smoke/linux-smoke.sh` via `make feature-smoke ... PLATFORM=linux`
  - macOS: `docs/project_management/_archived/doctor_scopes/smoke/macos-smoke.sh` via `make feature-smoke ... PLATFORM=macos`

## Smoke ↔ manual testing parity contract

- Manual testing is defined in `docs/project_management/_archived/doctor_scopes/manual_testing_playbook.md`.
- Smoke scripts must:
  - run the same CLI commands as the manual playbook (minimal subset), and
  - assert exit codes + required JSON keys/values (not just “command ran”).
