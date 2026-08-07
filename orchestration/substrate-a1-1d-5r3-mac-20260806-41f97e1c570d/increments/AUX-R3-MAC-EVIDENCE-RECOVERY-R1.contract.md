# AUX-R3-MAC-EVIDENCE-RECOVERY-R1 contract

## Selected outcome

Implement and remotely publish **R1 only** from the landed recovery plan at `d4bd729de8d344dfc567aaa62203838f68c2c895`: make
`scripts/substrate/dev-install-substrate.sh` read its canonical install-bootstrap context under
macOS `/bin/bash` 3.2 without allocating, overwriting, or closing a caller-owned file descriptor,
and prove that behavior with the isolated installer regression. Do not begin or partially implement
R2, R3, R4, R5, R6, host preparation, installation, or native evidence.

## Authoritative source truth

Read from the exact expected base before work:

- `AGENTS.md`;
- `.agents/skills/using-agent-skills/SKILL.md` and every skill it selects;
- `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/SPEC.md`;
- `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/PLAN.md`;
- `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/TASKS.md`;
- `llm-last-mile/runtime-refactor/03-phase-slice-map.md`;
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`;
- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`;
- `llm-last-mile/runtime-refactor/06-review-finding-inventory.md`;
- `llm-last-mile/runtime-refactor/review-control/README.md`;
- `llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py`.

Use the repository-local skills only. Load `using-agent-skills` first and apply at minimum
`source-driven-development`, `test-driven-development`, `incremental-implementation`,
`debugging-and-error-recovery`, `code-review-and-quality`, and `git-workflow-and-versioning`.
Record the loaded paths and how each constrained R1.

## Exact donor boundary

The preserved donor is read-only evidence:

- worktree: `/Users/spensermcconnell/.codex/worktrees/60cc/substrate`;
- donor base/tree: `d8a65fc8890dd37584aaeac2984c906188e5f06e` /
  `d8f25cc993f264f0c67fe3e00180eb393e41b2e3`;
- tracked binary-diff SHA-256:
  `39682fd4415a00ae4099580135a687d3e11858f62f84e9ef6ff3771bafd4b54c`;
- R1 untracked test SHA-256:
  `2de2da832cbb38c1722d2075ffcef781d7bb9d23a76318cfd6d3b54fee0b0664`.

Re-hash and inventory the donor before relying on it. Never mutate, stage, restore, checkout,
reset, clean, commit, archive, or execute tests in the donor. Manually recreate only the R1 lines
in the fresh task worktree. A matching path does not authorize any other donor line. If the donor
identity or hashes differ, return `BLOCKED_CONTRADICTION` without edits.

## Exact implementation boundary

### Completion claim

The installer reads its canonical bootstrap context under `/bin/bash` 3.2 without allocating or
closing a caller descriptor; context argv/environment remain canonical and no bootstrap temporary
file remains.

### Production allowlist

- `scripts/substrate/dev-install-substrate.sh`: only
  `resolve_install_bootstrap_context`, limited to the Bash-3.2-compatible descriptor lifecycle.

### Test allowlist

- `tests/installers/dev_install_bash32_fd_regression.sh`: only caller-owned FD survival,
  canonical context argv/environment, and no-bootstrap-temp assertions using a temporary fixture
  prefix.

### Review artifact allowlist

- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r1-review-cycle-record.json`;
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r1-review-authority-security.md`;
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r1-review-lifecycle-convergence.md`;
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r1-review-allowlist-evidence.md`.

The exact final changed-path set is the two implementation/test paths plus those four review
artifacts. `06-review-finding-inventory.md` may change only if a valid unfixed P3/P4 is discovered;
otherwise it must remain byte-identical. No other path may change.

### Frozen and prohibited surface

