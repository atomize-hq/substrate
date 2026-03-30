# Review Surfaces - Persist detected Linux distro + pkg manager

These diagrams orient the pack. They show the actual product/work shape that is expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.
Active and next seams still require seam-local `review.md` later.

## R1 - Successful Linux install metadata flow

```mermaid
flowchart LR
  U[Operator or automation] --> I[Hosted or dev installer]
  I --> D[Detect distro and selected pkg manager]
  D --> B{Eligible to persist}
  B -->|Linux success| R[Read existing install_state if present]
  B -->|Hosted dry run| N1[No write]
  B -->|macOS or Windows| N2[No platform metadata write]
  R --> M[Merge schema version one and preserve legacy plus unknown keys]
  M --> T[Write install_state temp file in same directory]
  T --> X[Replace canonical install_state file]
  X --> C[Canonical metadata available under effective prefix]
  C --> G[Future guidance may read persisted metadata]
```

Orientation note:
- This is the end-to-end product shape for a successful Linux producer flow.
- The diagram intentionally separates no-write branches from the successful Linux path because the feature contract depends on that distinction.

## R2 - Failure posture and degraded metadata behavior

```mermaid
flowchart TB
  OS[os release input] --> P[Platform metadata builder]
  DET[Detection contract outputs] --> P
  P --> F{Read or write issue}
  F -->|No issue| OK[Installer success with persisted metadata]
  F -->|Missing os release only| U[Persist unknown sentinel for os release and keep pkg manager fields]
  U --> OK
  F -->|Metadata read or write failure| W[Emit warning only]
  W --> S[Preserve installer success status]
```

Orientation note:
- The landed feature is not just a file write. It is a fail-open control flow where metadata issues do not redefine installer success for this pack.

## R3 - Authority boundary and file touch map

```mermaid
flowchart TB
  UP[Upstream detection contract] --> C1[Contract and schema authority]
  C1 --> HI[Hosted installer script]
  C1 --> DI[Dev installer script]
  HI --> ST[(effective prefix install_state file)]
  DI --> ST
  ST --> SM[Linux smoke harness]
  ST --> DOC[Installation documentation]
  SM --> CP[Checkpoint evidence]
  DOC --> OP[Operator guidance]
```

Orientation note:
- The upstream detection contract remains external authority.
- This pack owns persistence, runtime write behavior, smoke evidence, and operator wording that depend on that upstream truth.

## R4 - Legacy compatibility and rewrite surface

```mermaid
flowchart LR
  OLD[Existing install_state with group linger and unknown keys] --> READ[Read current document]
  READ --> REFRESH[Refresh platform block only]
  REFRESH --> KEEP[Keep legacy and unknown fields unchanged]
  KEEP --> TMP[Render complete temp document]
  TMP --> FINAL[Replace canonical document]
```

Orientation note:
- This diagram highlights why the seam split matters: the feature must preserve legacy content while adding one new platform block, and it must do so without in-place truncation.
