# Add non-APT system-package provisioning support - Review Surfaces

These diagrams orient the pack. They show the actual product/work shape that is expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.

Active and next seams still require seam-local `review.md` later. These pack-level review surfaces exist so reviewers can see the end-to-end manager-aware workflow before seam-local decomposition starts.

## R1 - High-level provisioning and runtime workflow

```mermaid
flowchart TD
  O[Operator] --> P["substrate world enable --provision-deps"]
  P --> WB[Enable world backend]
  WB --> PROBE["Probe inside world:<br/>/etc/os-release + command -v pacman"]
  PROBE -->|unsupported or contradiction| EXIT4P["Exit 4:<br/>fail closed, no host mutation"]
  PROBE -->|supported| REQS["Derive normalized APT + pacman requirement sets"]
  REQS --> MIX{Both managers in scope?}
  MIX -->|yes| EXIT4M["Exit 4:<br/>mixed-manager rejection"]
  MIX -->|no| MUTATE["Execute only the detected manager<br/>(pacman path in scope here)"]
  MUTATE --> DONE[Provisioned or no-op]

  O --> R["substrate world deps current sync|install"]
  R --> SCOPE[Select runtime in-scope items]
  SCOPE --> RREQS["Derive normalized APT + pacman requirement sets"]
  RREQS --> READONLY["Read-only probes only:<br/>dpkg-query / pacman -Q"]
  READONLY -->|all satisfied| CONTINUE["Proceed with upstream non-system-package behavior"]
  READONLY -->|missing| EXIT4R["Exit 4 + remediation:<br/>substrate world enable --provision-deps"]
```

## R2 - CLI, shell, world-service, and inventory/data flow

```mermaid
flowchart LR
  INV[World-deps inventory YAML] --> PARSE[inventory.rs validation]
  PARSE --> VIEW[surfaces.rs list/show output]
  PARSE --> ENABLED[Enabled-set resolution]

  ENABLED --> CLI1[world enable --provision-deps]
  CLI1 --> RUNNER[world_enable runner]
  RUNNER --> DISPATCH[dispatch/world_ops]
  DISPATCH --> AGENT[world-service service]
  AGENT --> PROBE[/etc/os-release + command -v pacman/]
  PROBE --> EXEC[Provisioning path<br/>apt or pacman only]

  ENABLED --> CLI2[deps current sync/install]
  CLI2 --> RUNTIME[world_deps runtime surfaces]
  RUNTIME --> CHECKS[Read-only probes<br/>dpkg-query / pacman -Q]
  CHECKS --> REMEDY[Manager-aware remediation]
```

## R3 - Touch surface map from contract to conformance

```mermaid
flowchart TB
  CONTRACT[contract.md + decision_register.md] --> PROBESEAM[NASP0 / probe support gate]
  CONTRACT --> SCHEMASEAM[NASP1 / pacman schema]
  PROBESEAM --> ENABLEFLOW[world_enable runner/helper_script/log_ops]
  SCHEMASEAM --> INVFLOW[inventory.rs + inventory views/tests]
  ENABLEFLOW --> DISPATCH[dispatch/world_ops + world-service/service]
  INVFLOW --> PROVISION[NASP2 / provisioning routing]
  PROVISION --> RUNTIME[NASP3 / runtime fail-early]
  RUNTIME --> CONF[NASP4 / parity + manual + smoke]
  CONF --> DOCS[ADR-0033 + APT pack + bundles contract + world/deps docs]
```

## R4 - Platform posture that must remain true after landing

```mermaid
flowchart LR
  LINUX[Linux host-native] --> LFAIL["Provisioning unsupported<br/>Exit 4, no host mutation"]
  MAC[macOS Lima guest] --> MSUP["Supported when in-world probe selects one manager<br/>Default smoke uses Ubuntu guest"]
  MAC --> MMANUAL["Arch-family pacman-success remains manual evidence lane"]
  WIN[Windows WSL] --> WFAIL["Provisioning unsupported<br/>Exit 4, unsupported on Windows"]

  LFAIL --> RUNTIMEALL[Runtime remains read-only and fail-early]
  MSUP --> RUNTIMEALL
  MMANUAL --> RUNTIMEALL
  WFAIL --> RUNTIMEALL
```
