# Decision Register — Agent Hub Concurrent Execution Output Routing

## DR-01 — Output classes: PTY bytes vs structured agent events

### Option A
- Define two distinct output classes:
  - **PTY bytes**: raw PTY stream output (binary-safe), rendered as bytes.
  - **Structured agent events**: Substrate-managed typed messages, rendered separately from PTY bytes.
- During PTY passthrough, structured events are buffered and flushed after passthrough ends (no injection into PTY byte stream).

### Option B
- Treat all output as strings and render through a single shared channel (PTY output and structured agent events both flow through the same printer).

### Tradeoffs
- A:
  - Pros: prevents terminal corruption during TUIs; keeps output attribution clean; aligns with ADR-0016 PTY passthrough invariants.
  - Cons: requires additional plumbing to render bytes safely while Reedline is active; buffering policy needed for structured events.
- B:
  - Pros: simplest implementation (single output path).
  - Cons: unsafe for TUIs/PTY passthrough; cannot represent arbitrary bytes; risks corrupting terminal state and user input.

### Decision
- Selected: Option A.
