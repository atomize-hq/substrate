# Review Surfaces - execution-surface-parity-hardening

These diagrams orient the pack. They show the actual execution surfaces, routing decisions, trace publication points, and abnormal-terminal-loss behavior that are expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.
`SEAM-1` and `SEAM-2` still require seam-local `review.md` artifacts later.

## R1 - High-level execution-surface workflow

```mermaid
flowchart LR
  O["Operator / maintainer"] --> CMD["substrate --command"]
  O --> RPY["substrate replay <span_id>"]
  O --> REPL["substrate --async-repl"]
  CMD --> SHELL["Shell execution layer"]
  RPY --> SHELL
  REPL --> SHELL
  SHELL --> DEC["Contract decisions"]
  DEC --> ROUTE["World / host routing"]
  DEC --> TRACE["Canonical trace expectations"]
  DEC --> EXIT["Exit code + diagnostic surface"]
```

## R2 - Replay routing parity flow

```mermaid
flowchart LR
  RPY["Replay request"] --> SNAP["Effective policy snapshot"]
  SNAP --> GATE{"world.net.filter active?"}
  GATE -->|"no"| HOST["No requested isolation"]
  GATE -->|"yes + ['*']"| HOST
  GATE -->|"yes + []"| ISO["Requested isolation<br/>empty allowlist"]
  GATE -->|"yes + restrictive list"| ALLOW["Requested isolation<br/>canonical domains"]
  HOST --> EXEC["Local or agent-backed replay executor"]
  ISO --> EXEC
  ALLOW --> EXEC
```

## R3 - Tracing behavior and validation flow

```mermaid
flowchart TB
  MODE["Execution mode<br/>wrap | script | interactive"] --> SRC["Builtin handling or child shell"]
  SRC --> PRE{"Preexec enabled here?"}
  PRE -->|"yes"| BC["builtin_command emitted<br/>body omitted"]
  PRE -->|"no"| NOBC["No preexec-derived builtin event"]
  SRC --> WPE["world_process_* when world telemetry is expected"]
  BC --> TRACE["Canonical trace.jsonl"]
  NOBC --> TRACE
  WPE --> TRACE
  TRACE --> PLAY["WPEP playbook + smoke assertions"]
```

## R4 - Interactive terminal-loss handling flow

```mermaid
flowchart LR
  PTY["Controlling TTY active"] --> READ["Prompt worker in Reedline read_line()"]
  READ --> LOSS["TTY revoked / disconnected"]
  LOSS --> DETECT["Disconnect classification + worker unwind"]
  DETECT --> DIAG["Single best-effort stderr diagnostic"]
  DETECT --> EXIT1["REPL session exit code = 1"]
  EXIT1 --> CLEAN["No orphaned CPU spin"]
```

## R5 - Touch-surface orientation map

```mermaid
flowchart TB
  ROUTE["Replay routing"] --> EXE["crates/replay/src/replay/executor.rs"]
  ROUTE --> SNAP["crates/shell/src/execution/policy_snapshot.rs"]
  TRACEFLOW["Tracing semantics"] --> PREEXEC["crates/shell/src/scripts/bash_preexec.rs"]
  TRACEFLOW --> MANAGER["crates/shell/src/execution/manager.rs"]
  TRACEFLOW --> DISPATCH["crates/shell/src/execution/routing/dispatch/exec.rs"]
  REPLFLOW["Interactive REPL resilience"] --> ASYNC["crates/shell/src/repl/async_repl.rs"]
  REPLFLOW --> EDITOR["crates/shell/src/repl/editor.rs"]
  EXE --> DOCS["docs/REPLAY.md"]
  PREEXEC --> TRACE["docs/TRACE.md"]
  ASYNC --> USAGE["docs/USAGE.md"]
  TRACEFLOW --> WPEP["world_process_exec_tracing_parity playbook + smoke"]
```
