# Seam Map - Best-Effort Distro Package Manager

## Overview

Four capability/doman seams extracted from the ADR-0031 scope:

| Seam | Type | Horizon | Purpose |
|------|------|---------|---------|
| SEAM-01 | capability | active | Distro detection via `/etc/os-release`, family mapping, and stable decision-line reporting |
| SEAM-02 | capability | next | Override precedence, failure classes, and deterministic fallback behavior |
| SEAM-03 | integration | future | Wrapper exit-status pass-through and operator/env-doc alignment |
| SEAM-04 | conformance | future | Hermetic validation, CI checkpoint, and contract lock-in |

## Seam Relationships

```
SEAM-01 ---> SEAM-02 ---> SEAM-03 ---> SEAM-04
   |            |            |            |
   |            |            |            v
   |            |            |       [Checkpoint]
   |            |            |
   |            |            v
   |            |       docs alignmnt
   |            v
   |       failure paths
   v
os-release contract
```

## Thread Overview

- THR-01: `SEAM-01` → `SEAM-02` — os-release parsed data and `pkg_manager.source` vocabulary
- THR-02: `SEAM-01` → `SEAM-04` — hermetic test os-release input contract
- THR-03: `SEAM-02` → `SEAM-03` — exit-code taxonomy and failure-class semantics
- THR-04: `SEAM-02` → `SEAM-04` — override precedence validation coverage
- THR-05: `SEAM-03` → `SEAM-04` — wrapper pass-through contract
- THR-06: `SEAM-01` → downstream pack — detection contract for persistence

## Execution Progression

1. **SEAM-01** (active): Establish detection, mapping, and reporting foundation
2. **SEAM-02** (next): Build override mechanisms and failure paths on SEAM-01 basis
3. **SEAM-03** (future): Integrate wrapper and docs once failure classes are stable
4. **SEAM-04** (future): Validate everything hermetically and lock the checkpoint

## Downstream Dependencies

- `persist-detected-linux-distro-pkg-manager`: Consumes `pkg_manager.selected`, `pkg_manager.source`, `<unknown>` sentinel, `SUBSTRATE_INSTALL_OS_RELEASE_PATH` contract
- `world-deps-apt-provisioning`: Boundary established to keep host installer separate from guest provisioning
- `ADR-0032` (stashing ferret): Inherits detection contract for persistence
