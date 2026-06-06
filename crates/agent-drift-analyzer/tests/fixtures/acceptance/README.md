# Packet R2 Acceptance Fixtures

This directory freezes the screened `R1E` analyzer-local acceptance corpus for Packet `R2`.

Included success-tail cases:

- `019e93fa-60d4-73d1-9092-014130b60e14`
- `019e940c-a91b-7fe0-a967-b0bdd595b581`
- `019e943c-668e-7a03-992b-6a98cf3055da`
- `019e894a-86c9-71e3-b57b-e3d3285f0988`

Excluded cases:

- `019e93f8-a5e9-7490-ac1a-955b74c92ad0` delegated session
- `019e9406-6736-79a2-946b-8a603e557422` delegated session
- `019e9401-9d69-7190-a43e-9ee3be08b369` non-success-tail session

Maintenance rules:

- The copied `manifest.json`, `rows.archival.jsonl`, `rows.compact.jsonl`, and
  `dedupe-audit.jsonl` files come from the landed `target/hybrid-drift-evals/*-r1e/compactor`
  artifacts.
- `expected.json` is the Packet `R2` local contract for final `dead_end_thrash` posture.
- Tests must fail closed if a checked-in case is missing; they must not read from `target/` or
  `~/.codex` at test time.
- Later corpus changes or expected-posture changes require explicit packet authority.
