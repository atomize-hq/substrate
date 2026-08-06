# R3 LINUX CLOSEOUT allowlist and evidence review

Terminal subject fingerprint:
`sha256:83a49872df62fc0a3c4c4baa05ac8ff164a5a2973f24011b7628866fe66cee86`

Exact packet changed-path fence:

1. `llm-last-mile/runtime-refactor/00-README.md`
2. `llm-last-mile/runtime-refactor/03-phase-slice-map.md`
3. `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
4. `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
5. `llm-last-mile/runtime-refactor/review-control/r3-linux-imp-01-evidence.json`
6. `llm-last-mile/runtime-refactor/review-control/r3-linux-imp-01-receipt.json`
7. `llm-last-mile/runtime-refactor/review-control/r3-linux-closeout-review-authority-security.md`
8. `llm-last-mile/runtime-refactor/review-control/r3-linux-closeout-review-lifecycle-convergence.md`
9. `llm-last-mile/runtime-refactor/review-control/r3-linux-closeout-review-allowlist-evidence.md`
10. `llm-last-mile/runtime-refactor/review-control/r3-linux-closeout-review-cycle-record.json`

Allowlist and evidence-integrity checks bound before publication:

- the copied artifact and receipt digests exactly match the validated external files;
- the exact packet fence stays inside `R3-DOCS`, the two permitted evidence files, and the exact
  `linux-closeout` review set;
- `python3 -m json.tool` succeeds for both copied JSON files;
- `git diff --check` is clean for the packet candidate; and
- a heuristic secret/conflict scan over the packet diff reported no matches.

No `06-review-finding-inventory.md` update is required because there are no valid unfixed `P3` or
`P4` findings for this packet.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.
