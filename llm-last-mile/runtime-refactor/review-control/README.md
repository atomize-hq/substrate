# Runtime-refactor review control

This directory contains the closed V1 review-cycle schema, its standard-library validator, and
durable process evidence referenced by the runtime-refactor control pack. Process evidence does not
replace product proof or packet authority.

## R2-3Z durable records

- [`r2-3z-terminal-receipt.json`](r2-3z-terminal-receipt.json) is a byte-identical copy of
  `/tmp/r2-3z-terminal-receipt.json` as observed during the pre-R2-4 bookkeeping correction.
  SHA-256: `681700be6b4c483a78896573f8c982591c6f026fae566fe8c45b5b829f82e58a`.
  Its terminal subject fingerprint is
  `sha256:2478c015df2851e05b767f81a8cc889cd1b936be38fbc11c33e816f2ff607a5a`.
- [`r2-3z-review-cycle-record.json`](r2-3z-review-cycle-record.json) is a byte-identical copy of
  `/home/spenser/.codex/visualizations/2026/07/31/019fba3a-9e25-7321-a900-dd410e530c67/review/review-record.json`.
  SHA-256: `a3236d6ec901906d1ef84851d45995b2d595820be93d20cb40e3af1bb9db73ef`.
  It validates as `complete` and ends with a `clean` closure cycle.

The receipt and review-cycle JSON are the only artifacts copied from those external locations. The
raw discovery review, final CLEAN review, and subject-fingerprint file named by the receipt remain
external; this repository does not claim to contain copies of them.

Validate the tracked cycle record from the repository root:

```bash
python3 llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py \
  llm-last-mile/runtime-refactor/review-control/r2-3z-review-cycle-record.json
```

## Pre-R2-4 readiness

The future dedicated Linux proof assignment and restoration boundary are recorded in
[`r2-4-linux-host-booking.md`](r2-4-linux-host-booking.md). It is a pre-start contract, not R2-4
execution or proof.
