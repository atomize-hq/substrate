# Executive Summary Standard (Operator Review)

Goal:
- Make ADR review fast for a human operator by summarizing UX/behavior deltas and the rationale.
- Prevent drift between the ADR body and the operator summary.

## Placement (required)

Every ADR must contain a section with the exact header:
- `## Executive Summary (Operator)`

This section is intended to be the **first thing a human reads**.

## Format (required)

The section must include:
- `ADR_BODY_SHA256: <64-hex>` (drift guard; see below)
- One or more change entries with:
  - `Existing:` what happens today (operator-visible)
  - `New:` what happens after this ADR (operator-visible)
  - `Why:` why the change exists (safety, correctness, simplicity, velocity, etc.)
  - `Links:` deep links so the operator never has to “go search”

Recommended structure:
```md
## Executive Summary (Operator)

ADR_BODY_SHA256: <auto>

### Changes (operator-facing)
- <change title>
  - Existing: <…>
  - New: <…>
  - Why: <…>
  - Links:
    - `docs/project_management/next/<feature>/<file>.md#L123`
```

## Deep links (strongly recommended)

Prefer linking to:
- the exact spec section that defines the behavior (authoritative), and
- the exact implementation surface (crate/module/script) when known.

Best-effort link formats:
- GitHub-style line anchors: `path/to/file.md#L123`
- Stable section anchors: `path/to/file.md#user-contract-authoritative`

Line anchors are expected to change as ADRs evolve; update them as part of any ADR edit.

## Drift guard (required)

The `ADR_BODY_SHA256` value is a hash of the ADR body **excluding** the `## Executive Summary (Operator)` section.

Mechanics:
- Check: `make adr-check ADR=<adr.md>`
- Fix hash: `make adr-fix ADR=<adr.md>`
