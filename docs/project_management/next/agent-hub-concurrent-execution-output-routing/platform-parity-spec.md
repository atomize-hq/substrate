# agent-hub-concurrent-execution-output-routing — platform parity spec

Owner standard:
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope
- This spec is authoritative for platform guarantees and permitted divergences for ADR-0017 output routing behavior.

## Required platforms
- Behavior platforms (smoke required): Linux, macOS, Windows
- CI parity platforms (parity required): Linux, macOS, Windows
- WSL required: false
- WSL task mode: bundled

## Guarantees (explicit)

What must behave identically across platforms:
- Structured agent events are never injected into PTY byte streams during PTY passthrough.
- PTY bytes are forwarded as raw bytes (no UTF-8 assumption; no re-encoding).
- During PTY passthrough, structured agent events are buffered up to the configured cap and dropped beyond the cap.
- When drops occur, exactly one suppression summary warning is emitted after passthrough ends and persisted to trace.
- While idle (prompt active), out-of-band PTY bytes and structured events do not corrupt the input buffer.

What may diverge (explicit list + rationale):
- Exact terminal redraw behavior while idle MAY differ based on the host terminal and platform line editor integration, but MUST preserve prompt/input correctness.
- If a platform/backend does not support PTY passthrough yet, PTY-specific guarantees are “N/A” until PTY passthrough exists; the structured-event path and trace persistence guarantees remain required.

## Known platform hazards (explicit)
- Windows terminals and PTY emulation differ; prompt redraw tests must be resilient while still detecting input corruption regressions.
- Non-UTF8 PTY byte output must not panic or corrupt rendering on any platform.

## Validation evidence (explicit)
- Smoke scripts required:
  - `docs/project_management/next/agent-hub-concurrent-execution-output-routing/smoke/linux-smoke.sh`
  - `docs/project_management/next/agent-hub-concurrent-execution-output-routing/smoke/macos-smoke.sh`
  - `docs/project_management/next/agent-hub-concurrent-execution-output-routing/smoke/windows-smoke.ps1`
- CI parity gates required:
  - Build + unit tests for touched crates (`crates/shell`, `crates/common`, `crates/trace`)
  - Platform smoke dispatch via CI when smoke scripts exist
- Manual playbook sections required:
  - Concurrent `:demo-agent` (structured events) while a PTY passthrough command is active
  - Out-of-band PTY bytes during prompt wait without input corruption

## Acceptance criteria (testable)
- On each required platform:
  - A PTY passthrough command can run while demo structured events are emitted without terminal corruption.
  - `trace.jsonl` contains `agent_event` records for structured events and a warning record when suppression occurs.

