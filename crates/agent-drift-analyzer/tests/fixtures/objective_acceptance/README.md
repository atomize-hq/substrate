# Objective acceptance fixture contract

Packet `SO-4.1` establishes only the committed harness scaffold and deterministic fixture layout.
It does **not** seed real cases yet, and it does **not** encode the detailed expected-shape
contract from `SO-4.2`.

The committed root must stay bounded to:

- `README.md`
- `design-set/`
- `locked-acceptance/`
- `stretch-external/`

Each family directory is intentionally placeholder-only in this packet. Real case directories land
later under `SO-5.*`, at which point the harness can enumerate `<case-id>/raw.json` and
`<case-id>/expected.json` entries deterministically.
