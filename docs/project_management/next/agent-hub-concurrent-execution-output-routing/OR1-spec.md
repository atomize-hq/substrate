# OR1 spec — REPL output routing during PTY passthrough (buffer/drop + deterministic warnings)

This slice implements the operator-visible routing contract for concurrent structured agent events during PTY passthrough without corrupting TUIs.

## Scope (in-scope; authoritative)

Implement and validate:
1) Strict output routing separation (PTY bytes vs structured agent events)
   - Contract: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/contract.md`
2) PTY passthrough buffering and drop behavior for structured event rendering
3) Deterministic warning records for:
   - PTY structured event drops (`code="pty_structured_event_drops"`)
   - config clamp (`code="config_value_clamped"`)
   - Telemetry source of truth: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/telemetry-spec.md`
4) Config knob plumbing for `repl.max_pty_buffered_lines`
   - Contract: `contract.md`
5) Manual testing playbook + smoke scripts (behavior platforms)
   - `manual_testing_playbook.md`
   - `smoke/*`

## Out of scope (explicit)

- Changing the world-agent PTY transport framing.
- Adding new CLI flags for this feature.
- Adding new policy surfaces (no broker changes).

## Behavior (authoritative)

### Output classes and invariants

Two output classes exist and are handled independently:
1) PTY bytes: forwarded as raw bytes (binary-safe).
2) Structured agent events: rendered via the structured printer and persisted to canonical trace.

Invariant:
- Structured agent output is never injected into PTY bytes during PTY passthrough.

### Structured-event rendering units (“lines”)

For purposes of buffering, each structured agent event is rendered to exactly one line:
- The structured renderer produces a single line per event.
- Any `\\n` in structured event messages is rendered as a literal `\\n` sequence (no multi-line output).

### PTY passthrough buffering and drops

During PTY passthrough:
- PTY bytes are forwarded immediately as bytes.
- Structured agent event lines are not printed live.
- Structured agent event lines are buffered up to the effective cap:
  - `cap = repl.max_pty_buffered_lines` (effective, after precedence and clamping).
- After the buffer reaches `cap`, every additional structured agent event line is dropped from the deferred-print buffer.
- Dropped structured event lines are still persisted to canonical trace as `event_type="agent_event"` records (OR0 foundation).

After PTY passthrough ends and before returning to the prompt:
1) Print all buffered structured agent event lines in the order received.
2) If any structured agent event lines were dropped, emit exactly one warning record and one human-readable warning line:
   - Warning record: `event_type="warning"`, `component="shell"`, `code="pty_structured_event_drops"`
   - Required fields are defined in `telemetry-spec.md`.

### Config: `repl.max_pty_buffered_lines`

Effective value computation:
- Workspace config overrides global config (see `contract.md`).
- No CLI flag exists for this key.

Validation:
- Invalid type/parse: hard error at config boundary with exit code `2`.
- Out-of-range integer: clamp into `[0, 16384]` and emit exactly one warning record:
  - `event_type="warning"`, `component="shell"`, `code="config_value_clamped"` (schema: `telemetry-spec.md`)
  - The emitted warning record does not include secrets and does not change command exit code.

### Platform notes (contract-consistent)

- Linux and macOS: PTY passthrough is supported; buffering/drop + warning semantics are validated by smoke and manual cases.
- Windows: PTY passthrough is not supported; PTY passthrough behavior is not applicable. Envelope + trace persistence behavior remains required and validated.

## Acceptance criteria (testable)

1) Routing invariants and buffering:
- A test suite exists and is green:
  - `cargo test -p shell --test repl_output_routing -- --nocapture` → exit `0`
- The tests assert:
  - structured agent events are not printed during PTY passthrough,
  - buffered structured output is printed after PTY passthrough ends,
  - exactly one `code="pty_structured_event_drops"` warning record is emitted when drops occur,
  - `max_pty_buffered_lines=0` results in zero buffered lines and a drop warning when at least one structured event occurs during passthrough.

2) Config clamp warning:
- A test suite exists and is green:
  - `cargo test -p shell --test repl_config_max_pty_buffered_lines -- --nocapture` → exit `0`
- The tests assert:
  - out-of-range values clamp and emit exactly one `code="config_value_clamped"` warning record.

3) Manual and smoke validation:
- Manual playbook is complete and runnable:
  - `docs/project_management/next/agent-hub-concurrent-execution-output-routing/manual_testing_playbook.md`
- Smoke scripts exist, are runnable, and pass on their target platforms:
  - Linux: `bash docs/project_management/next/agent-hub-concurrent-execution-output-routing/smoke/linux-smoke.sh` → exit `0`
  - macOS: `bash docs/project_management/next/agent-hub-concurrent-execution-output-routing/smoke/macos-smoke.sh` → exit `0`
  - Windows: `pwsh -NoProfile -File docs/project_management/next/agent-hub-concurrent-execution-output-routing/smoke/windows-smoke.ps1` → exit `0`
