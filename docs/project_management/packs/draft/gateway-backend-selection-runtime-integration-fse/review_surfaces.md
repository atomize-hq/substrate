# Review Surfaces - gateway-backend-selection-runtime-integration

These diagrams orient the pack. They show the actual operator flow, integrated runtime flow, and parity/rollout proof shape that are expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.
`SEAM-1` and `SEAM-2` still require seam-local `review.md` artifacts later.

## R1 - Selected-backend realization flow

```mermaid
flowchart LR
  CFG["effective config<br/>llm.routing.default_backend"] --> SEL["selected backend id"]
  POL["effective policy<br/>allowlists + auth-read gates"] --> GATE["selection and policy gate"]
  INV["backend inventory"] --> GATE
  SEL --> GATE
  GATE -->|"allowed + consistent"| BIND["integrated adapter binding"]
  GATE -->|"invalid / denied / unavailable"| FAIL["typed failure classification"]
  BIND --> RT["adapter-driven runtime realization"]
  RT --> CMD["status | sync | restart"]
```

## R2 - Auth handoff and managed artifact path

```mermaid
flowchart TB
  SRC["authorized auth source<br/>env or host credential file"] --> AUTH["integrated auth handoff"]
  AUTH --> WA["world-agent gateway runtime manager"]
  WA --> CFG["rendered runtime config"]
  WA --> MAN["runtime manifest"]
  WA --> LOG["managed stdout/stderr logs"]
  CFG --> GW["substrate-gateway start"]
  MAN --> GW
  LOG --> OPS["operator diagnostics"]
```

## R3 - Future parity and rollout proof

```mermaid
flowchart LR
  BASE["cli:codex regression floor"] --> MATRIX["validation matrix"]
  ADD["first non-cli:codex integrated backend"] --> MATRIX
  MATRIX --> LNX["Linux proof"]
  MATRIX --> MAC["macOS proof"]
  MATRIX --> WIN["Windows proof"]
  MATRIX --> UNSUP["explicit unsupported-backend outcome"]
  MATRIX --> ROLL["compatibility and rollout posture"]
```