All `tests/mac/*`, lifecycle binaries, manifests, lockfiles, common pairing contracts, shell
lifecycle clients, Keychain/XPC/Lima code, R3 control documents, `.agents`, orchestration state,
Windows paths, ordinary Linux host paths, evidence artifacts, and every R2-R6 path/symbol are
frozen. No install, Keychain, XPC, Lima, publisher, code-signing, pairing, native evidence, or host
mutation may run. No lifecycle binary build/copy, selector, transport, principal, or pairing change
is authorized.

## Impact and implementation workflow

Before editing `resolve_install_bootstrap_context`, run the repository-required GitNexus upstream
impact analysis and record direct callers, affected processes, and risk. If GitNexus is degraded,
refresh only as allowed by the repository and supplement with manual shell caller analysis. Any
HIGH/CRITICAL result or required unlisted symbol/path is a closed `BLOCKED_SCOPE_EXPANSION` unless
it is solely the already-authorized R1 Bash descriptor surface and can be justified inside this
exact fence. Do not edit first and justify later.

Use test-first or regression-first development. One editing subagent may own the two allowed
implementation paths; all reviewers are fresh and read-only. Every subagent and reviewer must use
`gpt-5.6-terra` with Extra High (`xhigh`) reasoning under the explicit user override. No reviewer
may review work it authored.

## Required deterministic proof

Run and record at minimum:

1. `/bin/bash tests/installers/dev_install_bash32_fd_regression.sh` on macOS;
2. `/bin/bash -n scripts/substrate/dev-install-substrate.sh`;
3. `git diff --check`;
4. exact path and symbol allowlist verification;
5. focused changed-byte secret/private-data scan;
6. a manual diff proving no build/copy, Windows, Linux-runtime, lifecycle, or native surface changed;
7. `gitnexus_detect_changes()` before publication and manual review of any degraded/unknown result.

The regression may operate only inside its temporary fixture prefix and must restore/remove its
fixture. Do not perform a real install.

## Causal cascading review

Freeze the exact full implementation subject and fingerprint after deterministic checks. Run one
fresh discovery review (or same-fingerprint burst) using all three read-only lenses:

1. authority/security: descriptor ownership, argv/environment integrity, injection, secrets;
2. lifecycle/convergence: FD lifetime, cleanup, error/interrupt behavior, temporary-file absence;
3. allowlist/evidence: exact path/symbol fence, Bash 3.2 execution, proof sufficiency, platform
   non-expansion.

Classify against the selected R1 outcome. Remediate valid P1/P2 only in one consolidated pass.
Record valid P3/P4 in `06-review-finding-inventory.md` without creating remediation cycles. Validate
`--next-cycle closure`, then use a different fresh read-only reviewer for closure on the remediated
subject/delta. Permit at most two supplemental causal cycles only for P1/P2 directly caused or
unmasked by the immediately preceding remediation under unchanged authority and risk. `CLEAN` is
terminal. Unresolved P1/P2, unrelated findings, invalid review records, or exhausted budget return
`BLOCKED_REVIEW` or `BLOCKED_SCOPE_EXPANSION` and prohibit publication.

Validate the review-cycle JSON with
`python3 llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py <record>` after every
cycle and before publication. Preserve all pre-existing R3 review records byte-for-byte.

## Publication and terminal contract

Publication mode is `remote`. Immediately before publication, require the live remote target still
equals `d4bd729de8d344dfc567aaa62203838f68c2c895` and the exact tree is `3d5005d38c83b68c0a9813c10cbe6cb11996aed0`. If and only if every gate and terminal review are
CLEAN, create exactly one normal commit and fast-forward push it to `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`. Never merge,
rebase, reset, clean, cherry-pick, amend, rewrite, or force-push. Verify the live remote equals the
new commit, ahead/behind is 0/0, and the task worktree/index are clean.

Return `LANDED_CLEAN` with `next_increment` exactly `AUTHORITY_REQUIRED:AUX-R3-MAC-EVIDENCE-RECOVERY-R2`. A generated successor is not start
authority. On any block, preserve the task and worktree byte-for-byte, return the exact blocked
receipt with a complete handoff, and do not request archival. Do not dispatch R2 or any successor.
