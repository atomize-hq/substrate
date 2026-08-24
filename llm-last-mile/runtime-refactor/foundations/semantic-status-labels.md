**Kind:** foundation
**Status:** canonical
**Canonical for:** semantic status labels and their promotion rule

## Semantic status labels

These labels describe the **target seam as a whole**, not the quality of individual functions.

| Label | Meaning |
|---|---|
| `ContractCorrectAndProven` | Correct owner, real path, enforcement point, and runtime proof all exist. |
| `UsefulFootholdButWrongBoundary` | Reusable logic/data exists, but ownership or call-path placement is wrong. |
| `DefensiveScaffoldingOnly` | The artifact reduces risk or enables transition, but does not implement the target authority contract. |
| `MislandedWrongModel` | The implementation encodes semantics that conflict with the target model and must be replaced or inverted. |
| `MissingSeam` | No meaningful implementation of the target boundary exists, even if neighboring primitives do. |

Promotion to `ContractCorrectAndProven` requires explicit evidence for all four landing conditions. A component test cannot promote a seam whose production path bypasses it.

**Source provenance:** extracted byte-for-byte from [`00-README.md#semantic-status-labels`](../00-README.md#semantic-status-labels), baseline lines 143–156
**Baseline span SHA-256:** `d9179c4258ed8d17f4a2127729d5b537fad56c284ea05d6e0488eb9f66853736`
