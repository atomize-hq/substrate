# Threading - Best-Effort Distro Package Manager

## Execution Horizon Summary

| Seam | Horizon | Role | Key Output |
|------|---------|------|------------|
| SEAM-01 | active | Distro detection and mapping | os-release contract, mapping table, decision line |
| SEAM-02 | next | Override and fallback | Precedence chain, failure classes, exit codes 2/3/4 |
| SEAM-03 | future | Wrapper and docs | Wrapper pass-through, operator doc alignment |
| SEAM-04 | future | Validation checkpoint | Hermetic coverage, CI checkpoint, contract lock-in |

## Contract Registry

- **Contract ID**: `C-01`
  - **Type**: schema
  - **Owner seam**: `SEAM-01`
  - **Direct consumers**: `SEAM-02`, `SEAM-04`
  - **Derived consumers**: downstream pack (`persist-detected-linux-distro-pkg-manager`)
  - **Thread IDs**: THR-01, THR-02, THR-06
  - **Definition**: os-release parsing contract (safe line-oriented parser, `ID`/`ID_LIKE` extraction, normalization, `<unknown>` sentinel)
  - **Versioning / compat**: v1; additive changes only; breaking changes require new contract ID

- **Contract ID**: `C-02`
  - **Type**: schema
  - **Owner seam**: `SEAM-01`
  - **Direct consumers**: `SEAM-02`, `SEAM-04`
  - **Derived consumers**: downstream pack
  - **Thread IDs**: THR-01, THR-06
  - **Definition**: Selected-manager vocabulary (`apt-get`, `dnf`, `yum`, `pacman`, `zypper`) and `pkg_manager.source` vocabulary (`flag`, `env`, `os_release`, `path_probe`)
  - **Versioning / compat**: v1; fixed vocabulary; new values require new contract ID

- **Contract ID**: `C-03`
  - **Type**: API
  - **Owner seam**: `SEAM-01`
  - **Direct consumers**: `SEAM-03`, `SEAM-04`
  - **Derived consumers**: operator docs
  - **Thread IDs**: THR-01
  - **Definition**: Stable stderr decision-line format and placement contract
  - **Versioning / compat**: v1; template stability required; fields may be additive

- **Contract ID**: `C-04`
  - **Type**: schema
  - **Owner seam**: `SEAM-02`
  - **Direct consumers**: `SEAM-03`, `SEAM-04`
  - **Derived consumers**: operator docs
  - **Thread IDs**: THR-03, THR-04
  - **Definition**: Exit-code taxonomy for package-manager failures (`2` invalid override, `3` forced missing, `4` none selected)
  - **Versioning / compat**: v1; aligned with shared EXIT_CODE_TAXONOMY.md

- **Contract ID**: `C-05`
  - **Type**: UX affordance
  - **Owner seam**: `SEAM-02`
  - **Direct consumers**: `SEAM-04`
  - **Derived consumers**: operator docs
  - **Thread IDs**: THR-03
  - **Definition**: Multi-manager PATH warning template and deterministic selection order
  - **Versioning / compat**: v1; warning text is contractual

- **Contract ID**: `C-06`
  - **Type**: config
  - **Owner seam**: `SEAM-02`
  - **Direct consumers**: `SEAM-04`
  - **Derived consumers**: downstream pack
  - **Thread IDs**: THR-02
  - **Definition**: `SUBSTRATE_INSTALL_OS_RELEASE_PATH` env var contract (alternate input path for hermetic tests)
  - **Versioning / compat**: v1; Linux-only installer-local env var

- **Contract ID**: `C-07`
  - **Type**: API
  - **Owner seam**: `SEAM-03`
  - **Direct consumers**: `SEAM-04`
  - **Derived consumers**: operator docs
  - **Thread IDs**: THR-05
  - **Definition**: Wrapper exit-status pass-through contract (preserves `0`, `2`, `3`, `4`)
  - **Versioning / compat**: v1; preserves upstream taxonomy

## Thread Registry

- **Thread ID**: `THR-01`
  - **Producer seam**: `SEAM-01`
  - **Consumer seam(s)**: `SEAM-02`, `SEAM-03`, `SEAM-04`, downstream pack
  - **Carried contract IDs**: C-01, C-02, C-03
  - **Purpose**: Carry parsed os-release data and emitted vocabulary from detection to all downstream seams
  - **State**: identified → defined (upon C-01/C-02 publication)
  - **Revalidation trigger**: os-release parser changes in SEAM-01
  - **Satisfied by**: SEAM-01 closeout with landed parser + vocabulary
  - **Notes**: Foundation thread; all other work depends on this

