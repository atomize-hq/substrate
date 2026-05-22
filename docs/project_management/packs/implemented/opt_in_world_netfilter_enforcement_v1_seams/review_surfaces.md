# Review Surfaces - Opt-in World Netfilter Enforcement

These diagrams orient the pack. They show the actual product/work shape that is expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.

For `SEAM-4` (active) and `SEAM-5` (next), the authoritative pre-exec review artifact remains the seam-local `threaded-seams/.../review.md` that will be created during downstream seam decomposition.

## R1 - High-level workflow

```mermaid
flowchart LR
  U["User / Operator"] -->|"sets config"| CFG["world.net.filter (host config)"]
  U -->|"runs command"| SH["substrate shell"]
  SH -->|"policy eval"| BR["broker (policy)"]
  BR -->|"effective policy"| SNAP["PolicySnapshotV3 (net_allowed)"]
  SH -->|"execute request (WorldSpec)"| WA["world-service"]
  WA -->|"spawns inside world"| W["world backend (cgroup/netns)"]
  W -->|"nftables/netfilter install (opt-in)"| NF["egress enforcement"]
```

## R2 - Control plane vs data plane separation

```mermaid
flowchart TB
  subgraph Host["Host / Control plane"]
    BR["broker policy evaluation"]
    SH["shell snapshot builder"]
    CFG["config gate: world.net.filter"]
  end
  subgraph World["World / Data plane"]
    WA["world-service service"]
    NF["netfilter (nftables)"]
    EX["executed process"]
  end

  BR -->|"net_allowed"| SH
  CFG -->|"request isolate_network?"| SH
  SH -->|"PolicySnapshotV3 + WorldSpec"| WA
  WA -->|"apply rules + attach cgroup"| NF
  NF -->|"enforced egress"| EX
```

## R3 - Touch surface map (key modules)

```mermaid
flowchart TB
  CLI["src/substrate (CLI)"] --> SCFG["crates/shell::config_model"]
  CLI --> SSNAP["crates/shell::policy_snapshot"]
  SSNAP --> AT["crates/transport-api-types::PolicySnapshotV3"]
  SSNAP --> WAX["crates/world-service (service + pty)"]
  WAX --> WLIB["crates/world (session + netfilter)"]
  WAX --> DOCT["world doctor endpoint"]
  SCFG --> DOCS["docs/reference/config/world.md + CONFIGURATION.md"]
```
