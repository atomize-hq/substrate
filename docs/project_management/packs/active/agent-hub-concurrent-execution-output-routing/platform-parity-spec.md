# agent-hub-concurrent-execution-output-routing — platform parity spec

Owner standard:

- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for platform guarantees and permitted divergences for ADR-0017 output routing behavior.

## Required platforms

- Behavior platforms (smoke required): `linux`, `macos`, `windows`
- CI parity platforms (parity required): `linux`, `macos`, `windows`
- WSL required: `false`

## Guarantees (explicit)

What must behave identically across platforms:

- Structured agent events are never injected into PTY byte streams during PTY passthrough.
- PTY bytes are forwarded as raw bytes (no UTF-8 assumption; no re-encoding).
- During PTY passthrough, structured agent events are buffered up to the configured cap and dropped beyond the cap.
- When drops occur, exactly one suppression summary warning is emitted after passthrough ends and persisted to trace.
- While idle (prompt active), out-of-band PTY bytes and structured events do not corrupt the input buffer.

What may diverge (explicit list + rationale):

- Exact terminal redraw escape sequences while idle are not specified; the observable invariant is prompt/input correctness and absence of injected structured output into PTY bytes.
- On platforms where Substrate does not support PTY passthrough (currently Windows), PTY-specific guarantees are not applicable; envelope + trace persistence guarantees remain required.

## Known platform hazards (explicit)

- Windows terminals and PTY emulation differ; prompt redraw tests must be resilient while still detecting input corruption regressions.
- Non-UTF8 PTY byte output must not panic or corrupt rendering on any platform.

## Validation evidence (explicit)

- Smoke scripts required:
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/linux-smoke.sh`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/macos-smoke.sh`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/windows-smoke.ps1`
- CI parity gates required:
  - Build + unit tests for touched crates (`crates/shell`, `crates/common`, `crates/trace`)
  - Platform smoke dispatch via CI when smoke scripts exist
- Manual playbook sections required:
  - Concurrent `:demo-agent` (structured events) while a PTY passthrough command is active
  - Out-of-band PTY bytes during prompt wait without input corruption

## Acceptance criteria (testable)

- Linux and macOS:
  - A PTY passthrough command runs while demo structured events are emitted without terminal corruption or prompt corruption.
  - If structured events are emitted during PTY passthrough, the canonical trace contains:
    - one `event_type="agent_event"` record per structured event, and
    - one `event_type="warning"` record with `code="pty_structured_event_drops"` when drops occur.
- Windows:
  - The canonical trace contains one `event_type="agent_event"` record per structured event emitted by `:demo-agent`.
  - PTY passthrough guarantees are not applicable because PTY passthrough is not supported on Windows.