- **Thread ID**: `THR-02`
  - **Producer seam**: `SEAM-01`
  - **Consumer seam(s)**: `SEAM-04`
  - **Carried contract IDs**: C-06 (carries C-01 contract for test inputs)
  - **Purpose**: Enable hermetic tests to inject os-release inputs without mutating host
  - **State**: identified → defined
  - **Revalidation trigger**: hermetic test harness changes
  - **Satisfied by**: SEAM-01 establishing C-06 contract
  - **Notes**: Test-only contract; non-test environments ignore this

- **Thread ID**: `THR-03`
  - **Producer seam**: `SEAM-02`
  - **Consumer seam(s)`: `SEAM-03`, `SEAM-04`
  - **Carried contract IDs**: C-04, C-05
  - **Purpose**: Carry failure-class taxonomy and warning semantics to wrapper and validation
  - **State**: identified
  - **Revalidation trigger**: exit-code meaning changes, warning template changes
  - **Satisfied by**: SEAM-02 closeout with landed failure handling
  - **Notes**: Critical for operator-visible contract stability

- **Thread ID**: `THR-04`
  - **Producer seam**: `SEAM-02`
  - **Consumer seam(s)**: `SEAM-04`
  - **Carried contract IDs**: C-04 (precedence chain validation)
  - **Purpose**: Enable hermetic tests to validate full precedence chain
  - **State**: identified
  - **Revalidation trigger**: override precedence changes
  - **Satisfied by**: SEAM-02 closeout with landed precedence implementation
  - **Notes**: Test validation thread

- **Thread ID**: `THR-05`
  - **Producer seam**: `SEAM-03`
  - **Consumer seam(s)**: `SEAM-04`
  - **Carried contract IDs**: C-07
  - **Purpose**: Carry wrapper pass-through contract to validation seam
  - **State**: identified
  - **Revalidation trigger**: wrapper implementation changes
  - **Satisfied by**: SEAM-03 closeout with landed wrapper changes
  - **Notes**: Ensures wrapper path preserves taxonomy

- **Thread ID**: `THR-06`
  - **Producer seam**: `SEAM-01`
  - **Consumer seam(s)**: downstream pack (`persist-detected-linux-distro-pkg-manager`)
  - **Carried contract IDs**: C-01, C-02
  - **Purpose**: Export detection contract for persistence layer
  - **State**: identified → defined (upon C-01/C-02 publication)
  - **Revalidation trigger**: detection contract changes
  - **Satisfied by**: SEAM-01 closeout
  - **Notes**: Cross-pack thread; downstream pack owns persistence

## Dependency Graph

```
           C-01          C-02          C-03
            |             |             |
            v             v             v
[SEAM-01] --+--[THR-01]--+--[THR-01]--+--[THR-01]--> [SEAM-02]
            |                                            |
            +--[THR-02]--> [SEAM-04]                     |
            |             (C-06)                         |
            |                                            v
            +----------------------------------> [downstream pack]
            (THR-06)                                   |
                                                     C-04, C-05
                                                       |
                                     +-----------------+-----------------+
                                     |                 |                 |
                                     v                 v                 v
                                   [THR-03]         [THR-04]          [THR-05]
                                     |                 |                 |
                                     v                 v                 v
                                   [SEAM-03]       [SEAM-04]          [SEAM-04]
                                     |                                   |
                                     +----------- [THR-05] ------------>+
                                     (C-07)                              |
                                                                         v
                                                                   [CI Checkpoint]
```

## Critical Path

```
SEAM-01 (C-01, C-02, C-03 defined)
    |
    v
SEAM-02 (C-04, C-05 defined; THR-01 consumed)
    |
    v
SEAM-03 (C-07 defined; THR-03 consumed)
    |
    v
SEAM-04 (all threads consumed; CI checkpoint)
```

## Workstreams

No separate workstream owners. All work is seam-owned. Classification comes from seam `type` field:
- `SEAM-01`: capability (detection capability)
- `SEAM-02`: capability (override capability)
- `SEAM-03`: integration (wrapper/docs integration)
- `SEAM-04`: conformance (validation checkpoint)
