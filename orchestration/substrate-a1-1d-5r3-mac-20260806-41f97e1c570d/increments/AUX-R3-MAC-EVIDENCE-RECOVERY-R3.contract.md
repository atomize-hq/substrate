# Increment contract — AUX-R3-MAC-EVIDENCE-RECOVERY-R3

## Objective

Implement only the trusted native-evidence project binding defined by the landed recovery plan.
The validator must accept an artifact only when its `product_project_id` equals a required,
dispatch-supplied expected project ID. The historical Linux artifact remains valid only when the
caller explicitly supplies its historical expected project ID.

## Exact source and successor

- Expected base: `fc150213826b8b423f042b4591d6b8b1cb3ace20`
- Expected tree: `02217ad2dbd106ae5befc60c8de862b6f3864647`
- Target: `origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- Publication: one normal fast-forward commit after terminal review `CLEAN`
- Sole successor: `AUX-R3-MAC-EVIDENCE-RECOVERY-R4`

## Exact path and symbol fence

Production:

- `scripts/ci/validate_r3_native_evidence.py`: only `validate_artifact`, `parse_args`, `main`, and
  the minimum direct argument plumbing necessary to make `--expected-product-project-id` required
  and compare it fail-closed with the artifact field.

Tests:

- `scripts/ci/test_validate_r3_native_evidence.py`: only exact valid, missing, mismatch, and
  historical-Linux-static-fixture cases for the caller-supplied expected project ID.

Append-only current-control documentation:

- `llm-last-mile/runtime-refactor/03-phase-slice-map.md`
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`

Append only the recovery-current authoritative invocation:

`validate_r3_native_evidence.py <artifact> --expected-evidence-id <id> --expected-source-commit <oid> --expected-source-tree <tree> --expected-source-ref <ref> --expected-product-project-id <dispatch-bound-project-id> --expected-gated-successor <value>`

The historical Linux check supplies `2ccb802f-301c-4af4-9bd5-51d22808f0a2`. Do not rewrite frozen
architecture, prior Linux evidence, or earlier review records.

Review metadata only:

- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r3-review-authority-security.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r3-review-lifecycle-convergence.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r3-review-allowlist-evidence.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r3-review-cycle-record.json`

No other path or symbol is authorized.

## Donor boundary

The preserved donor `/Users/spensermcconnell/.codex/worktrees/60cc/substrate` is read-only evidence.
Inspect it if useful, but manually recreate only the narrow R3 behavior in the fresh task worktree.
Never copy the donor diff wholesale, mutate it, reset it, clean it, check it out, commit it, or use
it as a publication base.

## Mandatory skills and analysis

Load `.agents/skills/using-agent-skills/SKILL.md` first, then invoke the relevant repository-local
skills including `orchestrate-top-level-tasks`, `source-driven-development`,
`test-driven-development`, `incremental-implementation`, `code-review-and-quality`,
`security-and-hardening`, and `git-workflow-and-versioning`. Before editing any existing symbol,
run GitNexus impact analysis when the existing index can answer it and supplement with exact manual
call-site/CLI analysis. Do not run `npx gitnexus analyze`; its tracked metadata injection is
forbidden. Treat degraded/UNKNOWN GitNexus output as inconclusive, not as zero impact.

Every subagent and reviewer must use `gpt-5.6-terra` at Extra High reasoning under the explicit
user override. Editing ownership must be disjoint where practical. Reviewers are fresh, read-only,
and independent of the implementation they review.

## TDD and deterministic proof

Prove the old fixed-project behavior fails the new caller-bound contract before the implementation
change, then make the narrow tests pass. Required proof includes:

1. `python3 -m unittest -v scripts.ci.test_validate_r3_native_evidence`.
2. Direct missing `--expected-product-project-id` rejection.
3. Direct mismatched expected project ID rejection.
4. Historical Linux artifact validation only with its explicitly supplied historical project ID.
5. A check that every recovery-current authoritative validator command includes the required flag.
6. Python syntax/compile, exact path/symbol allowlist, changed-byte secret scan, and `git diff --check`.

This is non-native validator work. Do not install, build platform targets, run lifecycle actions,
create official evidence artifacts, mutate Keychain/XPC/Lima/system state, or perform Linux/Windows
runtime work. The historical Linux fixture is read-only predecessor compatibility, not Linux work.

## Review and publication

Freeze the exact five-path product/test/documentation subject, excluding review metadata. Run the
repository-defined causal cascading review process: one discovery review or same-subject
authority/security, lifecycle/convergence, and allowlist/evidence burst; remediate valid P1/P2;
then one different fresh closure review. At most two supplemental causal cycles are allowed only
for a P1/P2 directly caused or unmasked by the immediately preceding remediation under unchanged
authority and risk. `CLEAN` ends the loop. Valid P3/P4 are recorded in
`06-review-finding-inventory.md` and do not create remediation/review cycles; because that inventory
path is outside this packet, stop for meta adjudication if an entry is required.

Validate the review-cycle record with
`llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py`. Run change detection with
the existing index if available and exact manual fallback otherwise. Before publication, require
the live target still equals the expected base, zero open P1/P2, exact nine-path final inventory,
and clean checks. Publish exactly one normal fast-forward commit. Never force-push, merge, rebase,
reset, or clean.

## Stops

Return `BASE_DRIFT` for remote/base/tree movement. Return `BLOCKED_CONTRADICTION` for identity,
donor, or exact-fence contradictions. Return `BLOCKED_SCOPE_EXPANSION` if implementation needs an
unlisted path/symbol, native action, new authority carrier, or Linux/Windows runtime work. Return
`BLOCKED_REVIEW` for unresolved P1/P2 or invalid review lineage. Preserve the task and worktree on
every stop; never request archival.

On success return `codex.top-level-task-receipt.v1` with `LANDED_CLEAN` and
`next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R4`. Sending the receipt to meta thread
`019fd8c3-b12c-7e73-919f-898600ab0f64` on host `local` must be the final tool action.
