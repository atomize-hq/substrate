# Review Surfaces - world-disabled-reason-attribution

These diagrams orient the pack. They show the actual replay behavior, provenance resolution path, and telemetry shape that are expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.
`SEAM-1` and `SEAM-2` still require seam-local `review.md` artifacts later.

## R1 - Replay decision and explanation workflow

```mermaid
flowchart LR
  U["Operator runs substrate --replay <span_id>"] --> I["Selection inputs<br/>--world / --no-world / SUBSTRATE_REPLAY_USE_WORLD / recorded origin"]
  I --> R["Replay routing"]
  R -->|"world replay selected"| W["World replay path"]
  R -->|"host replay selected"| H["Host replay path"]
  H --> C["Effective-disable classifier<br/>(only for world.enabled=false cases)"]
  C --> O["Origin summary line"]
  C --> HW["Host warning line"]
  C --> T["replay_strategy trace event"]
```

## R2 - Effective world-disable provenance resolution

```mermaid
flowchart TB
  S["Need host-replay explanation for world.enabled=false"] --> WS{"Workspace exists?"}
  WS -->|"Yes"| WP{"workspace.yaml sets world.enabled: false?"}
  WP -->|"Yes"| W["Use <workspace>/.substrate/workspace.yaml"]
  WP -->|"No"| GP{"global config sets world.enabled: false?"}
  WS -->|"No"| OE{"SUBSTRATE_OVERRIDE_WORLD=disabled?"}
  OE -->|"Yes"| E["Use SUBSTRATE_OVERRIDE_WORLD=disabled"]
  OE -->|"No"| GP
  GP -->|"Yes"| G["Use $SUBSTRATE_HOME/config.yaml"]
  GP -->|"No"| U["Use effective config (source unknown)"]
```

## R3 - Runtime output and telemetry publication

```mermaid
flowchart LR
  C["Classifier result"] --> OR["[replay] origin: ..."]
  C --> HW["[replay] warn: running on host (...)"]
  C --> RC["origin_reason_code"]
  C --> WDS["world_disable_source"]
  RC --> RS["replay_strategy"]
  WDS --> RS
```

## R4 - Touch-surface orientation map

```mermaid
flowchart TB
  CMD["substrate replay command"] --> RR["crates/shell/src/execution/routing/replay.rs"]
  RR --> CM["crates/shell/src/execution/config_model.rs<br/>or adjacent helper seam"]
  RR --> EX["crates/replay/src/replay/executor.rs"]
  RR --> TEST["crates/shell/tests/replay_world.rs"]
  EX --> TRACE["docs/TRACE.md examples"]
  RR --> RPD["docs/REPLAY.md"]
  RR --> CMDS["docs/COMMANDS.md"]
  TEST --> SMOKE["smoke/linux-smoke.sh<br/>smoke/macos-smoke.sh<br/>smoke/windows-smoke.ps1"]
```
