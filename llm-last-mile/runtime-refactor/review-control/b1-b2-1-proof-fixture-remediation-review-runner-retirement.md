# B1/B2.1 runner retirement and allowlist review

Subject fingerprint:
`sha256:c4027fca095cfb6004ef15341f9eb565d6cbe3e527f75254f099f1cb8b0f673e`

Evidence reviewed:

- `git diff --name-only --diff-filter=ACDMRTUXB`
- `git grep -n 'canonical_shell_wall_runner.py\\|test_canonical_shell_wall_runner.py\\|canonical_shell_wall_runner\\|test_canonical_shell_wall_runner' -- . ':(exclude)llm-last-mile/runtime-refactor/**'`
- `/home/spenser/.cache/substrate-runtime-refactor/b1-b2-1-joint-closeout`
- `/home/spenser/.cache/substrate-runtime-refactor/b1-b2-1-proof-fixture-remediation-final/20260803T171725Z/make-probes/probe-results.json`

Read-only findings:

- The only tracked path changes are `Makefile`, the six authorized control-pack files, and deletion of the two tracked Python-runner files.
- Whole-repo live-reference closure found no nonhistorical references to the retired runner outside the deleted files themselves.
- `/home/spenser/.cache/substrate-runtime-refactor/b1-b2-1-joint-closeout/direct_shell_wall.py` was removed only after recording SHA-256 `a60038842c13276d86904da4e75d983414e341006ba716d7dd1d8e67a9847f3b`.
- The cache root and its preserved diagnostic directories remain in place after the exact-file removal.
- The control pack now states that the Make targets are the sole normative broad shell-wall entrypoints and that the Python runner is historical diagnostic evidence only.

Findings:

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Verdict: `CLEAN`.
