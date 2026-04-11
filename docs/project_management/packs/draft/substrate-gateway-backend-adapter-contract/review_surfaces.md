# Review Surfaces - Substrate gateway backend adapter contract

These diagrams orient the pack. They show the actual product and service shape that should land.
They do not, by themselves, satisfy seam-local pre-exec review.
`SEAM-1` and `SEAM-2` still require seam-local `review.md` artifacts later.

## R1 - End-to-end selection and execution workflow

```mermaid
flowchart LR
  OP["Operator / caller"] --> SUB["Substrate selection boundary"]
  SUB --> CFG["ADR-0027 config + policy + inventory"]
  CFG --> BID["Stable backend id `<kind>:<name>`"]
  BID --> GATE["Fail-closed allowlist + selection check"]
  GATE --> GW["substrate-gateway adapter registry"]
  GW --> ADP["Selected backend adapter"]
  ADP --> BE["CLI or API backend"]
  BE --> ADP
  ADP --> EVT["Normalized response + local event translation"]
  EVT --> OUT["Substrate-owned status / trace / client response"]
```

## R2 - Ownership and boundary flow

```mermaid
flowchart TB
  HOST["Host / Substrate"] --> WORLD["World boundary when policy requires it"]
  WORLD --> GW["substrate-gateway runtime"]
  GW --> ADP["Adapter contract"]
  ADP --> PROV["Provider / wrapper mechanics"]

  HOST -. "owns selection, allowlisting, lifecycle, operator status" .-> GW
  ADP -. "owns capability validation, session handles, request/response translation" .-> PROV
  GW -. "hands off event envelope to ADR-0017" .-> E1["Structured event owner"]
  GW -. "hands off canonical trace vocabulary to ADR-0028" .-> E2["Trace owner"]
```

## R3 - Pack touch-surface map

```mermaid
flowchart LR
  ADR["ADR-0041 + pre-planning packet"] --> S1["SEAM-1 contract/policy docs"]
  S1 --> S2["SEAM-2 protocol/schema docs"]
  S1 --> EXT1["Existing operator/status/policy contracts"]
  S2 --> EXT2["ADR-0017 / ADR-0028 owner lines"]
  S2 --> S3["SEAM-3 parity/compat/manual validation"]
  EXT1 --> S3
  EXT2 --> S3
  S3 --> CLOSE["Governance closeout + downstream promotion basis"]
```
